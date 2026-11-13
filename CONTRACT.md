# ClipVault — Frozen Interface Contract (v1)

> **Every agent MUST read this before writing code.** These signatures are frozen.
> You own ONLY the files listed in your task. Never edit `src/main.rs`, `src/lib.rs`,
> `Cargo.toml`, `CONTRACT.md`, or another agent's files. If you need a change to a
> frozen signature, report it in your final message — do NOT edit it.
> Integrator (main loop) compiles and welds across waves.

## Toolchain (CONFIRMED, do not change)
- `rustc 1.99.1`, edition 2024, `x86_64-pc-windows-msvc`, Windows 11.
- Rust at `C:\Users\Dodot\.cargo\bin`. Every cargo/rust CLI must prepend:
  `export PATH="/c/Users/Dodot/.cargo/bin:$PATH"` then `cd "D:/job done/ClipVault"`.
- `iced = "0.13.1"` with features `["image","svg"]`. Iced 0.13 API is confirmed below.
- `rusqlite 0.40.2` (bundled, fts5 available via `rusqlite::ffi` — bundled SQLite has FTS5).
- `windows 0.62.2`, `windows-registry 0.6.1`, `image 0.25.9`, `sha2 0.11`,
  `serde 1`, `serde_json 1`, `tracing 0.1`, `tracing-subscriber 0.3`,
  `thiserror 2`, `anyhow 1`, `chrono 0.4`, `flume 0.12`, `dirs 7`, `uuid 1` (v4),
  `bitflags 2`, `tempfile 3` (dev), `winres 0.1` (build).

