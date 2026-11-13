//! Configuration module.
pub mod settings;

pub use settings::{
    ConfigError, Settings, ThemeMode, assets_dir, data_dir, db_path, log_path, settings_path,
};
