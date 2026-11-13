//! Brand palette and light/dark Iced themes.
//! Only the five official hexes act as accents; white/charcoal/grayscale are neutral.
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
            background: SAGE,
            text: MAUVE,
            primary: ROSE,
            success: PINK,
            danger: PEACH,
        },
    )
}

/// Complementary dark theme — NOT a simple inversion.
/// Deep charcoal surfaces, muted rose/sage accents, lighter mauve text.
pub fn dark_theme() -> Theme {
    Theme::custom(
        "ClipVault Dark".to_string(),
        Palette {
            background: DARK_BG,
            text: DARK_TEXT,
            primary: ROSE,
            success: PINK,
            danger: DARK_SURFACE_2,
        },
    )
}

pub fn theme_for(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Light => light_theme(),
        ThemeMode::Dark => dark_theme(),
    }
}

/// Linear blend of two colors (used for hover/press feedback).
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
            DARK_SURFACE_2
        } else {
            Color::from_rgba8(0x9D, 0x81, 0x89, 0.25)
        },
    }
}

/// Warm-white / charcoal card with a soft border and subtle shadow.
pub fn card_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(if is_dark(theme) { DARK_TEXT } else { MAUVE }),
        background: Some(Background::Color(if is_dark(theme) {
            DARK_SURFACE
        } else {
            CARD
        })),
        border: soft_border(theme),
        shadow: Shadow {
            color: Color::from_rgba8(0x9D, 0x81, 0x89, 0.08),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 12.0,
        },
    }
}

/// Peach / dark-surface panel (secondary surface).
pub fn panel_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(if is_dark(theme) { DARK_TEXT } else { MAUVE }),
        background: Some(Background::Color(if is_dark(theme) {
            DARK_SURFACE
        } else {
            PEACH
        })),
        border: soft_border(theme),
        shadow: Shadow::default(),
    }
}

fn button_bg(_theme: &Theme, status: button::Status, primary: bool) -> Option<Background> {
    let base = if primary { ROSE } else { PINK };
    let c = match status {
        // Hover lifts the button toward the lighter accent; press settles it
        // toward Muted Mauve for tactile feedback.
        button::Status::Hovered => blend(base, if primary { PINK } else { PEACH }, 0.55),
        button::Status::Pressed => blend(base, MAUVE, 0.18),
        button::Status::Disabled => Color::from_rgba8(0x9D, 0x81, 0x89, 0.4),
        _ => base,
    };
    Some(Background::Color(c))
}

fn button_shadow(status: button::Status) -> Shadow {
    match status {
        button::Status::Hovered | button::Status::Pressed => Shadow {
            color: Color::from_rgba8(0x9D, 0x81, 0x89, 0.18),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        _ => Shadow::default(),
    }
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: button_bg(theme, status, true),
        text_color: Color::from_rgb8(0x33, 0x2B, 0x2E),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 1.0,
            color: Color::from_rgba8(0x9D, 0x81, 0x89, 0.30),
        },
        shadow: button_shadow(status),
    }
}

pub fn subtle_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: button_bg(theme, status, false),
        text_color: Color::from_rgb8(0x33, 0x2B, 0x2E),
        border: Border {
            radius: sp::r(sp::RADIUS_BUTTON),
            width: 1.0,
            color: Color::from_rgba8(0x9D, 0x81, 0x89, 0.22),
        },
        shadow: button_shadow(status),
    }
}

/// Stateful toggle button: Dusty Rose when active, Pastel Pink when inactive.
/// Gives pin/favorite a clear on/off read without relying on Unicode glyphs
/// (Iced's bundled Fira Sans lacks star/dingbat glyphs and shows tofu boxes).
pub fn active_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let base = if active { ROSE } else { PINK };
        let c = match status {
            button::Status::Hovered => blend(base, PEACH, 0.55),
            button::Status::Pressed => blend(base, MAUVE, 0.18),
            button::Status::Disabled => Color::from_rgba8(0x9D, 0x81, 0x89, 0.4),
            _ => base,
        };
        button::Style {
            background: Some(Background::Color(c)),
            text_color: Color::from_rgb8(0x33, 0x2B, 0x2E),
            border: Border {
                radius: sp::r(sp::RADIUS_BUTTON),
                width: if active { 2.0 } else { 1.0 },
                color: if active { ROSE } else { PINK },
            },
            shadow: if active {
                Shadow {
                    color: Color::from_rgba8(0x9D, 0x81, 0x89, 0.16),
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
pub fn badge_style(kind: ContentKind, _theme: &Theme) -> container::Style {
    let bg = match kind {
        ContentKind::Text => PINK,
        ContentKind::Image => PEACH,
        ContentKind::Files => SAGE,
    };
    container::Style {
        text_color: Some(Color::from_rgb8(0x4A, 0x3B, 0x40)),
        background: Some(Background::Color(bg)),
        border: Border {
            radius: sp::r(sp::RADIUS_BADGE),
            width: 1.0,
            color: Color::from_rgba8(0x9D, 0x81, 0x89, 0.20),
        },
        shadow: Shadow::default(),
    }
}
