# Batch C — display & layout gap analysis (card, alert, avatar, progress, table, empty, item, scroll-area, resizable)

Reference: `original-shadcn-ui/apps/v4/registry/new-york-v4/ui/<name>.tsx` + `apps/v4/registry/styles/style-nova.css` (default style, the reference recipe). Rust: `egui-shadcn-ui/src/components/<name>.rs`, demos in `examples/gallery/demos/mod.rs`.

Batch cross-cutting findings (apply to several components):

- **(a) Nova recipe status — OK for batch C.** Every component here either implements the nova deltas via `theme.style == Style::Nova` dispatch (card, alert, progress, empty, item, resizable) or needs none (table, avatar, scroll-area: nova CSS == base TSX). The sibling issue ("nova not implemented, base used instead") does **not** apply to batch C.
- **(a2) Non-nova styles fall back to the base recipe, not their own recipes.** All 8 style files define per-style recipes for card/item/progress/empty etc. that the Rust ports treat as `_ => base`. Examples: card `--card-spacing`: maia 24 ✓, lyra/mira 16, sera 32, rhea 20 (Rust always 24); progress track: vega h-1.5=6, maia/luma h-3=12, sera h-0.5=2, rhea h-2=8 ✓ (Rust always 8); item radius maia/luma/rhea `rounded-2xl`, lyra/sera `rounded-none`, text-xs on lyra/mira (Rust always base md/base 14); all styles use `bg-muted` progress track, never `bg-primary/20`. P2 batch-level: extend the per-style match arms when non-nova styles are promoted, or state in README that only nova + base are supported.
- **(b) Font weights not conveyable** (no medium/semibold faces in egui). Affected here: CardTitle (`font-semibold` base / `font-medium` nova, card.tsx:34 + nova css:253), AlertTitle (`font-medium`, alert.tsx:41), EmptyTitle (`font-medium`, empty.tsx:64), ItemTitle (`font-medium`, item.tsx:124), TableHead (`font-medium`, table.tsx:72), TableFooter (`font-medium`, table.tsx:46). Not portable without bundling font faces; do not fake with `.strong()` (foreground is already the darkest color).
- **(c) Line-heights missing — fixable.** `epaint::TextFormat::line_height: Option<f32>` exists in egui 0.37 (text_layout_types.rs:497), so galley-painted components can set it. Affected: CardTitle `leading-none` (1.0×size) base / `leading-snug` (1.375) nova; AlertDescription `[&_p]:leading-relaxed` (1.625); EmptyDescription `text-sm/relaxed` (14/20px); ItemTitle `leading-snug`; ItemDescription `leading-normal` (1.5 → 21px @14). Fix: replace `painter().layout*` with a `LayoutJob` whose `TextFormat` sets `line_height` per the recipe.

---

### card — medium

**P0 (visual/API wrong — must fix)**
- **CardFooter lays out vertically** — card.tsx:77 `flex items-center px-6`; card.rs:366-380 `egui::Frame::new().inner_margin(...).show(ui, content)` inherits the parent's top-down layout, so multiple footer children (e.g. the demo's "Create"/"Cancel" buttons, demos/mod.rs:1477-1482) stack vertically. Fix: wrap content in `ui.horizontal(|ui| { ui.spacing_mut().item_spacing.x = 8.0; content(ui) })` inside the existing frame, keeping `inner_margin(Margin::symmetric(spacing, pad_v))` (nova band) unchanged for both nova and base arms.

**P1 (behavior/interaction gap)**
- **CardAction not inset by header padding** — card.rs:186-193 records `HeaderCtx { right: cursor.left() + ui.available_width() }` *before* the header Frame, so the action region's right edge is the card's inner edge (flush with the ring), while the title is inset by `px-6`/`px-4` (card.tsx:22, nova css:249). egui `new_child` does not clamp to the parent max_rect, so the demo's icon button touches the card edge. Fix: move the `push_header` call inside the `Frame::show` closure (record `cursor.left()/top()` and `ui.available_width()` of the already-padded ui), or subtract `pad` from `right`. Verified `AlertAction` and `ItemActions` record inside their padded containers and are correct.

