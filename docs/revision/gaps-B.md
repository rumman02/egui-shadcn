# Batch B (form inputs) — gap analysis

Reference: nova recipe = `original-shadcn-ui/apps/v4/registry/styles/style-nova.css` (verified against
`packages/registry/test/fixtures/transform-corpus/radix-nova--field.tsx`: the per-style transform
REPLACES the component's base classes with the `cn-*` recipe classes, so the nova `@apply` values are
the authoritative target wherever they define a property; base TSX classes fill the rest).
Originals: `apps/v4/registry/new-york-v4/ui/<name>.tsx`.

**Cross-cutting (affects most of the batch):**
- **(a) Nova recipe not implemented.** No component in this batch branches on `theme.style`
  (`grep` shows only `alert.rs` does, via `Style::Nova` match arms). Every component below implements
  the base recipe. The fix pattern already exists in `src/components/alert.rs:38-60` (style-conditional
  helper fns + `theme.style` read at `ShadcnTheme::of_ui`). Per-component nova deltas are listed below.
  All fixes: add a private `fn xxx(style: Style) -> T` helper (or read `theme.style == Style::Nova`)
  and use `theme.radius.lg_u8()` instead of `md_u8()` where nova says `rounded-lg`.
- **(b) font-medium.** Nova toggle/field-legend/field-label/field-title/input-group-addon use
  `font-medium`. egui has no medium face; `.strong()` only brightens. Not fixable per-component;
  needs a foundation decision (e.g. a `FontId` with a synthesized-semibold face or accept regular).
  Logged as P2 under each affected component.
- **(c) line-height.** Tailwind `text-sm` = 14px/20px, `text-base` = 16px/24px, `leading-snug` 1.375,
  `leading-normal` 1.5. egui defaults ≈1.3. Use `RichText::line_height()` / `FontId` + galley line
  height 20.0 for 14px text in multi-line contexts (textarea, input-group textarea). Single-line
  widgets are unaffected (vertically centered).

---

### input — medium
Rust: `src/components/input.rs` (base recipe in `resolve_input_colors` L49-77).
Original: `input.tsx`; nova: style-nova.css L677-679 (`.cn-input`).

**P0 (visual/API wrong — must fix)**
- Nova geometry wrong: rust renders `h-9` (36, `INPUT_HEIGHT` = `spacing::CONTROL`, L31-32, used L194),
  `rounded-md` (`radius.md_u8()` L229), `px-3` (`Margin::symmetric(MD, 0)` L196-199). Nova:
  `h-8 rounded-lg px-2.5 py-1 text-base md:text-sm`. Fix: style-conditional — height
  `Style::Nova => 32.0, _ => 36.0`; corner `if style == Style::Nova { theme.radius.lg_u8() } else { theme.radius.md_u8() }`;
  horizontal margin `Style::Nova => 10.0, _ => 12.0`.
- Nova disabled fill missing: nova adds `disabled:bg-input/50 dark:disabled:bg-input/80`
  (on top of `disabled:opacity-50`). Rust only fades text to 50% (L183-187); the fill stays
  transparent/dark-input/30. Fix: in `resolve_input_colors`, add `disabled: bool` param;
  disabled fill = `color_alpha(p.input, if dark { 0.8 } else { 0.5 })` (light: input/50 replaces
  transparent).

**P1 (behavior/interaction gap)**
- Disabled border not faded: base `disabled:opacity-50` fades the whole element (border included);
  rust keeps the stroke at full `p.input` (L60-66, 265-270). Fix: multiply `stroke.color` by 0.5
  alpha when disabled (same pattern as checkbox.rs L192-197).

**P2 (polish)**
- Focus ring radius uses `radius` of the frame (L275) — correct, but after the P0 radius fix ensure
  the ring follows `lg` too (it derives from `radius`, so it does once the frame changes).
