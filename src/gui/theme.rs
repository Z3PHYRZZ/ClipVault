//! Brand palette and light/dark Iced themes (v0.1.3 UI rework).
//! CREAM #F2E9E4 and INK #22223B are the ONLY chromatic colors. Neutral
//! white/charcoal are permitted for text contrast; hairlines/shadows are alpha
//! variants of the two brand colors. No other hues anywhere.
//!
//! Hover animation is message-driven: `AppState.hovered_card` / `hovered_nav`
//! flip on mouse_area on_enter/on_exit, and `card_hover_style` / `nav_hover_style`
//! recompute a lifted shadow + thicker border each frame.
use crate::config::ThemeMode;
use crate::contracts::*;
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use iced::theme::Palette;
use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn light_theme() -> Theme {
    Theme::custom(
        "ClipVault Light".to_string(),
        Palette {
            background: CREAM,
            text: INK,
            primary: INK,
            success: CREAM,
            danger: INK,
        },
    )
}

/// Complementary dark theme: INK ground, CREAM accents, CREAM text.
pub fn dark_theme() -> Theme {
    Theme::custom(
        "ClipVault Dark".to_string(),
        Palette {
            background: INK,
            text: CREAM,
            primary: CREAM,
            success: INK,
            danger: CREAM,
        },
    )
}

pub fn theme_for(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => light_theme(),
        ThemeMode::Dark => dark_theme(),
    }
}

fn blend(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn is_dark(theme: &Theme) -> bool {
    theme.palette().background.r < 0.5
}

/// Contrast text for a colored button surface: CREAM on INK, INK on CREAM.
/// Decided by the ACTUAL background (not by theme), so dark-theme buttons with
/// an INK fill never end up with invisible INK-on-INK text.
fn button_text(bg: Color) -> Color {
    let l = 0.2126 * bg.r + 0.7152 * bg.g + 0.0722 * bg.b;
    if l < 0.5 { CREAM } else { INK }
}

fn soft_border(theme: &Theme) -> Border {
    Border {
        radius: sp::r(sp::RADIUS_CARD),
        width: 1.0,
        color: alpha(INK, if is_dark(theme) { 0.55 } else { 0.22 }),
    }
}

fn card_surface(theme: &Theme) -> Color {
    if is_dark(theme) { INK } else { CREAM }
}

/// CREAM / INK card with a soft border and subtle shadow.
pub fn card_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(if is_dark(theme) { CREAM } else { INK }),
        background: Some(Background::Color(card_surface(theme))),
        border: soft_border(theme),
        shadow: Shadow {
            color: alpha(INK, 0.14),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 12.0,
        },
    }
}

/// Card style with an explicit hovered flag (message-driven hover animation).
/// Hover lifts the shadow by HOVER_LIFT, widens the border, and blends the
/// background slightly toward the contrasting brand color.
pub fn card_hover_style(theme: &Theme, hovered: bool) -> container::Style {
    let mut s = card_style(theme);
    if !hovered {
        return s;
    }
    let contrasting = if is_dark(theme) { CREAM } else { INK };
    s.background = Some(Background::Color(blend(
        card_surface(theme),
        contrasting,
        0.08,
    )));
    s.border = Border {
        radius: sp::r(sp::RADIUS_CARD),
        width: sp::HOVER_BORDER,
        color: alpha(INK, 0.45),
    };
    s.shadow = Shadow {
        color: alpha(INK, 0.22),
        offset: Vector::new(0.0, sp::HOVER_LIFT),
        blur_radius: 16.0,
    };
    s
}

/// INK / CREAM panel (secondary surface — the contrasting brand color).
pub fn panel_style(theme: &Theme) -> container::Style {
    let bg = if is_dark(theme) { CREAM } else { INK };
    container::Style {
        text_color: Some(button_text(bg)),
        background: Some(Background::Color(bg)),
        border: soft_border(theme),
        shadow: Shadow::default(),
    }
}

/// The button's background color for a given status (single source of truth
/// shared by the style background and the contrast-checked text color).
fn button_color(status: button::Status, primary: bool) -> Color {
    let base = if primary { INK } else { CREAM };
    match status {
        button::Status::Hovered => blend(base, if primary { CREAM } else { INK }, 0.35),
        button::Status::Pressed => blend(base, CHARCOAL, 0.18),
        button::Status::Disabled => alpha(INK, 0.4),
        _ => base,
    }
}

fn button_shadow(status: button::Status) -> Shadow {
    match status {
        button::Status::Hovered | button::Status::Pressed => Shadow {
            color: alpha(INK, 0.24),
            offset: Vector::new(0.0, sp::HOVER_LIFT),
            blur_radius: 8.0,
        },
        _ => Shadow::default(),
    }
}

pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = button_color(status, true);
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: button_text(bg),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 2.0,
            color: alpha(INK, 0.65),
        },
        shadow: button_shadow(status),
    }
}

pub fn subtle_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = button_color(status, false);
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: button_text(bg),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 1.0,
            color: alpha(INK, 0.30),
        },
        shadow: button_shadow(status),
    }
}

/// Stateful toggle: INK when active, CREAM when inactive (color + border read,
/// no Unicode glyphs — Fira Sans has no star/dingbat glyphs).
pub fn active_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let base = if active { INK } else { CREAM };
        let c = match status {
            button::Status::Hovered => blend(base, CREAM, 0.35),
            button::Status::Pressed => blend(base, CHARCOAL, 0.18),
            button::Status::Disabled => alpha(INK, 0.4),
            _ => base,
        };
        button::Style {
            background: Some(Background::Color(c)),
            text_color: button_text(c),
            border: Border {
                radius: sp::r(sp::RADIUS_BUTTON),
                width: if active { 2.0 } else { 1.0 },
                color: if active { INK } else { CREAM },
            },
            shadow: if active {
                Shadow {
                    color: alpha(INK, 0.22),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                }
            } else {
                button_shadow(status)
            },
        }
    }
}

/// Nav/button style with an explicit hovered flag (message-driven hover animation).
pub fn nav_hover_style(_theme: &Theme, hovered: bool, active: bool) -> button::Style {
    let base = if active { INK } else { CREAM };
    let contrasting = if active { CREAM } else { INK };
    let bg = if hovered {
        blend(base, contrasting, 0.35)
    } else {
        base
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: button_text(bg),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: if active || hovered {
                sp::HOVER_BORDER
            } else {
                1.0
            },
            color: if active { INK } else { alpha(INK, 0.30) },
        },
        shadow: if hovered {
            Shadow {
                color: alpha(INK, 0.22),
                offset: Vector::new(0.0, sp::HOVER_LIFT),
                blur_radius: 8.0,
            }
        } else {
            Shadow::default()
        },
    }
}

/// Content-kind badge colors drawn from the two brand colors only.
pub fn badge_style(kind: ContentKind, _theme: &Theme) -> container::Style {
    let bg = match kind {
        ContentKind::Text => INK,
        ContentKind::Image => CREAM,
        ContentKind::Files => INK,
    };
    container::Style {
        text_color: Some(button_text(bg)),
        background: Some(Background::Color(bg)),
        border: Border {
            radius: sp::r(sp::RADIUS_BADGE),
            width: 1.0,
            color: alpha(INK, 0.30),
        },
        shadow: Shadow::default(),
    }
}
