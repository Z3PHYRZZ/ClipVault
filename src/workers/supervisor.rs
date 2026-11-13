//! Spawns and owns every background thread; builds the initial AppState.
use crate::app::state::{AppState, WorkerHandles};
use crate::config::Settings;
use crate::domain::SearchFilter;
use crate::platform;
use crate::workers::{DbCommand, WorkerEvent, db_worker};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

/// Create channels, spawn workers, and return a fully wired AppState.
pub fn init() -> Result<AppState, String> {
    let settings = Settings::load();
    let (events_tx, events_rx) = flume::unbounded::<WorkerEvent>();
    let (db_tx, db_rx) = flume::bounded::<DbCommand>(64);

    // DB worker owns the single connection.
    let db_path = Settings::db_path();
    let ev = events_tx.clone();
    let db_thread = thread::Builder::new()
        .name("clipvault-db".into())
        .spawn(move || db_worker::run(db_path, db_rx, ev))
        .map_err(|e| format!("spawn db worker: {e}"))?;

    let paused_flag = Arc::new(AtomicBool::new(settings.pause_monitoring));

    // Platform subsystems (clipboard listener, hotkeys, tray) are cfg-gated.
    let hotkey_spec = platform::HotkeySpec::parse(&settings.show_hotkey);
    let handles = WorkerHandles {
        db: Some(db_thread),
        listener: platform::start_listener(events_tx.clone(), paused_flag.clone()).ok(),
        hotkeys: platform::start_hotkeys(events_tx.clone(), hotkey_spec).ok(),
        tray: platform::start_tray(events_tx.clone()).ok(),
    };

    // Kick off an initial search so the list is populated on first paint.
    let _ = db_tx.send(DbCommand::Search(SearchFilter::default()));

    Ok(AppState {
        settings,
        filter: SearchFilter::default(),
        entries: Vec::new(),
        total_count: 0,
        selected: None,
        preview_bytes: None,
        show_settings: false,
        show_clear_confirm: false,
        show_about: false,
        paused: paused_flag.load(Ordering::SeqCst),
        status: "Ready".into(),
        events_tx,
        events_rx,
        db_tx,
        paused_flag,
        handles,
    })
}

/// Re-export for cfg-gated code that toggles monitoring.
pub fn set_paused(flag: &Arc<AtomicBool>, paused: bool) {
    flag.store(paused, Ordering::SeqCst);
}
