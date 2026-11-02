// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::os::fd::IntoRawFd;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixDatagram;
use std::time::{Duration, Instant};

use crate::errno::errno_result;
use crate::errno::Error;
use crate::errno::Result;
use crate::event::EventWaitResult;
use crate::sys::unix::RawDescriptor;
use crate::AsRawDescriptor;
use crate::FromRawDescriptor;
use crate::IntoRawDescriptor;
use crate::SafeDescriptor;

/// A pollable event that can be transferred with SCM_RIGHTS on macOS.
///
/// A kqueue descriptor cannot be transferred over an AF_UNIX socket on macOS.
/// Use one connected-to-self Unix datagram socket so crosvm can transfer
/// virtio queue and MSI events between its control threads.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlatformEvent {
    socket: SafeDescriptor,
}

impl PlatformEvent {
    pub fn new() -> Result<PlatformEvent> {
        // Unix datagrams enqueue locally before send returns. UDP loopback may
        // deliver after reset drains the queue, violating Event::reset.
        // The private name exists only while connecting; the connected socket
        // remains valid after unlink and still uses one transferable descriptor.
        let dir = std::path::PathBuf::from("/tmp")
            .join(format!("crosvm-event-{}", uuid::Uuid::new_v4()));
        std::fs::DirBuilder::new().mode(0o700).create(&dir)?;
        struct NameGuard { path: std::path::PathBuf, armed: bool }
        impl Drop for NameGuard {
            fn drop(&mut self) {
                if self.armed {
                    let _ = std::fs::remove_file(self.path.join("s"));
                    let _ = std::fs::remove_dir(&self.path);
                }
            }
        }
        let mut guard = NameGuard { path: dir, armed: true };
        let name = guard.path.join("s");
        let datagram = UnixDatagram::bind(&name)?;
        datagram.connect(&name)?;
        std::fs::remove_file(&name)?;
        std::fs::remove_dir(&guard.path)?;
        guard.armed = false;
        // SAFETY: ownership of the fresh socket is transferred exactly once.
        let socket = unsafe { SafeDescriptor::from_raw_descriptor(datagram.into_raw_fd()) };
        Ok(PlatformEvent { socket })
    }

    pub fn signal(&self) -> Result<()> {
        let byte = [1u8];
        loop {
            // SAFETY: byte is readable and the descriptor is valid.
            let result = unsafe {
                libc::send(
                    self.as_raw_descriptor(),
                    byte.as_ptr() as *const libc::c_void,
                    byte.len(),
                    libc::MSG_DONTWAIT,
                )
            };
            if result >= 0 {
                return Ok(());
            }
            let error = Error::last();
            match error.errno() {
                libc::EINTR => continue,
                libc::EAGAIN | libc::ENOBUFS => {
                    // Coalesce only if the original queue is demonstrably readable.
                    let mut fd = libc::pollfd { fd: self.as_raw_descriptor(), events: libc::POLLIN, revents: 0 };
                    // SAFETY: valid descriptor and one writable pollfd.
                    let ready = unsafe { libc::poll(&mut fd, 1, 0) };
                    if ready > 0 && fd.revents & libc::POLLIN != 0 { return Ok(()); }
                    return Err(error);
                },
                _ => return Err(error),
            }
        }
    }

    fn drain(&self) -> Result<()> {
        // Darwin FIONREAD snapshots so_rcv.sb_cc, including queued record
        // overhead. It bounds the bytes/messages present at this instant.
        // New producers must not keep reset in an endless drain-to-EAGAIN loop.
        let mut remaining: libc::c_uint = 0;
        // SAFETY: the descriptor is live and remaining is a writable u32.
        if unsafe { libc::ioctl(self.as_raw_descriptor(), libc::FIONREAD, &mut remaining) } < 0 {
            return errno_result();
        }
        let mut bytes = [0u8; 64];
        while remaining > 0 {
            // SAFETY: valid descriptor and writable bounded buffer.
            let result = unsafe { libc::recv(self.as_raw_descriptor(),
                bytes.as_mut_ptr() as *mut libc::c_void, bytes.len(), libc::MSG_DONTWAIT) };
            if result > 0 {
                remaining = remaining.saturating_sub(result as u32);
            } else if result == 0 {
                // Event writers send one byte; do not spin on invalid records.
                return Err(Error::new(libc::EIO));
            } else {
                let error = Error::last();
                return if error.errno() == libc::EAGAIN { Ok(()) } else { Err(error) };
            }
        }
        Ok(())
    }

    fn wait_deadline(&self, deadline: Option<Instant>) -> Result<EventWaitResult> {
        let mut poll_fd = libc::pollfd {
            fd: self.as_raw_descriptor(),
            events: libc::POLLIN,
            revents: 0,
        };
        loop {
            // EINTR retries use the remaining budget, never a fresh timeout.
            let remaining = deadline.map_or(-1, |end| {
                let left = end.saturating_duration_since(Instant::now());
                left.as_nanos().div_ceil(1_000_000).min(i32::MAX as u128) as i32
            });
            // SAFETY: poll_fd is a valid one-element pollfd array.
            let result = unsafe { libc::poll(&mut poll_fd, 1, remaining) };
            if result > 0 {
                if poll_fd.revents & libc::POLLIN == 0 {
                    return Err(Error::new(if poll_fd.revents & libc::POLLNVAL != 0 { libc::EBADF } else { libc::EIO }));
                }
                self.drain()?;
                return Ok(EventWaitResult::Signaled);
            }
            if result == 0 {
                if deadline.is_some_and(|end| Instant::now() >= end) {
                    return Ok(EventWaitResult::TimedOut);
                }
                // Very long waits are divided into poll's signed-ms chunks.
                continue;
            }
            let error = Error::last();
            if error.errno() != libc::EINTR {
                return Err(error);
            }
            if deadline.is_some_and(|end| Instant::now() >= end) {
                return Ok(EventWaitResult::TimedOut);
            }
        }
    }

    pub fn wait(&self) -> Result<()> {
        self.wait_deadline(None).map(|_| ())
    }

    pub fn wait_timeout(&self, timeout: Duration) -> Result<EventWaitResult> {
        let deadline = Instant::now().checked_add(timeout)
            .ok_or_else(|| Error::new(libc::EINVAL))?;
        self.wait_deadline(Some(deadline))
    }

    pub fn reset(&self) -> Result<()> {
        self.drain()
    }

    pub fn try_clone(&self) -> Result<PlatformEvent> {
        self.socket
            .try_clone()
            .map(|socket| PlatformEvent { socket })
    }
}

impl AsRawDescriptor for PlatformEvent {
    fn as_raw_descriptor(&self) -> RawDescriptor {
        self.socket.as_raw_descriptor()
    }
}

impl FromRawDescriptor for PlatformEvent {
    unsafe fn from_raw_descriptor(descriptor: RawDescriptor) -> Self {
        PlatformEvent {
            socket: SafeDescriptor::from_raw_descriptor(descriptor),
        }
    }
}

impl IntoRawDescriptor for PlatformEvent {
    fn into_raw_descriptor(self) -> RawDescriptor {
        self.socket.into_raw_descriptor()
    }
}

impl From<PlatformEvent> for SafeDescriptor {
    fn from(evt: PlatformEvent) -> Self {
        evt.socket
    }
}

impl From<SafeDescriptor> for PlatformEvent {
    fn from(socket: SafeDescriptor) -> Self {
        PlatformEvent { socket }
    }
}