## Iced 0.13.1 API (compile-proven — use exactly this)
```rust
// main.rs pattern:
iced::application("ClipVault", update, view)
    .theme(theme)                       // fn(&State) -> Theme
    .subscription(subscription)          // fn(&State) -> Subscription<Message>
    .window(iced::window::Settings { size: Size::new(1100.0,720.0),
             min_size: Some(Size::new(820.0,560.0)), ..Default::default() })
    .run()
// State MUST impl Default. update returns iced::Task<Message> (Task::none()).
// view returns iced::Element<'_, Message, Theme, Renderer> — use `Element<'_, Message>`.
```
- `Theme::custom(name: String, palette: iced::theme::Palette)` — Palette fields:
  `background, text, primary, success, danger: iced::Color`.
- `Color::from_rgb8(r,g,b)`, `Color::from_rgba8(r,g,b,a)`, `Color::from_rgb(f32...)`.
- `iced::widget::{container, column, row, button, text, text_input, scrollable,
   checkbox, pick_list, Space, image, svg, mouse_area, stack, vertical_rule, rule}`.
- `container(x).padding(..).width(Length::..).height(..).center(..)
   .style(|theme: &Theme| container::Style { .. })` — Style fields:
  `text_color: Option<Color>, background: Option<Background>, border: Border, shadow: Shadow`.
- `button(x).on_press(Msg).style(|theme: &Theme, status: button::Status| button::Style {..})` —
  Style fields: `background: Option<Background>, text_color: Color, border: Border, shadow: Shadow`.
  `button::Status::{Active, Hovered, Pressed, Disabled}`.
- `iced::Border { radius: Radius::from(12.0).into() ... }` — use
  `iced::border::Radius` via `iced::Border { radius: 12.0.into(), width: 1.0, color: ... }`.
- `iced::widget::image::Handle::from_bytes(bytes)` / `::from_rgba(w,h,Vec<u8>)` /
  `::from_path(PathBuf)`. `image(handle).width(..).height(..)`.
- `iced::window::icon::from_rgba(Vec<u8>, w, h)` -> `Result<Icon, Error>`; set
  `window::Settings { icon: Some(icon), .. }`.
- **Subscription from a channel** (confirmed `iced::futures` + `iced::stream` are re-exported):
```rust
fn subscription(state: &AppState) -> iced::Subscription<Message> {
    let rx = state.events_rx.clone(); // flume::Receiver<WorkerEvent>
    iced::Subscription::run_with_id(
        "clipvault-workers",
        iced::stream::channel(100, move |mut output| async move {
            loop {
                match rx.recv_async().await {
                    Ok(ev) => { let _ = output.send(Message::Worker(ev)).await; }
                    Err(_) => break,
                }
            }
        }),
    )
}
```
  `iced::stream::channel` + `iced::futures::SinkExt` for `output.send(..).await`.
- `Task::perform(async { ... }, Message::Variant)` for async work if needed (avoid; use workers).
- `iced::Subscription::batch(vec![...])` to combine (e.g. worker stream + slow retention tick
  via `iced::time::every(Duration::from_secs(600)).map(|_| Message::RetentionTick)`).

## Module map & ownership (one owner per file)
```
src/
  main.rs           INTEGRATOR ONLY — wiring
  lib.rs            INTEGRATOR ONLY — `pub mod` declarations + re-exports
  contracts.rs      INTEGRATOR ONLY — shared constant defs (palette hexes, ids)
  domain/
    mod.rs          wave3 agent D1
    entry.rs        wave3 agent D1 — ClipEntry, ContentKind, FileMetadata, PreviewData
    hash.rs         wave3 agent D2 — content_hash(...), dedup helpers
    filter.rs       wave3 agent D2 — SearchFilter, DateRange, KindFilter
  config/
    mod.rs          wave3 agent D3
    settings.rs     wave3 agent D3 — Settings (serde), ThemeMode, paths()
  db/
    mod.rs          wave2 agent B1
    schema.rs       wave2 agent B1 — MIGRATIONS, migrate()
    store.rs        wave2 agent B1 — Store::open(), connection, WAL
    entries.rs      wave2 agent B2 — CRUD, upsert_dedup, prune
    search.rs       wave2 agent B3 — FTS5 query + filters
    image_assets.rs wave2 agent B3 — asset dir, save/load PNG
  platform/
    mod.rs          wave2 agent B4 — traits + cfg gating
    winutil.rs      wave2 agent B4 — message-only HWND, RAII guards
    clipboard.rs    wave2 agent B5 — ClipboardAccess impl (read/write text/image/files)
    clipboard_listener.rs wave2 agent B5 — AddClipboardFormatListener thread
    hotkeys.rs      wave3 agent D4 — RegisterHotKey thread, HotkeySpec
    tray.rs         wave3 agent D5 — Shell_NotifyIconW + menu
    startup.rs      wave3 agent D5 — HKCU Run
    winreg.rs       wave3 agent D5 — startup helpers (windows-registry)
    stub.rs         wave2 agent B4 — non-windows stubs
  workers/
    mod.rs          wave2 agent B6
    bus.rs          wave2 agent B6 — WorkerEvent, DbCommand, DbResult, Reply
    db_worker.rs    wave2 agent B7 — owns Connection, command loop
    supervisor.rs    wave2 agent B7 — spawn/join threads, shutdown
  gui/
    mod.rs          wave1 agent A1
    theme.rs        wave1 agent A2 — Palette, light/dark Theme, styles
    widgets.rs      wave1 agent A3 — card(), badge(), empty_state(), styled_button()
    views/
      mod.rs        wave1 agent A1
      sidebar.rs    wave1 agent A4 — sidebar(state) -> Element<Message>
      history.rs    wave1 agent A5 — history_list(state) -> Element<Message>
      search_bar.rs wave1 agent A5 — search_bar(state) -> Element<Message>
      preview.rs    wave3 agent D6 — preview_panel(state) -> Element<Message>
      settings.rs   wave3 agent D7 — settings_view(state) -> Element<Message>
      dialogs.rs    wave3 agent D7 — confirm_clear(state) -> Element<Message>
  app/
    mod.rs          INTEGRATOR (owns AppState)
    message.rs      wave1 agent A6 — Message enum (FROZEN, see below)
    state.rs        INTEGRATOR
    update.rs       wave1 agent A7 — update(&mut AppState, Message) -> Task<Message>
    view.rs         wave1 agent A8 — view(&AppState) -> Element<Message>
    subscription.rs wave1 agent A9 — subscription(&AppState) -> Subscription<Message>
