# Batch G gap analysis — calendar, date-picker, carousel, chart, data-table, form, sidebar

Reference recipe: nova (`apps/v4/registry/styles/style-nova.css`) replaces base classes where it
defines a property. Important crate-wide context found during this pass: `button.rs::resolve_colors`
(src/components/button.rs:142) implements the **base** button recipe (`hover:bg-accent`,
`dark:hover:bg-accent/50`, outline `dark:bg-input/30`), not nova (`hover:bg-muted`, outline hover
`bg-muted`). Where the components below hand-roll button-like painting, deviations are listed
against the crate's own Button so a central nova reconciliation can land in one pass.

### calendar — medium
**P0 (visual/API wrong — must fix)**
- Missing 8 px gap between week rows. Original calendar.tsx:97 `week: cn("mt-2 flex w-full", …)`
  puts an 8 px margin above every week row (including between the weekday header and the first
  week). The Rust grid draws rows back-to-back (calendar.rs:626-685 paints at `row * cell`), so a
  6-row month is 48 px too short and the range "bar" looks like one solid block instead of
  per-row bars. Fix: add `const WEEK_GAP: f32 = 8.0;` next to `WEEKDAY_ROW_HEIGHT` (calendar.rs:52);
  in `show()` (calendar.rs:547,619,630) compute `grid_origin.y = weekday_y + WEEKDAY_ROW_HEIGHT + WEEK_GAP`,
  per-row y `= grid_origin.y + row as f32 * (cell + WEEK_GAP)`, and `content_h += rows as f32 * WEEK_GAP`.
  Range-bar painting needs no change (bars stay per-row, matching the source).
- `TABLE_HEAD_HEIGHT = 36.0` (src/components/table.rs:51, used by data_table.rs) contradicts its own
  comment "`TableHead` height (`h-10`)" — h-10 is 40 px (original table.tsx:72 `h-10 px-2`). Fix in
  table.rs (batch C file; coordinate centrally): `pub const TABLE_HEAD_HEIGHT: f32 = 40.0;` and adjust
  data_table.rs:523-531 header cells to the new height.

**P1 (behavior/interaction gap)**
- No `captionLayout="dropdown"` month/year pickers. Original calendar.tsx:76-90 styles
  `dropdowns` (`h-(--cell-size)` row, `gap-1.5 text-sm font-medium`), `dropdown_root`
  (`rounded-md border border-input shadow-xs has-focus:border-ring has-focus:ring-[3px]
  has-focus:ring-ring/50`; nova `.cn-calendar-dropdown-root` same), `caption_label` for dropdown
  layout (`flex h-8 items-center gap-1 rounded-md pr-1 pl-2 text-sm [&>svg]:size-3.5
  [&>svg]:text-muted-foreground`; nova `h-6 pr-1 pl-1.5`) and ChevronDown icons at `size-4`. Fix:
  add `pub enum CaptionLayout { Label, Dropdown }` + `.caption_layout(...)`; when `Dropdown`, render
  the caption row as two `dropdown_root`-styled buttons (month name, year) at 28 px height with
  muted chevron-down icons that open a `DropdownMenu` (crate already has one) of months 1-12 /
  years ±12; `formatMonthDropdown` = `"Jan".."Dec"` short names (calendar.tsx:43-44).
- No `showWeekNumber`. Original calendar.tsx:98-105 + WeekNumber component: a leading
  `w-(--cell-size)` column of `text-[0.8rem] text-muted-foreground` week numbers, and
  calendar.tsx:108-110 switches the range-start rounding hook from first-child to
  `[&:nth-child(2)]`. Fix: `show_week_numbers(bool)` builder; when on, prepend an extra 28 px column
  (grid width 224), paint ISO week number = `((day.epoch_days() + 4 - start_of_iso_year) / 7) + 1`
  centered in muted foreground, and move the range-start left rounding to the second column.
- Keyboard Home/End missing. react-day-picker v9 Home = first day of the focused week, End = last.
  Fix in calendar.rs:690-712: `Home` → `focused - ((focused.weekday() + 7 - week_start) % 7)`,
  `End` → `Home + 6` (epoch-day arithmetic, then the existing view-month/refocus block at 740-748).

**P2 (polish)**
- Nav-button hover is full `p.accent` in both modes (calendar.rs:783). The crate's ghost button is
  `accent` light / `accent/50` dark (button.rs:203-206; nova is `muted` light / `muted/50` dark).
  Fix `ghost_icon_button`: `fill = if dark { color_alpha(p.accent, 0.5) } else { p.accent }`.
- Ghost hover text: source DayButton/nav use `hover:text-accent-foreground` in light too. In
  `paint_day_cell` (calendar.rs:857-863) set `text = p.accent_foreground` for `CellRole::Hover`
  unconditionally (currently dark-only; zinc light values coincide so this is invisible today).
- Day cell font: `CalendarDayButton` is a Button (`text-sm font-medium` via `font-medium` ghost
  label; calendar.tsx:211 `font-normal` overrides it — `leading-none font-normal`). Rust uses SM
  normal ✓, but no `font-medium` exists for the caption (`caption_label font-medium`, calendar.rs:594
  paints plain SM). egui has no medium face (`.strong()` brightens) — leave as a known limitation.
