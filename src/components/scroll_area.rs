//! shadcn `scroll-area.tsx` — a themed scrolling container.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/scroll-area.tsx`: a
//! `ScrollArea` root with a viewport and a `ScrollBar` whose thumb is
//! `rounded-full bg-border` and whose track is `w-2.5` (10px) with `p-px`
//! padding.
//!
//! # Verified final state (S10, all 4 configs)
//!
//! Every statement below is verified in
//! `batch-c/reports/scroll-area-s10-report.md` (S10) or decided in
//! `batch-c/reports/s8-decisions.md` §3.7 (S8 gate); S10 supersedes S5/S6
//! where they differ.
//!
//! ## Geometry (S10 §1, identical in all 4 configs)
//!
//! Five vertical thumb runs, same y-positions in every config: §1
//! (251–274, 24px), §3 (619–636, 18px), §4 (831–905, 75px), §5
//! (1126–1141, 16px by the S6 method / 18px painted span), §6 (1358–1377,
//! 20px). §2 horizontal thumb: 487px (nova) / 416px (vega), y537–544.
//! Viewport heights are exact (ring height − 9): §3 140px, §5 160px,
//! §6 140px, §7 80px; focus rings measure 1028×(h+1).
//!
//! ## The three S9 changes (all PASS at S10)
//!
//! * **SCA-01** (`63d1158`, P1): the axes passed to egui's
//!   `ScrollArea::new` were swapped (`[vertical, horizontal]` instead of
//!   `[horizontal, vertical]`); §1 now clips to its 160px viewport (only
//!   Items 01–07 visible) and §1–§7 all sit above the fold.
//! * **SCA-02** (`4ed09a5`, P2): `.height(h)` now pins the viewport at
//!   exactly `h` (`max_height(h).min_scrolled_height(h)`); §6 renders
//!   140px and §7 80px (both sat on egui's 64px `min_scrolled_size`
//!   floor pre-fix, measured headless at exactly 64.0px).
//! * **Thumb minimum 18px** (`8637868`, S8 §3.7 OPEN-3): the single
//!   constant `THUMB_MIN_LENGTH` went 12.0 → 18.0, feeding both egui's
//!   `handle_min_length` and the self-painted thumb clamp. S10 measured
//!   §3 18 (TRUE 20 light / 19 dark), §5 16 (TRUE 18 light / 17 dark —
//!   the painted span is 18 rows in both), §6 20 (TRUE 22): the 18px
//!   minimum holds on the painted span in all 4 configs.
//!
//! ## Interaction (S10 §2, all 4 configs)
//!
//! * Track-click: the thumb **jumps** to the click point (|centre − click|
//!   = 0.5px in all 4 configs) and the viewport defocuses (egui native
//!   behaviour, kept per S8 §3.7 — Radix's jump-then-drag is NOT
//!   implemented).
//! * Drags and wheel are exact and restore exactly: §1 down60
//!   (57,80)→(117,140) + back; §2 right100 + back; §6 down40 + back;
//!   wheel down3 +39/+38 + back.
//! * Arrow keys move **focus**, never content (S8 §3.7 OPEN-2 — Radix has
//!   no keyboard table; the port's viewport is focusable and egui's
//!   directional focus navigation takes the arrows). The thumb never
//!   moves on any key.
//! * Tab walk: 39–40 stops; landmarks 107 §3 / 109 §4 / 110 button /
//!   111 §5 / 112 §6 / 113 §7 all match, with reversal.
//!
//! ## States
//!
//! * **No hover state** (spec §4.2): the thumb is `bg-border` at rest,
//!   on hover, and while dragging — the audit's nova `hover:bg-accent`
//!   claim was wrong (zero hits in `style-nova.css`). S10 confirmed
//!   zero-change over 20 transitions per thumb per config.
//! * **Always-visible bar** (OPEN-1): a deliberate deviation from Radix's
//!   `type='hover'` default, RATIFIED at S8 §3.7 — a 600ms hide delay
//!   fits egui's immediate-mode model poorly.
//! * **Bar hides when content fits** (OPEN-4): the bar does NOT paint in
//!   §7 at rest or focused (zero bar pixels, all 4 configs, S10 §6) —
//!   `scrollbar_thumb_rect` returns `None` when content fits, matching
//!   Radix. The S8 record's contrary sentence was factually wrong and is
//!   corrected at S12.
//! * **Track**: no visible track (`extreme_bg_color = TRANSPARENT`, the
//!   source's `border-l-transparent`); `bg_fill = border` RATIFIED at
//!   S8 §3.7 as the correct port (spec §4.1/§3.4).
//!
//! ## Deviations and OPEN items (current dispositions)
//!
//! D1 (no ARIA tree), D2 (`:focus-visible` modality — ring shows on any
//! egui focus), D3 (overlay → layout width), D4 (thumb 10px vs source
//! 8px), D5 (no standalone `ScrollBar` export) — all PROPOSED, no
//! approval line filled; they go to the Expert Reviewer in the wave
//! pack. OPEN-1 RATIFIED (deliberate deviation); OPEN-2 DO NOT wire
//! arrow-key scrolling; OPEN-3 ADOPTED (18px, S9 `8637868`); OPEN-4
//! settled at S10 (bar hides when content fits — actual behaviour, not
//! a deviation); OPEN-5 (radius inheritance mechanism) stays OPEN.
//!
//! ## Fixed post-wave
//!
//! **SCA-03** (P2, `batch-c/reports/defects/SCA-03.md`, fixed on branch
//! `sca-03-fix`): the §4 inner button is a Tab stop (108) but when
//! focused painted NO visible indicator and the viewport did NOT scroll
//! to reveal it (whole-window diff vs rest exactly 0px) — egui 0.37 has
//! no auto-scroll-to-focused-widget. Fix shape (b), decided by the
//! Expert Reviewer (spec §11 rule): the component tracks the focused
//! descendant each frame and, on a focus *change* to a descendant
//! outside the viewport, scrolls the minimum amount to bring it into
//! view via `Ui::scroll_to_rect` (browser behaviour). The descendant
//! test is geometric — the focused widget's rect (from
//! `Context::read_response`) must lie inside the area's content span
//! and its clip-intersected `interact_rect` inside the viewport — so a
//! non-descendant whose screen rect merely overlaps the content span
//! (the "Focus viewport" button below §4) never scrolls the area. The
//! offset is never re-asserted while focus is unchanged, so user
//! scrolling is never fought; rest renders are unchanged.
//!
//! ## Budgets and standing caveats (S10 §5)
//!
//! Idle CPU 0.0%, animating 107–131% (<150%), RSS −2.42%..+0.9% with no
//! monotonic growth (the S7 RSS watch, +4.93% nova/light, is RESOLVED).
//! Determinism 0, flicker 0, round-trips byte-identical. Render floor
//! NOT VERIFIED (2.2–3.87 fps, pipeline-capped) and timing legs
//! UNRESOLVED in every video leg — the wave-wide pattern, not a
//! component defect.
//!
//! ## Tokens
//!
//! Thumb `bg-border` (light `oklch(0.922 0 0)`, dark
//! `oklch(1 0 0 / 10%)`); focus ring `ring/50` (light
//! `oklch(0.708 0 0)`, dark `oklch(0.556 0 0)`). No token disagreement
//! exists for this component (`border`/`ring` agree across
//! `globals.css` and `themes.ts`, spec §5).
//!
//! The scrollbar is egui's *floating* bar, so it overlays the viewport and
//! takes no layout width (N14a — the viewport is `size-full`), and its
//! outer edge is flush with the Root edge (`bar_outer_margin = 0`, N14b).
//! egui's handle spans the bar's full cross range and cannot express the
//! source's `p-px` thumb inset, so the handle is made invisible (all handle
//! opacities 0) and the thumb is painted by this module instead: 8px wide,
//! 1px inside the 10px bar on every side, radius 4 (`rounded-full`),
//! `bg-border` in every state — the source defines no thumb hover state.
//! Drag and track-click paging still run on egui's bar, whose geometry the
//! painted thumb mirrors exactly (see [`scrollbar_thumb_rect`]).
//!
//! The viewport is one Tab stop; on focus it paints the source's
//! `focus-visible` ring (3px `ring/50`) plus the 1px solid outline in
//! `ring/50`.
//!
//! The scrollbar look is applied through a *scoped* egui style
//! (`ScrollStyle::floating` + themed handle/track colors), so the global
//! style is never permanently mutated — nested `egui::ScrollArea`s created
//! inside the closure inherit it. The track is invisible
//! (`extreme_bg_color = TRANSPARENT`, the source's `border-l-transparent`);
//! the scoped widget `bg_fill`s stay `bg-border` in all three states (the
//! audit-verified P0 fix). They no longer paint the handle itself (it is
//! opacity-0) and are retained so content widgets inside the area keep
//! their current paint — removing them would change the paint of every
//! control nested in a scroll area, which is out of scope for this line.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ScrollArea::new().height(200.0).show(ui, |ui| {
//!     for i in 0..50 {
//!         ui.label(format!("Row {i}"));
//!     }
//! });
//! ```

