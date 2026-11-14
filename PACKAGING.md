# ClipVault — Windows Packaging Guide

Target: Windows 10 / 11, x86_64-pc-windows-msvc. No network at runtime.

## 1. Release build

```bash
export PATH="/c/Users/Dodot/.cargo/bin:$PATH"
cd "D:/job done/ClipVault"
cargo build --release
cargo test --release
cargo clippy --release -- -D warnings
cargo fmt --check
```

`Cargo.toml` release profile: `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`,
`strip = "symbols"`, `panic = "abort"`. Output: `target\release\clipvault.exe`
(a single self-contained PE; `rusqlite` bundled SQLite is statically linked,
`+crt-static` in `.cargo/config.toml` avoids a separate VC++ runtime dependency).

## 2. Application icon / manifest resource

`src/platform/icon.rs` generates the brand `.ico` at runtime
(`icon::write_ico(&path)`), and `icon::make_hicon()` builds the tray `HICON`.
To embed a version resource + icon into the executable itself, add `winres` to
`build.rs` (already a build-dependency):

```rust
// build.rs
fn main() {
    let mut res = winres::WindowsResource::new();
    // Write the generated icon to a temp path first (build scripts can't call
    // the library's runtime code); ship a checked-in assets/icon.ico instead.
    if let Ok(p) = std::env::var("CARGO_MANIFEST_DIR") {
        let ico = std::path::Path::new(&p).join("assets").join("icon.ico");
        if ico.exists() {
            res.set_icon(ico.to_str().unwrap_or(""));
        }
    }
    res.set("ProductName", "ClipVault")
       .set("FileDescription", "ClipVault — private clipboard manager")
       .set("CompanyName", "Manuel")
       .set("LegalCopyright", "MIT")
       .set("FileVersion", "0.1.6.0")
       .set("ProductVersion", "0.1.6.0");
    if let Err(e) = res.compile() {
        // Non-fatal on non-MSVC hosts; the app still builds.
        println!("cargo:warning=winres skipped: {e}");
    }
}
```

Generate the checked-in icon once with a tiny helper (or copy the runtime output):
`clipvault --dump-icon` is not wired; instead use `cargo test` (the
`platform::icon::write_ico_roundtrips` test writes a temp .ico) or a small
`examples/dump_icon.rs` that calls `clipvault::platform::icon::write_ico`.

**Practical limitation:** `winres` needs `rc.exe` from the Windows SDK (present at
`C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\rc.exe`). If it is not
on PATH, the build script warns and skips — the exe still runs and the tray icon
is generated at runtime, so this is graceful, not fatal.

## 3. Distribution options

### A. Standalone installer (RECOMMENDED — double-click to install)
**Delivered artifact:** `installer\Output\ClipVault-0.1.6-Setup.exe` (6.4 MB,
LZMA2-compressed, Inno Setup 6). Double-click it to install.

Rebuild from source:
```bat
installer\build-installer.bat
```
or manually:
```bash
cargo build --release
cargo run --example dump_icon
"C:\Users\Dodot\AppData\Local\Programs\Inno Setup 6\ISCC.exe" installer\ClipVault.iss
```
Inno Setup 6 is already installed (`winget install --id JRSoftware.InnoSetup -e`
if needed on another machine).

Installer behavior (verified end-to-end with a silent install + uninstall):
- **Per-user, no admin elevation** (`PrivilegesRequired=lowest`). Installs to
  `%LOCALAPPDATA%\Programs\ClipVault`.
- Bundles `clipvault.exe`, `README.md`, `LICENSE`, `icon.ico` (brand icon used for
  the setup wizard, Start Menu shortcut, and uninstall entry).
- Start Menu: `ClipVault` + `Uninstall ClipVault`. Optional desktop icon (unchecked).
- Optional "Launch when Windows starts" task (unchecked) writes ONLY the app's own
  `HKCU\...\Run\ClipVault` value — same value the in-app Settings toggle writes.
- Optional "Launch ClipVault" on the finishing page (unchecked).
- Welcome page states plainly: no admin, no network, no telemetry, local-only data,
  and that the uninstaller does NOT delete clipboard history.
- **Uninstall** removes the exe/shortcuts/registry key and leaves
  `%LOCALAPPDATA%\ClipVault` (DB + WAL + assets) intact — user history preserved.
- x64-only (`ArchitecturesAllowed=x64compatible`), Windows 10+ (`MinVersion=10.0`).

Silent install for testing:
```
ClipVault-0.1.6-Setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CURRENTUSER /DIR=<path>
```

### B. Portable zip (zero-install)
```
ClipVault-0.1.6/
  clipvault.exe
  README.md
  LICENSE
```
Extract anywhere, run. Settings/DB go to `%LOCALAPPDATA%\ClipVault`. Startup
registration (Settings → "Launch at Windows startup") writes the HKCU Run value
pointing at the current exe path — moving the exe afterward requires re-toggling
that setting.

### C. WiX MSI (alternative, not shipped)
Use `cargo wix` (WiX Toolset) or a hand-authored `.wxs`:
- Install to `LocalAppDataFolder\ClipVault\` (per-user, no elevation).
- Create Start Menu shortcut + optional desktop shortcut.
- Do NOT write HKCU Run from the installer — let the user opt in via Settings
  (the value must point at the installed path).
- Requires `candle.exe` / `light.exe` (WiX Toolset). Generate with:
  `cargo install cargo-wix && cargo wix --nocapture`.

## 4. What the installer must NOT do
- No network calls, no telemetry, no auto-update checker.
- No machine-wide (HKLM) writes, no service, no driver.
- No registry writes except our own `HKCU\...\Run\ClipVault` when the user opts in.
- No bundled vcredist needed with `+crt-static`.

## 5. Smoke test checklist (post-install)
1. Copy text → appears in history; copy again → `use_count` increments, no dup row.
2. Copy an image (e.g. from Paint) → PNG preview renders, dimensions shown.
3. Copy files in Explorer → file list + sizes shown; missing paths show "(missing)".
4. Search + type/date filters + pin/favorite/delete/clear-all.
5. Restore an entry (`Copy`) → pastes correctly; the listener self-filter does not
   re-insert the restored content as a new row.
6. Global hotkey shows/hides; `Ctrl+Shift+1..9` quick-paste.
7. Tray icon appears; right-click menu (Show/Settings/Pause/Quit) works; Quit
   removes the icon (`NIM_DELETE`) and checkpoints WAL.
8. Launch a second instance → exits (named mutex) without a second tray icon.
9. Settings persist across restart; retention prunes old non-pinned entries.
10. Idle CPU ~0% (listener/hotkey/tray block in `GetMessageW`; DB worker blocks on channel).

## 6. Graceful shutdown contract
`WorkerHandles::shutdown` → listener/hotkey/tray `ThreadHandle::shutdown`
(`PostThreadMessageW(WM_QUIT)` + join) → `DbCommand::Shutdown` → DB worker runs
`wal_checkpoint(TRUNCATE)` then exits → main joins the DB thread. No orphan tray
icons, no WAL left mid-transaction.
