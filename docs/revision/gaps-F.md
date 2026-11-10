# Batch F — Menus & selection: gap analysis (dropdown-menu, context-menu, menubar, navigation-menu, select, combobox, command)

References: `original-shadcn-ui/apps/v4/registry/new-york-v4/ui/<name>.tsx` (base), `apps/v4/registry/styles/style-nova.css` (nova `.cn-*` recipe), `packages/registry/test/fixtures/transform-corpus/radix-nova--context-menu.tsx` (canonical nova merge for the shared menu rows), `apps/v4/registry/new-york-v4/ui/combobox.tsx` (Base UI combobox). Rust: `src/components/<name>.rs`, demos in `examples/gallery/demos/mod.rs` (lines ~2723–3410).

Cross-cutting notes affecting this batch:
- **Popup animation timing.** All popups use `utils/animation::overlay_progress` = 150ms open / 75ms close; the nova menu recipes all say `duration-100`. Shared fix (matches batch E note): add `overlay_progress_secs(ctx, id, open, secs)` in `src/utils/animation.rs` and pass 0.1 from dropdown-menu, context-menu, menubar, select, combobox, command (dialog), navigation-menu. The `(1.0 - alpha) * 8.0` gap inflation already reproduces `slide-in-from-top-2` — keep it.
- **`tracking-widest` shortcuts.** `DropdownMenuShortcut` / `CommandShortcut` / menubar shortcut are `text-xs tracking-widest text-muted-foreground` (letter-spacing 0.1em). egui has no per-glyph spacing via `painter.text`; fixable only by laying out a galley with extra advance or accepting normal tracking. Note once here; do not repeat below.
- **`font-medium`.** Menu labels (`DropdownMenuLabel`, menubar triggers/labels, command group headings, nav triggers, chip text) are `font-medium`; `.strong()` only brightens (no medium face). Accepted limitation, same as batch E.
- **Nova radio-item indicator is a check, not a dot.** The canonical nova merge `radix-nova--context-menu.tsx:196-218` renders the `ContextMenuRadioItem` indicator with `CheckIcon` (16px, `[&_svg]:size-4`) — the transform replaces base `CircleIcon size-2 fill-current` for nova. Applies to dropdown/context/menubar radio items.
- Keyboard typeahead (Radix menus/select jump to matching items as you type) is missing everywhere in this batch; noted once, not repeated per component.

---

### dropdown-menu — high
Files: `src/components/dropdown_menu.rs`, demo `demos/mod.rs::dropdown_menu_demo` (mod.rs:2723).

**P0 (visual/API wrong — must fix)**
- None. Nova content (`min-w-32 rounded-lg p-1 shadow-md ring-1 ring-foreground/10`, dropdown_menu.rs:765-774), nova item rows (`rounded-md px-1.5 py-1` → 28px rows, `data-inset:pl-7` → 28px, dropdown_menu.rs:75-91,1058-1061), right-side indicators (`pr-8`, check at `right-2`, dot radius 4.0, dropdown_menu.rs:1112-1145), labels `text-xs text-muted-foreground` (dropdown_menu.rs:1147-1174), destructive `text-destructive` + `focus:bg-destructive/10` (dark `/20`, dropdown_menu.rs:692-706), disabled `opacity-50`, shortcut accent-on-highlight all match nova.