**P2 (polish)**
- **Base-recipe title size wrong** — card.rs:59-62 gives non-nova styles `title_size: type_scale::SM` (14px). Base CardTitle has no size class (card.tsx:34) and inherits 16px; nova is explicitly `text-base` (css:253). Both recipes render 16px. Fix: `_ => title_size: type_scale::BASE` (16.0); update `nova_uses_ring_and_larger_title` test.
- **Nova `data-[size=sm]` card not implemented** — nova css:245 `data-[size=sm]:[--card-spacing:--spacing(3)]` (12px) + title `group-data-[size=sm]/card:text-sm`. Fix: add `pub enum CardSize { Default, Sm }` + `Card::size(...)`, `card_spacing()` returning 12.0 for nova-sm, title 14px.
- **`overflow-hidden` / media image slots missing** — nova css:245 `overflow-hidden`, `has-[>img:first-child]:pt-0`, `*:[img:first-child]:rounded-t-xl`. No Card image-slot API; a full-bleed top image cannot clip to the rounded corners. Fix: optional `Card::image_top(ImageSource)` that paints a cover-cropped image above the inner content and reduces top padding to 0.
- Line-heights (cross-cutting c) and title weight (cross-cutting b).
- Demo label "Destructive alert inside a card" (demos/mod.rs:1506) describes a danger-zone card with a destructive button, not an alert. Rename to "Destructive action in a card".

**Not portable**
- `[.border-b]:pb-6` / `[.border-t]:pt-6` conditional padding — depends on classes the app adds manually; no Rust equivalent trigger.
- `@container/card-header` grid auto-rows semantics — approximated by the top-right overlay; equivalent for self-start actions.

**Confirmed OK** — nova `ring-foreground/10` 1px ring (card.rs:53), 16px `--card-spacing` vs base 24 (31-36), `py` only card margin + slot `px` (141, 196), gap chain (card 16/24, header 4/8, content/footer px), `bg-card` fill + `text-card-foreground` override (147), shadow-sm base / none nova (130-134), nova footer full-bleed `bg-muted/50 border-t rounded-b-xl` band with ring restoration (382-422), description `text-sm text-muted-foreground` (239-264), title min sizes, tests for spacing/colors/render.

---

### alert — high

**P0** — none.

**P1** — none.

**P2 (polish)**
- **`pr-18` reservation applies in the base style too** — alert.rs:188-191 reserves 72px whenever an `AlertAction` is present; the base TSX has no action slot (alert.tsx exports only Alert/Title/Description), and the reservation shifts content left in Vega..Rhea. Either gate on `theme.style == Style::Nova` or document the slot as a nova-era extension in the module docs (it already says "nova `absolute top-2 right-2`" — make the reservation nova-only for consistency).
- Line-height for description (`leading-relaxed` on `<p>`) — cross-cutting (c).
- Title `tracking-tight` not conveyed — no letter-spacing support on `FontId`; not portable.
- Demo note: "Success and info icons" alerts use the default variant with a foreground-colored icon — matches the source's `[&>svg]:text-current`; no change needed, but a comment would prevent future "fixes" toward colored icons.

**Not portable**
- `role="alert"` / aria wiring (alert.tsx:29) — egui has no live-region announcement; AccessKit roles could be added later (`response.widget_info`), out of scope.

**Confirmed OK** — `px-4 py-3` / nova `px-2.5 py-2` (39-52), `gap-y-0.5`=2px and `gap-x-3`=12 / nova `gap-x-2`=8 (56-61, 223, 239), `w-full` min width (220), 16px icon column + `translate-y-0.5` 2px nudge + 1.33 stroke (224-235), destructive `text-destructive` title / `text-destructive/90` description / `text-current` icon (82-88), default `bg-card text-card-foreground` + muted description (75-81), title `min-h-4` 16px (272-275), rounded-lg (180), action at `top-2 right-2` = 8px (199-203), `pr-18`=72px (191), light/dark render tests.

---

### avatar — high

**P0** — none.

**P1** — none.

**P2 (polish)**
- **Badge ring is 1px, not `ring-2`** — avatar.tsx:61 `ring-2 ring-background`; avatar.rs:247-249 draws bg circle at `bs*0.5 + 1.0` → 1px ring outside the badge. Fix: `painter.circle_filled(center, bs * 0.5 + 2.0, p.background)` (keep the group ring at radius+1/width 2 which is correct, 235-241).
- **Group ring overlaps the badge ring** — both draw `background` circles at the same corner; harmless visually (same color) but after the badge fix the badge ring (2px) and member ring (2px) coincide; no action needed, just verify in demo.
- Demo covers all sizes, image/icon fallbacks, plain/icon badges incl. sm-hides-icon, and groups with counts (demos/mod.rs:1597-1709) — complete.

