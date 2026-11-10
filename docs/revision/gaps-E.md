# Batch E — Overlays: gap analysis (dialog, alert-dialog, sheet, drawer, popover, tooltip, hover-card, sonner)

References: `original-shadcn-ui/apps/v4/registry/new-york-v4/ui/<name>.tsx` (base), `apps/v4/registry/styles/style-nova.css` (nova `.cn-*` recipe classes; tw-merge keeps them over base classes, state-keyed base utilities like `data-[state=open]:duration-500` still win over plain nova `duration-*`), `apps/v4/registry/__components__/base-nova.tsx` + `packages/registry/test/fixtures/transform-corpus/radix-nova--dialog.tsx` (canonical nova merges), `apps/v4/registry/bases/radix/ui/sonner.tsx` (nova sonner), `apps/v4/registry/new-york-v4/examples/hover-card-demo.tsx`. Rust: `src/components/<name>.rs`, demos in `examples/gallery/demos/`.

Cross-cutting notes affecting this batch:
- All four popup/modal components use `utils/animation::overlay_progress` = 150ms open / 75ms close; nova recipes use `duration-100` (100ms both ways). Shared fix: add `overlay_progress_secs(ctx, id, open, secs)` in `src/utils/animation.rs` and use 0.1 in dialog/alert-dialog/popover/hover-card (accordion's suggested `Duration::Md = 0.2s` is separate).
- Foundation has no `font-medium`; `DialogTitle`/`SheetTitle`/`DrawerTitle`/`AlertDialogTitle`/`PopoverTitle` render regular-weight 16px text. Unfixable without bundled fonts (same limitation as sibling batches); also `leading-none`/`leading-snug` line-heights are not applied (egui 0.37 `RichText::line_height` exists if a fixer wants it, but the galley-painting path would need reworking) — accepted, not repeated per component below.
- Nova style recipe IS implemented per component (bg-popover / ring-foreground/10 / bg-black/10 etc.); the palette itself is style-independent in the source, so no `resolve_colors` gap for batch E.

---

### dialog — medium
Files: `src/components/dialog.rs`, demo `examples/gallery/demos/dialog_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- **Extra `shadow-lg` on the panel that nova does not have.** `dialog.rs:430` adds `.shadow(shadow::shadow_lg(dark))`; the nova recipe is ring-only — `.cn-dialog-content` (style-nova.css:467-469) has `ring-foreground/10 ... ring-1` and NO shadow (the canonical nova merge `radix-nova--dialog.tsx:64` confirms: `rounded-xl bg-popover p-4 text-sm ring-1 ring-foreground/10`). Fix: delete the `.shadow(...)` line from the frame builder and drop the now-unused `shadow` import / `dark` local if unreferenced.
- **Close X is not the nova ghost icon button.** Nova renders `<Button variant="ghost" size="icon-sm">` at `absolute top-2 right-2` (radix-nova--dialog.tsx:70-87) → 28×28 box (h-7), idle text `p.foreground`, hover bg `p.muted` + text `p.foreground` (nova ghost: `hover:bg-muted hover:text-foreground`, style-nova.css:166). `dialog.rs:448-476` uses a 24×24 box, hover fill `p.accent` + fg `p.muted_foreground`. Fix: `Vec2::splat(28.0)`; x_rect origin `Pos2::new(ui.max_rect().right() + pad - 8.0 - 28.0, ui.max_rect().top() - pad + 8.0)` (8px inset stays, top-2/right-2); hover fill `p.muted`, hover fg `p.foreground`; keep idle `color_alpha(p.foreground, 0.7)` (base `opacity-70`).
- **Content vertical `gap-4` (16px) not applied — children sit at the theme's 8px.** Nova `.cn-dialog-content` has `grid gap-4` (style-nova.css:468); theme `item_spacing` is 8.0 (theme/mod.rs:336) and the demo papers over it with `ui.add_space(4.0)` (dialog_demo.rs:19). Fix: inside the frame ui in `DialogContent::show` (dialog.rs:438-447), after `set_min_width`: `ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::LG;` and remove the manual `add_space` in the demo.

**P2 (polish)**
- **Animation timing 150ms open / 75ms close vs nova `duration-100`.** `dialog.rs:405` uses `overlay_progress`. Fix: add `overlay_progress_secs(ctx, id, open, 0.1)` in utils/animation.rs and use it here (shared with alert-dialog/popover/hover-card).
- **`DialogFooter` lacks the source's `showCloseButton` prop** (dialog.tsx:99-118: appends an outline "Close" button wrapped in `DialogPrimitive.Close`). Fix: add `.show_close_button(bool)` field to `DialogFooter`; in `show`, after `content(&mut child)` render `DialogClose::new("Close")` (outline) when set — row is right_to_left, so it lands leftmost like `flex-row sm:justify-end` with the appended child last.
- **Dead animation factors**: `overlay_progress(...).scale/.offset_y` computed at dialog.rs:405 but never applied (fade+slide approximation of `zoom-in-95`; PROGRESS.md already words it that way). Either remove the dead reads in dialog.rs or leave as-is; do not try to scale widget layout.
- **Title line-height**: nova `text-base leading-none` = 16px line box; the galley uses the font's default (~20px). Optional: switch title rendering to `ui.label(RichText::new(text).size(16.0).line_height(16.0))` style.

**Not portable**
- `supports-backdrop-filter:backdrop-blur-xs` overlay blur (nova overlay, style-nova.css:464) — no backdrop blur in egui.
- `zoom-in-95` content scale on widgets — egui widgets don't scale; fade+slide stands in.

**Confirmed OK**
- `sm:max-w-sm` 384px + `max-w-[calc(100%-2rem)]` clamp (dialog.rs:46,411-416); `p-4` inner margin; `rounded-xl`; `ring-foreground/10` 1px stroke; `bg-popover`/`text-popover-foreground`; backdrop `bg-black/10` fading with progress (dialog.rs:52,419-422); centered modal.
- X at `top-2 right-2` (8px inset); header `gap-2` (SM, dialog.rs:518); footer strip `bg-muted/50 -mx-4 -mb-4 rounded-b-xl border-t p-4` full-bleed with right-aligned row (footer_strip, dialog.rs:606-658); title `text-base`; description `text-sm text-muted-foreground` wrapping.
- Escape + backdrop click close (modal `should_close()`); `DialogClose` widget; controlled write-back; trigger outside scope renders inert outline button; extensive tests incl. backdrop-click dismiss and controlled round-trip.
- Demo covers form dialog, controlled, `showCloseButton(false)`, custom width.

---

### alert-dialog — medium
Files: `src/components/alert_dialog.rs`, demo `examples/gallery/demos/alert_dialog_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- **Extra `shadow-lg` on the panel** — same as dialog: nova `.cn-alert-dialog-content` (style-nova.css:49-51) is `ring-foreground/10 ... ring-1` with no shadow. Fix: remove `.shadow(shadow::shadow_lg(dark))` at alert_dialog.rs:514 and clean up unused imports.

**P2 (polish)**
- **`AlertDialogMedia` is centered even for the Default size.** `alert_dialog.rs:664-668` does `center_x.max(rect.center().x)`, pushing the 40px box toward panel center; nova default size is `place-items-start` on desktop (style-nova.css:54) and only `sm` stays centered. Fix: read the active size from the scope (as `AlertDialogHeader::show` does at alert_dialog.rs:568-570); center only for `AlertDialogSize::Sm`, for Default use `rect` unchanged.
- **`sm` footer should be a 2-column grid, not a centered row.** Source `group-data-[size=sm]:grid grid-cols-2` (alert-dialog.tsx:94) → two equal-width cells (Cancel left, Action right); Rust centers children (alert_dialog.rs:737 `Layout::top_down(Align::Center)`). Fix: split `inner_rect` into two `Vec2::new(inner_w/2, content_h)` halves and lay one child into each (measure pass already counts content height; for >2 children overflow into rows).
- **Initial focus on Cancel** (Radix auto-focuses the cancel button on open). Fix: on the first frame the modal is open (a `state_id.with("focused")` temp guard), after buttons render `ctx.memory_mut(|m| m.request_focus(cancel_id))` — needs the cancel button's response id, e.g. store the last Cancel response id in scope data.
- **Mobile width**: nova `data-[size=default]:max-w-xs` below the 640px breakpoint (style-nova.css:50); Rust always uses 384 (alert_dialog.rs:501). Fix: `let bp = if ctx.content_rect().width() < 640.0 { 320.0 } else { self.size.width() };`.
- **Animation timing**: use `overlay_progress_secs(..., 0.1)` (nova `duration-100`, style-nova.css:50) — alert_dialog.rs:494.

**Not portable**
- Backdrop blur on overlay (style-nova.css:46); `zoom-in-95` widget scale — same reasons as dialog.

**Confirmed OK**
- Sizes: Default 384 (`sm:max-w-sm`), Sm 320 (`max-w-xs`) with enum + tests (alert_dialog.rs:52-80); `p-4`; `rounded-xl`; `ring-foreground/10`; backdrop `bg-black/10`.
- Radix semantics: no close X; backdrop click does NOT dismiss (explicitly tested, alert_dialog.rs:910-926); Escape closes without confirming the action (tested, alert_dialog.rs:1001-1036); Action/Cancel auto-close and default variants match (`default`/`outline`, alert-dialog.tsx:149,167) with `.variant()` override.
- Header alignment per size (`default` left / `sm` center, gap-1.5 = XS_HALF); media 40px `bg-muted` `rounded-md` with 24px icon (nova size-10/size-6, style-nova.css:57-59); footer strip identical nova classes; title `text-base`; description `text-sm text-muted-foreground`.
- Demo covers destructive confirm + result readout, media slot, `Sm` size.

---

### sheet — medium
Files: `src/components/sheet.rs`, demo `examples/gallery/demos/sheet_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- **Inner-edge border uses `foreground/10` instead of the `border` token.** Nova `.cn-sheet-content` keeps the base side borders (`data-[side=right]:border-l` etc., style-nova.css:1085-1087) → color `var(--border)`. `sheet.rs:472` paints `Stroke::new(1.0, color_alpha(p.foreground, 0.10))`. Fix: `let border = Stroke::new(1.0, p.border);`.
- **`SheetFooter` lays out a right-aligned horizontal row; the source is a left-aligned column.** sheet.tsx:97-105 `mt-auto flex flex-col gap-2 p-4` (nova `.cn-sheet-footer` keeps `gap-2 p-4`). `sheet.rs:669-671` uses `Layout::right_to_left(Align::Center)`. Fix: vertical top-down, `Align::LEFT`, `item_spacing.y = spacing::SM` (8px); keep the `mt-auto` bottom-push for full-height panels (sheet.rs:654-662) and the 16px padding box; update the demo (buttons currently rely on the row layout).

**P2 (polish)**
- **Close X not the nova ghost icon button.** Nova sheet close is `Button variant="ghost" size="icon-sm"` at `top-3 right-3` (base-nova--sheet.tsx) → 28×28, hover `bg-muted`/`text-foreground`; base X idles at `opacity-70`. Rust: 24×24 (sheet.rs:490-493), hover `p.secondary`/`p.secondary_foreground`. Fix: box 28.0; hover fill `p.muted`, hover fg `p.foreground`; idle `color_alpha(p.foreground, 0.7)`.
- **Panel content `gap-4` missing**: nova `.cn-sheet-content` = `flex flex-col gap-4 ...` (style-nova.css:1086). Fix: inside the panel frame ui (sheet.rs:458-465) `ui.spacing_mut().item_spacing.y = spacing::LG;`; remove the demo's `ui.add_space(4.0)` (sheet_demo.rs:43).
- **Overlay fade tied to slide progress**: nova overlay fades at 150ms (`transition-opacity duration-150`, base-nova--sheet.tsx) while the panel slides 500/300ms. Optional: animate backdrop alpha with its own `animate_value_with_time(id, open ? 1 : 0, 0.15)`.

**Not portable**
- `backdrop-blur-xs` overlay (style-nova.css:1082); vaul-style drag sizing on the sheet; Radix focus-trap internals (egui Modal blocks input instead).

**Confirmed OK**
- All four sides with correct anchoring, single inner-edge border side (sheet.rs:473-486) and slide direction (sheet.rs:428-433); `w-3/4 sm:max-w-sm` = min(384, 75% of screen) (sheet.rs:40-41,410-414); top/bottom auto-height with measured+cached panel height; slide 500ms open / 300ms close (base state-keyed durations win over nova's plain `duration-200` — sheet.rs:48-50 correct); backdrop `bg-black/10`; `ease-in-out` per nova.
- `SheetHeader p-4 gap-0.5` (HALF); title `text-base`; description `text-sm text-muted-foreground`; X at `top-3 right-3` (12px inset); Escape + backdrop click + X + `SheetClose`; controlled write-back; tests for trigger/close/escape/geometry.
- Demo covers all four sides, footer with inputs, no-close-button left sheet.

---

### drawer — medium
Files: `src/components/drawer.rs`, demo `examples/gallery/demos/drawer_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- **Missing `border-t`.** Nova `.cn-drawer-content`: `data-[vaul-drawer-direction=bottom]:rounded-t-xl data-[vaul-drawer-direction=bottom]:border-t` (style-nova.css:496-497). The frame at drawer.rs:395-403 has fill+radius+shadow but no stroke. Fix: after `frame.show(...)` (inside the modal ui, drawer.rs:436-450) paint the top edge: `let panel = ui.min_rect(); ui.painter().line_segment([Pos2::new(panel.left(), panel.top()+0.5), Pos2::new(panel.right(), panel.top()+0.5)], Stroke::new(1.0, p.border));`.
- **Invented `shadow-lg`.** Neither the base (drawer.tsx:58 `fixed z-50 flex h-auto flex-col bg-background`) nor nova css has any shadow on drawer content. Fix: remove `.shadow(shadow::shadow_lg(dark))` at drawer.rs:403 (drop unused imports).
- **Only the bottom direction exists; the source supports top/left/right.** drawer.tsx:59-62 + nova per-direction classes (bottom: rounded-t-xl/border-t; top: rounded-b-xl/border-b; left: rounded-r-xl/border-r; right: rounded-l-xl/border-l; sides `w-3/4 sm:max-w-sm`). Fix: add `pub enum DrawerDirection { Bottom, Top, Left, Right }` (default Bottom) with a `.direction(...)` builder; per direction set anchor (`CENTER_TOP`/`CENTER_BOTTOM`/`LEFT_CENTER`/`RIGHT_CENTER`), drag axis (y for top/bottom, x for sides, dismiss offset sign out of the screen), corner radii (only the screen-facing corners), and the single border edge; side panels `min(width, min(384, 75% screen))`, full-height.

**P2 (polish)**
- **`max-h-[80vh]` not enforced** (drawer.tsx:60): tall content can cover the screen. Fix: inside the frame ui, `ui.set_max_height(ctx.content_rect().height() * 0.8)` (top/bottom directions).
- **`DrawerFooter` should be a left-aligned column.** drawer.tsx:91 `mt-auto flex flex-col gap-2 p-4` (nova keeps `gap-2 p-4`, style-nova.css:528-530). Rust centers horizontally (drawer.rs:618-620). Fix: `Layout::top_down(Align::LEFT)`, `child.spacing_mut().item_spacing.y = spacing::SM`.
- **Header/description centering should be mobile-only.** Base `text-center` (bottom/top direction) is overridden by `md:text-left` (drawer.tsx:79; nova keeps `md:text-left`, style-nova.css:513). Rust always centers (drawer.rs:524-526, 582-585). Fix: center only when `ctx.content_rect().width() < 768.0`, else left-align.
- **`HEADROOM` comment is wrong** (drawer.rs:41-43 claims mt-24 "keeps the top 96 px free" while the constant is 24.0; source `mt-24` = 96px and has no visual effect on a bottom-anchored auto-height panel). Fix the comment, e.g. "the source's mt-24 (96px) is a no-op for a bottom-anchored panel; a small resting budget keeps the slide math positive".
- **Demo**: add a footer note/hint that only the bottom direction is supported until P1 lands.

**Not portable**
- vaul spring physics (Rust uses egui linear `animate_value_with_time`); touch swipe gestures beyond the pointer drag; `fadeFromPosition`/snap points (source default is dismissible-only; PROGRESS.md already notes this).

**Confirmed OK**
- Drag-to-dismiss with 25% (min 80px) threshold + snap-back below it (vaul default dismissible drawer), grab-point tracking so pointer jumps don't pollute the offset (tested both ways, drawer.rs:838-942).
- `bg-popover`/`text-popover-foreground`; top corners `rounded-t-xl` (style-nova.css:497); handle `bg-muted h-1 w-[100px] rounded-full mx-auto mt-4` (nova h-1, drawer.rs:439-448) with `.show_handle(false)`; backdrop `bg-black/10` tracking drag offset; Escape + backdrop click dismiss; `DrawerHeader p-4 gap-0.5` (HALF); title `text-base`; description `text-sm text-muted-foreground` centered for bottom.
- Demo covers basic bottom drawer, controlled, no-handle.

---

### popover — high
Files: `src/components/popover.rs`, demo `examples/gallery/demos/popover_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- None.

**P2 (polish)**
- **Content `gap-2.5` (10px) missing**: nova `.cn-popover-content` = `flex flex-col gap-2.5 rounded-lg p-2.5 ...` (style-nova.css:927). Theme spacing is 8px. Fix: in `popup.show` (popover.rs:424-430) `ui.spacing_mut().item_spacing.y = spacing::SM_HALF;`; the demo's manual `ui.add_space(6.0)` (popover_demo.rs:24) then becomes unnecessary.
- **No `side` API**: source `PopoverContent` accepts Radix `side` (docs demos use `side="top"` etc.); Rust always opens below. Fix: add `pub enum PopoverSide { Top, Bottom, Left, Right }` (default Bottom) + `.side(...)` on `PopoverContent`, mapping to `RectAlign::TOP/LEFT/RIGHT/BOTTOM`; keep the slide-in offset along the opening axis (gap formula popover.rs:401 generalizes as-is).
- **Timing**: `overlay_progress_secs(..., 0.1)` (nova `duration-100`, style-nova.css:927) — popover.rs:394.
- Title `font-medium` weight — see cross-cutting note.

**Not portable**
- `--radix-popover-content-transform-origin`; Radix collision resolution details (egui `Popup` flips/avoids edges differently — acceptable); `PopoverAnchor` (source export) — egui's `PopupAnchor::ParentRect(trigger.rect)` covers the common use; document as API mapping.

**Confirmed OK**
- `w-72` 288px (popover.rs:48); `sideOffset = 4` (popover.rs:50) plus 8px slide-in along the opening axis (`slide-in-from-top-2`); `p-2.5` (10px); `rounded-lg`; `ring-foreground/10` 1px; `shadow-md`; `bg-popover`/`text-popover-foreground`.
- Trigger click toggles; Escape and click-outside dismiss (modal-aware so dialogs keep Escape); controlled write-back + uncontrolled per `id_salt`; align start/center/end; header `gap-0.5` (HALF); description `text-muted-foreground`; tests for toggle/controlled/outside/escape.
- Demo covers basic (official "Dimensions" example), controlled, align start/end, custom width.

---

### tooltip — medium
Files: `src/components/tooltip.rs`, demo `examples/gallery/demos/tooltip_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- **Default hover delay 300ms; the source defaults to 0.** tooltip.tsx:7-10 `TooltipProvider({ delayDuration = 0 })` — shadcn tooltips open immediately. `tooltip.rs:32` `TOOLTIP_DEFAULT_DELAY_MS = 300`. Fix: change the constant to `0` and update the module docs (tooltip.rs:7-8) to "opens immediately by default (`delayDuration = 0`), like the source's provider"; keep the demo's explicit `TooltipProvider::new().delay_ms(300)` (that choice is demo-liberty).
- **`text-xs` (12px) body size not applied** (tooltip.tsx:44): content renders at the theme's 14px body size. Fix in `TooltipContent::show` (tooltip.rs:415-420), inside the frame ui: `ui.style_mut().text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(crate::theme::tokens::type_scale::XS));` (also `TextStyle::Button`/`Small` if kbd/buttons can appear inside).

**P2 (polish)**
- **Kbd-aware right padding**: nova `has-data-[slot=kbd]:pr-1.5` → right padding 6px when the content contains a Kbd (style-nova.css:1381). Fix: add a `.with_kbd(bool)` builder on `TooltipContent` that sets `inner_margin.right = 6` (demo's kbd case at tooltip_demo.rs:80-85 opts in).
- **Arrow tuck-in**: source arrow is `translate-y-[calc(-50%-2px)]` (tooltip.tsx:50) — the diamond overlaps the box edge by ~3px instead of dead-centering on it. Fix: shift the diamond center 2px into the box: Bottom → `Pos2::new(outer.center().x, outer.bottom() - arrow - 2.0)`; Top → `outer.top() + arrow + 2.0`; Left/Right analog (tooltip.rs:424-431).

**Not portable**
- Radix portal/`--radix-tooltip-content-transform-origin`; exact `animate-in` easing (egui linear interpolation).

**Confirmed OK**
- `bg-foreground`/`text-background` colors (tooltip.rs:382,418); `rounded-md`; `px-3 py-1.5` (12/6, tooltip.rs:34-35, tested); `sideOffset = 0` with 8px slide-in; arrow = `size-2.5` (10px) rotated-square diamond pointing at the trigger on all 4 sides; hover-open with delay + hover clock, stays open over the popup, closes on leave and Escape (modal-aware); `TooltipProvider` global delay; tests for delay/close/geometry.

---

### hover-card — high
Files: `src/components/hover_card.rs`, demo `examples/gallery/demos/hover_card_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- None.

**P2 (polish)**
- **No `align` API** (source `HoverCardContent align = "center"` default, hover-card.tsx:23); Rust hardcodes `RectAlign::BOTTOM` (hover_card.rs:412). Fix: add `.align(PopoverAlign)` reusing `components::popover::PopoverAlign` and map Start/Center/End → `BOTTOM_START/BOTTOM/BOTTOM_END`.
- **Default trigger underline is always drawn**; the official demo trigger is `<Button variant="link">@nextjs</Button>` (hover-card-demo.tsx) — underline only on hover (button link variant: `hover:underline`). Fix in `link_label` (hover_card.rs:304-309): draw the underline only when `response.hovered()`, keep `p.primary` idle / `color_alpha(p.primary, 0.8)` hover (ButtonVariant::Link behavior).
- **Timing**: `overlay_progress_secs(..., 0.1)` (nova `duration-100`, style-nova.css:669) — hover_card.rs:386.
- **Demo parity** (optional): mirror the official demo — content `.width(320.0)` (`w-80`), avatar-less row with title "text-sm font-semibold", description, muted `text-xs` "Joined December 2021".

**Not portable**
- `--radix-hover-card-content-transform-origin`; Radix collision specifics (egui Popup).

**Confirmed OK**
- `w-64` 256px; `p-2.5`; `rounded-lg`; `ring-foreground/10`; `shadow-md`; `sideOffset = 4` + 8px slide-in; `bg-popover`/`text-popover-foreground`.
- Radix delays 700ms open / 300ms close exposed as builders and tested (open/close grace, short-hover suppression); stays open while the pointer is over the card (interactive content works); Escape closes; static fallback outside a root.

---

### sonner — medium
Files: `src/components/sonner.rs`, demo `examples/gallery/demos/sonner_demo.rs`.

**P0 (visual/API wrong — must fix)**
- None.

**P1 (behavior/interaction gap)**
- **Toast corner radius is `lg` (10px); nova is `rounded-2xl` (18px).** Nova styles the toast via `toastOptions.classNames.toast = "cn-toast"` (bases/radix/ui/sonner.tsx:77) and `.cn-toast { @apply rounded-2xl }` (style-nova.css:1245-1247) — 1.8 × radius = 18px at base 10. `sonner.rs:264` uses `theme.radius.lg_u8()`. Fix: add `RadiusTokens::x2l_u8()` next to the other `*_u8` helpers in tokens.rs and use `CornerRadius::same(theme.radius.x2l_u8())`.

**P2 (polish)**
- **Icon glyphs for warning/error are circle stand-ins.** Source maps warning → `TriangleAlertIcon`, error → `OctagonXIcon` (sonner.tsx:21-24 / bases/radix/ui/sonner.tsx); Rust substitutes `Icon::CircleAlert`/`Icon::CircleX` (sonner.rs:71-73, documented). Fix: add `Icon::TriangleAlert` (triangle outline + exclamation dot/line, 24-unit lucide geometry) and `Icon::OctagonX` (octagon + x) to `src/icons/icons.rs`, then map `ToastVariant::Warning/Error` to them; update the variant test (sonner.rs:431-439).
- **Description size/opacity**: Rust renders the description at XS 12px with 0.7 alpha (sonner.rs:242, 321); sonner's default description is ~13px at ~0.9 opacity. Fix: use `type_scale::SM` for `desc_font` and `color_alpha(p.popover_foreground, 0.85)` for the muted color.
- **Enter/leave easing**: both reuse `LEAVE_SECS = 0.2` with a linear-ish egui curve (sonner.rs:45-46, 234-238); sonner's enter is a slightly longer springy rise. Optional: use 0.3s for the enter target and 0.2s for leave via two `animate_value_with_time` durations.

**Not portable**
- Swipe-to-dismiss and `visibleToasts` collapse (already noted in PROGRESS.md); `richColors`/`invert` (source does not enable them — the monochrome `--normal-*` styling is the faithful default); Toaster props like `position`/`expand` (accepted by the source component but unused).

**Confirmed OK**
- CSS variable mapping: `--normal-bg`→`p.popover`, `--normal-text`→`p.popover_foreground`, `--normal-border`→`p.border` (sonner.rs:276-282) and theme-following dark mode; icons `size-4` (16px) with spinning loader (sonner.rs:290-308); 356px width; 4s auto-dismiss; click-to-dismiss with exit animation; newest toast at the bottom (bottom-right, 16px offset); 14px inter-toast gap; `Sonner::clear()`; queue/animation lifecycle tested.
- Demo covers all 6 variants plus sticky/short/clear timing controls.

---

## Batch summary

No P0s in batch E — every component is structurally right, theme-token driven, and behaviorally close to Radix (tested open/close/dismiss semantics). Remaining work is mostly recipe-accuracy cleanup plus a few real feature gaps.

Ranked by work needed (most first):
1. **drawer (medium)** — biggest batch item: needs `border-t`, removal of the invented shadow, and a full `DrawerDirection` enum (top/left/right) with per-direction geometry; plus max-h-80vh and footer/header alignment fixes.
2. **dialog (medium)** — remove nova-contradicting `shadow-lg`, rebuild the X as the 28px ghost button (hover bg-muted/foreground), apply content `gap-4`, 100ms timing, `DialogFooter.showCloseButton`.
3. **alert-dialog (medium)** — remove `shadow-lg`; fix media centering per size, `sm` footer 2-col grid, initial focus on Cancel, mobile width clamp, 100ms timing.
4. **sheet (medium)** — border color → `p.border`, footer must be a left-aligned column (currently a right-aligned row), 28px ghost X, content `gap-4`.
5. **tooltip (medium)** — default delay must be 0ms (source `delayDuration = 0`), enforce `text-xs` on content; kbd padding and arrow tuck-in polish.
6. **sonner (medium)** — `rounded-2xl` radius (needs a tiny `x2l_u8` helper), true TriangleAlert/OctagonX icons, description size/opacity.
7. **popover (high)** — only gap-2.5 spacing, an optional `side` API, and 100ms timing.
8. **hover-card (high)** — hover-only underline, `align` API, 100ms timing; otherwise faithful.

Shared fix worth doing once: add `overlay_progress_secs(ctx, id, open, secs)` (100ms) in `src/utils/animation.rs` for dialog/alert-dialog/popover/hover-card, and a `RadiusTokens::x2l_u8()` helper in tokens.rs for sonner. PROGRESS.md rows for dialog/alert-dialog/sheet/hover-card/popover are accurate; after fixes, update rows for drawer (add "bottom-only direction gap" until P1 lands), tooltip (delay default + text-xs now fixed), and sonner (rounded-2xl + icon fidelity).
