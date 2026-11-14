# ClipVault v0.1.4 — Organization tools (plan)

Decided 2026-11-14: ship as **v0.1.4** with its own GitHub Release + CHANGELOG entry,
leaving v0.1.3 stable. Feature set: organization tools.

## Scope (5 features)
1. **Tags/labels per entry** — free-form tags on any entry; filter by tag; shown as
   chips in the history row + editable in the preview panel.
2. **Bulk select + multi-delete** — checkbox per row, "Select all"/"Clear selection",
   "Delete selected" (confirm dialog) and "Pin/Favorite selected" for convenience.
3. **Export / import history (JSON)** — portable, offline, no cloud. Export respects
   the current filter (or all); import dedups by `(kind, content_hash)` so re-import
   never duplicates. Image assets are exported as base64 so they survive the move.
4. **Per-type retention** — `Settings.max_entries_text/image/files` alongside the
   existing global `max_entries`; `prune()` applies the tighter per-type cap.
5. **List density toggle** — `Comfortable` (default) vs `Compact` row padding, in
   Settings and a toolbar button; persisted.

## Frozen interface additions
- `src/db/schema.rs`: `LATEST_VERSION = 5`; v5 migration adds
  `ALTER TABLE entries ADD COLUMN tags_json TEXT` + `ix_entries_tags` index.
  FTS5 triggers already cover text_content/file_paths; tags included in search
  via a `tags_json LIKE ?` clause (no FTS column change needed).
- `src/domain/entry.rs`: `ClipEntry.tags_json: Option<String>` (new field, all
  `row_to_entry` + `ClipEntry { .. }` constructors updated). Helper
  `ClipEntry::tags() -> Vec<String>` / `set_tags(&mut self, &[String])`.
- `src/domain/filter.rs`: `SearchFilter.tag: Option<String>` (default None);
  `ListDensity { Comfortable, Compact }` enum with `Display`.
- `src/config/settings.rs`: `Settings` gains `max_entries_text`,
  `max_entries_image`, `max_entries_files` (usize, 0 = use global cap),
  `list_density: ListDensity`, with `#[serde(default)]` on the struct so older
  `settings.json` files load cleanly.
- `src/workers/bus.rs`: `DbCommand::{SetTags(i64, Vec<String>), DeleteMany(Vec<i64>),
  SetPinMany(Vec<i64>, bool), SetFavoriteMany(Vec<i64>, bool), ExportAll(String),
  ImportAll(String), Prune {...}}`, `DbResult::{Exported(String), Imported(usize)}`.
  New `src/db/io.rs` for export/import serialization (pure functions, unit-tested).
  New `src/db/tags.rs` for set_tags / bulk ops (or extend entries.rs).
- `src/app/message.rs`: `SetTags(i64, Vec<String>)`, `ToggleSelect(i64)`,
  `SelectAll`, `ClearSelection`, `DeleteSelected`, `RequestDeleteSelected`,
  `ConfirmDeleteSelected`, `PinSelected(bool)`, `FavoriteSelected(bool)`,
  `ExportHistory`, `ImportHistory`, `SetListDensity(ListDensity)`,
  `SetMaxEntriesKind(ContentKind, usize)`, `FilterTag(Option<String>)`.
- `src/app/state.rs`: `selected_ids: std::collections::HashSet<i64>`,
  `show_delete_selected_confirm: bool`, `tag_filter: Option<String>`.
- GUI: history row gets a checkbox + tag chips; a selection toolbar appears when
  `selected_ids` is non-empty; settings view gets per-type caps + density picker +
  export/import buttons; new dialog for delete-selected confirm.
- Density: `src/gui/spacing.rs` gains `ROW_PAD_COMPACT: [u16;2]`;
  `history.rs` picks based on `state.settings.list_density`.

## Engineering constraints (unchanged)
- No `unwrap`/`expect` in runtime paths; `Result` everywhere.
- No blocking calls in `update()`/`view()` — all DB work via `DbCommand` to the
  single DB worker; file I/O for export/import ALSO on the worker (it owns disk).
- Two brand colors only (CREAM/INK) + neutrals; ASCII-only UI strings; `sp::*`
  tokens for all spacing/type.
- Versioned migration (v4→v5) must preserve rows; add a migration test.
- `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `cargo build --release`
  after each phase; full test suite before release.

## Phases (compile + test after each)
1. Docs (this file + SUMMARY + CHANGELOG skeleton) + version bump to 0.1.4.
2. Schema v5 + domain types + settings serde + migration test.
3. DB layer: tags CRUD, bulk delete/pin/favorite, per-type prune, export/import.
4. Worker bus + db_worker dispatch; unit tests for io/tags/prune.
5. Message/state/update wiring; selection + dialogs.
6. GUI: history checkbox/chips/toolbar, settings controls, search tag filter,
   density toggle, preview tag editor.
7. Full gate + release build + installer + install/run/uninstall + push + v0.1.4 Release.

## Known limitations (document, don't silently skip)
- Import re-writes image assets from base64 into the local asset dir; large
  histories produce large JSON. Cap export to a sane size (e.g. 50 MB) and report
  truncation rather than OOM.
- `tags_json` is a JSON array string, not a normalized join table — fine for a
  local single-user app; tag filtering uses LIKE and is case-insensitive.
- Per-type retention caps of 0 mean "use the global cap"; pinned/favorite rows are
  still exempt from pruning.
