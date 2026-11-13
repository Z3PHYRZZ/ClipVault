//! Migration test: an existing v3 database must upgrade to v4 without data loss.
use clipvault::db::store::Store;
use rusqlite::Connection;

fn ok<T, E: std::fmt::Display>(r: Result<T, E>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("expected Ok, got error: {e}"),
    }
}

#[test]
fn v3_database_upgrades_to_v4_preserving_rows() {
    // Build a v3-shaped DB (source_url column absent, user_version = 3) with a
    // legacy row, then run the real migration runner through Store::open.
    let dir = std::env::temp_dir().join(format!("clipvault_mig_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("legacy.db");
    let _ = std::fs::remove_file(&path);

    {
        let conn = ok(Connection::open(&path));
        ok(conn.execute_batch(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY AUTOINCREMENT, kind TEXT NOT NULL, \
             content_hash TEXT NOT NULL, text_content TEXT, image_path TEXT, image_w INTEGER, \
             image_h INTEGER, file_paths_json TEXT, file_meta_json TEXT, created_at INTEGER NOT NULL, \
             last_used_at INTEGER NOT NULL, use_count INTEGER NOT NULL DEFAULT 1, \
             pinned INTEGER NOT NULL DEFAULT 0, favorite INTEGER NOT NULL DEFAULT 0, \
             source_app TEXT, byte_size INTEGER NOT NULL DEFAULT 0); \
             INSERT INTO entries (kind, content_hash, text_content, created_at, last_used_at, byte_size) \
             VALUES ('text', 'abc', 'legacy row', 1, 1, 3); \
             PRAGMA user_version = 3;",
        ));
    }

    let store = ok(Store::open(&path));
    let version: u32 = ok(store
        .conn()
        .query_row("PRAGMA user_version", [], |r| r.get(0)));
    assert_eq!(version, 4, "migration must land on user_version 4");

    let cols: String = ok(store.conn().query_row(
        "SELECT GROUP_CONCAT(name) FROM pragma_table_info('entries')",
        [],
        |r| r.get(0),
    ));
    assert!(cols.contains("source_url"), "v4 must add source_url column");

    let n: i64 = ok(store
        .conn()
        .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0)));
    assert_eq!(n, 1, "existing rows must survive the migration");

    let text: String =
        ok(store
            .conn()
            .query_row("SELECT text_content FROM entries LIMIT 1", [], |r| r.get(0)));
    assert_eq!(text, "legacy row");

    drop(store);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}
