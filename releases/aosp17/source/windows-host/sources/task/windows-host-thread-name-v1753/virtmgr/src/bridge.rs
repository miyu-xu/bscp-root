// Copyright 2022, The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unified TCP-to-vsock bridge for ADB and other host-guest port forwarding.
//!
//! Bridges TCP connections received on a host port to a guest vsock port.
//! The vsock connection is provided by the caller, so this module contains
//! no platform-dependent code — it is used by both `crosvm_unix.rs` and
//! `crosvm_windows.rs`.

use anyhow::Context;
use log::error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use crate::windows_vsock_io::{Cancellation, OverlappedPipe};

/// Per-port bridge handle. Dropping or calling [`stop`](BridgeHandle::stop)
/// signals the listener loop to terminate.
pub struct BridgeHandle {
    _running: Arc<AtomicBool>,
}

/// Start a TCP listener on `127.0.0.1:<host_port>`.
///
/// For each accepted TCP connection, `on_accept` is called with the stream.
/// The listener loop runs until [`BridgeHandle::stop`] is called.
pub fn start_bridge<F>(host_port: u16, cid: u32, guest_port: u32, on_accept: F) -> io::Result<BridgeHandle>
where
    F: Fn(TcpStream) + Send + 'static,
{
    let listener = TcpListener::bind(("127.0.0.1", host_port))?;
    listener.set_nonblocking(true)?;

    let running = Arc::new(AtomicBool::new(true));
    let flag = Arc::clone(&running);

    thread::Builder::new()
        .name(format!("bridge-{host_port}"))
        .spawn(move || {
            while flag.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((tcp, remote)) => {
                        let _ = remote;
                        on_accept(tcp);
                    }
                    Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(err) => {
                        error!("[bridge cid={cid}] 127.0.0.1:{host_port} <-> vsock:{guest_port} — listener error: {err}");
                        break;
                    }
                }
            }
        })
        .map(|_| BridgeHandle { _running: running })
        .with_context(|| format!("failed to spawn bridge thread for localhost:{host_port}"))
        .map_err(|e: anyhow::Error| io::Error::new(io::ErrorKind::Other, e))
}

impl BridgeHandle {
    /// Stop the bridge listener. Already-established connections continue
    /// until they complete or the guest disconnects.
    pub fn stop(&self) {
        self._running.store(false, Ordering::Release);
    }
}

// std::io::copy/write_all retry Interrupted, including Win32 cancellation
// ERROR_OPERATION_ABORTED (995). Sticky cancellation must instead terminate.
#[cfg(windows)]
fn copy_pipe_pump(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 { return Ok(()); }
        let mut offset = 0;
        while offset < count {
            let written = writer.write(&buffer[offset..count])?;
            if written == 0 { return Err(io::ErrorKind::WriteZero.into()); }
            offset += written;
        }
    }
}

#[cfg(windows)]
fn copy_tcp_pump(reader: &mut TcpStream, writer: &mut impl Write, stopped: &AtomicBool) -> io::Result<()> {
    let mut buffer = [0u8; 64 * 1024];
    loop {
        if stopped.load(Ordering::Acquire) { return Ok(()); }
        let count = match reader.read(&mut buffer) {
            Ok(count) => count,
            Err(e) if matches!(e.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock) => continue,
            Err(e) => return Err(e),
        };
        if count == 0 { return Ok(()); }
        let mut offset = 0;
        while offset < count {
            let written = writer.write(&buffer[offset..count])?;
            if written == 0 { return Err(io::ErrorKind::WriteZero.into()); }
            offset += written;
        }
    }
}

#[cfg(windows)]
struct PumpShutdown {
    tcp: TcpStream,
    cancellation: Cancellation,
    stopped: Arc<AtomicBool>,
}
#[cfg(windows)]
impl PumpShutdown {
    fn finish(&self) -> io::Result<()> {
        self.stopped.store(true, Ordering::Release);
        let cancel = self.cancellation.cancel();
        // Always wake the TCP reader, even if signalling cancellation failed.
        let shutdown = self.tcp.shutdown(std::net::Shutdown::Both);
        cancel?;
        match shutdown {
            Err(e) if e.kind() == io::ErrorKind::NotConnected => Ok(()),
            result => result,
        }
    }
}
#[cfg(windows)]
impl Drop for PumpShutdown {
    fn drop(&mut self) { let _ = self.finish(); }
}

