# Batch D — Disclosure: gap analysis (accordion, collapsible, tabs, breadcrumb, pagination)

Reference: `original-shadcn-ui/apps/v4/registry/new-york-v4/ui/<name>.tsx` (base) + `apps/v4/registry/styles/style-nova.css` (nova recipe overrides) + `apps/v4/registry/new-york-v4/examples/*-demo.tsx`. Rust: `src/components/<name>.rs`, demos in `examples/gallery/demos/mod.rs`. Timing token source: `apps/v4/public/r/styles/new-york/registry.json` (`accordion-down/up: 0.2s ease-out`).

---

### accordion — medium
Files: `src/components/accordion.rs`, demo `examples/gallery/demos/mod.rs::accordion_demo` (L2149–2246).

**P0 (visual/API wrong — must fix)**
- None. Structure (Item border-b, trigger layout, chevron, animated content) matches; nova values used (`rounded-lg`, `py-2.5` height, content `pb-2.5`).

**P1 (behavior/interaction gap)**
- **`collapsible` defaults to `true`; Radix default is `false`.** `accordion.rs:203-208` (`Accordion::new()` sets `collapsible: true`); Radix `Accordion.Root` `collapsible` defaults to false for `type="single"`. Fix: change the default in `Accordion::new()` to `collapsible: false`, update the doc comment on `collapsible()` ("Defaults to `false`, like Radix; pass `.collapsible(true)` for the docs-demo behavior"), and add `.collapsible(true)` to the single accordion in `accordion_demo` (mod.rs:2157) plus to `triggers_ui` in tests (accordion.rs:667) so the toggle tests keep passing.
- **Arrow-key navigation between triggers not implemented (Radix roving focus).** Original Radix behavior: ArrowDown/ArrowRight → next trigger, ArrowUp/ArrowLeft → previous, Home → first, End → last. Fix in `AccordionTrigger::show` (accordion.rs:431–530): when `response.has_focus() && !self.disabled`, read `ui.input(|i| i.key_pressed(...))` for `Key::ArrowDown | Key::ArrowRight`, `Key::ArrowUp | Key::ArrowLeft`, `Key::Home`, `Key::End`; compute the sibling trigger id `scope.state_id.with("trigger").with(next_index)` (use `ui.ctx().data_mut` read of `root.state_id.with("item_total")` for bounds, skip nothing — disabled triggers still receive focus like Radix) and `ui.memory_mut(|m| m.request_focus(target_id))`. egui keeps Tab for overall traversal; Radix does not reserve Tab in accordions.
- **Open/close animation timing 150ms open / 75ms close vs source 200ms ease-out both ways.** `accordion.rs:442` uses `animation::open_progress` (`src/utils/animation.rs:31-39`, Duration::Default=150ms open, Fast=75ms close); source is `accordion-down 0.2s ease-out` / `accordion-up 0.2s ease-out` (registry.json:83-84) and the chevron rotation is `transition-transform duration-200` (accordion.tsx:43). Fix: add `Duration::Md` = 0.2s to `src/utils/animation.rs` (or an `animation::open_progress_secs(ctx, id, open, 0.2, 0.2)` helper) and use it at accordion.rs:442 for both the chevron rotation progress and (via the same constant) the content height — `animated_disclosure` (collapsible.rs:375-411) needs an optional duration parameter; keep collapsible at its current timing. Closing at 75ms is the most visible deviation.