- No `fromDate`/`toDate` bounds: nav buttons never disable. Add optional
  `.min_month(CalendarDate).max_month(CalendarDate)`; disable prev/next (render with
  `color_alpha(p.foreground, 0.5)` per `aria-disabled:opacity-50`, calendar.tsx:60,65) when the
  target month is out of bounds.
- `dark:hover:text-accent-foreground` on the DayButton (calendar.tsx:211) is covered by the Hover
  role; today-under-selected nuance (td `bg-accent` visible under the selected button's rounded
  corners) is not worth emulating — accept.

**Not portable**
- RTL icon mirroring (`rtl:**:[.rdp-button_next>svg]:rotate-180`, calendar.tsx:37-38): no RTL mode in
  egui; skip.
- CSS entry animations from react-day-picker (day focus transitions): sub-100 ms color transitions
  are out of scope for the painter approach (crate-wide).

**Confirmed OK**
- Nova recipe exact: `p-2` (8), `[--cell-size:--spacing(7)]` = 28 px cells, `--cell-radius:--radius-md`
  (style-nova.css:231-233) — CALENDAR_CELL_SIZE/CALENDAR_PADDING/CALENDAR_GRID_WIDTH (calendar.rs:42-46)
  match, tested at calendar.rs:1255-1260.
- Caption/nav layout: ghost 28 px chevron buttons at both edges (size-4 icons), centered
  `month_caption` (`h-(--cell-size)`, text-sm, calendar.rs:554-596), `month gap-4` = 16 px caption gap.
- Weekday header: `flex-1 rounded-md text-[0.8rem] text-muted-foreground` = 28 px columns, 12.8 px
  muted text (calendar.rs:598-616).
- Day cell states: single selected = `bg-primary`/`primary-foreground` rounded-md; today = `bg-accent`
  rounded-md (`today: rounded-md bg-accent text-accent-foreground`, calendar.tsx:119-122); range start/
  end = primary square over a full-accent row cell with `rounded-l-md`/`rounded-r-md` (calendar.tsx:113-118,
  211) — the Rust two-pass paint (accent bar behind, primary square on top, calendar.rs:812-841) is a
  faithful equivalent; range middle = `bg-accent rounded-none` + `accent-foreground`; outside =
  `text-muted-foreground` (still primary when selected — role overrides outside text, ✓ calendar.rs:805-810);
  disabled = `text-muted-foreground opacity-50` with no hover fill (calendar.rs:867-871).
- Focus ring: focused day gets 3 px `ring-ring/50` (calendar.rs:675-682) matching
  `group-data-[focused=true]/day:ring-[3px] ring-ring/50`.
- Keyboard: arrows ±1 day / ±7 days, Enter/Space select, PageUp/PageDown month, Shift = year
  (calendar.rs:690-748) — matches rdp v9; view month auto-flips and focus re-requests.
- Selection semantics: single toggle-off, multiple sorted toggle, range pending-start/restart
  (calendar.rs:277-324) match react-day-picker; unit-tested.
- Date math pure-Rust, leap/rollover/epoch tests pass (calendar.rs:889-1086).

### date-picker — medium
**P0 (visual/API wrong — must fix)**
- Trigger fill diverges from the outline Button in dark mode. Original demo button is `variant
  "outline"` (date-picker-demo.tsx:24) = `dark:bg-input/30 dark:hover:bg-input/50`; the crate's own
  Button paints exactly that (button.rs:180-193). date_picker.rs hand-rolls the trigger and paints
  `p.background` in both modes (date_picker.rs:203-212). Fix: `fill = if dark {
  color_alpha(p.input, 0.3) } else { p.background }`, hover dark `color_alpha(p.input, 0.5)`
  (keep `p.accent` light hover to match button.rs base recipe). Better: refactor the trigger to
  compose `Button::new(label).variant(Outline).width(240.0)` once Button supports icons+text.

**P1 (behavior/interaction gap)**
- No range mode or presets variant. Source ships date-picker-with-range.tsx and
  date-picker-with-presets.tsx (popover containing a Select of "Today/Tomorrow/In 3 days/In a week"
  above a bordered calendar, `flex w-auto flex-col space-y-2 p-2`). Fix: add
  `DatePicker::range(&mut Option<(CalendarDate, CalendarDate)>)` passing through to
  `Calendar::range` (do NOT close on pick — the source stays open until both ends are chosen), and a
  `.with_presets(vec![...])` that renders the `Select` (Sm size) + a `rounded-md border` frame
  around the calendar inside a p-2 popover.

**P2 (polish)**
- Disabled trigger: original keeps the full outline at `disabled:opacity-50` (whole button dimmed);
  date_picker.rs dims only via `color_alpha(fill, 0.5)` on background — with the P0 fill fix, apply
  the alpha to stroke and icon/text too (stroke already alpha'd, date_picker.rs:213-218 ✓; icon/text
  ✓). No further change needed after P0.
- Trigger radius: hand-rolled `radius_md` (date_picker.rs:181) — base button is `rounded-md` ✓ under
  the crate's base recipe; if the central nova pass lands (`rounded-lg`), switch to
  `theme.radius.lg_u8()` in the same change.
