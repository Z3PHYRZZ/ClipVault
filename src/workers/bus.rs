//! Typed messages crossing thread boundaries.
use crate::domain::{ClipEntry, FileMetadata, SearchFilter, ShortcutAction};

/// A clipboard payload captured from the OS.
#[derive(Debug, Clone)]
pub enum ClipPayload {
    Text(String),
    Image {
        png: Vec<u8>,
        w: u32,
        h: u32,
        hash: String,
    },
    Files(Vec<FileMetadata>),
}

impl ClipPayload {
    pub fn kind(&self) -> crate::domain::ContentKind {
        match self {
            ClipPayload::Text(_) => crate::domain::ContentKind::Text,
            ClipPayload::Image { .. } => crate::domain::ContentKind::Image,
            ClipPayload::Files(_) => crate::domain::ContentKind::Files,
        }
    }
}

/// Events produced by workers, consumed by the GUI subscription.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    NewClipboard(ClipPayload),
    Db(DbResult),
    Hotkey(ShortcutAction),
    Tray(TrayAction),
    Error(String),
}

/// Results from the database worker (matched by context in `update`).
#[derive(Debug, Clone)]
pub enum DbResult {
    Entries(Vec<ClipEntry>),
    Entry(Option<ClipEntry>),
    Count(usize),
    Done(String),
    Error(String),
}

/// Commands sent from the GUI to the database worker.
#[derive(Debug, Clone)]
pub enum DbCommand {
    Insert(ClipPayload),
    Search(SearchFilter),
    Get(i64),
    SetPin(i64, bool),
    SetFavorite(i64, bool),
    Delete(i64),
    ClearAll,
    Prune {
        max_entries: usize,
        max_age_days: u32,
    },
    Shutdown,
}

/// Tray menu actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Show,
    Settings,
    Pause,
    Resume,
    Quit,
}
