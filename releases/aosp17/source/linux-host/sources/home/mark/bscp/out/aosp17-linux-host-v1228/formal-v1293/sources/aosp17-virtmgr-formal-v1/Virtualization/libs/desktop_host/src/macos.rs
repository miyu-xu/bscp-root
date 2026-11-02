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

//! macOS implementation of [`DesktopHost`](crate::traits::DesktopHost).
//!
//! Uses UDS-based vsock emulation and mock permission/SELinux.

use crate::mock_permission::MockPermissionProvider;
use crate::mock_selinux::MockSelinuxProvider;
use crate::traits::*;
use crate::unix_common;
use anyhow::{Context, Result};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::os::unix::net::UnixStream;

/// macOS desktop host with mock permission/SELinux and UDS vsock simulation.
pub struct MacOSDesktopHost {
    permission: MockPermissionProvider,
    selinux: MockSelinuxProvider,
}

impl MacOSDesktopHost {
    pub fn new() -> Self {
        MacOSDesktopHost {
            permission: MockPermissionProvider::from_env(),
            selinux: MockSelinuxProvider::from_env(),
        }
    }

    fn uds_path_for_vsock(cid: u32, port: u32) -> String {
        format!("/tmp/binder_rpc_vsock_{cid}_{port}.sock")
    }
}

impl PermissionProvider for MacOSDesktopHost {
    fn check_permission(&self, perm: &str) -> Result<()> {
        self.permission.check_permission(perm)
    }
}

impl SelinuxProvider for MacOSDesktopHost {
    fn check_file_label(&self, file: &std::fs::File, name: &str) -> Result<()> {
        self.selinux.check_file_label(file, name)
    }

    fn check_label_for_partition(&self, label: &str) -> Result<()> {
        self.selinux.check_label_for_partition(label)
    }
}

impl StagedApexProvider for MacOSDesktopHost {
    fn get_staged_apex_module_names(&self) -> Result<Vec<String>> {
        // Staged APEX on macOS uses VIRTMGR_STAGED_APEX_DIR, same as Linux.
        // The actual APEX discovery is in virtmgr's payload.rs.
        Ok(Vec::new())
    }
}

impl VsockConnector for MacOSDesktopHost {
    fn connect(&self, cid: u32, port: u32) -> Result<binder::ParcelFileDescriptor> {
        let path = Self::uds_path_for_vsock(cid, port);
        let stream = UnixStream::connect(&path)
            .with_context(|| format!("Failed to connect UDS vsock at {path}"))?;
        let file = unsafe { std::fs::File::from_raw_fd(stream.into_raw_fd()) };
        Ok(binder::ParcelFileDescriptor::new(file))
    }
}

impl DebugPolicySource for MacOSDesktopHost {
    fn debug_policy_json_path(&self) -> Result<Option<std::path::PathBuf>> {
        Ok(std::env::var("VIRTMGR_DEBUG_POLICY_JSON")
            .ok()
            .map(std::path::PathBuf::from))
    }
}

impl DesktopHost for MacOSDesktopHost {
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
        "macOS (HVF)"
    }
    fn num_cpus(&self) -> Option<usize> {
        let ret = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) };
        if ret > 0 { ret.try_into().ok() } else { None }
    }
    fn path_for_fd(&self, file: &std::fs::File) -> Result<std::path::PathBuf> {
        unix_common::path_for_fd_unix(file)
    }
}
