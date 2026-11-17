//! Root view composition: sidebar | (search + list | preview), plus modals.
//! shadcn port: flat Card surfaces; dialogs get a black/80 scrim over an opaque
//! base so clicks never leak through the stack. Native backdrop branch preserved.
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::theme;
use iced::widget::{Space, column, container, opaque, row, stack};
use iced::{Element, Length, Theme};

/// Wrap a centered shadcn Dialog card with a scrim over an opaque base.
fn modal<'a>(base: Element<'a, Message>, dialog: Element<'a, Message>) -> Element<'a, Message> {
    let scrim = container(Space::with_width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::dialog_scrim_style);
    stack![
        opaque(base),
        iced::widget::mouse_area(scrim).on_press(Message::Noop),
        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ]
    .into()
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    let sidebar = crate::gui::views::sidebar::sidebar(state);

    let main: Element<'_, Message> = if state.show_settings {
        crate::gui::views::settings::settings_view(state)
    } else {
        let mut main_col = column![
            container(crate::gui::views::search_bar::search_bar(state))
                .padding(sp::pad(sp::PANEL_PAD))
                .style(theme::panel_style),
            Space::with_height(sp::GAP_TIGHT),
        ];
        if !state.selected_ids.is_empty() {
            main_col = main_col.push(crate::gui::views::history::selection_toolbar(state));
            main_col = main_col.push(Space::with_height(sp::GAP_TIGHT));
        }
        main_col = main_col
            .push(
                row![
                    container(crate::gui::views::history::history_list(state))
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                    crate::gui::views::preview::preview_panel(state),
                ]
                .spacing(sp::GAP_TIGHT),
            )
            .push(
                container(
                    iced::widget::row![
                        iced::widget::text(&state.status).size(sp::TEXT_CAPTION),
                        iced::widget::text("|").size(sp::TEXT_CAPTION),
                        iced::widget::text(format!("{} entries", state.total_count))
                            .size(sp::TEXT_CAPTION),
                        iced::widget::Space::with_width(Length::Fill),
                        crate::gui::widgets::quiet_button(
                            if state.settings.list_density == crate::domain::ListDensity::Compact {
                                "Comfortable"
                            } else {
                                "Compact"
                            },
                            Message::SetListDensity(
                                if state.settings.list_density
                                    == crate::domain::ListDensity::Compact
                                {
                                    crate::domain::ListDensity::Comfortable
                                } else {
                                    crate::domain::ListDensity::Compact
                                }
                            )
                        ),
                        crate::gui::widgets::destructive_button(
                            "Clear all",
                            Message::RequestClearAll
                        ),
                        crate::gui::widgets::quiet_button("Prev", Message::PrevPage),
                        crate::gui::widgets::quiet_button("Next", Message::NextPage),
                    ]
                    .spacing(sp::GAP_TIGHT)
                    .align_y(iced::Alignment::Center),
                )
                .padding(sp::pad(sp::STATUS_PAD))
                .style(theme::glass_surface),
            );
        main_col.spacing(sp::GAP_TIGHT).into()
    };

    // Paint the flat zinc surface always so light = true #FFFFFF and dark =
    // true #09090B. The native backdrop remains applied on the HWND, but no
    // longer tints the content through a transparent root (that made dark look
    // gray and light look silver).
    let root_style = |t: &Theme| iced::widget::container::Style {
        background: Some(theme::ambient_background(t)),
        ..Default::default()
    };

    let base =
        container(row![container(sidebar).style(theme::panel_style), main,].spacing(sp::GAP_TIGHT))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(sp::SM)
            .style(root_style);

    // Modal overlays: precedence delete > clear > about.
    if state.show_delete_selected_confirm {
        return modal(
            base.into(),
            crate::gui::views::dialogs::confirm_delete_selected(state),
        );
    }
    if state.show_clear_confirm {
        return modal(
            base.into(),
            crate::gui::views::dialogs::confirm_clear(state),
        );
    }
    if state.show_about {
        return modal(base.into(), crate::gui::views::dialogs::about(state));
    }
    base.into()
}
