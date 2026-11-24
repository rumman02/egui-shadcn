//! shadcn `item.tsx` — flexible list item rows with media/content/actions.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/item.tsx` with the nova
//! recipe as the baseline (`rounded-lg border`, sizes `px-3 py-2.5 gap-2.5`
//! default/sm and `px-2.5 py-2 gap-2` for the nova-only `xs`). Variants:
//! `default` (transparent), `outline` (bordered), `muted` (`bg-muted/50`).
//! Nova `.cn-item*` rules layer over the base recipe (LAYER, settled:
//! silence keeps the base value; see the frozen spec §0 and the
//! shadow-xs ruling evidence). There is no Radix primitive behind item —
//! the only Radix import is `Slot`, for `asChild` polymorphism (spec §1).
//!
//! Slots: [`ItemMedia`] (icon box or image), [`ItemContent`] (title +
//! description), [`ItemActions`] (right-aligned inside the item, like the
//! source's `justify-between` rows), plus [`ItemHeader`]/[`ItemFooter`]
//! full-width rows, [`ItemGroup`] (vertical list with a per-size gap) and
//! [`ItemSeparator`].
//!
//! # Verified final state (Batch C S10 + re-verification, all 4 configs)
//!
//! Every statement below is verified in the gallery unless marked as a
//! spec value or a limitation. S10 supersedes S5/S6 where they differ.
//!
//! ## Geometry — frames at the spec model (S10 §Geometry, reverify §ITM-04/05)
//!
//! Frames stretch to the available width (ITM-02, S9 `9724f9e`): the full
//! 1019px gallery column, §4/§6 at their 360px scope. Frame heights match
//! the spec model in all 4 configs — nova icon+desc 66, title-only 41,
//! §4 98, §5 66, §6 87; vega icon+desc 78, title-only 53, §4 110 (19px
//! galley rounding, −1), §5 78, §6 99; §3 without-image 62 nova / 74 vega
//! and without-icon 54 nova / 66 vega against the S10-corrected models
//! (the S5 models for those two frames were superseded: the pre-fix
//! narrow frame wrapped the title onto two lines). §7 Header/Footer rows
//! render at the corrected model **111 nova / 141 vega** (ITM-04, S9
//! `1117d1d`; the defect file's 109/133 was wrong — the footer row is the
//! Sm button's *outer* rect, 28 nova / 32 vega, and the vega badge/button
//! are taller than nova's, 22/32 vs 20/28). Actions sit at the item's
//! right edge (ITM-05, S9 `dd546a5`): §8 Launch ends 13px from the frame
//! edge under nova, 16px under vega (pad + the 1px Outline border);
//! Comp2 Follow moved with it. §7 Manage was already right-aligned and
//! is unchanged.
//!
//! ## The five S9 fixes — what changed and why
//!
//! * **ITM-01** (S9 `15aa1f0`): every frame rendered ~37px too tall because
//!   the root row, title row and content used `ui.horizontal`, whose 36px
//!   `interact_size` floor centred the content in a 36px row. The rows now
//!   allocate at content height (`allocate_ui_with_layout`, Center
//!   cross-align preserved); the N12 shifted media box allocates normally
//!   and re-interacts at row top + 2 instead of `allocate_rect`.
//! * **ITM-02** (S9 `9724f9e`): `ui.set_min_width(ui.available_width())`
//!   in the root-row closure and in `ItemGroup::show` — the `Frame` sizes
//!   to its content, so without this the item hugged its content width.
//! * **ITM-03** (S9 `38cdc4e`): `Item::show` calls `response.request_focus()`
//!   on click (the `Button` shape, `button.rs:1175-1177`), placed before
//!   the ring paint, so the ring paints on the click frame. Clicking the
//!   nested Launch button focuses the *item* row, not the button: egui's
//!   click is exclusive and the item's later-registered `interact` wins
//!   the hit-test tie, so the button never receives the click (verified
//!   against vendored egui 0.37 source and empirically).
//! * **ITM-04** (S9 `1117d1d`): `ItemHeader`/`ItemFooter` moved from
//!   `ui.horizontal` to `allocate_ui_with_layout` at the reserved actions
//!   height (the ITM-01 mechanism); `ItemActions` records its measured
//!   height into temp data so the row reserves it next frame, plus a
//!   0-width spacer at the reserved height that forces the row's
//!   `min_rect` to span the full row height. One-frame settle: frame 1
//!   reads 99 nova / 123 vega, settled from frame 2.
//! * **ITM-05** (S9 `dd546a5`): one line in `ItemContent::show`,
//!   `ui.set_min_width(w)` — the wrapping row advances by each child's
//!   `min_rect`, and the content Ui's `min_rect` was its max-content
//!   width, so the allocated flex width never reached the row and the
//!   actions landed right after the content's ink. Same `set_min_width`
//!   pattern as ITM-02; wrap budgets read `available_width()` and are
//!   unaffected.
//!
//! The `actions_w` / `actions_h` reservations are two-frame measurements:
//! any measurement that depends on them (actions position, §7 height)
//! settles from frame 2 — the same one-frame-settle class as alert D7.
//!
//! ## Interaction — anchor model, click-to-focus, keyboard
//!
//! Interactive rows are anchor rows (spec §7, D4): `Item::interactive(bool)`
//! is the explicit opt-in standing in for `asChild` + `<a>`. Only an
//! interactive row takes a Tab stop, paints a hover fill, or reports a
//! click; a plain `div` row takes no focus and reports no clicks. The R1
//! stop map is 82 stops, identical in all 4 configs and unchanged by the
//! S9 fixes (S10 §R1, reverify §R1). Clicking an interactive item toggles
//! it exactly once (R2 2/2 each config); Space and Return each toggle
//! exactly once after a click (R3, OBS-ITM-S6-2 resolved — egui activates
//! a focused click widget on Space/Enter, which differs from the browser's
//! Enter-only link activation and is recorded as part of D4, not as a
//! source requirement). The nested Launch button never takes focus and
//! there is no button focus ring (S10 §R7: the button-region diff bbox is
//! exactly the region — the item hover fill — so `classify_ring` mislabels
//! it; no button ring exists). Focus ring: 3px `ring/50` outside the
//! border box, never affecting layout; settled ring counts 4351 nova /
//! 4453 vega (S10 motion R2). Hover fill is endpoint-only: 41 plateaus, 0
//! intermediates in every config (S7/S10 distribution videos); vega/light
//! hover delta is 5 grey levels (250 vs 255), below diff tol 8 — use
//! tol=2 / ±1 classification (OBS-ITM-S7-5).
//!
//! ## Settled edge cases (spec §9 N10–N13)
//!
//! * **N10** — description links: a direct `<a>` child renders `underline`
//!   + 4px offset and recolours to `text-primary` on hover. Satisfied at
//!     S1 via the additive [`ItemDescription::link`]/[`ItemDescription::text`]
//!     builder (OPEN-3); no dedicated link builder is required by the spec.
//! * **N11** — title wrap: the title wraps at the content width
//!   (`flex w-fit`); the icon never wraps below the text, staying on the
//!   same flex line, vertically centred. §4's 4-line title row is
//!   312.1×76.0 (nova) at 19.25px/line (S10 §Geometry).
//! * **N12** — media with description: the media shifts 2px down and
//!   top-aligns. Rendered: icon top − item top 13.0 with description /
//!   11.0 without, shift exactly 2.0 (S10 §Geometry); the icon now
//!   genuinely sits at pad+border 11.0 without a description.
//! * **N13** — `flex-wrap`: the root row wraps; header/footer are
//!   `basis-full`, each occupying a full line by construction.
//! * Description clamps to 2 lines with an ellipsis (`line-clamp-2`,
//!   spec §2.9; D9 satisfied at S1 — truncation with "…" implemented,
//!   not a deviation). Title is always `font-medium` (500) at 19.25px
//!   leading; description always `font-normal` (400) at 21px leading.
//! * Group gap keys off the contained items' `data-size` (spec §2.4):
//!   16px default, 10px with an `sm` item, 8px with an `xs` item
//!   (xs wins on mixed sizes). `ItemGroup::size()` is a Rust extension
//!   (D5) applying one size to the whole group; mixed-size groups are
//!   OPEN-1.
//! * `xs` exists only under nova (spec §2.3). Under base styles the port
//!   applies the nova-xs geometry as an extension.
//!
//! ## States — none invented
//!
//! No disabled, selected, invalid, pressed or empty state is defined in
//! the source; none is invented (spec §4). No ARIA tree exists in egui
//! (D2). `:focus-visible` modality: the ring paints on ANY egui focus,
//! keyboard or pointer — only the trigger condition differs (D1).
//! `data-*` attributes are dropped; their values survive as `ItemCtx` /
//! builder state (D3). `transition-colors duration-100` recolours
//! instantly in egui — no per-property CSS transitions (D8, platform
//! limitation); the 3px ring is a `box-shadow`, outside the
//! `transition-colors` set, so it appears instantly even in the browser
//! (spec §6). `text-balance` has no egui equivalent (D7, limitation).
//! A second `ItemContent` is NOT `flex-none` (spec §2.7 says the second
//! content does not grow) — a recorded limitation: every content grows
//! via the ITM-05 `set_min_width(w)`.
//!
//! ## Tokens (spec §5)
//!
//! `--foreground` light `oklch(0% 0 0)` (`globals.css:102`) / dark
//! `oklch(0.985 0 0)`; `--muted` 0.97 / 0.269; `--muted-foreground`
//! 0.556 / 0.708; `--accent` 0.97 / 0.371 (dark Position A, SEL-VIS-03 —
//! `themes.ts:56` says 0.269); `--border` 0.922 / `oklch(1 0 0 / 10%)`;
//! `--ring` 0.708 / 0.556; `--primary` 0% / 0.922. **Light
//! `--foreground` is CONTESTED** (verified first-hand at S11):
//! `globals.css:102` says `oklch(0% 0 0)` while `themes.ts:11` says
//! `oklch(0.145 0 0)` — the spec §5.4 claim that the two sources agree
//! exactly is wrong for this token (same divergence card S11 and table
//! S11 recorded). **Light `--primary` is CONTESTED the same way**
//! (`globals.css:107` `oklch(0% 0 0)` vs `themes.ts:17` `oklch(0.205 0 0)`;
//! progress S11 recorded it too). All other item tokens agree exactly
//! between the two sources. Both go to S12 / Expert Reviewer; the
//! effective value the port consumes is the `themes.ts` value per the
//! standing R1 reading.
//!
//! ## Deviations and OPEN inventory — current dispositions
//!
//! D1 PROPOSED (`:focus-visible` modality), D2 PROPOSED (no ARIA tree),
//! D3 PROPOSED (`data-*` dropped), D4 PROPOSED (`asChild` →
//! `interactive(bool)` + Space/Enter activation), D5 PROPOSED
//! (`ItemGroup::size()` extension), D6 PROPOSED
//! (`ItemTitle::show_with` extension) — all six await the Expert
//! Reviewer. D7/D8 are recorded limitations (no approval needed). D9 is
//! satisfied (line-clamp implemented at S1), not a deviation. OPEN-1
//! OPEN (mixed-size group gap), OPEN-2 OPEN (standing `cn()` merge
//! semantics — never closes on substitute reasoning), OPEN-3 OPEN (the
//! `.link()`/`.text()` builder was S1's additive choice; whether it
//! closes the item is the reviewer's), OPEN-4 OPEN (cross-wave,
//! dropdown-menu `p-0`, Batch F), OPEN-5 OPEN (folds into OPEN-1).
//!
//! ## Observations carried to the Expert Reviewer / S12 (not defects)
//!
//! * **OBS-ITM-S10-01** — vega `xs` frames sit +3.75px over S5's derived
//!   model (vega §1 Xs 66 vs 62.25; §2 xs 41 vs 37.25): the port applies
//!   the nova-default py-10, not the nova-xs py-8, under vega. Base `xs`
//!   has no defined geometry (spec §2.3), so there is no spec value to
//!   violate; S12 defines it (62.25, 66, or pure-base 78.25).
//! * **OBS-ITM-S10-RV-01** — Launch ends exactly 1px right of
//!   Manage/Follow in both styles (nova inset 13 vs 14; vega 16 vs 17);
//!   the gallery item is 1019px wide vs the headless replay's 1018px
//!   root — fractional-layout rounding, same class as the S10-accepted
//!   vega §4 −1px galley rounding.
//!
//! ## Standing caveats (not defects)
//!
//! Render floor NOT VERIFIED in all 4 configs (wiggle 1.93–2.03 fps,
//! Tab-driven 2.0–2.4 fps — pipeline-capped, same class as S7). Timing
//! is UNRESOLVED at video level (presentation ~0.65 fps under load);
//! the distribution analysis (41 plateaus, 0 intermediates) is the
//! decisive instant-change evidence. Latency readings are poll upper
//! bounds, not measured latencies (worst on 152–283ms, worst off
//! 211–304ms at S10; the vega/dark 304ms reading is a 4ms overage on a
//! poll upper bound under load 3.74). Budgets hold: idle 0.0%,
//! animating 91.71–131.18% (<150%), RSS −1.09%..+0.25%, flicker 7×0,
//! determinism 0, round-trips byte-identical (standalone md5
//! `f34b9b9e967eaad14a50e7c7f22e0482`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ItemGroup::new().show(ui, |ui| {
//!     Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
//!         ItemMedia::icon(Icon::User).show(ui);
//!         ItemContent::new().show(ui, |ui| {
//!             ItemTitle::new("Ada Lovelace").show(ui);
//!             ItemDescription::new("Engineer").show(ui);
//!         });
//!         ItemActions::new().show(ui, |ui| {
//!             Button::new("Follow").size(ButtonSize::Sm).show(ui);
//!         });
//!     });
//! });
//! ```

