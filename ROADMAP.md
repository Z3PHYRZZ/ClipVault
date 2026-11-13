# ClipVault — Roadmap (11 phases)

Compile + test after every phase. Status: `pending` | `in_progress` | `done`.
Documentation files (ARCHITECTURE.md, ROADMAP.md, TASKS.md) created before any code — done.

| Phase | Goal | Key deliverables | Gate | Status |
|---|---|---|---|---|
| 1 | Inspect environment & establish structure | `Cargo.toml` (edition 2024), `rust-toolchain.toml`, `build.rs`, module tree, `SUMMARY.md`, docs | `cargo check` on Windows MSVC | **done** |
| 2 | Iced shell & theme | `gui/theme.rs` (exact 5-color light + complementary dark), Message/State/update/view/subscription, sidebar + empty states + rounded cards | `cargo build` + `cargo fmt --check` | **done** |
| 3 | SQLite storage & migrations | `db/` schema v1..v3 (FTS5 + triggers), WAL, CRUD, unique dedup index, FTS5 search | `cargo test` (9 integration) | **done** |
| 4 | Clipboard text monitoring | `winutil` (message-only HWND, bounded `OpenClipboard` retries), `clipboard_ext` (CF_UNICODETEXT), `listener_full` (AddClipboardFormatListener + sequence-number self-filter), worker bus + DB worker | `cargo test` + build | **done** |
| 5 | Image & file support | `dib.rs` (DIB/DIBV5→PNG, PNG→DIB, palettes/bitfields/top-down), `hdrop.rs` (DROPFILES build/parse + metadata), `clipboard_ext` image/file read/write, `image_assets.rs` | `cargo test` (DIB/HDROP synthetic) | **done** |
| 6 | Search, filters, pin/delete/clear/restore | FTS5 + type/date/pin/fav filters, paged history, context actions, clear-all confirm, restore-to-clipboard | `cargo test` search + integration | **done** |
| 7 | Global shortcuts & quick paste | `hotkeys_full.rs` (RegisterHotKey thread + Ctrl+Shift+1..9 quick-paste), HotkeySpec parser, conflict → rebind hint | build + manual smoke | **done** (manual smoke pending) |
| 8 | System tray & background startup | `tray_full.rs` (Shell_NotifyIconW + context menu + NIM_DELETE), `startup.rs` (HKCU Run), `singleton.rs` (named mutex), graceful quit via ThreadHandle | build + manual smoke | **done** (manual smoke pending) |
| 9 | Settings, retention, privacy | `config/settings.rs` serde JSON + atomic write, Settings view, retention prune worker, privacy toggles | `cargo test` | **done** |
| 10 | Performance & memory | event-driven listener (zero idle CPU), paged history, asset-on-disk images, `tracing` file/stderr, graceful shutdown + WAL checkpoint | `cargo clippy -D warnings`, `cargo test` | **done** |
| 11 | Automated tests & Windows packaging | 40 tests, `winres` icon embed, generated `.ico`, README/PACKAGING docs, `cargo build --release` | release build + `cargo test --release` | **done** |

## Cross-cutting invariants (checked every phase)
- No `unwrap()` / `expect()` in runtime paths.
- No blocking calls in `update()` / `view()`.
- No network / telemetry crates.
- Platform code isolated in `src/platform/` behind traits; `#[cfg]` stubs elsewhere so tests run headless.
- Palette: only the 5 brand hexes + neutral white/charcoal/grayscale.
- Every phase updates `ROADMAP.md`, `TASKS.md`, `SUMMARY.md`.

## Environment notes (phase 1 — RESOLVED)
- Host: Windows 11 Pro (10.0.26200), `win32`, shell bash (Git Bash).
- `rustc`/`cargo`/`rustup` were NOT on the initial PATH and no toolchain existed anywhere on disk.
  **Resolution:** bootstrapped via `winget install --id Rustlang.Rustup`. Confirmed:
  `rustc 1.99.1`, `cargo 1.99.1`, host `x86_64-pc-windows-msvc` (edition 2024 OK).
  Binaries at `C:\Users\Dodot\.cargo\bin`; every cargo command prepends
  `export PATH="/c/Users/Dodot/.cargo/bin:$PATH"`.
- MSVC linker: VS 18 Community at `C:\Program Files\Microsoft Visual Studio\18\Community`,
  cl/link at `...\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\`; Windows SDK `10.0.26100.0`.
  `windows-rs` links via this MSVC toolchain (confirmed).
- Iced pinned to **0.13.1** (0.14 needs rustc 1.88; cargo respected our 1.85 floor).
  API compile-proven: `iced::application(...).run_with(...)`, `Theme::custom`,
  `Subscription::run_with_id` + `iced::stream::channel`, `Space::with_height/with_width`,
  const `Color::from_rgb`.
- `windows 0.62.2` API compile-proven: clipboard fns in `Win32::System::DataExchange`,
  `GetClipboardData -> Result<HANDLE>`, `SetClipboardData(u32, Option<HANDLE>)`,
  `GlobalAlloc -> Result<HGLOBAL>`, `GlobalFree(Option<HGLOBAL>)` in Foundation,
  `RegisterHotKey` in `Win32::UI::Input::KeyboardAndMouse` with `HOT_KEY_MODIFIERS`,
  `GetMessageW` returns `BOOL`, `HWND_MESSAGE` const, `WNDPROC` needs `system` ABI.
