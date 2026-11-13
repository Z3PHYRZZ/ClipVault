//! Settings view: appearance, storage, privacy, startup, shortcuts.
use crate::app::{AppState, Message};
use crate::config::ThemeMode;
use crate::gui::spacing as sp;
use crate::gui::widgets;
use iced::widget::{
    Space, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Element, Length};

pub fn settings_view(state: &AppState) -> Element<'_, Message> {
    let s = &state.settings;
    let content = column![
        text("Settings").size(sp::TEXT_DISPLAY),
        Space::with_height(sp::GAP_SECTION),
        widgets::section_title("APPEARANCE"),
        widgets::label("Choose light or complementary dark theme."),
        row![
            text("Theme").width(Length::Fill),
            pick_list(
                vec![ThemeMode::Light, ThemeMode::Dark],
                Some(s.theme),
                Message::SetTheme,
            )
            .padding(sp::INPUT_PAD),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
        Space::with_height(sp::GAP_SECTION),
        widgets::section_title("STORAGE & RETENTION"),
        widgets::label("History is stored locally in SQLite."),
        row![
            text("Max entries").width(Length::Fill),
            text(format!("{}", s.max_entries)).size(sp::TEXT_LABEL),
        ]
        .spacing(sp::GAP_TIGHT),
        row![
            text("Max age (days)").width(Length::Fill),
            text(format!("{}", s.max_age_days)).size(sp::TEXT_LABEL),
        ]
        .spacing(sp::GAP_TIGHT),
        widgets::label("Retention prunes oldest non-pinned, non-favorite entries."),
        Space::with_height(sp::GAP_SECTION),
        widgets::section_title("PRIVACY"),
        widgets::label("Control what ClipVault keeps."),
        checkbox("Pause clipboard monitoring", s.pause_monitoring)
            .on_toggle(Message::SetPauseMonitoring),
        checkbox("Clear history on exit", s.clear_on_exit).on_toggle(Message::SetClearOnExit),
        Space::with_height(sp::GAP_SECTION),
        widgets::section_title("STARTUP & SHORTCUTS"),
        widgets::label("Shortcuts work from any application."),
        checkbox("Launch at Windows startup", s.launch_at_startup)
            .on_toggle(Message::SetLaunchAtStartup),
        checkbox("Start minimized to tray", s.start_minimized)
            .on_toggle(Message::SetStartMinimized),
        row![
            text("Global shortcut").width(Length::Fill),
            text_input(&s.show_hotkey, &s.show_hotkey)
                .on_input(|v| Message::RebindShortcut(crate::domain::ShortcutAction::ShowHide, v))
                .padding(sp::INPUT_PAD)
                .width(180.0),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
        widgets::label("Format: Ctrl+Shift+V (Win32 global hotkey)."),
        Space::with_height(sp::GAP_SECTION),
        widgets::icon_button("Done", Message::CloseSettings),
    ]
    .spacing(sp::GAP_SECTION)
    .padding(sp::CONTENT_PAD);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
