//! Typed messages crossing thread boundaries.
use crate::domain::{ClipEntry, FileMetadata, SearchFilter, ShortcutAction};

/// A clipboard payload captured from the OS.
#[derive(Debug, Clone)]
// Worker-bus messages are heterogeneous by design (small events next to
// payloads / DB rows); boxing every variant would add churn with no real win.
#[allow(clippy::large_enum_variant)]
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

/// A captured payload plus its provenance:
///
/// - `source_app`: the process that owned the clipboard (e.g. `chrome.exe`).
/// - `source_url`: the page URL when the copy came from a browser (CF_HTML
///   `SourceURL:` header, or a bare http(s) URL in the copied text).
///
/// Both are best-effort and may be None (clipboard owner can be short-lived).
#[derive(Debug, Clone)]
pub struct ClipCapture {
    pub payload: ClipPayload,
    pub source_app: Option<String>,
    pub source_url: Option<String>,
}

impl ClipCapture {
    pub fn new(payload: ClipPayload) -> Self {
        Self {
            payload,
            source_app: None,
            source_url: None,
        }
    }

    pub fn with_source(mut self, app: Option<String>, url: Option<String>) -> Self {
        self.source_app = app;
        self.source_url = url;
        self
    }
}

impl From<ClipPayload> for ClipCapture {
    fn from(payload: ClipPayload) -> Self {
        Self::new(payload)
    }
}

/// Events produced by workers, consumed by the GUI subscription.
#[derive(Debug, Clone)]
// Worker-bus messages are heterogeneous by design (small events next to
// payloads / DB rows); boxing every variant would add churn with no real win.
#[allow(clippy::large_enum_variant)]
pub enum WorkerEvent {
    NewClipboard(Box<ClipCapture>),
    Db(DbResult),
    Hotkey(ShortcutAction),
    Tray(TrayAction),
    Error(String),
}

/// Results from the database worker (matched by context in `update`).
#[derive(Debug, Clone)]
// Worker-bus messages are heterogeneous by design (small events next to
// payloads / DB rows); boxing every variant would add churn with no real win.
#[allow(clippy::large_enum_variant)]
pub enum DbResult {
    Entries(Vec<ClipEntry>),
    Entry(Option<ClipEntry>),
    Count(usize),
    Done(String),
    Error(String),
}

/// Commands sent from the GUI to the database worker.
#[derive(Debug, Clone)]
// Worker-bus messages are heterogeneous by design (small events next to
// payloads / DB rows); boxing every variant would add churn with no real win.
#[allow(clippy::large_enum_variant)]
pub enum DbCommand {
    Insert(Box<ClipCapture>),
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
