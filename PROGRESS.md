# PROGRESS — Porting shadcn/ui to Rust + egui

Source of truth for the port. Statuses: `⬜ todo`, `⏳ in progress`, `✅ done`, `⚠️ partial`.

## Scan summary

- Source: `original-shadcn-ui/apps/v4/registry/new-york-v4/ui/*.tsx` — **61 component files**.
- Styles found: `nova` (default), `luma`, `lyra`, `maia`, `mira`, `rhea`, `sera`, `vega` (`apps/v4/registry/styles/style-*.css`).
- Base colors: neutral, stone, zinc, mauve, olive, mist, taupe (`apps/v4/registry/themes.ts`, `base-colors.ts`).
- Theme (accent) colors: amber, blue, cyan, emerald, fuchsia, green, indigo, lime, orange, pink, purple, red, rose, sky, teal, violet, yellow — override `primary`, `secondary`, `chart-1..5`, `sidebar-primary` only.
- Theme tokens: OKLCH palette in `apps/v4/app/globals.css` + `themes.ts`. Radius: `--radius: 0.625rem`, sm = 0.6r, md = 0.8r, lg = r, xl = 1.4r, 2xl = 1.8r, 4xl = 2.6r.
- Derived component families with no dedicated `ui/*.tsx` (docs/examples only): `typography`, `date-picker`, `data-table`.

## Master checklist

