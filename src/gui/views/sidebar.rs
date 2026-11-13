//! Left navigation sidebar with message-driven hover animation.
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::widgets;
use iced::widget::{Space, column, container, mouse_area};
use iced::{Element, Length};

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    // A nav/bottom item = hover_button wrapped in mouse_area so pointer
    // enter/exit flips AppState.hovered_nav, which recomputes the lift style.
    let nav_item = |label_text: &str, n: u8, active: bool, msg: Message| -> Element<'_, Message> {
        let hovered = state.hovered_nav == Some(n);
        let button = widgets::hover_button(label_text, hovered, active, msg);
        mouse_area(button)
            .on_enter(Message::HoverNav(Some(n)))
            .on_exit(Message::HoverNav(None))
            .into()
    };

    let all_active = !state.filter.pinned_only && !state.filter.favorites_only;

    column![
        container(column![
            widgets::display("ClipVault"),
            Space::with_height(sp::GAP_TITLE),
            widgets::label("clipboard history"),
        ])
        .padding([sp::MD, sp::XS]),
        Space::with_height(sp::GAP_SECTION),
        nav_item("All clips", 0, all_active, Message::ClearFilters),
        nav_item(
            "Pinned",
            1,
            state.filter.pinned_only,
            Message::TogglePinnedOnly(!state.filter.pinned_only)
        ),
        nav_item(
            "Favorites",
            2,
            state.filter.favorites_only,
            Message::ToggleFavoritesOnly(!state.filter.favorites_only)
        ),
        Space::with_height(Length::Fill),
        nav_item("Settings", 3, false, Message::OpenSettings),
        Space::with_height(sp::GAP_TIGHT),
        nav_item("About", 4, false, Message::ToggleAbout),
        Space::with_height(sp::GAP_TIGHT),
        nav_item("Quit", 5, false, Message::Quit),
    ]
    .spacing(sp::GAP_TIGHT)
    .padding(sp::SIDEBAR_PAD)
    .width(224.0)
    .into()
}