- Demo: add a "with presets" section once P1 lands; the current demo covers default/preselected/
  disabled/options — good coverage of the existing surface.

**Not portable**
- `format(date, "PPP")` locale variants: only the en-US shape is implemented
  (`format_ppp`, calendar.rs:232-234) — acceptable, document.

**Confirmed OK**
- Trigger geometry: `w-[240px]` (DATE_PICKER_WIDTH=240), `h-9` (CONTROL=36), `px-4` + `gap-2` +
  size-4 CalendarIcon left-aligned with `font-normal` SM text (date_picker.rs:35-48,219-244) ✓.
- Muted placeholder + `text-muted-foreground` when empty (date_picker.rs:186-190) ✓ matches
  `!date && "text-muted-foreground"`.
- Popover: `w-auto p-0 align=start` → width = 212 (196 + 2×8), inner_margin 0, BOTTOM_START anchor,
  sideOffset 4 + slide-in-from-top-2 via overlay_progress (date_picker.rs:270-303); frame uses the
  same nova popover skin as popover.rs (`bg-popover`, `ring-foreground/10`, rounded-lg, shadow-md,
  date_picker.rs:283-288) ✓.
- Closes on pick / Escape / click-outside (date_picker.rs:330-355); fresh calendar per open via
  `open_count` salt (date_picker.rs:309-314); `initialFocus` on the selected/today day
  (date_picker.rs:531-536); controlled date; all tested (date_picker.rs:518-627).

### carousel — medium
**P0 (visual/API wrong — must fix)**
- Arrow button dark fill wrong (same base-vs-crate issue as date-picker). Original CarouselPrevious/
  Next = outline Button (carousel.tsx:174-202); base outline = `dark:bg-input/30`, hover
  `dark:bg-input/50`. carousel.rs:552 paints `p.background` always and strokes `p.input` in light
  (light outline border is `p.border`). Fix in `arrow_button`: fill light `p.background` / dark
  `color_alpha(p.input, 0.3)`; hover dark `color_alpha(p.input, 0.5)` (hover light `p.accent` ✓ base);
  stroke `if dark { p.input } else { p.border }`.

