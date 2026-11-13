# ClipVault v0.1.3 UI rework — frozen swarm spec (integrator-owned)

READ: `src/gui/spacing.rs`, `src/gui/theme.rs`, `src/gui/widgets.rs`, `CONTRACT.md`.
Palette is TWO colors only: CREAM `#F2E9E4` + INK `#22223B`, neutral white/charcoal
for text contrast. No other hues. ASCII-only user-visible strings (Fira Sans has
no star/dingbat glyphs). `cargo fmt` style, no unwrap/expect in runtime, no blocking
GUI calls. Each agent owns ONLY its listed files. Integrator compiles after each wave.

## Goals (Manuel's ask)
"rework the UI for easier to use and see everything — font, button and hover
animation too. revise everything." Concretely:
1. **Bigger, clearer type**: bump the scale so text is readable at a glance.
2. **Buttons that read as buttons**: stronger hierarchy, larger hit areas, clear
   active/hover/pressed states, consistent padding.
3. **Real hover animation**: not just built-in button status — message-driven
   hover state on history cards / nav items via `iced::widget::mouse_area`
   `.on_enter(Message)` / `.on_exit(Message)`, tracked in `AppState`, reflected in
   card style (lift shadow + border + slight background shift). Smooth-feeling
   because styles recompute on state change.
4. **See everything**: roomier rows, no clipped text, scrollable settings,
   prominent preview, legible status/provenance.

## Frozen new design tokens (WAVE 1 agent owns `src/gui/spacing.rs` ONLY)
Keep ALL existing public names (every view uses `sp::*`). Raise values:
- `TEXT_DISPLAY: u16 = 28`, `TEXT_TITLE: u16 = 21`, `TEXT_BODY: u16 = 16`,
  `TEXT_LABEL: u16 = 14`, `TEXT_CAPTION: u16 = 12`.
- `CARD_PAD: [u16;2] = [MD, LG]` ([12,16]) — roomier cards.
- `BUTTON_PAD: [u16;2] = [MD, LG]` ([12,16]) — bigger hit areas.
- `NAV_PAD: [u16;2] = [MD, LG]` (keep), `ROW_PAD: [u16;2] = [MD, LG]` (keep).
- `INPUT_PAD: u16 = LG` (16), `PANEL_PAD: [u16;2] = [LG, XL]` ([16,24]).
- `CONTENT_PAD: u16 = XXL` (32), `EMPTY_PAD: u16 = XXL` (32).
- `STATUS_PAD: [u16;2] = [MD, XL]` ([12,24]).
- `RADIUS_CARD: f32 = 16.0`, `RADIUS_BUTTON: f32 = 12.0`,
  `RADIUS_BADGE: f32 = 9.0`, `RADIUS_INPUT: f32 = 12.0`.
- `GAP_TIGHT: u16 = 8` (keep), `GAP_SECTION: u16 = 20`, `GAP_TITLE: u16 = 8`.
- ADD (public, additive — safe): `pub const HOVER_LIFT: f32 = 3.0;` and
  `pub const HOVER_BORDER: f32 = 2.0;`.
Update the module doc to describe the new scale. Do NOT remove `UNIT/XS/SM/MD/LG/XL/XXL`.

## Frozen widget/theme contract (WAVE 2 agents own `theme.rs` + `widgets.rs`)
`theme.rs` KEEP all 9 public fn names/signatures:
`light_theme, dark_theme, theme_for, card_style, panel_style, primary_button_style,
subtle_button_style, active_button_style, badge_style`.
ADD these public functions (integrator wires them to hover state):
```rust
/// Card style with an explicit hovered flag (message-driven hover animation).
pub fn card_hover_style(theme: &Theme, hovered: bool) -> container::Style;
/// Nav/button style with an explicit hovered flag.
pub fn nav_hover_style(theme: &Theme, hovered: bool, active: bool) -> button::Style;
```
- `card_hover_style`: when hovered, shadow offset `(0, sp::HOVER_LIFT)` with
  `blur_radius 16`, `alpha(INK, 0.22)` border width `sp::HOVER_BORDER`, and a
  subtle background blend toward the contrasting brand color (~8%). Otherwise
  identical to `card_style`.
