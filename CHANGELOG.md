# Changelog

All notable changes to ClipVault are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2026-11-14

Frosted-glass theme refinement. New GitHub Release tag `v0.1.5`; v0.1.4 stays stable.

### Changed (theme)
- **Soft ambient gradient base** behind all UI layers (`Background::Gradient` /
  `gradient::Linear`, 3 static stops) — depth without backdrop blur or clutter.
- **Frosted surface hierarchy**: sidebar / top search bar / selection toolbar get
  a deeper tinted glass (CREAM/INK at ~78-80% alpha + diffused 20px shadow);
  entry cards / preview / status bar get a softer ~64-66% glass with a 14px shadow.
- **1px specular borders**: `alpha(WHITE, 0.40)` in Light, `alpha(WHITE, 0.08)` in
  Dark, plus contrasting INK/CREAM hairlines separating overlapping layers.
- **Buttons, inputs, dropdowns**: crisp semi-transparent pills with specular edges
  (`theme::input_style` / `pick_list_style`), blended into the frosted containers.
- **Hover elevation**: shadow lifts (equivalent of `translateY(-1px)`), border
  thickens, tint shifts — message-driven, no animated blur.
- **Readability**: text contrast verified >= 6.28:1 on buttons and >= 11.7:1 on
  glass surfaces in both themes, across the whole gradient (WCAG AA is 4.5:1).
  Semi-transparent button fills are composited over the ambient base before the
  text-color decision, preventing invisible dark-on-dark text.
- **Performance**: no compositor blur filter at all; static gradients, constant
  alpha tints, and fixed-radius shadows only. Hover changes color/offset/border,
  never a blur radius, so scrolling and window resize stay smooth.

### Known limitation
- **True OS-level Mica/Acrylic translucency is NOT applied.** Iced 0.13 hides the
  HWND (`iced_winit` keeps it internal); `DwmSetWindowAttribute(
  DWMWA_SYSTEMBACKDROP_TYPE, DWMSBT_MAINWINDOW)` would require patching the GUI
  framework or a fragile title-based `FindWindowW` race. Mica is also Windows 11
  only. The ambient gradient + layered alpha-tinted surfaces are the
  framework-supported approximation and keep Windows 10 compatibility.
- Two brand colors only: CREAM `#F2E9E4` + INK `#22223B`, with WHITE/CHARCOAL
  neutrals. Frost tokens are alpha/tint derivatives, not new hues.

## [0.1.4] - 2026-11-14

Organization tools. New GitHub Release tag `v0.1.4`; v0.1.3 stays stable.

### Added
- **Tags/labels per entry** — free-form comma-separated tags, normalized
  (trim + dedup + sort), stored as a JSON array in SQLite schema **v5**
  (`tags_json` column + partial index). Shown as `#tag` chips in history rows,
  editable in the preview panel, and filterable from the search bar's tag picker.
- **Bulk select + multi-delete** — per-row checkbox, "Select all" / "Clear",
  "Delete selected" (confirm dialog), plus "Pin" / "Favorite" for the selection.
  All bulk ops run in one transaction on the DB worker; `entries_fts` stays in
  sync via the existing triggers.
- **Portable JSON export / import** — offline, no cloud. Export writes
  `%LOCALAPPDATA%\ClipVault\clipvault-export.json` with images inlined as base64;
  a 50 MB cap prevents OOM on large image histories. Import is dedup-safe by
  `(kind, content_hash)` and restores tags + pin/favorite flags. New `src/db/io.rs`
  (own base64 codec, format/version validation) + `src/db/bulk.rs`.
- **Per-type retention** — `Settings.max_entries_text/image/files` (0 = global
  cap). `prune_with_caps()` applies the tighter per-type cap after the global
  age/count prune; pinned and favorite rows remain exempt.
- **List density toggle** — Comfortable (default) vs Compact row padding, in
  Settings and a status-bar button; persisted via `#[serde(default)]` so older
  `settings.json` files load cleanly (all new Settings fields default).
- 9 new tests (4 `io.rs` unit, 1 `bulk.rs` unit, 5 `tests/org_tests.rs`
  integration incl. v4->v5 row-preserving migration). **49 tests total.**

