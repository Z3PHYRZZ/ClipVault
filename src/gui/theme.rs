//! shadcn-style multi-theme design system for iced 0.13.
//!
//! Six color themes (Zinc / Slate / Stone / Rose / Blue / Green), each with a
//! complete light+dark palette. Every widget role (background, card, primary,
//! muted, accent, border, ring, destructive) resolves through `spec(theme)`, so
//! all components stay consistent when the theme changes. Dark-mode detection,
//! the text-on-fill contrast guard, and animate-ui press feedback are preserved.
use crate::config::ThemeMode;
use crate::contracts::*;
use crate::domain::ContentKind;
use crate::gui::motion::ease_out_cubic;
use crate::gui::spacing as sp;
use iced::widget::{button, checkbox, container, pick_list, rule, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

fn c(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

/// shadcn Popover chrome for combo_box dropdowns.
pub fn menu_style(theme: &Theme) -> iced::overlay::menu::Style {
    iced::overlay::menu::Style {
        background: Background::Color(spec(theme).popover),
        border: Border {
            radius: sp::r(sp::RADIUS_MD),
            width: 1.0,
            color: border_color(theme),
        },
        text_color: foreground(theme),
        selected_text_color: spec(theme).accent_fg,
        selected_background: Background::Color(accent(theme)),
    }
}

/// animate-ui press feedback: short ease-out shadow on hover/press.
fn press_shadow(theme: &Theme, status: button::Status, animate: bool) -> Shadow {
    if !animate {
        return Shadow::default();
    }
    let t = match status {
        button::Status::Hovered => ease_out_cubic(0.45),
        button::Status::Pressed => ease_out_cubic(0.85),
        _ => 0.0,
    };
    if t <= 0.0 {
        Shadow::default()
    } else {
        let c = if is_dark(theme) {
            spec(theme).primary
        } else {
            spec(theme).muted
        };
        Shadow {
            color: Color { a: 0.10 * t, ..c },
            offset: Vector::new(0.0, 1.0 + 2.0 * t),
            blur_radius: 2.0 + 6.0 * t,
        }
    }
}

/// Every color role the UI uses, for one mode of one theme.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // complete palette spec; some fg roles are implicit via text_on()
struct PaletteSpec {
    dark: bool,
    background: Color,
    foreground: Color,
    card: Color,
    card_fg: Color,
    popover: Color,
    popover_fg: Color,
    primary: Color,
    primary_fg: Color,
    secondary: Color,
    secondary_fg: Color,
    muted: Color,
    muted_fg: Color,
    accent: Color,
    accent_fg: Color,
    destructive: Color,
    destructive_fg: Color,
    border: Color,
    input: Color,
    ring: Color,
}

/// Resolve the active spec from the iced theme. Each built theme stores its
/// ThemeMode+mode in the `success` palette slot, letting every style fn read
/// the right palette without signature changes across the app.
fn spec(theme: &Theme) -> PaletteSpec {
    let p = theme.palette();
    let encoded = (p.success.r * 14.0).round() as u8;
    let mode_idx = encoded / 2;
    let dark = encoded % 2 == 1;
    let mode = match mode_idx {
        0 => ThemeMode::Zinc,
        1 => ThemeMode::Slate,
        2 => ThemeMode::Stone,
        3 => ThemeMode::Rose,
        4 => ThemeMode::Blue,
        6 => ThemeMode::Violet,
        _ => ThemeMode::Green,
    };
    palette_spec(mode, dark)
}

fn palette_spec(mode: ThemeMode, dark: bool) -> PaletteSpec {
    // Four-tone soft ladder per theme: background < card < popover (raised),
    // plus muted/accent surface. All tones share one gentle hue family.
    let pick = |l: Color, d: Color| if dark { d } else { l };
    // bg, card, pop, mut_, pri, pri_fg, sec, sec_fg, acc, acc_fg, des, border, ring
    let (bg, card, pop, mut_, pri, pri_fg, sec, sec_fg, acc, acc_fg, des, border, ring) =
        match (mode, dark) {
            (ThemeMode::Zinc, false) => (
                c(0xF3, 0xF3, 0xF5),
                c(0xF9, 0xF9, 0xFB),
                c(0xFE, 0xFE, 0xFF),
                c(0xE6, 0xE6, 0xEA),
                c(0x4A, 0x4A, 0x55),
                c(0xFF, 0xFF, 0xFF),
                c(0xE4, 0xE4, 0xE8),
                c(0x33, 0x33, 0x3B),
                c(0xDC, 0xDC, 0xE2),
                c(0x33, 0x33, 0x3B),
                c(0xD9, 0x53, 0x4F),
                c(0xD8, 0xD8, 0xDE),
                c(0x4A, 0x4A, 0x55),
            ),
            (ThemeMode::Zinc, true) => (
                c(0x17, 0x17, 0x1A),
                c(0x2C, 0x2C, 0x32),
                c(0x36, 0x36, 0x3D),
                c(0x22, 0x22, 0x27),
                c(0xC9, 0xC9, 0xD2),
                c(0x17, 0x17, 0x1A),
                c(0x2C, 0x2C, 0x34),
                c(0x9A, 0x9A, 0xA5),
                c(0x34, 0x34, 0x3E),
                c(0xE6, 0xE6, 0xEC),
                c(0xC2, 0x5B, 0x57),
                c(0x32, 0x32, 0x39),
                c(0x9A, 0x9A, 0xA5),
            ),
            (ThemeMode::Slate, false) => (
                c(0xEF, 0xF3, 0xF8),
                c(0xF7, 0xFA, 0xFD),
                c(0xFC, 0xFD, 0xFE),
                c(0xDF, 0xE6, 0xEF),
                c(0x3F, 0x53, 0x6E),
                c(0xFF, 0xFF, 0xFF),
                c(0xDC, 0xE6, 0xF2),
                c(0x2E, 0x3D, 0x52),
                c(0xD2, 0xE0, 0xF0),
                c(0x2E, 0x3D, 0x52),
                c(0xD9, 0x53, 0x4F),
                c(0xD3, 0xDE, 0xEA),
                c(0x3F, 0x53, 0x6E),
            ),
            (ThemeMode::Slate, true) => (
                c(0x15, 0x1C, 0x24),
                c(0x2A, 0x38, 0x48),
                c(0x35, 0x45, 0x5A),
                c(0x20, 0x2B, 0x37),
                c(0xB9, 0xCB, 0xE0),
                c(0x10, 0x18, 0x23),
                c(0x22, 0x32, 0x48),
                c(0x8B, 0x9E, 0xB4),
                c(0x29, 0x3A, 0x50),
                c(0xDC, 0xE7, 0xF2),
                c(0xC2, 0x5B, 0x57),
                c(0x27, 0x34, 0x48),
                c(0x8B, 0x9E, 0xB4),
            ),
            (ThemeMode::Stone, false) => (
                c(0xF4, 0xF1, 0xEB),
                c(0xFA, 0xF8, 0xF3),
                c(0xFE, 0xFD, 0xF9),
                c(0xE8, 0xE3, 0xD9),
                c(0x5A, 0x4F, 0x43),
                c(0xFF, 0xFF, 0xFF),
                c(0xE8, 0xE0, 0xD4),
                c(0x40, 0x37, 0x2E),
                c(0xDE, 0xD4, 0xC5),
                c(0x40, 0x37, 0x2E),
                c(0xD9, 0x53, 0x4F),
                c(0xE0, 0xD7, 0xC9),
                c(0x5A, 0x4F, 0x43),
            ),
            (ThemeMode::Stone, true) => (
                c(0x1C, 0x1A, 0x16),
                c(0x33, 0x2F, 0x26),
                c(0x3E, 0x39, 0x30),
                c(0x28, 0x25, 0x1E),
                c(0xD8, 0xC9, 0xB2),
                c(0x17, 0x14, 0x0F),
                c(0x2E, 0x28, 0x1F),
                c(0xA3, 0x95, 0x7F),
                c(0x37, 0x30, 0x23),
                c(0xEF, 0xE6, 0xD7),
                c(0xC2, 0x5B, 0x57),
                c(0x31, 0x2B, 0x21),
                c(0xA3, 0x95, 0x7F),
            ),
            (ThemeMode::Rose, false) => (
                c(0xF5, 0xED, 0xF0),
                c(0xFA, 0xF4, 0xF7),
                c(0xFE, 0xF9, 0xFB),
                c(0xEB, 0xDC, 0xE2),
                c(0x9B, 0x4A, 0x63),
                c(0xFF, 0xFF, 0xFF),
                c(0xF2, 0xDC, 0xE2),
                c(0x6E, 0x3F, 0x50),
                c(0xEB, 0xD0, 0xD8),
                c(0x6E, 0x3F, 0x50),
                c(0xD9, 0x53, 0x4F),
                c(0xEC, 0xD6, 0xDD),
                c(0x9B, 0x4A, 0x63),
            ),
            (ThemeMode::Rose, true) => (
                c(0x20, 0x1A, 0x1E),
                c(0x34, 0x29, 0x2F),
                c(0x3E, 0x32, 0x39),
                c(0x2A, 0x21, 0x26),
                c(0xE8, 0xB9, 0xC7),
                c(0x1C, 0x12, 0x16),
                c(0x36, 0x23, 0x2C),
                c(0xB6, 0x90, 0x9D),
                c(0x41, 0x2A, 0x35),
                c(0xF7, 0xDC, 0xE4),
                c(0xC2, 0x5B, 0x57),
                c(0x38, 0x24, 0x2E),
                c(0xB6, 0x90, 0x9D),
            ),
            (ThemeMode::Blue, false) => (
                c(0xEC, 0xF3, 0xFC),
                c(0xF5, 0xFA, 0xFF),
                c(0xFB, 0xFD, 0xFF),
                c(0xDA, 0xE4, 0xF2),
                c(0x3A, 0x65, 0x8F),
                c(0xFF, 0xFF, 0xFF),
                c(0xD9, 0xE7, 0xF6),
                c(0x2C, 0x4B, 0x68),
                c(0xCF, 0xE0, 0xF2),
                c(0x2C, 0x4B, 0x68),
                c(0xD9, 0x53, 0x4F),
                c(0xD4, 0xE2, 0xF0),
                c(0x3A, 0x65, 0x8F),
            ),
            (ThemeMode::Blue, true) => (
                c(0x15, 0x1D, 0x2A),
                c(0x29, 0x39, 0x4F),
                c(0x34, 0x46, 0x5F),
                c(0x1F, 0x2C, 0x3E),
                c(0xAF, 0xCD, 0xE8),
                c(0x10, 0x1A, 0x26),
                c(0x21, 0x36, 0x51),
                c(0x88, 0xA4, 0xBE),
                c(0x28, 0x40, 0x5D),
                c(0xD9, 0xE9, 0xF7),
                c(0xC2, 0x5B, 0x57),
                c(0x24, 0x39, 0x4F),
                c(0x88, 0xA4, 0xBE),
            ),
            (ThemeMode::Green, false) => (
                c(0xEF, 0xF5, 0xF0),
                c(0xF7, 0xFB, 0xF8),
                c(0xFC, 0xFE, 0xFD),
                c(0xDF, 0xEB, 0xE2),
                c(0x3E, 0x73, 0x55),
                c(0xFF, 0xFF, 0xFF),
                c(0xD8, 0xEA, 0xDF),
                c(0x2C, 0x51, 0x3C),
                c(0xCD, 0xE4, 0xD5),
                c(0x2C, 0x51, 0x3C),
                c(0xD9, 0x53, 0x4F),
                c(0xD2, 0xE5, 0xD9),
                c(0x3E, 0x73, 0x55),
            ),
            (ThemeMode::Green, true) => (
                c(0x17, 0x1F, 0x1A),
                c(0x2B, 0x3A, 0x30),
                c(0x35, 0x47, 0x3B),
                c(0x21, 0x2D, 0x25),
                c(0xB3, 0xD8, 0xC1),
                c(0x10, 0x1B, 0x15),
                c(0x20, 0x38, 0x2A),
                c(0x88, 0xA9, 0x97),
                c(0x27, 0x45, 0x33),
                c(0xDB, 0xED, 0xE2),
                c(0xC2, 0x5B, 0x57),
                c(0x23, 0x39, 0x29),
                c(0x88, 0xA9, 0x97),
            ),
            (ThemeMode::Violet, false) => (
                c(0xF0, 0xEC, 0xF8),
                c(0xE0, 0xD8, 0xEE),
                c(0xF8, 0xF5, 0xFC),
                c(0xFD, 0xFB, 0xFF),
                c(0x7B, 0x5A, 0xA8),
                c(0xFF, 0xFF, 0xFF),
                c(0xE0, 0xD8, 0xEE),
                c(0x5E, 0x45, 0x82),
                c(0xD9, 0xCC, 0xEC),
                c(0x5E, 0x45, 0x82),
                c(0xD9, 0xCC, 0xEC),
                c(0xD3, 0xC9, 0xE2),
                c(0x7B, 0x5A, 0xA8),
            ),
            (ThemeMode::Violet, true) => (
                c(0x17, 0x14, 0x1D),
                c(0x22, 0x1C, 0x2E),
                c(0x2D, 0x24, 0x40),
                c(0x37, 0x2D, 0x50),
                c(0xC9, 0xB6, 0xE8),
                c(0x17, 0x14, 0x1D),
                c(0x3D, 0x2E, 0x5C),
                c(0xA9, 0x94, 0xC4),
                c(0x3D, 0x2E, 0x5C),
                c(0xD9, 0xCC, 0xEC),
                c(0x3D, 0x2E, 0x5C),
                c(0x2E, 0x26, 0x3D),
                c(0xA9, 0x94, 0xC4),
            ),
        };

    let fg = pick(c(0x33, 0x33, 0x3B), c(0xEE, 0xEE, 0xF2));
    PaletteSpec {
        dark,
        background: bg,
        foreground: fg,
        card,
        card_fg: fg,
        popover: pop,
        popover_fg: fg,
        primary: pri,
        primary_fg: pri_fg,
        secondary: sec,
        secondary_fg: sec_fg,
        muted: mut_,
        muted_fg: acc_fg,
        accent: acc,
        accent_fg: acc_fg,
        destructive: des,
        destructive_fg: c(0xFF, 0xFF, 0xFF),
        border,
        input: border,
        ring,
    }
}

fn make_theme(name: &str, mode: ThemeMode, dark: bool) -> Theme {
    let p = palette_spec(mode, dark);
    let idx = match mode {
        ThemeMode::Zinc => 0u8,
        ThemeMode::Slate => 1,
        ThemeMode::Stone => 2,
        ThemeMode::Rose => 3,
        ThemeMode::Blue => 4,
        ThemeMode::Green => 5,
        ThemeMode::Violet => 6,
    };
    let code = (idx * 2 + if dark { 1 } else { 0 }) as f32 / 14.0;
    Theme::custom(
        name.to_string(),
        iced::theme::Palette {
            background: p.background,
            text: p.foreground,
            primary: p.primary,
            success: Color {
                r: code,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            danger: p.destructive,
        },
    )
}

pub fn theme_for(mode: ThemeMode, dark: bool) -> Theme {
    let base = match mode {
        ThemeMode::Zinc => "Zinc",
        ThemeMode::Slate => "Slate",
        ThemeMode::Stone => "Stone",
        ThemeMode::Rose => "Rose",
        ThemeMode::Blue => "Blue",
        ThemeMode::Green => "Green",
        ThemeMode::Violet => "Violet",
    };
    let suffix = if dark { " Dark" } else { " Light" };
    make_theme(&format!("ClipVault {base}{suffix}"), mode, dark)
}

fn is_dark(theme: &Theme) -> bool {
    spec(theme).dark
}

fn role(theme: &Theme, light: Color, dark: Color) -> Color {
    if is_dark(theme) { dark } else { light }
}

fn foreground(theme: &Theme) -> Color {
    spec(theme).foreground
}
fn muted_foreground(theme: &Theme) -> Color {
    spec(theme).muted_fg
}
fn border_color(theme: &Theme) -> Color {
    spec(theme).border
}
fn input_color(theme: &Theme) -> Color {
    spec(theme).input
}
fn ring_color(theme: &Theme) -> Color {
    spec(theme).ring
}
fn surface(theme: &Theme) -> Color {
    spec(theme).background
}
fn card_color(theme: &Theme) -> Color {
    spec(theme).card
}
fn secondary(theme: &Theme) -> Color {
    spec(theme).secondary
}
fn accent(theme: &Theme) -> Color {
    spec(theme).accent
}
fn primary(theme: &Theme) -> Color {
    spec(theme).primary
}
fn primary_fg(theme: &Theme) -> Color {
    spec(theme).primary_fg
}
fn destructive(theme: &Theme) -> Color {
    spec(theme).destructive
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

fn text_on(bg: Color, theme: &Theme) -> Color {
    if bg.a < 0.85 {
        return foreground(theme);
    }
    if luminance(bg) < 0.5 {
        c(0xF7, 0xF5, 0xF2)
    } else {
        c(0x2E, 0x2E, 0x36)
    }
}

fn flat_border(theme: &Theme, radius: f32) -> Border {
    Border {
        radius: sp::r(radius),
        width: 1.0,
        color: border_color(theme),
    }
}

pub fn ambient_background(theme: &Theme) -> Background {
    Background::Color(surface(theme))
}

/// shadcn Card: CARD bg, 1px BORDER, radius-lg 8px, small shadow.
pub fn card_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(role(theme, ZINC_CARD_FG_LIGHT, ZINC_CARD_FG_DARK)),
        background: Some(Background::Color(card_color(theme))),
        border: flat_border(theme, sp::RADIUS_LG),
        shadow: Shadow {
            color: ZINC_SHADOW_SM,
            offset: Vector::new(0.0, 1.0),
            blur_radius: 2.0,
        },
    }
}

/// Flat panel (sidebar / search / toolbar): same Card surface, no shadow.
pub fn panel_style(theme: &Theme) -> container::Style {
    let mut s = card_style(theme);
    s.shadow = Shadow::default();
    s
}

/// Standalone surface (preview / status): Card with the medium shadow.
pub fn glass_surface(theme: &Theme) -> container::Style {
    let mut s = card_style(theme);
    s.shadow = Shadow {
        color: ZINC_SHADOW_MD,
        offset: Vector::new(0.0, 2.0),
        blur_radius: 8.0,
    };
    s
}

/// History row: hover -> ACCENT fill, selected -> MUTED, checked -> border RING.
/// No lift, no thick border — shadcn hover is a flat color change.
pub fn entry_card_style(
    theme: &Theme,
    hovered: bool,
    selected: bool,
    checked: bool,
) -> container::Style {
    let fill = if selected {
        secondary(theme)
    } else if hovered {
        accent(theme)
    } else {
        card_color(theme)
    };
    let border = if checked {
        Border {
            radius: sp::r(sp::RADIUS_LG),
            width: 1.0,
            color: ring_color(theme),
        }
    } else {
        flat_border(theme, sp::RADIUS_LG)
    };
    container::Style {
        text_color: Some(role(theme, ZINC_CARD_FG_LIGHT, ZINC_CARD_FG_DARK)),
        background: Some(Background::Color(fill)),
        border,
        shadow: Shadow::default(),
    }
}

/// shadcn Dialog Content: POPOVER bg, BORDER, radius-lg, large shadow.
pub fn dialog_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(role(theme, ZINC_POPOVER_FG_LIGHT, ZINC_POPOVER_FG_DARK)),
        background: Some(Background::Color(role(
            theme,
            ZINC_POPOVER_LIGHT,
            ZINC_POPOVER_DARK,
        ))),
        border: flat_border(theme, sp::RADIUS_LG),
        shadow: Shadow {
            color: ZINC_SHADOW_LG,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
    }
}

/// shadcn Dialog Overlay: black/80 scrim.
pub fn dialog_scrim_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ZINC_DIALOG_SCRIM)),
        ..Default::default()
    }
}

