// Copyright 2026, The Android Open Source Project
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

//! Maps guest (CID, port) vsock connects to a transport:
//! - **Linux / Android**: AF_VSOCK via the `vsock` crate.
//! - **Windows (host)**: a named pipe client, using the same naming scheme as
//!   `platform/namedpipe_vsock.h` (`NamedPipeVsockAddress`): `\\.\pipe\binder_rpc_vsock_{cid}_{port}`.
//! - **macOS (host)**: crosvm host-connect UDS at `/tmp/bscp_crosvm_vsock_{cid}.sock`, followed by
//!   the versioned `HDVS` handshake selecting the guest port.

use binder::ParcelFileDescriptor;

#[cfg(any(target_os = "linux", target_os = "android"))]
use std::fs::File;
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::os::unix::io::{FromRawFd, IntoRawFd};
#[cfg(any(target_os = "linux", target_os = "android"))]
use vsock::VsockStream;

#[cfg(target_os = "macos")]
use std::fs::File;
#[cfg(target_os = "macos")]
use std::io::Write as _;
#[cfg(target_os = "macos")]
use std::os::unix::io::{FromRawFd, IntoRawFd};
#[cfg(target_os = "macos")]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::io::{FromRawHandle, OwnedHandle, RawHandle};
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING,
};
#[cfg(windows)]
use windows_sys::Win32::System::Pipes::{SetNamedPipeHandleState, PIPE_READMODE_BYTE};

/// Same pattern as `android::NamedPipeVsockAddress` in `platform/namedpipe_vsock.h`.
pub fn named_pipe_path_for_vsock(cid: u32, port: u32) -> String {
    format!(r"\\.\pipe\binder_rpc_vsock_{cid}_{port}")
}

/// Host-initiated UDS exposed by crosvm's macOS virtio-vsock device.
#[cfg(target_os = "macos")]
pub fn uds_path_for_vsock_emulation(cid: u32, _port: u32) -> String {
    format!("/tmp/bscp_crosvm_vsock_{cid}.sock")
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn connect(cid: u32, port: u32) -> std::io::Result<VsockStream> {
    VsockStream::connect_with_cid_port(cid, port)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn into_parcel_file_descriptor(stream: VsockStream) -> ParcelFileDescriptor {
    let f = unsafe { File::from_raw_fd(stream.into_raw_fd()) };
    ParcelFileDescriptor::new(f)
}

#[cfg(target_os = "macos")]
pub fn connect(cid: u32, port: u32) -> std::io::Result<UnixStream> {
    connect_path(std::path::Path::new(&uds_path_for_vsock_emulation(cid, port)), port)
}

#[cfg(target_os = "macos")]
fn connect_path(path: &std::path::Path, port: u32) -> std::io::Result<UnixStream> {
    const HOST_CONNECT_MAGIC: &[u8; 4] = b"HDVS";
    const HOST_CONNECT_VERSION: u32 = 1;
    let mut stream = UnixStream::connect(path)?;
    let mut header = [0_u8; 12];
    header[..4].copy_from_slice(HOST_CONNECT_MAGIC);
    header[4..8].copy_from_slice(&HOST_CONNECT_VERSION.to_be_bytes());
    header[8..12].copy_from_slice(&port.to_be_bytes());
    stream.write_all(&header)?;
    Ok(stream)
}

#[cfg(target_os = "macos")]
pub fn into_parcel_file_descriptor(stream: UnixStream) -> ParcelFileDescriptor {
    let f = unsafe { File::from_raw_fd(stream.into_raw_fd()) };
    ParcelFileDescriptor::new(f)
}

#[cfg(windows)]
pub fn connect(cid: u32, port: u32) -> std::io::Result<OwnedHandle> {
    let path = named_pipe_path_for_vsock(cid, port);
    let wide: Vec<u16> = OsStr::new(&path).encode_wide().chain(std::iter::once(0)).collect();

    const GENERIC_READ: u32 = 0x8000_0000u32;
    const GENERIC_WRITE: u32 = 0x4000_0000u32;

    // SAFETY: Win32 API; `wide` is NUL-terminated. `CreateFileW` transfers ownership on success.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
            0,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let e = std::io::Error::last_os_error();
        return Err(e);
    }

    let mut mode = PIPE_READMODE_BYTE;
    let ok =
        unsafe { SetNamedPipeHandleState(handle, &mut mode, ptr::null_mut(), ptr::null_mut()) };
    if ok == 0 {
        let e = std::io::Error::last_os_error();
        unsafe {
            std::mem::drop(OwnedHandle::from_raw_handle(handle as RawHandle));
        }
        return Err(e);
    }

    // SAFETY: `handle` is a valid pipe handle from CreateFileW.
    unsafe { Ok(OwnedHandle::from_raw_handle(handle as RawHandle)) }
}

#[cfg(windows)]
pub fn into_parcel_file_descriptor(handle: OwnedHandle) -> ParcelFileDescriptor {
    ParcelFileDescriptor::new(handle)
}


#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;
    #[test]
    fn actual_connect_handshake_payload_and_descriptor_ownership() {
        let root=std::env::temp_dir().join(format!("a17-vsock-{}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path=root.join("s");
        let listener=UnixListener::bind(&path).unwrap();
        let stream=connect_path(&path,0x12345678).unwrap();
        let (mut peer,_)=listener.accept().unwrap();
        peer.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
        let mut header=[0;12]; peer.read_exact(&mut header).unwrap();
        assert_eq!(&header,b"HDVS\0\0\0\x01\x12\x34\x56\x78");
        let pfd=into_parcel_file_descriptor(stream);
        let mut file=std::fs::File::from(pfd.as_ref().try_clone().unwrap());
        file.write_all(b"payload").unwrap();
        let mut bytes=[0;7]; peer.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes,b"payload");
        drop(file); drop(pfd);
        assert_eq!(peer.read(&mut bytes).unwrap(),0);
        drop(peer); drop(listener);
        std::fs::remove_file(&path).unwrap(); std::fs::remove_dir(&root).unwrap();
    }
    #[test]
    fn missing_endpoint_is_error() {
        let path=std::env::temp_dir().join(format!("a17-absent-{}",std::process::id())).join("s");
        assert!(!path.exists());
        assert_eq!(connect_path(&path,1).unwrap_err().kind(),std::io::ErrorKind::NotFound);
    }
}
