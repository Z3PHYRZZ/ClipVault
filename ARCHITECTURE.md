# ClipVault — Architecture

> Production-quality Windows clipboard manager. Rust stable edition 2024, Iced GUI,
> SQLite (rusqlite, bundled), Windows clipboard/hotkey/tray via `windows-rs`.
> Owner: Manuel. Build orchestrated across a 40-agent fleet; single-writer integration.

## 1. Workspace layout

Single binary crate `clipvault` plus a platform-integration crate kept behind traits.

```
ClipVault/
  Cargo.toml                 # workspace + [package] clipvault, edition 2024
  build.rs                    # embeds manifest + icon resource (winres)
  ARCHITECTURE.md             # this file
  ROADMAP.md                  # 11 phases, status
  TASKS.md                    # task ledger
  SUMMARY.md                  # long-task state (goal, done, next)
  assets/
    icon.svg                   # brand-palette icon
    icon.ico                   # generated tray/application icon
  migrations/
    001_init.sql
    002_entry_metadata.sql
    003_fts5.sql
  src/
    main.rs                   # iced::application entry, tracing init, wiring
    lib.rs                     # re-exports, pub mod declarations
    app/
      mod.rs                  # AppState, Message enum, update(), view(), subscription()
      message.rs              # the single Message enum (all GUI events)
    config/
      mod.rs
      settings.rs             # Settings struct (serde), ThemeMode enum, defaults
    db/
      mod.rs
      schema.rs               # migration runner, user_version
      store.rs                 # Db handle, open(), WAL config
      entries.rs              # CRUD + duplicate detection + retention prune
      search.rs                # FTS5 query builder, filters (type/date/pin/fav)
      image_assets.rs          # asset dir, sha256-named PNG storage, load
    domain/
      mod.rs
      entry.rs                 # ClipEntry, ContentKind, FileMetadata, Preview
      hash.rs                  # content hashing (sha2) + dedup key
      filter.rs               # SearchFilter, ContentKind, DateRange
    platform/
      mod.rs                  # traits + handle types + cfg dispatch
      clipboard_ext.rs        # full clipboard read/write (text/image/files) -> ClipCapture
      clipboard.rs            # legacy text-only backend (kept, not dispatched)
      listener_full.rs        # AddClipboardFormatListener thread + self-filter
      source.rs              # copy provenance: owner process + CF_HTML SourceURL
      dib.rs                 # DIB/DIBV5 <-> PNG (headless-testable)
      hdrop.rs               # DROPFILES build/parse + file metadata
      format.rs              # CF_* constants + GetClipboardSequenceNumber
      hotkeys_full.rs        # RegisterHotKey thread + quick-paste
      tray_full.rs           # Shell_NotifyIconW + context menu + NIM_DELETE
      startup.rs             # HKCU Run registry value
      singleton.rs           # named-mutex single instance
      handles.rs             # ThreadHandle: WM_QUIT + join on shutdown
      icon.rs                # procedural RGBA icon + HICON/.ico
      winutil.rs             # message-only HWND + bounded OpenClipboard guard
    workers/
      mod.rs
      bus.rs                  # typed channels: Event/Command/Result
      db_worker.rs            # owns rusqlite::Connection, executes DbCommand
      supervisor.rs           # spawns/joins threads, graceful shutdown
    gui/
      mod.rs
      theme.rs                # brand palette, light/dark iced::Theme, custom styles
      widgets.rs              # rounded card, soft border, badge, empty-state
      views/
        mod.rs
        sidebar.rs
        history.rs
        search_bar.rs
        preview.rs
        settings.rs
        dialogs.rs
    icons/
      mod.rs                  # generate embedded tray/app icon (palette-based)
  tests/
    db_tests.rs
    search_tests.rs
    dedup_tests.rs
    settings_tests.rs
    platform_tests.rs        # skipped/non-windows guarded, clipboard gated
  .cargo/config.toml          # target + linker notes
  wix/ or installer notes     # packaging phase 11
```

## 2. Message-driven Iced architecture

One `Message` enum is the only thing that mutates `AppState`. `update()` is pure
and never blocks: it enqueues `DbCommand` onto an unbounded `flume`/`std::sync::mpsc`
channel to the DB worker, and returns a `Task<Message>` only for Iced commands.

Background → GUI communication uses a single `iced::Subscription` reading a
`mpsc::Receiver<WorkerEvent>` with `iced::futures` stream, mapping each event to a
`Message`. Threads: clipboard monitor, hotkey loop, tray window loop, DB worker.
All four are joined on shutdown via the supervisor; the DB worker flushes WAL
and checkpoints before exit (consistency requirement).

