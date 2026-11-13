//! Full-text + filtered, paged search over clipboard history.
use crate::db::DbError;
use crate::domain::{ClipEntry, ContentKind, SearchFilter};
use rusqlite::Connection;

const SELECT_COLS: &str = "e.id, e.kind, e.content_hash, e.text_content, e.image_path, e.image_w, \
     e.image_h, e.file_paths_json, e.file_meta_json, e.created_at, e.last_used_at, \
     e.use_count, e.pinned, e.favorite, e.source_app, e.source_url, e.byte_size";

fn row_to_entry(r: &rusqlite::Row<'_>) -> rusqlite::Result<ClipEntry> {
    let kind_s: String = r.get(1)?;
    Ok(ClipEntry {
        id: r.get(0)?,
        kind: ContentKind::parse_kind(&kind_s).unwrap_or(ContentKind::Text),
        content_hash: r.get(2)?,
        text_content: r.get(3)?,
        image_path: r.get(4)?,
        image_w: r.get(5)?,
        image_h: r.get(6)?,
        file_paths_json: r.get(7)?,
        file_meta_json: r.get(8)?,
        created_at: r.get(9)?,
        last_used_at: r.get(10)?,
        use_count: r.get(11)?,
        pinned: r.get::<_, i64>(12)? != 0,
        favorite: r.get::<_, i64>(13)? != 0,
        source_app: r.get(14)?,
        source_url: r.get(15)?,
        byte_size: r.get(16)?,
    })
}

/// Build the WHERE clause and its bound values. Kept separate for unit testing.
pub fn build_where(f: &SearchFilter) -> (String, Vec<rusqlite::types::Value>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut vals: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(kind) = f.kind {
        clauses.push("e.kind = ?".into());
        vals.push(kind.as_str().to_string().into());
    }
    if let Some(cutoff) = f.date.cutoff_unix(chrono::Utc::now().timestamp()) {
        clauses.push("e.created_at >= ?".into());
        vals.push(cutoff.into());
    }
    if f.pinned_only {
        clauses.push("e.pinned = 1".into());
    }
    if f.favorites_only {
        clauses.push("e.favorite = 1".into());
    }
    let q = f.query.trim();
    if !q.is_empty() {
        // FTS5 MATCH for token search; LIKE fallback handles substrings/CJK.
        clauses.push(
            "(e.id IN (SELECT rowid FROM entries_fts WHERE entries_fts MATCH ?) \
                       OR e.text_content LIKE ? OR e.file_paths_json LIKE ?)"
                .into(),
        );
        let escaped = format!("\"{}\"*", q.replace('"', "\"\""));
        vals.push(escaped.into());
        let like = format!("%{}%", q);
        vals.push(like.clone().into());
        vals.push(like.into());
    }

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    (where_sql, vals)
}

/// Paged search. Pinned/favorite entries are surfaced first, then by recency.
pub fn search(conn: &Connection, f: &SearchFilter) -> Result<Vec<ClipEntry>, DbError> {
    let (where_sql, mut vals) = build_where(f);
    let limit = f.page_size.min(200) as i64;
    let offset = (f.page * f.page_size) as i64;
    let sql = format!(
        "SELECT {SELECT_COLS} FROM entries e {where_sql} \
         ORDER BY e.pinned DESC, e.favorite DESC, e.created_at DESC LIMIT ? OFFSET ?"
    );
    vals.push(limit.into());
    vals.push(offset.into());
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(vals.iter()), row_to_entry)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Total rows matching the filter (for pagination UI).
pub fn count(conn: &Connection, f: &SearchFilter) -> Result<usize, DbError> {
    let (where_sql, vals) = build_where(f);
    let sql = format!("SELECT COUNT(*) FROM entries e {where_sql}");
    let n: i64 = conn.query_row(&sql, rusqlite::params_from_iter(vals.iter()), |r| r.get(0))?;
    Ok(n as usize)
}