- Placeholder color: hint text uses egui default gray; set `hint_color = color_alpha(p.muted_foreground, 0.5)` —
  actually use `p.muted_foreground` full (Tailwind `placeholder:text-muted-foreground`), not egui's
  built-in. Fix: `.hint_text(...)` + `TextEdit::hint_color` if available, else paint via layout —
  simplest: keep `hint_text` but call `.text_color()` unchanged and set `VerticalAlign`; verify the
  hint renders in `p.muted_foreground` (check visually in gallery).

**Not portable**
- `file:` input styles (`file:h-6 file:text-sm file:font-medium`) — no file-picker concept in egui.
- `selection:bg-primary selection:text-primary-foreground` — egui has no text-selection styling hook.
- `md:text-sm` responsive switch — egui is desktop-scale; rust already uses 14px everywhere (correct).

**Confirmed OK**
- Focus ring 3px `ring-ring/50` outside the border, invalid border `destructive` + ring
  `destructive/20` (dark /40) — exact (`resolve_input_colors` L60-71, L273-276).
- `dark:bg-input/30` fill (L55-58). `w-full` default, ButtonGroup border fusing, `shadow-xs`,
  password/char-limit/id_salt API, disabled `interactive(false)`.

### textarea — medium
Rust: `src/components/textarea.rs`. Original: `textarea.tsx`; nova L1337-1339 (`.cn-textarea`).

**P0**
- Nova radius/padding: rust `rounded-md` (L183) + `Margin::symmetric(MD=12, SM=8)` (L149-152). Nova:
  `rounded-lg ... px-2.5 py-2`. Fix: style-conditional radius `lg` under nova; margins
  `(10.0, 8.0)` under nova.
- Nova disabled fill missing (same as input): nova `disabled:bg-input/50 dark:disabled:bg-input/80`.
  Fix identical to input (pass `disabled` into `resolve_textarea_colors`, L27-51).

**P1**
- Disabled border not faded (same as input). Fix: 0.5 alpha on stroke when disabled.

**P2**
- Line-height: multiline body text should render at 20px for 14px font (cross-cutting (c));
  apply line-height to the TextEdit via a custom `FontId`/`RichText` if egui exposes per-widget
  line height, else accept.
- `min-h-16` (64px) correct (`TEXTAREA_MIN_HEIGHT` L22); `field-sizing-content` auto-grow is
  approximated by `desired_rows` — acceptable, note only.

**Not portable**
- `field-sizing-content` exact auto-grow (rust uses fixed `rows`/min-height; egui TextEdit does not
  auto-size). `resize` handle n/a.

**Confirmed OK**
- Same focus/invalid recipe as input (shared logic, unit-tested), `dark:bg-input/30`, `shadow-xs`,
  disabled `interactive(false)`, placeholder hint, `rows` clamp.

### checkbox — high
Rust: `src/components/checkbox.rs`. Original: `checkbox.tsx`; nova L283-293.

**P0** — none.

**P1**
- Focus border not recolored: base+nova both have `focus-visible:border-ring`; rust
  `resolve_checkbox_colors` keeps stroke = `p.input` on focus (L53-85) and only paints the ring
  (L216-225). Fix: pass `focused` in and set `stroke_color = p.ring` when `focused && !invalid`
  (invalid wins: `aria-invalid:border-destructive`).
- Nova `focus-visible:ring-3` + `ring-ring/50`: implemented. Nova adds
  `group-has-[:focus-visible]/field-label:ring-0` (label-wraps-checkbox pattern) — only relevant to
  the nested FieldLabel composition (see field), skip here.

**P2**
- Nova `transition-colors` (150ms color transition on check) — animate fill/stroke alpha with
  `utils::animation::animate_value(..., Duration::Fast)` if desired; low impact.
- font-medium n/a. Line-height n/a.

**Not portable**
- `disabled:cursor-not-allowed` (no cursor concept).
- `peer`/`group-has-*` selectors — handled via explicit field ctx in field.rs; fine.

