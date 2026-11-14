//! The single message enum. Frozen — all GUI/worker events flow through here.
use crate::config::ThemeMode;
use crate::domain::{ContentKind, DateRange, ListDensity, ShortcutAction};

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
    /// Message-driven hover animation (mouse_area on_enter/on_exit).
    /// `HoverCard(Some(id))` = hovering a history row; None = pointer left.
    /// `HoverNav(Some(n))` = 0 All, 1 Pinned, 2 Favorites, 3 Settings,
    /// 4 About, 5 Quit; None = pointer left the sidebar item.
    HoverCard(Option<i64>),
    HoverNav(Option<u8>),
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
    /// v0.1.4 organization tools.
    SetTags(i64, Vec<String>),
    TagsDraftChanged(String),
    ApplyTags,
    FilterTag(Option<String>),
    ToggleSelect(i64),
    SelectAll,
    ClearSelection,
    RequestDeleteSelected,
    ConfirmDeleteSelected,
    PinSelected(bool),
    FavoriteSelected(bool),
    ExportHistory,
    ImportHistory,
    SetListDensity(ListDensity),
    SetMaxEntriesKind(ContentKind, usize),
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
    /// v0.1.6: apply the native Windows backdrop once the window exists, and
    /// record which DWM path succeeded (for re-apply on theme change + status).
    ApplyBackdrop,
    BackdropApplied(crate::platform::backdrop::BackdropKind),
}