```
[Clipboard listener] ─┐
[Hotkey loop] ───────┼─► WorkerEvent ─► Subscription ─► Message ─► update()
[Tray loop] ─────────┤                                      │
[DB worker results] ─┘                                      ▼
                                                   DbCommand ─► [DB worker]
```

## 3. Windows integration (the hard constraints)

These are documented because they dictate the design and must not be silently skipped:

1. **Clipboard change notification is not a broadcast event.** Windows has no
   `WM_CLIPBOARDUPDATE` delivered to arbitrary processes without opting in.
   Correct mechanism: `AddClipboardFormatListener(hwnd)` on a message-only window,
   then handle `WM_CLIPBOARDUPDATE` in a dedicated thread with its own message
   loop (`GetMessageW`). This is event-driven — zero polling. The listener thread
   does NOT read the clipboard itself; it signals the monitor worker which calls
   `OpenClipboard` with **bounded retries** (e.g. 5 attempts, 15 ms backoff) because
   other processes hold the clipboard lock transiently (`OpenClipboard` fails with
   `ERROR_ACCESS_DENIED`/`ERROR_CLIPBOARD_NOT_OPEN` under contention).
   `GetClipboardSequenceNumber()` is used as a cheap change discriminator to reject
   self-induced updates (our own restoration) and duplicate notifications.

2. **Clipboard ownership & restoration.** `SetClipboardData` with `NULL` hMem uses
   delayed rendering and requires our HWND to stay alive as clipboard owner with a
   `WM_RENDERFORMAT` handler — fragile. Instead we use `EmptyClipboard` +
   `SetClipboardData` with **HGLOBAL-owned memory**, which transfers ownership to
   the system; no delayed rendering, no owner window needed. For text we set
   `CF_UNICODETEXT` (and let Windows synthesize `CF_TEXT`). For images we set
   `CF_DIB` / `CF_DIBV5` (DIB, not `CF_BITMAP` which is device-dependent). For
   files we set `CF_HDROP` with a correctly laid-out `DROPFILES` + double-NUL-
   terminated wide path list. `GlobalAlloc(GMEM_MOVEABLE)` + `GlobalLock/Unlock`;
   after a successful `SetClipboardData`, ownership transfers — do NOT `GlobalFree`.

3. **Unicode text.** Always `CF_UNICODETEXT` (UTF-16). `WideCharToMultiByte`/
   `MultiByteToWideChar` only where narrow formats are unavoidable; Rust `String`
   (UTF-8) conversion via `String::from_utf16_lossy`.

4. **Image formats.** Clipboard bitmaps arrive as `CF_DIB`/`CF_DIBV5` (a
   `BITMAPINFOHEADER` followed by pixel data, possibly with a color table and,
   for 32bpp BI_BITFIELDS, channel masks). We store a normalized PNG (via the
   `image` crate) in the asset dir keyed by sha256 of the DIB bytes; the DB stores
   the path + hash + dimensions. Preview loads the PNG through `iced::widget::image`.
   `CF_BITMAP` (HBITMAP) is converted via `GetDIBits` when it is the only format.

5. **File drop lists.** `CF_HDROP` → `DragQueryFileW` for count + paths, plus
   `GetFileAttributesW`/`GetFileSizeEx` for metadata. **We store paths + metadata
   only — never file contents.** Paths may vanish; preview re-checks existence and
   shows a "missing" state rather than failing.

6. **Global hotkeys.** `RegisterHotKey(hwnd, id, modifiers, vk)` on the same
   message-only window / its own thread; `WM_HOTKEY` → `WorkerEvent::Hotkey(id)`.
   `MOD_CONTROL|MOD_SHIFT|...` + virtual-key codes. `UnregisterHotKey` on shutdown.
   Registration can fail with `ERROR_HOTKEY_ALREADY_REGISTERED` → surface a clear
   settings error, bounded retry, never crash. Only one process can own a combo.

7. **System tray.** `Shell_NotifyIconW(NIM_ADD, NOTIFYICONDATAW)` with a message-only
   HWND and a callback message (`WM_APP+1`) carrying the mouse event; on
   `WM_RBUTTONUP`/`WM_LBUTTONUP` we `TrackPopupMenu` with Show / Settings /
   Pause monitoring / Quit. Icons are generated programmatically into an HICON
   from a small RGBA buffer in the brand palette (no external asset dependency).
   `NIM_DELETE` on shutdown (tray icons otherwise linger until explorer refresh).