**Confirmed OK**
- size-4 (16), literal `rounded-[4px]` (`CHECKBOX_RADIUS` L33), check icon `size-3.5` (14, L35),
  checked `bg-primary border-primary text-primary-foreground`, dark unchecked `bg-input/30`,
  nova's `aria-invalid:aria-checked:border-primary` rule (L74-78 — invalid+checked keeps primary,
  matches nova even though base TSX lacks it — good), indeterminate minus icon, disabled 0.5 alpha on
  fill/stroke/icon (L192-197), focus ring 3px, Space/Enter via egui, `Widget` impl.

### radio-group — medium
Rust: `src/components/radio_group.rs`. Original: `radio-group.tsx`; nova L967-986.

**P0**
- Nova checked look inverted: nova item is `data-checked:bg-primary data-checked:border-primary
  data-checked:text-primary-foreground` with indicator dot `bg-primary-foreground` — i.e. a SOLID
  primary disc with a light dot. Rust keeps the base look (transparent fill, `fill-primary` dot:
  `resolve_radio_colors` L93-102 paints dot `p.primary`). Fix: in `resolve_radio_colors` add
  `style: Style`; when `Style::Nova && checked`: `fill = p.primary`, `stroke.color = p.primary`,
  `dot = p.primary_foreground`; keep base behavior otherwise (`paint_radio` L134-150 already paints
  fill before the dot, so the solid disc works as-is).

**P1**
- Group gap: base `gap-3` (rust `GAP = spacing::MD` = 12, L33) vs nova `gap-2` (8). Fix:
  style-conditional `Style::Nova => 8.0, _ => 12.0` (used L347, L351).
- Focus border not recolored: base+nova `focus-visible:border-ring` — same fix as checkbox (pass
  `focused` into `resolve_radio_colors`, use `p.ring` unless invalid).

**P2**
- Nova `aria-invalid:aria-checked:border-primary` — invalid+checked keeps primary border; rust
  `stroke_color = if invalid { destructive }` unconditionally (L87). Fix: `if invalid && !checked`.
- Focus ring radius: `paint_radio` uses `radius` (md=8) on an 18px expanded rect — visually circular
  already; after nova changes keep radius = half the item (8).

**Not portable**
- Radix `RovingFocusGroup` wrap-around — rust relies on egui spatial nav (documented; acceptable).

**Confirmed OK**
- size-4 item, size-2 (8px) dot, `dark:bg-input/30` unchecked fill, invalid border+ring colors
  (light /20, dark /40), disabled 0.5 alpha on fill/stroke/dot, click-select (never unchecks),
  roving focus selection-on-focus, disabled options skipped by nav, standalone `RadioGroupItem`.

### switch — high
Rust: `src/components/switch.rs`. Original: `switch.tsx`; nova L1249-1263.

**P0** — none (nova size tokens match base exactly: 32×18.4 / 24×14 track, 16/12 thumb,
`checked_offset = thumb-2`; verified `SwitchSize` L41-71).

**P1**
- Focus border: base+nova `focus-visible:border-ring` on a `border border-transparent` track —
  rust draws no stroke when focused (L199-203 only strokes when invalid). Fix: when focused &&
  !invalid, stroke the track with `p.ring` 1px.

