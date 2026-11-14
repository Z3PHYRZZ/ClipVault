//! Shared constants. Integrator-owned. Fleet reads, never writes.
//!
//! Brand palette v0.1.3 — two colors only:
//!   CREAM `#F2E9E4` (warm off-white)  — light backgrounds, cards, light text
//!   INK   `#22223B` (deep navy-ink)   — dark backgrounds, panels, text, borders
//! Every chromatic color in the app resolves to one of these two. Neutral white
//! and charcoal are permitted ONLY for readability (text on colored surfaces).
use iced::Color;

pub const CREAM_RGB: (u8, u8, u8) = (0xF2, 0xE9, 0xE4);
pub const INK_RGB: (u8, u8, u8) = (0x22, 0x22, 0x3B);

const fn rgb8(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

/// The two official brand colors.
pub const CREAM: Color = rgb8(0xF2, 0xE9, 0xE4);
pub const INK: Color = rgb8(0x22, 0x22, 0x3B);

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

// ---- Frosted-glass tokens (v0.1.5) ----------------------------------------
// v0.1.6: the OS-level blur itself comes from DWM Mica/acrylic applied to the
// native HWND (see src/platform/backdrop.rs); these alpha-tinted surfaces sit on
// top of that real backdrop, not a simulated one. Chrome is near-clear, content
// cards stay near-opaque for readability. All values are alpha variants of the
// two brand colors + the permitted neutral WHITE; no new hue is introduced.
/// Chrome glass (sidebar / search bar / toolbar / status): near-clear so the
/// natively blurred desktop reads through. Still tinted with the brand hue.
pub const GLASS_PANEL_LIGHT: Color = Color {
    r: CREAM.r,
    g: CREAM.g,
    b: CREAM.b,
    a: 0.42,
};
pub const GLASS_PANEL_DARK: Color = Color {
    r: INK.r,
    g: INK.g,
    b: INK.b,
    a: 0.40,
};
/// Content scrim (entry cards / preview pane): kept near-opaque on purpose.
/// Full acrylic applies to the window chrome, never to text-bearing surfaces —
/// this is what guarantees readability regardless of the desktop behind it.
pub const GLASS_CARD_LIGHT: Color = Color {
    r: CREAM.r,
    g: CREAM.g,
    b: CREAM.b,
    a: 0.94,
};
pub const GLASS_CARD_DARK: Color = Color {
    r: INK.r,
    g: INK.g,
    b: INK.b,
    a: 0.93,
};
/// Specular 1px border: soft white highlight in light, faint in dark.
pub const SPECULAR_LIGHT: Color = Color {
    r: WHITE.r,
    g: WHITE.g,
    b: WHITE.b,
    a: 0.40,
};
pub const SPECULAR_DARK: Color = Color {
    r: WHITE.r,
    g: WHITE.g,
    b: WHITE.b,
    a: 0.08,
};
/// Contrasting hairline that separates overlapping frosted layers.
pub const HAIRLINE_LIGHT: Color = Color {
    r: INK.r,
    g: INK.g,
    b: INK.b,
    a: 0.18,
};
pub const HAIRLINE_DARK: Color = Color {
    r: CREAM.r,
    g: CREAM.g,
    b: CREAM.b,
    a: 0.14,
};
/// Diffused ambient shadow (0 4px 20px rgba(0,0,0,0.05) equivalent).
pub const SHADOW_AMBIENT: Color = Color {
    r: INK.r,
    g: INK.g,
    b: INK.b,
    a: 0.10,
};
/// Ambient gradient stops — a subtle luminance shift of the SAME two brand
/// hues toward the permitted WHITE/CHARCOAL neutrals. No new hue: light stops are
/// CREAM lifted toward WHITE, dark stops are INK lifted/darkened toward CHARCOAL.
const fn toward(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.0,
    }
}
pub const GRADIENT_LIGHT_TOP: Color = toward(CREAM, WHITE, 0.55);
pub const GRADIENT_LIGHT_BOTTOM: Color = toward(CREAM, CHARCOAL, 0.10);
pub const GRADIENT_DARK_TOP: Color = toward(INK, CREAM, 0.10);
pub const GRADIENT_DARK_BOTTOM: Color = toward(INK, CHARCOAL, 0.35);

// ---- Derived surface tokens (all resolve to the two brand colors) ----------
pub const CARD: Color = CREAM; // light cream card
pub const DARK_BG: Color = INK; // ink background
pub const DARK_SURFACE: Color = INK; // ink panel
pub const DARK_SURFACE_2: Color = INK; // ink hairline on dark surfaces
pub const DARK_TEXT: Color = CREAM; // cream text on ink
pub const DARK_TEXT_STRONG: Color = CREAM;

// ---- Backward-compatible role aliases -------------------------------------
// theme.rs / icon.rs keep their role names; they now resolve to the 2-color set.
pub const CHARCOAL_RGB: (u8, u8, u8) = (0x33, 0x2B, 0x2E);

pub const PINK_RGB: (u8, u8, u8) = CREAM_RGB;
pub const MAUVE_RGB: (u8, u8, u8) = INK_RGB;
pub const ROSE_RGB: (u8, u8, u8) = INK_RGB;
pub const SLATE_RGB: (u8, u8, u8) = INK_RGB;

pub const PINK: Color = CREAM;
pub const MAUVE: Color = INK;
pub const ROSE: Color = INK;
pub const SLATE: Color = INK;

pub const SAGE_RGB: (u8, u8, u8) = CREAM_RGB;
pub const PEACH_RGB: (u8, u8, u8) = CREAM_RGB;
pub const SAGE: Color = CREAM;
pub const PEACH: Color = CREAM;

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