| # | Component | Status | Batch | Commit | Notes |
|---|-----------|--------|-------|--------|-------|
| 1 | foundation: theme/tokens/colors | ✅ done | F0 | 4839425 | light+dark OKLCH palettes, radius/spacing/type scale |
| 2 | foundation: presets (styles + colors) | ✅ done | F0 | 4839425 | from_code("b0") round-trips 8 styles × 7 base colors × 17 accents, `Preset::from_code` |
| 3 | foundation: icons (lucide) | ✅ done | F0 | 0585bd9 | 36 icons, extendable pure-Rust Painter paths |
| 4 | foundation: utils (anim/focus/shadow/id) | ✅ done | F0 | afdafdd | |
| 5 | foundation: gallery skeleton | ✅ done | F0 | 87e5c64 | light/dark + preset pickers sidebar + light/dark + preset picker |
| 6 | button | ✅ done | A | ca7250a | variants: default/destructive/outline/secondary/ghost/link; sizes: xs..lg, icon-* |
| 7 | badge | ✅ done | A | 4bbdc59 | variants: default/secondary/destructive/outline/ghost/link |
| 8 | label | ✅ done | A | b29c2e0 | text-sm font-medium; for_id focus binding; disabled 50% (base recipe, no required dot in v4 source) |
| 9 | separator | ✅ done | A | 3d582dd | horizontal/vertical |
| 10 | kbd | ✅ done | A | 69d9bed | h-5 min-w-5 chip, sans font per source; KbdGroup gap-1 |
| 11 | spinner | ✅ done | A | 47b3ab0 | rotating lucide loader-circle arc, 1 rev/s; size builder (source is size-4) |
| 12 | skeleton | ✅ done | A | 91c8946 | animate-pulse cosine approx; nova fill bg-muted, base bg-accent |
| 13 | typography | ✅ done | A | 1ec0765 | h1..h4, p, blockquote, ul/ol, inline-code, code-block, lead, large, small, muted, table-caption |
| 14 | aspect-ratio | ✅ done | A | 03f3b3b | ratio container with child ui; max_width builder |
| 15 | marker | ✅ done | A | c99c5c5 | default/separator/border variants, MarkerIcon + MarkerContent |
| 16 | button-group | ✅ done | A | 5afcbf2 | horizontal/vertical, fused corners/borders via group plumbing; ButtonGroupText + ButtonGroupSeparator; nested groups lay out with 0 gap (no extra gap-2 like source) |
| 17 | input | ✅ done | B | d035ace | invalid/disabled/password, w-full default, ButtonGroup fusing |
| 18 | textarea | ✅ done | B | 4d1e95f | min-h-16, rows, placeholder, states |
| 19 | checkbox | ✅ done | B | 959c672+723d56a | checked/indeterminate (minus icon), disabled, invalid, Space+Enter toggle |
| 20 | radio-group | ✅ done | B | 09929bd | RadioGroup+Item, arrow roving focus via egui spatial nav (no wrap), disabled/invalid |
| 21 | switch | ✅ done | B | bb81ad0 | default+sm sizes, animated thumb, dark thumb tokens, Space toggle |
| 22 | slider | ✅ done | B | b144941 | single + range thumbs, step, arrow/home/end keys, track click, disabled |
| 23 | toggle | ✅ done | B | d4eee72 | default/outline variants, sm/default/lg sizes, icon+text, group fusing |
| 24 | toggle-group | ✅ done | B | 69d0eb7 | single (deselectable) + multiple modes, spacing 0 fusing via group protocol |
| 25 | input-otp | ✅ done | B | b65d9c9+eba4a8e | groups/separators, digit+backspace+arrows+paste, caret blink, numeric filter |
| 26 | native-select | ✅ done | B | 1f8bb2e | Popup-based dropdown, sm/default, chevron, hover/invalid/disabled, option rows |
| 27 | field | ✅ done | B | 31fb810 | Field/Label/Title/Description/Error/Content/Group/Set/Legend/Separator, horizontal+responsive |
| 28 | input-group | ✅ done | B | d4f1e2d | container+addons(4 aligns)+text+button+input+textarea; focus/invalid via group record; ButtonGroup fusing |
| 29 | card | ✅ done | C | 9adcbfe | Header/Title/Description/Content/Footer/Action; nova ring+16px spacing+footer band, base border+shadow-sm; CardAction top-right overlay slot |
| 30 | alert | ✅ done | C | ee66597 | default/destructive, icon slot, AlertTitle/Description/Action top-right slot (pr-18 reservation, self-correcting across frames) |
| 31 | avatar | ✅ done | C | ae21552 | AvatarImage via ImageSource (texture = circular cover; uri degrades to fallback), initials/icon fallback, badge, group overlap + count; sm/default/lg |
| 32 | progress | ✅ done | C | dbec4f1 | animated fill via animate_value_with_time (150ms transition-all); nova bg-muted h-1, base bg-primary/20 h-2; width/id_salt |
| 33 | table | ✅ done | C | 80dfb46 | Table/Header/Body/Footer/Row/Head/Cell; last-row border via 2-frame tracker; hover/selected bg; explicit column widths; caption reuses typography::TableCaption |
| 34 | empty | ✅ done | C | 7a3e060 | Empty/Header/Media(default+icon)/Title/Description/Content (+Footer ext.); centered text, optional dashed border, nova gap-4/size-8 box |
| 35 | item | ✅ done | C | 972c575 | Item/Media(icon|image)/Content/Actions/Group/Header/Footer/Separator; variants default/outline/muted, sizes default/sm/xs(nova); actions flex via 2-frame width reservation + right-align in header/footer |
| 36 | scroll-area | ✅ done | C | e50bb85 | egui ScrollArea wrapper; scoped ScrollStyle::solid (10px track, p-px) + extreme_bg_color=border thumb; vertical/horizontal/both; viewport focus-ring gap (egui SA not focusable) |
| 37 | resizable | ✅ done | C | 0725a5e | PanelGroup(h/v, length builder) + Panel(min/max fractions) + Handle(drag, arrow-key resize, grip chip, focus ring); fractions persist per group id; grip glyph drawn inline (no Icon::GripVertical in foundation) |
| 38 | accordion | ✅ done | D | | Item/Trigger/Content; single (collapsible opt) + multiple; chevron rotates 180°, animated height (shared helper w/ collapsible); egui built-in Tab/arrows focus + Enter/Space activate (no Radix arrow auto-activation) |
| 39 | collapsible | ✅ done | D | | Collapsible(root)/Trigger/Content; state persisted per id_salt or controlled &mut bool; animated height clip (open_progress, 150ms open / 75ms close) |
| 40 | tabs | ✅ done | D | | Tabs/List/Trigger/Content; variants default (bg-muted pill + active fill) and line (underline/side indicator), horizontal + vertical; value per id_salt or controlled &mut String; egui arrows/Enter/Space |
| 41 | breadcrumb | ✅ done | D | | Breadcrumb/List/Item/Link/Page/Separator/Ellipsis; muted text-sm trail, hover-brighten links, ChevronRight 14px separators, MoreHorizontal ellipsis (nova size-5) |
| 42 | pagination | ✅ done | D | | Pagination/Content/Item/Link/Previous/Next/Ellipsis; reuses Button (active=Outline, rest=Ghost per source buttonVariants, links size=icon); disabled prev/next at ends; ellipsis nova size-8 |
| 43 | dialog | ✅ done | E | e83de4e | nova recipe; zoom-in-95 approximated by fade+slide |
| 44 | alert-dialog | ✅ done | E | eaa51f4 | backdrop click does not dismiss (Radix); Esc closes without action |
| 45 | sheet | ✅ done | E | 8602bba | all 4 sides, slide 500/300ms; backdrop blur not reproducible |
| 46 | drawer | ✅ done | E | 4e28c84 | drag-to-dismiss with grab-point tracking + snap-back; no multi snap points (source default is dismissible only) |
| 47 | popover | ✅ done | E | 4c8857c | uncontrolled (id_salt) + controlled; auto side-flip via RectAlign |
| 48 | tooltip | ✅ done | E | 6a2a050 | 300ms delay default, rotated-square arrow, 4 sides |
| 49 | hover-card | ✅ done | E | 607dbfc | Radix delays (700ms open / 300ms close), interactive content |
| 50 | sonner | ✅ done | E | ca28588 | bottom-right stack, 4s auto-dismiss, click-to-dismiss, 6 variants; swipe-dismiss + visibleToasts collapse not ported |
| 51 | dropdown-menu | ✅ done | F | c5df08a | declarative builder (MenuBuilder); checkbox/radio/label/separator/sub-menus/shortcuts/icons/destructive/disabled; full Radix keyboard nav (arrows/Home/End/Enter/Escape, skip-disabled, wrap); hover-open subs; controlled+uncontrolled; nova recipe (min-w-32, indicator right) |
| 52 | context-menu | ✅ done | F | 86f782d | right-click region = Ui.min_rect(); menu at pointer; shares dropdown items/keyboard/checkbox/radio/sub-menus; controlled+uncontrolled; nova min-w-36 |
| 53 | menubar | ✅ done | F | a6d1c15 | h-8 bar, click/hover-enter switching, ArrowLeft/Right menu switch, shared items (checkbox left-indicator, radio), sub-menus, Escape/outside-click; controlled open index; close fade |
| 54 | navigation-menu | ✅ done | F | 514f0d6 | horizontal bar (hover/click open+switch, state-open accent/50), shared animated viewport + bg-border diamond indicator, inline viewport(false) panels, vertical orientation, links w/ descriptions, Escape/outside-click close, ArrowLeft/Right switch; no per-item keyboard roving inside panels |
| 55 | select | ✅ done | F | f22b537 | controlled (&mut Option<String>/&mut String) + uncontrolled via id_salt; h-9/h-8 trigger, placeholder/invalid/disabled; ScrollArea popup; labels/separators/disabled items; check indicator; keyboard nav (arrows/home/end/enter/space/escape, opens on selection highlight); no typeahead |
| 56 | combobox | ✅ done | F | ace65c7 | popover trigger (w-200, chevron, placeholder/invalid/disabled) + embedded Command list (search filter, check marks, empty state, keyboard); single &multi (removable chips, +N overflow, X-to-remove); &mut Option<String>/&mut String/&mut Vec<String>/uncontrolled |
| 57 | command | ✅ done | F | d1f4bdd | cmdk palette: builder groups/items/separators/shortcuts/icons, case-insensitive filter + should_filter pin, ScrollArea list, empty state, arrows/home/end/enter + hover/click select; CommandDialog via crate Dialog (autofocus, force keyboard, close on select/Escape/backdrop) |
| 58 | calendar | ✅ done | G | 8f86d29 | nova 28px cell grid, caption nav chevrons, today bg-accent, outside days muted, single/multiple/range (accent middle bar, rounded ends), disabled-matcher, Sun/Mon week start, pure-Rust date math (leap/rollover tested); keyboard arrows/Enter/Space/PageUp(±Shift year); gaps: dropdown captionLayout + week numbers |
| 59 | date-picker | ✅ done | G | e74f85d | docs composition: w-[240px] outline trigger (CalendarIcon, PPP format or muted placeholder) + p-0 align-start popover w/ single calendar; closes on pick/Escape/outside; initialFocus on selected day; fresh calendar per open; controlled date, id_salt open state |
| 60 | carousel | ✅ done | G | 4cce737 | pannable clipped strip (basis-full + pl-4 gutters, horiz/vert), snap-on-release anim, outline rounded-full size-8 arrows perched outside (disabled at bounds), dots (active primary, muted/50, click-to-jump), Arrow keys move slide; viewport = tallest slide; gaps: multi-slide-per-view basis fractions |
| 61 | chart | ✅ done | G | 6fb6090 | ChartContainer frame + Chart (area/line/bar) painted with egui Painter; nice-step y ticks, border/50 grid, muted text-xs ticks; hover crosshair (muted band for bars) + chart.tsx-style tooltip (dot/line/dashed indicators); clickable legend toggles visibility per id_salt; palette chart-1..5 cycled; gaps: curve smoothing (natural), entry animations, aspect-video default (fixed min-height like the demos) |
| 62 | data-table | ✅ done | G | b3bd706 | DataTable<T> column descriptors (width/align/cell/sort_key), click-to-sort (ArrowUp/Down instead of the symmetric ArrowUpDown 180° rotation), controlled filter toolbar + predicate, page-size select + Page x of y + first/prev/next/last, checkbox selection (header select-all indeterminate, selected bg), row-actions slot, rounded-md border wrapper, No results row; gaps: column-visibility dropdown |
| 63 | form | ✅ done | G | d3ef287 | Form<T> (&mut T) with closure fields (name, validate -> Option<String>, render with FormFieldCtx{invalid,error,id,disabled}), validate-on-submit gated by a persisted touched flag (or validate_on_change), submit_button/request_submit re-validating the post-edit value, on_submit, disabled/submitting, reset_errors_button; gaps: no built-in value reset (caller owns it), Enter-in-input submit |
| 64 | sidebar | ✅ done | G | 67943bc | Sidebar (uncontrolled per id_salt or controlled &mut bool) with animated width 16rem↔3rem (icon) / 0 (offcanvas), sidebar/floating/inset variants, left/right; Header/Content/Footer/Separator/Group/GroupLabel/GroupContent/Menu/MenuItem/MenuButton (default+outline, sm/default/lg, active, badge, collapsed icon-only + delay-0 tooltip)/MenuSub/SubButton, Trigger (stack or with_salt), Rail, Inset; Ctrl/Cmd+B + Escape-with-focus collapse; gaps: mobile Sheet mode, SidebarMenuAction/MenuBadge/MenuSkeleton (badge folded into MenuButton), content overflow-auto |
| 65 | attachment | ✅ done | H | 00f6792 | chip (rounded-xl border bg-card) with media icon/image/spinner, truncating title+description, ghost icon-xs actions, state styling (idle dashed border, error destructive/30, uploading/processing title shimmer→color pulse), sizes default/sm/xs, horizontal (min-w-40) + vertical (w-30, actions overlaid on media via Area), AttachmentGroup horizontal scroll strip, trigger=true = whole-chip click; gaps: w-24 media-only vertical not detected pre-layout, shimmer approximated as color pulse, no image-opacity easing |
| 66 | bubble | ✅ done | H | 36897d6 | Bubble (default/secondary/muted/tinted/outline/ghost/destructive; align start/end, max-w 80%, ghost full-width), BubbleContent pill (rounded-xl px-3 py-2 text-sm wrap), BubbleGroup (flex-col gap-2), BubbleReactions pill (bg-muted ring-3 ring-card, side top/bottom × align start/end); tinted recomputes oklch(from primary L c·k h) via a local sRGB→OKLCH helper; gaps: hover shifts only apply to button/link content in web (no-op for static content), no markdown (plain wrapped text or custom closure content) |
| 67 | message | ✅ done | H | 5146354 | Message row (align start = LTR avatar-left, end = right-to-left avatar-right, gap-2, bottom-aligned slots), MessageAvatar 32px slot hosting Avatar, MessageContent full-width column gap-2.5 (children right-align on end), MessageHeader/MessageFooter text-xs muted px-3 (footer right-aligns on end), MessageGroup gap-2; gaps: no MessageActions (not exported by this source version), group-has-ghost px-0 approximated with an explicit .flush() flag since immediate mode can't detect sibling variants retroactively, footer avatar -translate-y-8 shift not ported |
| 68 | message-scroller | ✅ done | H | c563ce1 | MessageScroller (egui ScrollArea, stick_to_bottom auto-follow that disengages on scroll-up and re-engages at bottom, per-id_salt state, gap-8 content spacing, height/gap/id_salt builders, themed scrollbar), MessageScrollerButton (floating secondary icon-sm ArrowDown at viewport bottom-4, appears when scrolled away, click jumps to bottom via scroll_to_cursor + re-engages follow), MessageScrollerResponse{at_bottom,following}; gaps: MessageScrollerItem scrollAnchor turn-anchoring not ported, smooth scroll animation/fades/role=log aria are web-only |
| 69 | direction | ⬜ todo | H | | RTL/LTR provider — egui layout direction wrapper (⚠️ likely partial) |

## Commit log convention

- `chore(foundation): ...`, `feat(theme): ...`, `feat(icons): ...`
- `feat(<component>): port shadcn <component> to egui`
- `fix(<component>): ...`

## Known gaps log

(filled as components land)
