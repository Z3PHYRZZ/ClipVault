# ClipVault — UI Polish Wave B Brief (design tokens)

READ FIRST: `src/gui/spacing.rs` — the frozen spacing/radius/type scale. `CONTRACT.md`
for palette rules. `src/gui/widgets.rs` and `src/gui/theme.rs` are being polished
concurrently by Wave A — do NOT edit them, just call their public functions.

## Hard rules (all agents)
- Own ONLY the files listed in YOUR task. Never edit spacing.rs, theme.rs,
  widgets.rs, gui/mod.rs, app/view.rs (unless assigned), app/state.rs, main.rs,
  Cargo.toml, message.rs, or another agent's files.
- Use `use crate::gui::spacing as sp;` and the tokens (never magic numbers):
  `sp::PANEL_PAD`, `sp::CONTENT_PAD`, `sp::SIDEBAR_PAD`, `sp::NAV_PAD`,
  `sp::ROW_PAD`, `sp::LIST_PAD`, `sp::STATUS_PAD`, `sp::BADGE_PAD`,
  `sp::BUTTON_PAD`, `sp::INPUT_PAD`, `sp::EMPTY_PAD`,
  `sp::GAP_TIGHT` (8), `sp::GAP_SECTION` (16), `sp::GAP_TITLE` (8),
  `sp::TEXT_DISPLAY` (24), `sp::TEXT_TITLE` (18), `sp::TEXT_BODY` (14),
  `sp::TEXT_LABEL` (13), `sp::TEXT_CAPTION` (11).
- `sp::pad([..])` returns `iced::Padding` for `.padding(...)`.
- ONLY the 5 brand hexes + neutral white/charcoal/grayscale as colors.
  No new accent colors. Get colors from `crate::contracts::*` / `gui::theme`.
- All user-visible strings MUST be plain ASCII (Iced's Fira Sans has no
  star/arrow/dingbat glyphs — they render as tofu boxes). No `★ ☆ ✕ ◀ ▶ × · …`.
- Keep every view's public signature: `fn xxx(state: &AppState) -> Element<'_, Message>`.
- No `unwrap()`/`expect()` in runtime paths. No blocking calls.
- `cargo fmt` style, 4-space indent, doc comments only where non-obvious.
- Do NOT run `cargo` — concurrent edits would collide. Integrator compiles after
  the wave. Write clean, compiling Rust against the current API.

## Component targets
- **Sidebar** (`src/gui/views/sidebar.rs`): shell padding `sp::SIDEBAR_PAD` (16),
  width 224.0, column `.spacing(sp::GAP_TIGHT)` (8). Brand block: `display("ClipVault")`
  at `sp::TEXT_DISPLAY` with a caption "clipboard history" at `sp::TEXT_CAPTION` —
  use `widgets::display(...)` and `widgets::label(...)` (Wave A adds these helpers;
  if unsure, just use `text(...).size(sp::TEXT_DISPLAY)` / `sp::TEXT_CAPTION` directly
  to be safe). Brand block padding `[sp::MD, sp::XS]`. Nav items use `sp::NAV_PAD`
  ([12,16]), radius `sp::r(sp::RADIUS_BUTTON)`, text `sp::TEXT_BODY`. Active nav =
  `theme::primary_button_style`, inactive = a *quieter* style: prefer
  `theme::subtle_button_style` but with a transparent/soft background so inactive
  items don't all look like filled buttons. Add `Space::with_height(sp::GAP_SECTION)`
  after the brand block. Bottom actions (Settings/About/Quit) separated by a thin
  `Space::with_height(sp::GAP_TIGHT)` and use `widgets::quiet_button`.
- **Search bar** (`src/gui/views/search_bar.rs`): row `.spacing(sp::GAP_TIGHT)` (8),
  text_input `.padding(sp::INPUT_PAD)` (12), pick_lists `.padding(sp::INPUT_PAD)`,
  `Space` not needed. Keep the `KindChoice` enum + `Display` impl exactly as-is.
  Keep frozen public fn `search_bar(state: &AppState) -> Element<'_, Message>`.
