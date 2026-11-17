//! Modal dialogs (clear-all confirmation, delete-selected confirmation, about).
//! shadcn Dialog Content styling; scrim is applied by root view.rs.
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::theme;
use crate::gui::widgets;
use iced::widget::{Space, column, container, row, text};
use iced::{Element, Length};

fn dialog_card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(sp::pad(sp::CARD_PAD))
        .width(Length::Fixed(420.0))
        .style(theme::dialog_style)
        .into()
}

pub fn confirm_clear(_state: &AppState) -> Element<'_, Message> {
    dialog_card(column![
        text("Clear all history?").size(sp::TEXT_TITLE),
        Space::with_height(sp::GAP_SECTION),
        text("Pinned and favorite entries are kept. This cannot be undone.").size(sp::TEXT_LABEL),
        Space::with_height(sp::GAP_SECTION),
        row![
            widgets::quiet_button("Cancel", Message::CancelClearAll),
            widgets::destructive_button("Clear all", Message::ConfirmClearAll),
        ]
        .spacing(sp::GAP_TIGHT),
    ])
}

pub fn confirm_delete_selected(state: &AppState) -> Element<'_, Message> {
    let n = state.selected_ids.len();
    dialog_card(column![
        text(format!("Delete {n} entries?")).size(sp::TEXT_TITLE),
        Space::with_height(sp::GAP_SECTION),
        text("Pinned and favorite entries are included. This cannot be undone.")
            .size(sp::TEXT_LABEL),
        Space::with_height(sp::GAP_SECTION),
        row![
            widgets::quiet_button("Cancel", Message::ClearSelection),
            widgets::destructive_button("Delete selected", Message::ConfirmDeleteSelected),
        ]
        .spacing(sp::GAP_TIGHT),
    ])
}

pub fn about(_state: &AppState) -> Element<'_, Message> {
    dialog_card(column![
        text("ClipVault").size(sp::TEXT_TITLE),
        text("A calm, private Windows clipboard manager.").size(sp::TEXT_LABEL),
        Space::with_height(sp::GAP_SECTION),
        text("Local SQLite storage - No network - No telemetry").size(sp::TEXT_CAPTION),
        Space::with_height(sp::GAP_SECTION),
        widgets::quiet_button("Close", Message::ToggleAbout),
    ])
}
