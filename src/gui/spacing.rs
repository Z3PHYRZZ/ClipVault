//! Design tokens — single source of truth for spacing, radius, type, geometry.
//!
//! 2026-11-17: ported to shadcn/ui's visual language (default zinc theme).
//! Base grid: 4px -> 4 / 8 / 12 / 16 / 24 / 32. Radius follows shadcn's
//! --radius 8px: xl = 12, lg = 8, md = 6, sm = 4, pill = 999. Type scale maps
//! shadcn text-2xl/lg/sm/sm/xs. Control heights map h-9/h-10/h-11 and h-4.
//! Integrator-owned; agents read.

/// Base grid unit (pixels).
pub const UNIT: u16 = 4;

// ---- Spacing scale ----
pub const XS: u16 = 4;
pub const SM: u16 = 8;
pub const MD: u16 = 12;
pub const LG: u16 = 16;
pub const XL: u16 = 24;
pub const XXL: u16 = 32;

// ---- Corner radii (shadcn --radius 8px) ----
pub const RADIUS_XL: f32 = 12.0;
pub const RADIUS_LG: f32 = 8.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_PILL: f32 = 999.0;
// Component aliases retained for readable call sites.
pub const RADIUS_CARD: f32 = RADIUS_LG;
pub const RADIUS_BUTTON: f32 = RADIUS_MD;
pub const RADIUS_BADGE: f32 = RADIUS_PILL;
pub const RADIUS_INPUT: f32 = RADIUS_MD;

// ---- Type sizes (pixels): shadcn text-2xl / lg / sm / sm / xs ----
pub const TEXT_DISPLAY: u16 = 24;
pub const TEXT_TITLE: u16 = 18;
pub const TEXT_BODY: u16 = 14;
pub const TEXT_LABEL: u16 = 14;
pub const TEXT_CAPTION: u16 = 12;

// ---- Control geometry (shadcn h-9 / h-10 / h-11, icon, h-4) ----
pub const CONTROL_H_SM: u16 = 36;
pub const CONTROL_H: u16 = 40;
pub const CONTROL_H_LG: u16 = 44;
pub const ICON_BTN: u16 = 40;
pub const CHECKBOX_SIZE: u16 = 16;

// ---- Component padding (iced `Padding` tuples) ----
/// Card content (p-6 = 24px).
pub const CARD_PAD: [u16; 2] = [XL, XL];
/// Secondary panel / search bar wrapper.
pub const PANEL_PAD: [u16; 2] = [LG, XL];
/// Badge chip (px-2.5 py-0.5).
pub const BADGE_PAD: [u16; 2] = [XS, 10];
/// Standard button (h-10 px-4).
pub const BUTTON_PAD: [u16; 2] = [SM, LG];
pub const BUTTON_PAD_SM: [u16; 2] = [SM, MD];
pub const BUTTON_PAD_LG: [u16; 2] = [SM, XXL];
/// Text input / pick_list (h-10 px-3).
pub const INPUT_PAD: u16 = MD;
/// Sidebar shell.
pub const SIDEBAR_PAD: u16 = LG;
/// Sidebar nav item.
pub const NAV_PAD: [u16; 2] = [SM, MD];
/// Settings / scrollable content.
pub const CONTENT_PAD: u16 = XL;
/// Empty state.
pub const EMPTY_PAD: u16 = XXL;
/// History row card.
pub const ROW_PAD: [u16; 2] = [MD, LG];
/// Compact density history row.
pub const ROW_PAD_COMPACT: [u16; 2] = [SM, MD];
/// List container gutter.
pub const LIST_PAD: u16 = SM;
/// Status bar.
pub const STATUS_PAD: [u16; 2] = [MD, XL];

// ---- Gaps ----
pub const GAP_TIGHT: u16 = SM;
pub const GAP_SECTION: u16 = 20;
pub const GAP_TITLE: u16 = SM;

// ---- Hardcoded view geometry, centralized ----
pub const SIDEBAR_WIDTH: f32 = 224.0;
pub const THUMB_W: u16 = 52;
pub const THUMB_H: u16 = 42;
pub const THUMB_H_COMPACT: u16 = 30;
pub const PREVIEW_IMAGE_H: u16 = 300;
pub const PROPERTY_LABEL_W: u16 = 90;
pub const URL_DISPLAY_CHARS: usize = 60;
pub const MAX_TAGS_SHOWN: usize = 4;
pub const SETTINGS_INPUT_W: f32 = 110.0;
pub const HOTKEY_INPUT_W: f32 = 180.0;

/// Convert an `[u16;2]` token into `iced::Padding` where needed.
pub fn pad(v: [u16; 2]) -> iced::Padding {
    iced::Padding::from(v)
}

/// Radius helper for `Border { radius: r(...) }`.
pub fn r(px: f32) -> iced::border::Radius {
    iced::border::Radius::from(px)
}
