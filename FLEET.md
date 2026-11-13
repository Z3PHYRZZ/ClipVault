# ClipVault — Fleet Interface Freeze (wave 1 new modules)

> Read `CONTRACT.md` first for shared types, palette, and hard rules.
> This file freezes the EXACT signatures of NEW modules introduced by the fleet.
> Code against these signatures even if the sibling file is not written yet —
> the integrator declares `pub mod` and compiles after each wave.
> Every agent owns EXACTLY ONE file (or the one file it is told to replace).
> Never edit `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `src/platform/mod.rs`,
> `src/app/state.rs`, `CONTRACT.md`, or `FLEET.md`. Those belong to the integrator.

## Windows-rs 0.62 facts (compile-proven by the integrator)
- Clipboard fns: `windows::Win32::System::DataExchange::{OpenClipboard, CloseClipboard,
  EmptyClipboard, GetClipboardData, SetClipboardData, AddClipboardFormatListener,
  RemoveClipboardFormatListener, GetClipboardSequenceNumber}`.
  `OpenClipboard(Option<HWND>) -> Result<()>`, `GetClipboardData(u32) -> Result<HANDLE>`,
  `SetClipboardData(u32, Option<HANDLE>) -> Result<HANDLE>`.
- Memory: `Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GlobalSize,
  GLOBAL_ALLOC_FLAGS}` (GlobalAlloc -> Result<HGLOBAL>); `GlobalFree(Option<HGLOBAL>)`
  lives in `Win32::Foundation`. `HANDLE` and `HGLOBAL` are distinct newtypes; convert
  with `HANDLE(h.0)` / `HGLOBAL(h.0)`.
- `GetMessageW` returns `BOOL` (compare `r.0 == 0 || r.0 == -1`).
- `RegisterHotKey`/`UnregisterHotKey` in `Win32::UI::Input::KeyboardAndMouse`, take
  `(Option<HWND>, i32, HOT_KEY_MODIFIERS, u32)`. `HOT_KEY_MODIFIERS(pub u32)`.
- `WNDPROC = Option<unsafe extern "system" fn(HWND,u32,WPARAM,LPARAM)->LRESULT>`.
  `DefWindowProcW` needs an `unsafe extern "system"` wrapper.
- `HWND_MESSAGE`, `HWND(pub *mut c_void)`, `HINSTANCE(pub *mut c_void)`,
  `HMODULE(pub *mut c_void)` are in `Win32::Foundation`. Convert `HMODULE`→`HINSTANCE`
  via `HINSTANCE(m.0)`.
- Registry: `windows_registry::CURRENT_USER.create(r"Software\...\Run")`,
  `.set_string(name,val)`, `.remove_value(name)`, `.get_string(name)`.
- `CreateIconIndirect`, `CreateBitmap`, `DeleteObject` in `Win32::Graphics::Gdi`;
  `LoadIconW` in `Win32::UI::WindowsAndMessaging`.
- Drag/drop: `DragQueryFileW(HDROP, u32, Option<&mut [u16]>) -> u32`,
  `DROPFILES` struct in `Win32::UI::Shell` (Common).
- All `windows`/`windows-registry` usage MUST be `#[cfg(windows)]`-gated with a
  `#[cfg(not(windows))]` headless stub so `cargo test` works.

## Frozen signatures — wave 1 new modules

### `src/platform/format.rs` (agent W1-6, NEW)
```rust
pub const CF_UNICODETEXT: u32 = 13;
pub const CF_HDROP: u32 = 15;
pub const CF_DIB: u32 = 8;
pub const CF_DIBV5: u32 = 17;
pub const CF_BITMAP: u32 = 2;
/// Formats to try when reading, in priority order (text > files > image).
pub fn read_priority() -> &'static [u32];
/// Cheap discriminator used to reject self-induced clipboard updates.
#[cfg(windows)] pub fn sequence_number() -> u32;
#[cfg(not(windows))] pub fn sequence_number() -> u32;
```

### `src/platform/dib.rs` (agent W1-1, NEW)
Pure-Rust DIB/DIBV5 parse → PNG via the `image` crate (0.25.9). No windows-rs
needed inside parsing (keeps it headless-testable); only CF constants come from format.rs.
```rust
/// Parse a CF_DIB/CF_DIBV5 byte blob (BITMAPINFOHEADER + optional masks + pixels)
/// into normalized PNG bytes + (width, height).
pub fn dib_to_png(dib: &[u8]) -> Result<(Vec<u8>, u32, u32), PlatformError>;
/// Convert PNG bytes back into a CF_DIB blob for SetClipboardData.
pub fn png_to_dib(png: &[u8]) -> Result<Vec<u8>, PlatformError>;
/// Read (width, height) from a DIB header without decoding pixels.
pub fn dib_dimensions(dib: &[u8]) -> Option<(u32, u32)>;
```
Handle: BITMAPINFOHEADER (40 bytes) and BITMAPV5HEADER (124 bytes); biHeight may be
negative (top-down); 1/4/8bpp palettes; 24/32bpp BGRA→RGBA; BI_BITFIELDS masks;
row stride padded to 4 bytes. Include `#[cfg(test)]` unit tests with synthetic DIBs.

