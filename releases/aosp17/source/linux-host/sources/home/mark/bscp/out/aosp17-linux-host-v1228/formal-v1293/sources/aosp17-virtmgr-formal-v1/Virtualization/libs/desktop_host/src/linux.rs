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

//! Linux / Android implementation of [`DesktopHost`](crate::traits::DesktopHost).

use crate::mock_permission::MockPermissionProvider;
use crate::mock_selinux::MockSelinuxProvider;
use crate::traits::*;
use crate::unix_common;
use anyhow::{Context, Result};
use log::warn;
use std::sync::Once;

/// Linux desktop host with mock permission/SELinux and real AF_VSOCK.
pub struct LinuxDesktopHost {
    permission: MockPermissionProvider,
    selinux: MockSelinuxProvider,
}

impl LinuxDesktopHost {
    pub fn new() -> Self {
        LinuxDesktopHost {
            permission: MockPermissionProvider::from_env(),
            selinux: MockSelinuxProvider::from_env(),
        }
    }
}

impl PermissionProvider for LinuxDesktopHost {
    fn check_permission(&self, perm: &str) -> Result<()> {
        self.permission.check_permission(perm)
    }
}

impl SelinuxProvider for LinuxDesktopHost {
    fn check_file_label(&self, file: &std::fs::File, name: &str) -> Result<()> {
        self.selinux.check_file_label(file, name)
    }

    fn check_label_for_partition(&self, label: &str) -> Result<()> {
        self.selinux.check_label_for_partition(label)
    }
}

impl StagedApexProvider for LinuxDesktopHost {
    fn get_staged_apex_module_names(&self) -> Result<Vec<String>> {
        // On Linux, staged APEX requires VIRTMGR_STAGED_APEX_DIR env.
        // Return empty and let virtmgr's payload.rs handle the discovery.
        Ok(Vec::new())
    }
}

impl VsockConnector for LinuxDesktopHost {
    fn connect(&self, cid: u32, port: u32) -> Result<binder::ParcelFileDescriptor> {
        let stream = vsock::VsockStream::connect_with_cid_port(cid, port)
            .context("Failed to connect vsock stream")?;
        let file = unsafe { std::fs::File::from_raw_fd(stream.into_raw_fd()) };
        Ok(binder::ParcelFileDescriptor::new(file))
    }
}

#[cfg(unix)]
use std::os::unix::io::{FromRawFd, IntoRawFd};

impl DebugPolicySource for LinuxDesktopHost {
    fn debug_policy_json_path(&self) -> Result<Option<std::path::PathBuf>> {
        Ok(std::env::var("VIRTMGR_DEBUG_POLICY_JSON")
            .ok()
            .map(std::path::PathBuf::from))
    }
}

impl DesktopHost for LinuxDesktopHost {
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
        "Linux (KVM)"
    }
    fn num_cpus(&self) -> Option<usize> {
        let ret = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) };
        if ret > 0 { ret.try_into().ok() } else { None }
    }
    fn path_for_fd(&self, file: &std::fs::File) -> Result<std::path::PathBuf> {
        unix_common::path_for_fd_unix(file)
    }
}

/// Android-specific desktop host (real SELinux, real permission service).
#[cfg(target_os = "android")]
pub mod android {
    use super::*;
    use crate::traits::*;

    pub struct AndroidDesktopHost;

    impl PermissionProvider for AndroidDesktopHost {
        fn check_permission(&self, _perm: &str) -> Result<()> {
            // On real Android this goes through IPermissionController.
            // The calling code in aidl.rs handles the full logic including
            // root bypass and early VM checks. This is a placeholder that
            // defers to the existing call site logic.
            Ok(())
        }
    }

    impl SelinuxProvider for AndroidDesktopHost {
        fn check_file_label(&self, _file: &std::fs::File, _name: &str) -> Result<()> {
            // Deferred to existing getfilecon logic in aidl.rs.
            Ok(())
        }

        fn check_label_for_partition(&self, _label: &str) -> Result<()> {
            Ok(())
        }
    }

    impl StagedApexProvider for AndroidDesktopHost {
        fn get_staged_apex_module_names(&self) -> Result<Vec<String>> {
            // Deferred to existing IPackageManagerNative logic.
            Ok(Vec::new())
        }
    }

    impl VsockConnector for AndroidDesktopHost {
        fn connect(&self, cid: u32, port: u32) -> Result<binder::ParcelFileDescriptor> {
            let stream = vsock::VsockStream::connect_with_cid_port(cid, port)
                .context("Failed to connect vsock stream")?;
            let file = unsafe { std::fs::File::from_raw_fd(stream.into_raw_fd()) };
            Ok(binder::ParcelFileDescriptor::new(file))
        }
    }

    impl DebugPolicySource for AndroidDesktopHost {
        fn debug_policy_json_path(&self) -> Result<Option<std::path::PathBuf>> {
            Ok(None)
        }
    }

    impl DesktopHost for AndroidDesktopHost {
        fn permission(&self) -> &dyn PermissionProvider { self }
        fn selinux(&self) -> &dyn SelinuxProvider { self }
        fn staged_apex(&self) -> &dyn StagedApexProvider { self }
        fn vsock(&self) -> &dyn VsockConnector { self }
        fn debug_policy(&self) -> &dyn DebugPolicySource { self }
        fn platform_name(&self) -> &'static str { "Android" }
        fn num_cpus(&self) -> Option<usize> {
            let ret = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) };
            if ret > 0 { ret.try_into().ok() } else { None }
        }
        fn path_for_fd(&self, file: &std::fs::File) -> Result<std::path::PathBuf> {
            unix_common::path_for_fd_unix(file)
        }
    }
}
