//! v0.1.4 organization-feature integration tests: tags, bulk ops, per-type
//! retention, and portable JSON export/import. All use in-memory SQLite.
use clipvault::db::bulk::{
    TypeCaps, delete_many, export_json, import_json, prune_with_caps, set_favorite_many,
    set_pin_many, set_tags,
};
use clipvault::db::entries::insert_dedup;
use clipvault::db::search::{count, search};
use clipvault::db::store::Store;
use clipvault::domain::SearchFilter;
use clipvault::workers::{ClipCapture, ClipPayload};

fn ok<T, E: std::fmt::Display>(r: Result<T, E>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("expected Ok, got error: {e}"),
    }
}

fn cap(p: ClipPayload) -> ClipCapture {
    ClipCapture::new(p)
}

fn filter() -> SearchFilter {
    SearchFilter {
        page_size: 200,
        ..Default::default()
    }
}

#[test]
fn tags_roundtrip_and_filter() {
    let s = ok(Store::open_in_memory());
    let c = s.conn();
    let a = ok(insert_dedup(c, &cap(ClipPayload::Text("alpha".into()))));
    let b = ok(insert_dedup(c, &cap(ClipPayload::Text("beta".into()))));
    ok(set_tags(
        c,
        a,
        &["work".into(), "todo".into(), "work".into()],
    ));
    ok(set_tags(c, b, &["personal".into()]));

    let ea = ok(clipvault::db::entries::get(c, a)).expect("entry a");
    // set_tags normalizes: trim + dedup + sort.
    assert_eq!(ea.tags(), vec!["todo", "work"]);

    let tagged = ok(search(
        c,
        &SearchFilter {
            tag: Some("work".into()),
            ..filter()
        },
    ));
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].id, a);

    // "work" must not match "workflow".
    ok(set_tags(c, b, &["workflow".into()]));
    let tagged2 = ok(search(
        c,
        &SearchFilter {
            tag: Some("work".into()),
            ..filter()
        },
    ));
    assert_eq!(tagged2.len(), 1, "exact tag match must not match a prefix");

    // Clearing tags removes the filter match.
    ok(set_tags(c, a, &[]));
    assert_eq!(
        ok(search(
            c,
            &SearchFilter {
                tag: Some("work".into()),
                ..filter()
            }
        ))
        .len(),
        0
    );
}

#[test]
fn bulk_delete_and_flags() {
    let s = ok(Store::open_in_memory());
    let c = s.conn();
    let mut ids = Vec::new();
    for i in 0..5 {
        ids.push(ok(insert_dedup(
            c,
            &cap(ClipPayload::Text(format!("row {i}"))),
        )));
    }
    assert_eq!(ok(count(c, &filter())), 5);

    let n = ok(set_pin_many(c, &ids[..2], true));
    assert_eq!(n, 2);
    let m = ok(set_favorite_many(c, &ids[3..], true));
    assert_eq!(m, 2);

    let pinned = ok(search(
        c,
        &SearchFilter {
            pinned_only: true,
            ..filter()
        },
    ));
    assert_eq!(pinned.len(), 2);

    let deleted = ok(delete_many(c, &ids[..2]));
    assert_eq!(deleted, 2);
    assert_eq!(ok(count(c, &filter())), 3);
    // Deleted rows must also disappear from FTS5 (trigger).
    assert_eq!(
        ok(search(
            c,
            &SearchFilter {
                query: "row 0".into(),
                ..filter()
            }
        ))
        .len(),
        0
    );
}

#[test]
fn per_type_prune_keeps_pinned_and_respects_cap() {
    let s = ok(Store::open_in_memory());
    let c = s.conn();
    let mut ids = Vec::new();
    for i in 0..6 {
        ids.push(ok(insert_dedup(
            c,
            &cap(ClipPayload::Text(format!("text {i}"))),
        )));
    }
    // Pin the oldest so it must survive.
    ok(set_pin_many(c, &[ids[0]], true));
    let caps = TypeCaps {
        text: 3,
        image: 0,
        files: 0,
    };
    let removed = ok(prune_with_caps(c, 500, 0, caps));
    assert!(removed >= 2, "should prune down to the per-type cap");
    let rows = ok(search(c, &filter()));
    assert!(
        rows.iter().any(|e| e.id == ids[0]),
        "pinned row must survive"
    );
    // Up to 3 unpinned may remain (cap 3), plus the pinned one.
    assert!(rows.len() <= 4, "per-type cap enforced, got {}", rows.len());
}

#[test]
fn export_import_roundtrip_is_dedup_safe() {
    let s = ok(Store::open_in_memory());
    let c = s.conn();
    let a = ok(insert_dedup(
        c,
        &cap(ClipPayload::Text("shared text".into())),
    ));
    ok(set_tags(c, a, &["important".into()]));
    ok(set_pin_many(c, &[a], true));
    let json = ok(export_json(c, None));
    assert!(json.contains("clipvault-history"));
    assert!(json.contains("important"));

    // Import into a fresh DB, twice: dedup must not create duplicate rows.
    let s2 = ok(Store::open_in_memory());
    let c2 = s2.conn();
    assert_eq!(ok(import_json(c2, &json)), 1);
    assert_eq!(ok(import_json(c2, &json)), 1);
    assert_eq!(ok(count(c2, &filter())), 1, "re-import must dedup");
    let imported = ok(search(c2, &filter()));
    assert_eq!(imported[0].tags(), vec!["important"]);
    assert!(imported[0].pinned);
    assert_eq!(imported[0].text_content.as_deref(), Some("shared text"));
}

#[test]
fn import_rejects_invalid_backup() {
    let s = ok(Store::open_in_memory());
    assert!(import_json(s.conn(), "not json").is_err());
    assert!(
        import_json(
            s.conn(),
            r#"{"format":"other","version":1,"exported_at":0,"entries":[]}"#
        )
        .is_err()
    );
}
