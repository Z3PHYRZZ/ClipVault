//! Settings view: appearance, storage, privacy, startup, shortcuts.
//! shadcn port: Cards/sections with Separators, Select triggers, Inputs,
//! Checkboxes, ghost/default buttons.
use crate::app::{AppState, Message};
use crate::gui::spacing as sp;
use crate::gui::theme;
use crate::gui::widgets;
use iced::widget::{Space, column, combo_box, container, row, scrollable, text, text_input};
use iced::{Element, Length, Theme};

pub fn settings_view(state: &AppState) -> Element<'_, Message> {
    let s = &state.settings;
    let content = column![
        widgets::display("Settings"),
        Space::with_height(sp::GAP_SECTION),
        widgets::section_title("APPEARANCE"),
        widgets::label("Choose a color theme and light/dark variant."),
        row![
            text("Color theme").width(Length::Fill),
            combo_box(
                &state.theme_combo,
                "Choose theme",
                Some(&s.theme),
                Message::SetTheme,
            )
            .width(sp::SETTINGS_INPUT_W)
            .input_style(theme::input_style)
            .menu_style(theme::menu_style),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
        widgets::checkbox("Dark mode", s.dark_mode, Message::SetDarkMode),
        widgets::separator(),
        widgets::section_title("STORAGE & RETENTION"),
        widgets::label("History is stored locally in SQLite."),
        row![
            text("Max entries").width(Length::Fill),
            text(format!("{}", s.max_entries)).size(sp::TEXT_LABEL),
        ],
        row![
            text("Max age (days)").width(Length::Fill),
            text(format!("{}", s.max_age_days)).size(sp::TEXT_LABEL),
        ],
        widgets::label("Per-type caps (0 = use global max). Pinned/favorites are kept."),
        row![
            text("Text max").width(Length::Fill),
            text_input("0", &s.max_entries_text.to_string())
                .on_input(|v| Message::SetMaxEntriesKind(
                    crate::domain::ContentKind::Text,
                    v.parse().unwrap_or(0)
                ))
                .padding(sp::INPUT_PAD)
                .style(|t: &Theme, st| theme::input_style(t, st))
                .width(sp::SETTINGS_INPUT_W),
            text("Image max").width(Length::FillPortion(1)),
            text_input("0", &s.max_entries_image.to_string())
                .on_input(|v| Message::SetMaxEntriesKind(
                    crate::domain::ContentKind::Image,
                    v.parse().unwrap_or(0)
                ))
                .padding(sp::INPUT_PAD)
                .style(|t: &Theme, st| theme::input_style(t, st))
                .width(sp::SETTINGS_INPUT_W),
            text("Files max").width(Length::FillPortion(1)),
            text_input("0", &s.max_entries_files.to_string())
                .on_input(|v| Message::SetMaxEntriesKind(
                    crate::domain::ContentKind::Files,
                    v.parse().unwrap_or(0)
                ))
                .padding(sp::INPUT_PAD)
                .style(|t: &Theme, st| theme::input_style(t, st))
                .width(sp::SETTINGS_INPUT_W),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
        widgets::label("Retention prunes oldest non-pinned, non-favorite entries."),
        widgets::separator(),
        widgets::section_title("APPEARANCE & DATA"),
        widgets::label("Row density and a portable offline backup of your history."),
        row![
            text("List density").width(Length::Fill),
            combo_box(
                &state.density_combo,
                "Choose density",
                Some(&s.list_density),
                Message::SetListDensity,
            )
            .width(sp::SETTINGS_INPUT_W)
            .input_style(theme::input_style)
            .menu_style(theme::menu_style),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
        row![
            widgets::quiet_button("Export history", Message::ExportHistory),
            widgets::quiet_button("Import history", Message::ImportHistory),
        ]
        .spacing(sp::GAP_TIGHT),
        widgets::label("Backup: %LOCALAPPDATA%\\ClipVault\\clipvault-export.json"),
        widgets::separator(),
        widgets::section_title("PRIVACY"),
        widgets::label("Control what ClipVault keeps."),
        widgets::checkbox(
            "Pause clipboard monitoring",
            s.pause_monitoring,
            Message::SetPauseMonitoring
        ),
        widgets::checkbox(
            "Clear history on exit",
            s.clear_on_exit,
            Message::SetClearOnExit
        ),
        widgets::separator(),
        widgets::section_title("STARTUP & SHORTCUTS"),
        widgets::label("Shortcuts work from any application."),
        widgets::checkbox(
            "Launch at Windows startup",
            s.launch_at_startup,
            Message::SetLaunchAtStartup
        ),
        widgets::checkbox(
            "Start minimized to tray",
            s.start_minimized,
            Message::SetStartMinimized
        ),
        row![
            text("Global shortcut").width(Length::Fill),
            text_input(&s.show_hotkey, &s.show_hotkey)
                .on_input(|v| Message::RebindShortcut(crate::domain::ShortcutAction::ShowHide, v))
                .padding(sp::INPUT_PAD)
                .style(|t: &Theme, st| theme::input_style(t, st))
                .width(sp::HOTKEY_INPUT_W),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
        widgets::label("Format: Ctrl+Shift+V (Win32 global hotkey)."),
        Space::with_height(sp::GAP_SECTION),
        widgets::icon_button("Done", Message::CloseSettings),
    ]
    .spacing(sp::GAP_SECTION)
    .padding(sp::CONTENT_PAD);

    container(scrollable(content).style(theme::scrollable_style))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
