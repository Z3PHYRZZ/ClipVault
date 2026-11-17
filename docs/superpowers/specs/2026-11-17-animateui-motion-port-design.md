# Design — AnimateUI motion language + ocean-wave theme transition for ClipVault

## Status — SUPERSEDED (2026-11-17)
Manuel found the wave "weird" and asked to revert it. The wave code was removed;
the final delivered work is six complete color themes (Zinc/Slate/Stone/Rose/Blue/
Green) with light+dark variants, flat root colors, plus the sidebar-button and
rounded-dropdown polish. animate-ui press feedback and reduced-motion query remain.
This document is kept as the record of the explored-and-reverted approach.

## Intent and why
Manuel asked to port animate-ui.com's motion language into the shadcn-themed ClipVault UI
"without breaking", and to make light/dark theme switching produce a wave/ocean effect.
ClipVault is native Iced 0.13 — animate-ui is React/Radix, so only its published motion
principles port (its exact numeric constants are not public):
- springs + short tweens on strong ease-out
- exits quicker than entrances
- animate transforms and opacity, not layout
- interruptible
- nothing on keyboard navigation
- reduced-motion respected everywhere

## Goals
- A reusable `gui/motion.rs` module: ease functions + a spring integrator + small
  tween/state helpers, all pure Rust, no new crates.
- Memory-driven animation: add `MotionState` to AppState; drive frames with an
  `iced::time` tick subscription only while an animation is active (zero CPU at idle).
- Ocean wave on theme switch: three-layer sine crest rises bottom-to-top across the
  full window; the new theme floods upward behind it and the base palette flips as the
  crest crosses mid-screen (raw p ~= 0.14 under quint ease-out) so the whole switch
  reads as one smooth upward sweep. Interruptible (restart on rapid toggles).
  Skipped entirely under reduced motion.
- Component motion following animate-ui rules: hover-lift easing on cards/nav (transform via
  shadow/border interpolation), press-scale on buttons, toggle thumb slide on Pin/Fav,
  entrance fade/rise on modals and history rows, status-bar toast slide — exits quicker
  than entrances.
- Reduced motion: read Windows `SPI_GETCLIENTAREAANIMATION` via the already-enabled
  `windows` crate; when off, skip all animation (instant final state).

## Non-goals
- No React/Tailwind/Framer Motion runtime — nothing from animate-ui installs here.
- No behavioral change to Message handling, workers, settings schema, or feature logic.
- No new heavy dependency; `canvas` is an existing iced feature.

## Anti-goals (failure even if compiles)
- Reintroducing INK-on-INK text (keep `text_on` contrast guard untouched).
- Breaking the 56-test green baseline or the frozen message/worker contract.
- Animating layout (causes reflow jitter) — only opacity/shadow/color/canvas transforms.
- Running a busy tick loop forever — frames only while active.

## Constraints
- iced 0.13.1; `canvas` feature gate exists (`canvas = ["iced_widget/canvas"]`).
- `canvas::Program::draw(&self, state, renderer, theme, bounds, cursor) -> Vec<Geometry>`.
- `Path::new(builder)`, `builder.move_to/line_to/bezier_curve_to`, `Frame::fill`.
- `container` style closure sees no Status; hover is message-driven already.
- Window is transparent for native DWM backdrop; wave overlay must draw on top of root
  with alpha and not erase the acrylic — use a translucent canvas layer in `stack!`.

## Approaches considered
1. **Canvas wave + state-driven tweens (chosen)** — real sine wave, native, no deps.
2. Stacked animated gradient containers — cannot make a true cresting wave; rejected.
3. Shader widget — heavier, needs wgpu feature complexity; rejected.

## Decisions (locked)
- Enable `iced` feature `canvas`.
- Motion duration: entrances 260ms ease-out, exits 160ms ease-out (exits quicker),
  spring for toggle thumb (stiffness 320, damping 26, mass 1 — settled ~250ms),
  stagger 24ms for list rows (capped at first 8 visible).
- Ease-out cubic `1-(1-t)^3` for tweens; spring integrated per frame at 60fps.
- Wave: 3 sine layers, amplitude 26/18/10px, wavelength ~window width/1.3, phase
  speed proportional to progress; background color lerps zinc light<->dark as the
  leading crest crosses mid-screen. Total 1500ms (slowed per Manuel: reads as a
  smooth ocean swell), interruptible (restart on rapid toggles). Overlay alpha is
  capped low (wash 0.16 / crest 0.22) so it never covers text. Skipped under reduced motion.
- New Messages: `MotionTick(Instant)`-style frame tick, `ThemeWaveStarted{...}`,
  `WaveFinished`. New AppState field `motion: MotionState`. All additions only.
- Theme palette continues to come from `theme_for(state.settings.theme)`; during the
  wave the *base* theme stays the old one until crest passes mid-point, then flips —
  implemented by driving a `wave_progress` value and switching theme in view via a
  derived mode while overlay draws.

## Parts
1. `Cargo.toml` — add `canvas` feature.
2. `src/gui/motion.rs` (new) — easing, spring, reduced-motion query, motion state.
3. `src/app/message.rs` / `state.rs` — additive variants + `motion` field.
4. `src/app/subscription.rs` — active-only frame tick.
5. `src/app/update.rs` — handle tick/wave messages; switch theme at crest midpoint.
6. `src/gui/views/wave.rs` (new) — canvas Program drawing 3-layer sine wave.
7. `src/app/view.rs` — overlay wave in stack; component motion hooks (fade/rise).
8. `theme.rs` / `widgets.rs` / views — apply easing to hover/press/toggle where safe.
9. Verification gate (fmt/check/clippy/tests/build) + screenshot smoke both themes.

## What's left to builder
Nothing — implement per locked decisions; verify canvas API against the vendored 0.13
source (already done: bezier_curve_to, Frame::fill, Program::draw confirmed).