/// Both pumps are joined on every started path. Cancellation is an error, not
/// proof that an outstanding business request completed successfully.
#[cfg(windows)]
pub fn bridge_connection(tcp: TcpStream, guest: OverlappedPipe) -> io::Result<()> {
    // Winsock shutdown from a duplicate does not reliably wake a synchronous
    // recv on all providers. Poll only the read wait; this is not an idle timeout.
    tcp.set_read_timeout(Some(Duration::from_millis(200)))?;
    let stopped = Arc::new(AtomicBool::new(false));
    let control = PumpShutdown { tcp: tcp.try_clone()?, cancellation: guest.cancellation(), stopped: stopped.clone() };
    let outgoing = PumpShutdown { tcp: tcp.try_clone()?, cancellation: guest.cancellation(), stopped: stopped.clone() };
    let incoming = PumpShutdown { tcp: tcp.try_clone()?, cancellation: guest.cancellation(), stopped: stopped.clone() };
    let mut tcp_reader = tcp.try_clone()?;
    let mut tcp_writer = tcp;
    let mut guest_writer = guest.try_clone()?;
    let mut guest_reader = guest;
    let host_to_guest = thread::Builder::new().name("bridge-host-to-guest".into()).spawn(move || {
        // Guard also runs during panic, waking the opposite pump before join.
        let guard = outgoing;
        let copied = copy_tcp_pump(&mut tcp_reader, &mut guest_writer, &guard.stopped);
        let closed = guard.finish();
        copied.and_then(|_| closed)
    })?;
    let guest_to_host = match thread::Builder::new().name("bridge-guest-to-host".into()).spawn(move || {
        let guard = incoming;
        let copied = copy_pipe_pump(&mut guest_reader, &mut tcp_writer);
        let closed = guard.finish();
        copied.and_then(|_| closed)
    }) {
        Ok(thread) => thread,
        Err(error) => {
            let _ = control.finish();
            let _ = host_to_guest.join();
            return Err(error);
        }
    };
    // Do not use `?` until both joins have completed.
    let outgoing = host_to_guest.join();
    let incoming = guest_to_host.join();
    let closed = control.finish();
    outgoing.map_err(|_| io::Error::new(io::ErrorKind::Other, "host_to_guest bridge thread panicked"))??;
    incoming.map_err(|_| io::Error::new(io::ErrorKind::Other, "guest_to_host bridge thread panicked"))??;
    closed
}

#[cfg(not(windows))]
pub fn bridge_connection(tcp: TcpStream, guest: File) -> io::Result<()> {
    let mut guest_reader = guest.try_clone()?;
    let mut guest_writer = guest;
    let mut tcp_reader = tcp.try_clone()?;
    let mut tcp_writer = tcp;

    let host_to_guest = thread::spawn(move || -> io::Result<u64> {
        use std::os::unix::io::AsRawFd;
        let copied = io::copy(&mut tcp_reader, &mut guest_writer);
        // The opposite pump holds a duplicated FD. Dropping this writer alone does not
        // deliver EOF to the Guest, so explicitly propagate the TCP write half-close.
        unsafe { libc::shutdown(guest_writer.as_raw_fd(), libc::SHUT_WR); }
        copied
    });
    let guest_to_host = thread::spawn(move || -> io::Result<u64> {
        let copied = io::copy(&mut guest_reader, &mut tcp_writer)?;
        let _ = tcp_writer.shutdown(std::net::Shutdown::Write)?;
        Ok(copied)
    });

    let _ = host_to_guest
        .join()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "host_to_guest bridge thread panicked"))?
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let _ = guest_to_host
        .join()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "guest_to_host bridge thread panicked"))?
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}