### `src/platform/hdrop.rs` (agent W1-2, NEW)
```rust
/// Build a CF_HDROP memory blob: DROPFILES header (fWide=true) + UTF-16 paths
/// separated by NUL and terminated by a double NUL. Return raw bytes.
pub fn build_dropfiles_wide(paths: &[String]) -> Vec<u8>;
/// Parse a DROPFILES blob back into paths (wide and narrow variants).
pub fn parse_dropfiles(buf: &[u8]) -> Vec<String>;
/// Stat each path into FileMetadata (size, is_dir, modified, exists).
/// On non-windows, existence/size only.
pub fn query_metadata(paths: &[String]) -> Vec<FileMetadata>;
```
`DROPFILES` layout: `pFiles: u32 (=20)`, `pt: POINT(i32,i32)`, `fNC: i32`, `fWide: i32`.
Include `#[cfg(test)]` unit tests for round-trip with an empty list, one path, many paths.

### `src/platform/icon.rs` (agent W1-3, NEW)
```rust
/// 32x32 RGBA brand icon (Sage bg, Dusty Rose rounded "V"/folder, Muted Mauve outline).
pub fn rgba_icon() -> Vec<u8>;
/// A valid .ico file (ICONDIR + ICONDIRENTRY + PNG payload) as bytes.
pub fn ico_bytes() -> Vec<u8>;
pub fn write_ico(path: &std::path::Path) -> Result<(), PlatformError>;
/// Build an HICON from rgba_icon() via CreateIconIndirect (windows only).
#[cfg(windows)] pub fn make_hicon() -> Result<isize, PlatformError>;
#[cfg(not(windows))] pub fn make_hicon() -> Result<isize, PlatformError>;
```

### `src/platform/singleton.rs` (agent W1-4, NEW)
```rust
/// Named-mutex single instance. Returns None if another instance already holds it.
pub struct SingletonGuard;
pub fn try_acquire() -> Result<Option<SingletonGuard>, PlatformError>;
/// Ask an already-running instance to show its window (registered window message).
pub fn signal_existing() -> Result<(), PlatformError>;
```
`CreateMutexW(None, true, w!("ClipVaultSingleton"))`; `ERROR_ALREADY_EXISTS`.
Guard closes the handle on Drop. `cfg`-gated stub returns `Some(guard)` on non-windows.

### `src/platform/handles.rs` (agent W1-5, NEW)
```rust
/// Owns a Win32 worker thread: posts WM_QUIT to its message queue and joins.
pub struct ThreadHandle {
    pub thread_id: u32,
    join: Option<std::thread::JoinHandle<()>>,
}
impl ThreadHandle {
    pub fn new(thread_id: u32, join: std::thread::JoinHandle<()>) -> Self;
    /// PostThreadMessageW(thread_id, WM_QUIT, 0, 0), then join. Idempotent.
    pub fn shutdown(&mut self);
    pub fn from_current_thread(join: std::thread::JoinHandle<()>) -> Self;
}
```
`#[cfg(windows)]` uses `PostThreadMessageW`; non-windows just joins.

### `src/platform/clipboard_ext.rs` (agent W1-7, NEW)
Full clipboard backend implementing `ClipboardAccess` (trait in `platform/mod.rs`).
Depends on `format.rs`, `dib.rs`, `hdrop.rs` with EXACT frozen signatures above.
```rust
pub struct FullClipboard;
impl ClipboardAccess for FullClipboard { /* read/write_text/write_image/write_files */ }
pub fn create() -> Box<dyn ClipboardAccess>;
```
`read()` tries formats in `format::read_priority()` order, using a `ClipboardGuard`
from `winutil::win32` (bounded retries). Text→`ClipPayload::Text`, DIB→`dib_to_png`→
`ClipPayload::Image{png,w,h,hash}` (hash = `crate::domain::hash_bytes(&dib)`), HDROP→
`hdrop::query_metadata`→`ClipPayload::Files`. Writes mirror this: text via CF_UNICODETEXT
HGLOBAL; image via `dib::png_to_dib`; files via `hdrop::build_dropfiles_wide`.
All `SetClipboardData` calls transfer HGLOBAL ownership to the system — never GlobalFree
after success. `cfg`-gated stubs for non-windows.

