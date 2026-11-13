//! Scrollable clipboard history list with per-entry actions.
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::widgets;
use iced::widget::{Space, column, container, row, scrollable, text};
use iced::{Background, Border, Element, Length, Theme};

pub fn history_list(state: &AppState) -> Element<'_, Message> {
    if state.entries.is_empty() {
        return widgets::empty_state(
            "No clipboard entries yet",
            "Copy something - text, an image, or files - and it will appear here.",
        );
    }

    let mut list = column![].spacing(sp::GAP_TIGHT);
    for entry in &state.entries {
        let selected = state.selected == Some(entry.id);
        let id = entry.id;
        let pinned = entry.pinned;
        let favorite = entry.favorite;

        let row_content = row![
            widgets::badge(entry.short_kind(), entry.kind),
            Space::with_width(sp::GAP_TIGHT),
            column![
                text(entry.display_text()).size(sp::TEXT_BODY),
                text(format!("used {} times", entry.use_count)).size(sp::TEXT_CAPTION),
            ]
            .width(Length::Fill),
            widgets::state_button(
                if pinned { "Unpin" } else { "Pin" },
                pinned,
                Message::PinEntry(id, !pinned)
            ),
            widgets::state_button(
                if favorite { "Unfav" } else { "Fav" },
                favorite,
                Message::FavoriteEntry(id, !favorite)
            ),
            widgets::quiet_button("Copy", Message::RestoreEntry(id)),
            widgets::quiet_button("Delete", Message::DeleteEntry(id)),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center);

        let card =
            container(iced::widget::mouse_area(row_content).on_press(Message::SelectEntry(id)))
                .padding(sp::pad(sp::ROW_PAD))
                .width(Length::Fill)
                .style(move |t: &Theme| {
                    let mut s = crate::gui::theme::card_style(t);
                    if selected {
                        s.background = Some(Background::Color(crate::contracts::PINK));
                        s.border = Border {
                            radius: sp::r(sp::RADIUS_CARD),
                            width: 2.0,
                            color: crate::contracts::ROSE,
                        };
                    }
                    s
                });
        list = list.push(card);
    }

    scrollable(container(list).padding(sp::LIST_PAD).width(Length::Fill))
        .height(Length::Fill)
        .into()
}
