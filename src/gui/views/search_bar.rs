//! Search bar with content-type and date filters.
use crate::app::{AppState, Message};
use crate::domain::{ContentKind, DateRange};
use crate::gui::spacing as sp;
use crate::gui::widgets;
use iced::widget::{pick_list, row, text_input};
use iced::{Element, Length};

/// Kind filter choices rendered in the pick list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindChoice {
    All,
    Text,
    Image,
    Files,
}

impl KindChoice {
    pub fn label(self) -> &'static str {
        match self {
            KindChoice::All => "All types",
            KindChoice::Text => "Text",
            KindChoice::Image => "Image",
            KindChoice::Files => "Files",
        }
    }
    pub fn all() -> &'static [KindChoice] {
        &[
            KindChoice::All,
            KindChoice::Text,
            KindChoice::Image,
            KindChoice::Files,
        ]
    }
    pub fn to_kind(self) -> Option<ContentKind> {
        match self {
            KindChoice::All => None,
            KindChoice::Text => Some(ContentKind::Text),
            KindChoice::Image => Some(ContentKind::Image),
            KindChoice::Files => Some(ContentKind::Files),
        }
    }
    pub fn from_kind(k: Option<ContentKind>) -> Self {
        match k {
            None => KindChoice::All,
            Some(ContentKind::Text) => KindChoice::Text,
            Some(ContentKind::Image) => KindChoice::Image,
            Some(ContentKind::Files) => KindChoice::Files,
        }
    }
}

impl std::fmt::Display for KindChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// "All tags" plus every distinct tag on the current page (sorted, deduped).
fn tag_choices(state: &AppState) -> Vec<String> {
    let mut tags: Vec<String> = state.entries.iter().flat_map(|e| e.tags()).collect();
    tags.sort();
    tags.dedup();
    let mut choices = vec!["All tags".to_string()];
    choices.extend(tags);
    choices
}

pub fn search_bar(state: &AppState) -> Element<'_, Message> {
    row![
        text_input("Search history...", &state.filter.query)
            .on_input(Message::SearchChanged)
            .padding(sp::INPUT_PAD)
            .width(Length::FillPortion(3)),
        pick_list(
            KindChoice::all().to_vec(),
            Some(KindChoice::from_kind(state.filter.kind)),
            |c| Message::FilterKind(c.to_kind()),
        )
        .padding(sp::INPUT_PAD)
        .width(Length::FillPortion(1)),
        pick_list(
            DateRange::all().to_vec(),
            Some(state.filter.date),
            Message::FilterDate,
        )
        .padding(sp::INPUT_PAD)
        .width(Length::FillPortion(1)),
        // Tag filter: union of tags on the currently visible page; "All tags"
        // is the None sentinel. Keeps the picker bounded to what the user sees.
        pick_list(
            tag_choices(state),
            Some(
                state
                    .filter
                    .tag
                    .clone()
                    .unwrap_or_else(|| "All tags".into())
            ),
            |sel| Message::FilterTag(if sel == "All tags" { None } else { Some(sel) }),
        )
        .padding(sp::INPUT_PAD)
        .width(Length::FillPortion(1)),
        widgets::quiet_button("Clear", Message::ClearFilters),
    ]
    .spacing(sp::GAP_TIGHT)
    .align_y(iced::Alignment::Center)
    .into()
}
