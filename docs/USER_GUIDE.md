# ClipVault user guide

## The idea

ClipVault quietly keeps a local history of everything you copy — text, images, and
file lists — so you can find it again, pin the things you reuse, and paste them back
with one keystroke. Nothing leaves your machine.

## Main window

```
+------------------+-----------------------------------------------+
| ClipVault        |  [ Search history... ] [kind] [date] [Clear]  |
| clipboard history|  +----------------------------+---------------+ |
|                  |  | history cards              | preview       | |
| [All clips]      |  |  TXT  "some text"          |               | |
| [Pinned]         |  |  Pin Fav Copy Delete       |  text / image | |
| [Favorites]      |  |  used 3 times              |  / file list  | |
|                  |  |  ...                       |               | |
|                  |  +----------------------------+---------------+ |
| [Settings]       |  status  12 entries   [Clear all] [Prev][Next]|
| [About]  [Quit]  |                                               |
+------------------+-----------------------------------------------+
```

- **All clips / Pinned / Favorites** — quick filters in the sidebar.
- **Search bar** — free-text (FTS5), content type (All / Text / Image / Files),
  and date (All time / Today / Last 7 days / Last 30 days). **Clear** resets all.
- **History cards** — one per entry, newest first (pinned and favorites float to
  the top). Each shows a type badge, a short preview, and a use count.
- **Preview** — click a card to see the full text, the photo (with dimensions), or
  the file list with sizes. Images also show a thumbnail right in the history list.
  A **SOURCE** section shows which app copied it and the web page URL when known.
- **Status bar** — last action/result, total entry count, clear-all, and paging.

## Entry actions

| Button | What it does |
|---|---|
| **Pin / Unpin** | Keeps the entry at the top and exempts it from retention pruning. |
| **Fav / Unfav** | Marks a favorite (also exempt from pruning). Active state is shown by an Ink fill and a thicker border. |
| **Copy** | Restores that entry to the Windows clipboard so you can paste it anywhere. |
| **Delete** | Removes just that entry. |
| **Clear all** | Removes all history except pinned and favorite entries. Asks for confirmation. |

Copies are de-duplicated: copying the same text/image/file list twice does not add
a second row — it bumps `use_count` and refreshes `last_used_at`.

## Organization tools (v0.1.4)

- **Tags** — select an entry, then type comma-separated tags in the preview panel's
  TAGS field and click **Apply**. Tags appear as `#tag` chips on the row. Use the
  tag picker in the search bar to show only entries with a given tag; "All tags"
  clears it. Tags are trimmed, deduplicated, and stored locally.
- **Bulk select** — tick the checkbox on any row. A toolbar appears with the count
  and: **Select all**, **Pin**, **Favorite**, **Delete** (asks for confirmation),
  and **Clear**. Bulk operations apply to every checked row at once.
- **Export / import** — in Settings under "APPEARANCE & DATA", use **Export history**
  to write a portable JSON backup (with images embedded) to
  `%LOCALAPPDATA%\ClipVault\clipvault-export.json`, and **Import history** to load
  it back. Import is dedup-safe, so importing the same file twice will not create
  duplicates. There is no cloud step; the file never leaves your machine unless you
  copy it yourself. Very large image histories are capped at 50 MB per export.
- **Per-type retention** — Settings lets you set separate maximum counts for text,
  images, and files. `0` means "use the global max". Pinned and favorite entries
  are never pruned.
- **List density** — switch Comfortable / Compact from Settings or the status-bar
  button. Compact shows more rows per screen.

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+V` (default, configurable) | Show / hide ClipVault |
| `Ctrl+Shift+1` ... `Ctrl+Shift+9` | Quick-paste the 1st..9th entry in the current list |

The show/hide combo is editable in Settings (format like `Ctrl+Shift+V`). Only one
application can own a global combo — if another app already has it, ClipVault shows
a hint in the status bar asking you to rebind.

## System tray

The two-color (Cream/Ink) brand tray icon keeps ClipVault running in the background.

- **Double-click** the icon — show the window.
- **Right-click** (or single click) — menu: **Show ClipVault**, **Settings**,
  **Pause monitoring** / **Resume monitoring**, **Quit**.
- **Quit** removes the tray icon cleanly and checkpoints the database.

## Settings

Open with the sidebar **Settings** button or the tray menu.

| Section | Options |
|---|---|
| **Appearance** | Light (Cream / Rose) or Dark (Ink / Slate) theme. |
| **Storage & retention** | Max entries, max age in days. Pruning removes the oldest non-pinned, non-favorite entries. |
| **Privacy** | Pause clipboard monitoring; clear history on exit. |
| **Startup & shortcuts** | Launch at Windows startup, start minimized to tray, global shortcut. |

Settings are saved to `settings.json` immediately (atomic write). Clipboard history
is never uploaded, and there is no telemetry or update checker.

## Content types

- **Text** — stored directly in SQLite as UTF-8. Unicode is fully supported.
- **Images** — the clipboard's device-independent bitmap (DIB) is normalized to a
  PNG stored in the asset directory; the database keeps only its path, hash, and
  dimensions. The history list shows a thumbnail and the preview renders the full
  photo with its pixel dimensions.
- **Files** — only paths and metadata (size, directory flag, modified time,
  existence) are stored. ClipVault never copies file contents into the database.
  If a file is later moved, the preview shows `(missing)` rather than failing.

## Copy source (provenance)

ClipVault records, on a best-effort basis, where each copy came from:
- **Source app** — the process that owned the clipboard (`chrome.exe`, `notepad.exe`,
  ...), resolved from the clipboard owner window. Only the executable name is stored.
- **Source URL** — browsers publish a `CF_HTML` header with `SourceURL:<page>`;
  ClipVault parses it. Otherwise a bare `http(s)://` link in the copied text is used.
Both appear in the history subtitle and the preview's SOURCE section. They are
optional — not every app publishes a URL, and the owner window can close before
the read, so entries without them are normal. Everything is local; no lookup is
performed against the network.

## Privacy notes

- Everything is local: `%LOCALAPPDATA%\ClipVault`.
- **Pause monitoring** stops all capture without closing the app.
- **Clear history on exit** wipes history each time you quit.
- Startup registration, if enabled, writes only the `HKCU\...\Run\ClipVault` value.

## Tips

- Pin your most-used snippets (emails, addresses, commands) so retention never
  prunes them.
- Use `Ctrl+Shift+1..9` for a fast paste without switching windows.
- Narrow by **Files** + **Today** to find a path you copied this morning.
- The diagnostics log lives at `%LOCALAPPDATA%\ClipVault\clipvault.log`.
