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
| 6 | button | ⬜ todo | A | | variants: default/destructive/outline/secondary/ghost/link; sizes: xs..lg, icon-* |
| 7 | badge | ⬜ todo | A | | variants: default/secondary/destructive/outline/ghost/link |
| 8 | label | ⬜ todo | A | | required dot, invalid/disabled |
| 9 | separator | ⬜ todo | A | | horizontal/vertical |
| 10 | kbd | ⬜ todo | A | | mono chip |
| 11 | spinner | ⬜ todo | A | | sizes sm/md/lg |
| 12 | skeleton | ⬜ todo | A | | animated pulse |
| 13 | typography | ⬜ todo | A | | docs family: h1..h4, p, blockquote, list, code, pre, muted |
| 14 | aspect-ratio | ⬜ todo | A | | fixed ratio container |
| 15 | marker | ⬜ todo | A | | variants incl. type/size (AI era component) |
| 16 | button-group | ⬜ todo | A | | orientation, sizes, split, nested |
| 17 | input | ⬜ todo | B | | invalid, disabled, sizes |
| 18 | textarea | ⬜ todo | B | | |
| 19 | checkbox | ⬜ todo | B | | checked/indeterminate, disabled |
| 20 | radio-group | ⬜ todo | B | | |
| 21 | switch | ⬜ todo | B | | |
| 22 | slider | ⬜ todo | B | | single + range |
| 23 | toggle | ⬜ todo | B | | variants/sizes |
| 24 | toggle-group | ⬜ todo | B | | single/multiple |
| 25 | input-otp | ⬜ todo | B | | N-slot OTP input |
| 26 | native-select | ⬜ todo | B | | egui ComboBox-based |
| 27 | field | ⬜ todo | B | | Field/Label/Description/Error/Group/Set |
| 28 | input-group | ⬜ todo | B | | input + addons/icons/inputs |
| 29 | card | ⬜ todo | C | | Header/Title/Description/Content/Footer/Action |
| 30 | alert | ⬜ todo | C | | default/destructive, icon/title/description/action |
| 31 | avatar | ⬜ todo | C | | image/fallback/group, sizes |
| 32 | progress | ⬜ todo | C | | value track |
| 33 | table | ⬜ todo | C | | Header/Body/Row/Head/Cell/Caption/Footer |
| 34 | empty | ⬜ todo | C | | header/media/title/description/content |
| 35 | item | ⬜ todo | C | | Item/Media/Content/Actions/Group/Header/Footer |
| 36 | scroll-area | ⬜ todo | C | | egui ScrollArea wrapper, styled scrollbar |
| 37 | resizable | ⬜ todo | C | | panels with drag handles |
| 38 | accordion | ⬜ todo | D | | single/multiple, animated |
| 39 | collapsible | ⬜ todo | D | | |
| 40 | tabs | ⬜ todo | D | | list/trigger/content, vertical |
| 41 | breadcrumb | ⬜ todo | D | | list/item/link/page/ellipsis/separator |
| 42 | pagination | ⬜ todo | D | | content/previous/next/ellipsis/links |
| 43 | dialog | ⬜ todo | E | | modal, backdrop, Esc close, sizes |
| 44 | alert-dialog | ⬜ todo | E | | confirm flow, footer actions |
| 45 | sheet | ⬜ todo | E | | side drawers (top/right/bottom/left) |
| 46 | drawer | ⬜ todo | E | | bottom sheet (vaul-style) |
| 47 | popover | ⬜ todo | E | | anchored, click-outside, Esc |
| 48 | tooltip | ⬜ todo | E | | delayed hover popup |
| 49 | hover-card | ⬜ todo | E | | rich hover preview |
| 50 | sonner | ⬜ todo | E | | toast manager + toaster, positions, variants |
| 51 | dropdown-menu | ⬜ todo | F | | items/checkbox/radio/label/separator/sub, kbd shortcuts |
| 52 | context-menu | ⬜ todo | F | | right-click menu |
| 53 | menubar | ⬜ todo | F | | app menu bar |
| 54 | navigation-menu | ⬜ todo | F | | horizontal/vertical, viewport panels |
| 55 | select | ⬜ todo | F | | trigger + content, scroll up/down |
| 56 | combobox | ⬜ todo | F | | popover + command filter |
| 57 | command | ⬜ todo | F | | ⌘K palette: groups, items, kbd |
| 58 | calendar | ⬜ todo | G | | month grid, nav, presets, multiple/range |
| 59 | date-picker | ⬜ todo | G | | popover + calendar + input |
| 60 | carousel | ⬜ todo | G | | pannable slides + dots + prev/next |
| 61 | chart | ⬜ todo | G | | egui-painted area/bar/line charts, tooltip |
| 62 | data-table | ⬜ todo | G | | table + toolbar + selection + pagination + sorting/filtering |
| 63 | form | ⬜ todo | G | | field + validation wiring |
| 64 | sidebar | ⬜ todo | G | | collapsible rail + header/content/footer/menu |
| 65 | attachment | ⬜ todo | H | | file chips, variants, state |
| 66 | bubble | ⬜ todo | H | | chat bubbles, variants, group |
| 67 | message | ⬜ todo | H | | message group/content/actions |
| 68 | message-scroller | ⬜ todo | H | | auto-scroll chat container |
| 69 | direction | ⬜ todo | H | | RTL/LTR provider — egui layout direction wrapper (⚠️ likely partial) |

## Commit log convention

- `chore(foundation): ...`, `feat(theme): ...`, `feat(icons): ...`
- `feat(<component>): port shadcn <component> to egui`
- `fix(<component>): ...`

## Known gaps log

(filled as components land)