**P2 (polish)**
- **Trigger row 38px; should be 40px.** `accordion.rs:58` `ACCORDION_TRIGGER_HEIGHT = 38.0` assumes a ~18px text line; Tailwind `text-sm` line-height is 20px, so nova `py-2.5` (10+10) + 20 = 40px. Fix: `pub const ACCORDION_TRIGGER_HEIGHT: f32 = 40.0;` and update the two assertions using `3.0 * ACCORDION_TRIGGER_HEIGHT` (accordion.rs:866, 889) — they still hold.
- **Chevron double-offset.** `accordion.rs:523` places the chevron at `center_y + 2.0`; in CSS (`items-start` + `translate-y-0.5`, accordion.tsx:37,43) both the single-line text box and the translated icon share the same center (row center for 40px). Fix: drop the `+ 2.0` (use `center_y`) once the row is 40px.
- **Hover underline sits low.** `accordion.rs:515` underline y = `center_y + SM*0.62 + 2.0` ≈ center+10.7; CSS underline renders ~3px under the baseline, ≈ center+6 (baseline of a 14px font in a 20px line box ≈ center+2.6, underline-offset ≈ +3). Fix: `let y = center_y + crate::theme::tokens::type_scale::SM * 0.5 + 1.5;` (≈ center+8.5, between descender and CSS look) — or center+7 for a tighter match.
- **`last:border-b-0` uses the previous frame's item count.** `accordion.rs:345-350`: on the first frame after the item count changes, the last item can wrongly show/miss its border for one frame. Acceptable; optionally seed `item_total` from `content` closure item counters using a two-pass pass-through as done by `item.rs`.
- **Demo spacing between content paragraphs.** The source demo content is `flex flex-col gap-4` (accordion-demo.tsx). Demo (mod.rs:2164-2171) uses consecutive `ui.label`s at the theme's 8px spacing. Fix: insert `ui.add_space(8.0)` between the two paragraphs in the "Product Information" content.

**Not portable**
- Radix's height-driven `--radix-accordion-content-height` keyframes are replaced by a measured-height clip (collapsible.rs:375-411) — same visual result, fine in egui.
- String item `value` ids (Radix `value=`/`defaultValue="item-1"`) are replaced by positional indices — acceptable API mapping, documented.

**Confirmed OK**
- Item bottom border 1px `p.border`, skipped on the last item (accordion.tsx:21 ↔ accordion.rs:376-389).
- Trigger: `text-sm font-medium` (FontId proportional SM), full-width row, chevron `size-4` (16px) `text-muted-foreground` with 2px-scaled lucide stroke, rotate 180° animated (`[&[data-state=open]>svg]:rotate-180`).
- Hover underline, disabled opacity 0.5 on text+icon with no hover/click (`disabled:pointer-events-none disabled:opacity-50`), focus ring 3px `ring/50` at nova `rounded-lg` radius via `utils::focus::paint_ring`.
- Content `pt-0 pb-2.5` (nova), `text-sm` region, closed content lays out nothing.
- single/multiple/collapsible toggle logic correct and unit-tested; controlled write-back round-trips; orphan renders don't panic.
- Demo covers single+collapsible (default_open), multiple (controlled flags), disabled trigger, non-collapsible, keyboard hint.

---

### collapsible — high
Files: `src/components/collapsible.rs`, demo mod.rs::collapsible_demo (L2544–2620).

**P0 (visual/API wrong — must fix)**
- None. The source component is an unstyled Radix wrapper; the ghost-button trigger (32px, px-12, ghost hover `accent`, dark `accent/50`, 3px ring/50) is a faithful stand-in for the official demo's `Button variant="ghost" size="sm|icon"` trigger, and the demo's bordered rows (`rounded-md border px-4 py-2 font-mono text-sm`) are reproduced with `BorderRow::draw`.

**P1 (behavior/interaction gap)**
- None. Click toggling, controlled `&mut bool` round-trip, per-`id_salt` persistence, disabled trigger, and animated height are all implemented and unit-tested.

