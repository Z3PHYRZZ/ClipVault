//! Shared constants. Integrator-owned. Fleet reads, never writes.
//!
//! Brand palette v0.1.2 — two colors only:
//!   PINK  `#FFCAD4` (light pastel pink)  — backgrounds, cards, light surfaces
//!   MAUVE `#9D8189` (muted mauve)         — panels, accents, text, borders, dark surfaces
//! Every chromatic color in the app resolves to one of these two. Neutral white
//! and charcoal are permitted ONLY for readability (text on colored surfaces).
use iced::Color;

pub const PINK_RGB: (u8, u8, u8) = (0xFF, 0xCA, 0xD4);
pub const MAUVE_RGB: (u8, u8, u8) = (0x9D, 0x81, 0x89);

const fn rgb8(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

/// The two official brand colors.
pub const PINK: Color = rgb8(0xFF, 0xCA, 0xD4);
pub const MAUVE: Color = rgb8(0x9D, 0x81, 0x89);

/// Neutral contrast colors (white / charcoal) — readability only, never accents.
pub const WHITE: Color = Color::WHITE;
pub const CHARCOAL: Color = rgb8(0x33, 0x2B, 0x2E);

/// Alpha variant of a brand color (hairlines / shadows — no new hues).
pub fn alpha(c: Color, a: f32) -> Color {
    Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a,
    }
}

// ---- Derived surface tokens (all resolve to the two brand colors) ----------
pub const CARD: Color = PINK; // light card
pub const DARK_BG: Color = MAUVE; // mauve-tinted dark background
pub const DARK_SURFACE: Color = MAUVE; // dark mauve panel
pub const DARK_SURFACE_2: Color = MAUVE; // mauve hairline on dark surfaces
pub const DARK_TEXT: Color = PINK; // light pink text on mauve
pub const DARK_TEXT_STRONG: Color = PINK; // strong light pink text

// ---- Backward-compatible role aliases -------------------------------------
// theme.rs / icon.rs keep their role names; they now resolve to the 2-color set.
pub const CHARCOAL_RGB: (u8, u8, u8) = (0x33, 0x2B, 0x2E);

pub const CREAM_RGB: (u8, u8, u8) = PINK_RGB;
pub const ROSE_RGB: (u8, u8, u8) = MAUVE_RGB;
pub const SLATE_RGB: (u8, u8, u8) = MAUVE_RGB;
pub const INK_RGB: (u8, u8, u8) = CHARCOAL_RGB;

pub const CREAM: Color = PINK;
pub const ROSE: Color = MAUVE;
pub const SLATE: Color = MAUVE;
pub const INK: Color = CHARCOAL;

pub const SAGE_RGB: (u8, u8, u8) = PINK_RGB;
pub const PEACH_RGB: (u8, u8, u8) = PINK_RGB;
pub const SAGE: Color = PINK;
pub const PEACH: Color = PINK;

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
