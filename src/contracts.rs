//! Shared constants. Integrator-owned. Fleet reads, never writes.
//!
//! Brand palette v0.1.1 — https://coolors.co/palette/22223b-4a4e69-9a8c98-c9ada7-f2e9e4
//! Five official hexes, used for EVERY color in the app (UI, tray icon, docs).
//! Light -> dark: CREAM f2e9e4, ROSE c9ada7, MAUVE 9a8c98, SLATE 4a4e69, INK 22223b.
//! Neutral white/charcoal/grayscale are permitted only for readability.
use iced::Color;

/// All brand colors are declared as `const` using the const `Color::from_rgb`.
/// (`from_rgb8` is not const in iced 0.13; RGB tuples stay here as the source of truth.)
pub const CREAM_RGB: (u8, u8, u8) = (0xF2, 0xE9, 0xE4);
pub const ROSE_RGB: (u8, u8, u8) = (0xC9, 0xAD, 0xA7);
pub const MAUVE_RGB: (u8, u8, u8) = (0x9A, 0x8C, 0x98);
pub const SLATE_RGB: (u8, u8, u8) = (0x4A, 0x4E, 0x69);
pub const INK_RGB: (u8, u8, u8) = (0x22, 0x22, 0x3B);

const fn rgb8(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

// The five official brand colors.
pub const CREAM: Color = rgb8(0xF2, 0xE9, 0xE4);
pub const ROSE: Color = rgb8(0xC9, 0xAD, 0xA7);
pub const MAUVE: Color = rgb8(0x9A, 0x8C, 0x98);
pub const SLATE: Color = rgb8(0x4A, 0x4E, 0x69);
pub const INK: Color = rgb8(0x22, 0x22, 0x3B);

// Derived surfaces: only blends/alpha of the five brand colors (no unrelated hues).
pub const CARD: Color = CREAM; // warm cream card
pub const DARK_BG: Color = INK; // deep navy-ink background
pub const DARK_SURFACE: Color = SLATE; // dark slate panel
pub const DARK_SURFACE_2: Color = MAUVE; // mauve hairline on dark surfaces
pub const DARK_TEXT: Color = CREAM; // light text on dark
pub const DARK_TEXT_STRONG: Color = CREAM; // strong light text

// ---- Backward-compatible role aliases -------------------------------------
// Theme/icon code still refers to SAGE/PEACH/PINK/ROSE/MAUVE RGB tuples by name.
// They now resolve to the new five-color palette so every surface uses it.
pub const SAGE_RGB: (u8, u8, u8) = CREAM_RGB; // light background
pub const PEACH_RGB: (u8, u8, u8) = ROSE_RGB; // secondary panel
pub const PINK_RGB: (u8, u8, u8) = MAUVE_RGB; // selection / primary accent

pub const SAGE: Color = CREAM;
pub const PEACH: Color = ROSE;
pub const PINK: Color = MAUVE;

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