// ---- Button variants ------------------------------------------------------
// shadcn cva(): default | secondary | destructive | outline | ghost | link.
// Link is unused in this app; the other five are exposed below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

fn button_for(variant: ButtonVariant, theme: &Theme, status: button::Status) -> button::Style {
    let fg = foreground(theme);
    let muted = muted_foreground(theme);
    let accent_fill = accent(theme);
    let accent_fg = role(theme, ZINC_ACCENT_FG_LIGHT, ZINC_ACCENT_FG_DARK);
    let secondary_fill = secondary(theme);
    let animate = !crate::gui::motion::reduced_motion();
    let press = press_shadow(theme, status, animate);

    let (bg, text, border, shadow) = match variant {
        ButtonVariant::Default => {
            let base = primary(theme);
            let fill = match status {
                button::Status::Hovered => mix(base, fg, 0.10),
                button::Status::Pressed => mix(base, fg, 0.18),
                button::Status::Disabled => mix(base, surface(theme), 0.50),
                _ => base,
            };
            (fill, primary_fg(theme), fill, press)
        }
        ButtonVariant::Secondary => {
            let fill = match status {
                button::Status::Hovered => mix(secondary_fill, fg, 0.05),
                button::Status::Pressed => mix(secondary_fill, fg, 0.10),
                button::Status::Disabled => mix(secondary_fill, surface(theme), 0.50),
                _ => secondary_fill,
            };
            let t = role(theme, ZINC_SECONDARY_FG_LIGHT, ZINC_SECONDARY_FG_DARK);
            (fill, t, fill, press)
        }
        ButtonVariant::Destructive => {
            let base = destructive(theme);
            let fill = match status {
                button::Status::Hovered => mix(base, Color::BLACK, 0.10),
                button::Status::Pressed => mix(base, Color::BLACK, 0.18),
                button::Status::Disabled => mix(base, surface(theme), 0.50),
                _ => base,
            };
            (fill, ZINC_DESTRUCTIVE_FG, fill, press)
        }
        ButtonVariant::Outline => {
            let (fill, text) = match status {
                button::Status::Hovered | button::Status::Pressed => (accent_fill, accent_fg),
                button::Status::Disabled => (surface(theme), muted),
                _ => (card_color(theme), fg),
            };
            (fill, text, border_color(theme), press)
        }
        ButtonVariant::Ghost => {
            let (fill, text) = match status {
                button::Status::Hovered | button::Status::Pressed => (accent_fill, accent_fg),
                button::Status::Disabled => (surface(theme), muted),
                _ => (
                    Color {
                        a: 0.0,
                        ..surface(theme)
                    },
                    fg,
                ),
            };
            (
                fill,
                text,
                Color {
                    a: 0.0,
                    ..surface(theme)
                },
                Shadow::default(),
            )
        }
    };

    button::Style {
        background: if variant == ButtonVariant::Ghost && matches!(status, button::Status::Active) {
            None
        } else {
            Some(Background::Color(bg))
        },
        text_color: if status == button::Status::Disabled {
            mix(text, bg, 0.40)
        } else {
            text
        },
        border: Border {
            radius: sp::r(sp::RADIUS_MD),
            width: if variant == ButtonVariant::Outline {
                1.0
            } else {
                0.0
            },
            color: border,
        },
        shadow,
    }
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button_for(ButtonVariant::Default, theme, status)
}
pub fn secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button_for(ButtonVariant::Secondary, theme, status)
}
pub fn outline_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button_for(ButtonVariant::Outline, theme, status)
}
pub fn ghost_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button_for(ButtonVariant::Ghost, theme, status)
}
pub fn destructive_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button_for(ButtonVariant::Destructive, theme, status)
}

