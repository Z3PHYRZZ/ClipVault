//! Design tokens - the single source of truth for spacing, radius, and type scale.
//! Every view/widget MUST use these constants instead of magic numbers so the
//! whole app reads as one calm, consistent system. Integrator-owned; agents read.
//!
//! Space scale: 4px base grid -> 4 / 8 / 12 / 16 / 24 / 32 (unchanged).
//! Type scale (v0.1.3 UI rework, raised for at-a-glance legibility):
//!   display 28, title 21, body 16, label 14, caption 12.
//! Padding is roomier across cards, buttons, panels, content, empty and status
//! surfaces; corner radii are rounder (card 16, button 12, badge 9, input 12).
//! HOVER_LIFT / HOVER_BORDER drive the message-driven hover animation: a 3px
//! shadow lift and a 2px border when a card or nav item is hovered.
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
pub const RADIUS_CARD: f32 = 16.0;
pub const RADIUS_BUTTON: f32 = 12.0;
pub const RADIUS_BADGE: f32 = 9.0;
pub const RADIUS_INPUT: f32 = 12.0;

// ---- Type sizes (pixels) ----
pub const TEXT_DISPLAY: u16 = 28;
pub const TEXT_TITLE: u16 = 21;
pub const TEXT_BODY: u16 = 16;
pub const TEXT_LABEL: u16 = 14;
pub const TEXT_CAPTION: u16 = 12;

// ---- Component-specific padding (iced `Padding` tuples) ----
/// Rounded card interior.
pub const CARD_PAD: [u16; 2] = [MD, LG]; // [12, 16] vertical/horizontal
/// Secondary panel / search bar wrapper.
pub const PANEL_PAD: [u16; 2] = [LG, XL]; // [16, 24]
/// Content-type badge chip.
pub const BADGE_PAD: [u16; 2] = [XS, SM]; // [4, 8]
/// Standard button (all variants).
pub const BUTTON_PAD: [u16; 2] = [MD, LG]; // [12, 16]
/// Text input / pick_list.
pub const INPUT_PAD: u16 = LG; // 16
/// Sidebar shell.
pub const SIDEBAR_PAD: u16 = LG; // 16
/// Sidebar nav item.
pub const NAV_PAD: [u16; 2] = [MD, LG]; // [12, 16]
/// Settings / scrollable content.
pub const CONTENT_PAD: u16 = XXL; // 32
/// Empty state.
pub const EMPTY_PAD: u16 = XXL; // 32
/// History row card.
pub const ROW_PAD: [u16; 2] = [MD, LG]; // [12, 16]
/// List container gutter.
pub const LIST_PAD: u16 = SM; // 8
/// Status bar.
pub const STATUS_PAD: [u16; 2] = [MD, XL]; // [12, 24]

// ---- Gaps ----
pub const GAP_TIGHT: u16 = SM; // 8  (between list cards, row children)
pub const GAP_SECTION: u16 = 20; // 20 (between settings sections; between LG 16 and XL 24)
pub const GAP_TITLE: u16 = SM; // 8  (title -> subtitle)

// ---- Hover animation tokens (message-driven, see theme.rs hover styles) ----
/// Shadow lift (px) applied to hovered cards / nav items.
pub const HOVER_LIFT: f32 = 3.0;
/// Border width (px) applied to hovered cards.
pub const HOVER_BORDER: f32 = 2.0;

/// Convert an `[u16;2]` token into `iced::Padding` where needed.
pub fn pad(v: [u16; 2]) -> iced::Padding {
    iced::Padding::from(v)
}

/// Radius helper for `Border { radius: r(...) }`.
pub fn r(px: f32) -> iced::border::Radius {
    iced::border::Radius::from(px)
}