**P2**
- Thumb animation: rust 150ms (`animation::Duration::Default`, L236-242) — source is a CSS
  transition (~150ms); OK. Nova `aria-invalid:ring-destructive/20 aria-invalid:border-destructive`
  implemented (L199-200, L108-112). `shadow-xs` kept — matches (nova doesn't remove it).

**Not portable**
- `disabled:cursor-not-allowed`.

**Confirmed OK**
- Track colors: checked `primary`, unchecked `input` (dark `input/80`) — exact; thumb `background`,
  dark unchecked `foreground`, dark checked `primary-foreground` — exact; invalid ring destructive
  alphas; disabled 0.5 alpha; rounded-full track/thumb; group fusing; `Widget` impl; both sizes.

### slider — medium
Rust: `src/components/slider.rs`. Original: `slider.tsx`; nova L1227-1242.

**P0**
- Nova thumb geometry/border: base thumb `size-4 border-primary`; nova thumb `size-3
  rounded-full border border-ring bg-white`. Rust: `SLIDER_THUMB_SIZE = 16.0` (L28),
  `thumb_stroke = Stroke::new(1.0, p.primary)` (L70). Fix: style-conditional —
  `Style::Nova => 12.0, _ => 16.0` thumb; nova border color `p.ring`, base `p.primary`.
- Nova track thickness: base `h-1.5` (6px) vs nova `h-1` (4px). Rust `SLIDER_TRACK_HEIGHT = 6.0`
  (L30). Fix: `Style::Nova => 4.0, _ => 6.0` (used L171, L186-212).
- Nova ring width: base `hover:ring-4 focus-visible:ring-4` vs nova `hover:ring-3
  focus-visible:ring-3 active:ring-3`. Rust paints 4.0 (L305-311). Fix: 3.0 under nova.

**P1**
- No vertical orientation: source supports `data-[orientation=vertical]` (`min-h-44` base /
  `min-h-40` nova, vertical track `w-1.5`/`w-1`, column layout). Rust is horizontal-only (whole
  file). Fix: add `Slider::orientation(GroupOrientation)` — vertical reserves a 160px-tall column
  (nova `min-h-40`), track `Rect::from_center_size(center, Vec2::new(TRACK_H, width))`, reuse the
  same frac math on y; range fill vertical; keep keyboard Up/Down primary.

**P2**
- Ring color `ring-ring/50` correct in both styles (`c.ring`, L71). Thumb `shadow-sm` (base; nova
  keeps it) — implemented L314-324. Hover ring shows on `hovered||dragged||has_focus` (L296-297) —
  matches base semantics (nova uses focus-visible; dragging≈active ok).
- Add `PageUp/PageDown` (Radix steps by 1/10th range... actually large step) — optional.

**Not portable**
- `touch-none` / touch gesture fine-tuning.

**Confirmed OK**
- Track `bg-muted` rounded-full, range `bg-primary` (both styles), single+range values, thumb
  clamping/crossing prevention, step snapping, track-click/drag nearest thumb, arrows+Home/End,
  `data-[disabled]:opacity-50` (alpha on fills), disabled sense=hover.

### toggle — medium
Rust: `src/components/toggle.rs`. Original: `toggle.tsx` + nova L1342-1368.

**P0**
- Nova sizes: base `default h-9 min-w-9 px-2 / sm h-8 min-w-8 px-1.5 / lg h-10 min-w-10 px-2.5`;
  nova `default h-8 min-w-8 px-2.5 / sm h-7 min-w-7 px-2.5 / lg h-9 min-w-9 px-2.5` and sm adds
  `rounded-[min(var(--radius-md),12px)] text-[0.8rem]`. Rust: heights 36/32/40, px 8/6/10
  (`ToggleSize` L65-87). Fix: style-conditional tables — nova heights 32/28/36, padding_x
  10/10/10, sm radius `min(radius.md,12)=8`, sm font 12.8px, sm icon 14px (base sm keeps 16px icon).
- Nova pressed colors: base `data-[state=on]:bg-accent data-[state=on]:text-accent-foreground`;
  nova `data-[state=on]:bg-muted` (text stays `accent-foreground`). Rust pressed = `p.accent` +
  `p.accent_foreground` (L120-126). Fix: `fill = if style==Nova { p.muted } else { p.accent }`
  when pressed; fg stays `p.accent_foreground` in both.
- Nova hover text: base `hover:text-muted-foreground` vs nova `hover:text-foreground` (hover bg
  stays `bg-muted`). Rust hover fg = `muted_foreground` (L120-126). Fix: fg on hover =
  `if style==Nova { p.foreground } else { p.muted_foreground }`.

**P1**
- Nova outline variant has NO shadow: `.cn-toggle-variant-outline` = `border-input hover:bg-muted
  border bg-transparent` (no `shadow-xs`). Rust draws shadow-xs for every outline toggle
  (L291-301). Fix: skip shadow when `style == Style::Nova`.
- Nova radius: `rounded-lg` (base `rounded-md`). Rust `theme.radius.md_u8()` (L286). Fix:
  `lg_u8()` under nova (before group fusing).
- Nova gap: `gap-1` (4px) vs base `gap-2` (8). Rust `gap() = SM` (L89-92). Fix: nova → 4.0.

**P2**
- font-medium (cross-cutting (b)). `transition-all` — animate fill alpha 150ms if desired.
- Invalid state: rust sets destructive ring+border (L276-279) — matches nova
  `aria-invalid:ring-destructive/20 dark:...40 aria-invalid:border-destructive`; base lacks
  `aria-invalid:border-destructive`?? base has it — fine either way.

**Not portable**
- `whitespace-nowrap` (single-line only anyway).

**Confirmed OK**
- Focus ring 3px `ring-ring/50` + (after P1) border, disabled 0.5 alpha incl. border, min-width
  behavior, icon+text centering with gap, group fusing via button group protocol, Space/Enter,
  `Widget` impl, invalid+disabled ordering.

### toggle-group — medium
Rust: `src/components/toggle_group.rs` (delegates to Toggle). Original: `toggle-group.tsx`;
nova L1370-1377.

**P0** — none beyond inherited toggle.rs fixes (sizes/pressed/hover/radius flow through automatically
since items are `Toggle`s).

**P1**
- Fused item padding: base item adds `px-3` at `spacing=0`; nova `.cn-toggle-group-item` overrides
  to `px-2`. Rust forces `px3 = spacing::MD` (12) when fused (L214, L228-232). Fix:
  style-conditional `Style::Nova => 8.0, _ => 12.0` for the fused padding.
- Fused outline shadow: base item has `data-[spacing=0]:shadow-none`; nova outline has no shadow at
  all. Rust still draws shadow-xs on fused outline items (toggle.rs L291-301, position not
  consulted). Fix: in `Toggle::show`, skip the outline shadow when `position != GroupPosition::Only`
  (i.e. inside a fused group) — covers both styles.

**P2**
- Nova group `data-[size=sm]:rounded-[min(var(--radius-md),10px)]` (8px) — after the toggle sm
  radius fix (`min(radius.md,12)=8`) items already match; no separate work.
- Corner fusing `first:rounded-l-lg last:rounded-r-lg` (nova) — handled by `fuse_radius`; verify it
  uses `lg` once Toggle's radius is style-conditional.

**Not portable**
- Radix roving tabindex mechanics (egui spatial nav instead; documented).

**Confirmed OK**
- single (deselect on re-press) / multiple modes, spacing>0 = individual rounded items with gap,
  vertical orientation, disabled group, icon items, size/variant propagation from group to items.

### input-otp — medium
Rust: `src/components/input_otp.rs`. Original: `input-otp.tsx`; nova L681-700.

**P0**
- Nova slot size: base `h-9 w-9` (36) vs nova `size-8` (32). Rust `OTP_SLOT_SIZE = 36.0` (L51,
  used L403, L450). Fix: style-conditional — read theme style inside `show` (the const is used in
  tests; keep the const for base, override per-style at the two allocation sites).
- Nova radius: first/last slots `rounded-l-lg`/`rounded-r-lg` vs rust `radius.md_u8()` (L572). Fix:
  `lg_u8()` under nova.

**P1**
- Disabled fade incomplete: container `has-disabled:opacity-50` fades borders+bg too. Rust only
  fades char/caret color (`alpha`, L609-625); stroke and dark fill stay full. Fix: multiply
  `stroke.color` and the dark fill by 0.5 when disabled.

**P2**
- Separator icon color: source inherits foreground (no color class); rust uses `muted_foreground`
  (L676-681). Fix: `p.foreground` (or keep — visually minor; the docs demo shows gray). Nova
  `[&_svg]:size-4` matches rust 16px.
- Invalid group ring (nova `has-aria-invalid:ring-3 ... rounded-lg` on the GROUP): rust rings only
  the active slot; nova also rings the whole group when any slot is invalid. Fix: in
  `InputOTPGroup::show`, when `ctx.invalid`, paint a 3px `destructive/20` (dark /40) ring around the
  group rect (needs the group's min_rect; `inner.response.rect`).

**Not portable**
- `inputMode="numeric"` mobile keyboard (numeric filter covers intent).

**Confirmed OK**
- Active slot `border-ring` + 3px `ring-ring/50` (invalid → destructive alphas), `border-y border-r
  border-input` with first `border-l`, per-group first/last corners, caret 1px×16px `bg-foreground`
  blinking at 1Hz, digit/backspace/arrows/paste keyboard model with focus advance, numeric filter,
  disabled sense=hover, text-sm glyphs, group/separator composition API.

### native-select — medium
Rust: `src/components/native_select.rs`. Original: `native-select.tsx`; nova L899-906.

**P0**
- Nova trigger geometry: base `h-9 rounded-md px-3 pr-9`, sm `h-8 py-1`; nova `h-8 rounded-lg py-1
  pr-8 pl-2.5`, sm `h-7 rounded-[min(var(--radius-md),10px)] py-0.5`. Rust: heights 36/32
  (`NativeSelectSize::height` L42-47), radius md (L285), text x = `rect.min.x + MD` (12, L311-313),
  chevron center `rect.max.x - 14 - 8` (right-3.5, L318-320). Fix style-conditional: heights
  32/28; radius `lg` (nova); pl 10; chevron right offset 10 (`rect.max.x - 10 - 8`); sm radius
  `min(radius.md,10) = 8`.
- Nova width padding: width calc `widest + 36.0 + 12.0` (pr-9 + pl-3, L262-264) → nova
  `widest + 32.0 + 10.0`.

**P1**
- Disabled fade: wrapper `has-[select:disabled]:opacity-50` fades border too. Rust fades text and
  chevron only (L121-125, L326-330). Fix: 0.5 alpha on stroke (and keep dark fill).

**P2**
- Option rows are the port's own popover design (the native browser list can't be styled in egui) —
  currently accent-filled selected/hover rows, popover bg, shadow-md: reasonable; optionally align
  row height with select-item (`py-1` + rounded-md rows) rather than full 36px rows.
- `placeholder:text-muted-foreground` implemented via `has_selection` (L121-125) — good.

**Not portable**
- `selection:`, native `<option>`/`<optgroup>` canvas colors — popup rows replace them.

**Confirmed OK**
- `dark:bg-input/30 dark:hover:bg-input/50` hover fill (exact, L100-108), focus ring 3px
  `ring-ring/50` + border-ring, invalid border+ring destructive alphas, chevron-down
  `muted-foreground` at 50% opacity, disabled option rows at 0.5, popup close-on-click, sm size.

### field — medium
Rust: `src/components/field.rs`. Original: `field.tsx`; nova L618-665.

**P0**
- Nova gaps (all style-conditional; base values currently hardcoded):
  - `Field` gap: base `gap-3` (rust 12, L161) → nova `gap-2` (8).
  - `FieldContent` gap: base `gap-1.5` (rust 6, L537) → nova `gap-0.5` (2).
  - `FieldGroup` gap: base `gap-7` (rust 28, L568-570) → nova `gap-5` (20).
  - `FieldSet` gap: base `gap-6` (rust 24, L600) → nova `gap-4` (16).
  - `FieldLegend` mb: base `mb-3` (rust 12, L664) → nova `mb-1.5` (6).
- Missing composition: the source `FieldLabel` can WRAP a nested `Field` (the choice-card pattern):
  `has-[>[data-slot=field]]:rounded-md border`, `[&>*]:data-[slot=field]:p-4` (base) / nova
  `rounded-lg border *:data-[slot=field]:p-2.5 hover:bg-muted/50 has-[:focus-visible]:border-ring
  ring-3 ring-ring/50` + `has-data-[state=checked]:bg-primary/5 border-primary/30` (dark
  `bg-primary/10 border-primary/20`) + `group-disabled:opacity-50`. Rust `FieldLabel` is text-only
  (L181-267). Fix: add `FieldLabel::show_with_card(ui, |ui| { ... })` (or a `card: bool` builder)
  that paints a bordered rounded container (radius md base / lg nova, 1px `p.border`,
  padding 16 base / 10 nova per child field), hover `bg-muted/50`, and when the wrapped control is
  checked (expose via a `checked: Option<bool>` builder) fill `color_alpha(p.primary, 0.05)`
  (dark 0.10) + border `color_alpha(p.primary, 0.3)` (dark 0.2).

**P1**
- Nova field-label/legend/title font-medium (cross-cutting (b)) — no egui medium face; foundation.

**P2**
- `FieldSeparator` height: source `h-5` (20px row, `-my-2`); rust uses 16 (`spacing::LG`, L701).
  Fix: 20.0. Chip `bg-background px-2 text-muted-foreground` implemented (L714-731) — good; nova
  matches (text-muted-foreground px-2).
- `FieldTitle` height factor 1.3 (L328) vs `leading-snug` 1.375 — cosmetic.
- `FieldError` bullet list uses "• " glyph + 16px indent — matches `ml-4 list-disc gap-1`; the
  `gap-1` (4px) between rows implemented (L505). `nth-last-2:-mt-1` / `[legend]+&:-mt-1.5`
  negative-margin tightening not implemented — add `-6.0` spacing before a description that follows
  a legend if trivially possible; else note.
- `FieldDescription` when field disabled: rust fades to 0.5 (L383-387) — source does not fade
  descriptions (only label/title have `group-data-disabled:opacity-50`)... actually source has no
  disabled rule on description; remove the fade to match, or keep (minor).
- Demo gaps: no `responsive` orientation, `FieldLegendVariant::Label`, or disabled-field row
  (demo L1255-1337). Add one row each.

**Not portable**
- `@container/field-group` responsive variants beyond the `@md` width check (rust approximates with
  `available_width() >= 448`, fine); `<a>` link styling inside descriptions (render as plain text).

**Confirmed OK**
- Orientation vertical/horizontal/responsive (with `@md`=448 threshold, unit-tested), invalid →
  destructive label/title via field ctx, disabled → 0.5, description wrapping, error
  single/list/empty-render-nothing, separator with centered chip, legend variants, ctx stack.

### input-group — medium
Rust: `src/components/input_group.rs`. Original: `input-group.tsx`; nova L1396-1447.

**P0**
- Nova container geometry: base `h-9 rounded-md` vs nova `h-8 rounded-lg`. Rust hint height
  `spacing::CONTROL` (36, L243) and radius `md_u8()` (L257, L303). Fix style-conditional: 32.0 /
  `lg_u8()` under nova.
- Focus border color: base+nova `has-[control:focus-visible]:border-ring`; rust stroke stays
  `p.input` when focused (L319-324) and only paints the ring (L329-337). Fix:
  `stroke = if invalid { destructive } else if record.focus { p.ring } else { p.input }`.
- Disabled container: nova/base `has-disabled:bg-input/50 dark:has-disabled:bg-input/80
  has-disabled:opacity-50`. Rust paints only the dark `input/30` fill (L263-269) and fades child
  text. Fix: when `self.disabled`, fill `color_alpha(p.input, if dark {0.8} else {0.5})` (light
  too) and paint the border at 0.5 alpha.

**P1**
- Addon paddings (style-conditional): inline-start `pl-3` (rust 12, L637) → nova `pl-2` (8);
  inline-end `pr-3` (L648) → nova `pr-2` (8); block-start/end `px-3 pt-3/pb-3` (rust 10/10,
  L668-681) → nova `px-2.5 pt-2/pb-2` (10/8).
- Input side padding with inline-end addon: base `[&>input]:pr-2` (nova `pr-1.5`=6); rust right
  margin is always 12 (L425). Fix: `right_pad = if record.inline_end { style-nova?6.0:8.0 } else {12.0}`.
- Inline-start input padding already conditional (L419-424, 8) — matches base `pl-2`; nova `pl-1.5`
  (6) — style-conditional.

**P2**
- Addon click-to-focus: source addon onClick focuses the input; rust approximates (doc comment
  L594-596). Fix: give the addon `Sense::click()` and on click `request_focus` the group's input id
  (record the input id in `GroupRecord` when `InputGroupInput` renders).
- Button negative margins `has-[>button]:ml-[-0.45rem]` (nova -0.3rem) not implemented — cosmetic.
- `[&>kbd]:rounded-[calc(var(--radius)-5px)]` (nova -3px): pass a kbd radius override if Kbd
  supports it; else skip.
- `InputGroupButtonSize::Sm` delegates to nova `ButtonSize::Sm` (h-7 under nova) but the
  input-group sm spec is `h-8 rounded-md px-2.5` in both styles (nova doesn't override it). Fix:
  map `InputGroupButtonSize::Sm` to an explicit 32px-tall ghost Button (or special-case height).
- Textarea in group: rust margin 12/12 (`py-3`, L566-569); nova `py-2` (8) — style-conditional.

**Not portable**
- `has-[]:` reactive selectors settle one frame later in egui (documented in module docs; the
  snapshot mechanism is the right call).

**Confirmed OK**
- 4 addon alignments with column switch for block/textarea (`h-auto flex-col`), `flex-1` input
  sizing via measured end-addon width, invalid container border + ring destructive alphas (dark
  /40), focus ring 3px `ring-ring/50` from any control, addon text/icon `muted-foreground` `gap-2`
  `text-sm`, `InputGroupText`/`InputGroupButton` (xs/icon-xs/icon-sm geometry), kbd hint demo,
  ButtonGroup fusing, frame-lag handled via snapshot.

---

## Batch summary

All 12 components implement the **base** recipe faithfully with correct tokens, focus rings,
invalid/disabled states and keyboard behavior; none branches on `theme.style`, so every one deviates
from the **nova** reference recipe (the systemic gap; the `alert.rs` style-conditional pattern is the
proven fix shape). No component is a rewrite: everything is parameter/geometry-level work.

Ranking by work needed (most → least):
1. **toggle** — nova changes height/padding/gap/radius/pressed-fill/hover-fg/outline-shadow and the
   sm size gains its own radius+font size.
2. **field** — five nova gap tokens plus the genuinely missing nested `FieldLabel`-wraps-`Field`
   card composition (new API surface), separator height.
3. **slider** — nova thumb 12px + `border-ring`, track 4px, ring 3px, plus the missing vertical
   orientation (new behavior).
4. **toggle-group** — inherits all toggle fixes; adds fused `px-2` and fused outline shadow removal.
5. **input-group** — nova h-8/lg radius, focus border color, disabled container fill/opacity, four
   addon padding values, inline-end input padding, addon click-to-focus.
6. **native-select** — nova heights 32/28, radius, pl/pr/chevron offsets, width padding math,
   disabled border fade.
7. **input / textarea** — identical small set: nova h-8 (input), radius lg, px-2.5, disabled fill
   `bg-input/50` (dark /80) and disabled border fade.
8. **radio-group** — nova solid-primary checked disc with `primary-foreground` dot, gap-2, focus
   border, invalid+checked border rule.
9. **input-otp** — nova 32px slots, lg corners, disabled fade, optional invalid group ring.
10. **checkbox** — one P1 (focus border-ring); everything else matches (incl. the nova-only
    invalid+checked rule).
11. **switch** — only the focus border-ring stroke; sizes/colors already exact for both styles.

Demo/PROGRESS nits: checkbox demo reuses `values[4]` for both the disabled and invalid rows
(`examples/gallery/demos/mod.rs:731-743`) and the radio demo reuses `values[0]` across three groups
(L791-802) — give each its own slot; radio demo hint says "wrapping" but the roving nav does not
wrap (L822) — reword. `PROGRESS.md` rows 17-28 are accurate but none mentions the nova-recipe gap;
add "nova recipe pending (style-conditional)" to each B row after these fixes land.