### `src/platform/tray_full.rs` (agent W1-8, NEW)
```rust
pub fn start(tx: flume::Sender<WorkerEvent>) -> Result<TrayHandle, PlatformError>;
```
Full `Shell_NotifyIconW(NIM_ADD/NIM_DELETE)` + `NOTIFYICONDATAW` with `uCallbackMessage =
WM_TRAY_CALLBACK (0x8001)`, `CreatePopupMenu` + `AppendMenuW` + `TrackPopupMenu` on
right-click, `GetCursorPos`. Menu → `TrayAction::{Show,Settings,Pause,Resume,Quit}` sent
via `WorkerEvent::Tray`. Icon from `icon::make_hicon()`. Dedicated thread with message
loop; `NIM_DELETE` on shutdown. `TrayHandle` is the type from `platform/mod.rs` (integrator
rewires it to `handles::ThreadHandle` after the wave).

### `src/platform/hotkeys_full.rs` (agent W1-9, NEW)
```rust
pub fn start(tx: flume::Sender<WorkerEvent>, spec: HotkeySpec) -> Result<HotkeyHandle, PlatformError>;
```
Registers the show/hide combo plus Ctrl+1..9 for `ShortcutAction::QuickPaste(1..9)`.
Handles `ERROR_HOTKEY_ALREADY_REGISTERED` by emitting `WorkerEvent::Error` with a
rebind hint (never crashes). `UnregisterHotKey` for every id on shutdown. Emits
`WorkerEvent::Hotkey(action)` on `WM_HOTKEY` (decode `wParam.0 as i32`).

### `src/platform/listener_full.rs` (agent W1-10, NEW)
```rust
pub fn start(tx: flume::Sender<WorkerEvent>, pause: std::sync::Arc<std::sync::atomic::AtomicBool>)
    -> Result<ListenerHandle, PlatformError>;
```
`AddClipboardFormatListener` on a `winutil::win32::create_message_window()`, dedicated
`GetMessageW` loop. On `WM_CLIPBOARDUPDATE`: capture `format::sequence_number()`; if it
equals the last-seen value, skip (self-induced/restored update); otherwise read via
`clipboard_ext::create().read()` and emit `WorkerEvent::NewClipboard`. Respects `pause`.
`RemoveClipboardFormatListener` + `DestroyWindow` on shutdown. Scriptable shutdown via
`handles::ThreadHandle` (integrator rewires).

## Integrator-only wiring (do NOT do in agents)
After wave 1, the integrator adds `pub mod` declarations for the 10 new files,
rewires `platform/mod.rs` handle structs to embed `ThreadHandle`, points
`clipboard()` → `clipboard_ext::create()` (cfg windows), `start_listener` →
`listener_full::start`, `start_hotkeys` → `hotkeys_full::start`, `start_tray` →
`tray_full::start`, and adds the singleton check to `main.rs`.

## Wave 2 (GUI polish, REPLACE exactly one file each)
theme.rs · widgets.rs · views/sidebar.rs · views/history.rs · views/search_bar.rs ·
views/preview.rs · views/settings.rs · views/dialogs.rs · app/view.rs · app/update.rs.
Keep every frozen signature from CONTRACT.md. Only the 5 brand hexes + neutrals.
No unwrap/expect, no blocking calls. Views return `Element<'_, Message>`.
`update.rs` must still handle every `Message` variant (exhaustive match).

## Wave 3 (tests + docs + packaging, NEW files)
tests/db_tests.rs · tests/domain_tests.rs · tests/search_tests.rs ·
tests/settings_tests.rs · tests/platform_tests.rs · tests/dib_tests.rs ·
tests/hdrop_tests.rs · README.md · PACKAGING.md ·
rust-toolchain.toml + .cargo/config.toml + clippy.toml + rustfmt.toml.
Tests use `clipvault::...` public API. Clipboard-touching tests must be
`#[cfg(windows)]` + `#[ignore]`. Build.rs winres icon embed belongs to the integrator.

## Wave 4 (review + quality, 10 agents, mostly read-only reports + targeted fixes)
db correctness · platform correctness · gui correctness · clippy -D warnings fixes ·
cargo fmt · perf/clone audit · unwrap/expect audit · palette compliance ·
Windows API limitation audit · graceful shutdown / WAL consistency.
Report concrete findings with file:line; apply only surgical fixes.
