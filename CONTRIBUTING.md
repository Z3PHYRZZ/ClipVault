# Contributing to ClipVault

## Dev setup (Windows 10/11)

```bash
# Rust stable >= 1.85 (edition 2024). This project pins stable via rust-toolchain.toml.
rustup show
# MSVC build tools + Windows SDK are required (windows-rs links with link.exe).
# On a machine with VS 2022 / VS 18 Community installed, cargo finds the linker.
cargo build
cargo test
```

Git Bash users: `export PATH="/c/Users/Dodot/.cargo/bin:$PATH"` if rustup is not on PATH.

## Code style

- `cargo fmt` (4-space indent, snake_case fns, PascalCase types).
- `cargo clippy --all-targets -- -D warnings` must be clean.
- **No `unwrap()` / `expect()` in runtime paths.** All fallible operations return
  `Result` / `Option`. `anyhow` is only for `main`; `thiserror` for domain errors.
- **No blocking calls in `update()` / `view()`.** DB, clipboard, tray, and hotkey
  work runs on dedicated threads; results come back through `WorkerEvent` on a
  channel `Subscription`.
- All `windows` / `windows-registry` usage is `#[cfg(windows)]`-gated in
  `src/platform/` with headless stubs so `cargo test` runs without a desktop.
- UI strings must be **plain ASCII** — Iced's bundled Fira Sans has no
  star/dingbat/arrow glyphs and renders them as tofu boxes.
- Colors: only the two brand hexes (`#F2E9E4`, `#22223B`) plus neutral
  white/charcoal for text contrast. Alpha variants of the two for hairlines/
  shadows. Spacing, radii, and type sizes come from `src/gui/spacing.rs` —
  never magic numbers.
- Comment only the non-obvious (especially Win32 ownership / threading details).

## Project layout

```
src/
  app/       AppState, Message, update, view, subscription (integrator-owned)
  domain/    ClipEntry, ContentKind, SearchFilter, hashing
  config/    Settings (serde JSON) + app data paths
  db/        migrations, Store, CRUD/dedup, FTS5 search, image assets
  platform/  all Win32 code behind traits (clipboard, listener, hotkeys, tray, ...)
  workers/   typed bus, DB worker command loop, supervisor
  gui/       theme, spacing tokens, widgets, views
tests/       integration tests
installer/   Inno Setup script + build helper
docs/        user + developer docs
```

Read `ARCHITECTURE.md` for the threading model and the Windows API limitations,
and `CONTRACT.md` / `FLEET.md` for the frozen interface signatures.

## Test gates (run before opening a PR)

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
installer/build-installer.bat    # optional: rebuild the Setup.exe
```

Clipboard-touching tests must be `#[cfg(windows)]` + `#[ignore]` so CI never races
the real clipboard. DB tests use `Store::open_in_memory()`; avoid tests that write
to the real `%LOCALAPPDATA%` assets dir.

## Adding a feature

1. Add/extend the domain type in `src/domain/`.
2. Add a `Message` variant (integrator-owned `src/app/message.rs`).
3. Handle it in `update()` — enqueue a `DbCommand` / `WorkerEvent`, never block.
4. Add the view piece under `src/gui/views/`.
5. Add the DB/platform implementation behind the existing traits.
6. Update `ARCHITECTURE.md`, `README.md`, and `CHANGELOG.md`.
7. Add a test and pass all gates.

## Reporting bugs

Include: Windows version, ClipVault version, steps to reproduce, and the contents
of `%LOCALAPPDATA%\ClipVault\clipvault.log` (set `RUST_LOG=clipvault=debug` for
verbose output). Redact any clipboard text you do not want to share — the log
contains only app events, but clipboard content may appear in error messages.

## License

MIT — see [LICENSE](LICENSE).
