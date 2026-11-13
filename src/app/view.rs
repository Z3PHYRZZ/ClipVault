//! Root view composition: sidebar | (search + list | preview), plus modals.
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::theme;
use iced::widget::{Space, column, container, row};
use iced::{Element, Length, Theme};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let sidebar = crate::gui::views::sidebar::sidebar(state);

    let main: Element<'_, Message> = if state.show_settings {
        crate::gui::views::settings::settings_view(state)
    } else {
        column![
            container(crate::gui::views::search_bar::search_bar(state))
                .padding(sp::pad(sp::PANEL_PAD))
                .style(|t: &Theme| theme::panel_style(t)),
            Space::with_height(sp::GAP_TIGHT),
            row![
                container(crate::gui::views::history::history_list(state))
                    .width(Length::FillPortion(3))
                    .height(Length::Fill),
                crate::gui::views::preview::preview_panel(state),
            ]
            .spacing(sp::GAP_TIGHT),
            container(
                iced::widget::row![
                    iced::widget::text(&state.status).size(sp::TEXT_CAPTION),
                    iced::widget::text(format!("{} entries", state.total_count))
                        .size(sp::TEXT_CAPTION),
                    iced::widget::Space::with_width(Length::Fill),
                    crate::gui::widgets::quiet_button("Clear all", Message::RequestClearAll),
                    crate::gui::widgets::quiet_button("Prev", Message::PrevPage),
                    crate::gui::widgets::quiet_button("Next", Message::NextPage),
                ]
                .spacing(sp::GAP_TIGHT)
                .align_y(iced::Alignment::Center)
            )
            .padding(sp::pad(sp::STATUS_PAD)),
        ]
        .spacing(sp::GAP_TIGHT)
        .into()
    };

    let base = container(row![
        container(sidebar).style(|t: &Theme| theme::panel_style(t)),
        main,
    ])
    .width(Length::Fill)
    .height(Length::Fill);

    // Modal overlays.
    if state.show_clear_confirm {
        return iced::widget::stack![base, crate::gui::views::dialogs::confirm_clear(state)].into();
    }
    if state.show_about {
        return iced::widget::stack![base, crate::gui::views::dialogs::about(state)].into();
    }
    base.into()
}
