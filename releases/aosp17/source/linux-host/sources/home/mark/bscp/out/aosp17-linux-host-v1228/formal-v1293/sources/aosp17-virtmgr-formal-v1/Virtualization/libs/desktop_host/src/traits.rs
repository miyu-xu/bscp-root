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

//! Core trait definitions for the DesktopHost platform abstraction layer.
//!
//! Each sub-trait covers one platform capability domain. The composite
//! [`DesktopHost`] trait bundles them all so platform selection happens once.

use anyhow::Result;

/// Permission checking strategy.
///
/// On Android this delegates to `IPermissionController`. On desktop hosts
/// it uses a configurable mock/allowlist or bypass.
pub trait PermissionProvider: Send + Sync {
    /// Check whether the caller holds the given permission.
    fn check_permission(&self, perm: &str) -> Result<()>;
}

/// SELinux label checking strategy.
///
/// On Android this uses `getfilecon`. On desktop hosts it uses a configurable
/// mock allowlist or bypass.
pub trait SelinuxProvider: Send + Sync {
    /// Check the SELinux label of an open file.
    fn check_file_label(&self, file: &std::fs::File, name: &str) -> Result<()>;

    /// Check a partition label string directly.
    fn check_label_for_partition(&self, label: &str) -> Result<()>;
}

/// Staged APEX resolution strategy.
///
/// On Android this queries `IPackageManagerNative`. On desktop hosts it reads
/// a local JSON directory structure (VIRTMGR_STAGED_APEX_DIR).
pub trait StagedApexProvider: Send + Sync {
    /// Return the module names of currently staged APEXes.
    fn get_staged_apex_module_names(&self) -> Result<Vec<String>>;
}

/// VSOCK transport connector for guest communication.
///
/// Real AF_VSOCK on Linux/Android, UDS simulation on macOS,
/// named-pipe simulation on Windows.
pub trait VsockConnector: Send + Sync {
    /// Connect to a guest vsock port.
    fn connect(&self, cid: u32, port: u32) -> Result<binder::ParcelFileDescriptor>;
}

/// Debug policy source.
///
/// On Android this reads from the device tree overlay. On desktop hosts it
/// loads from a JSON file (VIRTMGR_DEBUG_POLICY_JSON).
pub trait DebugPolicySource: Send + Sync {
    /// Load the debug policy, returning the path to the JSON file if set.
    fn debug_policy_json_path(&self) -> Result<Option<std::path::PathBuf>>;
}

/// Composite platform abstraction — one impl per host OS.
///
/// All sub-providers share the same lifecycle and are created together.
pub trait DesktopHost: Send + Sync {
    /// Permission checking provider.
    fn permission(&self) -> &dyn PermissionProvider;

    /// SELinux label checking provider.
    fn selinux(&self) -> &dyn SelinuxProvider;

    /// Staged APEX resolution provider.
    fn staged_apex(&self) -> &dyn StagedApexProvider;

    /// VSOCK transport connector.
    fn vsock(&self) -> &dyn VsockConnector;

    /// Debug policy source.
    fn debug_policy(&self) -> &dyn DebugPolicySource;

    /// Human-readable platform name for diagnostics.
    fn platform_name(&self) -> &'static str;

    /// Number of host CPUs (for telemetry).
    fn num_cpus(&self) -> Option<usize>;

    /// Path to an open file descriptor (for composite disk images).
    /// On Linux: /proc/self/fd/{fd}
    /// On macOS: fcntl(F_GETPATH) or /dev/fd/{fd}
    /// On Windows: GetFinalPathNameByHandleW
    fn path_for_fd(&self, file: &std::fs::File) -> Result<std::path::PathBuf>;
}
