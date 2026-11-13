//! Design tokens — the single source of truth for spacing, radius, and type scale.
//! Every view/widget MUST use these constants instead of magic numbers so the
//! whole app reads as one calm, consistent system. Integrator-owned; agents read.
//!
//! Scale: 4px base grid -> 4 / 8 / 12 / 16 / 24 / 32.
//! Typography hierarchy: display 24, title 18, body 14, label 13, caption 11.
//! Only the two brand hexes (CREAM/INK) + neutrals are accents (see theme.rs).

/// Base grid unit (pixels).
pub const UNIT: u16 = 4;

// ---- Spacing scale ----
pub const XS: u16 = 4;
pub const SM: u16 = 8;
pub const MD: u16 = 12;
pub const LG: u16 = 16;
pub const XL: u16 = 24;
pub const XXL: u16 = 32;

// ---- Corner radii ----
pub const RADIUS_CARD: f32 = 14.0;
pub const RADIUS_BUTTON: f32 = 10.0;
pub const RADIUS_BADGE: f32 = 8.0;
pub const RADIUS_INPUT: f32 = 10.0;

// ---- Type sizes (pixels) ----
pub const TEXT_DISPLAY: u16 = 24;
pub const TEXT_TITLE: u16 = 18;
pub const TEXT_BODY: u16 = 14;
pub const TEXT_LABEL: u16 = 13;
pub const TEXT_CAPTION: u16 = 11;

// ---- Component-specific padding (iced `Padding` tuples) ----
/// Rounded card interior.
pub const CARD_PAD: [u16; 2] = [MD, MD]; // [12, 12] vertical/horizontal
/// Secondary panel / search bar wrapper.
pub const PANEL_PAD: [u16; 2] = [MD, LG]; // [12, 16]
/// Content-type badge chip.
pub const BADGE_PAD: [u16; 2] = [XS, SM]; // [4, 8]
/// Standard button (all variants).
pub const BUTTON_PAD: [u16; 2] = [SM, MD]; // [8, 12]
/// Text input / pick_list.
pub const INPUT_PAD: u16 = MD; // 12
/// Sidebar shell.
pub const SIDEBAR_PAD: u16 = LG; // 16
/// Sidebar nav item.
pub const NAV_PAD: [u16; 2] = [MD, LG]; // [12, 16]
/// Settings / scrollable content.
pub const CONTENT_PAD: u16 = XL; // 24
/// Empty state.
pub const EMPTY_PAD: u16 = XL; // 24
/// History row card.
pub const ROW_PAD: [u16; 2] = [MD, LG]; // [12, 16]
/// List container gutter.
pub const LIST_PAD: u16 = SM; // 8
/// Status bar.
pub const STATUS_PAD: [u16; 2] = [SM, LG]; // [8, 16]

// ---- Gaps ----
pub const GAP_TIGHT: u16 = SM; // 8  (between list cards, row children)
pub const GAP_SECTION: u16 = LG; // 16 (between settings sections)
pub const GAP_TITLE: u16 = SM; // 8  (title -> subtitle)

/// Convert an `[u16;2]` token into `iced::Padding` where needed.
pub fn pad(v: [u16; 2]) -> iced::Padding {
    iced::Padding::from(v)
}

/// Radius helper for `Border { radius: r(...) }`.
pub fn r(px: f32) -> iced::border::Radius {
    iced::border::Radius::from(px)
}