use crate::components::avatar::{cover_uv, resolve_texture};
use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{
    Color32, FontId, ImageSource, InnerResponse, Margin, Pos2, Rect, Sense, Stroke, Ui, Vec2,
};

/// Title line-height multiplier (`leading-snug`).
const TITLE_LINE_HEIGHT: f32 = 1.375;
/// Description line-height multiplier (`leading-normal`).
const DESCRIPTION_LINE_HEIGHT: f32 = 1.5;
/// Link underline offset (`underline-offset-4`): the underline sits 4px
/// below the link baseline (N10).
const LINK_UNDERLINE_OFFSET: f32 = 4.0;
/// Minimum content width before the wrapping row moves the content to the
/// next line instead of squeezing it (N13). Below this the content asks
/// for the full-line width, which the wrapping layout places on a fresh
/// line.
const MIN_CONTENT_WIDTH: f32 = 48.0;

/// One inline segment of an [`ItemDescription`].
///
/// The description is `text-muted-foreground`; a direct `<a>` child of the
/// description renders underlined with a 4px offset and `hover:text-primary`
/// (`item.tsx:138`, N10). The builder API is [`ItemDescription::link`] for
/// a link segment and [`ItemDescription::text`] for plain text; `new(text)`
/// stays a single plain segment.
#[derive(Clone, Debug, PartialEq, Eq)]
enum DescriptionSegment {
    /// Plain description text (`text-muted-foreground`).
    Text(String),
    /// A direct-child link: underlined, 4px offset, `text-primary` on hover.
    Link(String),
}

impl DescriptionSegment {
    fn text(&self) -> &str {
        match self {
            DescriptionSegment::Text(text) | DescriptionSegment::Link(text) => text,
        }
    }

    fn is_link(&self) -> bool {
        matches!(self, DescriptionSegment::Link(_))
    }

    fn with_text(&self, text: String) -> Self {
        match self {
            DescriptionSegment::Text(_) => DescriptionSegment::Text(text),
            DescriptionSegment::Link(_) => DescriptionSegment::Link(text),
        }
    }
}

/// One underlined link run inside a laid-out description galley: the
/// underline segment (`from` → `to`, in galley coordinates) and the
/// hover hit rect (the run's x-extent, 2px wider on each side and as tall
/// as the row — the same shape the empty component uses).
#[derive(Clone, Copy, Debug)]
struct LinkRun {
    from: Pos2,
    to: Pos2,
    hit: Rect,
}

fn make_link_run(x0: f32, x1: f32, baseline: f32, row_top: f32, row_height: f32) -> LinkRun {
    let y = baseline + LINK_UNDERLINE_OFFSET;
    LinkRun {
        from: Pos2::new(x0, y),
        to: Pos2::new(x1, y),
        hit: Rect::from_min_max(
            Pos2::new(x0 - 2.0, row_top),
            Pos2::new(x1 + 2.0, row_top + row_height),
        ),
    }
}

/// Scans the laid-out galley against the expected segment text and returns
/// one [`LinkRun`] per link segment per visual row (a link that wraps
/// produces one run per row). Returns no runs when the galley and the
/// segments disagree (truncated, mixed-direction or font-substituted text),
/// so the description degrades to plain text instead of mis-underlining.
fn link_runs(galley: &egui::Galley, segments: &[DescriptionSegment]) -> Vec<LinkRun> {
    let mut expected: Vec<(char, bool)> = Vec::new();
    for segment in segments {
        expected.extend(segment.text().chars().map(|c| (c, segment.is_link())));
    }
    let mut runs = Vec::new();
    let mut idx = 0usize;
    for placed in &galley.rows {
        let row = &placed.row;
        let mut run: Option<(f32, f32)> = None;
        let mut run_end_x = 0.0;
        for glyph in &row.glyphs {
            let Some(&(ch, is_link)) = expected.get(idx) else {
                return Vec::new();
            };
            if glyph.chr != ch {
                return Vec::new();
            }
            idx += 1;
            if is_link {
                let baseline = placed.pos.y + glyph.pos.y;
                let x0 = placed.pos.x + glyph.pos.x;
                let x1 = x0 + glyph.advance_width;
                if run.is_none() {
                    run = Some((x0, baseline));
                }
                run_end_x = x1;
            } else if let Some((x0, baseline)) = run.take() {
                runs.push(make_link_run(
                    x0,
                    run_end_x,
                    baseline,
                    placed.pos.y,
                    row.size.y,
                ));
            }
        }
        if let Some((x0, baseline)) = run.take() {
            runs.push(make_link_run(
                x0,
                run_end_x,
                baseline,
                placed.pos.y,
                row.size.y,
            ));
        }
    }
    runs
}

/// Counts the visual lines of a laid-out galley, ignoring a trailing empty
/// row (e.g. from a trailing newline) so the `line-clamp-2` check counts
/// rendered lines only.
fn effective_rows(galley: &egui::Galley) -> usize {
    galley
        .rows
        .iter()
        .filter(|placed| !placed.row.glyphs.is_empty())
        .count()
}

/// Builds the description layout job for the given segments. Link segments
/// take `link_color`; plain segments take `base_color`.
fn description_job(
    segments: &[DescriptionSegment],
    size: f32,
    base_color: Color32,
    link_color: Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for segment in segments {
        let color = if segment.is_link() {
            link_color
        } else {
            base_color
        };
        job.append(
            segment.text(),
            0.0,
            egui::TextFormat {
                font_id: FontId::proportional(size),
                color,
                line_height: Some(size * DESCRIPTION_LINE_HEIGHT),
                ..egui::TextFormat::default()
            },
        );
    }
    job
}

/// Takes the first `n` chars of the segments, preserving segment
/// boundaries, and appends "…" to the last taken segment (inheriting its
/// style, so an ellipsis ending a link is underlined like the link).
fn take_prefix_segments(segments: &[DescriptionSegment], n: usize) -> Vec<DescriptionSegment> {
    let mut out: Vec<DescriptionSegment> = Vec::new();
    let mut remaining = n;
    for segment in segments {
        if remaining == 0 {
            break;
        }
        let take = remaining.min(segment.text().chars().count());
        let prefix: String = segment.text().chars().take(take).collect();
        out.push(segment.with_text(prefix));
        remaining -= take;
    }
    match out.last_mut() {
        Some(DescriptionSegment::Text(text)) | Some(DescriptionSegment::Link(text)) => {
            text.push('…')
        }
        None => out.push(DescriptionSegment::Text("…".to_string())),
    }
    out
}

/// Truncates the segments to at most `max_rows` visual lines with a
/// trailing "…" (`line-clamp-2`, spec §2.9 / D9). Returns the original
/// segments unchanged when they already fit. Binary-searches the longest
/// char prefix whose layout (with the "…" appended) fits in `max_rows`.
fn truncate_segments(
    segments: &[DescriptionSegment],
    size: f32,
    width: f32,
    max_rows: usize,
    ui: &Ui,
    base_color: Color32,
    link_color: Color32,
) -> Vec<DescriptionSegment> {
    let fits = |candidate: &[DescriptionSegment]| -> bool {
        let mut job = description_job(candidate, size, base_color, link_color);
        job.wrap.max_width = width;
        let galley = ui.painter().layout_job(job);
        effective_rows(&galley) <= max_rows
    };
    if fits(segments) {
        return segments.to_vec();
    }
    let total: usize = segments.iter().map(|s| s.text().chars().count()).sum();
    // Prefix length 0 ("…" alone) always fits in one row.
    let mut lo = 0usize;
    let mut hi = total;
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if fits(&take_prefix_segments(segments, mid)) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    take_prefix_segments(segments, lo)
}

/// Resolves the nova group gap from the contained items' sizes (spec
/// §2.4): 16px by default, 10px if any descendant item is `sm`, 8px if any
/// is `xs` (xs wins on mixed sizes — equal-specificity cascade, source
/// order). `member_sizes` is the set recorded last frame; when empty (first
/// frame, or a group with no items yet) the group's own size hint is used.
/// Base styles have no group gap (0).
pub(crate) fn resolve_group_gap(
    style: Style,
    group_size: ItemSize,
    member_sizes: &[ItemSize],
) -> f32 {
    if style != Style::Nova {
        return 0.0;
    }
    if member_sizes.contains(&ItemSize::Xs) {
        return 8.0;
    }
    if member_sizes.contains(&ItemSize::Sm) {
        return 10.0;
    }
    if !member_sizes.is_empty() {
        return 16.0;
    }
    group_gap(style, group_size)
}

/// Item variants, mirroring the source `itemVariants.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemVariant {
    /// `bg-transparent border-transparent`.
    #[default]
    Default,
    /// `border-border`.
    Outline,
    /// `bg-muted/50 border-transparent`.
    Muted,
}

/// Item sizes, mirroring the source `itemVariants.size` (plus the nova
/// `xs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemSize {
    /// `gap-4 p-4` (nova `gap-2.5 px-3 py-2.5`).
    #[default]
    Default,
    /// `gap-2.5 px-4 py-3` (nova same as default).
    Sm,
    /// Nova-only `gap-2 px-2.5 py-2`.
    Xs,
}

/// Geometry `(pad_x, pad_y, gap)` per style and size.
pub(crate) fn item_geometry(style: Style, size: ItemSize) -> (f32, f32, f32) {
    match style {
        Style::Nova => match size {
            ItemSize::Xs => (10.0, 8.0, 8.0),
            _ => (12.0, 10.0, 10.0),
        },
        _ => match size {
            ItemSize::Default => (16.0, 16.0, 16.0),
            ItemSize::Sm => (16.0, 12.0, 10.0),
            ItemSize::Xs => (12.0, 10.0, 8.0),
        },
    }
}

/// The gap an [`ItemGroup`] uses between items (nova `gap-4` / `gap-2.5` /
/// `gap-2` by size). The base recipe's group is a plain `flex flex-col`
/// with no gap, so non-nova styles return 0 (use [`ItemSeparator`] there).
pub(crate) fn group_gap(style: Style, size: ItemSize) -> f32 {
    match style {
        Style::Nova => match size {
            ItemSize::Xs => 8.0,
            ItemSize::Sm => 10.0,
            ItemSize::Default => 16.0,
        },
        _ => 0.0,
    }
}

/// Resolved item colors (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ItemColors {
    pub fill: Color32,
    pub stroke: Stroke,
}

pub(crate) fn resolve_item_colors(p: &Palette, variant: ItemVariant) -> ItemColors {
    match variant {
        ItemVariant::Default => ItemColors {
            fill: Color32::TRANSPARENT,
            stroke: Stroke::NONE,
        },
        ItemVariant::Outline => ItemColors {
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.0, p.border),
        },
        ItemVariant::Muted => ItemColors {
            fill: color_alpha(p.muted, 0.5),
            stroke: Stroke::NONE,
        },
    }
}

