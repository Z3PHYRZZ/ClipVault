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
    icon.svg                   # pastel rose brand mark
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
      mod.rs                  # traits: ClipboardAccess, HotkeySource, TrayIcon, Startup
      clipboard.rs            # windows-rs clipboard (text/Unicode/image/files)
      clipboard_monitor.rs    # event-driven listener + worker thread
      hotkeys.rs              # RegisterHotKey message-loop thread
      tray.rs                 # Shell_NotifyIcon + hidden message window + menu
      startup.rs              # HKCU Run registry key
      winutil.rs              # HWND_MESSAGE window, WM_* helpers, scoped guards
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
  source_app TEXT, byte_size INTEGER NOT NULL)`.
- Indexes: `content_hash` (dedup, UNIQUE-ish with kind), `created_at DESC`,
  `kind`, `pinned`, `favorite`. FTS5 virtual table `entries_fts(text_content,
  file_paths, content='entries')` kept in sync by triggers for fast substring/token
  search; LIKE fallback for CJK/prefix queries.
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

## 5. Theme & palette (exact five brand colors)

Design tokens live in `src/gui/theme.rs` (palette/styles) and `src/gui/spacing.rs`
(spacing, radii, type scale — the single source of truth). Every view uses
`sp::*` constants on a 4px grid (4/8/12/16/24/32) rather than magic numbers, with
a type hierarchy of 24/18/14/13/11 px. Rounded cards (14px), buttons (10px),
badges (8px), soft 1px Mauve-alpha hairline borders, subtle 0.08-alpha shadows,
and hover/press color blends toward PINK/PEACH/MAUVE. Pin/favorite state reads
through Dusty Rose fill + 2px border (`active_button_style`) instead of glyphs —
Iced's bundled Fira Sans has no star/dingbat glyphs and renders them as tofu, so
ALL user-visible UI strings are plain ASCII.

| Role | Hex | RGB |
|---|---|---|
| Sage (light bg) | `#D8E2DC` | 216,226,220 |
| Peach (secondary surfaces) | `#FFE5D9` | 255,229,217 |
| Pastel Pink (primary accent / selection) | `#FFCAD4` | 255,202,212 |
| Dusty Rose (buttons / active / hover) | `#F4ACB7` | 244,172,183 |
| Muted Mauve (secondary text / icons / borders) | `#9D8189` | 157,129,137 |

Light: Sage background, Peach panels, warm-neutral/white cards, Rose interactive,
Mauve typography. Dark: deep charcoal (`#1E1B1D`, `#262226`, `#2E282C` — permitted
neutrals) with Sage/Pink/Rose accents and a lighter Mauve (`#C9AEB5`) for text;
**not** a simple inversion. `iced::Theme::custom` with a `palette::Palette` derived
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
