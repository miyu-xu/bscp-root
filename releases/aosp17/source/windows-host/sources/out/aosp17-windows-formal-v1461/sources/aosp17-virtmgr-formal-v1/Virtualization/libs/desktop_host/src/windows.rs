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

//! Windows implementation of [`DesktopHost`](crate::traits::DesktopHost).
//!
//! Uses named pipes and actual per-request SID/token access checks; no SELinux emulation.

use crate::request_identity::{self, FileAccess};
use crate::traits::*;
use anyhow::Result;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, OwnedHandle, RawHandle};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING};
use windows_sys::Win32::System::Pipes::{SetNamedPipeHandleState, PIPE_READMODE_BYTE};

/// Windows desktop host with authenticated native dispatch authorization.
pub struct WindowsDesktopHost;
impl WindowsDesktopHost {
    pub fn new() -> Self { Self }

    fn named_pipe_path_for_vsock(cid: u32, port: u32) -> String {
        format!(r"\\.\pipe\binder_rpc_vsock_{cid}_{port}")
    }
}

impl PermissionProvider for WindowsDesktopHost {
    fn check_permission(&self, perm: &str) -> Result<()> {
        request_identity::check_owner()?;
        match perm {
            "android.permission.DUMP" | "android.permission.MANAGE_VIRTUAL_MACHINE" |
            "android.permission.USE_CUSTOM_VIRTUAL_MACHINE" => Ok(()),
            _ => anyhow::bail!("unsupported desktop permission"),
        }
    }
}

impl SelinuxProvider for WindowsDesktopHost {
    fn check_file_label(&self, file: &std::fs::File, name: &str) -> Result<()> {
        let _ = name; // Diagnostic label is not an authority source on Windows.
        request_identity::check_file(file, FileAccess::ReadImage)?;
        Ok(())
    }

    fn check_label_for_partition(&self, label: &str) -> Result<()> {
        anyhow::bail!("SELinux partition-label authorization is unavailable on Windows: {label}")
    }
}

impl StagedApexProvider for WindowsDesktopHost {
    fn get_staged_apex_module_names(&self) -> Result<Vec<String>> {
        // Staged APEX on Windows uses VIRTMGR_STAGED_APEX_DIR.
        // The actual APEX discovery is in virtmgr's payload.rs.
        Ok(Vec::new())
    }
}

impl VsockConnector for WindowsDesktopHost {
    fn connect(&self, cid: u32, port: u32) -> Result<binder::ParcelFileDescriptor> {
        const GENERIC_READ: u32 = 0x8000_0000u32;
        const GENERIC_WRITE: u32 = 0x4000_0000u32;

        let path = Self::named_pipe_path_for_vsock(cid, port);
        let wide: Vec<u16> = OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                0,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(anyhow::anyhow!(
                "Failed to open named pipe for vsock: {}",
                std::io::Error::last_os_error()
            ));
        }
        let owned_handle = unsafe { OwnedHandle::from_raw_handle(handle as RawHandle) };
        // Set pipe to byte read mode
        let mut mode = PIPE_READMODE_BYTE;
        let rc = unsafe { SetNamedPipeHandleState(handle, &mut mode, std::ptr::null_mut(), std::ptr::null_mut()) };
        if rc == 0 {
            return Err(anyhow::anyhow!(
                "Failed to set named pipe read mode: {}",
                std::io::Error::last_os_error()
            ));
        }
        let fd=binder::platform_fd::OwnedFd::try_from_handle(&owned_handle)?;
        Ok(binder::ParcelFileDescriptor::new(fd))
    }
}

impl DebugPolicySource for WindowsDesktopHost {
    fn debug_policy_json_path(&self) -> Result<Option<std::path::PathBuf>> {
        Ok(std::env::var("VIRTMGR_DEBUG_POLICY_JSON")
            .ok()
            .map(std::path::PathBuf::from))
    }
}

impl DesktopHost for WindowsDesktopHost {
    fn permission(&self) -> &dyn PermissionProvider {
        self
    }
    fn selinux(&self) -> &dyn SelinuxProvider {
        self
    }
    fn staged_apex(&self) -> &dyn StagedApexProvider {
        self
    }
    fn vsock(&self) -> &dyn VsockConnector {
        self
    }
    fn debug_policy(&self) -> &dyn DebugPolicySource {
        self
    }
    fn platform_name(&self) -> &'static str {
        "Windows (WHPX)"
    }
    fn num_cpus(&self) -> Option<usize> {
        std::thread::available_parallelism().ok().map(|n| n.get())
    }
    fn path_for_fd(&self, file: &std::fs::File) -> Result<std::path::PathBuf> {
        use std::os::windows::io::AsRawHandle;
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let h = file.as_raw_handle();
        let mut buf = vec![0u16; 32768];
        let len = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetFinalPathNameByHandleW(
                h as _,
                buf.as_mut_ptr(),
                buf.len() as u32,
                0,
            )
        };
        if len == 0 || len as usize >= buf.len() {
            return Err(anyhow::anyhow!(
                "GetFinalPathNameByHandleW failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        let os_str = OsString::from_wide(&buf[..len as usize]);
        Ok(std::path::PathBuf::from(os_str))
    }
}

#[cfg(test)] mod authenticated_permission_tests {
 use super::*;
 #[test] fn production_permissions_reject_without_dispatch_token() {
  let host=WindowsDesktopHost::new();
  for permission in ["android.permission.DUMP","android.permission.MANAGE_VIRTUAL_MACHINE","android.permission.USE_CUSTOM_VIRTUAL_MACHINE","unknown.permission"] {
   let error=host.check_permission(permission).unwrap_err();
   assert_eq!(error.downcast_ref::<std::io::Error>().and_then(|e|e.raw_os_error()),Some(1008));
  }
 }
}
