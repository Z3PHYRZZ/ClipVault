//! Left navigation sidebar.
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::theme;
use crate::gui::widgets;
use iced::widget::{Space, button, column, container};
use iced::{Element, Length, Theme};

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    let nav_item = |label_text: &str, active: bool, msg: Message| -> Element<'_, Message> {
        button(iced::widget::text(label_text.to_string()).size(sp::TEXT_BODY))
            .on_press(msg)
            .width(Length::Fill)
            .padding(sp::pad(sp::NAV_PAD))
            .style(move |t: &Theme, status| {
                let mut s = if active {
                    theme::primary_button_style(t, status)
                } else {
                    // Inactive nav items stay quiet: soft ROSE-panel surface,
                    // no filled button look.
                    let mut q = theme::subtle_button_style(t, status);
                    q.background = None;
                    q
                };
                s.border.radius = sp::r(sp::RADIUS_BUTTON);
                s
            })
            .into()
    };

    column![
        container(column![
            widgets::display("ClipVault"),
            Space::with_height(sp::GAP_TITLE),
            widgets::label("clipboard history"),
        ])
        .padding([sp::MD, sp::XS]),
        Space::with_height(sp::GAP_SECTION),
        nav_item(
            "All clips",
            !state.filter.pinned_only && !state.filter.favorites_only,
            Message::ClearFilters
        ),
        nav_item(
            "Pinned",
            state.filter.pinned_only,
            Message::TogglePinnedOnly(!state.filter.pinned_only)
        ),
        nav_item(
            "Favorites",
            state.filter.favorites_only,
            Message::ToggleFavoritesOnly(!state.filter.favorites_only)
        ),
        Space::with_height(Length::Fill),
        widgets::quiet_button("Settings", Message::OpenSettings),
        Space::with_height(sp::GAP_TIGHT),
        widgets::quiet_button("About", Message::ToggleAbout),
        Space::with_height(sp::GAP_TIGHT),
        widgets::quiet_button("Quit", Message::Quit),
    ]
    .spacing(sp::GAP_TIGHT)
    .padding(sp::SIDEBAR_PAD)
    .width(224.0)
    .into()
}