### Changed
- `SearchFilter.tag: Option<String>`; `ClipEntry.tags_json` + `tags()`/`set_tags()`.
- `DbCommand`/`DbResult` extended for tags, bulk ops, export/import, capped prune;
  `prune` now flows through `bulk::prune_with_caps`.
- GUI: history checkbox lives outside the row's `mouse_area` so selection clicks
  never race `SelectEntry`; selection toolbar appears only when rows are checked.

## [0.1.3] - 2026-11-13

### Fixed (post-release audit)
- **Global show/hide hotkey was a no-op**: `ShortcutAction::ShowHide`, tray
  `TrayAction::Show`, and `Message::ShowWindow`/`HideWindow` did nothing. Now
  `window::get_oldest()` -> `window::minimize(id, bool)` (and `gain_focus` on
  show), with `AppState.window_hidden` tracking the state. Documented gap:
  `start_minimized` remains stored-but-unenforced because Iced 0.13 has no true
  "start hidden in tray" primitive.
- Exhaustive-ultracode audit: verified release asset sha256 against GitHub,
  no clipboard text in logs, CF_HTML/HGLOBAL/GDI ownership, WCAG AA contrast,
  and corrected stale "35 tests" claims to 40.

### Changed (UI rework)
- **Bigger, readable type**: raised the scale to display 28 / title 21 / body 16 /
  label 14 / caption 12 px (`src/gui/spacing.rs`). Roomier card/button/panel/
  content/status padding ([12,16] -> [16,24] panels, 32px content), rounder radii
  (card 16, button 12, badge 9, input 12), larger 20px section gaps.
- **Buttons**: bigger hit areas (`BUTTON_PAD [12,16]`), 2px borders on primary,
  clearer active/hover/pressed blends, consistent `sp::*` sizing everywhere.
- **Message-driven hover animation**: `mouse_area.on_enter/on_exit` on history
  cards and every sidebar nav/bottom item flips `AppState.hovered_card` /
  `hovered_nav`; `card_hover_style` / `nav_hover_style` lift the shadow by 3px,
  widen the border to 2px and blend the background. Styles recompute per frame.
- New `hover_card` / `hover_button` widget helpers; `HoverCard(Option<i64>)` /
  `HoverNav(Option<u8>)` messages + state fields in all three `AppState`
  constructors (state / supervisor / main fallback).
- **Contrast fix (reviewed)**: button/panel/badge text is now chosen from the
  ACTUAL background luminance (`button_text(bg)`), removing invisible
  INK-on-INK text on dark-theme primary/active buttons and CREAM-on-CREAM on
  dark panels. All combinations now >= 4.5:1 (WCAG AA).

### Added
- **Photo preview**: image entries now show a lazy-loaded thumbnail (52x42) in
  the history list via `iced::image::Handle::from_path` — no GUI-thread file read.
  The preview panel renders the photo at 300px with `ContentFit::Contain` from
  either the loaded PNG bytes or the asset path, and shows the image dimensions.
- **Copy provenance**: each capture records where it came from.
  - `source_app`: the clipboard owner process (e.g. `chrome.exe`), resolved via
    `GetClipboardOwner` -> `GetWindowThreadProcessId` -> `OpenProcess` ->
    `QueryFullProcessImageNameW` (basename only, no private paths stored).
  - `source_url`: the web page URL, parsed from the browser's `CF_HTML`
    `SourceURL:` header (`RegisterClipboardFormatW("HTML Format")`), with a
    fallback to a bare `http(s)://` URL in the copied text.
  - Both are best-effort and NULL when unknown. Stored in SQLite (schema **v4**,
    `ALTER TABLE ... ADD COLUMN source_url`, partial index) and shown in the
    history subtitle + a dedicated SOURCE section in the preview.
- `src/platform/source.rs` (provenance helper, 3 unit tests) and
  `src/workers::ClipCapture` (payload + provenance wrapper). `ClipboardAccess::read`
  now returns `ClipCapture`; provenance is captured while the clipboard is still
  open, since `GetClipboardData`/`GetClipboardOwner` require the open lock.
- DB tests for provenance persistence/refresh + 40 total automated tests.

