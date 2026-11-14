//! Frosted-glass theme (v0.1.5).
//!
//! CREAM #F2E9E4 and INK #22223B are the only chromatic colors; WHITE/CHARCOAL
//! are permitted neutrals. Iced 0.13 has no compositor backdrop blur, so "frost"
//! is an alpha-tinted glass surface layered over a soft ambient gradient, with a
//! 1px specular hairline and a diffused shadow. No animated blur: hover changes
//! only color / border / shadow offset, keeping scrolling and resize smooth.
use crate::config::ThemeMode;
use crate::contracts::*;
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use iced::gradient::Linear;
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

/// Soft ambient gradient placed BEHIND all frosted layers (root container).
/// Static 3-stop linear gradient — cheap, no per-frame work.
pub fn ambient_background(theme: &Theme) -> Background {
    let (top, mid, bottom) = if is_dark(theme) {
        (GRADIENT_DARK_TOP, INK, GRADIENT_DARK_BOTTOM)
    } else {
        (GRADIENT_LIGHT_TOP, CREAM, GRADIENT_LIGHT_BOTTOM)
    };
    Background::Gradient(
        Linear::new(std::f32::consts::FRAC_PI_2)
            .add_stop(0.0, top)
            .add_stop(0.5, mid)
            .add_stop(1.0, bottom)
            .into(),
    )
}

/// 1px specular border. WHITE highlight (0.40 light / 0.08 dark) on top of a
/// contrasting hairline so overlapping frosted layers stay separated.
fn specular_border(theme: &Theme, radius: iced::border::Radius, width: f32) -> Border {
    Border {
        radius,
        width,
        color: if is_dark(theme) {
            SPECULAR_DARK
        } else {
            SPECULAR_LIGHT
        },
    }
}

/// Diffused ambient shadow: offset (0,4), blur 20, alpha 0.10.
fn glass_shadow(lift: bool) -> Shadow {
    Shadow {
        color: SHADOW_AMBIENT,
        offset: Vector::new(0.0, if lift { sp::HOVER_LIFT + 1.0 } else { 4.0 }),
        blur_radius: 20.0,
    }
}

fn glass_panel(theme: &Theme) -> Color {
    if is_dark(theme) {
        GLASS_PANEL_DARK
    } else {
        GLASS_PANEL_LIGHT
    }
}
fn glass_card(theme: &Theme) -> Color {
    if is_dark(theme) {
        GLASS_CARD_DARK
    } else {
        GLASS_CARD_LIGHT
    }
}
fn readable_text(theme: &Theme) -> Color {
    if is_dark(theme) { CREAM } else { INK }
}

/// Deeper frosted panel (sidebar / top search bar): ~78-80% tint, specular edge,
/// diffused shadow. Anchors navigation visually.
pub fn panel_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(readable_text(theme)),
        background: Some(Background::Color(glass_panel(theme))),
        border: specular_border(theme, sp::r(sp::RADIUS_CARD), 1.0),
        shadow: glass_shadow(false),
    }
}

/// Softer frosted card / preview surface: ~64-66% tint, lighter edge, softer shadow.
pub fn card_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(readable_text(theme)),
        background: Some(Background::Color(glass_card(theme))),
        border: specular_border(theme, sp::r(sp::RADIUS_CARD), 1.0),
        shadow: Shadow {
            color: SHADOW_AMBIENT,
            offset: Vector::new(0.0, 3.0),
            blur_radius: 14.0,
        },
    }
}

/// A standalone frosted pane (preview / status bar): same soft card glass but
/// with the full diffused shadow so it separates from the ambient base.
pub fn glass_surface(theme: &Theme) -> container::Style {
    let mut s = card_style(theme);
    s.shadow = glass_shadow(false);
    s
}

/// Hover elevation: shadow lifts (equivalent of translateY(-1px)), border
/// thickens with the contrasting hairline, and the glass tint shifts slightly.
pub fn card_hover_style(theme: &Theme, hovered: bool) -> container::Style {
    let mut s = card_style(theme);
    if !hovered {
        return s;
    }
    let contrasting = if is_dark(theme) { CREAM } else { INK };
    s.background = Some(Background::Color(blend(
        glass_card(theme),
        contrasting,
        0.08,
    )));
    s.border = Border {
        radius: sp::r(sp::RADIUS_CARD),
        width: sp::HOVER_BORDER,
        color: if is_dark(theme) {
            SPECULAR_DARK
        } else {
            SPECULAR_LIGHT
        },
    };
    s.shadow = Shadow {
        color: alpha(INK, 0.16),
        offset: Vector::new(0.0, sp::HOVER_LIFT),
        blur_radius: 16.0,
    };
    s
}

/// Crisp semi-transparent button fill that blends into the frosted containers.
fn button_color(status: button::Status, primary: bool, theme: &Theme) -> Color {
    let base = if primary {
        alpha(INK, 0.90)
    } else {
        alpha(CREAM, if is_dark(theme) { 0.16 } else { 0.70 })
    };
    match status {
        button::Status::Hovered => blend(base, if primary { CREAM } else { INK }, 0.18),
        button::Status::Pressed => blend(base, CHARCOAL, 0.18),
        button::Status::Disabled => alpha(INK, 0.35),
        _ => base,
    }
}