**P1 (behavior/interaction gap)**
- No multi-slide-per-view (`basis-1/2`, `basis-1/3`), the single biggest API gap — every source demo
  uses it (carousel-size.tsx `md:basis-1/2 lg:basis-1/3`; carousel-orientation.tsx `md:basis-1/2` with
  a fixed 200 px viewport and `pt-1` gutters; carousel-spacing.tsx `-ml-1 … pl-1` = 4 px gutters).
  Fix: add `.per_view(usize)` (1 default) and `.item_gap(f32)` (16 default) to Carousel; in
  CarouselItem (carousel.rs:622-676) divide content width by per_view and subtract the gap; pitch =
  `(viewport_extent - gap) / per_view + gap` per step; max_offset uses the pitch array from
  `slide_offsets` (already generic if `slide_width` becomes per-item widths). Also honor
  `align: start` (the demos' option) — current fixed-pitch offsets are align-start ✓.
- The rail/keyboard model: source handles ArrowLeft/ArrowRight on the root div via
  `onKeyDownCapture` (carousel.tsx:78-89,121) — i.e. whenever focus is anywhere inside the carousel,
  not only on the viewport. Fix: `ui.interact(viewport…, Sense::click_and_drag())` at carousel.rs:306
  should also claim focus on click (`viewport_response.request_focus()`), and the key check at
  carousel.rs:395 should test focus anywhere within the root rect (read `ctx.memory().focused()`
  rect ⊆ root_rect) instead of only the viewport response.

**P2 (polish)**
- Dots are an invention (no dots in carousel.tsx or any source demo; the api demo shows
  "Slide x of y" text, carousel-api.tsx:72-75). Keep them as an extension but (a) match demo spacing
  if kept: gap-1.5 = 6 px between dots (carousel.rs:425,435 use 16 px) and `py-2` above the row, and
  (b) consider defaulting `show_dots(false)` to match the source surface. The demo's "Slide x of y"
  label should use `text-sm text-muted-foreground` centered (`py-2 text-center`,
  carousel-api.tsx:73) — currently a plain left-aligned label (demos/mod.rs:3552-3555).
- `max_offset` is called with `viewport.height()` as the scroll-axis extent for horizontal
  carousels (carousel.rs:285) — works only because `min(last, …)` clamps; pass `viewport.width()`
  for horizontal / `viewport.height()` for vertical so a future tall-narrow horizontal viewport
  can't clip the last slide.
- Disabled arrow: base button is whole-button `disabled:opacity-50` (fill included). Rust keeps the
  full `background` circle and only dims stroke/icon (carousel.rs:536-557). Apply `alpha` to the
  fill as well: `color_alpha(fill, 0.5)` when disabled.
- Vertical orientation: source demo fixes the viewport height (`h-[200px]`) so basis-1/2 slides show
  two at a time; the Rust viewport = tallest slide (one at a time). This resolves naturally with the
  multi-per-view work (P1); optionally add `.viewport_height(f32)` for the vertical case.

**Not portable**
- embla's momentum/velocity fling on release: the Rust snap uses smoothstep over 0.25 s
  (carousel.rs:44-46,110-117) — no inertia. Acceptable; document.
- `plugins` (autoplay, etc.): out of scope for a pure-Rust port; no recharts-equivalent needed here.

**Confirmed OK**
- Slide strip math: `basis-full pl-4` = 16 px gutters (CAROUSEL_ITEM_GAP=16), `-ml-4` strip offset so
  slide 0 content lands flush (carousel.rs:327-344,636-661) ✓; pitch = viewport width ✓.
- Arrows: `size-8 rounded-full` (32 px circle, 16 px icons) perched at `-left-12`/`-right-12`
  (48 px outset, vertically centered; above/below for vertical with up/down icons replacing the
  source's rotate-90, carousel.rs:478-512) ✓; disabled at bounds ✓; snap-on-release to nearest slide
  ✓ (tested carousel.rs:854-890).
- Dots: active `bg-primary`, inactive `bg-muted/50` (carousel.rs:443-449), click-to-jump ✓.
- Keyboard: ArrowLeft/Right (horizontal), ArrowUp/Down (vertical) move one slide while focused
  (carousel.rs:395-419) ✓; `role="region"`/`aria-roledescription` equivalents documented.
- Viewport height = tallest slide with a stable 120 px first-frame fallback (carousel.rs:42-43,356-359) ✓.

### chart — medium
**P0 (visual/API wrong — must fix)**
- Tooltip default indicator: source default is `"dot"` (chart.tsx:118 `indicator = "dot"`);
  chart.rs:335 defaults to `ChartIndicator::Line`. Fix: `tooltip_indicator: ChartIndicator::Dot`.
- Dashed indicator shape is wrong. Source (chart.tsx:228-231) is `w-0 border-[1.5px] border-dashed
  bg-transparent` stretched to row height — a **vertical dashed line** (1.5 px dashes, row-height,
  3 px total width with left+right borders), not a dashed square outline. chart.rs:889-922 draws a
  dashed square. Fix: paint 3 vertical dashed strokes at x, x+1.5, x+3 spanning the row height
  (or one 3 px-wide dashed rect outline collapsed to the vertical pair), `Stroke::new(1.5, color)`.
- Tooltip shadow: source `shadow-xl` (chart.tsx:193; nova `.cn-chart-tooltip` same);
  chart.rs:837 uses `shadow::shadow_lg`. Fix: add `shadow_xl(dark)` to utils/shadow.rs (≈ shadow_lg
  with offset y 4→6, blur 15→25, alpha ×1.15) and use it here. (Batch E already floated central
  shadow/token additions — coordinate.)

**P1 (behavior/interaction gap)**
- No curve smoothing. Source demos use recharts default `type="monotone"` for Area/Line (and
  PROGRESS notes the gap). Fix: add `.smooth(bool)` (default true for Area/Line, matching recharts);
  implement Catmull-Rom → cubic Bézier on the pts array before `egui::Shape::cubic_bezier` segments
  (tension 0.5 standard monotone approximation); keep straight lines for `.smooth(false)`.
- Tooltip nestLabel: source shows the label row nested inside the single-series row when
  `payload.length === 1 && indicator !== "dot"` (chart.tsx:188,249). chart.rs ignores `nest_label`.
  Fix in draw_tooltip: `let nest = visible.len() == 1 && indicator != Dot;` when nest, skip the
  standalone label row and render the label under the series name (muted → foreground label text,
  6 px gap). Also `.my-0.5` on the dashed indicator then applies (2 px vertical inset).
- Legend value colors: legend dot uses `item.color` = series color ✓, but hidden-series dimming is
  an invention with no source counterpart — fine to keep, but the demo text implies legend toggling
  is a source feature; soften the demo caption to "(gallery extension)".

**P2 (polish)**
- `ChartContainer` draws its own rounded-lg border + p-2 (chart.rs:202-211), but the source
  ChartContainer is border-less (`flex aspect-video justify-center text-xs`, chart.tsx:67) — the
  border in the docs comes from wrapping in a Card. Options: (a) drop the border/padding and let the
  demo wrap in `Card`, or (b) keep and document as gallery framing. Prefer (a) for parity: remove
  `.corner_radius/.stroke/.inner_margin` from ChartContainer::show, default `min_height` stays 200.
- Aspect-video default: source container is `aspect-video` unless overridden by `min-h-[200px]`
  className (all demos pass min-h). PROGRESS documents the fixed min-height — acceptable, note only.
- Hover dots radius 3.5 (chart.rs:640-645) vs recharts `activeDot r=4-5` in the docs examples —
  bump to 4.0.
- Bar hover cursor band should sit behind bars (currently painted after series shapes? No —
  cursor_idx set after series_idx, so it paints on top, chart.rs:616-649; recharts draws the cursor
  behind). Fix: push the cursor shape before the series shapes (add a second Noop slot before
  series_idx and set it earlier).
- `format_value` groups with commas ✓ but locale variants (`toLocaleString`) are en-US only — fine.

**Not portable**
- Recharts animation entry (line draw / bar grow): skip, consistent with crate-wide no-entry-anim.
- `ChartStyle` CSS-var injection: replaced by palette `chart-1..5` tokens ✓ (chart.rs:279-287).
- Polar/radial charts, pie: not in the ported surface; PROGRESS should keep noting Area/Line/Bar only.

**Confirmed OK**
- Grid: horizontal-only lines at `stroke-border/50` (chart.rs:499-505) = 
  `[&_.recharts-cartesian-grid_line]:stroke-border/50`; y ticks muted `text-xs` = 12 px
  (chart.rs:467-481); x labels centered under bands with skip-when-cramped logic ✓.
- Bars: `barCategoryGap 10%` → usable = band×0.8, `barGap 4` px between grouped bars, radius 4 top
  corners (chart.rs:549-577) matching chart-bar-demo.tsx `radius={4}` ✓.
- Area fill: `fillOpacity` honored (default 0.4 per docs example; visitors card 0.15) with 2 px
  stroke ✓.
- Tooltip chrome: `min-w-[8rem]` 128 px, `rounded-lg`, `border-border/50`, `bg-background`,
  `px-2.5 py-1.5` (10/6), `text-xs`, gap-1.5 rows, dot indicator 10×10 rounded-2, line indicator
  4 px × row height, value in mono foreground, name muted (chart.rs:771-941) — matches
  chart.tsx:193-261 + nova `.cn-chart-tooltip` except the P0 items.
- Tooltip flip/clamp at screen edges ✓; Order::Tooltip non-interactable ✓.
- Legend: centered row, pt-3 (12 px), gap-4 (16 px), 8×8 rounded-[2px] dot + gap-1.5 + label
  (chart.rs:683-757) = ChartLegendContent ✓.
- nice-bounds/y-ticks/thousands formatting unit-tested; light+dark render tests; legend toggle
  persists per id_salt ✓.

### data-table — medium
**P0 (visual/API wrong — must fix)**
- Header row height 36 px instead of `h-10` = 40 px (shared root cause in table.rs:51 — see the
  calendar section; the fix lands in table.rs, data_table.rs:523-531 inherits it).

**P1 (behavior/interaction gap)**
- Column-visibility dropdown missing (toolbar right side). Original data-table-demo.tsx:213-240:
  outline Button `Columns <ChevronDown>` (`ml-auto`) opening a DropdownMenu with
  `DropdownMenuCheckboxItem` per hideable column (`capitalize`, checked = visible, align end).
  Feasible — the crate has `CheckboxItem::new(label, &mut bool)` + `MenuBuilder::checkbox_item`
  (dropdown_menu.rs:187,327). Fix: add
  `.column_visibility(&mut Vec<bool>)` builder (parallel to `columns`, index-matched; `enableHiding
  = false` for selection/actions columns — expose `.hideable(bool)` on DataTableColumn defaulting
  true for data columns, false for actions). Render the toolbar row as
  `ui.horizontal(|ui| { Input…; Button::new("Columns").variant(Outline).icon_end(Icon::ChevronDown)
  .show(ui); })` with the input pushed left and the button right-aligned
  (`ui.with_layout(right_to_left, …)` for the button). Filter the column list by
  `visibility[ci]` when computing `widths`/rendering cells (data_table.rs:476-533,554-597) and skip
  hidden columns in the body/header.
- Sort header affordance: the source wraps the header label in a **ghost Button** (whole
  label+icon clickable, `hover:bg-accent`, data-table-demo.tsx:160-172). data_table.rs:714-775 makes
  only the icon (±6 px pad) the hit target with no hover fill. Fix: make the entire header cell
  (text + icon rect) clickable via `ui.interact(Rect…Sense::click())` on the full 40 px-height cell
  and paint `p.accent` (crate base ghost) behind it on hover; keep the icon swap
  ArrowUp/ArrowDown/ArrowUpDown.

**P2 (polish)**
- Footer buttons: base outline `rounded-md` — data_table.rs:872-884 uses `theme.radius.sm_u8()`;
  switch to `theme.radius.md_u8()` (nova pass would want `lg_u8`; coordinate). Disabled state: base
  = whole button at `opacity-50` (border `input`, fill unchanged); data_table.rs:864-884 swaps
  border to `p.border` and keeps full fill — use `color_alpha(p.input, 0.5)` stroke +
  `color_alpha(fill, 0.5)` fill when disabled.
- "Rows per page" select: source pagination example uses `h-8 w-[70px]` trigger with
  `text-sm` label; the Rust uses SelectSize::Sm (h-7). Bump to default size (h-8) keeping width 70.
- Amount cell: demo uses `RichText::strong()` for `font-medium` — `.strong()` only brightens
  (crate-wide limitation, sibling note); acceptable, note in PROGRESS.
- The demo's second table uses a fresh `filter2` String each frame, so typed filters reset
  per-frame state oddly; hoist into DemoState (demos/mod.rs-style store, data_table_demo.rs:138).

**Not portable**
- TanStack virtualization / column resizing drag handles: out of scope (no resizable columns in the
  docs demo); PROGRESS already scopes the port to the demo surface.
- Keyboard row navigation (arrow-key row focus from the aria grid pattern): egui has no grid
  roving-focus primitive; skip.

**Confirmed OK**
- Row model pipeline filter → sort → paginate with `autoResetPageIndex` on filter change
  (data_table.rs:438-474) ✓; sort asc/desc toggle per click starting asc ✓; `sortFn_text`
  case-sensitive, numeric NaN-safe (data_table.rs:83-94, tested) ✓.
- Selection: header select-all with indeterminate state, page-scoped toggle, selected row
  `bg-muted` vs hover `bg-muted/50` via `row_bg` (data_table.rs:558-586; table.rs:40-47) = 
  `data-[state=selected]:bg-muted hover:bg-muted/50` (table.tsx:59) ✓.
- Toolbar input `max-w-sm` = 384 px with "Filter emails..." placeholder ✓; footer
  "x of y row(s) selected." `text-sm text-muted-foreground` ✓ (data_table.rs:622-652).
- `overflow-hidden rounded-md border` wrapper stroke (data_table.rs:601-614) ✓; row `border-b`
  dropped on last row ✓; "No results." h-24 (96 px) centered ✓ (data_table.rs:536-552).
- Row-actions dropdown cell right-aligned with the source's Actions label / Copy payment ID /
  separator / View items ✓ (demo); page-size select + Page x of y + first/prev/next/last is a
  documented superset of the classic pagination example ✓.
- Cell padding p-2 (8 px) via TABLE_CELL_PADDING ✓; selection/actions fixed widths reasonable.

### form — medium
**P0 (visual/API wrong — must fix)**
- None. Visual slots (label `text-sm font-medium` + destructive-on-error via
  `Field::new().invalid(ctx.invalid)`, description `text-sm text-muted-foreground`, message
  `text-sm text-destructive`, gap-2 stacking) are all delegated to field.rs (batch C) and wired in
  form.rs correctly — `FormLabel data-[error=true]:text-destructive` (form.tsx:100) is covered by
  FieldLabel reading the Field's invalid ctx (field.rs:219-227).

**P1 (behavior/interaction gap)**
- No Enter-in-input submit (PROGRESS notes it). Native `<form>` submits on Enter in any text input
  (input-form.tsx demo). Fix: in `FormUi::field` (form.rs:253-284) record
  `self.field_ids.push(ctx.id)`; add a `submit_on_enter: bool` builder on Form (default true);
  after `content(ui, &mut form)` in `Form::show` (form.rs:200), check
  `ui.input(|i| i.key_pressed(Key::Enter))` AND `ui.memory(|m| m.focused())` ∈ `form.field_ids` →
  `form.submit_requested = true`. Guards: skip when the focused widget is multiline (textarea ids
  can be registered via a new `.enter_submits(false)` per-field override).

**P2 (polish)**
- No built-in value reset (PROGRESS documents "caller owns it") — acceptable, but add a
  convenience `Form::reset_value_on_submit(false)` note or keep as-is; lowest priority.
- `FormFieldCtx.id` is synthesized per field_seq; ids are stable across frames only if field order
  is stable — fine for static forms; note in docs for dynamic forms.
- Submitting spinner: `Icon::LoaderCircle` is static; the source button `pending` state spins the
  loader — add `ui.painter()` rotation via `ctx.animate_value_with_time` on an angle, or accept
  (crate-wide animation helper could add a `spin_icon(painter, rect, icon, period)` in utils).

**Not portable**
- react-hook-form features beyond validate-on-submit/validate-on-change (nested field arrays,
  schema validation, async validators): out of scope for a closure-validator design — document.
- `aria-describedby` / `aria-invalid` attribute wiring: replaced by the `ctx.invalid` prop plumbed
  into Input `.invalid()`; the accessible description text can't be linked in egui — document.

**Confirmed OK**
- Validate-on-submit gating by a persisted touched flag; first submit reveals errors, later frames
  keep them; `validate_on_change` shows live errors (form.rs:92-94,182-228; tests 374-584) ✓.
- Submit re-validates the post-edit value and only runs `on_submit` when valid ✓ (tests).
- `submit_button` disables with form/submitting and renders the loader icon ✓;
  `reset_errors_button` clears the touched flag (form.rs:304-314) ✓.
- `disabled` flag reaches field closures and buttons ✓ (test at 682-708).
- Demo covers the docs bug-report composition (Card + FieldGroup + Input + Textarea + description +
  error slots + footer buttons), validate-on-change, and the submitting lock ✓.

### sidebar — medium
**P0 (visual/API wrong — must fix)**
- `SidebarMenuSub` content offset missing: source `mx-3.5 … px-2.5 py-0.5` (sidebar.tsx:646) means
  sub-menu content starts at 14 (mx-3.5) + 10 (px-2.5) = 24 px from the group edge; the Rust draws
  only the border line at +14 px and lays content at +0 px (sidebar.rs:1125-1133). Fix: wrap the
  content in `egui::Frame::new().inner_margin(Margin{ left: 24, top: 2, bottom: 2, right: 0 })`
  (or indent `ui` by 24 px), keeping the border at `cursor.left + 14 + 0.5` (sidebar.rs:1126) and
  the 2 px `py-0.5` vertical padding.

**P1 (behavior/interaction gap)**
- Rail dead when collapsed offcanvas: the content closure is skipped entirely at
  inner.width() ≤ 4 (sidebar.rs:336-357), so `SidebarRail` never renders and an offcanvas-collapsed
  sidebar can only re-open via `SidebarTrigger::with_salt` or Ctrl/Cmd+B. In the source the rail
  still works collapsed (`group-data-[collapsible=offcanvas]:translate-x-0 after:left-full`,
  sidebar.tsx:297-299). Fix: in `Sidebar::show`, when the content closure is skipped, still render
  a 16 px-wide clickable strip at the sidebar's edge (x = outer.left for side=Right, outer.right for
  side=Left, clamped to the screen edge) that calls `request_toggle`; draw the hover line in
  `sidebar_border` at the outer edge (after:left-full = at the very edge, sidebar.rs:1393-1402).
- `SidebarMenuAction` missing (source sidebar.tsx:548-578: absolute `top-1.5 right-1 aspect-square
  w-5` ghost icon button, hover `bg-sidebar-accent`, `showOnHover` fades `md:opacity-0` → 1 on row
  hover; sizes shift top offset per menu-button size). PROGRESS notes it; feasible: add
  `SidebarMenuAction::new(icon)` rendered inside `SidebarMenuItem` after the button — paint a 20 px
  hover-accent icon button at `rect.right() - 4 - 20, button_top(size)` with `show_on_hover(bool)`
  gating on the item row's hovered state; also add `SidebarGroupAction` (same pattern at
  `top-3.5 right-3 w-5`, sidebar.tsx:417-438) which PROGRESS omits entirely.
