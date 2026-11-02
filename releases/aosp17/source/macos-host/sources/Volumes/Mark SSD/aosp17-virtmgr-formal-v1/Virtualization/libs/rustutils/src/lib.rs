//! Minimal rustutils for portable virtmgr (host/port).

pub mod inherited_fd;

pub mod system_properties {
    use anyhow::Result;
    use std::env;

    fn normalize_key(key: &str) -> String {
        key.chars()
            .map(|ch| match ch {
                'a'..='z' => ch.to_ascii_uppercase(),
                'A'..='Z' | '0'..='9' | '_' => ch,
                _ => '_',
            })
            .collect()
    }

    pub fn read(key: &str) -> Result<Option<String>> {
        let normalized = normalize_key(key);
        for env_key in [
            format!("ANDROID_PROP_{normalized}"),
            format!("VIRTMGR_PROP_{normalized}"),
        ] {
            if let Ok(value) = env::var(&env_key) {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    pub fn read_bool(key: &str, default: bool) -> Result<bool> {
        let Some(value) = read(key)? else {
            return Ok(default);
        };

        let value = value.trim();
        if value.is_empty() {
            return Ok(default);
        }

        match value {
            "1" => Ok(true),
            "0" => Ok(false),
            _ if value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("y")
                || value.eq_ignore_ascii_case("yes")
                || value.eq_ignore_ascii_case("on") => Ok(true),
            _ if value.eq_ignore_ascii_case("false")
                || value.eq_ignore_ascii_case("n")
                || value.eq_ignore_ascii_case("no")
                || value.eq_ignore_ascii_case("off") => Ok(false),
            _ => anyhow::bail!("Invalid boolean property {key}={value}"),
        }
    }
}
