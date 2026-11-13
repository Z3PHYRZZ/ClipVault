//! Persistent user settings (serde JSON), stored under the local app-data dir.
use crate::contracts;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("could not determine app data directory")]
    NoDataDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: ThemeMode,
    pub max_entries: usize,
    pub max_age_days: u32,
    pub pause_monitoring: bool,
    pub clear_on_exit: bool,
    pub start_minimized: bool,
    pub launch_at_startup: bool,
    pub page_size: usize,
    pub show_hotkey: String,
    pub ignored_apps: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Light,
            max_entries: contracts::DEFAULT_MAX_ENTRIES,
            max_age_days: contracts::DEFAULT_MAX_AGE_DAYS,
            pause_monitoring: false,
            clear_on_exit: false,
            start_minimized: false,
            launch_at_startup: false,
            page_size: contracts::DEFAULT_PAGE_SIZE,
            show_hotkey: "Ctrl+Shift+V".to_string(),
            ignored_apps: Vec::new(),
        }
    }
}

/// `%LOCALAPPDATA%\ClipVault` (or a cwd-relative fallback).
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ClipVault")
}

pub fn db_path() -> PathBuf {
    data_dir().join(contracts::DB_FILE)
}

pub fn settings_path() -> PathBuf {
    data_dir().join(contracts::SETTINGS_FILE)
}

pub fn assets_dir() -> PathBuf {
    data_dir().join(contracts::ASSETS_DIR)
}

pub fn log_path() -> PathBuf {
    data_dir().join(contracts::LOG_FILE)
}

impl Settings {
    /// Load settings, tolerating a missing/corrupt file by returning defaults.
    pub fn load() -> Self {
        let path = settings_path();
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persist settings atomically (write temp then rename) to avoid truncation.
    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = data_dir();
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self)?;
        let tmp = dir.join("settings.json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, settings_path())?;
        Ok(())
    }

    pub fn data_dir() -> PathBuf {
        data_dir()
    }
    pub fn db_path() -> PathBuf {
        db_path()
    }
    pub fn assets_dir() -> PathBuf {
        assets_dir()
    }
    pub fn settings_path() -> PathBuf {
        settings_path()
    }
}
