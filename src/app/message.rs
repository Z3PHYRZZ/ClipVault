//! The single message enum. Frozen — all GUI/worker events flow through here.
use crate::config::ThemeMode;
use crate::domain::{ContentKind, DateRange, ShortcutAction};

// The single GUI message enum intentionally mixes tiny events with boxed worker
// events; boxing the worker variant already keeps the common path small.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    /// Worker -> GUI (clipboard/db/hotkey/tray/error).
    Worker(Box<crate::workers::WorkerEvent>),
    /// Search & filters.
    SearchChanged(String),
    FilterKind(Option<ContentKind>),
    FilterDate(DateRange),
    TogglePinnedOnly(bool),
    ToggleFavoritesOnly(bool),
    ClearFilters,
    /// Entry actions.
    SelectEntry(i64),
    PinEntry(i64, bool),
    FavoriteEntry(i64, bool),
    DeleteEntry(i64),
    RequestClearAll,
    ConfirmClearAll,
    CancelClearAll,
    RestoreEntry(i64),
    OpenPreviewInExplorer(i64),
    /// Paging.
    LoadMore,
    NextPage,
    PrevPage,
    /// Settings.
    OpenSettings,
    CloseSettings,
    SetTheme(ThemeMode),
    SetMaxEntries(usize),
    SetMaxAgeDays(u32),
    SetPauseMonitoring(bool),
    SetClearOnExit(bool),
    SetStartMinimized(bool),
    SetLaunchAtStartup(bool),
    RebindShortcut(ShortcutAction, String),
    /// Dialogs / window.
    ToggleAbout,
    Quit,
    /// Slow periodic retention tick.
    RetentionTick,
    ShowWindow,
    HideWindow,
}