- `SidebarContent` is not scrollable (`overflow-auto`, sidebar.tsx:377). Fix: wrap in
  `egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, …)` inside SidebarContent::show
  (sidebar.rs:555-569) — the column's width is fixed so only vertical scrolling applies.

**P2 (polish)**
- `SidebarInset` lacks the `shadow-sm` of `md:peer-data-[variant=inset]:shadow-sm` (sidebar.tsx:313;
  nova `.cn-sidebar-inset` keeps it). Fix: after the bg fill (sidebar.rs:1469-1476) add
  `shadow::shadow_sm(dark)` behind the rounded-xl rect (same RectShape+blur pattern used by the
  floating sidebar at sidebar.rs:380-389).
- Trigger hover color: source is the plain ghost Button (`hover:bg-accent`), the Rust paints
  `sidebar_accent` (sidebar.rs:1311-1324). Use `p.accent`/`p.accent_foreground` (base recipe; nova
  pass would switch both to `muted`/`foreground`) and `theme.radius.md_u8()` instead of `sm_u8`
  (sidebar.rs:1312-1313; base button `rounded-md`).
- Floating/inset collapsed column width: source container = `icon + spacing(4) + 2px` = 66 px
  (sidebar.tsx:238) vs Rust `icon_width + 16` = 64 (sidebar.rs:298-301). Use `icon_width + 18.0`
  for Floating/Inset collapsed.
