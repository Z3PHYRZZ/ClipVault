//! Bulk organization operations on clipboard history.
//!
//! All functions take `&Connection` and are called only by the DB worker, so
//! every statement runs off the GUI thread. Bulk deletes run in one transaction
//! and keep `entries_fts` in sync via the existing triggers.
use crate::db::{DbError, entries, image_assets, io, search};
use crate::domain::{ClipEntry, ContentKind};
use crate::workers::ClipCapture;
use crate::workers::ClipPayload;
use rusqlite::{Connection, params};

/// Replace an entry's tag list (normalized by `ClipEntry::set_tags`).
pub fn set_tags(conn: &Connection, id: i64, tags: &[String]) -> Result<(), DbError> {
    let mut e = entries::get(conn, id)?.ok_or_else(|| DbError::Other("entry not found".into()))?;
    e.set_tags(tags);
    conn.execute(
        "UPDATE entries SET tags_json = ?1 WHERE id = ?2",
        params![e.tags_json, id],
    )?;
    Ok(())
}

/// Delete many ids in one transaction. Returns the number of rows removed.
pub fn delete_many(conn: &Connection, ids: &[i64]) -> Result<usize, DbError> {
    if ids.is_empty() {
        return Ok(0);
    }
    let tx = conn.unchecked_transaction()?;
    let mut n = 0usize;
    {
        let mut stmt = tx.prepare("DELETE FROM entries WHERE id = ?1")?;
        for id in ids {
            n += stmt.execute(params![id])?;
        }
    }
    tx.commit()?;
    let _ = image_assets::delete_orphans(conn);
    Ok(n)
}

/// Pin/unpin many ids in one transaction.
pub fn set_pin_many(conn: &Connection, ids: &[i64], value: bool) -> Result<usize, DbError> {
    set_flag_many(conn, ids, value, "pinned")
}

/// Favorite/unfavorite many ids in one transaction.
pub fn set_favorite_many(conn: &Connection, ids: &[i64], value: bool) -> Result<usize, DbError> {
    set_flag_many(conn, ids, value, "favorite")
}

fn set_flag_many(
    conn: &Connection,
    ids: &[i64],
    value: bool,
    column: &str,
) -> Result<usize, DbError> {
    if ids.is_empty() {
        return Ok(0);
    }
    let sql = format!("UPDATE entries SET {column} = ?1 WHERE id = ?2");
    let tx = conn.unchecked_transaction()?;
    let mut n = 0usize;
    {
        let mut stmt = tx.prepare(&sql)?;
        let v = if value { 1i64 } else { 0i64 };
        for id in ids {
            n += stmt.execute(params![v, id])?;
        }
    }
    tx.commit()?;
    Ok(n)
}

/// Per-type retention caps. `0` means "use the global cap" for that kind.
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeCaps {
    pub text: usize,
    pub image: usize,
    pub files: usize,
}

fn cap_for(caps: TypeCaps, kind: ContentKind, global: usize) -> usize {
    let per_type = match kind {
        ContentKind::Text => caps.text,
        ContentKind::Image => caps.image,
        ContentKind::Files => caps.files,
    };
    if per_type == 0 {
        global
    } else {
        per_type.min(global)
    }
}

/// Prune with a tighter per-type cap applied after the global age/count prune.
/// Pinned and favorite rows are exempt. Returns total rows deleted.
pub fn prune_with_caps(
    conn: &Connection,
    max_entries: usize,
    max_age_days: u32,
    caps: TypeCaps,
) -> Result<usize, DbError> {
    let global_deleted = entries::prune(conn, max_entries, max_age_days)?;
    let tx = conn.unchecked_transaction()?;
    let mut extra = 0usize;
    for kind in [ContentKind::Text, ContentKind::Image, ContentKind::Files] {
        let cap = cap_for(caps, kind, max_entries);
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM entries WHERE kind = ?1 AND pinned = 0 AND favorite = 0",
            params![kind.as_str()],
            |r| r.get(0),
        )?;
        let excess = count - cap as i64;
        if excess > 0 {
            extra += tx.execute(
                "DELETE FROM entries WHERE id IN (
                    SELECT id FROM entries
                    WHERE kind = ?1 AND pinned = 0 AND favorite = 0
                    ORDER BY created_at ASC LIMIT ?2)",
                params![kind.as_str(), excess],
            )?;
        }
    }
    tx.commit()?;
    // Clean up any image assets orphaned by the extra deletions.
    if extra > 0 {
        let _ = image_assets::delete_orphans(conn);
    }
    Ok(global_deleted + extra)
}

