//! Owned application state. Integrator-owned; the fleet reads these fields.
use crate::config::Settings;
use crate::domain::ClipEntry;
use crate::domain::SearchFilter;
use crate::workers::{DbCommand, WorkerEvent};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Owns the shutdown handles of every background thread.
pub struct WorkerHandles {
    pub db: Option<std::thread::JoinHandle<()>>,
    pub listener: Option<crate::platform::ListenerHandle>,
    pub hotkeys: Option<crate::platform::HotkeyHandle>,
    pub tray: Option<crate::platform::TrayHandle>,
}

impl WorkerHandles {
    pub fn empty() -> Self {
        Self {
            db: None,
            listener: None,
            hotkeys: None,
            tray: None,
        }
    }

    /// Signal every subsystem to stop and join the DB worker last (WAL checkpoint).
    pub fn shutdown(&mut self, db_tx: &flume::Sender<DbCommand>) {
        if let Some(mut h) = self.listener.take() {
            h.shutdown();
        }
        if let Some(mut h) = self.hotkeys.take() {
            h.shutdown();
        }
        if let Some(mut h) = self.tray.take() {
            h.shutdown();
        }
        let _ = db_tx.send(DbCommand::Shutdown);
        if let Some(h) = self.db.take() {
            let _ = h.join();
        }
    }
}

pub struct AppState {
    pub settings: Settings,
    pub filter: SearchFilter,
    pub entries: Vec<ClipEntry>,
    pub total_count: usize,
    pub selected: Option<i64>,
    /// v0.1.4 bulk-selection set (row ids checked in the history list).
    pub selected_ids: std::collections::HashSet<i64>,
    pub show_delete_selected_confirm: bool,
    /// Draft comma-separated tags for the currently selected entry (tag editor).
    pub tags_input: String,
    pub preview_bytes: Option<Vec<u8>>,
    /// Explicit hover state (message-driven hover animation, not just button status).
    pub hovered_card: Option<i64>,
    pub hovered_nav: Option<u8>,
    /// Whether the main window is currently minimized (global show/hide hotkey).
    pub window_hidden: bool,
    pub show_settings: bool,
    pub show_clear_confirm: bool,
    pub show_about: bool,
    pub paused: bool,
    pub status: String,
    pub events_tx: flume::Sender<WorkerEvent>,
    pub events_rx: flume::Receiver<WorkerEvent>,
    pub db_tx: flume::Sender<DbCommand>,
    pub paused_flag: Arc<AtomicBool>,
    pub handles: WorkerHandles,
}

impl AppState {
    /// Build the state and spawn all workers. Called once from `run_with`.
    pub fn new() -> Result<Self, String> {
        crate::workers::supervisor::init()
    }
}
