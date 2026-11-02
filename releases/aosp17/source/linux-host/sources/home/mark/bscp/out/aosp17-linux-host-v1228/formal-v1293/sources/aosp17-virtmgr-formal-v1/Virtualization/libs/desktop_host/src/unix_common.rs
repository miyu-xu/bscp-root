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

//! Shared helpers for Unix platform implementations.

use anyhow::{Context, Result};

/// Resolve a file path from an fd on Unix.
///
/// On Linux/Android: reads `/proc/self/fd/{fd}`.
/// On macOS: uses `fcntl(F_GETPATH)` with `/dev/fd/{fd}` fallback.
#[cfg(target_os = "macos")]
pub fn path_for_fd_unix(file: &std::fs::File) -> Result<std::path::PathBuf> {
    use std::ffi::CStr;
    let fd = crate::unix_fd(file);
    let mut path_buf = [0u8; libc::PATH_MAX as usize];
    let rc = unsafe { libc::fcntl(fd, libc::F_GETPATH, path_buf.as_mut_ptr()) };
    if rc != -1 {
        if let Ok(s) = unsafe { CStr::from_ptr(path_buf.as_ptr().cast()) }.to_str() {
            return Ok(std::path::PathBuf::from(s));
        }
    }
    Ok(std::path::PathBuf::from(format!("/dev/fd/{}", fd)))
}

/// Resolve a file path from an fd on Linux/Android.
#[cfg(not(target_os = "macos"))]
pub fn path_for_fd_unix(file: &std::fs::File) -> Result<std::path::PathBuf> {
    let fd = crate::unix_fd(file);
    Ok(std::path::PathBuf::from(format!("/proc/self/fd/{}", fd)))
}

/// Get the raw fd from a File on Unix.
#[cfg(unix)]
pub fn unix_fd(file: &std::fs::File) -> i32 {
    use std::os::unix::io::AsRawFd;
    file.as_raw_fd()
}
