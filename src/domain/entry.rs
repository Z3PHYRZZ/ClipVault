//! Clipboard entry domain model.
use serde::{Deserialize, Serialize};

/// The kind of clipboard payload stored in an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentKind {
    Text,
    Image,
    Files,
}

impl ContentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ContentKind::Text => "text",
            ContentKind::Image => "image",
            ContentKind::Files => "files",
        }
    }

    pub fn parse_kind(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentKind::Text),
            "image" => Some(ContentKind::Image),
            "files" => Some(ContentKind::Files),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ContentKind::Text => "Text",
            ContentKind::Image => "Image",
            ContentKind::Files => "Files",
        }
    }
}

/// Metadata for a single file in a file-drop clipboard entry.
/// We store paths + metadata only, never file contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub size_bytes: u64,
    pub is_dir: bool,
    pub modified_unix: i64,
    pub exists: bool,
}

/// A single clipboard history row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipEntry {
    pub id: i64,
    pub kind: ContentKind,
    pub content_hash: String,
    pub text_content: Option<String>,
    pub image_path: Option<String>,
    pub image_w: Option<u32>,
    pub image_h: Option<u32>,
    pub file_paths_json: Option<String>,
    pub file_meta_json: Option<String>,
    pub created_at: i64,
    pub last_used_at: i64,
    pub use_count: i64,
    pub pinned: bool,
    pub favorite: bool,
    pub source_app: Option<String>,
    pub source_url: Option<String>,
    pub byte_size: i64,
}

impl ClipEntry {
    /// Short human-readable text for list rows.
    pub fn display_text(&self) -> String {
        match self.kind {
            ContentKind::Text => self
                .text_content
                .as_deref()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(120)
                .collect(),
            ContentKind::Image => format!(
                "Image {}x{}",
                self.image_w.unwrap_or(0),
                self.image_h.unwrap_or(0)
            ),
            ContentKind::Files => {
                let paths: Vec<String> = self
                    .file_paths_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                if paths.len() == 1 {
                    std::path::Path::new(&paths[0])
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| paths[0].clone())
                } else {
                    format!("{} files", paths.len())
                }
            }
        }
    }

    pub fn short_kind(&self) -> &'static str {
        match self.kind {
            ContentKind::Text => "TXT",
            ContentKind::Image => "IMG",
            ContentKind::Files => "FILE",
        }
    }
}