/// The icon media box side (`size-8`).
pub const ITEM_MEDIA_ICON_BOX: f32 = 32.0;
/// The image media side (`size-10`).
pub const ITEM_MEDIA_IMAGE_SIZE: f32 = 40.0;
/// The media icon glyph size (`size-4`).
pub const ITEM_MEDIA_ICON: f32 = 16.0;

/// Context for the item currently being laid out.
#[derive(Clone, Copy, Debug)]
struct ItemCtx {
    id: egui::Id,
    size: ItemSize,
    /// Whether the item had an [`ItemDescription`] descendant last frame
    /// (N12: the media is laid out before the description, so it reads
    /// last frame's flag).
    has_desc_prev: bool,
}

fn item_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_item_stack")
}

fn push_item(ui: &Ui, ctx: ItemCtx) {
    let mut stack: Vec<ItemCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(item_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(item_stack_id(), stack));
}

fn pop_item(ui: &Ui) {
    let mut stack: Vec<ItemCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(item_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(item_stack_id(), stack));
}

fn current_item(ui: &Ui) -> Option<ItemCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<ItemCtx>>(item_stack_id()))
        .and_then(|s| s.last().copied())
}

/// Context for the header/footer row currently being laid out (so
/// [`ItemActions`] can right-align, like the source's `justify-between`).
#[derive(Clone, Copy, Debug)]
struct RowCtx {
    left: f32,
    top: f32,
    right: f32,
    /// Temp-data key (suffixed to the enclosing item's id) under which
    /// [`ItemActions`] records the actions' measured height, so the row
    /// can reserve it next frame (ITM-04: the actions lay out in a
    /// separate child Ui, so their height never grows the row itself).
    actions_h_key: &'static str,
}

fn row_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_item_row_stack")
}

fn push_row(ui: &Ui, ctx: RowCtx) {
    let mut stack: Vec<RowCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(row_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(row_stack_id(), stack));
}

fn pop_row(ui: &Ui) {
    let mut stack: Vec<RowCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(row_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(row_stack_id(), stack));
}

fn current_row(ui: &Ui) -> Option<RowCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<RowCtx>>(row_stack_id()))
        .and_then(|s| s.last().copied())
}

/// Group membership for [`ItemGroup`] size inheritance, plus the group's
/// id (so items can record their resolved size into it for the §2.4
/// group-gap rule).
#[derive(Clone, Copy, Debug)]
struct GroupCtx {
    size: ItemSize,
    id: egui::Id,
}

fn group_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_item_group_stack")
}

fn push_group(ui: &Ui, ctx: GroupCtx) {
    let mut stack: Vec<GroupCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(group_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx()
        .data_mut(|d| d.insert_temp(group_stack_id(), stack));
}

fn pop_group(ui: &Ui) {
    let mut stack: Vec<GroupCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(group_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(group_stack_id(), stack));
}

fn current_group(ui: &Ui) -> Option<GroupCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<GroupCtx>>(group_stack_id()))
        .and_then(|s| s.last().copied())
}

/// All groups enclosing the current position (outermost first), for the
/// §2.4 group-gap recording (an item records its resolved size into every
/// enclosing group).
fn group_stack(ui: &Ui) -> Vec<GroupCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<GroupCtx>>(group_stack_id()))
        .unwrap_or_default()
}

/// A shadcn item. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct Item {
    variant: ItemVariant,
    size: Option<ItemSize>,
    interactive: bool,
    id_salt: Option<String>,
}

impl Item {
    /// Creates a default item.
    pub fn new() -> Self {
        Self {
            variant: ItemVariant::default(),
            size: None,
            interactive: false,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`ItemVariant::Default`]).
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the size (inherits the enclosing [`ItemGroup`] size when
    /// unset).
    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Makes the item interactive (the source's `asChild` anchor rows):
    /// hovering fills it with `bg-accent/50` (nova `bg-muted`) and focusing
    /// paints the `focus-visible:ring-[3px] ring-ring/50 border-ring`
    /// ring. The returned response reports `clicked()`.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Sets a stable id salt (item children coordinate through it).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the item around its slots.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_item_colors(&p, self.variant);
        let size = self
            .size
            .or_else(|| current_group(ui).map(|g| g.size))
            .unwrap_or_default();
        let (pad_x, pad_y, gap) = item_geometry(theme.style, size);
        let radius = if theme.style == Style::Nova {
            theme.radius.lg_u8()
        } else {
            theme.radius.md_u8()
        };

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_item"),
        };

        // N12: capture last frame's description flag, then clear it for
        // this frame's recording (the description sets it while the content
        // runs, and the media — laid out first — reads it next frame).
        let has_desc_prev: bool = ui
            .ctx()
            .data_mut(|d| d.get_temp(id.with("has_desc")))
            .unwrap_or(false);
        ui.ctx()
            .data_mut(|d| d.insert_temp(id.with("has_desc"), false));

        push_item(
            ui,
            ItemCtx {
                id,
                size,
                has_desc_prev,
            },
        );

        // §2.4: the nova group gap keys off the contained items' sizes, so
        // record this item's resolved size into every enclosing group (the
        // group reads it next frame to pick its gap).
        for g in group_stack(ui) {
            let key = g.id.with("member_sizes");
            let mut sizes: Vec<ItemSize> =
                ui.ctx().data_mut(|d| d.get_temp(key)).unwrap_or_default();
            if !sizes.contains(&size) {
                sizes.push(size);
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, sizes));
        }

        // `[a]:hover:bg-accent/50` (nova `bg-muted`): the hover fill
        // *replaces* the base fill, read from the previous frame's response
        // (one-frame lag, standard egui hover styling).
        let hover_bg = if theme.style == Style::Nova {
            p.muted
        } else {
            color_alpha(p.accent, 0.5)
        };
        let hovered = self.interactive
            && ui
                .ctx()
                .read_response(id)
                .map(|r| r.hovered())
                .unwrap_or(false);
        let fill = if hovered { hover_bg } else { c.fill };

        let inner = egui::Frame::new()
            .fill(fill)
            .stroke(c.stroke)
            .corner_radius(egui::CornerRadius::same(radius))
            .inner_margin(Margin {
                left: pad_x as i8,
                right: pad_x as i8,
                top: pad_y as i8,
                bottom: pad_y as i8,
            })
            .show(ui, |ui| {
                // N13: the item root is `flex flex-wrap` — the wrapping row
                // lets media, content and actions wrap onto new lines in
                // document order, and header/footer take full lines. The
                // same gap applies between slots on a line and between
                // wrapped lines (CSS `gap` covers both axes).
                //
                // ITM-01: do NOT use `ui.horizontal_wrapped` here — it
                // starts the row at the 36px `interact_size` height, and
                // children centre in that frame even when the row's real
                // height is smaller. Start the row at height 0 instead, so
                // it grows from its content (row height = max child
                // height, as in the source's `flex flex-wrap
                // items-center`). Cross-align stays Center
                // (`items-center`); only the initial height changes.
                let row_layout = if ui.layout().prefer_right_to_left() {
                    egui::Layout::right_to_left(egui::Align::Center)
                } else {
                    egui::Layout::left_to_right(egui::Align::Center)
                }
                .with_main_wrap(true);
                ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 0.0), row_layout, |ui| {
                    // ITM-02: the `Frame` sizes to its content, so without
                    // this the item sizes to content width. The source is
                    // block-level (`ItemGroup` is `flex flex-col`,
                    // `align-items: stretch`), so the item must stretch to
                    // the available width. `set_min_width` only expands
                    // the row's min_rect (it ignores wrap/justify), so the
                    // wrapping bookkeeping `ItemContent` relies on
                    // (`available_rect_before_wrap`) is unaffected.
                    ui.set_min_width(ui.available_width());
                    ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
                    content(ui)
                })
                .inner
            });

        pop_item(ui);

        let sense = if self.interactive {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(inner.response.rect, id, sense);

        // Native `<a>` behaviour (spec §7 + D4, ITM-03): a clicked
        // anchor row holds keyboard focus afterwards, so Space/Enter
        // then activate it. egui's `interact` does not move focus on
        // pointer press, so request it explicitly on click — the
        // pattern `Button` uses (`button.rs:1175-1177`). Requesting
        // before painting lets `has_focus()` below see the focus on
        // the click frame itself. Non-interactive items never take
        // focus.
        //
        // Nested-button note: egui's click is exclusive — the hit
        // test takes the last-registered widget on a tie, so the item
        // (whose `interact` runs after its children) wins the click
        // over a nested button, and the button's own
        // `request_focus()` never runs. The item taking focus here
        // therefore steals nothing from the button: before this fix a
        // click on Launch left focus nowhere; now it focuses the row,
        // and Space/Enter toggle it.
        if self.interactive && response.clicked() {
            response.request_focus();
        }

        if ui.is_rect_visible(response.rect) && response.has_focus() {
            // `focus-visible:border-ring`: recolor the border, then the 3px
            // `ring-ring/50` outline.
            ui.painter().rect_stroke(
                response.rect,
                egui::CornerRadius::same(radius),
                Stroke::new(1.0, p.ring),
                egui::epaint::StrokeKind::Inside,
            );
            crate::utils::focus::paint_focus_ring(ui, &response, radius as f32, true);
        }

        InnerResponse::new(inner.inner, response)
    }
}

/// `ItemMedia` — the leading media slot. [`ItemMedia::icon`] draws the
/// `size-8 rounded-sm border bg-muted` icon box; [`ItemMedia::image`] the
/// `size-10 rounded-sm` image box.
#[derive(Clone, Debug)]
pub struct ItemMedia {
    icon: Option<Icon>,
    image: Option<ImageSource<'static>>,
    id_salt: Option<String>,
}

impl Default for ItemMedia {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemMedia {
    /// Creates an empty (transparent) media slot.
    pub fn new() -> Self {
        Self {
            icon: None,
            image: None,
            id_salt: None,
        }
    }

    /// The icon media variant: `size-8 rounded-sm border bg-muted` box with
    /// a `size-4` glyph.
    pub fn icon(icon: Icon) -> Self {
        Self {
            icon: Some(icon),
            image: None,
            id_salt: None,
        }
    }

    /// The image media variant: `size-10 rounded-sm` box, texture painted
    /// aspect-cover (URI sources degrade to a muted box until loaded).
    pub fn image(image: impl Into<ImageSource<'static>>) -> Self {
        Self {
            icon: None,
            image: Some(image.into()),
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the media slot.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // `rounded-sm` = 0.6 × --radius (no sm_u8 helper on RadiusTokens).
        let radius = theme.radius.sm().round() as u8;
        // N12: when the enclosing item has a description (flag from last
        // frame), the media shifts 2px down and top-aligns (`self-start`).
        let shifted = current_item(ui).map(|c| c.has_desc_prev).unwrap_or(false);

        if let Some(icon) = self.icon {
            let (rect, response) = Self::allocate_media_box(ui, ITEM_MEDIA_ICON_BOX, shifted);
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.rect_filled(rect, radius, p.muted);
                painter.rect_stroke(
                    rect,
                    radius,
                    Stroke::new(1.0, p.border),
                    egui::epaint::StrokeKind::Inside,
                );
                let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(ITEM_MEDIA_ICON));
                icon.paint(
                    painter,
                    icon_rect,
                    p.foreground,
                    2.0 * ITEM_MEDIA_ICON / 24.0,
                );
            }
            return response;
        }

        if let Some(source) = self.image.as_ref() {
            let size = match current_item(ui).map(|c| c.size).unwrap_or_default() {
                ItemSize::Xs => 24.0, // nova xs size-6
                ItemSize::Sm => 32.0, // nova sm size-8
                ItemSize::Default => ITEM_MEDIA_IMAGE_SIZE,
            };
            let (rect, response) = Self::allocate_media_box(ui, size, shifted);
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.rect_filled(rect, radius, p.muted);
                if let Some(tex) = resolve_texture(ui, source, size) {
                    let uv = cover_uv(tex.size, rect.size());
                    painter.add(
                        egui::epaint::RectShape::filled(rect, radius, Color32::WHITE)
                            .with_texture(tex.id, uv),
                    );
                }
            }
            return response;
        }

