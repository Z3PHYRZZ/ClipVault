# Design — Port shadcn/ui visual language into ClipVault (Iced 0.13)

## Intent and why
Manuel asked to redo ClipVault's UI completely following https://ui.shadcn.com/docs/components.
ClipVault is a native Rust + Iced 0.13 desktop app; shadcn/ui is React + Tailwind + Radix and
cannot be installed here. The chosen approach (approved in conversation, "proceed with A") is to
port shadcn's **visual system and component patterns** into native Iced widgets: default zinc
palette (light + dark), 8px base radius, 1px hairline borders, flat surfaces, small shadows,
Tailwind-scale spacing, and the Button/Card/Badge/Input/Select/Checkbox/Separator/Dialog/ScrollArea
patterns. The Rust backend, every Message variant, AppState field, and worker path stays intact.

## Goals
- Full visual replacement of the CREAM/INK frosted-glass theme with shadcn zinc light/dark.
- Reusable style functions in `theme.rs` + widget helpers in `widgets.rs`, no magic values in views.
- Every shadcn component the app genuinely uses is expressed as a native Iced widget.
- All quality gates green: fmt, check, clippy -D warnings, tests, debug + release build.
- Native DWM backdrop kept working; root stays transparent when backdrop is native.

## Non-goals
- No React/Tailwind rewrite, no new heavy dependency, no web stack.
- No port of catalog components with no surface in a clipboard manager (Calendar, Carousel,
  Data Table, Command palette, Toast, Chart, etc.) — dead code would result.
- No change to feature behavior, Message enum, state shape, or worker logic.

## Anti-goals (what makes this a failure even if it compiles)
- Reintroducing the INK-on-INK invisible-text bug (contrast guard must stay).
- Tofu-box Unicode glyphs (all strings stay ASCII; BackdropKind labels untouched).
- Losing native acrylic blur by painting opaque surfaces over the whole transparent window.

## Constraints
- iced 0.13.1 / 0.13.4 API: one `Shadow` per Style; one border width/color (per-corner radius OK);
  `text` has no Style struct; `container` style closure takes only `&Theme`; pick_list popup uses the
  theme menu catalog, not `pick_list::Style`; no native modal focus trap.
- Hover is message-driven (`HoverCard`/`HoverNav` + `mouse_area`), load-bearing — restyle only.
- Windows 11 Pro, MSVC, rustc 1.99.1; no console subsystem change.

## Approaches considered
1. **Port shadcn system into Iced (chosen)** — full visible redesign, backend untouched, verifiable.
2. Rewrite as Tauri + React to run real shadcn — throws away ~5,100 lines of native integration;
   rejected.
3. Port all 60+ catalog components to Iced — most unrendered dead code; rejected.

## Decisions (locked)
- Zinc palette from canonical shadcn v3 theme; hex values in `contracts.rs`; `--radius: 8px`.
- Flat surfaces, no fake blur; DWM backdrop retained, visible in the root gutter.
- Button variants: default / secondary / outline / ghost / destructive; state/toggle + nav reuse them.
- Destructive delete actions get the destructive variant; Pin/Fav are toggles; nav is ghost→accent.
- Checkbox styled for the first time (was default iced); dialogs get a scrim + opaque base.
- Hardcoded view geometry absorbed into `spacing.rs` tokens.
- CREAM/INK/alpha constants retained in `contracts.rs` for non-GUI code (icon.rs); frosted tokens retired.

## Parts (implementation order)
1. `contracts.rs` — add zinc token block; keep legacy brand constants for icon.rs.
2. `spacing.rs` — new radii/type/geometry tokens; retire HOVER_LIFT/HOVER_BORDER.
3. `theme.rs` — rewrite: palettes, role()/is_dark()/mix()/text_on(), all style fns per spec.
4. `widgets.rs` — rewrite helpers: all button variants, styled checkbox, separator, remove dead hover_card.
5. Views: `app/view.rs`, `sidebar.rs`, `dialogs.rs`, `history.rs`, `search_bar.rs`, `preview.rs`,
   `settings.rs` — route through new styles/tokens, preserve all semantics listed in the research spec.
6. Verification gate; then docs (CHANGELOG, SUMMARY), no commit without separate confirm.

## What's left to builder
Nothing — full spec with exact signatures, token values, and per-file changes is in the research
workflow result and reproduced in the implementation. Verify against the real iced 0.13.4 source
when a Style field is uncertain (already done for checkbox/text_input/pick_list/scrollable).