**P2 (polish)**
- **Trigger focus ring radius uses `md`** (collapsible.rs:269, `theme.radius.md_u8()`), matching `rounded-md` ghost buttons — correct today, but it hardcodes the radius instead of deriving from Button size; fine to leave.
- **Content animation 150ms/75ms is an embellishment** — the base `collapsible.tsx` (33 lines) has no animation classes at all; Radix only unmounts/hides. Keep, but the doc claim "CSS animate-accordion-down/up-style easing" (collapsible.rs:11) is inaccurate: soften to "matches the accordion helper's timing for consistency".
- **Demo does not show an icon-only trigger** like the official demo (`Button variant="ghost" size="icon" size-8` with `ChevronsUpDown`, collapsible-demo.tsx). Optional: add a variant with `CollapsibleTrigger::new("").icon_start(Icon::ChevronsUpDown)` (32px square needs the trigger to size as icon when label empty — currently an empty label still adds `pad_x*2` width; if adding this, also collapse `pad_x` to 0 when there is no label and exactly one icon).

**Not portable**
- `asChild` composition (`CollapsibleTrigger asChild` wrapping arbitrary elements) — replaced by the built-in ghost styling + `icon_start`/`icon_end`; idiomatic for egui.

**Confirmed OK**
- Root scope stack lets any widget between Trigger and Content read the open state; orphan Trigger outside a root is inert (tested).
- Content measures, clips and reserves animated height; fully closed reserves zero layout (tested, collapsible.rs:571-597).
- Demo covers uncontrolled, icon trigger open by default, controlled (with live readout), disabled trigger, keyboard hint.

---

### tabs — medium
Files: `src/components/tabs.rs`, demo mod.rs::tabs_demo (L2248–2361).

**P0 (visual/API wrong — must fix)**
- **`line` variant horizontal indicator floats ~8px below the list.** Source (tabs.tsx:69): `after:bottom-[-5px] after:h-0.5` relative to the trigger → bar occupies `[trigger.max.y + 3, trigger.max.y + 5]`. Rust `tabs.rs:584-590` computes `y = rect.max.y + SM_HALF(10) - INDICATOR(2) + LIST_PAD(3)` = `rect.max.y + 11` → bar at `[+11, +13]`, detached below the 32px list (list bottom = trigger.max.y + 4). Fix: `let y = rect.max.y + 5.0 - INDICATOR;` (bar top = max.y+3, bottom = max.y+5), keep `Rect::from_min_size(Pos2::new(rect.min.x, y), Vec2::new(rect.width(), INDICATOR))`.
- **List gap between triggers is 6px in both variants; source has none (default) and 4px (`line`).** Source (tabs.tsx:28,33): base list has no gap (triggers are `flex-1` and touch); `line` adds `gap-1` (4px). Rust `tabs.rs:296` sets `item_spacing.x = XS_HALF` (6px) unconditionally. Fix: `let gap = match self.variant { TabsVariant::Default => 0.0, TabsVariant::Line => crate::theme::tokens::spacing::XS (4.0) };` and set `item_spacing.x` (horizontal) / `.y` (vertical) to that.

**P1 (behavior/interaction gap)**
- **No arrow-key navigation / automatic activation (Radix roving tabindex).** Radix default `activationMode="automatic"`: ArrowLeft/ArrowRight (horizontal) or ArrowUp/ArrowDown (vertical) move focus AND select the next enabled trigger; Home/End jump. Fix in `TabsTrigger::show` (tabs.rs:395-607): when `response.has_focus() && !self.disabled`, on `Key::ArrowRight|Key::ArrowDown` (or Left/Up) request focus on the sibling id `scope.value_id.with("trigger").with(next_value)`; because focusing needs the sibling's value id, keep a per-frame ordered list of trigger `(value, disabled)` pairs in ctx data under `scope.value_id.with("trigger_order")` (each trigger pushes its value during `show`, the list resets at `TabsList::show` start like the existing `scratch_id` union). On focus change, also `write_selected(...)` (automatic activation). Home/End to first/last enabled trigger.
- **Dark list fill is an invented `muted` at 60% alpha.** `tabs.rs:323-327`: `dark → color_alpha(muted, 0.6)`; source is plain `bg-muted` (tabs.tsx:32) in both modes and the dark `--muted` token (oklch 0.269) already encodes the dark appearance — it is faithfully ported in `theme_data.rs:46`. Fix: use `ShadcnTheme::of_ui(ui).palette.muted` for both modes.