- Rail geometry: source rail straddles the sidebar edge (`-right-4 -translate-x-1/2` → spans
  edge-8 … edge+8) and the 2 px hover line sits at the edge; the Rust rail is entirely inside
  (edge-16 … edge, sidebar.rs:1379-1401) with the line 1 px inside. Fix: rect =
  `Rect::from_min_max(edge-8, edge+8)` clipped to available width, line at `edge`.
- Badge-as-text: `SidebarMenuBadge` (`h-5 min-w-5 rounded-md px-1 text-xs font-medium`,
  sidebar.tsx:588-594) is folded into MenuButton as plain right-aligned text (sidebar.rs:1037-1074).
  Since the source badge has no background, the visual difference is just min-width 20 px and
  4 px inset — set `badge_x = rect.right() - 4.0 - galley.x` (✓ already) and enforce
  `min_w = 20.0` by padding; acceptable as documented fold-in.
- `SidebarInput` (h-8 w-full bg-background shadow-none, sidebar.tsx:321-333) missing: add a
  `SidebarInput::new(&mut String)` returning `Input::new(s).width(full).height(32.0)` with no
  shadow (Input already takes width; verify shadow flag exists or paint without shadow_xs).
- `SidebarMenuSkeleton` missing (h-8 px-2 gap-2 with a size-4 icon skeleton + h-4 text skeleton,
  sidebar.tsx:602-638): trivial with the crate's Skeleton; add
  `SidebarMenuSkeleton::new().show_icon(bool)` with a per-id random width persisted in memory
  (50-90%).