8. **Startup.** `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value
   `ClipVault` = quoted exe path. Read/write via `windows-registry` crate.
   Idempotent, reversible; removal deletes only our value.

9. **Background startup / single instance.** Use a named mutex
   (`CreateMutexW("ClipVaultSingleton")`) — if `ERROR_ALREADY_EXISTS`, the second
   instance exits after optionally signalling the first to show its window
   (via a registered window message + `HWND_BROADCAST`). Prevents two clipboard
   listeners and two hotkey owners racing.

10. **Threading.** Win32 clipboard, hotkey, and tray windows are thread-affine:
    each subsystem owns its thread and its HWND; `PostThreadMessageW`/`PostMessageW`
    crosses the boundary. Never touch an HWND from the GUI thread. All `windows-rs`
    calls live in `src/platform/`; the rest of the crate talks to traits so the
    code compiles (with `#[cfg(not(windows))]` stubs) and unit tests run headless.

## 4. Database

- `rusqlite` with the `bundled` feature (no system SQLite dependency, deterministic).
- WAL mode, `synchronous = NORMAL`, `foreign_keys = ON`, `busy_timeout = 5000`.
- `PRAGMA user_version` drives migrations; each migration is one transaction.
- Schema (v1): `entries(id INTEGER PK, kind TEXT NOT NULL, content_hash TEXT NOT NULL,
  text_content TEXT, image_path TEXT, image_w INTEGER, image_h INTEGER,
  file_meta TEXT, file_paths TEXT, created_at INTEGER NOT NULL,
  last_used_at INTEGER NOT NULL, use_count INTEGER NOT NULL DEFAULT 1,
  pinned INTEGER NOT NULL DEFAULT 0, favorite INTEGER NOT NULL DEFAULT 0,
  source_app TEXT, source_url TEXT, tags_json TEXT, byte_size INTEGER NOT NULL)`.
- Migration **v4** adds `source_url` (`ALTER TABLE ... ADD COLUMN`) with a partial
  index; `source_app` has existed since v1. Both hold best-effort copy provenance
  captured from the clipboard owner process / browser `CF_HTML` `SourceURL:` header.
- Migration **v5** (v0.1.4) adds `tags_json` (JSON array of strings) with a partial
  index; `src/db/bulk.rs` holds tag CRUD, bulk delete/pin/favorite, per-type
  retention, and portable JSON export/import (`src/db/io.rs`, own base64 codec,
  50 MB cap). Import goes through `insert_dedup` so `(kind, content_hash)` dedup holds.
- Indexes: `content_hash` (dedup, UNIQUE-ish with kind), `created_at DESC`,
  `kind`, `pinned`, `favorite`, `tags_json`. FTS5 virtual table `entries_fts(text_content,
  file_paths, content='entries')` kept in sync by triggers for fast substring/token
  search; LIKE fallback for CJK/prefix queries and exact tag matching.
- **Duplicate detection:** hash = sha256 over canonical bytes (UTF-8 text,
  original DIB bytes for images, normalized sorted path list for files). An existing
  row with the same `(kind, content_hash)` and `use_count` bump + `last_used_at`
  refresh; no new row. A UNIQUE index on `(kind, content_hash)` plus
  `INSERT ... ON CONFLICT DO UPDATE` makes this race-safe.
- **Retention:** configurable max entries and/or max age days; a `prune` command
  deletes oldest non-pinned, non-favorite rows, then asset files that are no longer
  referenced (after the transaction commits). Runs on a slow timer / at startup.
- **No unwrap/expect** in runtime paths: all DB calls return `Result<T, DbError>`;
  the worker maps errors to `WorkerEvent::Error` which the GUI shows as a toast.

## 5. Theme & palette (exact two brand colors)

Design tokens live in `src/gui/theme.rs` (palette/styles), `src/contracts.rs`
(color + frosted-glass tokens), and `src/gui/spacing.rs` (spacing, radii, type
scale — the single source of truth). Every view uses `sp::*` constants on a 4px
grid (4/8/12/16/24/32) rather than magic numbers, with a type hierarchy of
28/21/16/14/12 px. Rounded cards (16px), buttons (12px), badges (9px).