### Changed
- **Brand palette replaced with two colors** across the entire app (GUI theme,
  tray icon, app icon, badges, buttons, borders, shadows, docs):
  Cream `#F2E9E4` and Ink `#22223B`. No other chromatic colors — neutral
  white/charcoal for text contrast, alpha variants of the two for hairlines/shadows.
- Light theme: Cream background / Ink panels, Ink text. Dark theme: Ink ground /
  Cream accents, Cream text. Complementary, not inverted.
- `src/contracts.rs` exports `CREAM`/`INK` (+ RGB tuples, `WHITE`/`CHARCOAL`
  neutrals, `alpha()` helper) with backward-compatible aliases so every module
  resolves to the two colors without API churn. `src/gui/theme.rs` fully
  token-driven; button text picks contrast per surface.
- `src/platform/icon.rs` redrawn with only the two colors: Ink rounded square,
  Cream card, Ink header + "V", Cream outline.
- Version bumped to 0.1.5 in `Cargo.toml`, `build.rs` (FileVersion/ProductVersion),
  Inno Setup script, installer filename, and all docs.

## [0.1.2] - 2026-11-13

### Changed
- Brand palette reduced to two colors: Pastel Pink `#FFCAD4` and Muted Mauve
  `#9D8189`. `contracts.rs` exported `PINK`/`MAUVE`, `theme.rs` token-driven,
  icon redrawn with the two. Version 0.1.2.

## [0.1.1] - 2026-11-13

### Changed
- Brand palette replaced across the entire app with the five-color coolors set:
  Cream `#F2E9E4`, Rose `#C9ADA7`, Mauve `#9A8C98`, Slate `#4A4E69`,
  Ink `#22223B` — https://coolors.co/palette/22223b-4a4e69-9a8c98-c9ada7-f2e9e4.
  `src/contracts.rs` exported `CREAM/ROSE/MAUVE/SLATE/INK`, `theme.rs` became
  token-driven, and the icon was redrawn with all five. Version 0.1.1.

## [0.1.0] - 2026-11-13

First public release. A complete, private Windows clipboard manager built in Rust
(edition 2024) with Iced 0.13, rusqlite (bundled SQLite + FTS5), and windows-rs 0.62.

### Added

**Clipboard capture**
- Text (`CF_UNICODETEXT`, UTF-16 -> UTF-8), images (`CF_DIB` / `CF_DIBV5` -> PNG),
  and file drop lists (`CF_HDROP` / `DROPFILES` -> paths + metadata).
- Event-driven monitoring via `AddClipboardFormatListener` on a message-only window —
  zero polling, ~0% idle CPU.
- Bounded `OpenClipboard` retries (5 attempts, 15 ms backoff) for lock contention.
- `GetClipboardSequenceNumber` self-filter so restoring an entry is not re-captured.
- HGLOBAL-ownership-correct writes (no delayed rendering, no owner HWND needed).
- DIB parser handles BITMAPINFOHEADER / BITMAPV5HEADER, palettes (1/4/8bpp),
  BI_BITFIELDS masks, 16/24/32bpp, top-down (negative height) and DWORD-aligned stride.

**Storage & search**
- Versioned SQLite schema (migrations v1 -> v3) with `PRAGMA user_version`.
- WAL mode, `synchronous=NORMAL`, `busy_timeout`, `foreign_keys=ON`.
- SHA-256 content hashes with a `UNIQUE(kind, content_hash)` index and race-safe
  `ON CONFLICT DO UPDATE` upsert — repeats bump `use_count`, never create duplicates.
- FTS5 virtual table kept in sync by insert/delete/update triggers, plus a LIKE
  fallback for substrings and CJK.
- Filters: text query, content kind, date range, pinned-only, favorites-only, paging.
- Configurable retention pruning (max entries + max age days) that respects pinned
  and favorite rows; orphaned image assets are garbage-collected.

**GUI**
- Original five-color pastel-rose palette (replaced in 0.1.5): Sage `#D8E2DC`,
  Peach `#FFE5D9`, Pastel Pink `#FFCAD4`, Dusty Rose `#F4ACB7`, Muted Mauve `#9D8189`.
- Light theme (CREAM background, ROSE panels, CREAM cards) and a complementary
  dark theme (deep charcoal, muted rose/sage accents) — not a simple inversion.
