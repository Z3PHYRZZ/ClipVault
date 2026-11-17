//! Scrollable clipboard history list with per-entry actions + bulk selection.
//! shadcn port: Card-like row surfaces, Badge chips, ghost/outline/toggle/delete
//! buttons, styled Checkbox. Hover/selected/checked fold into entry_card_style.
use crate::app::{AppState, Message};
use crate::domain::{ContentKind, ListDensity};
use crate::gui::spacing as sp;
use crate::gui::theme;
use crate::gui::widgets;
use iced::widget::{Space, column, container, image, mouse_area, row, scrollable, text};
use iced::{Element, Length, Theme};

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
            widgets::destructive_button("Delete", Message::RequestDeleteSelected),
            widgets::quiet_button("Clear", Message::ClearSelection),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
    )
    .padding(sp::pad(sp::STATUS_PAD))
    .width(Length::Fill)
    .style(theme::panel_style)
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
    let thumb_h: u16 = if compact {
        sp::THUMB_H_COMPACT
    } else {
        sp::THUMB_H
    };

    let mut list = column![].spacing(sp::GAP_TIGHT);
    for entry in &state.entries {
        let selected = state.selected == Some(entry.id);
        let hovered = state.hovered_card == Some(entry.id);
        let checked = state.selected_ids.contains(&entry.id);
        let id = entry.id;
        let pinned = entry.pinned;
        let favorite = entry.favorite;

        // Checkbox lives OUTSIDE the row's mouse_area so a selection click never
        // races the card's SelectEntry press (race fix from v0.1.4).
        let check: Element<'_, Message> =
            widgets::checkbox("", checked, move |_| Message::ToggleSelect(id));

        let leading: Element<'_, Message> = if entry.kind == ContentKind::Image {
            if let Some(path) = entry.image_path.as_deref() {
                image(image::Handle::from_path(std::path::PathBuf::from(path)))
                    .width(sp::THUMB_W)
                    .height(thumb_h)
                    .into()
            } else {
                widgets::badge(entry.short_kind(), entry.kind)
            }
        } else {
            widgets::badge(entry.short_kind(), entry.kind)
        };

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
            provenance.push_str(&url.chars().take(sp::URL_DISPLAY_CHARS).collect::<String>());
        }
        if !provenance.is_empty() {
            subtitle = subtitle.push(text(provenance).size(sp::TEXT_CAPTION));
        }
        let tags = entry.tags();
        if !tags.is_empty() {
            let mut chips = row![].spacing(sp::XS);
            for t in tags.into_iter().take(sp::MAX_TAGS_SHOWN) {
                chips = chips.push(widgets::badge(&format!("#{t}"), entry.kind));
            }
            subtitle = subtitle.push(chips);
        }

        // Two-line card: text/subtitle get the full width on line 1; the
        // action row sits below so padded buttons cannot squeeze the text.
        let actions: Element<'_, Message> = row![
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
            widgets::destructive_button("Delete", Message::DeleteEntry(id)),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center)
        .into();

        let row_content: Element<'_, Message> = column![
            row![
                leading,
                Space::with_width(sp::GAP_TIGHT),
                column![text(entry.display_text()).size(sp::TEXT_BODY), subtitle]
                    .width(Length::Fill),
            ]
            .spacing(sp::GAP_TIGHT)
            .align_y(iced::Alignment::Center),
            actions,
        ]
        .spacing(sp::GAP_TIGHT)
        .into();

        let card = container(
            mouse_area(row_content)
                .on_enter(Message::HoverCard(Some(id)))
                .on_exit(Message::HoverCard(None))
                .on_press(Message::SelectEntry(id)),
        )
        .padding(sp::pad(row_pad))
        .width(Length::Fill)
        .style(move |t: &Theme| theme::entry_card_style(t, hovered, selected, checked));

        // Outer row: checkbox + card. The checkbox stays outside mouse_area.
        list = list.push(
            row![check, card]
                .spacing(sp::GAP_TIGHT)
                .align_y(iced::Alignment::Center),
        );
    }

    scrollable(container(list).padding(sp::LIST_PAD).width(Length::Fill))
        .height(Length::Fill)
        .style(theme::scrollable_style)
        .into()
}
