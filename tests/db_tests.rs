//! Integration tests for the public `clipvault` DB + domain API.
// Every test owns a private in-memory Store; no test touches the real data dir.
use clipvault::db::entries::{clear_all, delete, get, insert_dedup, prune, set_favorite, set_pin};
use clipvault::db::search::{count, search};
use clipvault::db::store::Store;
use clipvault::domain::{
    ContentKind, DateRange, FileMetadata, SearchFilter, dedup_key, hash_bytes, hash_paths,
    hash_text,
};
use clipvault::workers::ClipPayload;

fn ok<T, E: std::fmt::Display>(r: Result<T, E>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("expected Ok, got error: {e}"),
    }
}

fn some<T>(o: Option<T>) -> T {
    match o {
        Some(v) => v,
        None => panic!("expected Some, got None"),
    }
}

fn store() -> Store {
    ok(Store::open_in_memory())
}

fn fm(path: &str) -> FileMetadata {
    FileMetadata {
        path: path.into(),
        size_bytes: 0,
        is_dir: false,
        modified_unix: 0,
        exists: true,
    }
}

// page_size is clamped to 200 by search(); 200 covers every row these tests create.
fn filter() -> SearchFilter {
    SearchFilter {
        page_size: 200,
        ..Default::default()
    }
}

fn text(s: &str) -> ClipPayload {
    ClipPayload::Text(s.to_string())
}

#[test]
fn migration_applies_and_version() {
    let s = store();
    let c = s.conn();
    let version: u32 = ok(c.query_row("PRAGMA user_version", [], |r| r.get(0)));
    assert_eq!(version, 3, "migrations must land on user_version 3");

    let entries: i64 = ok(c.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
        [],
        |r| r.get(0),
    ));
    assert_eq!(entries, 1, "entries table must exist");

    // A live MATCH proves the FTS5 virtual table is compiled in and usable.
    let fts: i64 = ok(c.query_row(
        "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'anything'",
        [],
        |r| r.get(0),
    ));
    assert_eq!(fts, 0, "empty FTS index must match nothing");
}

#[test]
fn insert_text_then_get() {
    let s = store();
    let c = s.conn();
    let id = ok(insert_dedup(c, &text("hello world")));
    assert!(id > 0, "insert must return a real rowid");

    let e = some(ok(get(c, id)));
    assert_eq!(e.id, id);
    assert_eq!(e.kind, ContentKind::Text);
    assert_eq!(e.text_content.as_deref(), Some("hello world"));
    assert_eq!(e.use_count, 1);
    assert!(e.byte_size > 0, "text byte_size must be its UTF-8 length");
    assert_eq!(e.byte_size, 11);
    assert_eq!(e.content_hash, hash_text("hello world"));
    assert!(!e.pinned);
    assert!(!e.favorite);
}

#[test]
fn duplicate_text_bumps_use_count() {
    let s = store();
    let c = s.conn();
    let first = ok(insert_dedup(c, &text("duplicate text")));
    let second = ok(insert_dedup(c, &text("duplicate text")));
    assert_eq!(first, second, "dedup must reuse the same rowid");

    assert_eq!(ok(count(c, &filter())), 1, "duplicate must not add a row");
    let e = some(ok(get(c, first)));
    assert_eq!(e.use_count, 2);
    assert!(
        e.last_used_at >= e.created_at,
        "last_used_at must never move backwards"
    );
}

#[test]
fn different_kinds_no_collision() {
    let s = store();
    let c = s.conn();
    let text_id = ok(insert_dedup(c, &text("x")));
    let files_id = ok(insert_dedup(c, &ClipPayload::Files(vec![fm("C:\\x")])));

    assert_ne!(text_id, files_id, "kind is part of the dedup key");
    assert_eq!(ok(count(c, &filter())), 2);
    assert_eq!(some(ok(get(c, text_id))).kind, ContentKind::Text);
    assert_eq!(some(ok(get(c, files_id))).kind, ContentKind::Files);
}

#[test]
fn set_pin_favorite_then_search() {
    let s = store();
    let c = s.conn();
    let a = ok(insert_dedup(c, &text("first entry")));
    let b = ok(insert_dedup(c, &text("second entry")));
    let d = ok(insert_dedup(c, &text("third entry")));
    ok(set_pin(c, b, true));
    ok(set_favorite(c, d, true));

    let pinned = ok(search(
        c,
        &SearchFilter {
            pinned_only: true,
            ..filter()
        },
    ));
    assert_eq!(pinned.len(), 1, "pinned_only must return exactly one row");
    assert_eq!(pinned[0].id, b);
    assert!(pinned[0].pinned);

    let favs = ok(search(
        c,
        &SearchFilter {
            favorites_only: true,
            ..filter()
        },
    ));
    assert_eq!(favs.len(), 1, "favorites_only must return exactly one row");
    assert_eq!(favs[0].id, d);
    assert!(favs[0].favorite);

    assert_eq!(ok(count(c, &filter())), 3, "flags must not remove rows");
    assert!(some(ok(get(c, a))).id == a);
}

