//! Brand palette and light/dark Iced themes (v0.1.2 — two colors only).
//! PINK #FFCAD4 and MAUVE #9D8189 are the ONLY chromatic colors. Neutral
//! white/charcoal are permitted for text contrast; hairlines/shadows are alpha
//! variants of the two brand colors. No other hues anywhere.
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
            background: PINK,
            text: CHARCOAL,
            primary: MAUVE,
            success: PINK,
            danger: MAUVE,
        },
    )
}

/// Complementary dark theme: MAUVE ground, PINK accents, PINK/white text.
pub fn dark_theme() -> Theme {
    Theme::custom(
        "ClipVault Dark".to_string(),
        Palette {
            background: MAUVE,
            text: PINK,
            primary: PINK,
            success: MAUVE,
            danger: PINK,
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
    theme.palette().background.r < 0.7
}

fn soft_border(_theme: &Theme) -> Border {
    Border {
        radius: sp::r(sp::RADIUS_CARD),
        width: 1.0,
        color: alpha(MAUVE, if is_dark(_theme) { 0.55 } else { 0.35 }),
    }
}

/// PINK / MAUVE card with a soft border and subtle shadow.
pub fn card_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(if is_dark(theme) { PINK } else { CHARCOAL }),
        background: Some(Background::Color(if is_dark(theme) { MAUVE } else { PINK })),
        border: soft_border(theme),
        shadow: Shadow {
            color: alpha(MAUVE, 0.14),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 12.0,
        },
    }
}

/// MAUVE / PINK panel (secondary surface — the contrasting brand color).
pub fn panel_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(if is_dark(theme) { PINK } else { CHARCOAL }),
        background: Some(Background::Color(if is_dark(theme) { PINK } else { MAUVE })),
        border: soft_border(theme),
        shadow: Shadow::default(),
    }
}

fn button_bg(_theme: &Theme, status: button::Status, primary: bool) -> Option<Background> {
    let base = if primary { MAUVE } else { PINK };
    let c = match status {
        button::Status::Hovered => blend(base, if primary { PINK } else { MAUVE }, 0.45),
        button::Status::Pressed => blend(base, CHARCOAL, 0.18),
        button::Status::Disabled => alpha(MAUVE, 0.4),
        _ => base,
    };
    Some(Background::Color(c))
}

fn button_shadow(status: button::Status) -> Shadow {
    match status {
        button::Status::Hovered | button::Status::Pressed => Shadow {
            color: alpha(MAUVE, 0.24),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        _ => Shadow::default(),
    }
}

/// Contrast text on colored buttons: PINK on MAUVE surfaces, CHARCOAL on PINK.
fn button_text(theme: &Theme, primary: bool) -> Color {
    if is_dark(theme) || primary {
        PINK
    } else {
        CHARCOAL
    }
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: button_bg(theme, status, true),
        text_color: button_text(theme, true),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 1.0,
            color: alpha(MAUVE, 0.45),
        },
        shadow: button_shadow(status),
    }
}

pub fn subtle_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: button_bg(theme, status, false),
        text_color: button_text(theme, false),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 1.0,
            color: alpha(MAUVE, 0.30),
        },
        shadow: button_shadow(status),
    }
}

/// Stateful toggle: MAUVE when active, PINK when inactive (color + border read,
/// no Unicode glyphs — Fira Sans has no star/dingbat glyphs).
pub fn active_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let base = if active { MAUVE } else { PINK };
        let c = match status {
            button::Status::Hovered => blend(base, PINK, 0.45),
            button::Status::Pressed => blend(base, CHARCOAL, 0.18),
            button::Status::Disabled => alpha(MAUVE, 0.4),
            _ => base,
        };
        button::Style {
            background: Some(Background::Color(c)),
            text_color: button_text(theme, false),
            border: Border {
                radius: sp::r(sp::RADIUS_BUTTON),
                width: if active { 2.0 } else { 1.0 },
                color: if active { MAUVE } else { PINK },
            },
            shadow: if active {
                Shadow {
                    color: alpha(MAUVE, 0.22),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                }
            } else {
                button_shadow(status)
            },
        }
    }
}

/// Content-kind badge colors drawn from the two brand colors only.
pub fn badge_style(kind: ContentKind, theme: &Theme) -> container::Style {
    let bg = match kind {
        ContentKind::Text => MAUVE,
        ContentKind::Image => PINK,
        ContentKind::Files => MAUVE,
    };
    container::Style {
        text_color: Some(if is_dark(theme) { PINK } else { CHARCOAL }),
        background: Some(Background::Color(bg)),
        border: Border {
            radius: sp::r(sp::RADIUS_BADGE),
            width: 1.0,
            color: alpha(MAUVE, 0.30),
        },
        shadow: Shadow::default(),
    }
}
