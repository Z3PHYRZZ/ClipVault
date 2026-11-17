//! Reusable shadcn/ui-styled widget helpers.
//!
//! Thin wrappers over iced widgets that bind the theme functions from
//! `theme.rs` and the geometry tokens from `spacing.rs`. Views use these so no
//! color or radius magic values leak into layout code.
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use crate::gui::theme;
use iced::widget::{
    Space, button, checkbox as iced_checkbox, column, container, horizontal_rule, text,
};
use iced::{Element, Length, Theme};

/// shadcn Card: rounded surface wrapping arbitrary content.
pub fn card<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content)
        .padding(sp::pad(sp::CARD_PAD))
        .style(theme::card_style)
        .into()
}

/// Content-kind / tag chip (shadcn Badge, pill radius).
pub fn badge<'a, M: 'a>(label: &str, kind: ContentKind) -> Element<'a, M> {
    container(text(label.to_string()).size(sp::TEXT_CAPTION))
        .padding(sp::pad(sp::BADGE_PAD))
        .style(move |t: &Theme| theme::badge_style(kind, t))
        .into()
}

/// Centered title + subtitle empty state.
pub fn empty_state<'a, M: 'a>(title: &str, subtitle: &str) -> Element<'a, M> {
    container(
        column![
            text(title.to_string()).size(sp::TEXT_TITLE),
            Space::with_height(sp::GAP_TITLE),
            text(subtitle.to_string()).size(sp::TEXT_LABEL),
        ]
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding(sp::EMPTY_PAD)
    .center_x(Length::Fill)
    .into()
}

/// Primary/default button variant (prominent action).
pub fn icon_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::primary_button_style)
        .into()
}

/// Ghost button variant (low emphasis); name kept for existing call sites.
pub fn quiet_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::ghost_button_style)
        .into()
}

/// Secondary button variant.
pub fn secondary_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::secondary_button_style)
        .into()
}

/// Outline button variant.
pub fn outline_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::outline_button_style)
        .into()
}

/// Destructive button variant (delete / clear actions).
pub fn destructive_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::destructive_button_style)
        .into()
}

/// Toggle button: active = default visuals, inactive = outline.
pub fn state_button<'a, M: Clone + 'a>(label: &str, active: bool, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::active_button_style(active))
        .into()
}

/// Nav button with message-driven hovered/active flags (no iced Status).
pub fn hover_button<'a, M: Clone + 'a>(
    label: &str,
    hovered: bool,
    active: bool,
    msg: M,
) -> Element<'a, M> {
    button(
        text(label.to_string())
            .size(sp::TEXT_LABEL)
            .width(iced::Length::Fill),
    )
    .on_press(msg)
    .padding(sp::pad(sp::NAV_PAD))
    .width(iced::Length::Fill)
    .height(sp::CONTROL_H)
    .style(move |t: &Theme, _status| theme::nav_hover_style(t, hovered, active))
    .into()
}

/// Styled checkbox (was default iced styling before the shadcn port).
pub fn checkbox<'a, M: Clone + 'a>(
    label: &str,
    is_checked: bool,
    on_toggle: impl Fn(bool) -> M + 'a,
) -> Element<'a, M> {
    iced_checkbox(label.to_string(), is_checked)
        .on_toggle(on_toggle)
        .size(sp::CHECKBOX_SIZE)
        .style(theme::checkbox_style)
        .into()
}

/// shadcn Separator (horizontal hairline).
pub fn separator<'a, M: 'a>() -> Element<'a, M> {
    horizontal_rule(1).style(theme::separator_style).into()
}

pub fn section_title<'a, M: 'a>(s: &str) -> Element<'a, M> {
    text(s.to_string()).size(sp::TEXT_LABEL).into()
}

/// Small caption text for hints and secondary lines.
pub fn label<'a, M: 'a>(s: &str) -> Element<'a, M> {
    text(s.to_string()).size(sp::TEXT_CAPTION).into()
}

/// Large display text for the app / page title.
pub fn display<'a, M: 'a>(s: &str) -> Element<'a, M> {
    text(s.to_string()).size(sp::TEXT_DISPLAY).into()
}