- **History** (`src/gui/views/history.rs`): list `.spacing(sp::GAP_TIGHT)` (8),
  scrollable container padding `sp::LIST_PAD` (8). Row card padding `sp::ROW_PAD`
  ([12,16]), row children `.spacing(sp::GAP_TIGHT)`, text sizes `sp::TEXT_BODY` /
  `sp::TEXT_CAPTION`. Replace the raw `Space::with_width(10)` with `sp::GAP_TIGHT`.
  Selected card: use PINK background but add a 2px ROSE border for a clearer
  selected state (via `Border` on the `container::Style`). Keep `mouse_area`
  selection and all 4 buttons (`state_button` for Pin/Fav, `quiet_button` for
  Copy/Delete) with their exact `Message` variants unchanged. Wrap the text column
  in `.width(Length::Fill)` (keep). Add a subtle top accent: pinned/favorite entries
  get a small `widgets::badge("PIN", kind)` / `badge("FAV", kind)` before the text —
  OPTIONAL, only if it doesn't crowd the row; otherwise skip.
- **Settings** (`src/gui/views/settings.rs`): content `.padding(sp::CONTENT_PAD)`
  (24), `.spacing(sp::GAP_TIGHT)` (8). Title `sp::TEXT_DISPLAY` (24). Section gaps
  `sp::GAP_SECTION` (16) instead of the current 16/20 mix. `widgets::section_title`
  for headers (already does label size). Row `.spacing(sp::GAP_TIGHT)`.
  pick_list/text_input `.padding(sp::INPUT_PAD)`. Add a short caption under each
  section title using `widgets::label(...)` at `sp::TEXT_CAPTION` for a cleaner
  hierarchy (e.g. "Choose light or complementary dark theme."). Keep all
  `Message` variants and the `text_input` on_input closure exactly.
- **Dialogs** (`src/gui/views/dialogs.rs`): `widgets::card` already pads via tokens.
  Use `Space::with_height(sp::GAP_SECTION)` between blocks, `sp::TEXT_TITLE` for
  the confirm title, `sp::TEXT_LABEL` for body, `sp::TEXT_CAPTION` for the About
  sub-lines. Buttons row `.spacing(sp::GAP_TIGHT)`. Keep both public fns
  `confirm_clear(state)` and `about(state)` returning `Element<'_, Message>`.
- **Preview** (`src/gui/views/preview.rs`): container padding `sp::CONTENT_PAD`
  (24) instead of 16; column `.spacing(sp::GAP_TIGHT)` (8); header text
  `sp::TEXT_LABEL`; body text `.size(sp::TEXT_BODY)`; file rows `.spacing(sp::GAP_TIGHT)`
  with `sp::TEXT_LABEL` / `sp::TEXT_CAPTION`; image height 220 keep. Keep
  `row_text` private helper (returns `Element<'static, Message>`). ASCII only.
- **Root view** (`src/app/view.rs`): search bar wrapper `.padding(sp::PANEL_PAD)`
  ([12,16]); main column `.spacing(sp::GAP_TIGHT)` (8); the list/preview row
  `.spacing(sp::GAP_TIGHT)` (8) with proportions 3:2 keep; status bar
  `.padding(sp::STATUS_PAD)` ([8,16]) and `.spacing(sp::GAP_TIGHT)`; status text
  sizes `sp::TEXT_CAPTION`. Sidebar container keeps `theme::panel_style`.
  Keep both modal `stack!` overlays. Keep frozen `view(state) -> Element<'_, Message>`.

## Palette reminder (exact, v0.1.1)
CREAM #F2E9E4 · ROSE #C9ADA7 · MAUVE #9A8C98 · SLATE #4A4E69 · INK #22223B.
Light: CREAM bg, ROSE panels, CREAM cards, ROSE interactive, INK type.
Dark: INK bg, SLATE surfaces, CREAM/ROSE accents, CREAM text.

Return in your final message: files written, token changes applied, signatures
kept, any deviation, any Iced API uncertainty you worked around.
