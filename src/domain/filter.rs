//! Search filters and shortcut identifiers.
use crate::domain::ContentKind;

/// Date range presets for history filtering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DateRange {
    #[default]
    All,
    Today,
    Last7Days,
    Last30Days,
}

impl DateRange {
    /// Unix-seconds cutoff for `created_at >= ?`, if any.
    pub fn cutoff_unix(&self, now: i64) -> Option<i64> {
        match self {
            DateRange::All => None,
            DateRange::Today => Some(now - 86_400),
            DateRange::Last7Days => Some(now - 7 * 86_400),
            DateRange::Last30Days => Some(now - 30 * 86_400),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DateRange::All => "All time",
            DateRange::Today => "Today",
            DateRange::Last7Days => "Last 7 days",
            DateRange::Last30Days => "Last 30 days",
        }
    }

    pub fn all() -> &'static [DateRange] {
        &[
            DateRange::All,
            DateRange::Today,
            DateRange::Last7Days,
            DateRange::Last30Days,
        ]
    }
}

impl std::fmt::Display for DateRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Everything needed to build a paged search query.
#[derive(Debug, Clone)]
pub struct SearchFilter {
    pub query: String,
    pub kind: Option<ContentKind>,
    pub date: DateRange,
    pub pinned_only: bool,
    pub favorites_only: bool,
    /// When set, only entries whose `tags_json` contains this exact tag pass.
    pub tag: Option<String>,
    pub page: usize,
    pub page_size: usize,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            query: String::new(),
            kind: None,
            date: DateRange::All,
            pinned_only: false,
            favorites_only: false,
            tag: None,
            page: 0,
            page_size: crate::contracts::DEFAULT_PAGE_SIZE,
        }
    }
}

/// Row-density preference for the history list (persisted in Settings).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ListDensity {
    #[default]
    Comfortable,
    Compact,
}

impl ListDensity {
    pub fn label(self) -> &'static str {
        match self {
            ListDensity::Comfortable => "Comfortable",
            ListDensity::Compact => "Compact",
        }
    }
    pub fn all() -> &'static [ListDensity] {
        &[ListDensity::Comfortable, ListDensity::Compact]
    }
}

impl std::fmt::Display for ListDensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Actions bound to global shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    ShowHide,
    QuickPaste(u8),
}
