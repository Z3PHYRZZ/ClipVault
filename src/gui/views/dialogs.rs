//! Modal dialogs (clear-all confirmation, about).
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::widgets;
use iced::widget::{Space, column, container, text};
use iced::{Element, Length};

pub fn confirm_clear(_state: &AppState) -> Element<'_, Message> {
    let card = widgets::card(
        column![
            text("Clear all history?").size(sp::TEXT_TITLE),
            Space::with_height(sp::GAP_SECTION),
            text("Pinned and favorite entries are kept. This cannot be undone.")
                .size(sp::TEXT_LABEL),
            Space::with_height(sp::GAP_SECTION),
            iced::widget::row![
                widgets::quiet_button("Cancel", Message::CancelClearAll),
                widgets::icon_button("Clear all", Message::ConfirmClearAll),
            ]
            .spacing(sp::GAP_TIGHT),
        ]
        .spacing(sp::GAP_TIGHT),
    );
    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn confirm_delete_selected(state: &AppState) -> Element<'_, Message> {
    let n = state.selected_ids.len();
    let card = widgets::card(
        column![
            text(format!("Delete {n} entries?")).size(sp::TEXT_TITLE),
            Space::with_height(sp::GAP_SECTION),
            text("Pinned and favorite entries are included. This cannot be undone.")
                .size(sp::TEXT_LABEL),
            Space::with_height(sp::GAP_SECTION),
            iced::widget::row![
                widgets::quiet_button("Cancel", Message::ClearSelection),
                widgets::icon_button("Delete selected", Message::ConfirmDeleteSelected),
            ]
            .spacing(sp::GAP_TIGHT),
        ]
        .spacing(sp::GAP_TIGHT),
    );
    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn about(_state: &AppState) -> Element<'_, Message> {
    let card = widgets::card(
        column![
            text("ClipVault").size(sp::TEXT_TITLE),
            text("A calm, private Windows clipboard manager.").size(sp::TEXT_LABEL),
            Space::with_height(sp::GAP_SECTION),
            text("Local SQLite storage - No network - No telemetry").size(sp::TEXT_CAPTION),
            Space::with_height(sp::GAP_SECTION),
            widgets::quiet_button("Close", Message::ToggleAbout),
        ]
        .spacing(sp::GAP_TIGHT),
    );
    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