**Not portable**
- Radix `AvatarImage` loading/delay states — replaced by texture-or-fallback degrade (documented); equivalent behavior.
- `select-none` — no text selection on painted glyphs.

**Confirmed OK** — sizes 24/32/40 (46-54), fallback `bg-muted text-muted-foreground` with `text-sm`/sm `text-xs` (56-61, 204-216), initials helper, icon fallback, image cover-crop circular clip (224-232, 300-315), group `-space-x-2` = −8px spacing (399-402), member `ring-2 ring-background` (235-241), group-count size/icon adaptation `size-4/3/5` (81-88) and `bg-muted text-sm muted-foreground` (448-482), badge sizes `size-2/2.5/3` and sm icon hidden (64-79), light/dark render tests.

---

### progress — high

**P0** — none. **P1** — none.

**P2 (polish)**
- `value || 0` in the source coerces NaN→0; Rust `fill_fraction` handles NaN by clamping to 0.0 implicitly? `(f32::NAN / 100.0).clamp(0.0, 1.0)` returns NaN (clamp with NaN is identity). Fix: `if value.is_nan() { 0.0 } else { ... }` in `fill_fraction` (progress.rs:35-37) plus a test.
- Consider exposing a `label`/`value` text slot: nova css:959-963 defines `.cn-progress-label` / `.cn-progress-value` (`text-sm font-medium` / `muted-foreground ml-auto tabular-nums`) used by the newer progress with label — the TSX in scope has no label, so this is optional future parity.
- Demo covers default animation, 0/25/50/75/100, clamped over/under, fixed width (demos/mod.rs:1711-1766) — complete.

**Not portable**
- `role="progressbar"` aria valuenow/min/max — same AccessKit caveat.

**Confirmed OK** — track `h-2 w-full rounded-full` base / nova `h-1` (19-24), `bg-primary/20` base / nova `bg-muted` (27-32), indicator `bg-primary` fill (101), `transition-all` ≈ `animate_value_with_time` 150ms `Duration::Default` (93-97), value clamping 0..=100 (35-37), `w-full` default + `.width()` override (74-75), radius full (87), zero-value draws no fill (98), light/dark render tests, Widget impl.

---

### table — medium

**P0 (visual/API wrong — must fix)**
- **`TableHead` height is 36px, source is `h-10` = 40px** — table.tsx:72 `h-10 px-2`; table.rs:51 `pub const TABLE_HEAD_HEIGHT: f32 = 36.0;` (comment even says h-10) and the test at 583-587 asserts 36.0. Fix: `TABLE_HEAD_HEIGHT = 40.0` (or `tokens::spacing::CONTROL_LG`), update the test.

**P1 (behavior/interaction gap)**
- **Row click does not work; demo's click-to-select is dead** — table.rs:401 `ui.interact(rect, id, Sense::hover())`, so `resp.response.clicked()` in table_demo (demos/mod.rs:1856) can never fire (hover-sense responses never report clicks). Fix: add `TableRow::interactive(bool)` (builder field default false) and use `Sense::click_and_drag()` when set; update table_demo rows to `.interactive(true)`. Keep non-interactive rows hover-only to match the source (rows are not buttons by default).

**P2 (polish)**
- **Caption `mt-4` missing** — table.tsx:100 `mt-4 text-sm text-muted-foreground`; typography::TableCaption (typography.rs:620-634) is plain Muted with default 8px spacing. Fix: in `TableCaption::show`, `ui.add_space(16.0)` before the label (or document that callers add the space).
- **Footer `font-medium` not conveyed** (cross-cutting b) — table.rs TableFooter band is correct (`bg-muted/50 border-t`, 319-340) but cells render normal weight. Not portable; optionally allow `TableCell::text` to take a `strong` flag once a medium face exists.
- First-frame flash of the last-row border — the 2-frame tracker (59-138) drops the final `border-b` one frame late. Acceptable; note in docs.
- `[&>[role=checkbox]]:translate-y-[2px]` / checkbox `pr-0` — not applicable; the demo's badge cells already look right.

**Not portable**
- Horizontal scroll container (`overflow-x-auto`, table.tsx:10) — callers wrap in `ScrollArea::new().horizontal(true)`; mention in docs.
- `align-middle` on cells is honored via vertical centering in `show_text`; closure cells control their own layout.

