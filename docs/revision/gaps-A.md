# Batch A gap analysis — primitives & typography

Reference: `original-shadcn-ui/apps/v4/registry/new-york-v4/ui/*.tsx` + nova overrides in `apps/v4/registry/styles/style-nova.css`. **Nova is the default `Preset` (Style::Nova is `#[default]`) and the reference recipe**, so "P0" measures nova parity first. Rust files under `egui-shadcn-ui/src/components/`.

## Global limitations (apply to every component below — do not re-list)

- **Font weights are not available.** egui 0.37 ships only regular faces (`Ubuntu-Light`, `Hack-Regular`); `RichText::strong()` only brightens the color (egui 0.37 `widget_text.rs:250-255`, `widget_text.rs:504-505`), it does not pick a bold face, and `epaint::TextFormat` has no weight field. Every `font-medium` (button, badge, label, kbd, button-group-text, Small) and `font-semibold/extrabold` (headings, Large, InlineCode) renders regular. Not portable *exactly*; optional host-side workaround: an app can register a semibold TTF via `FontDefinitions` under a named `FontFamily` and the theme can map it — worth a README note.
- **aria-* attributes** (`aria-invalid`, `role="status"`, `aria-hidden`) have no egui a11y tree equivalent; the closest is `response.widget_info(WidgetInfo::…)`. Not portable.
- CSS `transition-*` has no direct equivalent; where a hover color eases, use `crate::utils::animation` helpers.

---

### button — low

Base recipe (`ui/button.tsx`) is well mirrored in `src/components/button.rs` (variant colors 1:1, sizes, disabled opacity, icon geometry, tests). But **the nova recipe is not implemented at all**, and the horizontal group-corner fusing is buggy.

**P0**
- **Horizontal `fuse_radius` zeroes the wrong corners** — `button.rs:545-548` (First: zeroes `sw,se` = bottom edge) and `button.rs:555-558` (Last: zeroes `nw,ne` = top edge). Source `button.tsx:13`: horizontal non-first children get `rounded-l-none` (left corners zeroed), non-last get `rounded-r-none` (right corners zeroed). egui `CornerRadius`: `nw`=top-left, `ne`=top-right, `sw`=bottom-left, `se`=bottom-right (epaint `corner_radius.rs:13-24`). Result: in a horizontal group the first button loses its bottom-LEFT rounding and the last button loses both top corner roundings — visibly wrong in the demo. The Vertical branches (`button.rs:559-572`) are correct. Fix in `fuse_radius`: `(Horizontal, First) => { corner.ne = 0; corner.se = 0; }`, `(Horizontal, Last) => { corner.nw = 0; corner.sw = 0; }`; keep Middle/Only. Update test `fuse_radius_horizontal` (`button.rs:810-822`) to expect first: `(nw,sw)=(r,r) (ne,se)=(0,0)`, last: `(ne,se)=(r,r) (nw,sw)=(0,0)`. `ButtonGroupText`/`group_adjustments` (`button_group.rs:92-113`) inherit the fix automatically.
- **Nova recipe not honored.** `resolve_colors` (`button.rs:144-236`) has no `Style` parameter and sizes are fixed base-recipe values, while the module doc (`button.rs:4-5`) claims "per-style nova CSS overrides are applied as radius decoration" — `button.rs:378` uses fixed `radius.md_u8()`. With the default Nova preset every metric is off. Fix by branching on `theme.style == Style::Nova` (pattern already used in card.rs/item.rs) with these exact nova values (`style-nova.css:149-207`):
  - geometry (`ButtonSize`): Xs h-6/px-2/gap-1/text-xs/icon-12 + radius `min(md,10px)`=8; Sm h-7(28)/px-2.5(10)/gap-1/text-[0.8rem]=12.8/icon-14 + radius 8; Default h-8(32)/px-2.5(10)/gap-1.5(6); Lg h-9(36)/px-2.5(10)/gap-1.5(6); IconXs 24/IconSm 28/Icon 32/IconLg 36; base radius `rounded-lg` = `radius.lg_u8()` (10). With icons: pl/pr-2 (8px) on Default/Lg, pl/pr-1.5 (6px) on Xs/Sm.
  - colors: Default hover `primary/80` (not /90); Destructive = tinted `bg-destructive/10 text-destructive`, hover `/20`, dark bg `/20` (not solid white-text); Outline hover `bg-muted text-foreground` (not accent), keep `dark:bg-input/30 dark:border-input dark:hover:bg-input/50`; Secondary hover = `color-mix(in oklch, secondary, foreground 5%)` ≈ `crate::theme::blend(p.secondary, p.foreground)` at 5% (blend(base, overlay) exists, `theme/mod.rs:162`); Ghost hover `bg-muted text-foreground`, dark `muted/50`.
  - active state: `active:translate-y-px` — offset the drawn rect +1px y when `response.is_pointer_button_down_on() && !disabled`.
  - `aria-invalid:border-destructive`: nova always draws a transparent 1px border, so invalid shows a destructive border on every variant — in nova mode draw `Stroke::new(1.0, p.destructive)` on invalid regardless of variant (dark: `p.destructive` at 50% via `color_alpha`).

