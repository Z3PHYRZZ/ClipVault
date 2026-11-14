//! Entry CRUD, duplicate-safe insert, and retention pruning.
use crate::db::{DbError, image_assets};
use crate::domain::{ClipEntry, ContentKind, FileMetadata};
use crate::workers::{ClipCapture, ClipPayload};
use rusqlite::{Connection, OptionalExtension, params};

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

fn row_to_entry(r: &rusqlite::Row<'_>) -> rusqlite::Result<ClipEntry> {
    let kind_s: String = r.get("kind")?;
    Ok(ClipEntry {
        id: r.get("id")?,
        kind: ContentKind::parse_kind(&kind_s).unwrap_or(ContentKind::Text),
        content_hash: r.get("content_hash")?,
        text_content: r.get("text_content")?,
        image_path: r.get("image_path")?,
        image_w: r.get("image_w")?,
        image_h: r.get("image_h")?,
        file_paths_json: r.get("file_paths_json")?,
        file_meta_json: r.get("file_meta_json")?,
        created_at: r.get("created_at")?,
        last_used_at: r.get("last_used_at")?,
        use_count: r.get("use_count")?,
        pinned: r.get::<_, i64>("pinned")? != 0,
        favorite: r.get::<_, i64>("favorite")? != 0,
        source_app: r.get("source_app")?,
        source_url: r.get("source_url")?,
        tags_json: r.get("tags_json")?,
        byte_size: r.get("byte_size")?,
    })
}

const SELECT_COLS: &str = "id, kind, content_hash, text_content, image_path, image_w, image_h, \
     file_paths_json, file_meta_json, created_at, last_used_at, use_count, pinned, favorite, \
     source_app, source_url, tags_json, byte_size";