/// Build a portable JSON backup from the current filter (or all when filter is None).
pub fn export_json(
    conn: &Connection,
    filter: Option<&crate::domain::SearchFilter>,
) -> Result<String, DbError> {
    let rows: Vec<ClipEntry> = match filter {
        Some(f) => search::search(conn, f)?,
        None => search::search(conn, &crate::domain::SearchFilter::default())?,
    };
    // Gather image bytes for inlining (bounded by the export cap in to_json()).
    let mut images = std::collections::HashMap::new();
    for e in &rows {
        if e.kind == ContentKind::Image {
            if let Some(path) = e.image_path.as_deref() {
                if let Ok(bytes) = image_assets::load_png(path) {
                    images.insert(e.id, bytes);
                }
            }
        }
    }
    io::build_backup(&rows, &images).to_json()
}

/// Import a backup. Re-inserts through `insert_dedup` so `(kind, content_hash)`
/// duplicates are merged instead of recreated. Returns rows inserted or bumped.
pub fn import_json(conn: &Connection, json: &str) -> Result<usize, DbError> {
    let backup = io::HistoryBackup::from_json(json)?;
    let mut n = 0usize;
    for item in &backup.entries {
        let kind = ContentKind::parse_kind(&item.kind).unwrap_or(ContentKind::Text);
        let payload = match kind {
            ContentKind::Text => ClipPayload::Text(item.text_content.clone().unwrap_or_default()),
            ContentKind::Image => {
                let png = item
                    .image_png_base64
                    .as_deref()
                    .and_then(io::base64_decode)
                    .unwrap_or_default();
                if png.is_empty() {
                    // No inlined bytes: keep the row as a text placeholder so
                    // history is not silently lost, with a clear marker.
                    ClipPayload::Text(format!("[image {}]", item.image_w.unwrap_or(0)))
                } else {
                    ClipPayload::Image {
                        hash: item.content_hash.clone(),
                        png,
                        w: item.image_w.unwrap_or(0),
                        h: item.image_h.unwrap_or(0),
                    }
                }
            }
            ContentKind::Files => {
                let metas: Vec<crate::domain::FileMetadata> = item
                    .file_meta_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                ClipPayload::Files(metas)
            }
        };
        let capture =
            ClipCapture::new(payload).with_source(item.source_app.clone(), item.source_url.clone());
        entries::insert_dedup(conn, &capture)?;
        // Restore tags + flags on the (possibly pre-existing) row.
        let id: i64 = conn.query_row(
            "SELECT id FROM entries WHERE kind = ?1 AND content_hash = ?2",
            params![kind.as_str(), item.content_hash],
            |r| r.get(0),
        )?;
        set_tags(conn, id, &item.tags)?;
        conn.execute(
            "UPDATE entries SET pinned = ?1, favorite = ?2 WHERE id = ?3",
            params![item.pinned as i64, item.favorite as i64, id],
        )?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_for_falls_back_to_global() {
        let caps = TypeCaps::default();
        assert_eq!(cap_for(caps, ContentKind::Text, 500), 500);
        let tighter = TypeCaps {
            text: 10,
            image: 0,
            files: 0,
        };
        assert_eq!(cap_for(tighter, ContentKind::Text, 500), 10);
        assert_eq!(cap_for(tighter, ContentKind::Image, 500), 500);
        // per-type can never exceed global
        assert_eq!(cap_for(tighter, ContentKind::Text, 5), 5);
    }
}