**P1**
- **`focus-visible:border-ring` missing** — source `button.tsx:7`; `button.rs:496-501` paints the ring but never recolors the border. Fix: when `response.has_focus()`, set the stroke color to `p.ring` for variants that draw a border (Outline, and nova transparent-border modes).
- **`transition-all`** — hover/focus color changes are instant. Add ~150ms easing via `crate::utils::animation` for the fill/ring colors (persist per-id last color; low risk to skip, but it is in the source recipe).

**P2**
- `has-[>svg]:px-3` (Default), `has-[>svg]:px-2.5` (Sm), `has-[>svg]:px-4` (Lg), `has-[>svg]:px-1.5` (Xs) — `content_size` (`button.rs:312-339`) uses fixed padding regardless of icons, so icon buttons are 4-8px wider than source. Use the icon-dependent padding above.
- Update module doc `button.rs:4-5` to say what is actually implemented (nova overrides after the fix, or "base recipe only" until then).
- PROGRESS.md row 23 should note the nova-recipe gap (and the fuse bug until fixed).

**Not portable**
- `whitespace-nowrap` is satisfied via `layout_no_wrap` (OK, not a gap). `aria-*` data attributes (see global). True `font-medium`.

**Confirmed OK**
- All 6 variants × base-recipe colors incl. `dark:bg-destructive/60`, `dark:bg-input/30`, ghost `dark:hover:bg-accent/50`, secondary `/80`, link underline on hover with `underline-offset-4` approximation (`button.rs:477-484`).
- Sizes Xs..Lg + 4 icon sizes, gap 8/6/4, text 14/12, icons 16/12; `rounded-md` = `0.8*radius` matches `globals.css:51`.
- Disabled = egui-disabled response + 50% alpha on fill/fg/stroke (`button.rs:390-395`); focus ring 3px `ring/50` expanded 1px (`button.rs:498-501`, `utils/focus.rs`); invalid ring `destructive/20` dark `/40` (`button.rs:386-388`); group border overlap `border-l-0` approximation (`button.rs:414-426`); group plumbing (begin/end/next_child_position) with tests.
- Demo covers all variants, all sizes incl. icon sizes, disabled ×3 variants, invalid, icons; keyboard hint.

---

### badge — medium

**P0**
- None (nova deltas below are geometry-only; colors are P1).

**P1**
- **Nova destructive variant is solid, source is tinted** — `badge.rs:56-64` (light: `p.destructive`, dark `/60`, fg white). Nova (`style-nova.css:111-114`): `bg-destructive/10 text-destructive hover:bg-destructive/20 dark:bg-destructive/20`. Fix: branch on `Style::Nova` → fill `color_alpha(p.destructive, 0.10)` (dark 0.20), fg `p.destructive`.
- **No hover states.** Nova badge variants define unprefixed hover for Ghost (`hover:bg-muted hover:text-muted-foreground dark:hover:bg-muted/50`) and Link (`hover:underline`); Default/Secondary/Outline/Destructive are `[a]:hover` (anchor-only → correctly no-op for a static span). Badge already allocates with `Sense::hover()` (`badge.rs:141-144`), so add: if nova && variant in {Ghost, Link} && `response.hovered()` → Ghost fill `p.muted`, fg `p.muted_foreground` (dark `muted/50`); Link draws underline (reuse button.rs:477-484 pattern).

