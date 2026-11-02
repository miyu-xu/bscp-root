// Copyright 2026 The Android Open Source Project
// SPDX-License-Identifier: Apache-2.0
//! Blocking byte I/O over an owned FILE_FLAG_OVERLAPPED named-pipe HANDLE.
//! Cancellation requests target this operation only, and completion is drained
//! before its buffer, event, OVERLAPPED or pipe can be released. OS completion
//! has no hard time bound; a deadline requests cancellation, not forced release.
#![cfg(windows)]

use std::{
    io::{self, Read, Write},
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    sync::Arc,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{
        GetHandleInformation, ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_NOT_FOUND, ERROR_NO_DATA,
        ERROR_OPERATION_ABORTED, ERROR_PIPE_NOT_CONNECTED, HANDLE, HANDLE_FLAG_INHERIT,
        WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    Storage::FileSystem::{GetFileType, ReadFile, WriteFile, FILE_TYPE_PIPE},
    System::{
        Pipes::{GetNamedPipeHandleStateW, PIPE_NOWAIT, PIPE_READMODE_MESSAGE},
        Threading::{
            CreateEventW, SetEvent, WaitForMultipleObjects, WaitForSingleObject, INFINITE,
        },
        IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED},
    },
};

fn event() -> io::Result<OwnedHandle> {
    // Unnamed manual-reset event, non-inheritable default security.
    let raw = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
    if raw == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
}
fn raw(handle: &OwnedHandle) -> HANDLE {
    handle.as_raw_handle() as HANDLE
}

/// Sticky cancellation shared by both directions. It does not own the pipe.
#[derive(Clone)]
pub struct Cancellation(Arc<OwnedHandle>);
impl Cancellation {
    pub fn cancel(&self) -> io::Result<()> {
        if unsafe { SetEvent(raw(&self.0)) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    fn check(&self) -> io::Result<()> {
        match unsafe { WaitForSingleObject(raw(&self.0), 0) } {
            WAIT_TIMEOUT => Ok(()),
            WAIT_OBJECT_0 => Err(io::Error::from_raw_os_error(ERROR_OPERATION_ABORTED as i32)),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

pub struct OverlappedPipe {
    pipe: OwnedHandle,
    cancellation: Cancellation,
    timeout: Option<Duration>,
}
impl OverlappedPipe {
    /// Consume an original/duplicated, non-inheritable byte-mode named-pipe HANDLE.
    ///
    /// # Safety
    /// The handle must have been opened with FILE_FLAG_OVERLAPPED, and no other
    /// owner may close it or change its pipe mode. Win32 cannot validate the
    /// overlapped creation flag here. Duplicating such a handle preserves it.
    /// The pipe must already be connected: a successful client CreateFileW, or
    /// a completed server ConnectNamedPipe, must precede adoption. Otherwise
    /// ERROR_PIPE_NOT_CONNECTED cannot be interpreted as peer-disconnect EOF.
    pub unsafe fn from_owned_handle(pipe: OwnedHandle) -> io::Result<Self> {
        let mut flags = 0;
        let mut mode = 0;
        if GetHandleInformation(raw(&pipe), &mut flags) == 0 {
            return Err(io::Error::last_os_error());
        }
        if flags & HANDLE_FLAG_INHERIT != 0 || GetFileType(raw(&pipe)) != FILE_TYPE_PIPE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected non-inheritable pipe",
            ));
        }
        if GetNamedPipeHandleStateW(
            raw(&pipe),
            &mut mode,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        if mode & (PIPE_READMODE_MESSAGE | PIPE_NOWAIT) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected blocking byte-mode pipe",
            ));
        }
        Ok(Self {
            pipe,
            cancellation: Cancellation(Arc::new(event()?)),
            timeout: None,
        })
    }
    /// Duplicate ownership, sharing cancellation; permits simultaneous read/write.
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            pipe: self.pipe.try_clone()?,
            cancellation: self.cancellation.clone(),
            timeout: self.timeout,
        })
    }
    pub fn cancellation(&self) -> Cancellation {
        self.cancellation.clone()
    }
    /// Per-operation wait budget. Timeout cancels and drains the precise request.
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    fn transfer(&mut self, buffer: *mut u8, len: usize, write: bool) -> io::Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        self.cancellation.check()?;
        let done = event()?;
        let mut operation: OVERLAPPED = unsafe { std::mem::zeroed() };
        operation.hEvent = raw(&done);
        let requested = len.min(16 * 1024 * 1024) as u32;
        let mut bytes = 0;
        // Every reference and ownership stays live through completion; no user
        // callback or fallible Rust operation occurs while a request is pending.
        let immediate = unsafe {
            if write {
                WriteFile(
                    raw(&self.pipe),
                    buffer.cast(),
                    requested,
                    &mut bytes,
                    &mut operation,
                )
            } else {
                ReadFile(
                    raw(&self.pipe),
                    buffer.cast(),
                    requested,
                    &mut bytes,
                    &mut operation,
                )
            }
        };
        let mut error = if immediate == 0 {
            io::Error::last_os_error().raw_os_error().unwrap() as u32
        } else {
            0
        };
        if error == ERROR_IO_PENDING {
            let handles = [raw(&done), raw(&self.cancellation.0)];
            let millis = self
                .timeout
                .map(|d| {
                    d.as_millis()
                        .saturating_add(u128::from(d.subsec_nanos() % 1_000_000 != 0))
                        .min((INFINITE - 1) as u128) as u32
                })
                .unwrap_or(INFINITE);
            let wait = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, millis) };
            let interrupted = if wait == WAIT_OBJECT_0 {
                None
            } else if wait == WAIT_OBJECT_0 + 1 {
                Some(io::Error::from_raw_os_error(ERROR_OPERATION_ABORTED as i32))
            } else if wait == WAIT_TIMEOUT {
                Some(io::Error::from(io::ErrorKind::TimedOut))
            } else {
                Some(io::Error::last_os_error())
            };
            let mut interrupted = interrupted;
            if interrupted.is_some() && unsafe { CancelIoEx(raw(&self.pipe), &operation) } == 0 {
                let e = io::Error::last_os_error();
                // NOT_FOUND is a completion race, never proof of completion.
                if e.raw_os_error() != Some(ERROR_NOT_FOUND as i32) {
                    interrupted = Some(e);
                }
            }
            // TRUE waits for completion even after CancelIoEx. No deadline is
            // claimed for this drain. Owned handles cannot be closed by a peer.
            error = if unsafe { GetOverlappedResult(raw(&self.pipe), &operation, &mut bytes, 1) }
                == 0
            {
                io::Error::last_os_error().raw_os_error().unwrap() as u32
            } else {
                0
            };
            // Preserve a successful raced transfer: reporting timeout after bytes
            // were consumed would silently discard data / encourage duplicate writes.
            if error != 0 {
                if let Some(e) = interrupted {
                    return Err(e);
                }
            }
        }
        if error != 0 {
            if !write
                && [ERROR_BROKEN_PIPE, ERROR_PIPE_NOT_CONNECTED, ERROR_NO_DATA].contains(&error)
            {
                return Ok(0);
            }
            return Err(io::Error::from_raw_os_error(error as i32));
        }
        if bytes > requested {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid pipe completion length",
            ));
        }
        Ok(bytes as usize)
    }
}
impl Read for OverlappedPipe {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.transfer(buffer.as_mut_ptr(), buffer.len(), false)
    }
}
impl Write for OverlappedPipe {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.transfer(buffer.as_ptr() as *mut u8, buffer.len(), true)
    }
    // No userspace buffering. FlushFileBuffers would wait for peer consumption.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
