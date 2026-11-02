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

//! # Desktop Host Platform Abstraction
//!
//! Provides a trait-based platform abstraction layer for the AVF desktop host
//! runtime (virtmgr). Each supported host OS provides a concrete implementation
//! of [`DesktopHost`], consolidating platform-specific code for:
//!
//! - **Permission checking** ([`PermissionProvider`])
//! - **SELinux label checking** ([`SelinuxProvider`])
//! - **Staged APEX resolution** ([`StagedApexProvider`])
//! - **VSOCK transport** ([`VsockConnector`])
//! - **Debug policy loading** ([`DebugPolicySource`])
//!
//! ## Architecture
//!
//! ```text
//! virtmgr
//!   └── desktop_host::DesktopHost  (composite trait)
//!         ├── permission()  → &dyn PermissionProvider
//!         ├── selinux()     → &dyn SelinuxProvider
//!         ├── staged_apex() → &dyn StagedApexProvider
//!         ├── vsock()       → &dyn VsockConnector
//!         ├── debug_policy()→ &dyn DebugPolicySource
//!         ├── platform_name()
//!         ├── num_cpus()
//!         └── path_for_fd()
//! ```
//!
//! Platform selection happens at compile time via `#[cfg]` in
//! [`platform_selector`]. Use [`create_desktop_host()`] to construct the
//! correct implementation for the current build target.

mod mock_permission;
mod mock_selinux;
mod platform_selector;
pub mod traits;

#[cfg(unix)]
mod unix_common;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(windows)]
mod windows;

// Convenience re-exports
pub use platform_selector::{ConcreteDesktopHost, create_desktop_host};
pub use traits::{DesktopHost, PermissionProvider, SelinuxProvider, StagedApexProvider,
                 VsockConnector, DebugPolicySource};

#[cfg(unix)]
pub use unix_common::unix_fd;
