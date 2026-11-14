# ClipVault v0.1.5 — Frosted glass theme (design translation)

> **CORRECTION (v0.1.6):** the "Iced hides the HWND" claim below was wrong.
> `iced::window::run_with_handle` DOES expose the raw window handle; v0.1.6 now
> applies real DWM Mica / acrylic translucency. See `V016_ACRYLIC_PLAN.md`.
> This file remains as the historical v0.1.5 record.

User spec is CSS-flavored (`backdrop-filter`, `rgba()`, `translateY`). ClipVault is
Rust / Iced 0.13 — no DOM, no CSS, no compositor blur exposed to widgets. This doc
records the faithful translation into Iced primitives and the ONE limitation.

## Translation map
| CSS intent | Iced 0.13 reality | Implementation |
|---|---|---|
| `backdrop-filter: blur(20px) saturate(150%)` on sidebar/search | No backdrop blur. Widgets draw solid/gradient backgrounds only. | Alpha-tinted CREAM/INK surface at ~78% opacity over an ambient gradient base; 1px specular border; diffused shadow. Reads as frosted depth, not literal blur. |
| `blur(14px) saturate(130%)` on cards/preview | Same | Softer alpha tint (~65%), lighter border, softer shadow than panels. |
| `translateY(-1px)` hover | No transform | Message-driven hover (already shipped): shadow offset `-HOVER_LIFT` + border thickens + background tint shifts. Equivalent perceived elevation. |
| `rgba(255,255,255,0.4)` specular border (light) / `0.08` (dark) | `Border { color: Color }` | `alpha(WHITE, 0.40)` light, `alpha(WHITE, 0.08)` dark — but spec says keep two brand colors; WHITE is the permitted neutral. Contrasting hairline uses `alpha(INK, ..)` for separation. |
| `0 4px 20px rgba(0,0,0,0.05)` shadow | `Shadow { color, offset, blur_radius }` | `alpha(INK, 0.10)` with `offset (0,4)` `blur 20` for panels; `(0,3)` `blur 14` for cards. |
| Native Mica backdrop | Iced 0.13 does not expose HWND (`iced_winit` keeps it internal). `DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE, DWMSBT_MAINWINDOW)` needs an HWND. | **Not applied.** Would require forking iced_winit or a fragile `FindWindowW(title)` race. Documented limitation. Ambient gradient base is the framework-supported substitute. |
| Optimized blur 12–20px, no animated blur | Any blur would be GPU-heavy | No blur filter at all; only cheap constant gradients + alpha tints + static shadows. Hover changes only color/offset/border (no animated blur). Scrolling/resize stay smooth. |

## Two-color palette stays
CREAM `#F2E9E4` + INK `#22223B`; WHITE/CHARCOAL permitted neutrals for contrast.
No new accent hue. All "frost" is alpha variants of CREAM/INK/WHITE.

## New design tokens (`src/contracts.rs`, additive)
- `GLASS_PANEL_LIGHT/DARK`, `GLASS_CARD_LIGHT/DARK`: `Color` with alpha 0.78 / 0.66
  (panel deeper, card softer). Derived from CREAM (light) / INK (dark) via `alpha()`.
- `SPECULAR_LIGHT: Color` = `alpha(WHITE, 0.40)`; `SPECULAR_DARK` = `alpha(WHITE, 0.08)`.
- `HAIRLINE_LIGHT/DARK`: `alpha(INK, 0.18)` / `alpha(CREAM, 0.14)`.
- `SHADOW_AMBIENT: Color` = `alpha(INK, 0.10)`.
- `GRADIENT_LIGHT_TOP/BOTTOM`, `GRADIENT_DARK_TOP/BOTTOM`: softly shifted CREAM/INK
  (still the same hue family, subtle luminance delta only).

## Changes
1. `src/gui/theme.rs`: new `ambient_background(theme) -> Background` returns a
   `Gradient::Linear` (2-3 ColorStops) for the root view. Panels/cards get the
   alpha-tinted glass surfaces + specular borders + diffused shadows. Buttons
   stay crisp pills with alpha-tinted fill. Badge/inputs get specular edges.
2. `src/app/view.rs`: wrap root in a `container` whose style uses
   `theme::ambient_background` so the gradient sits BEHIND all frosted layers.
3. `src/gui/widgets.rs`: `hover_card` uses softer card glass; `card` same.
4. `src/gui/views/*`: search bar wrapper + sidebar panel use the deeper panel
   glass; status bar uses card glass. No per-element magic colors — all via theme.
5. Performance: all gradients are static (`Gradient::Linear` with constant stops),
   no runtime blur, no animated filter. Only a handful of layers.

## Gates
fmt --check / check --all-targets / clippy -D warnings / cargo test (49 tests) /
cargo build --release / installer / install-run-uninstall / push / v0.1.5 Release.
Version bump 0.1.4 -> 0.1.5; `winres` stays 0.1.12.

## Known limitation (must be reported, not silently skipped)
True OS-level Mica/Acrylic translucency is NOT applied. Iced 0.13 hides the HWND;
applying DWM would require patching the GUI framework or an unreliable title-based
`FindWindowW`. The ambient gradient + alpha-tinted layered surfaces are the
framework-supported approximation and keep Windows 10 compatibility (Mica is
Windows 11 only anyway).
