//! Brand palette and light/dark Iced themes (v0.1.1 coolors palette).
//! ONLY the five official hexes are used as colors; alpha variants of them
//! serve as hairlines/shadows. Neutral white/charcoal/grayscale for contrast only.
//! Palette: CREAM #f2e9e4, ROSE #c9ada7, MAUVE #9a8c98, SLATE #4a4e69, INK #22223b.
use crate::config::ThemeMode;
use crate::contracts::*;
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use iced::theme::Palette;
use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Alpha variant of a brand color (used for hairline borders / soft shadows).
fn alpha(c: Color, a: f32) -> Color {
    Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a,
    }
}

pub fn light_theme() -> Theme {
    Theme::custom(
        "ClipVault Light".to_string(),
        Palette {
            background: CREAM,
            text: INK,
            primary: ROSE,
            success: MAUVE,
            danger: SLATE,
        },
    )
}

/// Complementary dark theme — deep INK/SLATE surfaces with CREAM/ROSE accents.
pub fn dark_theme() -> Theme {
    Theme::custom(
        "ClipVault Dark".to_string(),
        Palette {
            background: INK,
            text: CREAM,
            primary: ROSE,
            success: MAUVE,
            danger: SLATE,
        },
    )
}

pub fn theme_for(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => light_theme(),
        ThemeMode::Dark => dark_theme(),
    }
}

/// Linear blend of two brand colors (hover/press feedback — no new hues).
fn blend(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn is_dark(theme: &Theme) -> bool {
    theme.palette().background.r < 0.4
}

fn soft_border(theme: &Theme) -> Border {
    Border {
        radius: sp::r(sp::RADIUS_CARD),
        width: 1.0,
        color: if is_dark(theme) {
            alpha(MAUVE, 0.35)
        } else {
            alpha(MAUVE, 0.30)
        },
    }
}

/// CREAM / SLATE card with a soft border and subtle shadow.
pub fn card_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(if is_dark(theme) { CREAM } else { INK }),
        background: Some(Background::Color(if is_dark(theme) {
            SLATE
        } else {
            CREAM
        })),
        border: soft_border(theme),
        shadow: Shadow {
            color: alpha(INK, 0.10),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 12.0,
        },
    }
}

/// ROSE / SLATE panel (secondary surface).
pub fn panel_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(if is_dark(theme) { CREAM } else { INK }),
        background: Some(Background::Color(if is_dark(theme) { SLATE } else { ROSE })),
        border: soft_border(theme),
        shadow: Shadow::default(),
    }
}

fn button_bg(_theme: &Theme, status: button::Status, primary: bool) -> Option<Background> {
    let base = if primary { ROSE } else { MAUVE };
    let c = match status {
        button::Status::Hovered => blend(base, CREAM, 0.45),
        button::Status::Pressed => blend(base, INK, 0.20),
        button::Status::Disabled => alpha(MAUVE, 0.4),
        _ => base,
    };
    Some(Background::Color(c))
}

fn button_shadow(status: button::Status) -> Shadow {
    match status {
        button::Status::Hovered | button::Status::Pressed => Shadow {
            color: alpha(INK, 0.20),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        _ => Shadow::default(),
    }
}

/// High-contrast text on brand-colored buttons (INK on light, CREAM on dark).
fn button_text(theme: &Theme) -> Color {
    if is_dark(theme) { CREAM } else { INK }
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: button_bg(theme, status, true),
        text_color: button_text(theme),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 1.0,
            color: alpha(INK, 0.30),
        },
        shadow: button_shadow(status),
    }
}

pub fn subtle_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: button_bg(theme, status, false),
        text_color: button_text(theme),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 1.0,
            color: alpha(INK, 0.22),
        },
        shadow: button_shadow(status),
    }
}

/// Stateful toggle: ROSE when active, MAUVE when inactive (color + border read,
/// no Unicode glyphs — Fira Sans has no star/dingbat glyphs).
pub fn active_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let base = if active { ROSE } else { MAUVE };
        let c = match status {
            button::Status::Hovered => blend(base, CREAM, 0.45),
            button::Status::Pressed => blend(base, INK, 0.20),
            button::Status::Disabled => alpha(MAUVE, 0.4),
            _ => base,
        };
        button::Style {
            background: Some(Background::Color(c)),
            text_color: button_text(theme),
            border: Border {
                radius: sp::r(sp::RADIUS_BUTTON),
                width: if active { 2.0 } else { 1.0 },
                color: if active { ROSE } else { MAUVE },
            },
            shadow: if active {
                Shadow {
                    color: alpha(INK, 0.18),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                }
            } else {
                button_shadow(status)
            },
        }
    }
}

/// Content-kind badge colors drawn from the palette only.
pub fn badge_style(kind: ContentKind, theme: &Theme) -> container::Style {
    let bg = match kind {
        ContentKind::Text => MAUVE,
        ContentKind::Image => ROSE,
        ContentKind::Files => CREAM,
    };
    container::Style {
        text_color: Some(if is_dark(theme) { CREAM } else { INK }),
        background: Some(Background::Color(bg)),
        border: Border {
            radius: sp::r(sp::RADIUS_BADGE),
            width: 1.0,
            color: alpha(INK, 0.20),
        },
        shadow: Shadow::default(),
    }
}