/// Insert a captured payload (+ provenance), or bump usage counters on a duplicate.
/// A UNIQUE(kind, content_hash) index + ON CONFLICT makes this race-safe.
/// Provenance is only written when present, so a re-copy with an unknown source
/// does not erase an earlier known source.
pub fn insert_dedup(conn: &Connection, cap: &ClipCapture) -> Result<i64, DbError> {
    let now = now_unix();
    let app = cap.source_app.as_deref();
    let url = cap.source_url.as_deref();
    match &cap.payload {
        ClipPayload::Text(s) => {
            let hash = crate::domain::hash_text(s);
            let size = s.len() as i64;
            let tx = conn.unchecked_transaction()?;
            tx.execute(
                "INSERT INTO entries (kind, content_hash, text_content, created_at, last_used_at, \
                 use_count, byte_size, source_app, source_url) \
                 VALUES ('text', ?1, ?2, ?3, ?3, 1, ?4, ?5, ?6) \
                 ON CONFLICT(kind, content_hash) DO UPDATE SET \
                 use_count = use_count + 1, last_used_at = excluded.last_used_at, \
                 source_app = COALESCE(excluded.source_app, entries.source_app), \
                 source_url = COALESCE(excluded.source_url, entries.source_url)",
                params![hash, s, now, size, app, url],
            )?;
            let id: i64 = tx.query_row(
                "SELECT id FROM entries WHERE kind='text' AND content_hash=?1",
                params![hash],
                |r| r.get(0),
            )?;
            tx.commit()?;
            Ok(id)
        }
        ClipPayload::Image { png, w, h, hash } => {
            // Persist PNG to the asset dir keyed by content hash; DB stores the path only.
            let path = image_assets::save_png(png, hash)?;
            let rel = path.to_string_lossy().into_owned();
            let size = png.len() as i64;
            let tx = conn.unchecked_transaction()?;
            tx.execute(
                "INSERT INTO entries (kind, content_hash, image_path, image_w, image_h, created_at, \
                 last_used_at, use_count, byte_size, source_app, source_url) \
                 VALUES ('image', ?1, ?2, ?3, ?4, ?5, ?5, 1, ?6, ?7, ?8) \
                 ON CONFLICT(kind, content_hash) DO UPDATE SET \
                 use_count = use_count + 1, last_used_at = excluded.last_used_at, \
                 source_app = COALESCE(excluded.source_app, entries.source_app), \
                 source_url = COALESCE(excluded.source_url, entries.source_url)",
                params![hash, rel, w, h, now, size, app, url],
            )?;
            let id: i64 = tx.query_row(
                "SELECT id FROM entries WHERE kind='image' AND content_hash=?1",
                params![hash],
                |r| r.get(0),
            )?;
            tx.commit()?;
            Ok(id)
        }
        ClipPayload::Files(metas) => {
            let paths: Vec<String> = metas.iter().map(|m| m.path.clone()).collect();
            let hash = crate::domain::hash_paths(&paths);
            let paths_json = serde_json::to_string(&paths).unwrap_or_default();
            let meta_json = serde_json::to_string(metas).unwrap_or_default();
            let size: i64 = metas.iter().map(|m| m.size_bytes as i64).sum();
            let tx = conn.unchecked_transaction()?;
            tx.execute(
                "INSERT INTO entries (kind, content_hash, file_paths_json, file_meta_json, \
                 created_at, last_used_at, use_count, byte_size, source_app, source_url) \
                 VALUES ('files', ?1, ?2, ?3, ?4, ?4, 1, ?5, ?6, ?7) \
                 ON CONFLICT(kind, content_hash) DO UPDATE SET \
                 use_count = use_count + 1, last_used_at = excluded.last_used_at, \
                 file_meta_json = excluded.file_meta_json, \
                 source_app = COALESCE(excluded.source_app, entries.source_app), \
                 source_url = COALESCE(excluded.source_url, entries.source_url)",
                params![hash, paths_json, meta_json, now, size, app, url],
            )?;
            let id: i64 = tx.query_row(
                "SELECT id FROM entries WHERE kind='files' AND content_hash=?1",
                params![hash],
                |r| r.get(0),
            )?;
            tx.commit()?;
            Ok(id)
        }
    }
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<ClipEntry>, DbError> {
    let sql = format!("SELECT {SELECT_COLS} FROM entries WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let e = stmt.query_row(params![id], row_to_entry).optional()?;
    Ok(e)
}

pub fn set_pin(conn: &Connection, id: i64, v: bool) -> Result<(), DbError> {
    conn.execute(
        "UPDATE entries SET pinned = ?1 WHERE id = ?2",
        params![v as i64, id],
    )?;
    Ok(())
}

pub fn set_favorite(conn: &Connection, id: i64, v: bool) -> Result<(), DbError> {
    conn.execute(
        "UPDATE entries SET favorite = ?1 WHERE id = ?2",
        params![v as i64, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), DbError> {
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear_all(conn: &Connection) -> Result<(), DbError> {
    conn.execute("DELETE FROM entries", [])?;
    Ok(())
}

/// Remove oldest non-pinned, non-favorite rows exceeding count/age limits.
/// Returns the number of deleted rows. Asset GC runs after commit in the worker.
pub fn prune(conn: &Connection, max_entries: usize, max_age_days: u32) -> Result<usize, DbError> {
    let now = now_unix();
    let cutoff = now - (max_age_days as i64) * 86_400;
    let tx = conn.unchecked_transaction()?;
    // Age-based removal first, respecting pinned/favorite.
    let age_deleted = tx.execute(
        "DELETE FROM entries WHERE pinned = 0 AND favorite = 0 AND created_at < ?1",
        params![cutoff],
    )?;
    // Count-based: keep the newest (max_entries) rows, plus all pinned/favorite.
    let over: i64 = tx.query_row(
        "SELECT COUNT(*) FROM entries WHERE pinned = 0 AND favorite = 0",
        [],
        |r| r.get(0),
    )?;
    let excess = (over - max_entries as i64).max(0);
    if excess > 0 {
        tx.execute(
            "DELETE FROM entries WHERE id IN (
                SELECT id FROM entries WHERE pinned = 0 AND favorite = 0
                ORDER BY created_at ASC LIMIT ?1)",
            params![excess],
        )?;
    }
    let deleted = age_deleted + excess as usize;
    tx.commit()?;
    Ok(deleted)
}

/// Parse stored file metadata for preview (tolerates missing/stale JSON).
pub fn file_metadata(entry: &ClipEntry) -> Vec<FileMetadata> {
    entry
        .file_meta_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}
