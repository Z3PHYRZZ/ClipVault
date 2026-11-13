//! Scrollable clipboard history list with per-entry actions.
use crate::app::{AppState, Message};
use crate::contracts::{CREAM, INK};
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use crate::gui::theme;
use crate::gui::widgets;
use iced::widget::{Space, column, container, image, row, scrollable, text};
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
        let hovered = state.hovered_card == Some(entry.id);
        let id = entry.id;
        let pinned = entry.pinned;
        let favorite = entry.favorite;

        // Photo preview: Iced loads the PNG from disk lazily (no GUI-thread read).
        let leading: Element<'_, Message> = if entry.kind == ContentKind::Image {
            if let Some(path) = entry.image_path.as_deref() {
                image(image::Handle::from_path(std::path::PathBuf::from(path)))
                    .width(52)
                    .height(42)
                    .into()
            } else {
                widgets::badge(entry.short_kind(), entry.kind)
            }
        } else {
            widgets::badge(entry.short_kind(), entry.kind)
        };

        // Provenance line: app name and/or web source URL, when captured.
        let mut subtitle =
            column![text(format!("used {} times", entry.use_count)).size(sp::TEXT_CAPTION)]
                .spacing(sp::GAP_TIGHT);
        let mut provenance = String::new();
        if let Some(app) = entry.source_app.as_deref() {
            provenance.push_str("from ");
            provenance.push_str(app);
        }
        if let Some(url) = entry.source_url.as_deref() {
            if !provenance.is_empty() {
                provenance.push_str(" - ");
            }
            let short = url.chars().take(60).collect::<String>();
            provenance.push_str(&short);
        }
        if !provenance.is_empty() {
            subtitle = subtitle.push(text(provenance).size(sp::TEXT_CAPTION));
        }

        let row_content = row![
            leading,
            Space::with_width(sp::GAP_TIGHT),
            column![text(entry.display_text()).size(sp::TEXT_BODY), subtitle].width(Length::Fill),
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

        let card = container(
            iced::widget::mouse_area(row_content)
                .on_enter(Message::HoverCard(Some(id)))
                .on_exit(Message::HoverCard(None))
                .on_press(Message::SelectEntry(id)),
        )
        .padding(sp::pad(sp::ROW_PAD))
        .width(Length::Fill)
        .style(move |t: &Theme| {
            let mut s = theme::card_hover_style(t, hovered);
            if selected {
                s.background = Some(Background::Color(CREAM));
                s.border = Border {
                    radius: sp::r(sp::RADIUS_CARD),
                    width: sp::HOVER_BORDER,
                    color: INK,
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
