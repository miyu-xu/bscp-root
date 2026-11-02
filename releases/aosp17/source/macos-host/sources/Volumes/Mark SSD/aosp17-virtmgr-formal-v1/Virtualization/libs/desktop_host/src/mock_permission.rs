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

//! Shared mock permission provider for desktop host builds.
//!
//! Supports three modes:
//! - **bypass** (default): all permissions granted, logged once
//! - **allowlist**: permissions checked against a configurable list
//! - **strict**: `VIRTMGR_STRICT_PARITY=1` raises an error
//!
//! Configuration is read from environment variables or JSON files:
//!
//! | Priority | Source | Format |
//! |----------|--------|--------|
//! | 1 (highest) | `VIRTMGR_MOCK_PERMISSION_JSON` | JSON file path |
//! | 2 | `VIRTMGR_MOCK_PERMISSION_ALLOWLIST` | Comma-separated CSV |
//! | 3 | `VIRTMGR_MOCK_PERMISSION_ALLOWLIST_FILE` | One permission per line |
//! | 4 (default) | No env set | Bypass mode (warn once) |

use crate::traits::PermissionProvider;
use anyhow::{anyhow, Result};
use log::warn;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Once;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MockPermissionMode {
    #[default]
    Bypass,
    Allowlist,
    Strict,
}

/// JSON config file format for `VIRTMGR_MOCK_PERMISSION_JSON`.
#[derive(Debug, Default, Deserialize)]
struct PermissionJsonConfig {
    #[serde(default)]
    mode: MockPermissionMode,
    #[serde(default)]
    allowlist: Vec<String>,
}

/// A permission provider that delegates to environment-configured mock logic.
///
/// Mode is selected at construction time:
/// - `Some(allowlist)` — check against the list
/// - `None` — bypass (warn once)
pub struct MockPermissionProvider {
    allowlist: Option<HashSet<String>>,
    strict: bool,
}

impl MockPermissionProvider {
    /// Build a provider from the standard env vars, checking in priority order:
    ///
    /// 1. `VIRTMGR_MOCK_PERMISSION_JSON` — JSON file with `{ mode, allowlist }`
    /// 2. `VIRTMGR_MOCK_PERMISSION_ALLOWLIST` — comma-separated CSV
    /// 3. `VIRTMGR_MOCK_PERMISSION_ALLOWLIST_FILE` — file with one permission per line
    /// 4. `VIRTMGR_STRICT_PARITY=1` — fail instead of bypass (applies to all sources)
    pub fn from_env() -> Self {
        // JSON config has highest priority
        if let Ok(json_path) = std::env::var("VIRTMGR_MOCK_PERMISSION_JSON") {
            if let Ok(config) = Self::parse_json_config(&json_path) {
                return Self::from_json_config(config);
            }
            warn!("Failed to parse VIRTMGR_MOCK_PERMISSION_JSON={json_path}, falling back");
        }

        // CSV env var or file
        let allowlist = Self::parse_allowlist_from_env_or_file(
            "VIRTMGR_MOCK_PERMISSION_ALLOWLIST",
            "VIRTMGR_MOCK_PERMISSION_ALLOWLIST_FILE",
        );
        let strict = std::env::var("VIRTMGR_STRICT_PARITY")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        MockPermissionProvider { allowlist, strict }
    }

    fn parse_json_config(path: &str) -> Result<PermissionJsonConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: PermissionJsonConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn from_json_config(config: PermissionJsonConfig) -> Self {
        let allowlist = if !config.allowlist.is_empty() {
            Some(config.allowlist.into_iter().collect())
        } else {
            None
        };
        let strict = matches!(config.mode, MockPermissionMode::Strict);
        if matches!(config.mode, MockPermissionMode::Bypass) || (allowlist.is_none() && !strict) {
            MockPermissionProvider { allowlist: None, strict: false }
        } else {
            MockPermissionProvider { allowlist, strict }
        }
    }

    /// Build with explicit configuration (useful for tests).
    pub fn new(allowlist: Option<HashSet<String>>, strict: bool) -> Self {
        MockPermissionProvider { allowlist, strict }
    }

    fn parse_allowlist_from_env_or_file(
        env_csv_key: &str,
        env_file_key: &str,
    ) -> Option<HashSet<String>> {
        if let Ok(csv) = std::env::var(env_csv_key) {
            let parsed: HashSet<String> = csv
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            return Some(parsed);
        }
        if let Ok(path) = std::env::var(env_file_key) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let parsed: HashSet<String> = content
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty() && !s.starts_with('#'))
                    .map(ToOwned::to_owned)
                    .collect();
                return Some(parsed);
            }
        }
        None
    }
}

impl PermissionProvider for MockPermissionProvider {
    fn check_permission(&self, perm: &str) -> Result<()> {
        if let Some(ref allowlist) = self.allowlist {
            if allowlist.contains(perm) {
                return Ok(());
            }
            return Err(anyhow!("mock permission provider denied: {perm}"));
        }
        if self.strict {
            return Err(anyhow!(
                "VIRTMGR_STRICT_PARITY=1: permission checks require Android permission service"
            ));
        }
        static WARN_ONCE: Once = Once::new();
        WARN_ONCE.call_once(|| {
            warn!("Host runtime mode: permission checks are bypassed (Android permission service unavailable)");
        });
        let _ = perm;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bypass_default() {
        let provider = MockPermissionProvider::new(None, false);
        assert!(provider.check_permission("android.permission.MANAGE_VIRTUALIZATION").is_ok());
    }

    #[test]
    fn test_allowlist_granted() {
        let mut set = HashSet::new();
        set.insert("test.perm".into());
        let provider = MockPermissionProvider::new(Some(set), false);
        assert!(provider.check_permission("test.perm").is_ok());
    }

    #[test]
    fn test_allowlist_denied() {
        let set = HashSet::new();
        let provider = MockPermissionProvider::new(Some(set), false);
        assert!(provider.check_permission("test.perm").is_err());
    }

    #[test]
    fn test_strict() {
        let provider = MockPermissionProvider::new(None, true);
        assert!(provider.check_permission("test.perm").is_err());
    }

    #[test]
    fn test_json_bypass_mode() {
        let config = PermissionJsonConfig {
            mode: MockPermissionMode::Bypass,
            allowlist: vec![],
        };
        let provider = MockPermissionProvider::from_json_config(config);
        assert!(provider.allowlist.is_none());
        assert!(!provider.strict);
        assert!(provider.check_permission("anything").is_ok());
    }

    #[test]
    fn test_json_allowlist_mode() {
        let config = PermissionJsonConfig {
            mode: MockPermissionMode::Allowlist,
            allowlist: vec!["allowed.perm".into()],
        };
        let provider = MockPermissionProvider::from_json_config(config);
        assert!(provider.check_permission("allowed.perm").is_ok());
        assert!(provider.check_permission("denied.perm").is_err());
    }

    #[test]
    fn test_json_strict_mode() {
        let config = PermissionJsonConfig {
            mode: MockPermissionMode::Strict,
            allowlist: vec![],
        };
        let provider = MockPermissionProvider::from_json_config(config);
        assert!(provider.strict);
        assert!(provider.check_permission("anything").is_err());
    }
}