```

## Frozen Message enum (agent A6 writes it EXACTLY; nobody else edits)
```rust
#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    // worker -> gui
    Worker(WorkerEvent),
    // search / filters
    SearchChanged(String),
    FilterKind(Option<ContentKind>),
    FilterDate(DateRange),
    TogglePinnedOnly(bool), ToggleFavoritesOnly(bool),
    ClearFilters,
    // entry actions
    SelectEntry(i64),
    PinEntry(i64, bool), FavoriteEntry(i64, bool),
    DeleteEntry(i64), RequestClearAll, ConfirmClearAll, CancelClearAll,
    RestoreEntry(i64),                 // copy back to OS clipboard
    OpenPreviewInExplorer(i64),
    // paging
    LoadMore, NextPage, PrevPage,
    // settings
    OpenSettings, CloseSettings,
    SetTheme(ThemeMode),
    SetMaxEntries(usize), SetMaxAgeDays(u32),
    SetPauseMonitoring(bool), SetClearOnExit(bool),
    SetStartMinimized(bool), SetLaunchAtStartup(bool),
    RebindShortcut(ShortcutAction, String),
    // dialogs / misc
    ToggleAbout, Quit,
    // tray / hotkey events arriving via WorkerEvent
    RetentionTick,
    ShowWindow, HideWindow,
}
```

## Frozen domain types (agents implement these EXACT names/signatures)
```rust
// domain/entry.rs
#[derive(Debug,Clone,Copy,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
pub enum ContentKind { Text, Image, Files }

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct FileMetadata { pub path: String, pub size_bytes: u64,
    pub is_dir: bool, pub modified_unix: i64, pub exists: bool }

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct ClipEntry {
    pub id: i64, pub kind: ContentKind, pub content_hash: String,
    pub text_content: Option<String>,
    pub image_path: Option<String>, pub image_w: Option<u32>, pub image_h: Option<u32>,
    pub file_paths_json: Option<String>, pub file_meta_json: Option<String>,
    pub created_at: i64, pub last_used_at: i64, pub use_count: i64,
    pub pinned: bool, pub favorite: bool, pub source_app: Option<String>,
    pub byte_size: i64,
}
impl ClipEntry { pub fn display_text(&self) -> String; pub fn short_kind(&self) -> &'static str; }

// domain/filter.rs
#[derive(Debug,Clone,Default)] pub struct SearchFilter {
    pub query: String, pub kind: Option<ContentKind>, pub date: DateRange,
    pub pinned_only: bool, pub favorites_only: bool, pub page: usize, pub page_size: usize, }
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)] pub enum DateRange { #[default] All, Today, Last7Days, Last30Days }
impl DateRange { pub fn cutoff_unix(&self, now: i64) -> Option<i64>; pub fn label(&self) -> &'static str; }
#[derive(Debug,Clone,Copy)] pub enum ShortcutAction { ShowHide, QuickPaste(u8), }

// domain/hash.rs
pub fn hash_text(s: &str) -> String;              // sha256 hex of UTF-8 bytes
pub fn hash_bytes(b: &[u8]) -> String;             // sha256 hex
pub fn hash_paths(paths: &[String]) -> String;    // sha256 of sorted, lowercased, joined with \n
pub fn dedup_key(kind: ContentKind, hash: &str) -> String; // "text:<hash>" etc.

// config/settings.rs
#[derive(Debug,Clone,Copy,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
pub enum ThemeMode { Light, Dark }
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)] pub struct Settings {
    pub theme: ThemeMode, pub max_entries: usize, pub max_age_days: u32,
    pub pause_monitoring: bool, pub clear_on_exit: bool, pub start_minimized: bool,
    pub launch_at_startup: bool, pub page_size: usize,
    pub show_hotkey: String, pub ignored_apps: Vec<String>, }
impl Settings { pub fn load() -> Self; pub fn save(&self) -> Result<(), ConfigError>;
    pub fn data_dir() -> PathBuf; pub fn db_path() -> PathBuf; pub fn assets_dir() -> PathBuf;
    pub fn settings_path() -> PathBuf; }
impl Default for Settings { ... } // sensible defaults
#[derive(Debug,thiserror::Error)] pub enum ConfigError { ... }
```

## Frozen worker bus (agent B6 writes EXACTLY)
```rust
// workers/bus.rs
#[derive(Debug,Clone)] pub enum WorkerEvent {
    NewClipboard(ClipPayload),        // monitor -> gui -> db
    Db(DbResult), Hotkey(ShortcutAction), Tray(TrayAction),
    Error(String),
}
#[derive(Debug,Clone)] pub enum ClipPayload { Text(String), Image{ png: Vec<u8>, w:u32, h:u32, hash:String },
    Files(Vec<FileMetadata>), }
#[derive(Debug,Clone)] pub enum DbResult {
    Entries(Vec<ClipEntry>), Entry(Option<ClipEntry>), Count(usize),
    Done(String), Error(String), }
#[derive(Debug,Clone)] pub enum DbCommand {
    Insert(ClipPayload), Search(SearchFilter), Get(i64),
    SetPin(i64,bool), SetFavorite(i64,bool), Delete(i64), ClearAll,
    Prune{ max_entries:usize, max_age_days:u32 }, Shutdown, }