use crate::theme::ShadcnTheme;
use crate::utils::focus::{is_focused, paint_focus_ring, paint_ring};
use egui::{CornerRadius, InnerResponse, Rect, Ui};

/// The scrollbar track width (`w-2.5`).
pub const SCROLLBAR_WIDTH: f32 = 10.0;

/// The thumb width: the 10px bar minus `p-px` (1px) on each side.
pub const SCROLLBAR_THUMB_WIDTH: f32 = 8.0;

/// The thumb inset inside the bar (`p-px`, 1px on every side).
const THUMB_INSET: f32 = 1.0;

/// The thumb corner radius (`rounded-full` on the 8px thumb).
const THUMB_RADIUS: u8 = 4;

/// Minimum thumb length (Radix `@radix-ui/react-scroll-area@1.2.10`:
/// `Math.max(thumbSize, 18)` — "minimum of 18 matches macOS minimum").
/// OPEN-3 settled at the S8 gate (see `s8-decisions.md` §3.7): the 18px
/// value is adopted from the Radix source; the previous 12.0 was egui's
/// default with no source basis.
const THUMB_MIN_LENGTH: f32 = 18.0;

/// A shadcn scroll area. See the [module docs](self) for the S10-verified
/// final state (geometry, interaction, deviations and OPEN dispositions).
#[derive(Clone, Debug)]
pub struct ScrollArea {
    vertical: bool,
    horizontal: bool,
    height: Option<f32>,
    width: Option<f32>,
    id_salt: Option<String>,
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollArea {
    /// Creates a vertical scroll area (the source default orientation).
    pub fn new() -> Self {
        Self {
            vertical: true,
            horizontal: false,
            height: None,
            width: None,
            id_salt: None,
        }
    }

    /// Enables/disables the vertical scrollbar.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Enables the horizontal scrollbar (`orientation="horizontal"`).
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Fixes the viewport height (default: available height).
    ///
    /// The height is enforced, not merely capped: egui's `max_height`
    /// alone collapses to the available height, which is ~0 for a
    /// ScrollArea laid out far below the fold of a scrolling parent
    /// (SCA-02), so `show` also sets `min_scrolled_height` to the same
    /// value. With `auto_shrink([false, false])` the viewport then
    /// renders at exactly this height even when the content is shorter.
    /// S10-verified: §6 renders 140px and §7 80px in all 4 configs
    /// (both sat on egui's 64px floor pre-fix).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Fixes the viewport width (default: available width).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a stable id salt so scroll position persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Builds the scoped scrollbar style.
    ///
    /// Floating bar (`N14a` — overlays the viewport, no layout width):
    /// `floating_allocated_width = 0`, `bar_width = 10.0` (`w-2.5`),
    /// `bar_outer_margin = 0` (`N14b` — flush with the Root edge),
    /// `floating_width = 10.0` (never shrinks to the thin width),
    /// `handle_min_length = 18.0` (OPEN-3, Radix `Math.max(thumbSize, 18)`,
    /// settled at the S8 gate).
    ///
    /// egui's own handle is invisible in every state (all handle opacities
    /// 0): it spans the bar's full cross range and cannot express the
    /// source's `p-px` thumb inset, so S1 paints the thumb itself (8px,
    /// 1px inset, radius 4 — see [`scrollbar_thumb_rect`]). Interaction
    /// (drag, track-click paging) still runs on egui's bar.
    pub(crate) fn themed_scroll_style() -> egui::style::ScrollStyle {
        let mut s = egui::style::ScrollStyle::floating();
        s.bar_width = SCROLLBAR_WIDTH; // w-2.5
        s.bar_inner_margin = THUMB_INSET; // p-px (documents the inset; unused by floating cross placement)
        s.bar_outer_margin = 0.0; // N14b: flush with the Root edge
        s.handle_min_length = THUMB_MIN_LENGTH; // OPEN-3: Radix 18px minimum (S8)
        s.floating_width = SCROLLBAR_WIDTH; // never shrink to the thin width
        s.floating_allocated_width = 0.0; // N14a: no layout width
        s.foreground_color = false; // handle uses the widget bg_fill (= border), as before
                                    // egui's handle is invisible; the thumb is painted by this module.
        s.dormant_handle_opacity = 0.0;
        s.active_handle_opacity = 0.0;
        s.interact_handle_opacity = 0.0;
        // The track is invisible (extreme_bg_color = TRANSPARENT, below);
        // the background opacities are 0 for the same reason.
        s.dormant_background_opacity = 0.0;
        s.active_background_opacity = 0.0;
        s.interact_background_opacity = 0.0;
        s
    }

    /// Applies the shadcn scrollbar colors to the scoped visuals.
    ///
    /// The track becomes invisible (the source only paints the thumb) and
    /// the handle uses the `bg-border` token in every widget state. The
    /// handle itself is painted invisible by the opacities in
    /// [`Self::themed_scroll_style`] (egui's handle cannot express the
    /// `p-px` inset); these `bg_fill`s are retained so content widgets
    /// inside the area keep their current paint — see the module docs.
    pub(crate) fn themed_scroll_visuals(visuals: &mut egui::Visuals, border: egui::Color32) {
        visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_fill = border;
        visuals.widgets.hovered.bg_fill = border;
        visuals.widgets.active.bg_fill = border;
    }

    /// SCA-03 (spec §11 rule): scroll a focused descendant into view.
    ///
    /// Called at the end of the content closure, after the content has
    /// run so the focused widget's rect is registered this pass.
    /// `Ui::scroll_to_rect` sets a pass-state scroll target that this
    /// area's own `show` post-processing consumes (it is the innermost
    /// scroll area finishing after the call), scrolling the minimum
    /// amount to reveal the rect (`align: None`) — the target rect is
    /// expanded by the 4px focus-ring margin so the ring itself clears
    /// the viewport edge instead of being clipped by it.
    ///
    /// The focused widget's rect comes from
    /// [`egui::Context::read_response`] (egui 0.37 has no public
    /// id-to-rect lookup besides it); its `interact_rect` is the
    /// clip-intersected rect. A widget is treated as a descendant only
    /// when, on every enabled axis, its rect lies inside this area's
    /// content span (`ui.min_rect()`, the allocated-content bounding
    /// box) AND its interact rect lies inside the viewport — or is
    /// empty, when the widget is fully clipped out. The second half is
    /// what rejects non-descendants whose screen rect merely overlaps
    /// the content span: e.g. the "Focus viewport" button just below
    /// §4 keeps its full interact rect, which sticks out of the
    /// viewport, so it is not a descendant and never scrolls this area.
    ///
    /// Acts only on a focus *change* (the last focused id is kept in
    /// temp data per instance, keyed by the content Ui id), so the
    /// offset is never re-asserted while focus is unchanged and user
    /// scrolling is never fought. Fully-visible descendants, the
    /// viewport itself, and off-screen areas are no-ops.
    fn auto_scroll_focused_into_view(&self, ui: &mut Ui) {
        let focused = ui.memory(|m| m.focused());
        let key = ui.scope_id().with("sca03_last_focused");
        let last: Option<egui::Id> = ui
            .ctx()
            .data_mut(|d| d.get_temp::<Option<egui::Id>>(key))
            .flatten();
        // Record the current focus every frame so a change is seen once.
        ui.ctx().data_mut(|d| d.insert_temp(key, focused));
        let Some(focused) = focused else { return };
        if last == Some(focused) {
            return; // only on focus change, never continuously
        }
        let Some(response) = ui.ctx().read_response(focused) else {
            return;
        };
        let rect = response.rect;
        let interact = response.interact_rect;
        // The content Ui's clip rect is the viewport (egui's ScrollArea
        // sets it to the inner rect intersected with the parent clip).
        let viewport = ui.clip_rect();
        if !viewport.is_positive() {
            return; // the whole area is off-screen; scrolling it reveals nothing
        }
        if viewport.contains_rect(rect) {
            return; // already fully visible
        }
        if rect.contains_rect(viewport) {
            return; // the viewport itself (or a widget covering it)
        }
        let content = ui.min_rect();
        for d in 0..2 {
            let enabled = if d == 0 {
                self.horizontal
            } else {
                self.vertical
            };
            if !enabled {
                continue;
            }
            // The widget must lie inside this area's content span.
            if !(content.min[d] <= rect.min[d] && rect.max[d] <= content.max[d]) {
                return;
            }
            // ... and be clipped consistently with this viewport: a
            // non-empty interact rect must lie inside the viewport.
            // An empty interact rect (fully clipped out) passes.
            if interact.min[d] < interact.max[d]
                && !(viewport.min[d] <= interact.min[d] && interact.max[d] <= viewport.max[d])
            {
                return;
            }
        }
        // Scroll the widget into view — expanded by the focus-ring
        // margin so the ring itself is visible, not just the widget.
        // `scroll_to_rect` with `align: None` scrolls the minimum
        // amount to reveal the target; scrolling the bare widget rect
        // parks its edge flush with the viewport edge, and the ring
        // (painted ~4px outside the widget, the same expand(4.0) as
        // the viewport's own focus ring) is clipped by the viewport's
        // clip rect — in the gallery the §4 button's ring lost its
        // whole bottom edge that way. The expansion applies only when
        // a scroll is already happening (the fully-visible early
        // return above uses the bare rect), so visible descendants
        // still never scroll.
        ui.scroll_to_rect(rect.expand(4.0), None);
    }

    /// Renders the scroll area around its contents.
    ///
    /// S10-verified interaction (all 4 configs): track-click jumps the
    /// thumb to the click point (|centre − click| = 0.5px) and defocuses
    /// the viewport; drags and wheel moves are exact and restore exactly;
    /// arrow keys move focus, never content (S8 §3.7 OPEN-2); the viewport
    /// is one Tab stop and on focus paints the 3px `ring/50` ring plus the
    /// 1px solid outline (settled ring 9528 / ring50 9188, S10 §5).
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let outer = ui.scope(|ui| {
            // Scoped style: only scroll areas created inside inherit it.
            ui.style_mut().spacing.scroll = Self::themed_scroll_style();
            // egui 0.37 paints the solid track with `extreme_bg_color` and
            // the handle with the widget `bg_fill`s; retarget them so the
            // track disappears and the thumb is `bg-border`, rounded-full.
            Self::themed_scroll_visuals(&mut ui.style_mut().visuals, p.border);

            let mut area =
                egui::ScrollArea::new([self.horizontal, self.vertical]).auto_shrink([false, false]);
            if let Some(h) = self.height {
                // SCA-02: `max_height` alone is only a cap — egui takes
                // `min(available_height, max_height)`, and the available
                // height is ~0 for an area laid out far below the fold of
                // a scrolling parent, so the viewport collapsed to egui's
                // 64px `min_scrolled_size` floor. Pin the floor to the
                // same value so the viewport renders at exactly `h`
                // regardless of the available height (the spec's `.height()`
                // is a fixed size, "cap or fix" per spec §3).
                area = area.max_height(h).min_scrolled_height(h);
            }
            if let Some(salt) = self.id_salt.as_deref() {
                area = area.id_salt(ui.scope_id().with(salt));
            }
            let inner = area.show(ui, |ui| {
                if let Some(w) = self.width {
                    ui.set_min_width(w);
                    ui.set_max_width(w);
                }
                let r = content(ui);
                // SCA-03 (spec §11): auto-scroll a focused descendant
                // into view. egui has no auto-scroll-to-focused-widget,
                // so the component tracks the focused descendant each
                // frame and, on a focus *change* to a descendant outside
                // the viewport, scrolls the minimum amount to reveal it
                // (browser behaviour for overflow containers). Runs after
                // the content so the focused widget's rect is registered
                // this pass; `scroll_to_rect` is consumed by this area's
                // own `show` post-processing (innermost finishing area).
                self.auto_scroll_focused_into_view(ui);
                r
            });

            // Paint the thumbs. egui's own handle is invisible in every
            // state (opacities 0 in `themed_scroll_style`); the source
            // thumb is 8px wide, 1px inside the 10px bar on every side,
            // radius 4, `bg-border` in all states (no hover state, §4.2).
            // Drag and track-click paging run on egui's bar, whose geometry
            // this mirrors exactly.
            let offset = inner.state.clamped_offset();
            let clip = ui.clip_rect();
            if self.vertical {
                if let Some(thumb) =
                    scrollbar_thumb_rect(true, inner.inner_rect, inner.content_size, offset, clip)
                {
                    ui.painter()
                        .rect_filled(thumb, CornerRadius::same(THUMB_RADIUS), p.border);
                }
            }
            if self.horizontal {
                if let Some(thumb) =
                    scrollbar_thumb_rect(false, inner.inner_rect, inner.content_size, offset, clip)
                {
                    ui.painter()
                        .rect_filled(thumb, CornerRadius::same(THUMB_RADIUS), p.border);
                }
            }

            // The viewport is one Tab stop (spec §7): focusable but not
            // interactive, so it never steals clicks from the content
            // (dialog.rs precedent). egui 0.37's ScrollAreaOutput has no
            // Response; synthesize one over the visible inner rect.
            // On focus, paint the source's `focus-visible` ring (3px
            // `ring/50`) plus the 1px solid outline in `ring/50`
            // (`globals.css:193`), just outside the ring. D2: egui cannot
            // distinguish keyboard from pointer focus, so the ring shows on
            // any focus.
            let response =
                ui.allocate_rect(inner.inner_rect, egui::Sense::focusable_noninteractive());
            if is_focused(&response) {
                paint_focus_ring(ui, &response, 0.0, true);
                let ring = crate::theme::color_alpha(p.ring, 0.5);
                paint_ring(ui.painter(), response.rect.expand(4.0), ring, 1.0, 0.0);
            }
            InnerResponse::new(inner.inner, response)
        });
        outer.inner
    }
}

