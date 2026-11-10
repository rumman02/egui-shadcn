# Gap analysis — Batch H (AI/chat components)

Scope: `attachment`, `bubble`, `message`, `message-scroller`, `direction`.
Originals: `original-shadcn-ui/apps/v4/registry/new-york-v4/ui/{attachment,bubble,message,message-scroller,direction}.tsx`.
Rust: `egui-shadcn-ui/src/components/{attachment,bubble,message,message_scroller,direction}.rs`. Demos: `examples/gallery/demos/mod.rs` (`attachment_demo` L3609, `bubble_demo` L3753, `message_demo` L3821, `message_scroller_demo` L3879, `direction_demo` L3962).

---

### attachment — high

**P0 (visual/API wrong — must fix)**

- None.

**P1 (behavior/interaction gap)**

- Vertical `AttachmentContent` implements `px-1` as vertical space, not horizontal inset. `attachment.tsx:87` (`group-data-[orientation=vertical]/attachment:px-1`) insets the text column 4px on each side; `attachment.rs:404-407` does `ui.set_min_width(...)` then `ui.add_space(4.0)`, which in a `top_down` layout shifts the column **down 4px** and never insets it. Fix in `AttachmentContent::show`: drop `add_space(4.0)`; when orientation is Vertical, wrap the content closure in `egui::Frame::new().inner_margin(egui::Margin::symmetric(4, 0)).show(ui, content)` (keep the `set_min_width(100.0)`).

**P2 (polish)**

- Vertical media side ignores the active chip size: `attachment.rs:284` computes `VERTICAL_CHIP_WIDTH - 2.0 * AttachmentSize::default().pad().x` (always 100px). For Sm/Xs vertical chips the padding is `px-2`/`px-1.5` → sides 104/108. Fix: read `pad.x` from the scope's actual `s.size` (`scope.map(|s| s.size.pad().x)`).
- `AttachmentGroup` shows a scrollbar; original is `scrollbar-none` (`attachment.tsx:186`). Fix: `egui::ScrollArea::horizontal().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)` in `attachment.rs:851`.
- `AttachmentTitle` ignores `font-medium` (`attachment.tsx:103`). Fix: `.strong()` on the RichText at `attachment.rs:451` (library-wide medium-weight approximation; no medium font is configured anywhere).
- `AttachmentAction` hardcodes `Ghost` + `IconXs` (`attachment.rs:597-604`); original accepts any Button `variant`/`size` (`attachment.tsx:144-159`). Fix: add `variant(Option<ButtonVariant>)`/`size(Option<ButtonSize>)` builders defaulting to Ghost/IconXs.
- `Attachment::trigger(true)` is `Sense::click` only (`attachment.rs:745-750`); the original `AttachmentTrigger` is a focusable `<button>`. Fix: use `Sense::click_and_focus()`... i.e. `Sense::click()` plus `ui.memory_mut` focus — simplest faithful step: build the interact with `Sense::click()` and call `response.widget_info |= focusable`; acceptable minimal fix: add `Sense::focusable_noninteractive()` combined — concretely: `Sense::click_and_focus()` if the egui version exposes it, else leave and document.
- Demo gaps: `AttachmentMediaVariant::Image` never shown (only icon + spinner), and a Sm/Xs vertical chip is never shown. Add one image-media chip and one Xs vertical chip to `attachment_demo`.
- PROGRESS row 82 note "no image-opacity easing" is stale — the original swaps opacity with no transition either; delete that clause.

**Not portable**

- CSS gradient sweep of `shimmer` (title) — approximated as a 1.6s muted↔foreground color pulse (documented, acceptable).
- `flex-wrap` chip content wrapping; `snap-x snap-mandatory` scroll snapping; `overscroll-x-contain`; `transition-colors` (egui restyles instantly).

**Confirmed OK**

