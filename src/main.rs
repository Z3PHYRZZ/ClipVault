// SPDX-License-Identifier: MIT
// ClipVault — Windows clipboard manager. Entry point / integrator wiring.
// GUI subsystem: do NOT allocate a console window when launched (double-click,
// Start Menu, or startup). Without this, Windows spawns a cmd/conhost window
// beside the Iced window because the PE is linked as a CUI (console) app.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use clipvault::app::{AppState, Message};
use clipvault::gui::theme::theme_for;
use iced::{Size, Task};

fn update(state: &mut AppState, msg: Message) -> Task<Message> {
    clipvault::app::update::update(state, msg)
}

fn view(state: &AppState) -> iced::Element<'_, Message> {
    clipvault::app::view::view(state)
}

fn theme(state: &AppState) -> iced::Theme {
    theme_for(state.settings.theme)
}

fn subscription(state: &AppState) -> iced::Subscription<Message> {
    clipvault::app::subscription::subscription(state)
}

/// Build state + spawn workers via `run_with`. The supervisor already queues an
/// initial Search, so no extra command is needed here.
fn initialize() -> (AppState, Task<Message>) {
    match AppState::new() {
        Ok(state) => (state, Task::none()),
        Err(e) => {
            tracing::error!("worker init failed: {e}");
            // Fall back to a state whose workers are inert but the GUI still
            // opens, so the user sees an error rather than a silent crash.
            let (events_tx, events_rx) = flume::unbounded();
            let (db_tx, _db_rx) = flume::bounded(64);
            let _ = events_tx.send(clipvault::workers::WorkerEvent::Error(e));
            (
                AppState {
                    settings: clipvault::config::Settings::default(),
                    filter: clipvault::domain::SearchFilter::default(),
                    entries: Vec::new(),
                    total_count: 0,
                    selected: None,
                    preview_bytes: None,
                    show_settings: false,
                    show_clear_confirm: false,
                    show_about: false,
                    paused: false,
                    status: "Worker initialization failed".into(),
                    events_tx,
                    events_rx,
                    db_tx,
                    paused_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    handles: clipvault::app::state::WorkerHandles::empty(),
                },
                Task::none(),
            )
        }
    }
}

pub fn main() -> iced::Result {
    // Logging: to a file in the app data dir. Under the GUI subsystem there is
    // no console/stderr, so a console writer would silently drop every event.
    // We open the log once and share it through a Mutex-backed MakeWriter.
    let log_path = clipvault::config::log_path();
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(file) => {
            // Arc<File> implements MakeWriter via `&File: Write` — no custom
            // type, no per-event reopen, no extra dependency.
            let writer = std::sync::Arc::new(file);
            let _ = tracing_subscriber::fmt()
                .with_writer(writer)
                .with_ansi(false)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("clipvault=info")),
                )
                .try_init();
        }
        Err(e) => {
            // No log file available: keep the app running, just without logs.
            eprintln!("ClipVault: log unavailable: {e}");
        }
    }
    tracing::info!("ClipVault starting");

    // Single instance: if another ClipVault owns the named mutex, signal it to
    // show its window and exit — prevents two clipboard listeners / hotkey owners.
    // The guard is bound to `main`'s scope so it lives until the app exits.
    let _singleton = match clipvault::platform::singleton::try_acquire() {
        Ok(Some(guard)) => Some(guard),
        Ok(None) => {
            tracing::info!("another instance is running; signalling it and exiting");
            let _ = clipvault::platform::singleton::signal_existing();
            return Ok(());
        }
        Err(e) => {
            tracing::warn!("single-instance check failed (continuing): {e}");
            None
        }
    };

    iced::application("ClipVault", update, view)
        .theme(theme)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: Size::new(1100.0, 720.0),
            min_size: Some(Size::new(820.0, 560.0)),
            ..Default::default()
        })
        .run_with(initialize)
}