- Escape-collapses-with-focus is an egui-friendly addition (source only has Ctrl/Cmd+B) — fine, but
  the demo/help text should keep calling it an extension.
- Group label uses CONTROL_SM (32 px) ✓ `h-8`; text `sidebar-foreground/70` at 0.7 alpha ✓ 12 px
  `text-xs` ✓; collapsed label rendering nothing ≈ source `-mt-8 opacity-0` ✓.

**Not portable**
- Mobile Sheet mode (`isMobile → Sheet w-(--sidebar-width-mobile)=18rem`, sidebar.tsx:183-206):
  egui has no viewport/media query; PROGRESS documents the gap. A desktop-width app could opt into
  `Sheet`-style presentation manually — keep as documented gap.
- Cookie persistence (`sidebar_state` cookie): replaced by egui memory per id_salt ✓ (equivalent).
- `transition-[left,right,width]` on the fixed container: the Rust animates the outer width only
  (sidebar.rs:306-309, 200 ms linear ✓ matches `duration-200 ease-linear`); the container itself
  doesn't slide for offcanvas — visually equivalent since width → 0 and content is clipped.

**Confirmed OK**
- Constants: SIDEBAR_WIDTH 256 (16rem), SIDEBAR_WIDTH_ICON 48 (3rem) ✓; animated width
  200 ms linear via animate_value_with_time = `transition-[width] duration-200 ease-linear` ✓.