- Sizes: media 40/32/28 (`media_side`), icon 16/14, pad (10,8)/(8,6)/(6,4), gap 8/10/6, text 14/12 — all match `attachment.tsx:12-16,49`.
- Chip: `rounded-xl` (Xs → `rounded-lg`), 1px border, `bg-card`/`text-card-foreground`, `min-w-40` (160px), vertical `w-30` (120px) with actions overlaid at media top-right (`top-3 right-3` = 12px inset, `gap-1` = 4px).
- States: idle dashed border (perimeter dash 4/4), error `border-destructive/30` + media `bg-destructive/10 text-destructive` + description `destructive/80`, uploading/processing shimmer, hover `bg-muted/50` gated on `has-actions`/trigger.
- `focus-within:ring-1 ring-ring/50` painted when an action holds focus.
- Media image variant: `object-cover` via `cover_uv`, `opacity-60` unless Done/Idle.
- Group: `gap-3` (12px), `py-1` (4px) — and egui 0.37's default `scroll.fade` already reproduces `scroll-fade-x`.

---

### bubble — low (one P0; otherwise high parity)

**P0 (visual/API wrong — must fix)**

- **Variant text color is discarded** — the default pill renders `foreground` text on `bg-primary` (near-black-on-near-black in light mode, near-white-on-near-white in dark). Original: `*:data-[slot=bubble-content]:text-primary-foreground` etc. (`bubble.tsx:22-34`). Rust: `bubble.rs:392-394` computes `(fill, text_color, border)` then `let _ = text_color;`; the built-in label (`bubble.rs:356-363`) adds only `.size(SM)`, and the theme sets `visuals.override_text_color = Some(foreground)` (theme/mod.rs:278). Fix in `BubbleContent::show_with`: replace `let _ = text_color;` with `ui.style_mut().visuals.override_text_color = Some(text_color);` placed **inside** the `ui.with_layout` closure that runs `content(ui)` (it is scoped to that child ui, so it recolors both the built-in label and custom closure content). Add a unit assertion: for `Default`, text != fill.

**P1 (behavior/interaction gap)**

- Bubble does not follow the enclosing `Message`'s alignment: original `group-data-[align=end]/message:self-end` (`bubble.tsx:17`) right-aligns every bubble inside an end-aligned message; `Bubble::show` (`bubble.rs:307-315`) reads only `self.align`. Fix: in `Bubble::show`, `let msg_end = crate::components::message::scope::current(ui).map(|s| s.align == crate::components::message::MessageAlign::End).unwrap_or(false);` and use `Align::Max` when `self.align == End || msg_end` (both modules are `pub(crate)`, no API change).

**P2 (polish)**

- Reactions pill ignores `has-[button]:p-0` (`bubble.tsx:85`): no interactive buttons inside `BubbleReactions` (it is a static text pill, `bubble.rs:455-523`). Fix: accept optional closure content (`show_with`) and, when present, drop the 6/2 padding and measure the child rect.
- Demo: add one `BubbleContent::show_with` example (e.g. markdown text) and one end-aligned bubble inside `Message::align(End)` to lock the P1 fix.

**Not portable**

- `leading-relaxed` (1.625 line-height) — egui line spacing is font-metric driven, no per-label override.
- `:hover` shifts (`bg-primary/80`, color-mix 5%, etc.) apply only to button/link content (`bubble.tsx:22-34`); static text never triggers them — the port documents this as a no-op, which is correct for text pills.
- `wrap-break-word` long-word breaking: egui wraps at grapheme boundaries, close enough; no `min-content` breaking.

**Confirmed OK**

- Geometry: `px-3 py-2` (12/8), `rounded-xl`, `text-sm`, gap-1 (4px) between contents, BubbleGroup gap-2 (8px), `max-w-[80%]` (0.8 × available), ghost = `max-w-full` + `rounded-none p-0` transparent.
- Colors: secondary/muted/outline/ghost exact tokens; destructive `/10` light `/20` dark; tinted recomputes `oklch(from primary 0.93|0.30 c*0.4 h)` (hover `0.88|0.35`, c×0.5) with a tested sRGB→OKLCH round-trip.
- Reactions: `px-1.5 py-0.5` (6/2), `rounded-full bg-muted`, `ring-3 ring-card` painted outside, side/align math matches `-translate-y-3/4` / `translate-y-3/4` and `left-3`/`right-3` (12px) exactly (unit-tested in `reaction_pill_geometry`).