**P2**
- **Nova geometry** (`style-nova.css:95`): fixed height `h-5` (20px) — Rust height is galley+4px ≈ 18px (`badge.rs:133-136`); radius `rounded-4xl` (26px) — visually identical to `full` at h-5, no action needed. Fix height: `desired.y = 20.0` under nova (keep text centered).
- **No icon slot** — source `[&>svg]:size-3` + `gap-1` (`badge.tsx:7`, nova adds `has-data-[icon=inline-start]:pl-1.5` / inline-end `pr-1.5`). Add `Badge::icon_start(Icon)`/`icon_end(Icon)` drawing a 12px icon with 4px gap (nova) / no gap (base has `gap-1` too — badge.tsx:7 `gap-1`), nova pl/pr 6px adjustments.
- Nova outline hover is `bg-muted text-muted-foreground` (not base `accent`/`accent-foreground`) — only matters if P1 hover is implemented; include in that change.
- PROGRESS.md row 24: add "nova: h-5, tinted destructive, ghost/link hover".

**Not portable**
- `transition-[color,box-shadow]` (see global). `overflow-hidden` (n/a for a single label).

**Confirmed OK**
- Base recipe faithful: `px-2 py-0.5 text-xs` (`badge.rs:127-136`), `rounded-full`, all 6 variants with exact base colors incl. `dark:bg-destructive/60`, outline `border-border text-foreground`, ghost/link fg, disabled 50% fade; unit tests cover resolve_colors.
- Demo covers all 6 variants, inline-with-text, disabled.

---

### label — high

**P0** — none.

**P1** — none.

**P2**
- No child composition — source Label is a flex row (`gap-2`) accepting arbitrary children (icon + text). Rust `Label` is text-only (`label.rs:30-35`). Optional: add `Label::new("x").with(|ui| …)` or an `icon` builder laying out icon+text at 8px gap, vertically centered.
- `leading-none` (line-height 1 → 14px) — galley uses font default (~18px). Fix with `RichText::line_height(Some(14.0))` if the height matters to callers (it slightly tightens vertical centering).
- PROGRESS.md row 25 is accurate (base recipe, no required dot, disabled 50%).

**Not portable**
- `select-none` is moot (egui labels aren't selectable here); `peer-disabled:*` sibling selectors (the port's explicit `.disabled()` covers the use case).

**Confirmed OK**
- `text-sm` 14px foreground, `for_id` click → `request_focus` (Radix label semantics, label.rs:100-104), disabled = 50% alpha + non-interactive, tests for `text_color`.

---

### separator — high

**P0/P1** — none.

**P2**
- `decorative` only exists as a getter (`separator.rs:77-80,123-125`); fine (HTML-only semantics), but document in the field doc that it has no visual/behavioral effect in egui (already partially done).

**Not portable** — Radix `role="separator"` semantics; nothing else.

**Confirmed OK** — `bg-border`, `h-px w-full` / `w-px h-full` with `length()` override, tests for sizes and token (`separator.rs:140-173`); demo horizontal/vertical/fixed. Nova override identical to base. Nothing to change.

---

### kbd — high

**P0/P1** — none.

**P2**
- No icon slot — source `[&_svg:not([class*='size-'])]:size-3` (`kbd.tsx:9`). A `Kbd::icon(Icon)` (12px glyph, `gap-1`) would let callers render key+arrow chips like the docs; low priority since dropdown shortcuts use text.
- Tooltip-context variant `[[data-slot=tooltip-content]_&]:bg-background/20` (`kbd.tsx:10`) is composition-scoped (only inside tooltip content); if the tooltip component wants it, expose `Kbd::on_dark()` fill = `color_alpha(p.background, 0.2)` fg = `p.background` (dark: `background/10`). Defer to batch E.

**Not portable** — `pointer-events-none` (moot).

**Confirmed OK** — `h-5 min-w-5 px-1 rounded-sm bg-muted text-muted-foreground font-sans` all match (`kbd.rs:56-91`, radius sm = 0.6×radius matches globals.css:50); `KbdGroup` gap-1; chip_width math tested incl. min-w-5; demo covers single keys, groups, inline. Nova kbd identical to base — no delta.

---

### spinner — high

**P0/P1** — none.

**P2**
- **Arc geometry differs slightly from lucide.** `spinner.rs:19-22`: `ARC_RADIUS=10.0`, `ARC_SWEEP=280°`. lucide `Loader2Icon` (loader-circle) path is `M21 12a9 9 0 1 1-9-9` → center (12,12), **radius 9**, start at 3 o'clock, **sweep 270° clockwise**, gap = the top-right quadrant. Fix: set `ARC_RADIUS=9.0`, `ARC_SWEEP=270.0` (tests at `spinner.rs:130-151` need updated endpoints: last point at 270° = (12, 2) on the 24-grid). Same constant is duplicated in `src/icons/icons.rs:124` (`El::Arc([12,12], 10.0, -80.0, 200.0)`) — update both in one commit so inline icons and Spinner match.
- `ui.ctx().request_repaint()` runs even when the rect is offscreen (`spinner.rs:93-94`); move it inside `if ui.is_rect_visible(rect)`.