        // Transparent media slot: no intrinsic size.
        ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1
    }

    /// Allocates the media box and returns the rect to paint plus the
    /// response. When `shifted` (N12) the box is positioned at the current
    /// line's top plus 2px (`self-start` + `translate-y-0.5`); the
    /// allocated rect IS the painted box, so the returned response matches
    /// the visual position. Otherwise the box is allocated (and vertically
    /// centred) by the row layout, as before.
    fn allocate_media_box(ui: &mut Ui, size: f32, shifted: bool) -> (Rect, egui::Response) {
        if !shifted {
            return ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        }
        // `self-start`: the row centres children, so position the box
        // explicitly at the current line's top, translated 2px down. If
        // too little room is left on this line, move to a fresh line first
        // (the wrapping row's `end_row`).
        let line_w = ui.available_width();
        if ui.available_rect_before_wrap().width() < size
            && ui.available_rect_before_wrap().width() < line_w
        {
            ui.end_row();
        }
        // ITM-01: do NOT allocate the shifted rect with `allocate_rect`.
        // The wrapping row takes the first explicit rect as the new row's
        // anchor (`Layout::advance_after_rects`), so the row top moves
        // down to `cursor.top() + 2` and the content — and the frame —
        // grow 2px. Allocate the box normally instead, so the row top
        // stays anchored, then re-interact at the shifted rect so the
        // response matches the painted box (the earlier registration is
        // overwritten by id).
        let row_top = ui.cursor().top();
        let (_, alloc) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        let rect = Rect::from_min_size(
            Pos2::new(alloc.rect.min.x, row_top + 2.0),
            Vec2::splat(size),
        );
        let response = ui.interact(rect, alloc.id, Sense::hover());
        (rect, response)
    }
}

/// `ItemContent` — the flexible middle column (`flex-1 flex-col gap-1`).
/// It yields the width the item's [`ItemActions`] reserve (measured last
/// frame, self-correcting), and grows to fill the row, pushing the actions
/// to the item's right edge (ITM-05, S9 `dd546a5`: `ui.set_min_width(w)` —
/// the wrapping row advances by each child's `min_rect`, and without it
/// the content sized to max-content and the actions sat mid-frame).
/// A second `ItemContent` is NOT `flex-none` (spec §2.7) — a recorded
/// limitation: every content grows the same way.
#[derive(Clone, Debug, Default)]
pub struct ItemContent {
    id_salt: Option<String>,
}

impl ItemContent {
    /// Creates an item content slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the content slot.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // Reserve the width measured for trailing actions (two-frame).
        let (reserved, gap, size) = current_item(ui)
            .map(|c| {
                let theme = ShadcnTheme::of_ui(ui);
                let actions_w: f32 = ui
                    .ctx()
                    .data_mut(|d| d.get_temp(c.id.with("actions_w")))
                    .unwrap_or(0.0);
                (actions_w, item_geometry(theme.style, c.size).2, c.size)
            })
            .unwrap_or((0.0, 10.0, ItemSize::default()));
        // N13: in the wrapping item row `available_width()` is the FULL
        // line width while `available_rect_before_wrap()` is what is left
        // on the current line. Size the content to the remaining space
        // (leaving the actions reservation); if too little room is left,
        // ask for the full-line width so the wrapping layout moves the
        // content to the next line instead of squeezing it. In a
        // non-wrapping parent both widths are the remaining width, so this
        // reduces to the old behaviour.
        let line_w = ui.available_width();
        let remaining = ui.available_rect_before_wrap().width();
        let mut w = (remaining - reserved - gap).max(0.0);
        if w < MIN_CONTENT_WIDTH && remaining < line_w {
            w = (line_w - reserved - gap).max(0.0);
        }
        // §2.7: the content gap is `gap-1` (4px), except 0 at `xs` under
        // nova.
        let content_gap = if size == ItemSize::Xs && ShadcnTheme::of_ui(ui).style == Style::Nova {
            0.0
        } else {
            4.0
        };
        // ITM-01: do NOT allocate the content at height 0 — the row's
        // Center cross-align centres the child in the row's current
        // height, which (before the root fix) is the 36px row floor and
        // pushes the content ~18px down. Start the allocation at the
        // row's current height (`ui.cursor().height()` — the tallest
        // thing allocated on the line so far, i.e. the media box, or 0
        // with no media) so it starts at the row top.
        // ITM-05 (spec §2.7 `flex-1`): the content column must grow to
        // fill the row, pushing the actions to the item's right edge.
        // The allocated width `w` alone does not do that — the
        // wrapping row advances by each child's `min_rect` (egui 0.37
        // `ui.rs:2337-2338` `scope_dyn` → `advance_cursor_after_rect`,
        // `layout.rs:751-754`), and this Ui's min_rect is its
        // max-content width (title `w-fit` + description galley), so
        // the actions landed immediately after the content, ~545px
        // from the right edge. Expand the min_rect to `w` — the same
        // `set_min_width` pattern ITM-02 uses at the root row (it only
        // expands the min_rect; the wrap/justify bookkeeping and the
        // title/description wrap budgets, which read
        // `available_width()`, are unaffected).
        let inner = ui.allocate_ui(Vec2::new(w, ui.cursor().height()), |ui| {
            ui.set_min_width(w);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = content_gap;
                content(ui)
            })
            .inner
        });
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemTitle` — `text-sm font-medium leading-snug` (`flex w-fit
/// items-center gap-2`, so icons can be hosted next to the text via
/// [`ItemTitle::show_with`]).
#[derive(Clone, Debug)]
pub struct ItemTitle {
    text: String,
}

impl ItemTitle {
    /// Creates an item title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The title font: `font-medium` (500) at `text-sm` (14px) — the
    /// embedded Inter Medium face.
    pub(crate) fn font() -> FontId {
        crate::theme::fonts::medium(crate::theme::tokens::type_scale::SM)
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        self.show_with(ui, |_| {}).response
    }