fn button_text(bg: Color, theme: &Theme) -> Color {
    // Composite the semi-transparent pill over the ambient base before judging
    // luminance, otherwise a light pill on a dark background would wrongly get
    // dark text.
    let base = if is_dark(theme) {
        GRADIENT_DARK_TOP
    } else {
        GRADIENT_LIGHT_TOP
    };
    let a = bg.a;
    let eff = Color {
        r: bg.r * a + base.r * (1.0 - a),
        g: bg.g * a + base.g * (1.0 - a),
        b: bg.b * a + base.b * (1.0 - a),
        a: 1.0,
    };
    let l = 0.2126 * eff.r + 0.7152 * eff.g + 0.0722 * eff.b;
    if l < 0.5 { CREAM } else { INK }
}

fn button_shadow(status: button::Status) -> Shadow {
    match status {
        button::Status::Hovered | button::Status::Pressed => Shadow {
            color: SHADOW_AMBIENT,
            offset: Vector::new(0.0, sp::HOVER_LIFT),
            blur_radius: 12.0,
        },
        _ => Shadow::default(),
    }
}

fn pill_border(theme: &Theme, width: f32) -> Border {
    specular_border(theme, sp::r(sp::RADIUS_BUTTON), width)
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let bg = button_color(status, true, theme);
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: button_text(bg, theme),
        border: pill_border(theme, 1.0),
        shadow: button_shadow(status),
    }
}

pub fn subtle_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let bg = button_color(status, false, theme);
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: button_text(bg, theme),
        border: pill_border(theme, 1.0),
        shadow: button_shadow(status),
    }
}

pub fn active_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let base = if active {
            alpha(INK, 0.90)
        } else {
            alpha(CREAM, 0.70)
        };
        let c = match status {
            button::Status::Hovered => blend(base, if active { CREAM } else { INK }, 0.18),
            button::Status::Pressed => blend(base, CHARCOAL, 0.18),
            button::Status::Disabled => alpha(INK, 0.35),
            _ => base,
        };
        button::Style {
            background: Some(Background::Color(c)),
            text_color: button_text(c, theme),
            border: pill_border(theme, if active { 2.0 } else { 1.0 }),
            shadow: if active {
                Shadow {
                    color: SHADOW_AMBIENT,
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 10.0,
                }
            } else {
                button_shadow(status)
            },
        }
    }
}

pub fn nav_hover_style(theme: &Theme, hovered: bool, active: bool) -> button::Style {
    let base = if active {
        alpha(INK, 0.90)
    } else {
        alpha(CREAM, 0.70)
    };
    let bg = if hovered {
        blend(base, if active { CREAM } else { INK }, 0.18)
    } else {
        base
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: button_text(bg, theme),
        border: pill_border(
            theme,
            if active || hovered {
                sp::HOVER_BORDER
            } else {
                1.0
            },
        ),
        shadow: if hovered {
            Shadow {
                color: SHADOW_AMBIENT,
                offset: Vector::new(0.0, sp::HOVER_LIFT),
                blur_radius: 12.0,
            }
        } else {
            Shadow::default()
        },
    }
}

/// Crisp semi-transparent text input that blends into frosted containers.
pub fn input_style(
    theme: &Theme,
    _status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let bg = if is_dark(theme) {
        alpha(CREAM, 0.10)
    } else {
        alpha(WHITE, 0.55)
    };
    iced::widget::text_input::Style {
        background: Background::Color(bg),
        border: specular_border(theme, sp::r(sp::RADIUS_INPUT), 1.0),
        icon: readable_text(theme),
        placeholder: alpha(readable_text(theme), 0.55),
        value: readable_text(theme),
        selection: alpha(INK, 0.22),
    }
}

/// Crisp semi-transparent dropdown pill.
pub fn pick_list_style(
    theme: &Theme,
    _status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let bg = if is_dark(theme) {
        alpha(CREAM, 0.12)
    } else {
        alpha(WHITE, 0.60)
    };
    iced::widget::pick_list::Style {
        text_color: readable_text(theme),
        placeholder_color: alpha(readable_text(theme), 0.55),
        handle_color: readable_text(theme),
        background: Background::Color(bg),
        border: specular_border(theme, sp::r(sp::RADIUS_INPUT), 1.0),
    }
}

/// Content-kind badge: crisp frosted pill with specular edge.
pub fn badge_style(kind: ContentKind, theme: &Theme) -> container::Style {
    let bg = match kind {
        ContentKind::Text => alpha(INK, 0.88),
        ContentKind::Image => alpha(CREAM, 0.82),
        ContentKind::Files => alpha(INK, 0.88),
    };
    container::Style {
        text_color: Some(button_text(bg, theme)),
        background: Some(Background::Color(bg)),
        border: specular_border(theme, sp::r(sp::RADIUS_BADGE), 1.0),
        shadow: Shadow::default(),
    }
}
