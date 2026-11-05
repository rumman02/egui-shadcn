# Porting Guide — shadcn/ui → egui (read this before porting any component)

Everything you need to port a component consistently. Read this file plus the
specific source file(s) for your component — nothing else.

## Paths

- **Original source (read-only):** `/Users/rumman/rumman/projects/egui-shadcn-ui/original-shadcn-ui/apps/v4/registry/new-york-v4/ui/<component>.tsx`
- **Style CSS (per-style decoration):** `/Users/rumman/rumman/projects/egui-shadcn-ui/original-shadcn-ui/apps/v4/registry/styles/style-*.css` (search for `cn-<component>`)
- **Examples/demos:** `/Users/rumman/rumman/projects/egui-shadcn-ui/original-shadcn-ui/apps/v4/registry/new-york-v4/examples/` (if present for your component)
- **Target:** `src/components/<component_name>.rs` (snake_case), one file per component
- **Demo:** `examples/gallery/demos/<component_name>_demo.rs` (or add `show: Some(demos::x)` in `main.rs` once written — see Gallery section)
- **Registry:** export from `src/components/mod.rs` (same commit)

## egui 0.37 API notes (this crate pins egui/eframe 0.37)

egui 0.37 changed several APIs vs older docs/examples you may remember:

- **`eframe::App::ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame)`** replaces `update(ctx, frame)`. There is no root `ctx` argument — use `ui.ctx()`.
- **Panels are unified:** `egui::Panel::left(id)` / `right` / `top` / `bottom` (no `SidePanel`/`TopBottomPanel`). Sizing: `.default_size(200.0)`, `.exact_size(…)`, `.size_range(…)`. They take `&mut Ui`: `Panel::left("x").show(ui, |ui| { … })`.
- **`egui::CentralPanel::default().show(ui, …)`** takes `&mut Ui` now.
- **`egui::Modal`** exists and is the right primitive for dialogs (`egui::Modal::new(id).show(ui, |ui| …)` — check `egui::containers::modal` for the exact API).
- **`egui::Popup`** / `egui::Tooltip` / `egui::ComboBox` / `egui::Window` / `egui::MenuBar` all exist. For anchored popovers use `Popup::from_response(response)…` style APIs where available, else `egui::Area` anchored to `response.rect`.
- **Ids:** `egui::Id::new` is deprecated. Use `egui::Id::unique("stable-string")` (deterministic — fixed seeds) or `IdSalt`. Widget-level ids use `ComboBox::from_id_salt`, `ui.push_id`, `response.id.with("suffix")`.
- **`Color32` is premultiplied alpha.** `Color32::from_rgba_premultiplied(r,g,b,a)`; egui's `from_black_alpha`/`from_white_alpha` are premultiplied-correct.
- **`CornerRadius`/`Margin` use integer types** (`u8`/`i8`): `egui::CornerRadius::same(8)`, `egui::Margin::symmetric(4, 4)`. Theme helpers `radius.md_u8()` etc. convert for you.
- **Spacing lives on `Style`, not `Visuals`**: `ctx.all_styles_mut(|s| s.spacing.item_spacing = …)`.
- **`painter.rect_stroke(rect, corner_radius, stroke, StrokeKind::Inside|Outside)`** (StrokeKind arg is required).
- **`egui::Shape::line(points, stroke)`**, `painter.circle_stroke(center, radius, stroke)`, `painter.circle_filled(center, r, color)`.
- **Animations:** `ctx.animate_value_with_time(id, target, secs)`; helpers in `crate::utils::animation` (`open_progress`, `overlay_progress`, `Duration::{Fast,Default,Slow}`).

## Theme access

```rust
use crate::theme::{ShadcnTheme, color_alpha};

let theme = ShadcnTheme::of_ui(ui);          // clone (~cheap); cache in the fn, not per widget
let p = &theme.palette;                       // all 41 tokens as Color32 (premultiplied)
p.primary; p.primary_foreground; p.border; p.muted; p.muted_foreground; p.ring; p.destructive; …
theme.radius.md()   // rounded-md in px (0.8 × --radius)
theme.radius.md_u8() // for CornerRadius
color_alpha(p.primary, 0.5)  // Tailwind bg-primary/50
crate::utils::blend(bg, overlay)  // flatten a translucent color over a backdrop
```

Components MUST read everything from the theme — never hardcode hex values.
Both light and dark must look right (the gallery toggles modes).

## Style variants

`theme.style` is one of `Style::{Nova,Vega,Maia,Lyra,Mira,Luma,Sera,Rhea}`.
The source styles differ in per-component decoration; implement **nova fully**
as the baseline, and adapt where the style CSS for your component clearly
differs (radius scale, border thickness, shadows, uppercase, etc.). If a style
requires a different structure (not just decoration), note it in PROGRESS.md
as a known gap instead of building a second implementation.