**Confirmed OK** — row `border-b` + hover `bg-muted/50` + selected `bg-muted` with selected-wins precedence (40-48, 400-410), body/footer last-row `border-0` tracker (97-137, 389-391, 411-419), head `px-2` + `text-foreground` + vertical centering (458-479), cell `p-2` (523), `whitespace-nowrap` via `layout_no_wrap` (463, 537), `w-full` min width (198), footer full-width band + top border (319-340), nova == base (no style dispatch needed — verified against style-nova.css:1267-1313), caption reuse, light/dark render test with header/body/footer/selected rows.

---

### empty — medium

**P0** — none.

**P1 (behavior/interaction gap)**
- **`mb-2` applied above the media instead of below** — empty.tsx:31 `mb-2`; empty.rs:314-315 (`EmptyMedia::show`) and 339 (`show_with`) call `ui.add_space(8.0)` *before* allocating. Net effect: 8px gap above the media (source: 0) and gap-below = item_spacing only (source: item_spacing + 8). Fix: move `ui.add_space(8.0)` after the media allocation/`allocate_ui` in both `show` and `show_with`.

**P2 (polish)**
- **`EmptyFooter` is an invented slot** — not exported by empty.tsx (96-103: Empty, Header, Title, Description, Content, Media). empty.rs:444-475 + demo usage (demos/mod.rs:1938). Either remove it (update demo to `EmptyContent`) or mark `#[doc(hidden)]`/document as an extension in PROGRESS.md (row 34 already says "+Footer ext.").
- **`md:p-12` not implemented** — empty.tsx:9 pads 48px at md; empty.rs:156 always 24 (nova p-6 = 24 ✓). For the base style consider `Empty::padding(f32)` or note the fixed 24px.
- **Description link styling missing** — empty.tsx:75 `[&>a]:underline underline-offset-4 hover:text-primary`. Fix option: add `EmptyDescription::link(text)` rendering RichText `.underline()` in `p.primary` with hover color swap (egui `RichText::underline()` exists; hover recolor needs a Sense::hover response + pointer check).
- Title `tracking-tight` (empty.tsx:64) — not portable.
- `text-balance` (empty.tsx:9) — no balanced line-breaking in egui; not portable.
- Dashed border approximation (177-221) draws straight dashed segments and skips the corners — acceptable; the dash pattern 6/4 could be noted in docs.

**Not portable**
- `line-clamp` n/a; `text-balance` n/a.

**Confirmed OK** — base `gap-6`/nova `gap-4` (41-46), base `rounded-lg`/nova `rounded-xl` (146-150), `p-6` = 24 (156), centered column `items-center text-center` (162), header `max-w-sm`=384 + `gap-2`=8 (73, 250-258), media icon variant base `size-10`+`size-6` glyph / nova `size-8`+`size-4`, `rounded-lg bg-muted text-foreground` (49-62, 316-328), title base `text-lg`/nova `text-sm` (64-70), description `text-sm text-muted-foreground` (377-400), content `gap-4`/nova `gap-2.5` (421-437), `border-dashed` only-when-bordered matching the docs markup (124-127), fill-height opt-in for `flex-1 justify-center`, light/dark render tests.

---

### item — medium

**P0** — none.

**P1 (behavior/interaction gap)**
- **No interactive-item path (hover bg + focus ring)** — item.tsx:34 `focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 [a]:hover:bg-accent/50` (nova css:704 `[a]:hover:bg-muted`). The Rust `Item` (item.rs:265-308) is a static frame with no Sense and no ring, so the documented "item as a clickable row" recipe (source demos use `asChild` anchors) is impossible. Fix: add `Item::interactive(bool)`; when true, allocate the frame with `Sense::click()` on the frame response, paint hover fill `color_alpha(p.accent, 0.5)` (base) / `p.muted` (nova) over `c.fill`, and paint the focus ring via `crate::utils::focus::paint_focus_ring(ui, &response, radius, response.has_focus())` (ring color `color_alpha(p.ring, 0.5)`, 3px, plus stroke recolor to `p.ring`). Return the `Response` (extend `InnerResponse` usage as-is).

**P2 (polish)**
- **ItemGroup gap applied in base style** — base ItemGroup is `flex flex-col` with no gap (item.tsx:13); item.rs:80-86 applies nova gaps (16/10/8) to all styles. Fix: `group_gap(style, size)` returns 0.0 for non-nova styles, or document.
- **Xs-size description not shrunk** — nova css:755 `group-data-[size=xs]/item:text-xs`; item.rs:507-513 always uses SM. Fix: read `current_item(ui).map(|c| c.size)` in `ItemDescription::show` and use `type_scale::XS` (12.0) for `ItemSize::Xs`.
- **ItemTitle cannot host an icon** — item.tsx:124 `flex w-fit items-center gap-2`; current API is text-only (469-491). Add `ItemTitle::show_with(ui, |ui| ...)` closure variant.
- **`line-clamp-2` on description** (item.tsx:137) — could truncate by galley rows (`galley.rows.len() > 2` → elide); mark low priority.
- Media box radius uses `sm()` = 6px (`rounded-sm` = 0.6×radius ✓ item.rs:368) — correct; no change.