- Variants: sidebar = `bg-sidebar` + outer-edge 1 px `sidebar_border` rule (border-r/border-l,
  sidebar.rs:403-413); floating = p-2 + rounded-lg + `ring-sidebar-border` 1 px + shadow-sm
  (sidebar.rs:324-401); inset pairs with SidebarInset (m-2 ml-0, ml-2 collapsed, rounded-xl
  bg-background, sidebar.rs:1448-1477) ✓.
- Collapsible: offcanvas → width 0 + content skipped; icon → 48 px rail with `size-8 p-2` buttons
  (CONTROL_SM=32 ✓), labels hidden, subs hidden (`group-data-[collapsible=icon]:hidden`) ✓; none =
  fixed ✓ (tests 1523-1812).
- Menu button: sizes h-8/h-7/h-12 (32/28/48) with text-sm/text-xs ✓; `data-active:bg-sidebar-accent
  + accent-foreground + font-medium(strong limitation)`; hover = accent bg; outline =
  `bg-background` + 1 px sidebar-border ring that swaps to sidebar-accent on hover
  (`shadow-[0_0_0_1px_var(--sidebar-border)]`) ✓ (resolve_menu_button_colors, tested 1629-1652);
  disabled 50% ✓; label truncation with ellipsis ✓.
- Sub menu: `gap-1` (4 px) between sub buttons, sub buttons h-7 px-2 with sm/md text sizes,
  active/hover accent, icons always `sidebar-accent-foreground` ✓ (sidebar.rs:1194-1249); border-l
  1 px at +14 px ✓ (missing only the P0 content inset).
- Trigger: ghost size-7 (28 px) PanelLeft icon ✓; `Ctrl/Cmd+B` toggle ✓; Escape-with-focus collapse
  (extension) ✓; Trigger::with_salt for out-of-closure use ✓; controlled + uncontrolled open state
  with `controlled_updated` sync ✓ (tests).
- Tooltip on collapsed buttons: instant (delayDuration 0), right of the rail, popover skin
  (rounded-md, bg-popover, border, shadow-md, px-2.5 py-1.5... wait — source TooltipContent is
  `bg-primary text-primary-foreground` (dark inverted tooltip per tooltip.tsx) — see note below).

> Note found while confirming: `show_rail_tooltip` (sidebar.rs:852-882) paints the collapsed-button
> tooltip with `bg-popover`/`popover_foreground`, but the source `Tooltip` component renders
> `bg-primary text-primary-foreground` (inverted) — that's a tooltip.rs (batch E) concern for the
> shared tooltip skin; flagging it here so the central pass reconciles the rail tooltip with
> tooltip.rs.

## Batch summary

All seven components are structurally sound, theme-token driven, unit-tested, and demoed; nothing
scores "low". Ranked by remaining work:

1. **sidebar (most work)** — one P0 layout bug (menu-sub inset), a dead rail when collapsed
   offcanvas (P1), two missing subcomponents (MenuAction/GroupAction), no content scrolling, plus a
   handful of P2 skin/geometry nits. The fixes are all local and concrete.
2. **data-table** — the missing column-visibility dropdown is the headline gap (all API pieces
   exist), the ghost-button sort header, and the shared TABLE_HEAD_HEIGHT 36→40 fix (table.rs,
   batch C).
3. **calendar** — the week-row 8 px gap (P0, changes overall rhythm and range-bar look) plus
   dropdown captionLayout / week numbers / Home-End keyboard, all feasible with the existing
   DropdownMenu and painter primitives.
4. **chart** — defaults (dot indicator), the dashed-indicator shape, shadow-xl, monotone smoothing
   and nestLabel; the ChartContainer border question needs a design call (border vs Card wrap).
5. **carousel** — multi-per-view is the big API lift (the PROGRESS-flagged gap); the dark-mode
   arrow fill and keyboard-while-focused model are small; dots need a "keep as extension" decision
   with demo spacing alignment.
6. **date-picker** — smallest surface: dark trigger fill (P0), range/presets variants (P1), demo
   addition. Most other behavior is already faithful and tested.
7. **form** — cleanest of the batch: one P1 (Enter-to-submit via focused-field check) and minor
   polish; visuals ride on field.rs (batch C).