/// Computes the thumb rect for one axis, mirroring egui 0.37's
/// `calculate_handle_rect` (`containers/scroll_area.rs`) with the source's
/// `p-px` inset applied: the thumb is 8px wide inside the 10px bar
/// (`rounded-full` → radius 4), 1px inside the bar on every side, and
/// floored at the 18px Radix minimum (`THUMB_MIN_LENGTH`, OPEN-3 settled
/// at S8 §3.7; S10 measured §3 18 / §5 18 painted / §6 20 in all 4
/// configs).
///
/// `viewport` is the scroll area's inner rect (== the Root rect for the
/// floating bar); `content_size` / `offset` are the along-axis content size
/// and clamped scroll offset from the `ScrollAreaOutput`; `clip_rect` is
/// the outer ui's clip rect — egui clamps the bar so it stays visible when
/// the area is scrolled out of view in a parent (spec §9 item 3).
///
/// Returns `None` when the content fits (no bar), matching egui's
/// `content_is_too_large` check — S10 confirmed §7 paints no bar at rest
/// or focused (OPEN-4 settled: the bar hides when content fits).
pub(crate) fn scrollbar_thumb_rect(
    vertical: bool,
    viewport: Rect,
    content_size: egui::Vec2,
    offset: egui::Vec2,
    clip_rect: Rect,
) -> Option<Rect> {
    let d = usize::from(vertical); // 1 = y (vertical), 0 = x (horizontal)
    let along = viewport.size()[d];
    let content = content_size[d];
    // egui: `limit_rect.size()[d].ceil() < content_size[d]`
    if along.ceil() >= content {
        return None;
    }
    let max_offset = content - along;
    // egui: `max_cross = min(outer_rect.max, clip_rect.max) - outer_margin`
    // (outer_margin is 0 — N14b flush; outer_rect == viewport for floating).
    let cross_max = viewport.max[1 - d].min(clip_rect.max[1 - d]);
    let bar_min_cross = cross_max - SCROLLBAR_WIDTH;
    // Along-axis bar range (egui's `scroll_bar_rect` defaults to inner_rect).
    let bar_min = viewport.min[d];
    let bar_max = viewport.max[d];
    let from_content = |c: f32| egui::emath::remap_clamp(c, 0.0..=content, bar_min..=bar_max);
    let handle_size =
        (from_content(offset[d] + along) - from_content(offset[d])).max(THUMB_MIN_LENGTH);
    let handle_start = egui::emath::remap_clamp(
        offset[d],
        0.0..=max_offset,
        bar_min..=(bar_max - handle_size),
    );
    let mut min = viewport.min;
    let mut max = viewport.max;
    min[1 - d] = bar_min_cross + THUMB_INSET;
    max[1 - d] = cross_max - THUMB_INSET;
    min[d] = handle_start;
    max[d] = handle_start + handle_size;
    Some(Rect::from_min_max(min, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect, Vec2};

    #[test]
    fn scrollbar_style_matches_source() {
        let s = ScrollArea::themed_scroll_style();
        assert!(s.floating); // N14a: overlay — no layout width
        assert_eq!(s.bar_width, 10.0); // w-2.5
        assert_eq!(s.bar_inner_margin, 1.0); // p-px
        assert_eq!(s.bar_outer_margin, 0.0); // N14b: flush with the Root edge
        assert_eq!(s.floating_width, 10.0); // never shrinks to the thin width
        assert_eq!(s.floating_allocated_width, 0.0); // N14a: no layout width
        assert_eq!(s.handle_min_length, 18.0); // OPEN-3: Radix 18px minimum (S8)
                                               // egui's own handle is invisible in every state; S1 paints the
                                               // thumb itself (see `scrollbar_thumb_rect`).
        assert_eq!(s.dormant_handle_opacity, 0.0);
        assert_eq!(s.active_handle_opacity, 0.0);
        assert_eq!(s.interact_handle_opacity, 0.0);
        assert_eq!(s.dormant_background_opacity, 0.0);
        assert_eq!(s.active_background_opacity, 0.0);
        assert_eq!(s.interact_background_opacity, 0.0);
    }

    #[test]
    fn scrollbar_thumb_rect_matches_source() {
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
        let clip = viewport; // unclipped
                             // Vertical, content twice the viewport, offset 0: thumb is the top
                             // half of the bar, 8px wide, 1px inside the bar on every side.
        let thumb = scrollbar_thumb_rect(true, viewport, Vec2::new(300.0, 600.0), Vec2::ZERO, clip)
            .expect("bar when content overflows");
        assert_eq!(thumb.width(), SCROLLBAR_THUMB_WIDTH);
        assert_eq!(thumb.min.x, 300.0 - SCROLLBAR_WIDTH + THUMB_INSET);
        assert_eq!(thumb.max.x, 300.0 - THUMB_INSET);
        assert_eq!(thumb.min.y, 0.0);
        assert_eq!(thumb.height(), 150.0); // half the content visible -> half the bar
                                           // Scrolled to the bottom: the thumb sits at the bar's bottom end.
        let thumb = scrollbar_thumb_rect(
            true,
            viewport,
            Vec2::new(300.0, 600.0),
            Vec2::new(0.0, 300.0),
            clip,
        )
        .expect("bar when content overflows");
        assert_eq!(thumb.max.y, 300.0);
        assert_eq!(thumb.height(), 150.0);
        // Content fits: no thumb.
        assert!(
            scrollbar_thumb_rect(true, viewport, Vec2::new(300.0, 300.0), Vec2::ZERO, clip)
                .is_none()
        );
        // Horizontal mirror: 8px tall, 1px inside the bar top/bottom.
        let thumb =
            scrollbar_thumb_rect(false, viewport, Vec2::new(600.0, 300.0), Vec2::ZERO, clip)
                .expect("bar when content overflows");
        assert_eq!(thumb.height(), SCROLLBAR_THUMB_WIDTH);
        assert_eq!(thumb.min.y, 300.0 - SCROLLBAR_WIDTH + THUMB_INSET);
        assert_eq!(thumb.max.y, 300.0 - THUMB_INSET);
        assert_eq!(thumb.width(), 150.0);
        // Clipped viewport: the bar moves inward so it stays visible
        // (egui's "move the scrollbar so it is visible" clamp).
        let clip = Rect::from_min_size(Pos2::ZERO, Vec2::new(280.0, 300.0));
        let thumb = scrollbar_thumb_rect(true, viewport, Vec2::new(300.0, 600.0), Vec2::ZERO, clip)
            .expect("bar when content overflows");
        assert_eq!(thumb.max.x, 280.0 - THUMB_INSET);
    }

    #[test]
    fn scrollbar_visuals_match_source() {
        // The source has no visible track (`border-l-transparent`) and a
        // `rounded-full bg-border` thumb; egui paints the track with
        // `extreme_bg_color` and the handle with the widget `bg_fill`s.
        let ctx = egui::Context::default();
        let preset = Preset::default();
        preset.build(Mode::Light).install(&ctx);
        let border = preset.build(Mode::Light).palette.border;
        let mut captured: Option<egui::Visuals> = None;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("scroll_area_visuals_test"),
                egui::UiBuilder::new(),
            );
            ScrollArea::new().show(ui, |ui| {
                captured = Some(ui.style().visuals.clone());
            });
        });
        out.textures_delta.clear();
        let v = captured.expect("scroll area content ran");
        assert_eq!(v.extreme_bg_color, egui::Color32::TRANSPARENT);
        assert_eq!(v.widgets.inactive.bg_fill, border);
        assert_eq!(v.widgets.hovered.bg_fill, border);
        assert_eq!(v.widgets.active.bg_fill, border);
        // The default (unscoped) style must not be mutated.
        let global = ctx.style_of(egui::Theme::Light).visuals.clone();
        assert_ne!(global.extreme_bg_color, egui::Color32::TRANSPARENT);
    }

    #[test]
    fn scrollbar_overlays_content() {
        // N14a: a vertical scroll area with overflowing content in a
        // 300px-wide Ui — the viewport keeps the full 300px because the bar
        // overlays it and takes no layout width. With the old solid bar the
        // content width was 288px (300 - 10 bar - 2 margins).
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut content_width = None;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("scroll_area_overlay_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
            );
            ScrollArea::new().height(120.0).show(ui, |ui| {
                content_width = Some(ui.available_width());
                for i in 0..20 {
                    ui.label(format!("Row {i}"));
                }
            });
        });
        out.textures_delta.clear();
        assert_eq!(content_width, Some(300.0));
    }

    #[test]
    fn thumb_painted_8px_wide() {
        // Painted-shape closing evidence for the 8px thumb (D4): exactly
        // one border-filled rect, 8px wide, 1px inside the bar's right edge,
        // radius 4; no 10px-wide border-filled rect (the old egui handle).
        let ctx = egui::Context::default();
        let preset = Preset::default();
        preset.build(Mode::Light).install(&ctx);
        let border = preset.build(Mode::Light).palette.border;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("scroll_area_thumb_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
            );
            ScrollArea::new()
                .height(120.0)
                .id_salt("thumb")
                .show(ui, |ui| {
                    for i in 0..20 {
                        ui.label(format!("Row {i}"));
                    }
                });
        });
        out.textures_delta.clear();
        let fills: Vec<(Rect, CornerRadius)> = out
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.fill == border => Some((rs.rect, rs.corner_radius)),
                _ => None,
            })
            .collect();
        assert_eq!(fills.len(), 1, "thumb fills: {fills:?}");
        let (thumb, radius) = fills[0];
        assert_eq!(thumb.width(), SCROLLBAR_THUMB_WIDTH);
        assert_eq!(thumb.min.x, 300.0 - SCROLLBAR_WIDTH + THUMB_INSET);
        assert_eq!(thumb.max.x, 300.0 - THUMB_INSET);
        assert_eq!(thumb.min.y, 0.0);
        assert_eq!(radius, CornerRadius::same(THUMB_RADIUS));
    }

    #[test]
    fn viewport_focus_ring_paints_on_focus() {
        // The viewport is one Tab stop (spec §7); on focus it paints the
        // 3px `ring/50` ring plus the 1px solid outline in `ring/50`.
        let ctx = egui::Context::default();
        let preset = Preset::default();
        preset.build(Mode::Light).install(&ctx);
        let ring50 = crate::theme::color_alpha(preset.build(Mode::Light).palette.ring, 0.5);

        let frame = |ctx: &egui::Context, input: egui::RawInput| {
            let mut id = egui::Id::NULL;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("scroll_area_focus_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                id = ScrollArea::new()
                    .height(120.0)
                    .id_salt("focus_ring")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                    })
                    .response
                    .id;
            });
            out.textures_delta.clear();
            (id, out.shapes)
        };

        // Frame 1: render, record the viewport response id; then focus it.
        let (id, _) = frame(&ctx, egui::RawInput::default());
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        // Frame 2: the focused viewport paints the ring + the outline.
        let (_, shapes) = frame(&ctx, egui::RawInput::default());
        let ring_strokes: Vec<(Rect, egui::Stroke)> = shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.stroke.color == ring50 => Some((rs.rect, rs.stroke)),
                _ => None,
            })
            .collect();
        assert!(
            ring_strokes.iter().any(|(_, s)| s.width == 3.0),
            "3px focus ring in ring/50 painted; ring strokes: {ring_strokes:?}"
        );
        assert!(
            ring_strokes.iter().any(|(_, s)| s.width == 1.0),
            "1px focus outline in ring/50 painted; ring strokes: {ring_strokes:?}"
        );
        // The ring sits just outside the viewport rect (expanded by 1px,
        // 3px stroke outside); the outline sits just outside the ring.
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 120.0));
        assert!(
            ring_strokes
                .iter()
                .any(|(r, s)| s.width == 3.0 && *r == viewport.expand(1.0)),
            "ring rect is the viewport expanded by 1px; ring strokes: {ring_strokes:?}"
        );
        assert!(
            ring_strokes
                .iter()
                .any(|(r, s)| s.width == 1.0 && *r == viewport.expand(4.0)),
            "outline rect is the viewport expanded by 4px; ring strokes: {ring_strokes:?}"
        );
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("scroll_area_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                ScrollArea::new()
                    .height(120.0)
                    .id_salt("render_scroll")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                    });
                ScrollArea::new()
                    .horizontal(true)
                    .vertical(false)
                    .height(60.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for i in 0..30 {
                                ui.label(format!("Item {i}"));
                            }
                        });
                    });
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn focused_descendant_outside_viewport_scrolls_into_view() {
        // SCA-03 (spec §11): a focused descendant below the viewport
        // scrolls the minimum amount into view on the focus change, and
        // the offset then stays put while focus is unchanged.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let button_id = std::cell::Cell::new(egui::Id::NULL);

        // One frame at `time`; returns (viewport rect, button rect).
        // The Ui id is stable across frames (Id::new, not unique) so
        // the per-instance last-focused tracking persists.
        let frame = |time: f64| -> (Rect, Rect) {
            let mut vp = Rect::NOTHING;
            let mut btn = Rect::NOTHING;
            let input = egui::RawInput {
                time: Some(time),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("sca03_scroll_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                let area = ScrollArea::new()
                    .height(120.0)
                    .id_salt("sca03_scroll")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                        let r = ui.button("Inner button");
                        button_id.set(r.id);
                        btn = r.rect;
                        // Trailing content so the button is not the last
                        // item: at the maximum offset the 4px ring margin
                        // would be clamped away (the offset cannot exceed
                        // content - viewport), so the margin is only
                        // reachable with slack below the button.
                        ui.label("Trailing — slack so the ring margin is reachable");
                    });
                vp = area.response.rect;
            });
            out.textures_delta.clear();
            (vp, btn)
        };

        // Frame 1: rest. The button sits below the 120px viewport.
        let (vp, btn0) = frame(0.0);
        assert_eq!(vp.height(), 120.0, "viewport is 120px");
        assert!(
            btn0.min.y > vp.max.y,
            "button starts below the viewport: button {btn0:?}, viewport {vp:?}"
        );

        // Focus the button, then drive frames with an advancing clock
        // until the scroll animation (<= 0.3s) settles.
        ctx.memory_mut(|m| m.request_focus(button_id.get()));
        let mut last = (vp, btn0);
        for step in 1..=20 {
            last = frame(0.05 * step as f64);
        }
        let (vp, btn) = last;
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(button_id.get()),
            "button keeps focus"
        );
        // Fully visible...
        assert!(
            btn.min.y >= vp.min.y - 0.5 && btn.max.y <= vp.max.y + 0.5,
            "button revealed in viewport: button {btn:?}, viewport {vp:?}"
        );
        // ... with the minimum scroll: the button sits at the bottom
        // edge (egui adds one item spacing), not at the top — plus the
        // 4px ring margin, so the focus ring clears the viewport edge
        // (the trailing label gives the scroll slack to reach it; for
        // a last item the margin is clamped away by the max offset).
        let gap = vp.max.y - btn.max.y;
        assert!(
            (2.0..=12.0).contains(&gap),
            "minimum scroll + ring margin: button bottom {gap:.2}px above viewport bottom"
        );
        assert!(
            btn.min.y >= vp.min.y + 40.0,
            "did not overshoot to the top: button {btn:?}, viewport {vp:?}"
        );

        // Focus unchanged: five more frames move nothing (no
        // re-assertion, no fight with the settled offset).
        for step in 21..=25 {
            let next = frame(0.05 * step as f64);
            assert_eq!(next, last, "offset stable while focus is unchanged");
        }
    }

    #[test]
    fn focus_on_non_descendant_does_not_scroll() {
        // SCA-03 guard: a focused widget OUTSIDE the area whose screen
        // rect merely overlaps the content span (the "Focus viewport"
        // button just below §4) must not scroll this area. Its interact
        // rect is its full rect, which sticks out of the viewport.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let inner_id = std::cell::Cell::new(egui::Id::NULL);
        let outer_id = std::cell::Cell::new(egui::Id::NULL);

        let frame = |time: f64| -> (Rect, Rect, Rect) {
            let mut vp = Rect::NOTHING;
            let mut inner = Rect::NOTHING;
            let mut outer = Rect::NOTHING;
            let input = egui::RawInput {
                time: Some(time),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("sca03_nondesc_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                let area = ScrollArea::new()
                    .height(120.0)
                    .id_salt("sca03_nondesc")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                        let r = ui.button("Inner button");
                        inner_id.set(r.id);
                        inner = r.rect;
                    });
                vp = area.response.rect;
                let r = ui.button("Outside button");
                outer_id.set(r.id);
                outer = r.rect;
            });
            out.textures_delta.clear();
            (vp, inner, outer)
        };

        let (vp, inner0, outer0) = frame(0.0);
        // The outer button overlaps the content span (below the
        // viewport) but is not a descendant of the area.
        assert!(
            outer0.min.y > vp.max.y && outer0.min.y < inner0.min.y,
            "outer button overlaps the content span: outer {outer0:?}, inner {inner0:?}"
        );

        ctx.memory_mut(|m| m.request_focus(outer_id.get()));
        for step in 1..=10 {
            let (vp2, inner2, _) = frame(0.05 * step as f64);
            assert_eq!(vp2, vp, "viewport unmoved");
            assert_eq!(
                inner2, inner0,
                "area did not scroll for a non-descendant (step {step})"
            );
        }
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(outer_id.get()),
            "outer button keeps focus"
        );
    }

    #[test]
    fn focus_on_visible_descendant_does_not_scroll() {
        // SCA-03: a focused descendant already inside the viewport
        // needs no scroll; the offset stays 0.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let top_id = std::cell::Cell::new(egui::Id::NULL);
        let inner_id = std::cell::Cell::new(egui::Id::NULL);

        let frame = |time: f64| -> (Rect, Rect) {
            let mut top = Rect::NOTHING;
            let mut inner = Rect::NOTHING;
            let input = egui::RawInput {
                time: Some(time),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("sca03_visible_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                ScrollArea::new()
                    .height(120.0)
                    .id_salt("sca03_visible")
                    .show(ui, |ui| {
                        let r = ui.button("Top button");
                        top_id.set(r.id);
                        top = r.rect;
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                        let r = ui.button("Inner button");
                        inner_id.set(r.id);
                        inner = r.rect;
                    });
            });
            out.textures_delta.clear();
            (top, inner)
        };

        let (top0, inner0) = frame(0.0);
        ctx.memory_mut(|m| m.request_focus(top_id.get()));
        for step in 1..=10 {
            let (top, inner) = frame(0.05 * step as f64);
            assert_eq!(top, top0, "visible button unmoved (step {step})");
            assert_eq!(inner, inner0, "offset stayed 0 (step {step})");
        }
    }

    #[test]
    fn focused_descendant_scrolls_horizontally_into_view() {
        // SCA-03 on the horizontal axis (§2 shape): a focused button
        // past the right edge scrolls the minimum amount into view.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let last_id = std::cell::Cell::new(egui::Id::NULL);

        let frame = |time: f64| -> (Rect, Rect) {
            let mut vp = Rect::NOTHING;
            let mut last = Rect::NOTHING;
            let input = egui::RawInput {
                time: Some(time),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("sca03_hscroll_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                let area = ScrollArea::new()
                    .vertical(false)
                    .horizontal(true)
                    .height(60.0)
                    .id_salt("sca03_hscroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for i in 0..30 {
                                let r = ui.button(format!("Tag {i}"));
                                if i == 29 {
                                    last_id.set(r.id);
                                    last = r.rect;
                                }
                            }
                            // Trailing slack so the 4px ring margin is
                            // reachable (see the vertical test).
                            ui.label("end");
                        });
                    });
                vp = area.response.rect;
            });
            out.textures_delta.clear();
            (vp, last)
        };

        let (vp, last0) = frame(0.0);
        assert!(
            last0.min.x > vp.max.x,
            "last tag starts past the right edge: tag {last0:?}, viewport {vp:?}"
        );

        ctx.memory_mut(|m| m.request_focus(last_id.get()));
        let mut settled = (vp, last0);
        for step in 1..=20 {
            settled = frame(0.05 * step as f64);
        }
        let (vp, last) = settled;
        assert!(
            last.min.x >= vp.min.x - 0.5 && last.max.x <= vp.max.x + 0.5,
            "last tag revealed in viewport: tag {last:?}, viewport {vp:?}"
        );
        let gap = vp.max.x - last.max.x;
        assert!(
            (2.0..=16.0).contains(&gap),
            "minimum horizontal scroll + ring margin: tag right edge {gap:.2}px from viewport right"
        );
    }

    #[test]
    fn height_is_enforced_far_down_a_scrolling_page() {
        // SCA-02: a ScrollArea with an explicit `.height()` laid out far
        // below the fold of a parent vertical ScrollArea rendered at
        // egui's 64px `min_scrolled_size` floor instead of the requested
        // height. `.height()` only set `max_height` (a cap), and the
        // available height at that cursor position is ~0 because the
        // parent content Ui's max_rect is the page viewport, so
        // `outer_size = min(available, max)` collapsed and the 64px
        // floor was all that remained. The requested height must be
        // enforced regardless of the available height.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut heights = (0.0, 0.0);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("scroll_area_height_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 3300.0))),
            );
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Push the cursor ~13000px down the page, as §5's 400
                // overflowing rows do on the gallery page.
                ui.add_space(13000.0);
                let both = ScrollArea::new()
                    .horizontal(true)
                    .height(140.0)
                    .id_salt("height_both")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                    });
                heights.0 = both.response.rect.height();
                let short = ScrollArea::new()
                    .height(80.0)
                    .id_salt("height_short")
                    .show(ui, |ui| {
                        ui.label("Just a couple of lines.");
                        ui.label("Second line.");
                    });
                heights.1 = short.response.rect.height();
            });
        });
        out.textures_delta.clear();
        assert_eq!(heights.0, 140.0, "§6 both-axes viewport is 140px");
        assert_eq!(heights.1, 80.0, "§7 short viewport is 80px");
    }
}