- `nav_hover_style`: hovered -> blend toward CREAM/INK + stronger shadow; active
  -> INK fill; else CREAM fill. Contrast text via existing `button_text` logic.
- Buttons: use `sp::RADIUS_BUTTON`, `sp::BUTTON_PAD` implicitly via widgets;
  `primary_button_style` gets a **2px INK border** and hover shadow lift;
  `subtle_button_style` gets 1px + hover; `active_button_style` keeps 2px active.
- Hover/press blends may use `blend()` HELPER only (no new hues).
`widgets.rs` KEEP all 7 original public fn names/signatures + `label`/`display`.
ADD (additive):
```rust
/// Button with an explicit hover flag (used with mouse_area hover tracking).
pub fn hover_button<'a, M: Clone + 'a>(label: &str, hovered: bool, active: bool, msg: M) -> Element<'a, M>;
/// A card whose style reacts to a hovered flag.
pub fn hover_card<'a, M: 'a>(content: impl Into<Element<'a, M>>, hovered: bool) -> Element<'a, M>;
```
All button labels use `sp::TEXT_LABEL` (now 14), padding `sp::BUTTON_PAD`.
`icon_button` should use `sp::TEXT_LABEL` too. Keep `Element<'a, M>` (default
Theme/Renderer) everywhere — NO generic Theme/Renderer params.

## Frozen AppState + Message additions (INTEGRATOR owns `app/state.rs`, `app/message.rs`)
Integrator adds:
- `Message::HoverCard(Option<i64>)` (hovered history entry id, None on exit)
  and `Message::HoverNav(Option<u8>)` (0=All,1=Pinned,2=Favorites,3=Settings,
  4=About,5=Quit) — both `#[derive(Debug, Clone)]` enum stays.
- `AppState.hovered_card: Option<i64>`, `AppState.hovered_nav: Option<u8>`.
- `update()` handles both by setting the field, returns `Task::none()`.
- `main.rs` fallback `AppState { .. }` gets the two new fields (integrator).

## WAVE 3 view agents (disjoint files)
- **A** `src/gui/views/history.rs`: wrap each row card in
  `mouse_area(...).on_enter(Message::HoverCard(Some(id))).on_exit(Message::HoverCard(None))`,
  use `widgets::hover_card(..., state.hovered_card == Some(id))`. Keep the
  thumbnail/badge, provenance subtitle, all 4 buttons + `SelectEntry` unchanged.
  Bump nothing else; tokens drive sizes.
- **B** `src/gui/views/sidebar.rs`: use `widgets::hover_button` + `mouse_area`
  `.on_enter(Message::HoverNav(Some(n)))`/`.on_exit(Message::HoverNav(None))`
  for nav items & bottom actions; brand block uses `widgets::display`/`label`.
  Keep `sidebar(state) -> Element<'_, Message>`.
- **C** `src/gui/views/settings.rs` + `src/gui/views/dialogs.rs`: roomier sections
  (tokens), `section_title` + `label` hints, scrollable; dialogs use `widgets::card`
  with larger title. Keep both public fn signatures.
- **D** `src/gui/views/search_bar.rs` + `src/gui/views/preview.rs`: bigger inputs
  (`sp::INPUT_PAD`), preview SOURCE section readable, `content_fit` keep, all
  text via tokens. Keep `search_bar`/`preview_panel` signatures.
- **E** `src/app/view.rs`: root spacing/padding via tokens, status bar readable,
  keep modal `stack!` overlays + `view(state)` signature.

## Hard rules
- ASCII-only strings. Two brand colors only. No magic numbers (use `sp::*`).
- Do NOT run `cargo` (concurrent edits collide; integrator compiles per wave).
- Do NOT edit `Cargo.toml`, `main.rs`, `app/message.rs`, `app/state.rs`,
  `app/update.rs`, `spacing.rs` (unless that's your file), `contracts.rs`,
  `CONTRACT.md`, or another agent's files.
- Report: files written, signatures kept/added, deviations, any Iced uncertainty.
