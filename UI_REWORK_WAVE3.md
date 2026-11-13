# ClipVault UI rework — Wave 3 C/D/E agent briefs (dispatch verbatim)

Common prefix for each agent (paste before the task):
"You are a Rust UI-rework subagent on ClipVault at `C:\Users\Dodot\Documents\GitHub\ClipVault`
(bash `cd \"C:/Users/Dodot/Documents/GitHub/ClipVault\"`, prepend
`export PATH=\"/c/Users/Dodot/.cargo/bin:$PATH\"`). READ FIRST: `UI_REWORK_SPEC.md`,
`src/gui/spacing.rs` (NEW bigger tokens), `src/gui/widgets.rs`, `src/gui/theme.rs`,
and your target files. Follow hard rules: ASCII-only strings, two brand colors
(CREAM/INK) only, no magic numbers (use `sp::*`), no unwrap/expect, cargo fmt style.
Do NOT run `cargo` (siblings edited concurrently; integrator compiles after wave).
Do NOT edit any file not listed as yours. Report: files written, signatures kept,
deviations."

## Agent C — settings.rs + dialogs.rs
Own EXACTLY: `src/gui/views/settings.rs`, `src/gui/views/dialogs.rs`.
KEEP signatures `settings_view(&AppState)->Element<'_,Message>`,
`confirm_clear(&AppState)->Element<'_,Message>`, `about(&AppState)->Element<'_,Message>`.
Goal: "see everything" — roomier, readable.
- settings: content `.padding(sp::CONTENT_PAD)` (now 32), `.spacing(sp::GAP_SECTION)`
  (now 20) BETWEEN sections; inside a section `.spacing(sp::GAP_TIGHT)`.
  Title `text("Settings").size(sp::TEXT_DISPLAY)`.
  Each section = `widgets::section_title("APPEARANCE")` + a `widgets::label(...)`
  hint at `sp::TEXT_CAPTION`, then the controls. Row `.spacing(sp::GAP_TIGHT)`.
  pick_list / text_input `.padding(sp::INPUT_PAD)` (now 16).
  Keep ALL Message variants + the `text_input` on_input closure EXACTLY.
  Keep `scrollable(content)`.
- dialogs: `widgets::card` (now roomier), title `sp::TEXT_TITLE`, body `sp::TEXT_LABEL`,
  About sub-lines `sp::TEXT_CAPTION`, `Space::with_height(sp::GAP_SECTION)` between
  blocks, buttons row `.spacing(sp::GAP_TIGHT)`. Keep both stack overlays in view.rs
  untouched (integrator owns it).

## Agent D — search_bar.rs + preview.rs
Own EXACTLY: `src/gui/views/search_bar.rs`, `src/gui/views/preview.rs`.
KEEP `search_bar(&AppState)->Element<'_,Message>`,
`preview_panel(&AppState)->Element<'_,Message>`, and `KindChoice` enum + Display.
- search_bar: row `.spacing(sp::GAP_TIGHT)`, text_input + both pick_lists
  `.padding(sp::INPUT_PAD)` (16). Keep `KindChoice::all/to_kind/from_kind` unchanged.
- preview: container `.padding(sp::CONTENT_PAD)` (32), column `.spacing(sp::GAP_TIGHT)`,
  header `sp::TEXT_LABEL`, body text `sp::TEXT_BODY`, image `.height(300)`
  `.content_fit(iced::ContentFit::Contain)` keep, file rows `sp::GAP_TIGHT` with
  `sp::TEXT_LABEL`/`sp::TEXT_CAPTION`. Keep the SOURCE section (source_app,
  source_url, dimensions) and `row_text` private helper. ASCII only.

## Agent E — app/view.rs
Own EXACTLY: `src/app/view.rs`.
KEEP `view(&AppState)->Element<'_,Message>` signature + both modal `stack!` overlays.
Goal: readable, roomy root.
- search bar wrapper `.padding(sp::pad(sp::PANEL_PAD))` (now [16,24]).
- main column `.spacing(sp::GAP_TIGHT)`, list/preview row `.spacing(sp::GAP_TIGHT)`
  with 3:2 proportions, status bar `.padding(sp::pad(sp::STATUS_PAD))` (now [12,24])
  and `.spacing(sp::GAP_TIGHT)`, status text `.size(sp::TEXT_CAPTION)`.
- Sidebar container keeps `theme::panel_style`. Keep `view(state)` and both stack
  overlays for `show_clear_confirm` / `show_about`.