/// Toggle/ToggleGroup: active = default variant, inactive = outline. No lift.
pub fn active_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        if active {
            button_for(ButtonVariant::Default, theme, status)
        } else {
            button_for(ButtonVariant::Outline, theme, status)
        }
    }
}

/// Sidebar nav items look like real shadcn buttons: visible outline, card
/// fill, rounded-md, fixed height. Active = primary fill; hover = accent+border.
pub fn nav_hover_style(theme: &Theme, hovered: bool, active: bool) -> button::Style {
    let (fill, text, border_c) = if active {
        (primary(theme), primary_fg(theme), primary(theme))
    } else if hovered {
        (
            accent(theme),
            role(theme, ZINC_ACCENT_FG_LIGHT, ZINC_ACCENT_FG_DARK),
            border_color(theme),
        )
    } else {
        (card_color(theme), foreground(theme), border_color(theme))
    };
    button::Style {
        background: Some(Background::Color(fill)),
        text_color: text,
        border: Border {
            radius: sp::r(sp::RADIUS_MD),
            width: 1.0,
            color: border_c,
        },
        shadow: if active || hovered {
            press_shadow(
                theme,
                button::Status::Hovered,
                !crate::gui::motion::reduced_motion(),
            )
        } else {
            Shadow::default()
        },
    }
}

