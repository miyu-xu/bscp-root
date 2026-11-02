// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod config;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod fork;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod helpers;

pub use crate::config::JailConfig;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::fork::fork_process;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::helpers::*;

// TODO(b/268407006): We define Minijail as an empty struct as a stub for minijail::Minijail on
// Windows because the concept of jailing is baked into a bunch of places where it isn't easy to
// compile it out. In the long term, this should go away.
#[cfg(windows)]
pub struct FakeMinijailStub {}

// macOS has no minijail implementation. No value of this type can be constructed.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
pub enum MacUnsupportedJail {}

// Absence of a jail is explicit. A requested jail must never become a successful no-op.
#[cfg(target_os = "macos")]
pub fn simple_jail(config: Option<&JailConfig>, _policy: &str) -> anyhow::Result<Option<MacUnsupportedJail>> {
    anyhow::ensure!(config.is_none(), "sandbox/minijail is unsupported on macOS");
    Ok(None)
}
#[cfg(all(test, target_os = "macos"))]
mod mac_jail_tests {
    use super::*;
    #[test]
    fn absence_is_explicit_and_request_is_rejected() {
        assert!(simple_jail(None, "test").unwrap().is_none());
        assert!(simple_jail(Some(&JailConfig::default()), "test").is_err());
    }
}
