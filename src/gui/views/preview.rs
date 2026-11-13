//! Right-hand preview panel: text, image, or file metadata.
use crate::app::{AppState, Message};
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use crate::gui::widgets;
use iced::widget::{Space, column, container, image, text};
use iced::{Element, Length};

pub fn preview_panel(state: &AppState) -> Element<'_, Message> {
    let Some(id) = state.selected else {
        return container(widgets::empty_state(
            "Nothing selected",
            "Pick an entry to preview it here.",
        ))
        .width(Length::FillPortion(2))
        .padding(sp::CONTENT_PAD)
        .into();
    };
    let Some(entry) = state.entries.iter().find(|e| e.id == id) else {
        return container(widgets::empty_state("Entry unavailable", ""))
            .width(Length::FillPortion(2))
            .padding(sp::CONTENT_PAD)
            .into();
    };

    let body: Element<'_, Message> = match entry.kind {
        ContentKind::Text => text(entry.text_content.as_deref().unwrap_or(""))
            .size(sp::TEXT_BODY)
            .into(),
        ContentKind::Image => {
            if let Some(bytes) = state.preview_bytes.as_ref() {
                image(image::Handle::from_bytes(bytes.clone()))
                    .width(Length::Fill)
                    .height(300)
                    .content_fit(iced::ContentFit::Contain)
                    .into()
            } else if let Some(path) = entry.image_path.as_deref() {
                image(image::Handle::from_path(std::path::PathBuf::from(path)))
                    .width(Length::Fill)
                    .height(300)
                    .content_fit(iced::ContentFit::Contain)
                    .into()
            } else {
                text("Image preview unavailable").size(sp::TEXT_BODY).into()
            }
        }
        ContentKind::Files => {
            let metas = crate::db::entries::file_metadata(entry);
            let mut c = column![].spacing(sp::GAP_TIGHT);
            for m in metas {
                let size = if m.size_bytes > 1024 * 1024 {
                    format!("{:.1} MB", m.size_bytes as f64 / 1_048_576.0)
                } else {
                    format!("{} B", m.size_bytes)
                };
                c = c.push(row_text(m.path.clone(), size.clone(), m.exists));
            }
            c.into()
        }
    };

    // Provenance: which app copied it and, when available, the web page URL.
    let mut source_lines = column![].spacing(sp::GAP_TIGHT);
    if let Some(app) = entry.source_app.as_deref() {
        source_lines = source_lines.push(
            iced::widget::row![
                text("Source app").size(sp::TEXT_CAPTION).width(90),
                text(app.to_string()).size(sp::TEXT_LABEL),
            ]
            .spacing(sp::GAP_TIGHT),
        );
    }
    if let Some(url) = entry.source_url.as_deref() {
        source_lines = source_lines.push(
            iced::widget::row![
                text("Source URL").size(sp::TEXT_CAPTION).width(90),
                text(url.to_string()).size(sp::TEXT_LABEL),
            ]
            .spacing(sp::GAP_TIGHT),
        );
    }
    if entry.kind == ContentKind::Image {
        source_lines = source_lines.push(
            iced::widget::row![
                text("Dimensions").size(sp::TEXT_CAPTION).width(90),
                text(format!(
                    "{} x {} px",
                    entry.image_w.unwrap_or(0),
                    entry.image_h.unwrap_or(0)
                ))
                .size(sp::TEXT_LABEL),
            ]
            .spacing(sp::GAP_TIGHT),
        );
    }
    let has_source = entry.source_app.is_some()
        || entry.source_url.is_some()
        || entry.kind == ContentKind::Image;

    let mut content = column![
        text(format!("{} - {}", entry.kind.label(), entry.short_kind())).size(sp::TEXT_LABEL),
        Space::with_height(sp::GAP_TIGHT),
        body,
        Space::with_height(sp::GAP_TIGHT),
    ]
    .spacing(sp::GAP_TIGHT);
    if has_source {
        content = content.push(widgets::section_title("SOURCE"));
        content = content.push(source_lines);
        content = content.push(Space::with_height(sp::GAP_TIGHT));
    }
    content = content.push(widgets::quiet_button(
        "Copy to clipboard",
        Message::RestoreEntry(id),
    ));

    container(content)
        .width(Length::FillPortion(2))
        .padding(sp::CONTENT_PAD)
        .into()
}

fn row_text(path: String, size: String, exists: bool) -> Element<'static, Message> {
    let status = if exists { "" } else { " (missing)" };
    iced::widget::row![
        text(path).size(sp::TEXT_LABEL).width(Length::Fill),
        text(format!("{size}{status}")).size(sp::TEXT_CAPTION),
    ]
    .spacing(sp::GAP_TIGHT)
    .into()
}
