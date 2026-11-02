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

//! Compile-time platform selection. Exposes the concrete [`DesktopHost`]
//! implementation as a struct type alias.

/// The concrete `DesktopHost` implementation for the current platform.
///
/// Use [`crate::create_desktop_host()`] to construct an instance.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub type ConcreteDesktopHost = crate::linux::LinuxDesktopHost;

#[cfg(target_os = "macos")]
pub type ConcreteDesktopHost = crate::macos::MacOSDesktopHost;

#[cfg(windows)]
pub type ConcreteDesktopHost = crate::windows::WindowsDesktopHost;

/// Create a new platform-specific DesktopHost instance.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn create_desktop_host() -> ConcreteDesktopHost {
    crate::linux::LinuxDesktopHost::new()
}

#[cfg(target_os = "macos")]
pub fn create_desktop_host() -> ConcreteDesktopHost {
    crate::macos::MacOSDesktopHost::new()
}

#[cfg(windows)]
pub fn create_desktop_host() -> ConcreteDesktopHost {
    crate::windows::WindowsDesktopHost::new()
}