#[derive(Debug,Clone,Copy)] pub enum TrayAction { Show, Settings, Pause, Resume, Quit }
```
Channels: `flume::unbounded` for `WorkerEvent` (many producers -> gui),
`flume::bounded(64)` for `DbCommand` (gui -> db worker), replies come back as
`WorkerEvent::Db(...)`. Every command carries no reply handle — DbResult variants
are matched in `update()` by context (document which result you expect in update comments).

## Frozen db API (agents B1/B2/B3)
```rust
// db/store.rs
pub struct Store { conn: rusqlite::Connection }
impl Store { pub fn open(path: &Path) -> Result<Self, DbError>; pub fn conn(&self) -> &rusqlite::Connection; }
// db/schema.rs
pub fn migrate(conn: &mut rusqlite::Connection) -> Result<u32, DbError>; // user_version -> latest
// db/entries.rs
pub fn insert_dedup(conn: &Connection, p: &ClipPayload) -> Result<i64, DbError>;
pub fn set_pin(conn: &Connection, id:i64, v:bool) -> Result<(), DbError>;
pub fn set_favorite(conn: &Connection, id:i64, v:bool) -> Result<(), DbError>;
pub fn delete(conn: &Connection, id:i64) -> Result<(), DbError>;
pub fn clear_all(conn: &Connection) -> Result<(), DbError>;
pub fn get(conn: &Connection, id:i64) -> Result<Option<ClipEntry>, DbError>;
pub fn prune(conn: &Connection, max_entries:usize, max_age_days:u32) -> Result<usize, DbError>;
// db/search.rs
pub fn search(conn: &Connection, f: &SearchFilter) -> Result<Vec<ClipEntry>, DbError>;
pub fn count(conn: &Connection, f: &SearchFilter) -> Result<usize, DbError>;
// db/image_assets.rs
pub fn dir() -> PathBuf;
pub fn save_png(png: &[u8], hash: &str) -> Result<PathBuf, DbError>;
pub fn load_png(rel: &str) -> Result<Vec<u8>, DbError>;
pub fn delete_orphans(conn: &Connection) -> Result<usize, DbError>;
#[derive(Debug,thiserror::Error)] pub enum DbError { #[error("{0}")] Sqlite(#[from] rusqlite::Error),
   #[error("{0}")] Io(#[from] std::io::Error), #[error("{0}")] Other(String) }
```

## Frozen platform API
```rust
// platform/mod.rs — traits (cfg-gated impls)
pub trait ClipboardAccess: Send { fn read(&self) -> Result<Option<ClipPayload>, PlatformError>;
    fn write_text(&self, s:&str) -> Result<(), PlatformError>;
    fn write_image(&self, png:&[u8]) -> Result<(), PlatformError>;
    fn write_files(&self, paths:&[String]) -> Result<(), PlatformError>; }