**Not portable** — `role="status" aria-label="Loading"` (global a11y note).

**Confirmed OK** — `size-4` default 16px, 1 rev/s linear rotation = `animate-spin`, `currentColor` → foreground default, `color()` override, scale-correct stroke (2px at 24-grid), tests for rotation/arc; demo sizes/colors/in-button.

---

### skeleton — medium

**P1**
- **Fill wrong for 7 of 8 styles.** `skeleton.rs:25-30` returns `p.accent` for any style ≠ Nova. **Every** style file overrides `.cn-skeleton` to `bg-muted` (nova/vega/mira `style-*.css` — all 8 checked: `@apply bg-muted …`), so the `bg-accent` base recipe only applies to an unstyled skeleton that shadcn never ships. Fix: `fill_color` returns `p.muted` for all styles (keep the Style param only for the radius below, or drop it).
- **Per-style radius missing.** nova/mira/vega: `rounded-md`; luma/rhea: `rounded-2xl`; maia: `rounded-xl`; lyra: `rounded-none`; sera: base `rounded-md`. Fix in `Skeleton::show` (`skeleton.rs:97`): `match theme.style { Luma|Rhea => radius.x2l_u8(), Maia => radius.xl_u8(), Lyra => 0, _ => radius.md_u8() }` unless `corner_radius` was set explicitly.

**P2**
- PROGRESS.md row 29 says "base bg-accent" — after the fix, reword to "all styles bg-muted; radius per style (nova md)".
- Demo covers text lines, card placeholder, the docs' `h-[125px] w-[250px]` block, and animate-off. Good.

**Not portable** — nothing.

**Confirmed OK** — `rounded-md` default, 2s pulse 1.0→0.5 with cosine approximating `cubic-bezier(0.4,0,0.6,1)` (tests at skeleton.rs:117-125), repaint only while animating, size builder matches demo examples.

---

### typography — medium

