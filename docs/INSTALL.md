# Installing ClipVault

## Requirements

- Windows 10 or Windows 11, 64-bit (x64).
- No administrator rights needed — the installer is per-user.
- No .NET, no VC++ redistributable, no internet connection required.

## Install

1. Download **`ClipVault-0.1.6-Setup.exe`** from the
   [GitHub Release](https://github.com/Z3PHYRZZ/ClipVault/releases/tag/v0.1.6).
2. Double-click it. If Windows SmartScreen warns about an unsigned app, click
   **More info > Run anyway** (the binary is built from source in this repo; it is
   not code-signed).
3. Choose the install folder (default `%LOCALAPPDATA%\Programs\ClipVault`).
4. Optional tasks (both unchecked by default):
   - **Create a desktop icon**
   - **Launch ClipVault when Windows starts** (writes the HKCU Run value)
5. On the finishing page, optionally check **Launch ClipVault**.

## Silent / scripted install

```
ClipVault-0.1.6-Setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CURRENTUSER /DIR=<path>
```

Useful flags: `/LOG=<file>` for an install log, `/NOICONS` to skip shortcuts,
`/TASKS="desktopicon,startupicon"` to opt into the optional tasks.

## Run

- Start Menu > **ClipVault**, or the desktop shortcut, or double-click `clipvault.exe`.
- On first launch, ClipVault creates `%LOCALAPPDATA%\ClipVault\` and begins watching
  the clipboard. A tray icon appears (brand-palette "V").
- **No console window appears** — the app is a GUI-subsystem binary.

## Data locations

| What | Where |
|---|---|
| Settings | `%LOCALAPPDATA%\ClipVault\settings.json` |
| Database (WAL) | `%LOCALAPPDATA%\ClipVault\clipvault.db` (+ `-wal`, `-shm`) |
| Image assets | `%LOCALAPPDATA%\ClipVault\assets\<shard>\<hash>.png` |
| Diagnostics log | `%LOCALAPPDATA%\ClipVault\clipvault.log` |
| Startup entry | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\ClipVault` |

All data stays on your machine. Nothing is uploaded.

## Uninstall

- Start Menu > **Uninstall ClipVault**, or Windows Settings > Apps, or run
  `unins000.exe` with `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`.

The uninstaller removes the executable, shortcuts, and registry entry. **It does
not delete your clipboard history** — `%LOCALAPPDATA%\ClipVault` is left intact so
reinstalling keeps your data. To remove history manually, delete that folder while
ClipVault is not running.

## Portable use (no install)

Grab `clipvault.exe` from the Release assets and run it from any folder. Settings
and history still go to `%LOCALAPPDATA%\ClipVault`. Note that if you later move the
exe, the optional startup entry will point at the old path — re-toggle
**Launch at Windows startup** in Settings.

## Troubleshooting

| Symptom | Fix |
|---|---|
| No tray icon | Quit and relaunch; check `clipvault.log` |
| A global hotkey does nothing | Another app owns that combo — rebind in Settings |
| History empty after copy | Settings > make sure "Pause clipboard monitoring" is off |
| Missing file paths in preview | The original files were moved/deleted; ClipVault stores paths only |
| Installer says file in use | Quit the running ClipVault (tray > Quit), then retry |

Diagnostics: open `%LOCALAPPDATA%\ClipVault\clipvault.log`. Set `RUST_LOG=clipvault=debug`
in the environment before launching for verbose logging (no telemetry is collected).