**Not portable**
- `asChild`/Slot polymorphism (item.tsx:62) — replaced by composition closures; fine.
- `transition-colors duration-100` — instant recolor; egui has no per-property CSS transitions here.

**Confirmed OK** — nova geometry default/sm `px-3 py-2.5 gap-2.5` and xs `px-2.5 py-2 gap-2`; base default `p-4 gap-4`, sm `px-4 py-3 gap-2.5` (64-76, tests 744-761), variants default/outline/muted fills+strokes (95-110), nova `rounded-lg` vs base `rounded-md` (274-278), media icon box `size-8 rounded-sm border bg-muted` + `size-4` glyph (113-117, 370-391), image media `size-10` with nova sm `size-8`/xs `size-6` (393-398), content `gap-1`=4 (459), actions `gap-2`=8 with 2-frame width reservation and right-align inside header/footer (543-570), header/footer `basis-full` + justify-between emulation (593-649), separator base `my-0`/nova `my-2` (716-731), group size inheritance (683-693), light/dark render test.

---

### scroll-area — medium

**P0 (visual/API wrong — must fix)**
- **Scrollbar colors are inverted: the doc's assumption about egui 0.37 is wrong** — scroll_area.rs:115 sets `visuals.extreme_bg_color = p.border` believing it colors the handle, but in egui 0.37 the solid bar paints the *track/background* with `Visuals::extreme_bg_color` and the *handle* with `widgets.inactive/hovered/active.bg_fill` (egui-0.37.0/src/containers/scroll_area.rs:1638-1657; `ScrollStyle::solid()` sets `foreground_color: false`, style.rs:608). Result: a 10px opaque `bg-border` track (source has none — scroll-area.tsx:42 `border-l-transparent`) with a `p.secondary` handle that turns `p.accent` on hover — the opposite of shadcn's invisible track + `bg-border` thumb (scroll-area.tsx:51, nova css:998-999). Fix inside `ScrollArea::show`'s scope: `let v = ui.style_mut().visuals; v.extreme_bg_color = Color32::TRANSPARENT; v.widgets.inactive.bg_fill = p.border; v.widgets.hovered.bg_fill = p.border; v.widgets.active.bg_fill = p.border; v.widgets.inactive.corner_radius = egui::CornerRadius::same(5);` (5 = bar_width/2 for a true `rounded-full` thumb — widgets.inactive.corner_radius is otherwise md=8px). Nested TextEdits are safe: `text_edit_bg_color` is `Some(p.input)` from `apply_visuals`, so it never falls back to `extreme_bg_color`. Update the module docs and the `scrollbar_style_matches_source` test.

**P1** — none.

**P2 (polish)**
- **Viewport focus ring gap** (scroll-area.tsx:20 `focus-visible:ring-[3px] ring-ring/50 outline-1`) — already noted in PROGRESS.md row 36 (egui ScrollArea is not focusable); keep as known gap.
- The source scrollbar overlays content (`absolute`, radix) while egui solid bars consume 10px of layout width — inherent egui difference; consider documenting (floating bars would hide the thumb when idle).
- Thumb min length `handle_min_length: 12.0` is an invented value — fine, note in docs.

**Not portable**
- `transition-[color,box-shadow]` on the viewport, radix `Corner` element.

**Confirmed OK** — track width `w-2.5`=10 (27, 94), `p-px` = 1px inner/outer margins (95-96), solid always-visible thumb, orientation toggles vertical/horizontal/both (117-121), `rounded-full` thumb (after P0 fix), id_salt scroll persistence, `auto_shrink([false,false])` for `size-full`, width/height builders, light/dark render test incl. horizontal, scoped style (no global mutation).

---

### resizable — medium

**P0** — none.

