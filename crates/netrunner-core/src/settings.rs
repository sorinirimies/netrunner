//! User settings, persisted as human-editable JSON.
//!
//! Settings are small and rarely written, so JSON is a better fit than the
//! embedded database used for [`crate::history`]. The file lives next to the
//! history database at `<config-dir>/netrunner/settings.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::types::{DetailLevel, TestConfig};

/// Persistent user preferences shared across runs.
///
/// `#[serde(default)]` makes the on-disk format forward/backward compatible:
/// unknown fields are ignored and missing fields fall back to [`Default`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// Preferred test server URL.
    pub server_url: String,
    /// Test payload size in megabytes.
    pub test_size_mb: u64,
    /// Per-test timeout in seconds.
    pub timeout_seconds: u64,
    /// Output detail level.
    pub detail_level: DetailLevel,
    /// Whether animations/live charts are enabled.
    pub animations: bool,
    /// Automatically run a test when the app launches.
    pub auto_run: bool,
    /// How many past runs to show in history views.
    pub max_history: usize,
}

impl Default for Settings {
    fn default() -> Self {
        let cfg = TestConfig::default();
        Self {
            server_url: cfg.server_url,
            test_size_mb: cfg.test_size_mb,
            timeout_seconds: cfg.timeout_seconds,
            detail_level: cfg.detail_level,
            animations: cfg.animation_enabled,
            auto_run: true,
            max_history: 20,
        }
    }
}

impl Settings {
    /// Path to the settings file, creating the parent directory if needed.
    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = dirs::config_dir()
            .ok_or("Failed to find config directory")?
            .join("netrunner");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("settings.json"))
    }

    /// Load settings from disk, falling back to defaults on any error
    /// (missing file, partial JSON, parse failure).
    pub fn load() -> Self {
        Self::config_path()
            .ok()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist settings to disk as pretty-printed JSON.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path()?;
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Build a [`TestConfig`] from these settings.
    pub fn to_config(&self) -> TestConfig {
        TestConfig {
            server_url: self.server_url.clone(),
            test_size_mb: self.test_size_mb,
            timeout_seconds: self.timeout_seconds,
            json_output: false,
            animation_enabled: self.animations,
            detail_level: self.detail_level,
            max_servers: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_test_config() {
        let s = Settings::default();
        let c = s.to_config();
        assert_eq!(c.server_url, TestConfig::default().server_url);
        assert_eq!(c.test_size_mb, 10);
        assert!(s.auto_run);
        assert_eq!(s.max_history, 20);
    }

    #[test]
    fn json_roundtrip() {
        let s = Settings {
            timeout_seconds: 42,
            auto_run: false,
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn partial_json_uses_defaults() {
        // Only one field present — the rest must fall back to defaults.
        let back: Settings = serde_json::from_str(r#"{ "timeout_seconds": 99 }"#).unwrap();
        assert_eq!(back.timeout_seconds, 99);
        assert_eq!(back.test_size_mb, Settings::default().test_size_mb);
        assert!(back.auto_run);
    }
}