#[test]
fn search_matches_fts_and_filters() {
    let s = store();
    let c = s.conn();
    ok(insert_dedup(c, &text("alpha one")));
    ok(insert_dedup(c, &text("beta two")));
    ok(insert_dedup(c, &text("gamma three")));

    // build_where emits a `"alpha"*` FTS5 prefix query plus LIKE fallbacks.
    let hits = ok(search(
        c,
        &SearchFilter {
            query: "alpha".to_string(),
            ..filter()
        },
    ));
    assert_eq!(hits.len(), 1, "only the alpha row should match");
    assert_eq!(hits[0].text_content.as_deref(), Some("alpha one"));

    let images = ok(search(
        c,
        &SearchFilter {
            kind: Some(ContentKind::Image),
            ..filter()
        },
    ));
    assert!(images.is_empty(), "no image rows exist");
    assert_eq!(
        ok(count(
            c,
            &SearchFilter {
                kind: Some(ContentKind::Image),
                ..filter()
            }
        )),
        0
    );
    assert_eq!(
        ok(count(
            c,
            &SearchFilter {
                kind: Some(ContentKind::Text),
                ..filter()
            }
        )),
        3
    );

    let all = ok(search(
        c,
        &SearchFilter {
            date: DateRange::All,
            ..filter()
        },
    ));
    assert_eq!(all.len(), 3, "DateRange::All must not filter anything");
    assert_eq!(DateRange::All.cutoff_unix(1_700_000_000), None);
}

#[test]
fn delete_and_clear_all() {
    let s = store();
    let c = s.conn();
    let a = ok(insert_dedup(c, &text("keep me")));
    let b = ok(insert_dedup(c, &text("delete me")));
    ok(insert_dedup(c, &text("also keep")));

    ok(delete(c, b));
    assert_eq!(ok(count(c, &filter())), 2);
    assert!(ok(get(c, b)).is_none(), "deleted row must be gone");
    assert!(ok(get(c, a)).is_some());
    assert!(
        ok(search(
            c,
            &SearchFilter {
                query: "delete me".to_string(),
                ..filter()
            }
        ))
        .is_empty(),
        "FTS delete trigger must drop the row from the index"
    );

    ok(clear_all(c));
    assert_eq!(ok(count(c, &filter())), 0);
    assert!(ok(search(c, &filter())).is_empty());
}

#[test]
fn prune_respects_pins() {
    let s = store();
    let c = s.conn();
    let mut ids = Vec::new();
    for i in 0..5 {
        ids.push(ok(insert_dedup(c, &text(&format!("prune entry {i}")))));
    }
    let pinned_id = ids[2];
    ok(set_pin(c, pinned_id, true));

    // max_age_days=0 sets cutoff=now; created_at < now may or may not hold depending
    // on a second boundary, so assert on the surviving set rather than on age deletions.
    let deleted = ok(prune(c, 2, 0));
    let total = ok(count(c, &filter()));
    assert!(
        total <= 3,
        "at most the pinned row plus max_entries newest may survive"
    );
    assert!(total >= 1, "the pinned row must survive");
    assert!(
        deleted >= 2,
        "5 non-pinned rows with max_entries=2 must prune"
    );

    let pinned = some(ok(get(c, pinned_id)));
    assert!(pinned.pinned, "pinned row must still be present and pinned");
    assert_eq!(pinned.text_content.as_deref(), Some("prune entry 2"));
    assert!(
        ok(search(
            c,
            &SearchFilter {
                pinned_only: true,
                ..filter()
            }
        ))
        .len()
            == 1
    );
}

#[test]
fn hash_paths_and_dedup_key_are_deterministic() {
    assert_eq!(hash_text("hello"), hash_text("hello"));
    assert_ne!(hash_text("hello"), hash_text("world"));
    assert_eq!(
        hash_text("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(hash_bytes(b"abc"), hash_text("abc"));

    // hash_paths sorts, lowercases and dedups, so order/case cannot change the key.
    let a = vec!["C:\\A".to_string(), "c:\\b".to_string()];
    let b = vec!["c:\\b".to_string(), "C:\\a".to_string()];
    assert_eq!(hash_paths(&a), hash_paths(&b));
    assert_ne!(hash_paths(&a), hash_paths(&["C:\\A".to_string()]));

    assert_eq!(dedup_key(ContentKind::Text, "abc"), "text:abc");
    assert_eq!(dedup_key(ContentKind::Image, "abc"), "image:abc");
    assert_ne!(
        dedup_key(ContentKind::Text, "abc"),
        dedup_key(ContentKind::Image, "abc"),
        "identical bytes of different kinds must never collide"
    );
}