**P1 (behavior/interaction gap)**
- **Nova grip chip is wrong** — nova css:990-992 `.cn-resizable-handle-icon { bg-border h-6 w-1 rounded-lg }` = a 24×4px plain pill (no border, no glyph); the base chip is `h-4 w-3 rounded-xs border bg-border` + `GripVerticalIcon size-2.5` (resizable.tsx:44-45). resizable.rs:487-522 draws the base chip (12×16, border, 3 hand-drawn dots) for every style. Fix: branch on `theme.style == Style::Nova` → chip `Rect::from_center_size(center, (4, 24))` horizontal / `(24, 4)` vertical, `CornerRadius::same(2)` (rounded-lg = 10px clamps to half the 4px width ≈ 2; use 2), fill `p.border`, no stroke, no dots. Base arm: keep the chip but replace the 3-dot glyph with `Icon::GripVertical` painted at 10px (`Icon::GripVertical` now exists in the icon set — icons.rs:72 — the PROGRESS.md note "no Icon::GripVertical in foundation" is stale), rotated 90° for vertical groups like `[&[aria-orientation=horizontal]>div]:rotate-90` (the chip orientation swap already handles this).

**P2 (polish)**
- **Hit area slightly wider than source** — resizable.rs:470-474 expands 3px per side (7px total); the source `after:w-1` = 4px total (±1.5px around the 1px line, resizable.tsx:38). Fix: `expand2(vec2(1.5, 0.0))` horizontal / `vec2(0.0, 1.5)` vertical; keep ≥4px total for usability.
- **Arrow-key step 5%** — react-resizable-panels' default keyboard increment is not verified against 5% (resizable.rs:552); keep but flag in docs as an approximation. Home/End keys (collapse/expand) unimplemented — add `Key::Home → 0`, `Key::End → remaining` when focused if desired.
- **No collapsible panels / pixel min-max** — the source panel API supports `collapsible`, `defaultSize`, `minSize` in px; Rust supports fractions only (326-354). Document as a scoped port.
- Grip dot rendering `dot_color` = background token (584-588) is reasonable; after switching to `Icon::GripVertical` remove `dot_color` and its test.

**Not portable**
- Nested/`direction` prop of react-resizable-panels (nested groups would need stacked group stacks; current single-level stack `group_stack_id` actually supports nesting via push/pop — verify nested demo before claiming).
- Drag-to-collapse edge zones.

**Confirmed OK** — 1px `bg-border` handle line (461-467, 483-485), orientation layouts h/v with `aria-[orientation]` equivalents (295-306, 455), per-group persisted fractions with equal-split fallback (270-319), panel min/max clamping both directions (182-214), pointer drag with resize cursors (475-481), keyboard resize when focused with repaint (549-578), focus ring `ring-1 ring-ring ring-offset-1` ≈ paint_ring(expand 1, width 1, p.ring) (524-533), handle count/limits learned across frames, light/dark render + drag + key tests.

---

## Batch summary

All nine components compile-tested with light/dark render tests and their demos are broad, but **none of them can be called visually finished until four concrete regressions are fixed**, in this order of effort:

1. **scroll-area** (P0, ~20 lines): inverted track/handle colors from a wrong egui-0.37 assumption — the most visible defect in the batch (opaque border-colored track where the design has none).
2. **card** (P0 + 2×P1): vertical footer layout breaks the flagship demo; CardAction flush with the card edge; one-line title-size fix for base styles; optional CardSize::Sm.
3. **table** (P0 + P1): 36px→40px header height (one constant + test), and dead row clicks (demo promises interaction that Sense::hover can't deliver).
4. **item** (P1): missing interactive path (hover bg + focus ring) — API addition, moderate.
5. **empty** (P1): mb-2 on the wrong side — two-line fix; plus decide the fate of the invented EmptyFooter.
6. **resizable** (P1): nova grip pill vs base grip glyph — needs Icon::GripVertical integration and a style branch.
7. **avatar** (P2 only): badge ring 1px→2px.
8. **alert** (P2 only): nova-gate the pr-18 reservation; otherwise faithful.
9. **progress** (P2 only): NaN clamp nit; otherwise the closest to parity in the batch.

Batch-wide follow-ups: (i) per-style recipes for Maia..Rhea are approximated by the base recipe (P2, document or extend the match arms); (ii) `TextFormat::line_height` (egui 0.37) should be wired into the galley-painted titles/descriptions listed under cross-cutting (c); (iii) font-weight fidelity (cross-cutting b) stays not-portable unless a medium/semibold face is bundled; (iv) PROGRESS.md rows 29-37 are accurate except row 37's stale "no Icon::GripVertical" note and the missing h-10/footer-layout/scrollbar-color caveats on rows 33, 29, 36.