/// shadcn Input: BACKGROUND bg, INPUT border, RING border on focus.
pub fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let focused = matches!(status, text_input::Status::Focused);
    let border = if focused {
        ring_color(theme)
    } else {
        input_color(theme)
    };
    text_input::Style {
        background: Background::Color(card_color(theme)),
        border: Border {
            radius: sp::r(sp::RADIUS_MD),
            width: 1.0,
            color: border,
        },
        icon: muted_foreground(theme),
        placeholder: muted_foreground(theme),
        value: foreground(theme),
        selection: role(theme, ZINC_PRIMARY_LIGHT, ZINC_PRIMARY_DARK),
    }
}

/// shadcn Select trigger: BACKGROUND bg, INPUT border, RING when opened.
pub fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let border = if status == pick_list::Status::Opened {
        ring_color(theme)
    } else {
        input_color(theme)
    };
    pick_list::Style {
        text_color: foreground(theme),
        placeholder_color: muted_foreground(theme),
        handle_color: muted_foreground(theme),
        background: Background::Color(card_color(theme)),
        border: Border {
            radius: sp::r(sp::RADIUS_MD),
            width: 1.0,
            color: border,
        },
    }
}

/// shadcn Checkbox: 1px INPUT border; PRIMARY fill + PRIMARY_FG icon when checked.
pub fn checkbox_style(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let (is_checked, hovered) = match status {
        checkbox::Status::Active { is_checked } => (is_checked, false),
        checkbox::Status::Hovered { is_checked } => (is_checked, true),
        checkbox::Status::Disabled { is_checked } => (is_checked, false),
    };
    let (bg, icon, border) = if is_checked {
        (primary(theme), primary_fg(theme), primary(theme))
    } else {
        let b = if hovered {
            ring_color(theme)
        } else {
            input_color(theme)
        };
        (card_color(theme), foreground(theme), b)
    };
    checkbox::Style {
        background: Background::Color(bg),
        icon_color: icon,
        border: Border {
            radius: sp::r(sp::RADIUS_SM),
            width: 1.0,
            color: border,
        },
        text_color: Some(foreground(theme)),
    }
}

