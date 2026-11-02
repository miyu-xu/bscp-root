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

//! Shared mock SELinux provider for desktop host builds.
//!
//! Supports three modes matching [`MockPermissionProvider`]:
//! - **bypass** (default): all labels accepted, logged once
//! - **allowlist**: labels checked against a configurable list
//! - **strict**: `VIRTMGR_STRICT_PARITY=1` raises an error
//!
//! Configuration is read from environment variables or JSON files:
//!
//! | Priority | Source | Format |
//! |----------|--------|--------|
//! | 1 (highest) | `VIRTMGR_MOCK_SELINUX_JSON` | JSON file path |
//! | 2 | `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST` | Comma-separated CSV |
//! | 3 | `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST_FILE` | One label per line |
//! | 4 (default) | No env set | Bypass mode (warn once) |

use crate::traits::SelinuxProvider;
use anyhow::{anyhow, bail, Result};
use log::warn;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Once;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MockSelinuxMode {
    #[default]
    Bypass,
    Allowlist,
    Strict,
}

/// JSON config file format for `VIRTMGR_MOCK_SELINUX_JSON`.
#[derive(Debug, Default, Deserialize)]
struct SelinuxJsonConfig {
    #[serde(default)]
    mode: MockSelinuxMode,
    #[serde(default)]
    allowlist: Vec<String>,
}

/// A SELinux provider that delegates to environment-configured mock logic.
pub struct MockSelinuxProvider {
    allowlist: Option<HashSet<String>>,
    strict: bool,
}

impl MockSelinuxProvider {
    /// Build from the standard environment variables, checking in priority order:
    ///
    /// 1. `VIRTMGR_MOCK_SELINUX_JSON` — JSON file with `{ mode, allowlist }`
    /// 2. `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST` — comma-separated CSV
    /// 3. `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST_FILE` — file with one label per line
    /// 4. `VIRTMGR_STRICT_PARITY=1` — fail instead of bypass (applies to all sources)
    pub fn from_env() -> Self {
        // JSON config has highest priority
        if let Ok(json_path) = std::env::var("VIRTMGR_MOCK_SELINUX_JSON") {
            if let Ok(config) = Self::parse_json_config(&json_path) {
                return Self::from_json_config(config);
            }
            warn!("Failed to parse VIRTMGR_MOCK_SELINUX_JSON={json_path}, falling back");
        }

        let allowlist = Self::parse_allowlist_from_env_or_file(
            "VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST",
            "VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST_FILE",
        );
        let strict = std::env::var("VIRTMGR_STRICT_PARITY")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        MockSelinuxProvider { allowlist, strict }
    }

    fn parse_json_config(path: &str) -> Result<SelinuxJsonConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: SelinuxJsonConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn from_json_config(config: SelinuxJsonConfig) -> Self {
        let allowlist = if !config.allowlist.is_empty() {
            Some(config.allowlist.into_iter().collect())
        } else {
            None
        };
        let strict = matches!(config.mode, MockSelinuxMode::Strict);
        if matches!(config.mode, MockSelinuxMode::Bypass) || (allowlist.is_none() && !strict) {
            MockSelinuxProvider { allowlist: None, strict: false }
        } else {
            MockSelinuxProvider { allowlist, strict }
        }
    }

    /// Build with explicit configuration (useful for tests).
    pub fn new(allowlist: Option<HashSet<String>>, strict: bool) -> Self {
        MockSelinuxProvider { allowlist, strict }
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

impl SelinuxProvider for MockSelinuxProvider {
    fn check_file_label(&self, _file: &std::fs::File, name: &str) -> Result<()> {
        if let Some(ref allowlist) = self.allowlist {
            if allowlist.contains(name) {
                return Ok(());
            }
            bail!("mock SELinux provider denied file label for {name}");
        }
        if self.strict {
            bail!("VIRTMGR_STRICT_PARITY=1: SELinux label checks require Android SELinux runtime");
        }
        static WARN_ONCE: Once = Once::new();
        WARN_ONCE.call_once(|| {
            warn!("Host runtime mode: SELinux label checks are bypassed");
        });
        let _ = _file;
        Ok(())
    }

    fn check_label_for_partition(&self, label: &str) -> Result<()> {
        if let Some(ref allowlist) = self.allowlist {
            if allowlist.contains(label) {
                return Ok(());
            }
            bail!("mock SELinux provider denied partition label {label}");
        }
        if self.strict {
            bail!("VIRTMGR_STRICT_PARITY=1: SELinux label checks require Android SELinux runtime");
        }
        static WARN_ONCE: Once = Once::new();
        WARN_ONCE.call_once(|| {
            warn!("Host runtime mode: SELinux label checks are bypassed");
        });
        let _ = label;
        Ok(())
    }
}
