//! Shared constants. Integrator-owned. Fleet reads, never writes.
use iced::Color;

/// All brand colors are declared as `const` using the const `Color::from_rgb`.
/// (`from_rgb8` is not const in iced 0.13; RGB tuples stay here as the source of truth.)
pub const SAGE_RGB: (u8, u8, u8) = (0xD8, 0xE2, 0xDC);
pub const PEACH_RGB: (u8, u8, u8) = (0xFF, 0xE5, 0xD9);
pub const PINK_RGB: (u8, u8, u8) = (0xFF, 0xCA, 0xD4);
pub const ROSE_RGB: (u8, u8, u8) = (0xF4, 0xAC, 0xB7);
pub const MAUVE_RGB: (u8, u8, u8) = (0x9D, 0x81, 0x89);

const fn rgb8(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub const SAGE: Color = rgb8(0xD8, 0xE2, 0xDC);
pub const PEACH: Color = rgb8(0xFF, 0xE5, 0xD9);
pub const PINK: Color = rgb8(0xFF, 0xCA, 0xD4);
pub const ROSE: Color = rgb8(0xF4, 0xAC, 0xB7);
pub const MAUVE: Color = rgb8(0x9D, 0x81, 0x89);

/// Warm-neutral card surface (permitted neutral for readability).
pub const CARD: Color = rgb8(0xFD, 0xFB, 0xF9);
pub const DARK_BG: Color = rgb8(0x1E, 0x1B, 0x1D);
pub const DARK_SURFACE: Color = rgb8(0x26, 0x22, 0x26);
pub const DARK_SURFACE_2: Color = rgb8(0x2E, 0x28, 0x2C);
pub const DARK_TEXT: Color = rgb8(0xC9, 0xAE, 0xB5);
pub const DARK_TEXT_STRONG: Color = rgb8(0xF0, 0xE7, 0xE6);

pub const APP_ID: &str = "clipvault";
pub const DB_FILE: &str = "clipvault.db";
pub const SETTINGS_FILE: &str = "settings.json";
pub const ASSETS_DIR: &str = "assets";
pub const LOG_FILE: &str = "clipvault.log";

pub const DEFAULT_MAX_ENTRIES: usize = 500;
pub const DEFAULT_MAX_AGE_DAYS: u32 = 30;
pub const DEFAULT_PAGE_SIZE: usize = 50;

/// Hotkey id used by RegisterHotKey for the show/hide combo.
pub const HOTKEY_ID_SHOW: i32 = 0xC1FF;
/// Tray callback message (WM_APP + 1).
pub const WM_TRAY_CALLBACK: u32 = 0x8001;