---

### message — medium

**P0 (visual/API wrong — must fix)**

- None.

**P1 (behavior/interaction gap)**

- Header right-aligns in end-aligned messages. Original: only `MessageFooter` has `group-data-[align=end]/message:justify-end` (`message.tsx:76`); `MessageHeader` (`message.tsx:58-69`) is always left-aligned. Rust `show_message_row_text` (`message.rs:340-358`) applies the `right_to_left` flip to **both**. Fix: add a `justify_on_end: bool` parameter — `MessageFooter::show` passes `true`, `MessageHeader::show` passes `false`; in the function, when `align == End && !justify_on_end`, keep `Layout::left_to_right(Align::Min)`.
- Avatar `-translate-y-8` not ported: original raises the bottom-aligned avatar 32px when the message contains a footer (`message.tsx:37`, `group-has-data-[slot=message-footer]/message:-translate-y-8`). Fix following the existing `.flush()` pattern (immediate mode can't detect siblings retroactively): add `MessageAvatar::raised(bool)` (default false); when true, allocate a `Vec2::new(size, size + 32.0)` slot instead of `Vec2::splat(size)` with the inner layout `Layout::left_to_right(Align::Min)` so the avatar lands 32px above the row bottom. Document that callers pass `raised(true)` when a footer follows.

**P2 (polish)**

- Header/footer `font-medium` (`message.tsx:63,75`) not applied: `.strong()` on the RichText at `message.rs:351-355`.
- `MessageAvatar` paints no `bg-muted rounded-full` backdrop (`message.tsx:37`): if the child is smaller than 32px the original shows a muted circle. Fix: before running `content`, `ui.painter().circle_filled(ui.cursor().center() ... )` — concretely paint `circle_filled(slot_center, size/2, p.muted)` in `MessageAvatar::show` (the Avatar child paints over it).
- Demo: add an end-aligned `MessageHeader` row and a `flush` header after the P1 fixes (footer flush is already shown, `message_demo`).

**Not portable**

- `group-has-*` retroactive variant detection (ghost bubble → `px-0`, footer → avatar shift) is handled by explicit `.flush()`/`.raised()` flags — an acceptable, documented immediate-mode adaptation.

**Confirmed OK**

- Row: `gap-2` (8px), `flex-row-reverse` via `right_to_left`, slots bottom-aligned (`Align::Max`), `w-full` content column (min-width = available), `MessageContent` gap-2.5 (10px), children `self-end` on end (`Align::Max`).
- Header/footer: `text-xs` + `text-muted-foreground` + `px-3` (12px), footer `justify-end` on end, `flush` flag for `group-has-ghost:px-0`.
- `MessageGroup` gap-2 (8px); no `MessageActions` (correctly matches the source exports, message.tsx:84-91).

---

### message-scroller — medium

**P0 (visual/API wrong — must fix)**

- None.

**P1 (behavior/interaction gap)**

- Scroll button is a circle; the original is a `rounded-md` square. `message-scroller.tsx:84-105` renders `<Button variant=… size="icon-sm"/>` whose base class is `rounded-md` (`button.tsx:7`, icon-sm adds only `size-8`, `button.tsx:29`); the Rust `Button` correctly paints `radius.md_u8()` (button.rs:378), but the hand-painted `MessageScrollerButton::show_at` paints `theme.radius.full_u8()` (`message_scroller.rs:111`). Fix: `theme.radius.md_u8()` for both the fill and the stroke at `message_scroller.rs:111-117`. Also paint a focus ring (`utils::focus::paint_ring`, 3px `ring/50`) and make the interact `Sense::click_and_focus()`-equivalent when the button is active, since the original is a real focusable button.

**P2 (polish)**

- No `direction="start"` variant: original `MessageScrollerButton` supports jump-to-start (`message-scroller.tsx:85-101`, `top-4`, ArrowDown rotated 180°). Fix: add `pub enum MessageScrollerDirection { End, Start }`, a `.direction(...)` builder; for Start anchor at `viewport_rect.top() + 16.0 + h/2`, paint `Icon::ArrowUp`, show when `!at_top` (compute like `is_at_bottom` with offset ≈ 0); extend `MessageScrollerResponse` with `at_top`.
- Show/hide is instant; original animates scale/opacity over 200ms (`data-[active=false]:scale-95 opacity-0 duration-200`, `message-scroller.tsx:102`). Fix: `crate::utils::animation::animate_value(ctx, id.with("btn"), target, Duration::from_millis(200))` — paint `ui.set_opacity(alpha)` + scaled rect while alpha > 0.002, skip the interact when alpha < 0.5.
- Manual `MessageScrollerButton::new().show_at(...)` without `id_salt` uses `Id::unique("shadcn_message_scroller_button")` (`message_scroller.rs:94-97`) — collides across instances. Fix: default to `ui.scope_id().with("shadcn_message_scroller_button")` instead of `Id::unique(...)`.
- `data-autoscrolling:scrollbar-none` (hide scrollbar while auto-scrolling) not implemented — optionally hide while `state.follow` is true.
- PROGRESS row 68 says fades are web-only — wrong for egui 0.37: `spacing.scroll.fade` (default strength 0.5 / size 20px) already paints `scroll-fade-b`. Optionally strengthen via `ui.style_mut().spacing.scroll.fade = egui::ScrollFadeStyle { strength: 0.75, size: 24.0 }` in the viewport scope (`message_scroller.rs:199-203`); update the row note.
- Demo: never exercises the jump button's hover or a second scroller instance.

**Not portable**

- `MessageScrollerItem` `scrollAnchor` turn-anchoring + `content-visibility:auto` item virtualization — egui has no per-item content visibility; `ScrollArea::show_rows` requires uniform row heights, a redesign (documented in PROGRESS row 68, still valid).
- Smooth (animated) scroll on jump — `scroll_to_cursor` is instant.
- `MessageScrollerProvider` + `useMessageScroller*` hooks — context-API machinery; immediate mode exposes the same info via `MessageScrollerResponse`.

**Confirmed OK**

- Auto-follow bookkeeping (`stick_to_bottom`, disengage on scroll-up, re-engage at bottom; 1px tolerance) — unit-tested.
- `gap-8` (32px), `height`/`gap`/`id_salt` builders, themed scrollbar (`thumb = bg-border` via `extreme_bg_color`).
- Button geometry: icon-sm 32px (`size-8`), centered (`inset-s-1/2 -translate-x-1/2`), `bottom-4` (16px), `bg-background border-border hover:bg-muted text-foreground` — matches the className overrides, not the nominal `variant="secondary"`; ArrowDown 16px.
- Click → `scroll_to_cursor(Bottom)` next pass + follow re-engages.

---

### direction — medium (partial by design; adoption is the gap)

**P0 (visual/API wrong — must fix)**

- None.

**P1 (behavior/interaction gap)**

- No already-ported component consults the direction stack (verified: only `direction.rs` and `mod.rs` mention it) — `DirectionProvider` currently flips raw `ui` content only. Realistic, bounded scope-closer (see plan below) — mirror **popup placement and chevrons only** in the four popup components.

**Scope-closing plan for `direction` (bounded, ~4 files)**

1. Helpers in `direction.rs` (add + export from `mod.rs` prelude):
   - `pub fn popup_align(ui: &Ui, ltr: egui::RectAlign, rtl: egui::RectAlign) -> egui::RectAlign { if is_rtl(ui) { rtl } else { ltr } }`
   - `pub fn mirrored_chevron(ui: &Ui, ltr: Icon, rtl: Icon) -> Icon` (trivial, same shape).
2. `dropdown_menu.rs`: at the level-align choice (`MenuAlign::rect_align()`, L1212-1214) wrap the result — `popup_align(ui, RectAlign::BOTTOM_START, RectAlign::BOTTOM_END)` for Start, inverse for End, `RectAlign::BOTTOM` unchanged for Center; submenu direction `RectAlign::RIGHT_START` (L967) → `RectAlign::LEFT_START` under RTL; submenu chevron `Icon::ChevronRight` (L937) → `Icon::ChevronLeft` under RTL.
3. `select.rs`: popup `.align(RectAlign::BOTTOM_START)` (L882) → `popup_align(ui, BOTTOM_START, BOTTOM_END)`.
4. `combobox.rs`: same for `.align(RectAlign::BOTTOM_START)` (L754).
5. `popover.rs`: flip `PopoverAlign::Start/End` mappings (L100-102) through `popup_align`.
6. Tests: one test per component asserting the flipped `RectAlign` under `DirectionProvider::rtl()`. Demo: add an RTL dropdown-menu section to `direction_demo`.

Out of scope (flag, do **not** attempt in this pass): row-internal mirroring (icon left↔right, `ml-auto` shortcut position, trigger label/chevron order) in `dropdown_menu::paint_item_row`, select/combobox triggers, `button_group`, `input_group`, `breadcrumb`, `pagination` — touches ~10 components' hand-painted coordinates with low payoff; and Unicode bidi/shaping is impossible at the egui layer (see Not portable).

**P2 (polish)**

- Doc claims are wrong about bidi: `direction.rs:14` and PROGRESS row 69 say "Unicode bidi still renders within labels" — egui 0.37 does **no** bidi reordering (`epaint-0.37.0/src/text/unicode.rs:15` — "TODO(emilk): heed bidi characters"). Arabic text renders in logical order without contextual shaping. Fix both texts to say: RTL runs are **not** reordered/shaped by egui; the provider flips layout geometry only.
- Demo: add the RTL popup section after the P1 plan lands.

**Not portable**

- Text-level bidi reordering / Arabic shaping (egui does neither — upstream TODO).
- Radix's per-component mirror logic beyond popups (row padding asymmetry, keyboard side effects) — flagged out of scope above.

**Confirmed OK**

- Stack semantics: push/pop with correct nesting restore (unit-tested), `current`/`is_rtl`/`row_layout`/`cross_align` pure helpers (unit-tested), `Direction::rtl(ui, …)`/`ltr(ui, …)` convenience scopes, RTL scope right-aligns block content and flips row `main_dir`.

---

## Batch summary

Ranking by severity-then-effort (most work first):

1. **direction** — needs the adoption plan to leave `⚠️ partial`: popup/chevron mirroring in `dropdown_menu`, `select`, `combobox`, `popover` via two new helpers; ~4 files, mechanical, each independently committable. Row-internal mirroring and bidi are permanently out of scope. Also correct the two bidi doc claims.
2. **message-scroller** — one P1 (button shape circle → `rounded-md` + focus ring), plus the start-direction variant and 200ms show/hide animation as P2s; the "fades are web-only" PROGRESS note is stale and should be corrected.
3. **message** — two P1s (header must not right-align on end; avatar `-translate-y-8` needs a `raised` flag) plus font-medium and avatar backdrop polish.
4. **attachment** — closest to done: one small P1 (vertical `px-1` implemented as vertical space), scrollbar hiding, `font-medium`, demo additions.
5. **bubble** — best geometry fidelity in the batch but ships the batch's only P0: variant text color discarded, making the default (and dark) variants unreadable in both modes; the fix is two lines (`override_text_color` inside the content closure). After that, only the message-scope alignment hook remains.

Foundation notes for the fix agent: `color_alpha` (theme/mod.rs:147) is premultiplied-correct; `type_scale::SM/XS` = 14/12; `RadiusTokens::md/lg/xl/full` exist in f32 and u8 forms; `ButtonSize::IconXs/IconSm` = 24/32px; egui 0.37 provides built-in scroll fades (`spacing.scroll.fade`, on by default) and `RectAlign::{BOTTOM_START, BOTTOM_END, LEFT_START, RIGHT_START}` — all the primitives the fixes need are already in place.