Source: docs composition (`apps/v4/registry/new-york-v4/examples/typography-*.tsx`, no ui/*.tsx). Element coverage is complete and matches PROGRESS row 30. The misses are typographic detail.

**P0** — none.

**P1**
- **No line heights.** Tailwind v4 pairs every text size with a line-height (`text-4xl`=40, `text-3xl`=36, `text-2xl`=32, `text-xl`=28, `text-lg`=28, `text-base`=24, `text-sm`=20) and the recipes override (`leading-7`=28 on P, `leading-none`=14 on Small). egui's default line spacing (~1.3× font size) makes wrapped paragraphs denser than the source. egui 0.37 supports this: `RichText::line_height(Some(px))` (egui 0.37 `widget_text.rs:174`) or `TextFormat { line_height: Some(px), .. }` in the LayoutJob used by `galley_for` (`typography.rs:53-68`). Fix: add `line_height: Option<f32>` to `galley_for` and set per recipe — H1 40, H2 36, H3 32, H4 28, P 28 (`leading-7`), Lead 28, Large 28, Small 14 (`leading-none`), Muted 20, Blockquote 24. Height math (`typography.rs:128,260,477`) already uses galley size, so it picks this up.
- **Blockquote is not italic** — source `italic` (`typography-blockquote.tsx`); `typography.rs:249-279` draws the bar + text but never sets italics. Fix: add `italics: bool` to `galley_for` → `TextFormat.italics = true` for Blockquote only.

**P2**
- `InlineCode` radius: source uses plain `rounded` = 4px; `typography.rs:435` uses `radius.sm()` = 6px. Fix: literal `4` (plain `rounded` is not remapped by shadcn's `--radius-*` block, `globals.css:50-56` only overrides sm..4xl).
- Block margins (`mt-6` p/blockquote, `mt-10` h2, `mt-8` h3, `my-6` lists, `first:mt-0`) are left to callers — fine in egui, but document each recipe's margins in the module table (`typography.rs:5-21`) so demo/callers match (the demo uses 8-12px gaps, source is 24/32/40px).
- `CodeBlock` has no source recipe (docs prose styling; `rounded-md bg-muted p-4 font-mono` is the porter's invention) — say so in its doc comment (typography.rs:454).
- Demo lacks the docs' typography-table composition (the `table` demo covers tables; acceptable — add a pointer note or drop from the doc table).
- Headings render regular-weight; bold is unavailable (global note). If a semibold face gets registered by the host, thread it through `heading()`.

**Not portable**
- `text-balance` (h1), `scroll-m-20`, `[&:not(:first-child)]:mt-6` auto-margins (immediate mode has no sibling knowledge), `even:bg-muted` table striping belongs to the table component, not typography.

**Confirmed OK**
- Exact sizes 36/30/24/20/18/16/14 (tests typography.rs:647-656), `tracking-tight` ≈ default (letter-spacing 0 is acceptable approximation, note if tightened later), H2 `border-b pb-2` with correct 8px gap (`typography.rs:127-140`), Blockquote `pl-6 border-l-2` = 26px pad (`typography.rs:47`), Ul/Ol `ml-6` + `li mt-2` (`typography.rs:328-331`), InlineCode px-[0.3rem]/py-[0.2rem] = 4.8/3.2pt (`typography.rs:429-431`), CodeBlock p-4 + mono + md radius, Lead/Large/Small/Muted/TableCaption colors and sizes, galley wrap tests.

---

### aspect-ratio — high

**P0/P1/P2** — none. Thin Radix wrapper with no styles; Rust maps it to an allocate+child-ui with `height_for_width` clamped against 0/NaN/inf (tested, `aspect_ratio.rs:79-95`), `max_width` covers the common demo case. Radix's height-driven inverse sizing (fixed height, derived width) is unused in every demo; if ever needed, add a `height()` builder.

**Confirmed OK** — 16/9, 1/1, 4/3 demos; `Default` = 16/9 like the docs example.

---

### marker — medium

**P0** — none.

**P1**
- **Border variant has a fixed 25px row** — `marker.rs:77-83` computes `row_height()` = 16+8+1 before knowing the content, and `show` allocates that exact height (`marker.rs:98`), painting the bottom rule at `rect.top() + height - 0.5` (`marker.rs:138-145`). Source `border-b pb-2` grows with content (`min-h-4` is a floor); wrapped `MarkerContent` overflows the 25px box. Fix: paint the rule from the child's measured rect instead — after `content(&mut child)` use `let m = child.min_rect(); painter.line_segment([m.left(), m.bottom()+8.0+0.5 → m.right(), …])` i.e. rule at `m.bottom() + 8.0` (pb-2), and reserve height via the two-frame pattern used elsewhere in the crate (store last frame's `m.height() + 9.0` under the id_salt in `ctx.data_temp`, default 25.0 first frame).

**P2**
- **Separator rule gap is 4px, source is 12px** — flex `gap-2` (8px) applies between the before/after pseudo-elements and the content, plus `before:mr-1`/`after:ml-1` (4px) (`marker.tsx:13`). `marker.rs:114` uses `spacing::XS` (4). Fix: `let gap = crate::theme::tokens::spacing::SM + crate::theme::tokens::spacing::XS;` (12.0).
- `[a]:underline [a]:underline-offset-3 [a]:hover:text-foreground` link styling inside MarkerContent — `MarkerContent` is a plain label. If needed, allow `MarkerContent::with(|ui| …)` so callers can compose (egui `Label` + underline via painter). Mostly web composition; document.

**Not portable** — nothing else.

**Confirmed OK** — `text-sm text-muted-foreground gap-2 min-h-4 w-full`, `size-4` icon with muted color and lucide-scaled stroke, all 3 variants, separator rules centered on the row (`marker.rs:117-134`), demo covers all variants incl. icon+content separator; nova identical to base.

---

### button-group — low

Layout plumbing (begin/end/next_child_position, tests) is solid, but the shared horizontal corner fusing is broken (see button P0) and nova deltas are unimplemented.

**P0**
- **Horizontal first/last corners wrong** — inherited from `button.rs:545-558` via `group_adjustments` (`button_group.rs:92-113`) and `fuse_radius`; fix there (Horizontal First → zero `ne,se`; Horizontal Last → zero `nw,sw`) and fix the duplicated expectations in `button.rs` tests. The demo's "Day Month Year" and outline pairs visibly show square bottom/top corners on the wrong sides.
- **ButtonGroupText nova padding/radius** — `button_group.rs:176-179` uses px-4 (16) and `button_group.rs:200` uses `radius.md_u8()`; nova (`style-nova.css:222-225`) is `rounded-lg px-2.5` (10px). Fix under nova: desired width `galley.x + 2*10.0`, radius `radius.lg_u8()`.

**P1**
- **Nested-group `gap-2` missing** — source `button-group.tsx:8` `has-[>[data-slot=button-group]]:gap-2` (kept in nova, line 210-212). Rust forces 0 spacing (`button_group.rs:76-83`) and PROGRESS row 16 documents the gap. Fix: add `ButtonGroup::gap(f32)` (default 0.0) and set the group's `item_spacing` from it; when the fix agent can't auto-detect nested groups, demos/callers of nested groups pass `.gap(spacing::SM)`. Update the PROGRESS row after.
- **`items-stretch` not honored** — children keep their own heights (a group mixing `Sm` buttons and a default `ButtonGroupText` gets ragged heights; CSS stretches all to the tallest). Fix: extend `button::group::GroupState` with `max_child_height` (last frame's max of children that opt in — Button, ButtonGroupText, ButtonGroupSeparator), and stretch each member to it when > its own height (2-frame pattern already in place for totals).

**P2**
- Nova orientation overrides (`style-nova.css:214-217`): last child forced to `rounded-r-lg!` / `rounded-b-lg!` — apply after the fuse fix as a nova-branch on the last child's outer corners.
- ButtonGroupText has no icon slot though the code reserves "gap-2 slack for leading icons" (`button_group.rs:175-179`) — either add `icon_start(Icon)` (16px icon + 8px gap, `[&_svg]:size-4`) or drop the slack; currently dead width.
- Separator `self-stretch` — source stretches to the row (`button-group.tsx:70`); Rust uses fixed `length` default 36 (`button_group.rs:255-261`). Covered by the P1 stretch; alternatively keep `length()` and note it.
- Demo: add a nested-group case (outer group containing a ButtonGroup) once `.gap()` exists; add an invalid/`aria-expanded` accent case not needed (source has none here).

**Not portable**
- `[&>*]:focus-visible:z-10` stacking (egui paints in order; a focused child could be re-painted last via `ui.painter().add` ordering if it ever matters — skip).
- `[&>select[aria-hidden=true]:last-child]` select-trigger corner rule — handled by native-select integration if/when needed.

**Confirmed OK**
- Zero item spacing fusing, `border-l-0`/`border-t-0` overlap trick (1px draw-rect shift, `button.rs:414-426` / `button_group.rs:99-112`), Vertical corner fusing correct, `ButtonGroupSeparator` uses `bg-input` token (`button_group.rs:296`), Text slot `bg-muted border shadow-xs text-sm` colors tested, vertical demo with separators, disabled member demo.

---

## Batch summary

Work needed, most first:

1. **button (low)** — biggest item: implement the nova recipe (geometry + all 6 variant color sets + active shift + invalid border) behind `Style::Nova`, fix the horizontal `fuse_radius` bug (also tested-in wrong behavior), add `focus-visible:border-ring`. The base recipe itself is already faithful, so this is additive branching, not a rewrite.
2. **button-group (low)** — mostly rides on the button.rs fuse fix; own items are nova ButtonGroupText metrics, the nested-group `gap-2` API, and cross-child height stretch (items-stretch).
3. **typography (medium)** — line heights per recipe via egui 0.37 `RichText::line_height`/`TextFormat.line_height` (concrete px list above) and Blockquote italics are quick wins; InlineCode radius 4px. Heading weights stay regular unless the host registers fonts.
4. **badge (medium)** — nova tinted destructive + fixed h-5 height + optional ghost/link hover + icon slot.
5. **skeleton (medium)** — one-line fill fix (bg-muted for all styles) + per-style radius table; the current non-nova fill contradicts every style file.
6. **marker (medium)** — separator rule gap 4→12px and content-growing border variant.
7. **spinner (high)** — one P2: arc constants to lucide's 270°/r9 (shared with `icons/icons.rs`).
8. **kbd (high)** — one P2: optional 12px icon slot.
9. **label (high)** — one P2: optional child/icon composition; otherwise faithful.
10. **separator (high)** / **aspect-ratio (high)** — done; no changes.

Gallery housekeeping noticed while checking demos (not batch A files): `examples/gallery/main.rs:326-357` contains stale duplicate demo-page entries with `show: None` (popover, tooltip, dropdown-menu, context-menu, menubar appear twice) that render "Demo not ported yet." pages — delete the duplicates.

PROGRESS.md rows 6-16 are accurate as descriptions but none of them records the nova-recipe gaps for button/badge/button-group, the skeleton fill claim ("base bg-accent") is misleading (all styles use bg-muted), and row 23/16 should flag the fuse bug until fixed.
