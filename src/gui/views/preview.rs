//! Right-hand preview panel: text, image, or file metadata.
//! shadcn port: Card surface, Input tag editor, ghost/default buttons.
use crate::app::{AppState, Message};
use crate::domain::ContentKind;
use crate::gui::spacing as sp;
use crate::gui::theme;
use crate::gui::widgets;
use iced::widget::{Space, column, container, image, row, text, text_input};
use iced::{Element, Length, Theme};

fn wrap_preview<'a>(body: Element<'a, Message>) -> Element<'a, Message> {
    container(body)
        .width(Length::FillPortion(2))
        .padding(sp::CONTENT_PAD)
        .style(theme::glass_surface)
        .into()
}

pub fn preview_panel(state: &AppState) -> Element<'_, Message> {
    let wrap = wrap_preview;

    let Some(id) = state.selected else {
        return wrap(widgets::empty_state(
            "Nothing selected",
            "Pick an entry to preview it here.",
        ));
    };
    let Some(entry) = state.entries.iter().find(|e| e.id == id) else {
        return wrap(widgets::empty_state("Entry unavailable", ""));
    };

    let body: Element<'_, Message> = match entry.kind {
        ContentKind::Text => text(entry.text_content.as_deref().unwrap_or(""))
            .size(sp::TEXT_BODY)
            .into(),
        ContentKind::Image => {
            if let Some(bytes) = state.preview_bytes.as_ref() {
                image(image::Handle::from_bytes(bytes.clone()))
                    .width(Length::Fill)
                    .height(sp::PREVIEW_IMAGE_H)
                    .into()
            } else if let Some(path) = entry.image_path.as_deref() {
                image(image::Handle::from_path(std::path::PathBuf::from(path)))
                    .width(Length::Fill)
                    .height(sp::PREVIEW_IMAGE_H)
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

    let mut source_lines = column![].spacing(sp::GAP_TIGHT);
    if let Some(app) = entry.source_app.as_deref() {
        source_lines = source_lines.push(property("Source app", app.to_string()));
    }
    if let Some(url) = entry.source_url.as_deref() {
        source_lines = source_lines.push(property("Source URL", url.to_string()));
    }
    if entry.kind == ContentKind::Image {
        source_lines = source_lines.push(property(
            "Dimensions",
            format!(
                "{} x {} px",
                entry.image_w.unwrap_or(0),
                entry.image_h.unwrap_or(0)
            ),
        ));
    }
    let has_source = entry.source_app.is_some()
        || entry.source_url.is_some()
        || entry.kind == ContentKind::Image;

    let mut content = column![
        text(format!("{} - {}", entry.kind.label(), entry.short_kind())).size(sp::TEXT_LABEL),
        Space::with_height(sp::GAP_TIGHT),
        body,
        widgets::separator(),
    ]
    .spacing(sp::GAP_TIGHT);
    content = content.push(widgets::section_title("TAGS"));
    content = content.push(
        row![
            text_input("tag1, tag2", &state.tags_input)
                .on_input(Message::TagsDraftChanged)
                .padding(sp::INPUT_PAD)
                .style(|t: &Theme, st| theme::input_style(t, st))
                .width(Length::Fill),
            widgets::icon_button("Apply", Message::ApplyTags),
        ]
        .spacing(sp::GAP_TIGHT)
        .align_y(iced::Alignment::Center),
    );
    if has_source {
        content = content.push(widgets::separator());
        content = content.push(widgets::section_title("SOURCE"));
        content = content.push(source_lines);
    }
    content = content.push(widgets::separator());
    content = content.push(widgets::icon_button(
        "Copy to clipboard",
        Message::RestoreEntry(id),
    ));

    wrap(content.into())
}

fn property(label: &str, value: String) -> Element<'_, Message> {
    row![
        text(label.to_string())
            .size(sp::TEXT_CAPTION)
            .width(sp::PROPERTY_LABEL_W),
        text(value).size(sp::TEXT_LABEL),
    ]
    .spacing(sp::GAP_TIGHT)
    .into()
}

fn row_text(path: String, size: String, exists: bool) -> Element<'static, Message> {
    let status = if exists { "" } else { " (missing)" };
    row![
        text(path).size(sp::TEXT_LABEL).width(Length::Fill),
        text(format!("{size}{status}")).size(sp::TEXT_CAPTION),
    ]
    .spacing(sp::GAP_TIGHT)
    .into()
}
