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
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{ERROR_IO_PENDING, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
#[cfg(windows)]
use windows_sys::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::CreateEventW;

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

#[cfg(windows)]
fn new_overlapped_event() -> io::Result<OwnedHandle> {
    // SAFETY: default security, manual reset, initially nonsignaled, and no name are valid event
    // parameters. Ownership of the returned handle moves into OwnedHandle.
    let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
    if event == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: CreateEventW returned a unique owned HANDLE.
    Ok(unsafe { OwnedHandle::from_raw_handle(event as RawHandle) })
}

#[cfg(windows)]
fn finish_overlapped(
    pipe: RawHandle,
    event: &OwnedHandle,
    operation: impl FnOnce(*mut OVERLAPPED) -> i32,
) -> io::Result<u32> {
    // A distinct event/OVERLAPPED pair per direction allows ReadFile and WriteFile to remain
    // outstanding concurrently on the same full-duplex named-pipe instance.
    let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
    overlapped.hEvent = event.as_raw_handle() as HANDLE;
    let immediate = operation(&mut overlapped);
    if immediate == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_IO_PENDING as i32) {
            return Err(error);
        }
    }
    let mut transferred = 0_u32;
    // SAFETY: pipe, OVERLAPPED and its event stay live until the operation completes.
    if unsafe {
        GetOverlappedResult(
            pipe as HANDLE,
            &mut overlapped,
            &mut transferred,
            1,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(transferred)
}

#[cfg(windows)]
fn pipe_read(pipe: RawHandle, event: &OwnedHandle, buffer: &mut [u8]) -> io::Result<usize> {
    let requested = u32::try_from(buffer.len()).expect("bridge buffer length fits in u32");
    finish_overlapped(pipe, event, |overlapped| {
        // SAFETY: buffer is writable for requested bytes and remains live through completion.
        unsafe {
            ReadFile(
                pipe as HANDLE,
                buffer.as_mut_ptr().cast(),
                requested,
                std::ptr::null_mut(),
                overlapped,
            )
        }
    })
    .map(|count| count as usize)
}

#[cfg(windows)]
fn pipe_write(pipe: RawHandle, event: &OwnedHandle, buffer: &[u8]) -> io::Result<()> {
    let requested = u32::try_from(buffer.len()).expect("bridge buffer length fits in u32");
    let transferred = finish_overlapped(pipe, event, |overlapped| {
        // SAFETY: buffer is readable for requested bytes and remains live through completion.
        unsafe {
            WriteFile(
                pipe as HANDLE,
                buffer.as_ptr().cast(),
                requested,
                std::ptr::null_mut(),
                overlapped,
            )
        }
    })?;
    if transferred != requested {
        return Err(io::Error::new(
            io::ErrorKind::WriteZero,
            format!("named-pipe bridge wrote {transferred} of {requested} bytes"),
        ));
    }
    Ok(())
}

/// Bridge a single TCP connection to a guest vsock file using two blocking
/// copy pumps. Blocks until both directions finish.
pub fn bridge_connection(tcp: TcpStream, guest: File) -> io::Result<()> {
    #[cfg(windows)]
    {
        let guest = Arc::new(guest);
        let reader_guest = Arc::clone(&guest);
        let writer_guest = Arc::clone(&guest);
        let mut tcp_reader = tcp.try_clone()?;
        let mut tcp_writer = tcp;
        let host_to_guest = thread::spawn(move || -> io::Result<u64> {
            let event = new_overlapped_event()?;
            let mut buffer = [0_u8; 64 * 1024];
            let mut total = 0_u64;
            loop {
                let count = tcp_reader.read(&mut buffer)?;
                if count == 0 {
                    let _ = tcp_reader.shutdown(std::net::Shutdown::Both);
                    return Ok(total);
                }
                if let Err(error) =
                    pipe_write(writer_guest.as_raw_handle(), &event, &buffer[..count])
                {
                    // Closing the TCP transport wakes the opposite pump and tells the ADB server
                    // to establish a fresh transport after Guest adbd restarts.
                    let _ = tcp_reader.shutdown(std::net::Shutdown::Both);
                    return Err(error);
                }
                total += count as u64;
            }
        });
        let guest_to_host = thread::spawn(move || -> io::Result<u64> {
            let event = new_overlapped_event()?;
            let mut buffer = [0_u8; 64 * 1024];
            let mut total = 0_u64;
            loop {
                let count = match pipe_read(reader_guest.as_raw_handle(), &event, &mut buffer) {
                    Ok(count) => count,
                    Err(error) => {
                        let _ = tcp_writer.shutdown(std::net::Shutdown::Both);
                        return Err(error);
                    }
                };
                if count == 0 {
                    tcp_writer.shutdown(std::net::Shutdown::Both)?;
                    return Ok(total);
                }
                if let Err(error) = tcp_writer.write_all(&buffer[..count]) {
                    let _ = tcp_writer.shutdown(std::net::Shutdown::Both);
                    return Err(error);
                }
                total += count as u64;
            }
        });

        host_to_guest
            .join()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "host_to_guest bridge thread panicked"))??;
        guest_to_host
            .join()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "guest_to_host bridge thread panicked"))??;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
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
}
