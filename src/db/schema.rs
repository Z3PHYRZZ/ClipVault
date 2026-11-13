//! Versioned schema migrations via `PRAGMA user_version`.
use crate::db::DbError;
use rusqlite::Connection;

pub const LATEST_VERSION: u32 = 4;

/// Each migration is applied inside its own transaction and bumps user_version.
const MIGRATIONS: &[&str] = &[
    // v1 — core entries table + indexes.
    "CREATE TABLE IF NOT EXISTS entries (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        kind TEXT NOT NULL,
        content_hash TEXT NOT NULL,
        text_content TEXT,
        image_path TEXT,
        image_w INTEGER,
        image_h INTEGER,
        file_paths_json TEXT,
        file_meta_json TEXT,
        created_at INTEGER NOT NULL,
        last_used_at INTEGER NOT NULL,
        use_count INTEGER NOT NULL DEFAULT 1,
        pinned INTEGER NOT NULL DEFAULT 0,
        favorite INTEGER NOT NULL DEFAULT 0,
        source_app TEXT,
        byte_size INTEGER NOT NULL DEFAULT 0
    );
    CREATE UNIQUE INDEX IF NOT EXISTS ux_entries_kind_hash ON entries(kind, content_hash);
    CREATE INDEX IF NOT EXISTS ix_entries_created ON entries(created_at DESC);
    CREATE INDEX IF NOT EXISTS ix_entries_kind ON entries(kind);
    CREATE INDEX IF NOT EXISTS ix_entries_pinned ON entries(pinned);
    CREATE INDEX IF NOT EXISTS ix_entries_favorite ON entries(favorite);",
    // v2 — FTS5 virtual table for fast token search, kept in sync by triggers.
    "CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
        text_content, file_paths, content='entries', content_rowid='id',
        tokenize='unicode61');
    CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
        INSERT INTO entries_fts(rowid, text_content, file_paths)
        VALUES (new.id, COALESCE(new.text_content,''), COALESCE(new.file_paths_json,''));
    END;
    CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
        INSERT INTO entries_fts(entries_fts, rowid, text_content, file_paths)
        VALUES ('delete', old.id, COALESCE(old.text_content,''), COALESCE(old.file_paths_json,''));
    END;
    CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
        INSERT INTO entries_fts(entries_fts, rowid, text_content, file_paths)
        VALUES ('delete', old.id, COALESCE(old.text_content,''), COALESCE(old.file_paths_json,''));
        INSERT INTO entries_fts(rowid, text_content, file_paths)
        VALUES (new.id, COALESCE(new.text_content,''), COALESCE(new.file_paths_json,''));
    END;",
    // v3 — retained config + source index (schema evolution example).
    "CREATE INDEX IF NOT EXISTS ix_entries_last_used ON entries(last_used_at DESC);",
    // v4 — provenance: web source URL alongside the existing source_app column.
    // Uses ADD COLUMN (idempotent-safe for a fresh v4 DB) and an index for
    // "copied from where" lookups. NULL = not from a web page / unknown.
    "ALTER TABLE entries ADD COLUMN source_url TEXT;
    CREATE INDEX IF NOT EXISTS ix_entries_source_url ON entries(source_url)
        WHERE source_url IS NOT NULL;",
];

/// Run all pending migrations, returning the resulting user_version.
pub fn migrate(conn: &mut Connection) -> Result<u32, DbError> {
    let current: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if current >= LATEST_VERSION {
        return Ok(current);
    }
    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as u32;
        if version <= current {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(LATEST_VERSION)
}