/// shadcn Badge: pill; default(primary) / secondary / outline by ContentKind.
pub fn badge_style(kind: ContentKind, theme: &Theme) -> container::Style {
    let bg = match kind {
        ContentKind::Text => primary(theme),
        ContentKind::Image => secondary(theme),
        ContentKind::Files => accent(theme),
    };
    container::Style {
        text_color: Some(text_on(bg, theme)),
        background: Some(Background::Color(bg)),
        border: Border {
            radius: sp::r(sp::RADIUS_PILL),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
    }
}

/// shadcn Separator: 1px BORDER rule, full width.
pub fn separator_style(theme: &Theme) -> rule::Style {
    rule::Style {
        color: border_color(theme),
        width: 1,
        radius: sp::r(0.0),
        fill_mode: rule::FillMode::Full,
    }
}

/// Scroll Area: transparent rails, BORDER-tinted pill scroller.
pub fn scrollable_style(theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let scroller = scrollable::Scroller {
        color: role(theme, ZINC_MUTED_LIGHT, ZINC_MUTED_DARK),
        border: Border {
            radius: sp::r(sp::RADIUS_PILL),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    };
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller,
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_themes_have_distinct_primaries() {
        let mut seen = std::collections::HashSet::new();
        for mode in ThemeMode::all() {
            let t = theme_for(*mode, false);
            let p = spec(&t);
            seen.insert((
                (p.primary.r * 255.0) as u32,
                (p.primary.g * 255.0) as u32,
                (p.primary.b * 255.0) as u32,
            ));
        }
        assert_eq!(seen.len(), 7);
    }

    #[test]
    fn dark_and_light_variants_differ() {
        for mode in ThemeMode::all() {
            let light = theme_for(*mode, false);
            let dark = theme_for(*mode, true);
            assert!(!is_dark(&light));
            assert!(is_dark(&dark));
            assert_ne!(spec(&light).background, spec(&dark).background);
        }
    }

    #[test]
    fn text_on_picks_legible_text() {
        let light = theme_for(ThemeMode::Zinc, false);
        let dark = theme_for(ThemeMode::Zinc, true);
        assert_eq!(text_on(spec(&light).primary, &light), c(0xF7, 0xF5, 0xF2));
        assert_eq!(text_on(spec(&dark).primary, &dark), c(0x2E, 0x2E, 0x36));
    }
}