pub fn clipboard() -> Box<dyn ClipboardAccess>;   // windows real, else stub
pub fn start_listener(tx: flume::Sender<WorkerEvent>, pause: Arc<AtomicBool>) -> Result<ListenerHandle, PlatformError>;
pub fn start_hotkeys(tx: flume::Sender<WorkerEvent>, spec: HotkeySpec) -> Result<HotkeyHandle, PlatformError>;
pub fn start_tray(tx: flume::Sender<WorkerEvent>) -> Result<TrayHandle, PlatformError>;
pub fn set_launch_at_startup(enable: bool) -> Result<(), PlatformError>;
pub fn launch_at_startup() -> bool;
pub struct ListenerHandle { /* join on Drop */ pub fn shutdown(&self); }
pub struct HotkeyHandle { pub fn shutdown(&self); }
pub struct TrayHandle { pub fn shutdown(&self); pub fn set_pause(&self, paused:bool); }
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)] pub struct HotkeySpec { pub modifiers: u32, pub vk: u32 }
#[derive(Debug,thiserror::Error)] pub enum PlatformError { ... } // windows::core::Error, io, string
```
`start_listener` uses `AddClipboardFormatListener` on a message-only HWND in its own
thread (event-driven, no polling); on `WM_CLIPBOARDUPDATE` it signals the clipboard
reader with bounded `OpenClipboard` retries (5 tries, 15ms backoff). It MUST filter
self-induced updates using `GetClipboardSequenceNumber()`. All `windows-rs` calls
live ONLY in `platform/`. Non-windows stub returns `Ok(None)` and no-op handles.

## Frozen gui API (agents A1..A9, D6, D7)
```rust
// gui/theme.rs
pub const SAGE: Color; pub const PEACH: Color; pub const PINK: Color;
pub const ROSE: Color; pub const MAUVE: Color;
pub fn light_theme() -> Theme; pub fn dark_theme() -> Theme; // complementary dark, NOT inverted
pub fn theme_for(mode: ThemeMode) -> Theme;
pub fn card_style(theme: &Theme) -> container::Style;
pub fn panel_style(theme: &Theme) -> container::Style;
pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style;
pub fn subtle_button_style(theme: &Theme, status: button::Status) -> button::Style;
pub fn badge_style(kind: ContentKind, theme: &Theme) -> container::Style;
// gui/widgets.rs
pub fn card<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M>;
pub fn badge<'a, M: 'a>(label: &str, kind: ContentKind) -> Element<'a, M>;
pub fn empty_state<'a, M: 'a>(title: &str, subtitle: &str) -> Element<'a, M>;
pub fn icon_button<'a, M: Clone + 'a>(label: &str, msg: M) -> Element<'a, M>;
pub fn section_title<'a, M: 'a>(s: &str) -> Element<'a, M>;
// gui/views/*.rs — each returns Element<'_, Message>
pub fn sidebar(state: &AppState) -> Element<'_, Message>;
pub fn history_list(state: &AppState) -> Element<'_, Message>;
pub fn search_bar(state: &AppState) -> Element<'_, Message>;
pub fn preview_panel(state: &AppState) -> Element<'_, Message>;
pub fn settings_view(state: &AppState) -> Element<'_, Message>;
pub fn confirm_clear(state: &AppState) -> Element<'_, Message>;
```

## Frozen AppState (INTEGRATOR owns; agents may READ these fields)
```rust
pub struct AppState {
    pub settings: Settings,
    pub filter: SearchFilter,
    pub entries: Vec<ClipEntry>,
    pub total_count: usize,
    pub selected: Option<i64>,
    pub preview_bytes: Option<Vec<u8>>,
    pub show_settings: bool,
    pub show_clear_confirm: bool,
    pub show_about: bool,
    pub paused: bool,
    pub status: String,
    pub events_tx: flume::Sender<WorkerEvent>,
    pub events_rx: flume::Receiver<WorkerEvent>,   // subscription reads this
    pub db_tx: flume::Sender<DbCommand>,
    pub paused_flag: Arc<AtomicBool>,
    pub handles: WorkerHandles,   // listener/hotkey/tray handles for shutdown
}
```

## Hard rules for ALL agents
1. NO `unwrap()` / `expect()` in runtime paths (tests may use them). Return `Result`/`Option`.
2. NO blocking calls in update/view. All DB + clipboard + tray work on worker threads.
3. NO network / telemetry crates. No `reqwest`, no `tokio` (flume + std threads + iced runtime only).
4. Only the 5 brand hexes as accents; neutral white/charcoal/grayscale allowed for contrast.
5. Every public fn has a short doc comment; comment only the non-obvious (Windows API specifics).
6. `#[cfg(windows)]` gate all `windows`/`windows-registry` usage. Provide a `stub` for other targets
   so `cargo test` works headless. Use `#[cfg(windows)]` in `platform/mod.rs` to select impls.
7. Files use 4-space indent, snake_case fns, PascalCase types. `cargo fmt` style.
8. Do NOT create a single giant file. Each module owns its own file.
9. When you hit a Windows API limitation, EXPLAIN it in a doc comment and propose the
   practical solution in your final message — never silently stub core functionality.
10. `main.rs`/`lib.rs`/`app/state.rs`/`app/mod.rs` are INTEGRATOR-ONLY. Do not write them.
11. Compile-check YOUR file's syntax mentally; the integrator runs `cargo check` per wave.
12. Return in your final message: files written, the exact contract signatures you implemented,
    any deviation from the contract and why, and any Windows limitation + proposed resolution.

## Palette reference (v0.1.3, use `src/contracts.rs` constants, never raw hex elsewhere)
CREAM `#F2E9E4` (242,233,228) · INK `#22223B` (34,34,59) — the ONLY two
chromatic colors. Neutral `WHITE`/`CHARCOAL` for text contrast; `alpha()` of the
two brand colors for hairlines/shadows. Light bg CREAM, panels/buttons/active INK,
text INK. Dark: INK bg, CREAM accents, CREAM text — complementary, not inverted.
