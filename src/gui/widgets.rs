//! Reusable styled widgets: rounded cards, badges, empty states, buttons.
//! v0.1.3 UI rework: bigger type (via `sp::*` tokens), roomier hit areas, and
//! message-driven hover via `hover_card` / `hover_button`.
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use crate::gui::theme;
use iced::widget::{Space, button, container, text};
use iced::{Element, Length, Theme};

/// A rounded card with soft border and subtle shadow.
pub fn card<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content)
        .padding(sp::pad(sp::CARD_PAD))
        .style(|t: &Theme| theme::card_style(t))
        .into()
}

/// A card whose style reacts to a hovered flag (message-driven lift animation).
pub fn hover_card<'a, M: 'a>(content: impl Into<Element<'a, M>>, hovered: bool) -> Element<'a, M> {
    container(content)
        .padding(sp::pad(sp::CARD_PAD))
        .style(move |t: &Theme| theme::card_hover_style(t, hovered))
        .into()
}

/// Small content-type indicator chip.
pub fn badge<'a, M: 'a>(label: &str, kind: ContentKind) -> Element<'a, M> {
    container(text(label.to_string()).size(sp::TEXT_CAPTION))
        .padding(sp::pad(sp::BADGE_PAD))
        .style(move |t: &Theme| theme::badge_style(kind, t))
        .into()
}

/// Carefully designed empty state with title + hint.
pub fn empty_state<'a, M: 'a>(title: &str, subtitle: &str) -> Element<'a, M> {
    container(
        iced::widget::column![
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

/// A prominent primary button.
pub fn icon_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::primary_button_style)
        .into()
}

/// A quiet, low-emphasis text button.
pub fn quiet_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::subtle_button_style)
        .into()
}

/// A toggle button whose active state is shown by color + border (no glyph).
pub fn state_button<'a, M: Clone + 'a>(label: &str, active: bool, msg: M) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(theme::active_button_style(active))
        .into()
}

/// Button with an explicit hover flag (message-driven hover animation).
/// `active` selects the INK fill; `hovered` lifts the shadow + thickens border.
pub fn hover_button<'a, M: Clone + 'a>(
    label: &str,
    hovered: bool,
    active: bool,
    msg: M,
) -> Element<'a, M> {
    button(text(label.to_string()).size(sp::TEXT_LABEL))
        .on_press(msg)
        .padding(sp::pad(sp::BUTTON_PAD))
        .style(move |t: &Theme, _status| theme::nav_hover_style(t, hovered, active))
        .into()
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