**Frosted-glass theme (v0.1.5):** `src/gui/theme.rs::ambient_background()` returns a
static `Background::Gradient` (3 stops) placed behind all UI layers. Surfaces are
alpha-tinted CREAM/INK glass — deeper (`~0.78-0.80`) for sidebar/search/toolbar,
softer (`~0.64-0.66`) for cards/preview/status — with 1px specular WHITE borders
(0.40 light / 0.08 dark) and diffused shadows (20px panel / 14px card). Buttons,
inputs, and dropdowns are crisp semi-transparent pills. Hover lifts the shadow
offset (`translateY(-1px)` equivalent) and thickens the border; no animated blur.
No compositor blur filter exists in Iced 0.13, and native DWM Mica is not applied
because `iced_winit` hides the HWND — documented in `V015_FROSTED_PLAN.md`. All
frost tokens are alpha/tint derivatives of CREAM/INK/WHITE, not new hues. Text
contrast is >= 6.28:1 on buttons and >= 11.7:1 on glass in both themes.

Pin/favorite state reads through an INK fill + 2px border (`active_button_style`)
instead of glyphs —
Iced's bundled Fira Sans has no star/dingbat glyphs and renders them as tofu, so
ALL user-visible UI strings are plain ASCII.

| Role | Hex | RGB |
|---|---|---|
| Cream (light bg / cards / light text / selection) | `#F2E9E4` | 242,233,228 |
| Ink (panels / buttons / active / borders / dark surfaces / text) | `#22223B` | 34,34,59 |

Light: CREAM background, INK panels, CREAM cards, INK interactive, INK typography.
Dark: INK bg, CREAM accents, CREAM text — **not** a simple inversion.
`iced::Theme::custom` with a `palette::Palette` derived
from these hexes; all widget styling goes through `gui/theme.rs` so no stray color
ever enters. Neutral white/charcoal/grayscale only for contrast. Rounded cards
(radius 12), 1px soft borders (Mauve at low alpha), subtle shadow via layered
container, spacing on a 4/8/16/24 grid, clear type hierarchy.

## 6. Performance

- Idle: listener blocks in `GetMessageW` (no CPU), DB worker blocks on channel
  recv, no polling timers except a single slow retention tick (configurable, ≥ minutes).
- Images stored on disk, only the selected entry's PNG is loaded into an iced
  `image::Handle`; history rows render thumbnails lazily from a small LRU cache.
- History view is virtualized/paged (page size from settings); never materialize
  the full blob set at once.
- `Arc<str>` / `Rc` where shared; avoid cloning text blobs across messages.
- `windows_subsystem = "windows"` on the binary crate: the PE is linked as a
  WINDOWS/GUI subsystem app (subsystem 2), so double-click, Start Menu, and
  startup launches never spawn a conhost/cmd window. Because a GUI-subsystem
  process has no stderr, `tracing` writes to `%LOCALAPPDATA%\ClipVault\clipvault.log`
  via an `Arc<File>` MakeWriter instead of a console writer — diagnostics survive
  without a console. `+crt-static` in `.cargo/config.toml` avoids a VC++ redist.
- `tracing` with a file subscriber; debug logging gated by settings;
  no telemetry, no network crates.

## 7. Error handling & tests

`Result`-based throughout (`thiserror` for domain error enums). `anyhow` only in
`main`. Unit tests: hash/dedup, filter builder, settings serde round-trip, migration
versioning, DIB parse (synthetic bytes), DROPFILES layout. Integration tests:
in-memory SQLite CRUD + FTS search + retention + prune. Platform tests are
`#[cfg(windows)]`-gated and clipboard-touching tests are `#[ignore]` by default
so CI never races the real clipboard. `cargo clippy -D warnings`, `cargo fmt --check`,
`cargo test` gate every phase.

## 8. Known limitations & practical resolutions

| Limitation | Resolution |
|---|---|
| No global clipboard "event" without a listener window | `AddClipboardFormatListener` + message-only window thread (phase 4) |
| Clipboard lock contention | bounded retries with backoff; never spin |
| `CF_HDROP` paths can go stale | store paths + metadata only, re-stat on preview, show missing state |
| One owner per global hotkey combo | detect `ERROR_HOTKEY_ALREADY_REGISTERED`, surface in Settings, allow rebind |
| Tray needs a window + message loop | dedicated tray thread with `HWND_MESSAGE` parent |
| Iced has no built-in tray | tray is a separate Win32 thread wired to the app via `WorkerEvent` |
| `windows-rs` needs MSVC linker | document `vs_buildtools` requirement; `x86_64-pc-windows-msvc` target |
| Edition 2024 requires Rust ≥ 1.85 | pin `rust-toolchain.toml` to stable ≥ 1.85 |

## 9. Packaging (phase 11)

MSI via `cargo wix`/WiX or a self-contained zip with the exe + `vcredist`
assumption. Icon embedded with `winres`. `cargo build --release`, strip symbols,
`cargo test --release`. Installer writes no registry keys except our own Run value.
