//! Scrollable clipboard history list with per-entry actions + bulk selection.
use crate::app::{AppState, Message};
use crate::contracts::{CREAM, INK};
use crate::domain::{ContentKind, ListDensity};
use crate::gui::spacing as sp;
use crate::gui::theme;
use crate::gui::widgets;
use iced::widget::{Space, checkbox, column, container, image, mouse_area, row, scrollable, text};
use iced::{Background, Border, Element, Length, Theme};

/// Selection toolbar shown above the list when at least one row is checked.
pub fn selection_toolbar(state: &AppState) -> Element<'_, Message> {
    let n = state.selected_ids.len();
    container(
        row![
            text(format!("{n} selected")).size(sp::TEXT_LABEL),
            Space::with_width(Length::Fill),
            widgets::quiet_button("Select all", Message::SelectAll),
            widgets::quiet_button("Pin", Message::PinSelected(true)),
            widgets::quiet_button("Favorite", Message::FavoriteSelected(true)),
            widgets::icon_button("Delete", Message::RequestDeleteSelected),
            widgets::quiet_button("Clear", Message::ClearSelection),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
    )
    .padding(sp::pad(sp::STATUS_PAD))
    .width(Length::Fill)
    .style(|t: &Theme| theme::panel_style(t))
    .into()
}

pub fn history_list(state: &AppState) -> Element<'_, Message> {
    if state.entries.is_empty() {
        return widgets::empty_state(
            "No clipboard entries yet",
            "Copy something - text, an image, or files - and it will appear here.",
        );
    }

    let compact = state.settings.list_density == ListDensity::Compact;
    let row_pad = if compact {
        sp::ROW_PAD_COMPACT
    } else {
        sp::ROW_PAD
    };
    let thumb_h: u16 = if compact { 30 } else { 42 };

    let mut list = column![].spacing(sp::GAP_TIGHT);
    for entry in &state.entries {
        let selected = state.selected == Some(entry.id);
        let hovered = state.hovered_card == Some(entry.id);
        let checked = state.selected_ids.contains(&entry.id);
        let id = entry.id;
        let pinned = entry.pinned;
        let favorite = entry.favorite;

        // Checkbox lives OUTSIDE the row's mouse_area so a selection click never
        // races the card's SelectEntry press.
        let check: Element<'_, Message> = checkbox("", checked)
            .on_toggle(move |_| Message::ToggleSelect(id))
            .size(18)
            .into();

        // Photo preview: Iced loads the PNG from disk lazily (no GUI-thread read).
        let leading: Element<'_, Message> = if entry.kind == ContentKind::Image {
            if let Some(path) = entry.image_path.as_deref() {
                image(image::Handle::from_path(std::path::PathBuf::from(path)))
                    .width(52)
                    .height(thumb_h)
                    .into()
            } else {
                widgets::badge(entry.short_kind(), entry.kind)
            }
        } else {
            widgets::badge(entry.short_kind(), entry.kind)
        };

        // Provenance + tag chips line.
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
            provenance.push_str(&url.chars().take(60).collect::<String>());
        }
        if !provenance.is_empty() {
            subtitle = subtitle.push(text(provenance).size(sp::TEXT_CAPTION));
        }
        let tags = entry.tags();
        if !tags.is_empty() {
            let mut chips = row![].spacing(sp::XS);
            for t in tags.into_iter().take(4) {
                chips = chips.push(widgets::badge(&format!("#{t}"), entry.kind));
            }
            subtitle = subtitle.push(chips);
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
            mouse_area(row_content)
                .on_enter(Message::HoverCard(Some(id)))
                .on_exit(Message::HoverCard(None))
                .on_press(Message::SelectEntry(id)),
        )
        .padding(sp::pad(row_pad))
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
            if checked {
                s.border = Border {
                    radius: sp::r(sp::RADIUS_CARD),
                    width: sp::HOVER_BORDER,
                    color: INK,
                };
            }
            s
        });

        // Outer row: checkbox + card. The checkbox is deliberately not inside
        // mouse_area so both interactions stay independent.
        list = list.push(
            row![check, card]
                .spacing(sp::GAP_TIGHT)
                .align_y(iced::Alignment::Center),
        );
    }

    scrollable(container(list).padding(sp::LIST_PAD).width(Length::Fill))
        .height(Length::Fill)
        .into()
}
