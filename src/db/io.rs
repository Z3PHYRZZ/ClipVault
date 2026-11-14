//! Portable JSON export / import of clipboard history.
//!
//! Pure serialization lives here (headless-testable); disk I/O is performed by
//! the DB worker so the GUI thread never blocks. Images are embedded as base64
//! PNG bytes so a backup survives moving to another machine. Re-import dedups
//! by `(kind, content_hash)` through the existing `insert_dedup` path.
use crate::db::DbError;
use crate::domain::ClipEntry;

/// Maximum exported JSON size (bytes). Prevents an OOM on huge image histories;
/// the caller reports truncation rather than silently dropping data.
pub const MAX_EXPORT_BYTES: usize = 50 * 1024 * 1024;

/// A single exported entry, with image bytes inlined when available.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportedEntry {
    pub kind: String,
    pub content_hash: String,
    pub text_content: Option<String>,
    pub image_png_base64: Option<String>,
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
    pub tags: Vec<String>,
    pub byte_size: i64,
}

/// Root document. `version` lets future readers handle older backups.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryBackup {
    pub format: String,
    pub version: u32,
    pub exported_at: i64,
    pub entries: Vec<ExportedEntry>,
}

impl HistoryBackup {
    pub fn format_id() -> &'static str {
        "clipvault-history"
    }
    pub fn format_version() -> u32 {
        1
    }

    /// Serialize a backup to pretty JSON. Returns `Other` if it exceeds the cap.
    pub fn to_json(&self) -> Result<String, DbError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| DbError::Other(format!("serialize backup: {e}")))?;
        if json.len() > MAX_EXPORT_BYTES {
            return Err(DbError::Other(format!(
                "export exceeds {} MB cap ({} bytes); narrow the filter or drop large images",
                MAX_EXPORT_BYTES / (1024 * 1024),
                json.len()
            )));
        }
        Ok(json)
    }

    /// Parse a backup, validating the format marker and version.
    pub fn from_json(json: &str) -> Result<Self, DbError> {
        let backup: HistoryBackup = serde_json::from_str(json)
            .map_err(|e| DbError::Other(format!("invalid backup JSON: {e}")))?;
        if backup.format != Self::format_id() {
            return Err(DbError::Other(format!(
                "unexpected backup format {:?}",
                backup.format
            )));
        }
        if backup.version > Self::format_version() {
            return Err(DbError::Other(format!(
                "backup version {} is newer than supported {}",
                backup.version,
                Self::format_version()
            )));
        }
        Ok(backup)
    }
}

/// Convert DB rows into a backup. `images` maps entry id -> raw PNG bytes,
/// gathered by the worker; entries without an inlined image keep their path
/// out of the backup (privacy: no absolute local paths exported).
pub fn build_backup(
    entries: &[ClipEntry],
    images: &std::collections::HashMap<i64, Vec<u8>>,
) -> HistoryBackup {
    let exported = entries
        .iter()
        .map(|e| ExportedEntry {
            kind: e.kind.as_str().to_string(),
            content_hash: e.content_hash.clone(),
            text_content: e.text_content.clone(),
            image_png_base64: images.get(&e.id).map(|b| base64_encode(b)),
            image_w: e.image_w,
            image_h: e.image_h,
            file_paths_json: e.file_paths_json.clone(),
            file_meta_json: e.file_meta_json.clone(),
            created_at: e.created_at,
            last_used_at: e.last_used_at,
            use_count: e.use_count,
            pinned: e.pinned,
            favorite: e.favorite,
            source_app: e.source_app.clone(),
            source_url: e.source_url.clone(),
            tags: e.tags(),
            byte_size: e.byte_size,
        })
        .collect();
    HistoryBackup {
        format: HistoryBackup::format_id().to_string(),
        version: HistoryBackup::format_version(),
        exported_at: chrono::Utc::now().timestamp(),
        entries: exported,
    }
}

/// Minimal base64 encoder (avoids a new dependency; used only for image bytes).
pub fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Minimal base64 decoder; returns None on invalid input.
pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b'\r' | b'\n' | b' ' => continue,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip_various_lengths() {
        for len in [0usize, 1, 2, 3, 4, 5, 100, 1024] {
            let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let encoded = base64_encode(&data);
            assert_eq!(base64_decode(&encoded), Some(data), "len {len}");
        }
    }

    #[test]
    fn backup_json_roundtrip_preserves_tags() {
        let backup = HistoryBackup {
            format: HistoryBackup::format_id().to_string(),
            version: 1,
            exported_at: 123,
            entries: vec![ExportedEntry {
                kind: "text".into(),
                content_hash: "abc".into(),
                text_content: Some("hello".into()),
                image_png_base64: None,
                image_w: None,
                image_h: None,
                file_paths_json: None,
                file_meta_json: None,
                created_at: 1,
                last_used_at: 2,
                use_count: 3,
                pinned: true,
                favorite: false,
                source_app: Some("chrome.exe".into()),
                source_url: None,
                tags: vec!["work".into(), "todo".into()],
                byte_size: 5,
            }],
        };
        let json = backup.to_json().expect("serialize");
        let parsed = HistoryBackup::from_json(&json).expect("parse");
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].tags, vec!["work", "todo"]);
        assert!(parsed.entries[0].pinned);
    }

    #[test]
    fn rejects_wrong_format_and_future_version() {
        let mut b = HistoryBackup {
            format: "something-else".into(),
            version: 1,
            exported_at: 0,
            entries: vec![],
        };
        assert!(HistoryBackup::from_json(&serde_json::to_string(&b).unwrap()).is_err());
        b.format = HistoryBackup::format_id().into();
        b.version = 99;
        assert!(HistoryBackup::from_json(&serde_json::to_string(&b).unwrap()).is_err());
    }
}