- Token-driven spacing system (`src/gui/spacing.rs`): 4px grid, 14/10/8px radii,
  24/18/14/13/11px type scale; rounded cards, soft borders, subtle shadows,
  hover/press transitions, clear empty states.
- Pin, favorite, delete, clear-all (confirm dialog), restore-to-clipboard actions.
- Text / image / file-metadata preview panel.

**Windows integration**
- Global hotkeys via `RegisterHotKey` (show/hide + `Ctrl+Shift+1..9` quick-paste),
  with a clear rebind hint when a combo is already owned by another app.
- System tray via `Shell_NotifyIconW` with a context menu (Show / Settings /
  Pause / Resume / Quit) and `NIM_DELETE` on shutdown.
- Launch-at-startup via the `HKCU\...\Run\ClipVault` registry value.
- Named-mutex single instance; second instance signals the first and exits.
- Procedurally generated brand icon (32x32 RGBA), embedded `.ico` + version resource
  via `winres`, and a runtime `HICON` for the tray.
- GUI subsystem (`windows_subsystem = "windows"`) — no console window on launch;
  `tracing` writes to `%LOCALAPPDATA%\ClipVault\clipvault.log`.
- Graceful shutdown: `WM_QUIT` posted to every worker thread, DB worker runs
  `wal_checkpoint(TRUNCATE)` before exit.

**Settings, privacy, engineering**
- Persistent settings (serde JSON, atomic temp+rename write): theme, retention,
  pause monitoring, clear-on-exit, start-minimized, launch-at-startup, shortcut,
  ignored apps, page size.
- No network dependency, no telemetry. Local-only storage.
- Message-driven Iced architecture; no blocking calls in the GUI event loop;
  all DB / clipboard / tray / hotkey work on dedicated threads joined on shutdown.
- `Result`-based error handling throughout; no `unwrap` / `expect` in runtime paths.
- 35 automated tests (26 unit + 9 integration) covering migrations, dedup, FTS5
  search, filters, prune, DIB encode/decode, DROPFILES round-trip, hashing, icons.
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean.

**Packaging & docs**
- Inno Setup 6 installer (`installer/ClipVault.iss`): per-user, no admin, x64,
  Win10+, Start Menu + optional desktop shortcut, optional startup task, clean
  uninstall that preserves clipboard history. One-click `installer/build-installer.bat`.
- `ARCHITECTURE.md`, `README.md`, `PACKAGING.md`, `ROADMAP.md`, `TASKS.md`,
  `CONTRACT.md`, `FLEET.md`, `SUMMARY.md`, `UI_POLISH_BRIEF.md`, `CHANGELOG.md`.

### Fixed
- Unicode glyphs (`★ ☆ ✕ ◀ ▶ × · …`) rendered as tofu boxes with Iced's bundled
  Fira Sans — replaced with plain ASCII labels (`Pin`/`Unpin`, `Fav`/`Unfav`,
  `Copy`, `Delete`, `Prev`/`Next`) and color/border-based state styling.
- Quit previously tore down workers but left the Iced window open with dead workers —
  now calls `iced::window::close`.
- Quick-paste indices were emitted but ignored — now restore the nth visible entry.
- `clear_on_exit` was stored but not enforced — now clears on quit.
- Duplicate initial `Search` command on startup removed.
- `AppState::new()` failure no longer panics (falls back to a visible error state).
- Console window spawned on every launch — fixed via the GUI subsystem attribute.

### Known limitations
- Manual desktop smoke tests (real clipboard, tray menu, hotkeys) require a live
  Windows session; not automatable headlessly.
- `signal_existing()` broadcasts a "show" message, but a running instance does not
  yet register a receiver (message-only windows don't receive `HWND_BROADCAST`).
- `ignored_apps` is stored but not enforced in the listener (pause is enforced).
- `CF_BITMAP` (device-dependent HBITMAP) is intentionally skipped; DIB/DIBV5 are
  preferred. RLE-compressed DIBs are rejected.

[0.1.5]: https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.5
[0.1.4]: https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.4
[0.1.3]: https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.3
[0.1.2]: https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.2
[0.1.1]: https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.1
[0.1.0]: https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.0