**P1 (behavior/interaction gap)**
- **No max-height clamp/scroll on the content.** dropdown-menu.tsx:44 has `max-h-(--radix-dropdown-menu-content-available-height) overflow-y-auto`; `render_level` (dropdown_menu.rs:776-795) grows unbounded — a long menu can overflow the window with no scroll. Fix: inside the popup `frame.show` body, wrap `render_rows` in `crate::components::scroll_area::ScrollArea::new().max_height(ctx.screen_rect().height() - anchor_rect.bottom() - gap - 16.0)` (mirrors select.rs's `MAX_CONTENT_HEIGHT` pattern but viewport-derived); keep `ui.set_min_width`.
- **No typeahead.** Radix DropdownMenu moves highlight to the first item starting with the typed character (with ~1s buffer). Fix: in `handle_menu_keys` (dropdown_menu.rs:1253), collect `ctx.input(|i| i.events)` text characters while open, buffer them in `MenuState` (add `typeahead: String` + `typeahead_at: Option<f64>`), and on each printable char search `entries` (labels, case-insensitive, wrapping from current highlight + 1) via a helper like `select::action_index_of_value`; clear the buffer after 1.0s (`ctx.input(|i| i.time)`).

**P2 (polish)**
- **Sub-menu popups inherit the root width.** `SUB_MENU_MIN_WIDTH` (96.0, dropdown_menu.rs:73) is never used — the sub `render_level` call (dropdown_menu.rs:959-972) reuses `shared.spec.min_width`, so a 224px root yields 224px sub-menus. Nova sub-content is `min-w-[96px]` (style-nova.css:577-579). Fix: add `min_width_override: Option<f32>` to `LevelShared` (or a parameter on `render_level`); in the sub branch pass `Some(96.0)` and use `min_width.unwrap_or(shared.spec.min_width)` for the Popup `.width()` + `set_min_width`.
- **Animation duration 150ms vs nova `duration-100`** (style-nova.css:538). Fix via shared `overlay_progress_secs(..., 0.1)`.
- **`zoom-in-95` scale never applied** — `OverlayProgress.scale` computed in overlay_progress (animation.rs:51-56) is unused in `render_level`. Accepted stand-in (fade+slide); do not try to scale layout.
- **Sub-trigger `inset` unreachable via API.** `EntryKind::Sub.inset` exists (dropdown_menu.rs:288) but `MenuBuilder::sub_menu`/`sub_menu_disabled` (dropdown_menu.rs:354-383) hard-code `inset: false`, so `data-inset:pl-7` sub-triggers can't be expressed. Fix: add `pub fn sub_menu_inset(&mut self, label, build)` mirroring `label_inset`.
- **Nova radio indicator should be a 16px `Icon::Check`, not the 8px dot** (see cross-cutting note; base is the dot). Fix: in `paint_indicator` (dropdown_menu.rs:1120-1145), when `radio` paint `Icon::Check` in a 16px rect like the checkbox branch. If base-recipe parity is also wanted, gate on a const in `MenuSpec`.
- **Shortcut `tracking-widest` not applied** (cross-cutting; accepted or implement via galley with letter advance).

**Not portable**
- Radix `side`/`collisionPadding`/`avoidCollisions` fine-tuning — egui `Popup` handles flipping internally with no per-side hooks.
- `origin-(--radix-dropdown-menu-content-transform-origin)` zoom origin — no transform-origin concept; fade+slide stands in.

**Confirmed OK**
- `sideOffset=4` gap (dropdown_menu.rs:1224,1596); `MenuAlign` Start/Center/End → `BOTTOM_START/BOTTOM/BOTTOM_END`; outline-button default trigger + `trigger_widget` (asChild); declarative `MenuBuilder` with items/checkbox/radio/label(inset)/separator/sub-menus; keyboard model (ArrowUp/Down skip disabled + wrap, ArrowRight/Left open/close subs, Home/End, Enter/Space, Escape pops deepest first, dropdown_menu.rs:1253-1356); hover-open subs + hover highlight (dropdown_menu.rs:998-1023); click-outside dismissal incl. sub popups (dropdown_menu.rs:1361-1383); close fade with inert rows during fade; controlled + uncontrolled open; `close_on_select(false)`; separator `-mx-1 my-1 h-px bg-border` full-bleed (dropdown_menu.rs:1177-1190); icons `size-4` muted / destructive-red; extensive tests (tree helpers, keyboard, click, checkbox toggle).
- Demo covers the docs basic list (w-224 align-start), checkboxes incl. disabled, radio group, destructive+icon, all three alignments, controlled open, activation readout.

---

### context-menu — high
Files: `src/components/context_menu.rs`, demo `demos/mod.rs::context_menu_demo` (mod.rs:2865).

**P0 (visual/API wrong — must fix)**
- None. Shares the dropdown renderer; nova `min-w-36` (context_menu.rs:54) and the shared item recipe match `style-nova.css:414-452`.

**P1 (behavior/interaction gap)**
- **No max-height clamp/scroll on the content** — context-menu.tsx:104 `max-h-(--radix-context-menu-content-available-height) overflow-y-auto`; same `render_level` path as dropdown. Fix: identical ScrollArea clamp inside `render_level` (fix once for both).
- **No typeahead** (Radix ContextMenu supports it). Fix: same shared `handle_menu_keys` change as dropdown-menu.

**P2 (polish)**
- **Sub-content border color.** Nova `.cn-context-menu-sub-content` is `border ... shadow-lg` (style-nova.css:454-456) — i.e. the `border` token, while the root content is `ring-foreground/10`. The shared `render_level` always strokes `color_alpha(p.foreground, 0.10)` (dropdown_menu.rs:767). Fix: add a `border_is_ring: bool` (or stroke color) field on `MenuSpec`; set `false` for context-menu subs and stroke `p.border` there. (Dropdown subs stay ring.)
- **Animation 150ms vs `duration-100`** — shared `overlay_progress_secs(..., 0.1)` fix.
- **Radio indicator check-vs-dot** — same shared `paint_indicator` fix as dropdown-menu.
- **Menu can spawn flush at screen edges.** Radix flips/shifts the context menu inside the viewport; egui Popup flips too but the 1×1 anchor at the pointer means near-edge right-clicks can clip. Acceptable; note only.

**Not portable**
- Radix `SubMenuTrigger` pointer-grace timing heuristics (directional re-entry) — egui hover model has no grace area; instant switch is the accepted stand-in.

**Confirmed OK**
- Right-click on `ui.min_rect()` region opens at pointer (`Rect::from_min_size(pos, ONE)` anchor + `BOTTOM_START`, gap 0 — context_menu.rs:141-156,197-213); any-button click outside closes (incl. right-click outside, tested); Escape pops deepest level; hover highlight/open-subs; checkbox/radio side effects; controlled + uncontrolled; shares dropdown keyboard helpers and row recipe; nova `min-w-36` asserted in tests.
- Demo covers basic (shortcuts/icons/disabled/destructive/sub), checkbox+radio (with inset label), controlled.

---

### menubar — medium
Files: `src/components/menubar.rs`, demo `demos/mod.rs::menubar_demo` (mod.rs:2979).

**P0 (visual/API wrong — must fix)**
- **Checkbox/radio label collides with the left indicator.** Nova menubar checkable rows are always `pl-7` with the indicator at `left-1.5` (style-nova.css:818-832: `.cn-menubar-checkbox-item { py-1 pr-1.5 pl-7 }`, `.cn-menubar-checkbox-item-indicator { left-1.5 size-4 }`) — NOT gated on `data-inset`. The shared renderer indents text to 28px only when `item.inset` is set (dropdown_menu.rs:1058-1061), while `IndicatorSide::Left` paints the indicator at `rect.min.x + 6 + 8` → spans 6..22px (dropdown_menu.rs:1132-1135). The menubar demo's View-menu checkbox rows (mod.rs:3029-3035) and Zoom-menu radio rows (mod.rs:3049-3055) do NOT set `.inset(true)`, so the check/dot overlaps the first ~2 characters of the label. Fix: in `render_rows` (dropdown_menu.rs:804-977), for `EntryKind::Checkbox` and `RadioGroup` item rows compute `let row_inset = item.inset || shared.spec.indicator == IndicatorSide::Left;` and pass that to `RowPaint.inset` (right pad stays `MENU_ROW_PAD_X` 6px = `pr-1.5`). After the fix the demo's manual `.inset(true)` on View radio rows becomes redundant but harmless.

**P1 (behavior/interaction gap)**
- **No keyboard access to the bar when closed.** Radix Menubar roves focus across triggers (Tab), opens with Enter/Space or ArrowDown, and ArrowLeft/Right move trigger focus even with nothing open. Rust only switches menus with ArrowLeft/Right while one is open (menubar.rs:254-300) — with nothing open there is no keyboard entry point (triggers are `Sense::click()` interactables, so egui Tab can focus them, but Enter/Space on a focused trigger does toggle via `trigger.clicked()`; verify and, if it works, document). Fix minimally: while closed and a trigger has focus, ArrowRight/Left move focus (`response.request_focus()` on the neighbor trigger) and ArrowDown opens; keep existing open-state switching.
- **No typeahead inside menus** (shared fix).
- **Label row height.** Nova menubar label is `py-1 text-sm` → 28px (style-nova.css:834-836); the shared `paint_label` uses `MENU_LABEL_HEIGHT = 20.0` for both muted-XS and foreground-SM labels (dropdown_menu.rs:1147-1148). Fix: make label height spec-driven: add `label_height: f32` to `MenuSpec` (20.0 for dropdown/context, 28.0 for menubar) or compute from `label_muted`.

**P2 (polish)**
- **Trigger focus background.** Base menubar-trigger keeps `focus:bg-accent focus:text-accent-foreground`; nova adds `hover:bg-muted aria-expanded:bg-muted` (style-nova.css:802-804). `paint_trigger` (menubar.rs:517-558) tints text accent on focus but never paints a focus bg. Fix: `let fill = if open || hovered { p.muted } else if response.has_focus() { p.accent } else { TRANSPARENT }`.
- **Animation 150ms vs `duration-100`** (style-nova.css:806-808) — shared `overlay_progress_secs` fix.
- **Radio indicator check-vs-dot** — shared `paint_indicator` fix.
- **Trigger height 26 vs 24.** Nova trigger `py-[2px]` + text-sm = 24px content centered in the 26px row (bar h-8 p-[3px]); Rust allocates the full 26px (menubar.rs:76,528). Invisible; note only.
- **Sub-popup width** inherits root min-w-36 (same LevelShared fix as dropdown; nova menubar sub-content is `min-w-32`-equivalent? — actually base `min-w-[8rem]`; use 96–128 and match dropdown's fix).

**Not portable**
- Radix `loop` option / roving tabindex plumbing — approximated by egui focus as above.

**Confirmed OK**
- Bar frame `h-8 gap-0.5 rounded-lg border bg-background p-[3px] shadow-xs` (menubar.rs:302-309, constants tested); trigger `px-1.5` + muted hover/open fill + focus ring + focus text; click opens, second click closes (press-capture keeps a hover-switched menu open, menubar.rs:326-345); pointer-enter switches while open (menubar.rs:349-355); ArrowLeft/Right switch open menus with wrap; Escape pops subs then closes with fade; activation closes + side effects; checkbox LEFT indicator + non-muted SM labels via `MenuSpec` (menubar.rs:473-477); `alignOffset=-4`/`sideOffset=8` (menubar.rs:489-504); controlled open index; close fade.
- Demo covers 4 menus with subs, checkbox/radio/inset rows, destructive, shortcuts, controlled bar.

---

### navigation-menu — medium
Files: `src/components/navigation_menu.rs`, demo `demos/mod.rs::navigation_menu_demo` (mod.rs:3335).

**P0 (visual/API wrong — must fix)**
- **Base recipe used, not nova.** Rust trigger/link colors use the accent family and `px-4`/`rounded-md`: `resolve_trigger_colors` paints `hover:bg-accent`, `data-open:bg-accent/50` (navigation_menu.rs:215-240), trigger pad `NAV_TRIGGER_PAD_X = 16.0` "px-4" (navigation_menu.rs:58-59), radius `md_u8()` (navigation_menu.rs:693,768). Nova (style-nova.css:863-869) is `hover:bg-muted focus:bg-muted data-open:bg-muted/50 data-popup-open:bg-muted/50 rounded-lg px-2.5` and links `hover:bg-muted data-active:bg-muted/50 rounded-lg p-2` (in-content `rounded-md`). Fix: in `resolve_trigger_colors` return muted tints (`hover → p.muted`, `open → color_alpha(p.muted, 0.5)`, text stays `p.foreground` — nova does NOT set accent foreground); change `NAV_TRIGGER_PAD_X` to 10.0; use `theme.radius.lg_u8()` for bar items and `lg_u8()` for standalone links / `md_u8()` for in-panel links (`resolve_link_colors`: hover → `p.muted`, active → `color_alpha(p.muted, 0.5)`). Update the two color unit tests (navigation_menu.rs:1073-1099) accordingly.

**P1 (behavior/interaction gap)**
- **Hover-opened menu never closes on pointer-leave.** Radix closes a hover-opened menu once the pointer leaves trigger+viewport (with grace delay). Rust keeps it open until an outside click / Escape / trigger re-click (navigation_menu.rs:484-508, 572-582; module doc states this). Fix: track hover state; if the menu was hover-opened (`state.hover_open`) and neither the open trigger rect, the bar rect, nor any popup rect contains the pointer for ~150ms (use `ctx.input(|i| i.time)` + a `hover_left_at: Option<f64>` in NavState), close it. Pointer re-entry into the viewport cancels the timer.
- **Keyboard model thin.** Radix: Enter/Space open the focused trigger, ArrowDown/Up move between links inside an open panel, Home/End, Tab roves triggers. Rust has only ArrowLeft/Right switching of the open trigger (navigation_menu.rs:511-531); panel links are mouse-only. Fix minimally: while open, ArrowDown/ArrowUp move a link-highlight inside the panel (store `link_highlight: Option<usize>` in NavState; paint accent/muted bg on the highlighted link; Enter clicks it); document the rest as a known gap in PROGRESS.md.

**P2 (polish)**
- **Diamond indicator lacks `rounded-tl-sm shadow-md`.** Nova `.cn-navigation-menu-indicator-arrow { bg-border rounded-tl-sm shadow-md }` (style-nova.css:887-889); Rust paints a plain convex polygon (navigation_menu.rs:880-897). Fix: rotate the polygon points 45° so the top-left corner is a square corner and add `Stroke::NONE` shadow via a slightly offset dark shape, or accept (it's a 8px diamond). Cheapest faithful fix: keep polygon but paint a `shadow_md(dark)`-colored duplicate offset by (0, 2) beneath it.
- **Viewport shadow weight.** Nova viewport is `shadow` (single shadow, style-nova.css:879-881), Rust uses `shadow_md` (navigation_menu.rs:904). Fix: add/use `shadow::shadow(dark)` (between sm and md) or reuse `shadow_sm`.
- **Chevron rotation animation.** `duration-300 group-data-open:rotate-180` — Rust swaps ChevronDown/Up instantly (navigation_menu.rs:712-717). Fixable with `ctx.animate_value_with_time` driving a rotation angle passed to `Icon::paint` if the icon painter supports rotation; otherwise accept.
- **Viewport width/height transition between panels** (Radix animates the shared viewport when switching triggers). Rust re-renders per trigger; animation id is shared (`viewport_id`) so the fade replays — acceptable; note only.
- **Panel content padding `pr-2.5`.** Base content is `p-2 pr-2.5`; Rust uses 8.0 both sides (navigation_menu.rs:64-65). Fix: right margin `Margin { left: 8, right: 10, top: 8, bottom: 8 }` in the two frame builders.
- **Animation timing** — nova viewport `duration-100`, float panels `duration-300`; shared helper + per-component secs.

**Not portable**
- `--radix-navigation-menu-viewport-width/height` measured transitions between arbitrary panel sizes — egui popup sizes to content; fade stands in.

**Confirmed OK**
- Trigger `h-9` + text-sm + chevron `size-3 ml-1` swapped/rotated-open representation; bar gap 1 (4px); viewport below bar at `mt-1.5` (6px) with `bg-popover` ring-foreground/10 stroke + lg radius (navigation_menu.rs:899-904); diamond under the open trigger center-x; `viewport(false)` inline panels + vertical orientation (RIGHT_START); hover opens/switches while click opens/toggles with the hover-adopt-on-click nuance; Escape/outside-click close; disabled trigger skips hover/toggle; links with wrapped muted descriptions; panel width measured from content, clamped 200–400; Escape respects modal layers; unit tests for trigger indices, colors, click/hover/escape/arrow flows.
- Demo covers horizontal viewport, inline panels, vertical, disabled trigger, description links.

---

### select — medium
Files: `src/components/select.rs`, demo `demos/mod.rs::select_demo` (mod.rs:3104).

**P0 (visual/API wrong — must fix)**
- **Base recipe, not nova.** Rust implements the base trigger/content/item recipe and its own doc comment says so (select.rs:3-13). Nova (style-nova.css:1003-1053) differs on every measurable value:
  - Trigger: nova `h-8` default (32px) and `h-7` sm (28px) — Rust `CONTROL`=36 / `CONTROL_SM`=32 (select.rs:344-361); nova `rounded-lg` (sm: `rounded-[min(var(--radius-md),10px)]`) — Rust `md_u8()` (select.rs:642); nova `pl-2.5 pr-2` (10/8px) — Rust `px-3`/`MD`=12 both sides (select.rs:668-676); nova `gap-1.5` between value and icon — Rust 8px-ish implicit.
  - Chevron: nova `cn-select-trigger-icon { text-muted-foreground size-4 }` — full-opacity muted; Rust `color_alpha(p.muted_foreground, 0.5)` (select.rs:335).
  - Content: nova `min-w-36` (144) — Rust `SELECT_MIN_WIDTH` 128 (select.rs:64,866); nova `rounded-lg` — Rust `md_u8()` (select.rs:870); nova ring `ring-foreground/10` ✓ already (select.rs:869).
  - Item: nova `rounded-md py-1 pl-1.5 pr-8` → 28px rows, text pad 6px — Rust `ITEM_HEIGHT` 32.0 / `ITEM_PAD_X` 8.0 (select.rs:68-69,961,995); nova item radius `rounded-md` — Rust `sm_u8()` (select.rs:989).
  - Label: nova `px-1.5 py-1` → 20px — Rust `LABEL_HEIGHT` 24 / pad 8 (select.rs:74,1018-1029).
  Fix: introduce a small const block mirroring the menu components (`SELECT_TRIGGER_HEIGHT_DEFAULT = 32.0`, `SELECT_TRIGGER_HEIGHT_SM = 28.0`, `SELECT_MIN_WIDTH = 144.0`, `ITEM_HEIGHT = 28.0`, `ITEM_PAD_X = 6.0`, `LABEL_HEIGHT = 20.0`, trigger radius `lg_u8()`, item/label radius `md_u8()`), chevron full `p.muted_foreground` (disabled keeps 0.5 alpha), text x = 10.0, chevron x = 8.0 from right. Update geometry tests.
- **Trigger focus-ring radius/size on invalid (nova)**: nova adds `dark:aria-invalid:border-destructive/50` — dark invalid border is 50% destructive, Rust uses full `p.destructive` in dark too (select.rs:313-318). Fix: `if dark { color_alpha(p.destructive, 0.5) }`.

**P1 (behavior/interaction gap)**
- **No typeahead** — Radix Select jumps to options as you type while open AND while closed (focused trigger). Already flagged in PROGRESS.md. Fix: same buffered-char search over `entries` values/labels as the menu fix; while closed it should also select the matched option directly (Radix updates the value live on the trigger).
- **Scroll buttons.** Source renders `SelectScrollUpButton`/`DownButton` rows (`py-1` + chevron, `bg-popover z-10`) at the list edges; Rust relies on the native ScrollArea (select.rs:922-926). egui has no press-and-hold auto-scroll affordance; a faithful stand-in is two 28px header/footer rows with ChevronUp/Down that call `ScrollArea::scroll_to_*` while hovered — or accept and note in PROGRESS.md (currently unnoted).
- **Tab while open.** Radix closes on Tab; Rust leaves the popup open when focus moves (select.rs:761-799 handles no Tab). Fix: add `Key::Tab` to the open-popup key handling → `state.open = false`.

**P2 (polish)**
- **`position="item-aligned"` default.** The source `SelectContent` defaults to `position="item-aligned"` (select.tsx:55) — the popup overlays the trigger aligned so the selected item sits over the trigger; the docs demos use popper. Rust always anchors `BOTTOM_START` (select.rs:882). Accept (popper is what the docs show), or add an `item_aligned(bool)` option later.
- **Popper `translate-y-1`**: source popper content shifts 4px (`data-[side=bottom]:translate-y-1`, select.tsx:66) on top of the default sideOffset — Rust gap is `DROPDOWN_SIDE_OFFSET` 4.0 (select.rs:883); effective offset should be 8.0. Fix: `.gap(8.0 + (1.0 - alpha) * 8.0)`.
- **List max height.** Source clamps to `max-h-(--radix-select-content-available-height)`; Rust hard-codes 300 (select.rs:78). Fix: clamp to available screen space like the P1 ScrollArea fix in dropdown.
- **Animation 150ms vs `duration-100`** — shared helper.
- **Separator full-bleed & viewport `p-1`** match; nothing to do.

**Not portable**
- Radix scroll-button hover auto-repeat mechanics — egui input has no key-repeat-like pointer hold API; only a coarse stand-in possible.

**Confirmed OK**
- Controlled `Option<String>` / `String` bindings + uncontrolled store; placeholder `text-muted-foreground`; disabled opacity-50 + hover-sense; dark `bg-input/30` / `hover:bg-input/50` (select.rs:304-312, tested); invalid border+ring (light /20, dark /40); focus ring 3px ring/50 painted on focus; opening highlights the current selection (select.rs:713-721); ArrowUp/Down skip disabled + wrap, Home/End, Enter/Space select, Escape closes; click-outside dismissal after visible frame; check indicator `right-2` accent; labels muted XS + separators full-bleed; width fits widest option ≥ min; close fade with inert rows.
- Demo covers controlled with label/separator/disabled, both sizes, disabled/invalid states, prefilled, 40-option scrolling list.

---

### combobox — medium
Files: `src/components/combobox.rs`, demo `demos/mod.rs::combobox_demo` (mod.rs:3265).

**P0 (visual/API wrong — must fix)**
- None blocking — the port is a coherent, working composition. The "must fix" below is scoped to bringing it to the actual v4 component.

**P1 (behavior/interaction gap)**
- **Port implements the old docs composition, not `ui/combobox.tsx`.** The module doc claims "There is no dedicated `ui/combobox.tsx` in the source" (combobox.rs:3-4) — false: `apps/v4/registry/new-york-v4/ui/combobox.tsx` (Base UI Combobox) exists with its own parts: trigger is an `InputGroup` (input + `icon-xs` ghost chevron button + optional clear), popup `max-h-96 w-(--anchor-width) min-w-[calc(var(--anchor-width)+28px)] rounded-md ring-1 ring-foreground/10 shadow-md duration-100` with the embedded search field styled `m-1 mb-0 h-8 bg-input/30 border-input/30 shadow-none rounded-lg` (combobox.tsx:119), items `rounded-sm py-1.5 pl-2 pr-8 data-highlighted:bg-accent`, label `px-2 py-1.5 text-xs text-muted-foreground`, empty `py-2 text-center text-sm text-muted-foreground`, chips container `min-h-9 flex-wrap gap-1.5 rounded-md border border-input px-2.5 py-1.5 focus-within:ring-[3px] ring-ring/50` with chips `h-[22px] bg-muted px-1.5 text-xs font-medium rounded-sm gap-1` + remove X `-ml-1 opacity-50 hover:opacity-100`. Fix (visual subset, keep the Rust API):
  1. Correct the module doc to name `ui/combobox.tsx` and say the port follows the docs demo composition with deviations.
  2. Popup frame: radius `md_u8()` (not lg, combobox.rs:742); gap 6.0 (sideOffset=6, combobox.rs:755); keep width = trigger width but also allow `min_width = width + 28.0` when content overflows.
  3. Popup search field: pass a nova-styled compact input to the embedded Command — add `CommandInput::compact(bool)` (or a `Command::input_style(InputStyle::Combobox)`) that renders a 32px rounded-lg `bg-input/30 border-input/30 shadow-none` field inset 4px/0px instead of the `h-9 border-b px-3` row; align with the command nova fix below.
  4. Empty state: render `py-2` muted (`text-muted-foreground`) instead of cmdk's `py-6` foreground — currently inherited from Command (command.rs:952-964). Add a `Command::empty_style` knob or set `py-2`+muted when embedded in combobox.
  5. List max height: `max-h-96 − 36px` ≈ 348 (combobox.rs:70 uses 300).
  6. Multi chips: container should be the input area (min-h-9, wraps to multiple rows, focus ring) with 22px `bg-muted` chips (Rust: 20px `p.secondary`, single-line + `+N` overflow, combobox.rs:74-76,285-287,584-662). Fix: chip bg `p.muted`, `CHIP_HEIGHT = 22.0`, and if feasible let the trigger grow: allocate `max(36.0, rows*26+10)` height with wrapped rows (measure chips into lines at `width - 2*pad - chevron`). If single-line must stay, keep +N and note it.
  7. Add the clear (X) affordance for single-select (`showClear`): a 24px ghost X button left of the chevron that empties the binding (hover `p.muted` bg, `color_alpha(p.foreground, 0.5)` idle → foreground hover).

**P2 (polish)**
- **Trigger is not an InputGroup.** v4 trigger = bordered input-look (like Input/InputGroup: `dark:bg-input/30 dark:hover:bg-input/50`) rather than the docs outline Button — Rust's custom painting (combobox.rs:546-588) is visually close to the outline button (bg background, hover accent light). If (P1.7) is done, also switch light-mode hover to `p.muted`-style input hover? — the docs demo genuinely uses `<Button variant="outline">`, so the current look matches the docs; note only.
- **Check reserves no `pr-8` space.** v4 item reserves 32px for the indicator (`pr-8`); Rust draws the check over the shortcut zone (command.rs:1014-1023 reserves only after-check space). Long labels can underlap the check. Fix in Command::paint_row: reserve `ITEM_PAD_X + ICON_SIZE` right pad when `checked` (already done for `right_pad`) — actually right_pad is added but the label is not truncated to avoid it. Fix: truncate/wrap label width to `rect.width() - text_x - right_reserve` where `right_reserve = 8 + 16` when checked, else 8.
- **`pointer-coarse` size variants** (`size-5` check on coarse pointers) — desktop-only egui; skip.
- **Animation 150ms vs `duration-100`** — shared helper.

**Not portable**
- Base UI Combobox `alwaysSelectOnBlur`, `fillInputOnSelect`, `defaultOpen` input-autocomplete behaviors — different interaction model; the Rust API is docs-composition-shaped.

**Confirmed OK**
- Single/multi/String/uncontrolled bindings with chip removal + `toggled`/`changed` reporting; Escape/outside-click close; check marks on picked rows; disabled options skipped; search filtering via embedded Command with should_filter; +N overflow chip; invalid/disabled trigger states incl. dark input fills (tested); popup width = trigger width; keyboard via embedded Command (arrows/Enter).
- Demo covers single, multi chips, disabled/invalid/string-bound, long list.

---

### command — medium
Files: `src/components/command.rs`, demo `demos/mod.rs::command_demo` (mod.rs:3194).

**P0 (visual/API wrong — must fix)**
- **Selected-row colors use the base recipe, not nova.** `resolve_row_colors` paints `bg-accent text-accent-foreground` (command.rs:426-446); nova `.cn-command-item` is `data-selected:bg-muted data-selected:text-foreground data-selected:*:[svg]:text-foreground` (style-nova.css:397-399) — muted bg, foreground text, icons foreground. Shortcut likewise: nova `group-data-selected/command-item:text-foreground` (command.rs shortcut stays muted otherwise, matches). Fix: in `resolve_row_colors`, selected → `fill = p.muted`, `text = p.foreground`, `icon = p.foreground`, `shortcut = p.foreground`; keep icon muted when unselected; update the test at command.rs:1295-1308.

**P1 (behavior/interaction gap)**
- **Nova root/input chrome missing.** Nova `.cn-command { rounded-xl! p-1 }` and the input is an InputGroup: wrapper `p-1 pb-0`, field `bg-input/30 border-input/30 h-8 rounded-lg shadow-none` with addon `pl-2` (style-nova.css:357-371) — base is `h-9 border-b px-3` full-width (command.rs:763-790). Fix: add `Command::style(CommandStyle::Nova)`-ish plumbing or just switch `render_input` to the nova look: wrapper = 4px top/left/right margin + 0 bottom; inner field: 32px tall, `CornerRadius::lg`, fill `color_alpha(p.input, 0.3)` (dark) / `color_alpha(p.input, 0.3)`? — nova `bg-input/30` applies in both modes; border `color_alpha(p.input, 0.3)`; search icon inside at left `pl-2`; text pad 8+16+8; remove the bottom border line. Also add 4px `p-1` outer padding to the command root (chrome and dialog modes) and `rounded-xl` (`x2l_u8()` — batch E's suggested `RadiusTokens::x2l_u8()` helper; if not added, use `radius.x2l() as u8`).
- **Dialog wrapper width vs `max-w-lg` + list `max-h-72`.** `CommandDialog` width default 512 ✓, but nova list is `max-h-72` (288px, style-nova.css:381-383) — Rust `COMMAND_LIST_MAX_HEIGHT` 300 (command.rs:62). Fix: 288.0.
- **In-dialog item radius.** Nova `in-data-[slot=dialog-content]:rounded-lg!` — items inside the dialog are rounded-lg. Fix: `paint_row` radius: `if in_dialog { radius.lg_u8() } else { radius.sm_u8() }` — thread a bool through `Command` (dialogs set it alongside `force_keyboard`).

**P2 (polish)**
- **`tracking-widest` shortcut** (cross-cutting; accepted).
- **cmdk weight-based ranking.** cmdk ranks matches (score by position/length); Rust is a plain substring filter (command.rs:316-322). Fix: order matching items by (match index asc, label len asc) in `collect_visible_items` when a query is present — cheap approximation.
- **Empty-state text color.** cmdk empty inherits `text-foreground`; ✓ Rust uses popover_foreground. Nova keeps `py-6 text-center text-sm` ✓. Only the combobox-embedding case differs (see combobox P1.4). No change here.
- **Group hiding.** cmdk hides empty groups (headings included) ✓ Rust does (command.rs:869-872). Nothing to do.

**Not portable**
- cmdk `<Command.Input>` value/`debounce` hooks and `Command.Filter` custom filters — Rust exposes `bind`/query instead; API-shaped difference, acceptable.
- `no-scrollbar` (nova list) — egui ScrollArea scrollbar styling is global; accept.

**Confirmed OK**
- Builder groups/headings/items/separators/empty-text; case-insensitive substring filter + `should_filter(false)` pin (tested); selection normalize/fallback, arrows skip disabled + wrap, Home/End, Enter activates, hover moves selection, click activates (hover/click separated from keyboard); input autofocus once per open + `reset_autofocus`; `force_keyboard` for dialogs; CommandDialog via crate Dialog with Escape/backdrop/X close and close-on-select; input `h-9 border-b px-3 gap-2` + `size-4 opacity-50` search icon (base ✓); groups `p-1` + heading `px-2 py-1.5 text-xs text-muted-foreground`; items `rounded-sm px-2 py-1.5` 32px rows with icons `size-4` muted; disabled opacity-50; empty `py-6 text-center text-sm`; separators `-mx-1 h-px bg-border`; standalone chrome `rounded-lg border shadow-md bg-popover` + `md:w-[300px]` + `max-h-[300px]` list.
- Demo covers standalone palette (icons/shortcuts/disabled/groups), dialog palette, activation/query readout.

---

## Batch summary

Ranking by work needed (most → least):

1. **select (medium, biggest visual gap)** — the entire trigger/content/item geometry is base-recipe (h-9/h-8, px-3, 32px rows, min-w-32, md radii, half-opacity chevron); nova wants h-8/h-7, pl-2.5/pr-2, 28px rows, min-w-36, lg radii, full muted chevron, dark invalid border /50. One focused geometry pass + typeahead/Tab-close.
2. **navigation-menu (medium)** — accent→muted color family swap (trigger + links), px-4→px-2.5, md→lg radii, plus the hover-leave close timer and panel-link keyboard nav. All localized in two `resolve_*_colors` fns + `paint_bar_item` constants.
3. **combobox (medium)** — works well as the docs composition but should converge toward `ui/combobox.tsx`: nova popup radius/gap, compact embedded input, py-2 muted empty state, 22px bg-muted wrapping chips, clear button, corrected module doc. Largest API surface touched.
4. **command (medium)** — nova selected colors (muted/foreground) are a small but pervasive P0; nova input-group chrome + `p-1`/`rounded-xl` root + `max-h-72` + in-dialog item radius.
5. **menubar (medium, one sharp P0)** — checkbox/radio label/indicator overlap when `inset` is not manually set (nova `pl-7` is unconditional); fix belongs in the shared `render_rows`. Plus label height 28, trigger focus bg, keyboard entry when closed.
6. **context-menu (high)** — only shared-fix items: content max-height clamp, typeahead, sub-content `border` token, duration-100, radio-check indicator.
7. **dropdown-menu (high)** — same shared-fix items plus sub-popup width override and sub-menu inset API. Nothing dropdown-specific is wrong.

Shared/foundation fixes this batch needs (coordinate centrally): (a) `overlay_progress_secs(ctx, id, open, secs)` in `src/utils/animation.rs` (all seven components use nova `duration-100`); (b) `RadiusTokens::x2l_u8()` in `src/theme/tokens.rs` (command nova root `rounded-xl`); (c) the shared menu renderer changes in `dropdown_menu.rs` (ScrollArea clamp, typeahead, menubar pl-7 unconditional indent, spec-driven label height, sub-popup min-width override, nova radio Check indicator, MenuSpec border-vs-ring for context subs) — these land once in dropdown_menu.rs and fix dropdown, context-menu and menubar together; (d) no new font faces required (font-medium accepted).