## Tailwind → px mapping (Tailwind v4, 1rem = 16px)

| Class | px | Class | px |
|---|---|---|---|
| h-6 | 24 | gap-1 / p-1 | 4 |
| h-8 | 32 | gap-1.5 | 6 |
| h-9 | 36 | gap-2 / p-2 | 8 |
| h-10 | 40 | gap-2.5 / p-2.5 | 10 |
| text-xs | 12 | gap-3 / p-3 | 12 |
| text-sm | 14 | gap-4 / px-4 / p-4 | 16 |
| text-base | 16 | gap-6 / p-6 | 24 |
| text-lg | 18 | gap-8 / p-8 | 32 |
| px-2 py-0.5 (badge) | 8 / 2 | px-4 py-2 (button) | 16 / 8 |

Hover/active/focus/disabled states in the source use alpha variants:
`hover:bg-primary/90` → `color_alpha(p.primary, 0.9)`; disabled =
`opacity-50` → scale all colors by 0.5 or use `visuals.disabled_alpha`.
`aria-invalid` → destructive border + `ring-destructive/20`.

Focus ring: `crate::utils::focus::paint_focus_ring(ui, &response, radius_px, response.has_focus())`
— 3px `ring` at 50% drawn outside the widget. Do it for every interactive
control that has `focus-visible:` in the source.

## Component file conventions

Each `src/components/<name>.rs` contains:

1. `///` doc comment with a short example at the top of the main struct.
2. Variant enums named exactly like the source (`ButtonVariant::{Default,
   Destructive, Outline, Secondary, Ghost, Link}` — Default variants get
   `#[default]`).
3. The main struct with `new(...)`, chainable setters, and `show(self, ui: &mut Ui) -> Response`
   (or `-> WidgetResponse` for compound components). Implement `egui::Widget`
   for leaf widgets so `ui.add(...)` works.
4. Compound components (Card, Dialog, Tabs, Accordion…) take closures:
   `Card::new().show(ui, |ui| { … })` with subcomponents as separate structs
   (`CardHeader`, `CardTitle`…).
5. Controlled state via `&mut` where the source is controlled
   (`Checkbox::new(&mut checked)`); internal state (open/closed, animation)
   goes in `ui.ctx().data_mut` keyed by a stable `egui::Id` — components that
   keep internal state MUST accept `id_salt(&str)` (builder method storing an
   `Option<String>`) so multiple instances work.
6. `disabled`, `invalid`, and keyboard behavior (Tab focus, Enter/Space
   activate, Arrow keys in groups, Escape closes overlays) as the source's
   Radix behavior dictates.
7. Unit tests at the bottom (`#[cfg(test)]`) — at minimum: variant/scale math,
   state toggling logic, and any pure helpers. egui-dependent drawing is
   verified via the gallery demo.

Drawing pattern for rounded filled buttons/inputs:

```rust
let response = ui.allocate_rect(rect, Sense::click());
if ui.is_rect_visible(response.rect) {
    let fill = match (variant, hovered, disabled) { … };
    painter.rect_filled(rect, theme.radius.md_u8(), fill);
    // border: painter.rect_stroke(rect, r, Stroke::new(1.0, p.border), StrokeKind::Inside)
    // focus ring: utils::focus::paint_focus_ring(...)
    painter.text(center, Align2::CENTER_CENTER, label, TextStyle::Button, fg);
}
```

## Verification (every component, before commit)

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
cargo run --example gallery   # eyeball the demo; toggle dark mode
```

Then:

1. Add the demo page to the gallery (see below).
2. Update `PROGRESS.md`: set the row to `✅ done` (or `⚠️ partial` + reason),
   add the short commit hash.
3. Commit: `feat(<component>): port shadcn <component> to egui`
   (fixes later: `fix(<component>): …`). One commit per component. Never
   batch, never push, never touch `original-shadcn-ui/`.

## Gallery demo

- Write the demo inline in `examples/gallery/main.rs` by adding a `show` fn
  pointer to `demo_pages()` for your component, e.g.

```rust
fn button_demo(ui: &mut egui::Ui) {
    use egui_shadcn_ui::prelude::*;
    ui.heading("Button");
    // every variant × size × state
}
```

and set `DemoPage { name: "button", show: Some(button_demo) }`. Keep demos
self-contained: they must compile with only the crate prelude. Show **every**
variant, size, and disabled/invalid state, roughly grouped like the shadcn
docs page.

## Forbidden

- No JS/TS/Node/WASM-glue, no new dependencies without orchestration approval
  (crate currently depends only on `egui`).
- No `unsafe`, no panics in normal use, no `unwrap()` on external data.
- No hardcoded colors/sizes where theme tokens exist.
- Do not edit files outside your batch (report foundation gaps back instead).
- Do not modify `original-shadcn-ui/`.