**P2 (polish)**
- **Triggers don't stretch equally (`flex-1`) in horizontal lists.** Source triggers share equal width; Rust triggers are content-sized (tabs.rs:432-439), so "Account"/"Password" differ ~10px and the muted pill hugs content. Fix (two-pass): in `TabsList::show`, measure all trigger widths in pass 1 (they already push into ctx via the union scratch), store the max width in ctx (`scratch_id.with("equal_w")`), and in `TabsTrigger::show` allocate `Vec2::new(equal_w.unwrap_or(inner_w + pad_x*2), height)` in horizontal orientation only.
- **Vertical triggers not full width (`w-full`)** — same mechanism as above applied to vertical (stretch to the widest sibling's width).
- **Vertical trigger height 22px; nova is 24px.** `tabs.rs:70` `TRIGGER_H_VERTICAL = 22.0` assumes base `py-1`; nova trigger is `px-1.5 py-0.5` (style-nova.css:1325) → 2+20+2 = 24px. Fix: `const TRIGGER_H_VERTICAL: f32 = 24.0;`.
- **Vertical `line` indicator 2px too far right.** Source `after:-right-1 after:w-0.5` (tabs.tsx:69) → bar spans `[trigger.max.x + 2, trigger.max.x + 4]`. Rust `tabs.rs:578` places it at `rect.max.x + XS(4)` → `[+4, +6]`. Fix: `Pos2::new(rect.max.x + 4.0 - INDICATOR, rect.min.y)`.
- **Demo missing the `line` + vertical combination** and an icon in the `line` variant; add one `TabsList::variant(Line)` inside the vertical section.

**Not portable**
- `flex-1` is approximated by the two-pass stretch above; exact CSS flex behavior doesn't exist in egui.
- Focus-visible `outline-1 outline-ring` in addition to the 3px ring is dropped (ring already conveys focus).

**Confirmed OK**
- Root `flex gap-2` (theme item_spacing 8px vertical for horizontal orientation; explicit 8px horizontal for vertical, tabs.rs:212).
- List: nova `rounded-lg p-[3px] h-8` (32px, L64-68), `rounded-none` + transparent for `line`, bg painted behind triggers from their union + 3px.
- Trigger: nova `px-1.5 py-0.5`, `gap-1.5` icon gap, `text-sm font-medium`, inactive `text-foreground/60` light / `dark:text-muted-foreground`, hover/active `text-foreground`, active default-variant `bg-background shadow-sm` (light, real shadow via `utils::shadow::shadow_sm`) and `dark:bg-input/30` + `border-input` stroke, disabled opacity 0.5 and no interaction, 3px `ring/50` focus ring at `rounded-md`.
- First shown trigger becomes active (Radix `defaultValue` behavior), controlled `&mut String` round-trips, content only renders when selected, disabled trigger can't be selected or clicked (tested).
- Demo covers default, line, controlled (live readout), vertical, icons, disabled trigger, keyboard hint.

---

### breadcrumb — high
Files: `src/components/breadcrumb.rs`, demo mod.rs::breadcrumb_demo (L2363–2427).

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- None. Link hover brighten (`hover:text-foreground`), static page, 14px chevron separators, ellipsis slot, click reporting — all present. (Keyboard Enter/Space activation of a focused link works via egui's synthesized clicks for `Sense::click` widgets, same mechanism the accordion trigger tests rely on.)

**P2 (polish)**
- **List doesn't wrap (`flex-wrap`).** Source `breadcrumb.tsx:15` has `flex-wrap break-words`; `BreadcrumbList::show` (breadcrumb.rs:90-96) uses `ui.horizontal` (no wrap). Fix: replace with `ui.horizontal_wrapped(|ui| { ui.spacing_mut().item_spacing.x = self.gap; ... })`.
- **Row height 18.2px; CSS line-height is 20px.** `breadcrumb.rs:161,226` `line_h = TEXT_SIZE * 1.3` (=18.2) for Link/Page. Fix: use a `const LINE_HEIGHT: f32 = 20.0;` (Tailwind `text-sm` line-height) in both `BreadcrumbLink::show` and `BreadcrumbPage::show`.
- **`text-sm` weight/size fine, but `BreadcrumbEllipsis` icon box is 20px while the glyph is 16px** — correct per nova (`size-5 [&>svg]:size-4`, style-nova.css:145). No change.
- **Demo could show a custom separator** (source accepts arbitrary children, breadcrumb.tsx:64-79; Rust supports `.icon(Icon::...)`). Optional: add one trail using `BreadcrumbSeparator::new().icon(Icon::Slash)`-style custom icon to document the builder.

**Not portable**
- `aria-label="breadcrumb"` on nav, `aria-current="page"`/`aria-disabled` on page — semantic HTML attributes without egui equivalent; the page being non-interactive (`Sense::hover`) conveys the same affordance.

**Confirmed OK**
- nova gaps: list `gap-1.5` (6px, breadcrumb.rs:79), item `gap-1` (4px, breadcrumb.rs:117).
- Link muted → foreground on hover, focus ring 3px ring/50, `id_salt` support.
- Page `font-normal text-foreground`, non-interactive.
- Separator `ChevronRight` 14px muted with `2.0*size/24` lucide stroke scaling; ellipsis `MoreHorizontal` 16px in nova 20px box.
- Demo covers default trail, ellipsis trail, hover behavior note, clickable-link counter.

---

### pagination — medium
Files: `src/components/pagination.rs`, demo mod.rs::pagination_demo (L2429–2542).

**P0 (visual/API wrong — must fix)**
- None (all deviations are sizing/color-level).

**P1 (behavior/interaction gap)**
- **Previous/Next button metrics don't match the source's overrides.** Source: `PaginationPrevious` = ghost Button `size="default"` + `gap-1 px-2.5 sm:pl-2.5` (pagination.tsx:76), nova adds `pl-1.5!` (style-nova.css:918); nova Button default size is `h-8 gap-1.5 px-2.5` (style-nova.css:185-186). So nova prev = h-8 (32px), left pad 6px, right pad 10px, icon-label gap 4px. Rust (pagination.rs:234-245) renders `ButtonSize::Default` → h-9 (36px), pad_x 16px, gap 8px (button.rs:76,105,115). Fix: since Button's metrics are fixed per size, add chainable overrides to `Button` (`pub fn padding_x(mut self, px: f32)`, `pub fn gap(mut self, px: f32)`, `pub fn height(...)` — coordinate with the batch-A owner of `src/components/button.rs`) and render Previous as `.padding_x(10.0).gap(4.0)` with a `pl_override(6.0)` for nova (or simply `padding_x(6.0)` asymmetric variant `padding_left/padding_right`); base-recipe fallback values: h-9, pl 10px, pr 10px, gap 4px. Next mirrors (pr override, nova `pr-1.5!` → base keeps `px-2.5` right 10px, pagination.tsx:93).
- **PaginationLink icon-size square is 36px; nova is 32px (`size-8`).** Source base `size="icon"` = `size-9` (36px) but nova `.cn-button-size-icon { size-8 }` (style-nova.css:201-203). Rust `PaginationLink::new` uses `ButtonSize::Icon` → 36px (pagination.rs:140, button.rs:76). Fix path A (correct, cross-batch): make `ButtonSize::height()` style-aware (nova → Icon 32, Default 32 h-8, Sm 28 h-7, Lg 36 h-9 per style-nova.css:177-207) — that fixes every Button consumer at once; path B (local): `PaginationLink::show` maps `ButtonSize::Icon` to a 32px square if the theme style is nova. Recommend A, tracked with button.rs owner.

**P2 (polish)**
- **Ellipsis icon color is muted; source inherits foreground.** Source ellipsis span (pagination.tsx:102-117) has no text color class and the nav/content ancestors set none, so `MoreHorizontalIcon` renders in `text-foreground`. Rust `pagination.rs:340` uses `p.muted_foreground`. Fix: use `theme.palette.foreground`. (Breadcrumb's ellipsis/separator stay muted — their `<ol>` sets `text-muted-foreground`; breadcrumb.rs is correct.)
- **Row not centered (`mx-auto w-full justify-center`).** `pagination.tsx:17` centers the content; `Pagination::show` (pagination.rs:58-60) is a passthrough so the row is left-aligned. Fix: `Pagination::show` wraps content in `ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| content(ui))` — child horizontal rows are then centered.
- **Nova `gap-0.5` (2px) is used but with 36px buttons the row reads denser than source base `gap-1`** — after the P1 size fix (32px squares), keep nova `HALF` (2px); no further change.
- **Demo `ButtonSize::Sm` page label ("100")** exercises the size builder — good; optionally also show a wide `PaginationLink::new("…").size(ButtonSize::Default)` to document non-icon links.

**Not portable**
- `<nav role="navigation" aria-label="pagination">`, `aria-current="page"` — no a11y tree in egui; the Outline-variant active link conveys state visually.
- Real `<a href>` links — replaced by Response::clicked; callers navigate.

**Confirmed OK**
- Content row gap nova `gap-0.5` (2px, pagination.rs:77); Item pure passthrough.
- Active link = Outline variant, rest Ghost via `resolved_variant` mirroring `buttonVariants({variant: isActive ? "outline" : "ghost"})` (pagination.tsx:57-60) — tested.
- Default link `size="icon"`; disabled Previous/Next at the ends; ellipsis nova `size-8` (32px) with 16px glyph centered.
- Demo covers interactive paging (prev/next/ellipsis/wide last page), disabled states at first and last page, and documents the variant mapping.

---

## Batch summary

All five components are structurally faithful and demoed; the gaps are concentrated in fine-grained metrics, the tabs `line` indicator geometry, and Radix keyboard semantics.

Ranking by work needed (most first):
1. **tabs (medium)** — 2 P0s (mis-positioned `line` underline 8px too low; wrong inter-trigger gaps: 0/4px vs 6px), 1 P1 (invented dark `muted/60` fill), 1 P1 keyboard (arrow navigation + automatic activation), plus flex-1 stretch polish. Biggest but well-bounded.
2. **pagination (medium)** — 2 P1s that both hinge on Button metrics (nova sizes: 32px icon squares, compact Previous/Next paddings/gap); the clean fix is style-aware `ButtonSize` in `button.rs` (cross-batch coordination), plus two small P2s (ellipsis foreground color, centered row).
3. **accordion (medium)** — 1 P1 API default (`collapsible` should default false like Radix), 1 P1 keyboard (arrow-key traversal), 1 P1 animation timing (200ms both ways vs 150/75ms), plus 40px trigger row and small offsets.
4. **breadcrumb (high)** — polish only: wrap support, 20px line height, optional custom-separator demo item.
5. **collapsible (high)** — nothing to fix; optionally soften one doc claim and add an icon-only trigger demo variant.

Shared cross-batch note for the fix agents: the 200ms disclosure timing and the arrow-key roving-focus pattern are needed by both accordion and tabs — implement once in `src/utils/animation.rs` (new `Duration::Md`/explicit-duration helper) and as a small helper for reading ordered sibling trigger ids in ctx data (the `scratch` pattern already used by `TabsList`).

`PROGRESS.md` rows 38–42 are accurate as written (their behavior notes are still true); after fixes, append the arrow-key and timing notes and fill the empty Commit column for rows 38–42 (other batches have hashes there).