    /// Renders the title with extra content beside the text (the source's
    /// `flex w-fit items-center gap-2` allows e.g. an icon or badge).
    ///
    /// N11: the title is `flex w-fit` — it wraps when the content column
    /// is narrower than the text's max-content width, and the hosted
    /// content stays on the same flex line, vertically centred. The wrap
    /// budget is the available width minus the hosted content's measured
    /// width (two-frame, same pattern as the actions-width reservation);
    /// frame 1 uses the full width and self-corrects.
    pub fn show_with<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = crate::theme::tokens::type_scale::SM;
        // `font-medium` (500): the embedded Inter Medium face.
        let font = Self::font();
        // `leading-snug` is set explicitly through the TextFormat.
        let format = egui::TextFormat {
            font_id: font,
            color: p.foreground,
            line_height: Some(size * TITLE_LINE_HEIGHT),
            ..egui::TextFormat::default()
        };
        let icon_key = ui.auto_id_with("shadcn_item_title_icon_w");
        let icon_w: f32 = ui.ctx().data_mut(|d| d.get_temp(icon_key)).unwrap_or(0.0);
        let avail = ui.available_width();
        let gap = 8.0; // gap-2
        let text_wrap = if icon_w > 0.0 {
            (avail - icon_w - gap).max(0.0)
        } else {
            avail
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = text_wrap;
        let galley = ui.painter().layout_job(job);
        // ITM-01: do NOT use `ui.horizontal` here — it starts the row at
        // the 36px `interact_size` height, so the title row is 36px tall
        // instead of the title's line height (19.25px at 14px /
        // `leading-snug`). Start the row at the galley's height instead;
        // it grows to the hosted content when that is taller, and the
        // Center cross-align keeps hosted content vertically centred on
        // the title (the source's `flex items-center`).
        let title_layout = if ui.layout().prefer_right_to_left() {
            egui::Layout::right_to_left(egui::Align::Center)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };
        let inner =
            ui.allocate_ui_with_layout(Vec2::new(avail, galley.size().y), title_layout, |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
                if ui.is_rect_visible(rect) {
                    ui.painter().galley(rect.min, galley.clone(), p.foreground);
                }
                content(ui)
            });
        // Measure the hosted content's width for next frame's wrap budget.
        let row_w = inner.response.rect.width();
        let measured = (row_w - galley.size().x - gap).max(0.0);
        ui.ctx().data_mut(|d| d.insert_temp(icon_key, measured));
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemDescription` — `text-sm leading-normal text-muted-foreground`
/// (nova `group-data-[size=xs]/item:text-xs`), wraps, and clamps to two
/// lines with an ellipsis (`line-clamp-2`). A link segment
/// ([`ItemDescription::link`]) renders underlined with a 4px offset and
/// recolours to `text-primary` on hover (N10).
#[derive(Clone, Debug, Default)]
pub struct ItemDescription {
    segments: Vec<DescriptionSegment>,
}

impl ItemDescription {
    /// Creates an item description (a single plain-text segment).
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            segments: vec![DescriptionSegment::Text(text.into())],
        }
    }

    /// Appends a link segment: a direct `<a>` child of the description,
    /// underlined with a 4px offset, `text-primary` on hover (N10).
    pub fn link(mut self, text: impl Into<String>) -> Self {
        self.segments.push(DescriptionSegment::Link(text.into()));
        self
    }

    /// Appends a plain-text segment.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.segments.push(DescriptionSegment::Text(text.into()));
        self
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        // N12: record that this item has a description descendant (the
        // media reads it next frame to apply its shift).
        if let Some(ctx) = current_item(ui) {
            ui.ctx()
                .data_mut(|d| d.insert_temp(ctx.id.with("has_desc"), true));
        }
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // nova shrinks the description to `text-xs` inside xs items.
        let size = match current_item(ui).map(|c| c.size) {
            Some(ItemSize::Xs) if theme.style == Style::Nova => {
                crate::theme::tokens::type_scale::XS
            }
            _ => crate::theme::tokens::type_scale::SM,
        };
        let width = ui.available_width().max(0.0);
        // D9: clamp to two lines with a trailing "…" when the text
        // overflows.
        let segments = truncate_segments(
            &self.segments,
            size,
            width,
            2,
            ui,
            p.muted_foreground,
            p.muted_foreground,
        );
        // Pass 1: resting colours, to learn the link geometry.
        let mut job = description_job(&segments, size, p.muted_foreground, p.muted_foreground);
        job.wrap.max_width = width;
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        let runs = link_runs(&galley, &segments);
        let hovered = response.hover_pos().is_some_and(|pos| {
            runs.iter()
                .any(|run| run.hit.translate(rect.min.to_vec2()).contains(pos))
        });
        // Pass 2 (only when a link is hovered): recolour the links to
        // `text-primary`. Geometry is identical — only colours changed.
        let (galley, runs) = if hovered {
            let mut job = description_job(&segments, size, p.muted_foreground, p.primary);
            job.wrap.max_width = width;
            let galley = ui.painter().layout_job(job);
            let runs = link_runs(&galley, &segments);
            (galley, runs)
        } else {
            (galley, runs)
        };
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.muted_foreground);
            let link_color = if hovered {
                p.primary
            } else {
                p.muted_foreground
            };
            for run in &runs {
                ui.painter().line_segment(
                    [rect.min + run.from.to_vec2(), rect.min + run.to.to_vec2()],
                    Stroke::new(1.0, link_color),
                );
            }
        }
        response
    }
}

/// `ItemActions` — the trailing actions row (`gap-2`). Inside an [`Item`],
/// it reserves its width so [`ItemContent`] flexes around it; inside an
/// [`ItemHeader`]/[`ItemFooter`] it right-aligns (`justify-between`).
#[derive(Clone, Debug, Default)]
pub struct ItemActions {
    id_salt: Option<String>,
}

impl ItemActions {
    /// Creates an item actions slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the actions.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        if let Some(row) = current_row(ui) {
            // Right-align at the header/footer row's right edge.
            let region = Rect::from_min_max(
                Pos2::new(row.left, row.top),
                Pos2::new(row.right, row.top + 400.0),
            );
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(region)
                    .layout(egui::Layout::right_to_left(egui::Align::Min)),
            );
            let ret = content(&mut child);
            // ITM-04: the actions lay out in this separate child Ui, so
            // their height never grows the header/footer row itself (the
            // row only sees its direct children). Record the measured
            // height so the row can reserve it next frame — the same
            // two-frame pattern as the `actions_w` reservation below
            // (ITM-02). `min_rect()` is this frame's content rect;
            // `child.response()` would read last frame's rect.
            if let Some(item) = current_item(ui) {
                let h = child.min_rect().height();
                ui.ctx()
                    .data_mut(|d| d.insert_temp(item.id.with(row.actions_h_key), h));
            }
            return InnerResponse::new(ret, child.response());
        }

        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0; // gap-2
            content(ui)
        });
        // Record the measured width so ItemContent can flex next frame.
        if let Some(ctx) = current_item(ui) {
            let w = inner.response.rect.width();
            ui.ctx()
                .data_mut(|d| d.insert_temp(ctx.id.with("actions_w"), w));
        }
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemHeader` — a full-width row above the main slots
/// (`items-center justify-between gap-2`). It sizes to its content height
/// (ITM-04, S9 `1117d1d`): the row reserves the measured actions height
/// and settles from frame 2; §7 header is 20 nova / 22 vega.
#[derive(Clone, Debug, Default)]
pub struct ItemHeader {
    id_salt: Option<String>,
}

impl ItemHeader {
    /// Creates an item header row.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header row.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // N13: header/footer are `basis-full` full lines. In the wrapping
        // item row `available_width()` is the full line width; if we are
        // mid-line (less than that remains), move to a fresh line first so
        // the row occupies a full line instead of the remaining width. In a
        // non-wrapping parent both widths are the remaining width, so this
        // is a no-op there.
        let full_w = ui.available_width();
        if ui.available_rect_before_wrap().width() < full_w {
            ui.end_row();
        }
        let cursor = ui.cursor();
        push_row(
            ui,
            RowCtx {
                left: cursor.left(),
                top: cursor.top(),
                right: cursor.left() + full_w,
                actions_h_key: "header_actions_h",
            },
        );
        // ITM-04: the actions' height, measured last frame (see
        // `ItemActions::show`); 0.0 on the first frame, so frame 1
        // sizes to the direct children only (one-frame settle, same
        // class as the `actions_w` reservation / alert D7).
        let actions_h: f32 = current_item(ui)
            .and_then(|item| {
                ui.ctx()
                    .data_mut(|d| d.get_temp(item.id.with("header_actions_h")))
            })
            .unwrap_or(0.0);
        let inner = {
            // ITM-04: do NOT use `ui.horizontal` here — it starts the row
            // at the 36px `interact_size` height, so the header row is
            // 36px tall instead of its content height (max(title, badge)
            // — 20px in nova, 22px in vega). Start the row at the reserved actions
            // height instead (the badge lays out in a separate child Ui,
            // so it never grows the row itself — see `ItemActions::show`),
            // the same way ITM-01 fixed the root/title rows; the row
            // grows to its tallest child, and the Center cross-align
            // keeps the title vertically centred on the badge (the
            // source's `flex items-center`). RTL and no-wrap match
            // `ui.horizontal`.
            let header_layout = if ui.layout().prefer_right_to_left() {
                egui::Layout::right_to_left(egui::Align::Center)
            } else {
                egui::Layout::left_to_right(egui::Align::Center)
            };
            ui.allocate_ui_with_layout(Vec2::new(full_w, actions_h), header_layout, |ui| {
                ui.set_min_width(ui.available_width()); // basis-full
                ui.spacing_mut().item_spacing.x = 8.0;
                let ret = content(ui);
                // ITM-04: the title lays out in a nested child Ui (ITM-01),
                // so the row advances by the title's own 19px rect, not the
                // 20px frame the badge needs — the frame's 0.5px top/bottom
                // is lost and the row lands 0.5px short of the model. A
                // zero-width spacer at the reserved actions height forces
                // the row's min_rect to span the full row height: for a
                // Center cross-aligned row the spacer's frame fills the
                // whole row height and `advance_after_rects` unions the
                // frame into the min_rect. It is 0px wide, so nothing
                // shifts horizontally. On frame 1 `actions_h` is 0 and the
                // spacer is a no-op.
                ui.allocate_space(Vec2::new(0.0, actions_h));
                ret
            })
        };
        pop_row(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemFooter` — a full-width row below the main slots
/// (`items-center justify-between gap-2`). It sizes to its content height
/// (ITM-04, S9 `1117d1d`): the row reserves the measured actions height
/// and settles from frame 2; §7 footer is the Sm button's outer rect,
/// 28 nova / 32 vega.
#[derive(Clone, Debug, Default)]
pub struct ItemFooter {
    id_salt: Option<String>,
}

impl ItemFooter {
    /// Creates an item footer row.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer row.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // N13: same full-line rule as the header (see above).
        let full_w = ui.available_width();
        if ui.available_rect_before_wrap().width() < full_w {
            ui.end_row();
        }
        let cursor = ui.cursor();
        push_row(
            ui,
            RowCtx {
                left: cursor.left(),
                top: cursor.top(),
                right: cursor.left() + full_w,
                actions_h_key: "footer_actions_h",
            },
        );
        // ITM-04: the actions' height, measured last frame (see
        // `ItemActions::show`); 0.0 on the first frame (one-frame
        // settle, as in the header).
        let actions_h: f32 = current_item(ui)
            .and_then(|item| {
                ui.ctx()
                    .data_mut(|d| d.get_temp(item.id.with("footer_actions_h")))
            })
            .unwrap_or(0.0);
        let inner = {
            // ITM-04: same fix as the header (see above) — the footer row
            // is the Sm button's outer rect (28px in nova, 32px in vega;
            // its painted face is 26px), not the 36px `interact_size`
            // floor `ui.horizontal` starts at. The button lays out in a
            // separate child Ui, so the row reserves its height from
            // last frame instead of growing to it.
            let footer_layout = if ui.layout().prefer_right_to_left() {
                egui::Layout::right_to_left(egui::Align::Center)
            } else {
                egui::Layout::left_to_right(egui::Align::Center)
            };
            ui.allocate_ui_with_layout(Vec2::new(full_w, actions_h), footer_layout, |ui| {
                ui.set_min_width(ui.available_width());
                ui.spacing_mut().item_spacing.x = 8.0;
                let ret = content(ui);
                // ITM-04: same zero-width spacer as the header — the
                // label allocates directly, so its frame already spans
                // the row height, but the spacer keeps the two rows
                // symmetric and guards the row if the label ever lays
                // out in a nested child. On frame 1 `actions_h` is 0
                // and the spacer is a no-op.
                ui.allocate_space(Vec2::new(0.0, actions_h));
                ret
            })
        };
        pop_row(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemGroup` — a vertical list of items with a per-size gap
/// (nova `gap-4` default, `gap-2.5` sm, `gap-2` xs).
#[derive(Clone, Debug, Default)]
pub struct ItemGroup {
    size: ItemSize,
    id_salt: Option<String>,
}

impl ItemGroup {
    /// Creates an item group.
    pub fn new() -> Self {
        Self {
            size: ItemSize::default(),
            id_salt: None,
        }
    }

    /// Sets the group size (items inherit it unless overridden).
    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = size;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its items.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        // §2.4: the nova group gap keys off the contained items' sizes
        // (recorded last frame); fall back to the group's size hint on the
        // first frame or when the group has no items yet. The recording is
        // cleared here so this frame's items repopulate it.
        let id = ui.auto_id_with("shadcn_item_group");
        let recorded: Vec<ItemSize> = ui
            .ctx()
            .data_mut(|d| d.get_temp(id.with("member_sizes")))
            .unwrap_or_default();
        ui.ctx()
            .data_mut(|d| d.insert_temp(id.with("member_sizes"), Vec::<ItemSize>::new()));
        let g = resolve_group_gap(theme.style, self.size, &recorded);
        push_group(
            ui,
            GroupCtx {
                size: self.size,
                id,
            },
        );
        let inner = ui.vertical(|ui| {
            // ITM-02: the group is `flex flex-col` (block-level), so it
            // stretches to the available width instead of sizing to its
            // widest item. The items inside stretch to the group's width
            // via the `Item::show` min-width (see there).
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = g;
            content(ui)
        });
        pop_group(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemSeparator` — a horizontal [`crate::components::separator::Separator`]
/// between items (nova `my-2`).
#[derive(Clone, Debug, Default)]
pub struct ItemSeparator {
    id_salt: Option<String>,
}

impl ItemSeparator {
    /// Creates an item separator.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the separator.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let nova = theme.style == Style::Nova;
        if nova {
            ui.add_space(8.0); // my-2
        }
        let mut sep = crate::components::separator::Separator::new();
        if let Some(salt) = self.id_salt.as_deref() {
            sep = sep.id_salt(salt);
        }
        let response = sep.show(ui);
        if nova {
            ui.add_space(8.0);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn geometry_matches_style_recipes() {
        // nova default/sm px-3 py-2.5 gap-2.5; xs px-2.5 py-2 gap-2.
        assert_eq!(
            item_geometry(Style::Nova, ItemSize::Default),
            (12.0, 10.0, 10.0)
        );
        assert_eq!(item_geometry(Style::Nova, ItemSize::Sm), (12.0, 10.0, 10.0));
        assert_eq!(item_geometry(Style::Nova, ItemSize::Xs), (10.0, 8.0, 8.0));
        // base default p-4 gap-4; sm px-4 py-3 gap-2.5.
        assert_eq!(
            item_geometry(Style::Vega, ItemSize::Default),
            (16.0, 16.0, 16.0)
        );
        assert_eq!(item_geometry(Style::Vega, ItemSize::Sm), (16.0, 12.0, 10.0));
        // group gaps: nova gap-4 / gap-2.5 / gap-2; the base group is a
        // plain flex column with no gap.
        assert_eq!(group_gap(Style::Nova, ItemSize::Default), 16.0);
        assert_eq!(group_gap(Style::Nova, ItemSize::Sm), 10.0);
        assert_eq!(group_gap(Style::Nova, ItemSize::Xs), 8.0);
        assert_eq!(group_gap(Style::Vega, ItemSize::Default), 0.0);
        assert_eq!(group_gap(Style::Vega, ItemSize::Sm), 0.0);
    }

    #[test]
    fn interactive_items_sense_clicks() {
        let ctx = egui::Context::default();
        let mut plain_senses_clicks = true;
        let mut interactive_senses_clicks = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let root = egui::Id::unique("item_interactive_test");
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                root,
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            Item::new().id_salt("plain_item").show(ui, |ui| {
                ItemTitle::new("Plain").show(ui);
            });
            Item::new()
                .interactive(true)
                .id_salt("click_item")
                .show(ui, |ui| {
                    ItemTitle::new("Clickable").show(ui);
                });
            let plain = ui.ctx().read_response(root.with("plain_item"));
            let clickable = ui.ctx().read_response(root.with("click_item"));
            if let Some(r) = plain {
                plain_senses_clicks = r.sense.senses_click();
            }
            if let Some(r) = clickable {
                interactive_senses_clicks = r.sense.senses_click();
            }
        });
        out.textures_delta.clear();
        assert!(!plain_senses_clicks);
        assert!(interactive_senses_clicks);
    }

    #[test]
    fn variants_resolve_colors() {
        let p = palette(Mode::Light);
        assert_eq!(
            resolve_item_colors(&p, ItemVariant::Default).fill,
            Color32::TRANSPARENT
        );
        assert_eq!(
            resolve_item_colors(&p, ItemVariant::Muted).fill,
            color_alpha(p.muted, 0.5)
        );
        let outline = resolve_item_colors(&p, ItemVariant::Outline);
        assert_eq!(outline.stroke.color, p.border);
        assert_eq!(outline.fill, Color32::TRANSPARENT);
    }

    #[test]
    fn media_constants_match_source() {
        assert_eq!(ITEM_MEDIA_ICON_BOX, 32.0); // size-8
        assert_eq!(ITEM_MEDIA_IMAGE_SIZE, 40.0); // size-10
        assert_eq!(ITEM_MEDIA_ICON, 16.0); // size-4
    }

    #[test]
    fn renders_items_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("item_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
                );
                ItemGroup::new().show(ui, |ui| {
                    Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
                        ItemMedia::icon(Icon::User).show(ui);
                        ItemContent::new().show(ui, |ui| {
                            ItemTitle::new("Ada Lovelace").show(ui);
                            ItemDescription::new("Writes the first programs.").show(ui);
                        });
                        ItemActions::new().show(ui, |ui| {
                            ui.label("follow");
                        });
                    });
                    Item::new().variant(ItemVariant::Muted).show(ui, |ui| {
                        ItemContent::new().show(ui, |ui| {
                            ItemTitle::new("Plain muted").show(ui);
                        });
                    });
                    ItemSeparator::new().show(ui);
                    Item::new().show(ui, |ui| {
                        ItemHeader::new().show(ui, |ui| {
                            ItemTitle::new("Team settings").show(ui);
                            ItemActions::new().show(ui, |ui| {
                                ui.label("edit");
                            });
                        });
                        ItemContent::new().show(ui, |ui| {
                            ItemDescription::new("Invite members, manage roles.").show(ui);
                        });
                        ItemFooter::new().show(ui, |ui| {
                            ui.label("footer");
                        });
                    });
                });
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn title_uses_medium_face() {
        assert_eq!(ItemTitle::font(), crate::theme::fonts::medium(14.0));
        assert_eq!(
            ItemTitle::font().family,
            crate::theme::fonts::medium_family()
        );
    }

    #[test]
    fn group_gap_resolves_from_member_sizes() {
        // §2.4: the nova gap keys off the contained items' sizes (xs wins).
        assert_eq!(
            resolve_group_gap(Style::Nova, ItemSize::Default, &[ItemSize::Default]),
            16.0
        );
        assert_eq!(
            resolve_group_gap(Style::Nova, ItemSize::Default, &[ItemSize::Sm]),
            10.0
        );
        assert_eq!(
            resolve_group_gap(Style::Nova, ItemSize::Default, &[ItemSize::Xs]),
            8.0
        );
        // xs wins on mixed sizes, even when the group hint is Default.
        assert_eq!(
            resolve_group_gap(
                Style::Nova,
                ItemSize::Default,
                &[ItemSize::Sm, ItemSize::Xs]
            ),
            8.0
        );
        assert_eq!(
            resolve_group_gap(
                Style::Nova,
                ItemSize::Default,
                &[ItemSize::Default, ItemSize::Sm]
            ),
            10.0
        );
        // First frame / empty group falls back to the group's own size hint.
        assert_eq!(resolve_group_gap(Style::Nova, ItemSize::Sm, &[]), 10.0);
        assert_eq!(resolve_group_gap(Style::Nova, ItemSize::Xs, &[]), 8.0);
        // Base styles have no group gap.
        assert_eq!(
            resolve_group_gap(Style::Vega, ItemSize::Default, &[ItemSize::Xs]),
            0.0
        );
    }

    /// Renders `build` in a root Ui of the given size twice on the same
    /// context (so two-frame values — the description flag, the icon-width
    /// reservation, the group-gap recording — settle) and returns the
    /// (first, second) outputs.
    fn render_twice<T>(width: f32, height: f32, build: impl Fn(&mut Ui) -> T) -> (T, T) {
        render_twice_with_preset(Preset::default(), width, height, build)
    }

    /// Like [`render_twice`], but with an explicit preset (so a non-nova
    /// style can be measured headlessly).
    fn render_twice_with_preset<T>(
        preset: Preset,
        width: f32,
        height: f32,
        build: impl Fn(&mut Ui) -> T,
    ) -> (T, T) {
        let ctx = egui::Context::default();
        preset.build(Mode::Light).install(&ctx);
        let run = |build: &dyn Fn(&mut Ui) -> T| -> T {
            let mut out_val: Option<T> = None;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("item_test_root"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, height))),
                );
                out_val = Some(build(ui));
            });
            out.textures_delta.clear();
            out_val.expect("render produced a value")
        };
        let first = run(&build);
        let second = run(&build);
        (first, second)
    }

    #[test]
    fn title_wraps_at_content_width() {
        // N11: a long title in a narrow content column wraps instead of
        // overflowing it.
        let title_rect = render_twice(200.0, 400.0, |ui| {
            let mut title = Rect::NOTHING;
            Item::new().id_salt("wrap_title").show(ui, |ui| {
                ItemMedia::icon(Icon::User).show(ui);
                ItemContent::new().show(ui, |ui| {
                    title = ItemTitle::new(
                        "Supercalifragilisticexpialidocious and more words here for wrapping",
                    )
                    .show(ui)
                    .rect;
                    ItemDescription::new("Short.").show(ui);
                });
            });
            title
        })
        .1;
        // Content width: 200 - 2×12 pad - 32 media - 2×10 gap = 124.
        assert!(
            title_rect.width() <= 124.5,
            "title must not overflow the content column, got width {}",
            title_rect.width()
        );
        // More than one line: the row is taller than the 36px one-line
        // minimum.
        assert!(
            title_rect.height() > 36.5,
            "title must wrap to more than one line, got height {}",
            title_rect.height()
        );
    }

    #[test]
    fn title_icon_stays_centred_on_row() {
        // N11: the hosted content stays on the title's flex line,
        // vertically centred (`items-center`). Run twice so the icon-width
        // reservation settles.
        let (title_rect, icon_rect) = render_twice(200.0, 400.0, |ui| {
            let mut title = Rect::NOTHING;
            let mut icon = Rect::NOTHING;
            Item::new().id_salt("icon_title").show(ui, |ui| {
                ItemMedia::icon(Icon::User).show(ui);
                ItemContent::new().show(ui, |ui| {
                    let r = ItemTitle::new(
                        "Supercalifragilisticexpialidocious and more words here for wrapping",
                    )
                    .show_with(ui, |ui| {
                        icon = ui.label("NEW").rect;
                    });
                    title = r.response.rect;
                });
            });
            (title, icon)
        })
        .1;
        assert!(
            title_rect.height() > 36.5,
            "title must wrap, got height {}",
            title_rect.height()
        );
        assert!(
            (icon_rect.center().y - title_rect.center().y).abs() < 1.0,
            "icon must be vertically centred on the title row"
        );
        // The icon stays inside the content column (right edge = 200 - 12).
        assert!(
            icon_rect.right() <= 188.5,
            "icon must stay inside the content column, got right {}",
            icon_rect.right()
        );
    }

    #[test]
    fn media_shifts_with_description() {
        // N12: image media (40px) + icon media (32px) + title +
        // description. Frame 1 the medias are centred in the row; frame 2
        // (flag set) both sit at the line top + 2px (`translate-y-0.5` +
        // `self-start`).
        let (first, second) = render_twice(400.0, 600.0, |ui| {
            let tex = ui.ctx().load_texture(
                "item_test_texture",
                egui::ColorImage::example(),
                egui::TextureOptions::default(),
            );
            let source = egui::ImageSource::Texture(egui::load::SizedTexture::new(
                tex.id(),
                Vec2::splat(ITEM_MEDIA_IMAGE_SIZE),
            ));
            let mut image = Rect::NOTHING;
            let mut icon = Rect::NOTHING;
            let r = Item::new().id_salt("shift_item").show(ui, |ui| {
                image = ItemMedia::image(source).show(ui).rect;
                icon = ItemMedia::icon(Icon::User).show(ui).rect;
                ItemContent::new().show(ui, |ui| {
                    ItemTitle::new("Ada Lovelace").show(ui);
                    ItemDescription::new("Engineer").show(ui);
                });
            });
            (image, icon, r.response.rect)
        });
        let (_, pad_y, _) = item_geometry(Style::Nova, ItemSize::Default);
        // Frame 2: both medias at the line top + 2px.
        assert!(
            (second.0.top() - (second.2.top() + pad_y + 2.0)).abs() < 0.5,
            "image media top must be item top + pad_y + 2, got {} vs {}",
            second.0.top(),
            second.2.top() + pad_y + 2.0
        );
        assert!(
            (second.1.top() - (second.2.top() + pad_y + 2.0)).abs() < 0.5,
            "icon media top must be item top + pad_y + 2, got {} vs {}",
            second.1.top(),
            second.2.top() + pad_y + 2.0
        );
        // The image moves down 2px from its centred position; the icon
        // moves up from its centred position to the line top.
        assert!(
            (second.0.top() - first.0.top() - 2.0).abs() < 0.5,
            "image media must shift down 2px, {} -> {}",
            first.0.top(),
            second.0.top()
        );
        assert!(
            second.1.top() < first.1.top(),
            "icon media must move up to the line top, {} -> {}",
            first.1.top(),
            second.1.top()
        );
    }

    #[test]
    fn media_does_not_shift_without_description() {
        // N12: with no description the media stays where the row layout
        // puts it (no shift on either frame).
        let (first, second) = render_twice(400.0, 600.0, |ui| {
            let mut media = Rect::NOTHING;
            Item::new().id_salt("noshift_item").show(ui, |ui| {
                media = ItemMedia::icon(Icon::User).show(ui).rect;
                ItemContent::new().show(ui, |ui| {
                    ItemTitle::new("Ada Lovelace").show(ui);
                });
            });
            media
        });
        assert!(
            (second.top() - first.top()).abs() < 0.5,
            "media must not move without a description, {} -> {}",
            first.top(),
            second.top()
        );
    }

    #[test]
    fn root_wraps_header_content_footer() {
        // N13: header and footer are full lines; the content sits on its
        // own line between them (the old `ui.horizontal` gave the content
        // ~0 width and a 600px-tall one-char-per-line description).
        let (header, content, desc, footer, item) = render_twice(400.0, 600.0, |ui| {
            let mut header = Rect::NOTHING;
            let mut content = Rect::NOTHING;
            let mut desc = Rect::NOTHING;
            let mut footer = Rect::NOTHING;
            let r = Item::new().id_salt("wrap_hcf").show(ui, |ui| {
                let h = ItemHeader::new().show(ui, |ui| {
                    ItemTitle::new("Team settings").show(ui);
                });
                header = h.response.rect;
                let c = ItemContent::new().show(ui, |ui| {
                    desc = ItemDescription::new("Invite members, manage roles.")
                        .show(ui)
                        .rect;
                });
                content = c.response.rect;
                let f = ItemFooter::new().show(ui, |ui| {
                    ui.label("footer");
                });
                footer = f.response.rect;
            });
            (header, content, desc, footer, r.response.rect)
        })
        .1;
        // Header and footer are full lines (400 - 2×12 pad = 376).
        assert!(
            (header.width() - 376.0).abs() < 0.5,
            "header width {}",
            header.width()
        );
        assert!(
            (footer.width() - 376.0).abs() < 0.5,
            "footer width {}",
            footer.width()
        );
        // The content starts at the line left, below the header, and its
        // description is one line (21px) — not squeezed to ~0 width.
        assert!(
            (content.left() - header.left()).abs() < 0.5,
            "content left {}",
            content.left()
        );
        assert!(
            content.top() > header.bottom() + 5.0,
            "content below header"
        );
        assert!(
            (desc.height() - 21.0).abs() < 0.5,
            "desc height {}",
            desc.height()
        );
        assert!(
            footer.top() > content.bottom() + 5.0,
            "footer below content"
        );
        assert!(item.height() < 200.0, "item height {}", item.height());
    }

    #[test]
    fn group_gap_uses_member_sizes() {
        // §2.4 end-to-end: the nova group gap is 16/10/8 by the contained
        // items' sizes. The mixed group (Default group, one Sm member)
        // must take the Sm gap (10), not the group's own hint (16).
        let (g_default, g_sm, g_xs, g_mix) = render_twice(400.0, 600.0, |ui| {
            let mut out = (0.0, 0.0, 0.0, 0.0);
            ItemGroup::new().show(ui, |ui| {
                let a = Item::new().id_salt("g_default_1").show(ui, |ui| {
                    ItemTitle::new("One").show(ui);
                });
                let b = Item::new().id_salt("g_default_2").show(ui, |ui| {
                    ItemTitle::new("Two").show(ui);
                });
                out.0 = b.response.rect.top() - a.response.rect.bottom();
            });
            ItemGroup::new().size(ItemSize::Sm).show(ui, |ui| {
                let a = Item::new().id_salt("g_sm_1").show(ui, |ui| {
                    ItemTitle::new("One").show(ui);
                });
                let b = Item::new().id_salt("g_sm_2").show(ui, |ui| {
                    ItemTitle::new("Two").show(ui);
                });
                out.1 = b.response.rect.top() - a.response.rect.bottom();
            });
            ItemGroup::new().size(ItemSize::Xs).show(ui, |ui| {
                let a = Item::new().id_salt("g_xs_1").show(ui, |ui| {
                    ItemTitle::new("One").show(ui);
                });
                let b = Item::new().id_salt("g_xs_2").show(ui, |ui| {
                    ItemTitle::new("Two").show(ui);
                });
                out.2 = b.response.rect.top() - a.response.rect.bottom();
            });
            ItemGroup::new().show(ui, |ui| {
                let a = Item::new()
                    .size(ItemSize::Sm)
                    .id_salt("g_mix_1")
                    .show(ui, |ui| {
                        ItemTitle::new("One").show(ui);
                    });
                let b = Item::new().id_salt("g_mix_2").show(ui, |ui| {
                    ItemTitle::new("Two").show(ui);
                });
                out.3 = b.response.rect.top() - a.response.rect.bottom();
            });
            out
        })
        .1;
        assert!((g_default - 16.0).abs() < 0.5, "default gap {g_default}");
        assert!((g_sm - 10.0).abs() < 0.5, "sm gap {g_sm}");
        assert!((g_xs - 8.0).abs() < 0.5, "xs gap {g_xs}");
        assert!((g_mix - 10.0).abs() < 0.5, "mixed gap {g_mix}");
    }

    #[test]
    fn content_gap_is_zero_at_xs_nova() {
        // §2.7: the content gap is 4px, except 0 at xs under nova.
        let (gap_default, gap_xs) = render_twice(400.0, 600.0, |ui| {
            let mut gap_default = 0.0;
            let mut gap_xs = 0.0;
            Item::new().id_salt("gap_default").show(ui, |ui| {
                ItemContent::new().show(ui, |ui| {
                    let t = ItemTitle::new("T").show(ui);
                    let d = ItemDescription::new("D").show(ui);
                    gap_default = d.rect.top() - t.rect.bottom();
                });
            });
            Item::new()
                .size(ItemSize::Xs)
                .id_salt("gap_xs")
                .show(ui, |ui| {
                    ItemContent::new().show(ui, |ui| {
                        let t = ItemTitle::new("T").show(ui);
                        let d = ItemDescription::new("D").show(ui);
                        gap_xs = d.rect.top() - t.rect.bottom();
                    });
                });
            (gap_default, gap_xs)
        })
        .1;
        assert!(
            (gap_default - 4.0).abs() < 0.5,
            "default content gap {gap_default}"
        );
        assert!((gap_xs - 0.0).abs() < 0.5, "xs-nova content gap {gap_xs}");
    }

    #[test]
    fn description_clamps_to_two_lines() {
        // D9 (pure): `truncate_segments` clamps a long description to two
        // lines with a trailing "…" and leaves a short one unchanged.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("item_clamp_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
            );
            let long = "This is a very long description that will certainly wrap to more \
                        than two lines when laid out in a narrow column.";
            let segments = vec![DescriptionSegment::Text(long.to_string())];
            let truncated = truncate_segments(
                &segments,
                14.0,
                120.0,
                2,
                ui,
                Color32::BLACK,
                Color32::BLACK,
            );
            let text: String = truncated.iter().map(|s| s.text()).collect();
            assert!(
                text.ends_with('…'),
                "clamped text ends with an ellipsis: {text}"
            );
            assert!(text.len() < long.len(), "clamped text is shorter");
            let mut job = description_job(&truncated, 14.0, Color32::BLACK, Color32::BLACK);
            job.wrap.max_width = 120.0;
            let galley = ui.painter().layout_job(job);
            assert!(
                effective_rows(&galley) <= 2,
                "clamped galley is at most 2 rows, got {}",
                effective_rows(&galley)
            );
            let short = vec![DescriptionSegment::Text("Short.".to_string())];
            assert_eq!(
                truncate_segments(&short, 14.0, 120.0, 2, ui, Color32::BLACK, Color32::BLACK),
                short,
                "short text is unchanged"
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn description_renders_at_most_two_lines() {
        // D9 (end-to-end): a long description in a narrow content column
        // renders as exactly two lines (42px), not the full wrapped height.
        let height = render_twice(200.0, 400.0, |ui| {
            let mut h = 0.0;
            Item::new().id_salt("clamp_render").show(ui, |ui| {
                ItemContent::new().show(ui, |ui| {
                    ItemTitle::new("T").show(ui);
                    let r = ItemDescription::new(
                        "This is a very long description that will certainly wrap to more \
                         than two lines when laid out in a narrow column.",
                    )
                    .show(ui);
                    h = r.rect.height();
                });
            });
            h
        })
        .1;
        assert!(
            (height - 42.0).abs() < 0.5,
            "description height is 2 lines, got {height}"
        );
    }

    #[test]
    fn frame_heights_match_spec_model() {
        // ITM-01: the frame height must match the frozen spec model
        // (title + gap + description + padding + 2px border), not the
        // +36.75px overshoot from the 36px horizontal floor plus the 18px
        // centring offset. All cases use the Outline variant (1px border),
        // like the S3 demo's composition items, so the expected heights
        // below include the 2px border.
        //
        // Nova Default/Sm, title + description:
        //   19.25 (title, 14px x 1.375) + 4 (gap-1) + 21 (desc, 14px x 1.5)
        //   + 20 (pad_y 10 x 2) + 2 (border) = 66.25.
        // Nova Xs, title + description:
        //   19.25 + 0 (gap 0 at xs nova) + 18 (desc text-xs 12px x 1.5)
        //   + 16 (pad_y 8 x 2) + 2 = 55.25.
        // Nova Default, title only: 19.25 + 20 + 2 = 41.25.
        // Nova Default, title only + icon media: the 32px media dominates
        //   the row, so 32 + 20 + 2 = 54.
        // Vega Default, title + description:
        //   19.25 + 4 + 21 + 32 (pad_y 16 x 2) + 2 = 78.25.
        // Vega Default, title only: 19.25 + 32 + 2 = 53.25.
        let check = |name: &str, got: f32, want: f32| {
            assert!(
                (got - want).abs() < 0.5,
                "{name}: frame height must be the spec model {want}, got {got}"
            );
        };

        // Nova, title + description, one case per media variant.
        let h = render_twice(400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("h_icon_desc")
                .show(ui, |ui| {
                    ItemMedia::icon(Icon::User).show(ui);
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                        ItemDescription::new("Engineer").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("nova icon+desc", h, 66.25);

        let h = render_twice(400.0, 600.0, |ui| {
            let tex = ui.ctx().load_texture(
                "item_height_texture",
                egui::ColorImage::example(),
                egui::TextureOptions::default(),
            );
            let source = egui::ImageSource::Texture(egui::load::SizedTexture::new(
                tex.id(),
                Vec2::splat(ITEM_MEDIA_IMAGE_SIZE),
            ));
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("h_image_desc")
                .show(ui, |ui| {
                    ItemMedia::image(source).show(ui);
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                        ItemDescription::new("Engineer").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("nova image+desc", h, 66.25);

        let h = render_twice(400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("h_nomedia_desc")
                .show(ui, |ui| {
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                        ItemDescription::new("Engineer").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("nova no-media+desc", h, 66.25);

        // Nova Sm takes the same geometry as Default.
        let h = render_twice(400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .size(ItemSize::Sm)
                .id_salt("h_sm_desc")
                .show(ui, |ui| {
                    ItemMedia::icon(Icon::User).show(ui);
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                        ItemDescription::new("Engineer").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("nova sm icon+desc", h, 66.25);

        // Nova Xs: zero content gap, 12px description.
        let h = render_twice(400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .size(ItemSize::Xs)
                .id_salt("h_xs_desc")
                .show(ui, |ui| {
                    ItemMedia::icon(Icon::User).show(ui);
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                        ItemDescription::new("Engineer").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("nova xs icon+desc", h, 55.25);

        // Title-only cases.
        let h = render_twice(400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("h_title_only")
                .show(ui, |ui| {
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("nova title-only", h, 41.25);

        let h = render_twice(400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("h_title_only_icon")
                .show(ui, |ui| {
                    ItemMedia::icon(Icon::User).show(ui);
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("nova title-only+icon", h, 54.0);

        // Vega (base recipe): larger padding, same text block.
        let vega = Preset::default().with_style(Style::Vega);
        let h = render_twice_with_preset(vega.clone(), 400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("h_vega_desc")
                .show(ui, |ui| {
                    ItemMedia::icon(Icon::User).show(ui);
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                        ItemDescription::new("Engineer").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("vega icon+desc", h, 78.25);

        let h = render_twice_with_preset(vega, 400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("h_vega_title_only")
                .show(ui, |ui| {
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                    });
                })
                .response
                .rect
                .height()
        })
        .1;
        check("vega title-only", h, 53.25);
    }

    #[test]
    fn header_footer_rows_match_spec_model() {
        // ITM-04: the §7 Header/Footer rows must allocate at content
        // height, not the 36px `ui.horizontal` floor. The stack is the §7
        // demo stack (Outline variant, Default size, 600px root so the
        // one-line description does not wrap): header = title + Admin
        // badge, content = one-line description, footer = label + Sm
        // Manage button.
        //
        // Nova model: 20 (py 10 x 2) + 20 (header: max(19 title, 20
        // badge)) + 10 (gap-2.5) + 21 (desc, 14px x 1.5) + 10 (gap-2.5)
        // + 28 (footer: Sm button outer rect) + 2 (border) = 111.
        // Vega model: 32 (py 16 x 2) + 22 (header: max(19 title, 22
        // badge)) + 16 (gap-4) + 21 + 16 + 32 (footer: Sm button outer
        // rect) + 2 = 141.
        //
        // The badge (20 nova / 22 vega) and Sm button (28 nova / 32
        // vega) are taller in vega than nova — style-dependent
        // component geometry, not item geometry — so the vega frame
        // is 141, not 135. The defect file's 109/133 model used the
        // Sm button's 26px painted face (and nova heights for vega);
        // the row height is the button's outer rect (its response
        // rect), so 111/141 is correct.
        //
        // One-frame settle: the rows reserve the actions' height from
        // last frame (ITM-04), so frame 1 sizes to the direct children
        // only — header 19 (title), footer 17 (label) — for 99 nova /
        // 123 vega; frame 2 settles at the model. Same class as the
        // `actions_w` reservation (ITM-02) and alert D7.
        let check = |name: &str, got: f32, want: f32| {
            assert!(
                (got - want).abs() < 0.5,
                "{name}: frame height must be the spec model {want}, got {got}"
            );
        };

        let (first, second) = render_twice(600.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("hf_nova")
                .show(ui, |ui| {
                    ItemHeader::new().show(ui, |ui| {
                        ItemTitle::new("Team settings").show(ui);
                        ItemActions::new().show(ui, |ui| {
                            crate::components::badge::Badge::new("Admin").show(ui);
                        });
                    });
                    ItemContent::new().show(ui, |ui| {
                        ItemDescription::new("Invite members and manage roles for this workspace.")
                            .show(ui);
                    });
                    ItemFooter::new().show(ui, |ui| {
                        ui.label("12 members");
                        ItemActions::new().show(ui, |ui| {
                            crate::components::button::Button::new("Manage")
                                .size(crate::components::button::ButtonSize::Sm)
                                .show(ui);
                        });
                    });
                })
                .response
                .rect
                .height()
        });
        check("nova header/footer stack frame 1 (settle)", first, 99.0);
        check("nova header/footer stack", second, 111.0);

        let vega = Preset::default().with_style(Style::Vega);
        let (first, second) = render_twice_with_preset(vega, 600.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("hf_vega")
                .show(ui, |ui| {
                    ItemHeader::new().show(ui, |ui| {
                        ItemTitle::new("Team settings").show(ui);
                        ItemActions::new().show(ui, |ui| {
                            crate::components::badge::Badge::new("Admin").show(ui);
                        });
                    });
                    ItemContent::new().show(ui, |ui| {
                        ItemDescription::new("Invite members and manage roles for this workspace.")
                            .show(ui);
                    });
                    ItemFooter::new().show(ui, |ui| {
                        ui.label("12 members");
                        ItemActions::new().show(ui, |ui| {
                            crate::components::button::Button::new("Manage")
                                .size(crate::components::button::ButtonSize::Sm)
                                .show(ui);
                        });
                    });
                })
                .response
                .rect
                .height()
        });
        check("vega header/footer stack frame 1 (settle)", first, 123.0);
        check("vega header/footer stack", second, 141.0);
    }

    #[test]
    fn items_stretch_to_available_width() {
        // ITM-02: items are block-level (`ItemGroup` is `flex flex-col`,
        // `align-items: stretch`), so every item stretches to the
        // available width instead of sizing to its content. Pre-fix the
        // frames measured 24–44% of the column (§2 Alpha/Beta ~50px in a
        // 1019px column). The response rect is the frame's outer rect
        // (border to border), so in a 400px root it must be 400 wide.
        let check_w = |name: &str, got: f32, want: f32| {
            assert!(
                (got - want).abs() < 0.5,
                "{name}: frame width must stretch to {want}, got {got}"
            );
        };

        // Standalone item, icon + description.
        let (w, h) = render_twice(400.0, 600.0, |ui| {
            let r = Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("w_icon_desc")
                .show(ui, |ui| {
                    ItemMedia::icon(Icon::User).show(ui);
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Ada Lovelace").show(ui);
                        ItemDescription::new("Engineer").show(ui);
                    });
                })
                .response
                .rect;
            (r.width(), r.height())
        })
        .1;
        check_w("standalone icon+desc", w, 400.0);
        // The ITM-01 height model still holds at full width (the short
        // title/description do not re-wrap).
        assert!(
            (h - 66.25).abs() < 0.5,
            "standalone icon+desc height must stay the spec model 66.25, got {h}"
        );

        // Title-only tiny item (§2 Alpha/Beta shape): was ~50px wide.
        let w = render_twice(400.0, 600.0, |ui| {
            Item::new()
                .variant(ItemVariant::Outline)
                .id_salt("w_title_only")
                .show(ui, |ui| {
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Alpha").show(ui);
                    });
                })
                .response
                .rect
                .width()
        })
        .1;
        check_w("title-only", w, 400.0);

        // Item inside a group: the group stretches too, so the item gets
        // the full root width.
        let w = render_twice(400.0, 600.0, |ui| {
            ItemGroup::new()
                .id_salt("w_group")
                .show(ui, |ui| {
                    Item::new()
                        .variant(ItemVariant::Outline)
                        .id_salt("w_grouped")
                        .show(ui, |ui| {
                            ItemContent::new().show(ui, |ui| {
                                ItemTitle::new("Ada Lovelace").show(ui);
                            });
                        })
                        .response
                        .rect
                        .width()
                })
                .inner
        })
        .1;
        check_w("grouped item", w, 400.0);

        // Narrow scope (§4 shape): the item stretches to the scope's
        // width, not the page width.
        let w = render_twice(400.0, 600.0, |ui| {
            let mut w = 0.0;
            ui.allocate_ui(Vec2::new(360.0, 600.0), |ui| {
                w = Item::new()
                    .variant(ItemVariant::Outline)
                    .id_salt("w_scoped")
                    .show(ui, |ui| {
                        ItemContent::new().show(ui, |ui| {
                            ItemTitle::new("Ada Lovelace").show(ui);
                            ItemDescription::new("Engineer").show(ui);
                        });
                    })
                    .response
                    .rect
                    .width();
            });
            w
        })
        .1;
        check_w("scoped item", w, 360.0);
    }

    #[test]
    fn actions_push_to_right_edge() {
        // ITM-05 (spec §2.7 `flex-1`): the content column grows to fill
        // the row, pushing the actions to the item's right edge. Pre-fix
        // the content sized to max-content (title `w-fit` + description
        // galley), so the actions sat immediately after the content —
        // the §8 Launch button at x 548–610 in a frame spanning to
        // 1191. Root cause (egui 0.37 `ui.rs:2337-2338`,
        // `layout.rs:751-754`): the wrapping row advances by each
        // child's `min_rect`, and the content Ui's min_rect is its
        // max-content width — the width `ItemContent` allocates never
        // reaches the row.
        let check = |name: &str, button_right: f32, item_right: f32, inset: f32| {
            let want = item_right - inset;
            assert!(
                (button_right - want).abs() < 0.5,
                "{name}: actions must end at the item's right edge minus the content inset ({want}), got {button_right}"
            );
        };

        let build = |ui: &mut Ui| {
            let mut button = Rect::NOTHING;
            let mut content_rect = Rect::NOTHING;
            let item = Item::new()
                .interactive(true)
                .variant(ItemVariant::Outline)
                .id_salt("actions_right_edge")
                .show(ui, |ui| {
                    ItemMedia::icon(Icon::ChevronsRight).show(ui);
                    content_rect = ItemContent::new()
                        .show(ui, |ui| {
                            ItemTitle::new("Launch checklist").show_with(ui, |ui| {
                                crate::components::badge::Badge::new("Ready").show(ui);
                            });
                            ItemDescription::new("The item hovers like the source's anchor rows.")
                                .show(ui);
                        })
                        .response
                        .rect;
                    ItemActions::new().show(ui, |ui| {
                        button = crate::components::button::Button::new("Launch")
                            .size(crate::components::button::ButtonSize::Sm)
                            .show(ui)
                            .rect;
                    });
                })
                .response
                .rect;
            (item, button, content_rect)
        };

        // Gallery column width (frame 173–1191): wide enough that the
        // description fits on one line, so max-content is much narrower
        // than the row — pre-fix the button sat ~545px from the right
        // edge. Frame 2 (settled `actions_w` reservation).
        //
        // The content inset is pad + the Outline border: egui's
        // `Frame::total_margin` is inner_margin + stroke.width +
        // outer_margin (`frame.rs:362-366`), and the Outline stroke
        // is 1.0 (`resolve_item_colors`), so the content area ends at
        // item_right − pad − 1 (12 + 1 nova, 16 + 1 vega) — not at
        // item_right − pad.
        let (item_rect, button_rect, _content_rect) = render_twice(1018.0, 600.0, build).1;
        check("nova", button_rect.right(), item_rect.right(), 13.0);

        let vega = Preset::default().with_style(Style::Vega);
        let (item_rect, button_rect, _content_rect) =
            render_twice_with_preset(vega, 1018.0, 600.0, build).1;
        check("vega", button_rect.right(), item_rect.right(), 17.0);
    }

    #[test]
    fn description_link_runs_underlined_with_offset() {
        // N10 (pure): a link segment produces one underline run, 4px below
        // the link glyphs' baseline, with a hit rect 2px wider each side.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("item_link_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
            );
            let segments = vec![
                DescriptionSegment::Text("See ".into()),
                DescriptionSegment::Link("docs".into()),
                DescriptionSegment::Text(" for details.".into()),
            ];
            let job = description_job(&segments, 14.0, Color32::BLACK, Color32::BLACK);
            let galley = ui.painter().layout_job(job);
            let runs = link_runs(&galley, &segments);
            assert_eq!(runs.len(), 1, "one underline run for the link segment");
            let run = runs[0];
            assert!(run.from.x < run.to.x, "run has width");
            assert_eq!(run.from.y, run.to.y, "run is horizontal");
            let baseline = galley.rows[0]
                .row
                .glyphs
                .iter()
                .find(|g| g.chr == 'd')
                .map(|g| galley.rows[0].pos.y + g.pos.y)
                .expect("link glyph present");
            assert!(
                (run.from.y - (baseline + 4.0)).abs() < 0.01,
                "underline offset is 4px below the baseline"
            );
            assert!(
                (run.hit.width() - ((run.to.x - run.from.x) + 4.0)).abs() < 0.01,
                "hit rect is 2px wider each side"
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn click_moves_keyboard_focus_to_the_item() {
        // ITM-03: native `<a>` behaviour (spec §7 + D4) — a clicked
        // anchor row holds keyboard focus afterwards, so Space/Enter
        // then activate it. Pre-fix the click toggled the item but left
        // focus nowhere, so Space/Return after a click did nothing.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("item_click_focus_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            )
        };
        let click_event = |pos: Pos2, pressed: bool| egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let build = |ui: &mut Ui| {
            Item::new()
                .interactive(true)
                .id_salt("click_focus_item")
                .show(ui, |ui| {
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Launch checklist").show(ui);
                    });
                })
                .response
        };

        // Frame 1: render, record rect + id; nothing focused yet.
        let mut rect = Rect::NOTHING;
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let r = build(ui);
            rect = r.rect;
            id = r.id;
        });
        out.textures_delta.clear();
        assert!(!ctx.memory(|m| m.has_focus(id)));

        // Frames 2-3: press + release on the item = a click.
        let center = rect.center();
        let mut out = ctx.run_ui(click_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _r = build(ui);
        });
        out.textures_delta.clear();
        let mut clicked = false;
        let mut out = ctx.run_ui(click_event(center, false), |ctx| {
            let ui = &mut make_ui(ctx);
            clicked = build(ui).clicked();
        });
        out.textures_delta.clear();
        assert!(clicked, "press + release must register a click");
        assert!(
            ctx.memory(|m| m.has_focus(id)),
            "click must move keyboard focus to the item"
        );
    }

    #[test]
    fn click_on_nested_button_focuses_the_item() {
        // ITM-03, nested-button case: egui's click is exclusive and the
        // hit test takes the last-registered widget on a tie, so the
        // item (whose `interact` runs after its children) wins the
        // click over a nested button — the button never receives the
        // click, so its own `request_focus()` never runs. The item
        // taking focus here therefore steals nothing from the button:
        // before this fix a click on Launch left focus nowhere (S6: no
        // ring, Tab wrapped to the theme-toggle); now it focuses the
        // row, and Space/Enter toggle it.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("item_nested_button_focus_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            )
        };
        let click_event = |pos: Pos2, pressed: bool| egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        // Renders the §8 shape — interactive item with a nested Launch
        // button — and returns (item response, button response).
        let build = |ui: &mut Ui| {
            let mut button = None;
            let item = Item::new()
                .interactive(true)
                .id_salt("nested_button_item")
                .show(ui, |ui| {
                    ItemContent::new().show(ui, |ui| {
                        ItemTitle::new("Launch checklist").show(ui);
                    });
                    ItemActions::new().show(ui, |ui| {
                        button = Some(
                            crate::components::button::Button::new("Launch")
                                .size(crate::components::button::ButtonSize::Sm)
                                .show(ui),
                        );
                    });
                })
                .response;
            (item, button.expect("nested button rendered"))
        };

        // Frame 1: render; nothing focused yet. Frame 2: re-render and
        // record the SETTLED rects — the button's position depends on
        // the `actions_w` reservation (ITM-02/ITM-05), which is 0 on
        // frame 1, so on frame 1 the button sits past the item's right
        // edge and only lands at its settled position from frame 2.
        // Clicking at the frame-1 position would press outside the
        // settled item. Same one-frame settle class as ITM-04/D7.
        let mut item_rect = Rect::NOTHING;
        let mut item_id = egui::Id::NULL;
        let mut button_rect = Rect::NOTHING;
        let mut button_id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let _r = build(ui);
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let (item, button) = build(ui);
            item_rect = item.rect;
            item_id = item.id;
            button_rect = button.rect;
            button_id = button.id;
        });
        out.textures_delta.clear();
        assert!(!ctx.memory(|m| m.has_focus(item_id)));
        assert!(!ctx.memory(|m| m.has_focus(button_id)));
        assert!(
            button_rect.center() != item_rect.center(),
            "the nested button must sit away from the item centre"
        );

        // Frames 3-4: press + release on the nested button = a click.
        let center = button_rect.center();
        let mut out = ctx.run_ui(click_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _r = build(ui);
        });
        out.textures_delta.clear();
        let mut item_clicked = false;
        let mut button_clicked = false;
        let mut out = ctx.run_ui(click_event(center, false), |ctx| {
            let ui = &mut make_ui(ctx);
            let (item, button) = build(ui);
            item_clicked = item.clicked();
            button_clicked = button.clicked();
        });
        out.textures_delta.clear();
        assert!(
            item_clicked,
            "the item must win the click over the nested button"
        );
        assert!(
            !button_clicked,
            "the nested button must not receive the click (the item's later interact wins the hit test)"
        );
        assert!(
            ctx.memory(|m| m.has_focus(item_id)),
            "clicking the nested button must focus the item row"
        );
        assert!(
            !ctx.memory(|m| m.has_focus(button_id)),
            "the nested button must not hold focus it never received"
        );
    }
}
