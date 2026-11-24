//! shadcn `resizable.tsx` — draggable split-panel groups.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/resizable.tsx` (the
//! `react-resizable-panels` v4 wrapper): a [`ResizablePanelGroup`] lays out
//! [`ResizablePanel`] children separated by [`ResizableHandle`]s. Handles
//! are `bg-border` 1px lines with a 4px centred hit band and an optional
//! `with_handle` grip chip (base: `h-4 w-3 rounded-xs border bg-border` +
//! 10px grip glyph; nova: a plain `h-6 w-1 rounded-lg bg-border` pill,
//! 4x24 on a vertical bar / 24x4 on a horizontal bar). Focus ring: 1px
//! `ring` token, 1px offset, no outline (spec §2–§3).
//!
//! Verified final state (S10 re-verification, `resizable-s10-report.md`,
//! all four configs — nova/vega x light/dark — on commit `ca94bc0`):
//! rest geometry equals S7 exactly in every config — §1 handle x 393,
//! §1 177 rows nova / 155 vega (y 245–421 / 245–399); §3 handles at
//! 319 / 466; §4 y 889–924 nova / 893–928 vega; §6 y 1265–1300 nova /
//! 1286–1321 vega; §2 handles y 537 / 604 nova, 541 / 608 vega;
//! §5 y 1094 nova / 1115 vega. All five S9/S10 fixes verified FIXED in
//! all four configs: RSZ-01 arrow-key focus leak, RSZ-02 cascade,
//! RSZ-03 partner-line jitter, RSZ-04 restoring-drag shortfall, RSZ-05
//! panel-height collapse.
//!
//! Keyboard (S6/S10 verified): Tab reaches every handle (85-stop walk,
//! S10 R1); Arrow keys resize by exactly 5% of the group length per
//! press — the library value (`onKeyDown.ts` ±5pp), NOT an
//! approximation: OPEN-2 CLOSED at S8 §3.8. While a handle is focused
//! its arrows are locked to resizing by an `EventFilter` (the egui
//! `Slider` mechanism) plus `consume_key` (RSZ-01, `1312d54`); a
//! wrong-axis key moves neither the line nor the focus, and repeated
//! presses step again. Known limitation, shared with egui's
//! `Slider`/`TextEdit`: an arrow arriving in the very first pass after
//! focus is gained can still move focus. Home / End / Enter / F6 /
//! Space / PageUp / PageDown are no-ops — the library defines Home,
//! End, Enter and F6 (R10 OPEN-3 SETTLED) but S8 §3.8 decided DO NOT
//! ADD this wave (no `collapsible` API, so Enter would be a no-op; F6
//! duplicates Tab): a known scope gap for the Expert Reviewer, not a
//! defect. Clicking a handle does NOT focus it — the library's
//! pointerdown `preventDefault()` suppresses focus-on-mousedown, so
//! this is a documented equivalence (R10 OPEN-4 SETTLED), not a
//! divergence. Clicking does clear existing keyboard focus
//! (OBS-RSZ-S6-1), ACCEPTED at S8 §3.8 as documented behaviour — the
//! frozen spec does not pin what a click does to focus.
//!
//! Drag (S10 verified): a pointer press anywhere on the 4px hit band
//! starts a drag; the split moves with the pointer and converges
//! exactly on release (RSZ-04, `67c7d5c` — per-frame `drag_delta()`
//! drops the sub-6px first step, so the handle applies only the
//! not-yet-applied part of `total_drag_delta()`). When the immediate
//! neighbour hits its limit the excess CASCADES outward across panels
//! (RSZ-02, `d6e9d0e`): S8 §3.1 corrects frozen spec §10 edge case 2
//! from "excess is dropped" to the library's cascade
//! (`adjustLayoutByDelta.ts`); the S12 spec amendment records it.
//! Handle-1-max lands at (519, 520) in all four configs. Restoring
//! drags land exactly — h1 +50 → (369, 466), h1 −50 → (319, 466),
//! h2 +25 → (319, 491), h2 −25 → (319, 466) — and the non-dragged
//! partner line reads its settled value in every distinct frame
//! (RSZ-03, `e05dbeb`: per-frame fraction snapshot, since `adjust()`
//! mutates the live fractions mid-layout). R1 net-zero: end state
//! (319, 466) = rest, devs empty. Panels occupy exactly their fraction
//! width (RSZ-02) and their height comes from content (RSZ-05,
//! `ca94bc0`: single-pass, width-confined child Ui; the RSZ-02
//! `allocate_exact_size` arm froze horizontal panels at the 36px
//! `Ui::horizontal` seed height).
//!
//! States: the source defines NO hover, dragging, or disabled visuals
//! (spec §4.2/§4.4/§4.5) — a hovered or dragged handle looks exactly
//! like a resting one. Resize cursors on hover are a Rust extension
//! (E1). No animation: resizes apply immediately (spec §6). Budgets
//! (S10): idle CPU 0.0%, animating 85–118%, RSS −2.78%…+1.11%,
//! flicker zero, determinism zero. Render floor NOT VERIFIED (2.2–3.2
//! fps measured, pipeline-capped — the standing S7 pattern, not a
//! component defect). Timing UNRESOLVED in video legs (same caveat).
//!
//! Deviations (frozen spec §11 — all PROPOSED, for the Expert
//! Reviewer): D1 `:focus-visible` modality — the ring shows on any egui
//! focus, not keyboard-only (geometry and colours unchanged); D2
//! `data-slot` attributes dropped (no egui analogue); D3 no ARIA tree
//! — `aria-orientation` / `role="separator"` are unexpressible in egui,
//! orientation is the `ResizableOrientation` enum instead; D4
//! fraction-only sizing — pixel `defaultSize`/`minSize`/`maxSize` and
//! `collapsible` panels are out of scope (the pixel half is a real
//! scope-out: R10 OPEN-10 SETTLED — the library DOES accept pixels).
//! E1 resize cursors on hover — extension, confirmation only.
//!
//! OPEN inventory (frozen spec §12; R10 library-source evidence at S6,
//! `resizable-s6-report.md` R10): OPEN-1 `cn()` merge semantics STAYS
//! OPEN (S8 §3.8 — never closes on substitute reasoning); OPEN-2
//! keyboard step CLOSED at S8 §3.8 (5% is the library value); OPEN-3
//! Home/End/Enter/F6 SETTLED from the library source but NOT ADDED
//! this wave (S8 §3.8 scope gap); OPEN-4 handle focusability SETTLED
//! (`tabIndex={0}`, DOM order, click-does-not-focus equivalence);
//! OPEN-5 default direction SETTLED (horizontal); OPEN-6 initial
//! layout SETTLED (equal split — the port's first frame matches);
//! OPEN-7 v4 prop surface SETTLED (the audit's `autoSaveId` /
//! `storage` / `onLayout` / `onCollapse` / `onExpand` / `onDragging`
//! names are v2-era and do not exist in v4); OPEN-8 nesting SETTLED
//! in principle, runtime-unverified (no nested demo); OPEN-9 drag
//! mechanics SETTLED (percentage deltas from the pointerdown layout,
//! window-level pointer listeners, cascade clamp, no animation);
//! OPEN-10 pixel sizes SETTLED (accepted by the library — D4's pixel
//! half does NOT close as N/A); OPEN-11 nova pill border STAYS OPEN
//! (S8 §3.8: KEEP the frozen borderless value).
//!
//! Tokens (spec §5): `border` and `ring` agree between `globals.css`
//! and `themes.ts` — no divergence for this component's own tokens.
//! The base chip border and glyph use `currentColor` (foreground,
//! spec §0 correction 5, accepted at sign-off — NOT the `border`
//! token). Note: the light `--foreground` value itself is CONTESTED
//! wave-wide (card/table S11: `globals.css` `oklch(0% 0 0)` vs
//! `themes.ts` `oklch(0.145 0 0)`) — carried to S12 / Expert Reviewer.
//!
//! Scope notes: sizes are fractional (D4); panel/handle counts and
//! limits are learned across frames — the first frame uses an equal
//! split (OPEN-6, matches the library); fractions persist in the egui
//! context keyed by the group's id salt (the port's mapping of the
//! library's persistence); nested groups are supported in principle
//! by the group stack (OPEN-8, unverified); a handle whose group state
//! was evicted degrades to a static 1px line.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ResizablePanelGroup::new()
//!     .id_salt("demo")
//!     .height(200.0)
//!     .show(ui, |ui| {
//!         ResizablePanel::new().min(0.2).show(ui, |ui| ui.label("Left"));
//!         ResizableHandle::new().with_handle(true).show(ui);
//!         ResizablePanel::new().show(ui, |ui| ui.label("Right"));
//!     });
//! ```

use crate::icons::Icon;
use crate::theme::{ShadcnTheme, Style};
use egui::{
    Color32, CornerRadius, CursorIcon, Id, InnerResponse, Pos2, Rect, Response, Sense, Stroke, Ui,
    Vec2,
};

/// Group orientation, mirroring `aria-orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResizableOrientation {
    /// Panels side by side (`flex`).
    #[default]
    Horizontal,
    /// Panels stacked (`flex-col`).
    Vertical,
}

/// A handle between the panel at its index and the following panel.
#[derive(Clone, Copy, Debug)]
struct PanelLimit {
    min: f32,
    max: f32,
}

/// Per-group sizing state, persisted across frames in the egui context.
#[derive(Clone, Debug)]
struct GroupState {
    frame: u64,
    panels_drawn: usize,
    handles_drawn: usize,
    total_panels: usize,
    total_handles: usize,
    /// Fraction of the panel area per panel index.
    fractions: Vec<f32>,
    /// min/max fraction per panel index.
    limits: Vec<PanelLimit>,
    /// Total length along the layout axis (px).
    total_len: f32,
}

impl GroupState {
    /// Equal split for the current panel count.
    fn equal_fractions(&self) -> Vec<f32> {
        let n = self.total_panels.max(1);
        vec![1.0 / n as f32; n]
    }

    /// The pixel length available to panels (total minus handle lines).
    fn panels_px(&self) -> f32 {
        (self.total_len - self.total_handles as f32).max(0.0)
    }

    /// The fraction for panel `idx`, falling back to an equal split.
    fn fraction(&self, idx: usize) -> f32 {
        if self.fractions.len() == self.total_panels && self.total_panels > 0 {
            self.fractions[idx]
        } else {
            1.0 / self.total_panels.max(1) as f32
        }
    }
}

fn states_id() -> Id {
    Id::unique("shadcn_resizable_states")
}

fn read_state(ctx: &egui::Context, id: Id) -> Option<GroupState> {
    ctx.data_mut(|d| {
        d.get_temp::<Vec<(Id, GroupState)>>(states_id())
            .and_then(|all| all.into_iter().find(|(i, _)| *i == id).map(|(_, s)| s))
    })
}

fn write_state(ctx: &egui::Context, id: Id, state: GroupState) {
    ctx.data_mut(|d| {
        let mut all: Vec<(Id, GroupState)> = d.get_temp(states_id()).unwrap_or_default();
        if let Some(slot) = all.iter_mut().find(|(i, _)| *i == id) {
            slot.1 = state;
        } else {
            all.push((id, state));
        }
        d.insert_temp(states_id(), all);
    });
}

/// Per-frame fraction snapshot (RSZ-03): the fractions as they were when
/// the group's layout started this frame. `adjust()` mutates the live
/// fractions mid-layout (inside the dragged handle's `show`, which runs
/// after the panels before it but before the panels after it), so without
/// a snapshot the panels after the dragged handle render one drag step
/// ahead of the panels before it — the non-dragged partner line moves by
/// exactly one frame's drag delta for every frame of the drag, then
/// settles exactly. Panels lay out from this snapshot, so every panel in
/// the group sees the same fractions in a frame; the live fractions take
/// effect on the next frame's snapshot. Keyed by group id, same shape as
/// the states map above.
fn frame_fractions_id() -> Id {
    Id::unique("shadcn_resizable_frame_fractions")
}

fn write_frame_fractions(ctx: &egui::Context, id: Id, fractions: Vec<f32>) {
    ctx.data_mut(|d| {
        let mut all: Vec<(Id, Vec<f32>)> = d.get_temp(frame_fractions_id()).unwrap_or_default();
        if let Some(slot) = all.iter_mut().find(|(i, _)| *i == id) {
            slot.1 = fractions;
        } else {
            all.push((id, fractions));
        }
        d.insert_temp(frame_fractions_id(), all);
    });
}

fn read_frame_fractions(ctx: &egui::Context, id: Id) -> Option<Vec<f32>> {
    ctx.data_mut(|d| {
        d.get_temp::<Vec<(Id, Vec<f32>)>>(frame_fractions_id())
            .and_then(|all| all.into_iter().find(|(i, _)| *i == id).map(|(_, f)| f))
    })
}

/// Per-handle drag convergence state (RSZ-04), persisted in temp data
/// keyed by the handle's response id. `applied_px` is the total
/// displacement already applied to the split in the current drag;
/// `origin_px` is the pointer's axis position at the press, recovered
/// on the first dragged frame as the pointer position minus
/// `total_drag_delta()` — it lets the release frame converge on the
/// final pointer position even though `total_drag_delta()` is
/// unavailable once the button is up.
#[derive(Clone, Copy, Debug, Default)]
struct DragConverge {
    applied_px: f32,
    origin_px: Option<f32>,
}

fn drag_states_id() -> Id {
    Id::unique("shadcn_resizable_drag_states")
}

fn read_drag_state(ctx: &egui::Context, id: Id) -> DragConverge {
    ctx.data_mut(|d| {
        d.get_temp::<Vec<(Id, DragConverge)>>(drag_states_id())
            .and_then(|all| all.into_iter().find(|(i, _)| *i == id).map(|(_, s)| s))
            .unwrap_or_default()
    })
}

fn write_drag_state(ctx: &egui::Context, id: Id, state: DragConverge) {
    ctx.data_mut(|d| {
        let mut all: Vec<(Id, DragConverge)> = d.get_temp(drag_states_id()).unwrap_or_default();
        if let Some(slot) = all.iter_mut().find(|(i, _)| *i == id) {
            slot.1 = state;
        } else {
            all.push((id, state));
        }
        d.insert_temp(drag_states_id(), all);
    });
}

/// Group context shared with panels and handles during layout.
#[derive(Clone, Copy, Debug)]
struct GroupCtx {
    id: Id,
    orientation: ResizableOrientation,
}

fn group_stack_id() -> Id {
    Id::unique("shadcn_resizable_group_stack")
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

/// Registers a panel; returns its index.
fn next_panel(ui: &Ui, id: Id, limit: PanelLimit) -> usize {
    let ctx = ui.ctx().clone();
    let mut st = read_state(&ctx, id).unwrap_or(GroupState {
        frame: ctx.cumulative_frame_nr(),
        panels_drawn: 0,
        handles_drawn: 0,
        total_panels: 0,
        total_handles: 0,
        fractions: vec![],
        limits: vec![],
        total_len: 0.0,
    });
    let idx = st.panels_drawn;
    st.panels_drawn += 1;
    if st.limits.len() > idx {
        st.limits[idx] = limit;
    } else {
        st.limits.push(limit);
    }
    write_state(&ctx, id, st);
    idx
}

/// Registers a handle; returns its index and the group state snapshot.
fn next_handle(ui: &Ui, id: Id) -> Option<(usize, GroupState)> {
    let ctx = ui.ctx().clone();
    let mut st = read_state(&ctx, id)?;
    let idx = st.handles_drawn;
    st.handles_drawn += 1;
    write_state(&ctx, id, st.clone());
    Some((idx, st))
}

/// Adjusts the split at separator `idx` by a fractional delta `d`,
/// clamped to the panels' limits. When the immediate neighbour hits its
/// limit, the leftover continues into the next panel in the same
/// direction — the library's `adjustLayoutByDelta` cascade (S8 §3.1
/// corrects frozen spec §10 edge case 2 to the cascade; the "excess is
/// dropped" rule was written without the drag-algorithm evidence).
/// Returns the applied delta.
///
/// Direction: `d > 0` grows panel `idx` and shrinks panels `idx + 1`,
/// `idx + 2`, … in turn; `d < 0` grows panel `idx + 1` and shrinks
/// panels `idx`, `idx - 1`, … in turn. The growing panel's own max is a
/// hard stop — the walk only ever continues on the shrinking side.
fn adjust(ui: &Ui, id: Id, idx: usize, d: f32) -> f32 {
    let ctx = ui.ctx().clone();
    let Some(mut st) = read_state(&ctx, id) else {
        return 0.0;
    };
    if st.fractions.len() != st.total_panels || st.total_panels < 2 || idx + 1 >= st.total_panels {
        // Fall back to an equal split before adjusting.
        st.fractions = st.equal_fractions();
        if st.total_panels < 2 || idx + 1 >= st.total_panels {
            write_state(&ctx, id, st);
            return 0.0;
        }
    }
    let get = |st: &GroupState, i: usize| -> (f32, f32) {
        let lim = st
            .limits
            .get(i)
            .copied()
            .unwrap_or(PanelLimit { min: 0.0, max: 1.0 });
        (lim.min, lim.max)
    };
    let n = st.total_panels;
    // Room on the growing side (the single adjacent panel) and on the
    // shrinking side (every panel outward from the separator).
    let applied = if d > 0.0 {
        let (_, lmax) = get(&st, idx);
        let grow = (lmax - st.fractions[idx]).max(0.0);
        let shrink = (idx + 1..n)
            .map(|j| {
                let (jmin, _) = get(&st, j);
                (st.fractions[j] - jmin).max(0.0)
            })
            .sum::<f32>();
        d.clamp(0.0, grow.min(shrink))
    } else {
        let (_, rmax) = get(&st, idx + 1);
        let grow = (rmax - st.fractions[idx + 1]).max(0.0);
        let shrink = (0..=idx)
            .map(|j| {
                let (jmin, _) = get(&st, j);
                (st.fractions[j] - jmin).max(0.0)
            })
            .sum::<f32>();
        d.clamp(-grow.min(shrink), 0.0)
    };
    if applied == 0.0 {
        return 0.0;
    }
    if applied > 0.0 {
        st.fractions[idx] += applied;
        let mut remaining = applied;
        for j in idx + 1..n {
            if remaining <= 0.0 {
                break;
            }
            let (jmin, _) = get(&st, j);
            let take = remaining.min((st.fractions[j] - jmin).max(0.0));
            st.fractions[j] -= take;
            remaining -= take;
        }
    } else {
        st.fractions[idx + 1] -= applied;
        let mut remaining = -applied;
        for j in (0..=idx).rev() {
            if remaining <= 0.0 {
                break;
            }
            let (jmin, _) = get(&st, j);
            let take = remaining.min((st.fractions[j] - jmin).max(0.0));
            st.fractions[j] -= take;
            remaining -= take;
        }
    }
    write_state(&ctx, id, st);
    applied
}

/// A shadcn resizable panel group. See the [module docs](self) for an example.
/// Horizontal by default — the library default (R10 OPEN-5 SETTLED).
/// Fractions persist per group id salt; the first frame lays panels out
/// at an equal split (OPEN-6, matches the library).
#[derive(Clone, Debug, Default)]
pub struct ResizablePanelGroup {
    orientation: ResizableOrientation,
    /// Fixed length along the layout axis; defaults to the available space
    /// (`h-full w-full`).
    length: Option<f32>,
    id_salt: Option<String>,
}

impl ResizablePanelGroup {
    /// Creates a horizontal panel group.
    pub fn new() -> Self {
        Self {
            orientation: ResizableOrientation::Horizontal,
            length: None,
            id_salt: None,
        }
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: ResizableOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Fixes the group length along the layout axis (width for horizontal,
    /// height for vertical). Defaults to the available space.
    pub fn length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }

    /// Sets a stable id salt; sizes persist per group under this id.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its panels and handles.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_resizable_group"),
        };
        let total_len = match (self.orientation, self.length) {
            (_, Some(l)) => l.max(0.0),
            (ResizableOrientation::Horizontal, None) => ui.available_width(),
            (ResizableOrientation::Vertical, None) => ui.available_height(),
        };

        // Begin: reset per-frame counters, keep fractions.
        let ctx = ui.ctx().clone();
        let frame = ctx.cumulative_frame_nr();
        let mut st = read_state(&ctx, id).unwrap_or(GroupState {
            frame,
            panels_drawn: 0,
            handles_drawn: 0,
            total_panels: 0,
            total_handles: 0,
            fractions: vec![],
            limits: vec![],
            total_len,
        });
        if st.frame != frame {
            st.frame = frame;
            st.panels_drawn = 0;
            st.handles_drawn = 0;
            st.total_len = total_len;
        }
        // RSZ-03: freeze this frame's fractions for the layout below.
        // The dragged handle's `adjust()` mutates the live fractions
        // mid-layout; panels read the snapshot instead, so every panel
        // in the group lays out from the same fractions.
        write_frame_fractions(&ctx, id, st.fractions.clone());
        write_state(&ctx, id, st);
        push_group(
            ui,
            GroupCtx {
                id,
                orientation: self.orientation,
            },
        );

        let inner = match self.orientation {
            ResizableOrientation::Horizontal => ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.set_min_width(total_len);
                content(ui)
            }),
            ResizableOrientation::Vertical => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.set_min_height(total_len);
                content(ui)
            }),
        };

        pop_group(ui);

        // End: record totals for the next frame.
        let ctx = ui.ctx().clone();
        if let Some(mut st) = read_state(&ctx, id) {
            st.total_panels = st.panels_drawn;
            st.total_handles = st.handles_drawn;
            if st.fractions.len() != st.total_panels && st.total_panels > 0 {
                st.fractions = st.equal_fractions();
            }
            write_state(&ctx, id, st);
        }

        InnerResponse::new(inner.inner, inner.response)
    }
}

/// A resizable panel: a flexible child of [`ResizablePanelGroup`].
/// Its size is a fraction of the group (`.min` / `.max` clamp it); its
/// width is exactly its fraction (RSZ-02) and its height comes from its
/// content (RSZ-05).
#[derive(Clone, Copy, Debug)]
pub struct ResizablePanel {
    min: f32,
    max: f32,
}

impl Default for ResizablePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ResizablePanel {
    /// Creates a panel with default limits (0..1 of the group).
    pub fn new() -> Self {
        Self { min: 0.0, max: 1.0 }
    }

    /// Minimum fraction of the group this panel can shrink to.
    pub fn min(mut self, min: f32) -> Self {
        self.min = min.clamp(0.0, 1.0);
        self
    }

    /// Maximum fraction of the group this panel can grow to.
    pub fn max(mut self, max: f32) -> Self {
        self.max = max.clamp(0.0, 1.0);
        self
    }

    /// Renders the panel around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let Some(group) = current_group(ui) else {
            // Outside a group: lay out normally in the flow.
            let inner = ui.vertical(content);
            return InnerResponse::new(inner.inner, inner.response);
        };
        let idx = next_panel(
            ui,
            group.id,
            PanelLimit {
                min: self.min,
                max: self.max,
            },
        );
        let st = read_state(ui.ctx(), group.id).unwrap_or_else(|| GroupState {
            frame: 0,
            panels_drawn: 1,
            handles_drawn: 0,
            total_panels: 1,
            total_handles: 0,
            fractions: vec![],
            limits: vec![],
            total_len: 0.0,
        });
        // RSZ-03: lay out from this frame's fraction snapshot, not the
        // live fractions — the dragged handle's `adjust()` mutates the
        // live fractions mid-layout, and panels after the handle would
        // otherwise render one drag step ahead of the panels before it
        // (the partner line moves by exactly one frame's drag delta for
        // every frame of the drag). Falls back to the live fractions
        // when no snapshot covers this panel count (first frame).
        let frac = match read_frame_fractions(ui.ctx(), group.id) {
            Some(snap) if snap.len() == st.total_panels && idx < snap.len() => snap[idx],
            _ => st.fraction(idx),
        } * st.panels_px();

        // RSZ-02: a panel occupies exactly its fraction width — no more,
        // no less. `ui.allocate_ui` grows to the content's `min_rect`
        // ("if the contents overflow, more space will be allocated"), so
        // a panel whose fraction shrank below its content's minimum width
        // pushed the next handle right: dragging handle 1 past the
        // middle's minimum left the fractions exact (middle 0) but the
        // middle panel still rendered ~14px wide and l2 sat at 509
        // instead of the cascaded 520.
        //
        // RSZ-05: the width must stay exactly `frac`, but the HEIGHT
        // must come from the content — the RSZ-02 `allocate_exact_size`
        // froze it at `ui.available_height()`, which inside a horizontal
        // group is the row height so far, seeded at
        // `spacing().interact_size.y` = 36px (`Ui::horizontal`).
        // `allocate_exact_size` never grows to content, so every
        // horizontal panel rendered 36px tall and §1's labels and
        // "Add item" button were clipped invisible. The fix lays the
        // content out once in a child Ui confined to `frac` width (the
        // same child the old `allocate_ui` path used, with the same
        // `available_height()` max height), then allocates exactly
        // `frac` × the child's `min_rect` height in the parent — the
        // old code's height behaviour, with the width pinned. Content
        // runs once, so no double side effects (the "Add item" click
        // cannot double-count). The clip rect is a horizontal band
        // only: the parent's clip rect already bounds the paint
        // vertically (window / scroll viewport), and clipping to the
        // panel height was the RSZ-05 bug; the band keeps the RSZ-02
        // `overflow: hidden` behaviour for content wider than `frac`.
        let inner = match group.orientation {
            ResizableOrientation::Horizontal => {
                let origin = ui.cursor().min;
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(
                    origin,
                    Vec2::new(frac, ui.available_height()),
                )));
                let clip = child.clip_rect();
                child.shrink_clip_rect(Rect::from_min_max(
                    Pos2::new(origin.x, clip.min.y),
                    Pos2::new(origin.x + frac, clip.max.y),
                ));
                child.set_min_width(frac);
                child.set_max_width(frac);
                let ret = child.vertical(content).inner;
                let panel_rect =
                    Rect::from_min_size(origin, Vec2::new(frac, child.min_rect().height()));
                let response = ui.allocate_rect(panel_rect, Sense::hover());
                InnerResponse::new(ret, response)
            }
            ResizableOrientation::Vertical => {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), frac), Sense::hover());
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                child.shrink_clip_rect(rect);
                child.set_min_height(frac);
                child.set_max_height(frac);
                let ret = child.vertical(content).inner;
                InnerResponse::new(ret, response)
            }
        };
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// Frozen grip geometry (§2.4, §2.5, §3.3).
///
/// Base chip: 12×16 on a vertical bar, 16×12 on a horizontal bar
/// (rotated 90°). Nova pill: 4×24 on a vertical bar, 24×4 on a
/// horizontal bar.
fn chip_size(nova: bool, horizontal_group: bool) -> Vec2 {
    match (nova, horizontal_group) {
        (true, true) => Vec2::new(4.0, 24.0),
        (true, false) => Vec2::new(24.0, 4.0),
        (false, true) => Vec2::new(12.0, 16.0),
        (false, false) => Vec2::new(16.0, 12.0),
    }
}

/// Base grip chip border colour: `currentColor` — the inherited text
/// colour, i.e. the foreground token (§0 correction 5, accepted by the
/// coordinator at sign-off). NOT the `border` token. The nova pill has
/// no border (OPEN-11 frozen value).
fn chip_border_color(p: &crate::theme::Palette) -> Color32 {
    p.foreground
}

/// A draggable separator between panels: a 1px `bg-border` line with an
/// optional grip chip. Drag it, or focus it and use the Arrow keys.
/// Arrow steps are 5% of the group length — the library value (OPEN-2
/// CLOSED at S8 §3.8). Clicking does not focus it (R10 OPEN-4
/// equivalence: the library's pointerdown `preventDefault()`).
#[derive(Clone, Debug, Default)]
pub struct ResizableHandle {
    with_handle: bool,
    id_salt: Option<String>,
}

impl ResizableHandle {
    /// Creates a thin handle.
    pub fn new() -> Self {
        Self {
            with_handle: false,
            id_salt: None,
        }
    }

    /// Shows the grip chip (`h-4 w-3 rounded-xs border bg-border`).
    pub fn with_handle(mut self, with_handle: bool) -> Self {
        self.with_handle = with_handle;
        self
    }

    /// Sets a stable id salt (focus state persists per handle).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the handle.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let Some(group) = current_group(ui) else {
            // Outside a group: draw a static separator line.
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(1.0, ui.available_height()), Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 0, p.border);
            }
            return response;
        };
        let Some((idx, st)) = next_handle(ui, group.id) else {
            // State was evicted mid-frame; degrade to a static line.
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(1.0, ui.available_height()), Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 0, p.border);
            }
            return response;
        };

        let horizontal_group = group.orientation == ResizableOrientation::Horizontal;
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => group.id.with(format!("resizable_handle_{idx}")),
        };

        // Flow: 1px line across the cross axis.
        let desired = if horizontal_group {
            Vec2::new(1.0, ui.available_height())
        } else {
            Vec2::new(ui.available_width(), 1.0)
        };
        let (_, flow_rect) = ui.allocate_space(desired);

        // Expanded interaction rect (the source's `after:` hit area, a 4px
        // band centered on the 1px line).
        let hit = if horizontal_group {
            flow_rect.expand2(egui::vec2(1.5, 0.0))
        } else {
            flow_rect.expand2(egui::vec2(0.0, 1.5))
        };
        let response = ui
            .interact(hit, id, Sense::click_and_drag())
            .on_hover_cursor(if horizontal_group {
                CursorIcon::ResizeHorizontal
            } else {
                CursorIcon::ResizeVertical
            });

        if ui.is_rect_visible(flow_rect) {
            let painter = ui.painter();
            painter.rect_filled(flow_rect, 0, p.border);

            if self.with_handle {
                let nova = theme.style == Style::Nova;
                if nova {
                    // Nova grip: a plain `bg-border` pill (`h-6 w-1
                    // rounded-lg`, rotated on horizontal groups), no glyph,
                    // no border (OPEN-11 frozen value).
                    let chip = Rect::from_center_size(
                        flow_rect.center(),
                        chip_size(true, horizontal_group),
                    );
                    painter.rect_filled(chip, CornerRadius::same(2), p.border);
                } else {
                    // Base grip chip: `h-4 w-3 rounded-xs border bg-border`
                    // with the `size-2.5` grip glyph (the source rotates the
                    // chip on horizontal dividers, so the glyph follows).
                    let chip = Rect::from_center_size(
                        flow_rect.center(),
                        chip_size(false, horizontal_group),
                    );
                    painter.rect_filled(chip, CornerRadius::same(2), p.border);
                    painter.rect_stroke(
                        chip,
                        CornerRadius::same(2),
                        // `border` = 1px solid `currentColor` (foreground),
                        // §0 correction 5 — NOT the `border` token.
                        Stroke::new(1.0, chip_border_color(&p)),
                        egui::epaint::StrokeKind::Inside,
                    );
                    let glyph = if horizontal_group {
                        Icon::GripVertical
                    } else {
                        Icon::GripHorizontal
                    };
                    let glyph_rect = Rect::from_center_size(chip.center(), Vec2::splat(10.0));
                    // The lucide glyph inherits the text color (`currentColor`)
                    // over the `bg-border` chip.
                    glyph.paint(painter, glyph_rect, p.foreground, 2.0 * 10.0 / 24.0);
                }
            }

            // `focus-visible:ring-1 focus-visible:ring-ring`
            if response.has_focus() {
                crate::utils::focus::paint_ring(
                    painter,
                    flow_rect.expand2(egui::vec2(1.0, 1.0)),
                    color_ring(&p),
                    1.0,
                    2.0,
                );
            }
        }

        // Pointer drag adjusts the split. RSZ-04: per-frame
        // `drag_delta()` is lossy — any frame in which `dragged()` is
        // false (notably the first frames after pointer-down, before
        // egui's click-vs-drag decision) drops that frame's movement,
        // and the widget never recovers it because it never applies an
        // absolute position. Instead, apply the not-yet-applied part of
        // the total drag displacement: `total_drag_delta()` is the
        // pointer position minus the press origin, so once the
        // decision is made it already includes the pre-decision
        // movement, and the line converges on the pointer regardless
        // of frame alignment. The already-applied total is tracked
        // per handle below, so a clamp in one frame does not lose the
        // remainder — the next frame's increment still carries it.
        let ctx = ui.ctx().clone();
        let mut drag = read_drag_state(&ctx, response.id);
        let pointer_axis =
            response
                .interact_pointer_pos()
                .map(|p| if horizontal_group { p.x } else { p.y });
        let dragged_px = if response.dragged() {
            match (response.total_drag_delta(), pointer_axis) {
                (Some(total), Some(pos)) => {
                    let total_px = if horizontal_group { total.x } else { total.y };
                    if drag.origin_px.is_none() {
                        drag.origin_px = Some(pos - total_px);
                    }
                    total_px - drag.applied_px
                }
                _ => 0.0,
            }
        } else if response.drag_stopped() {
            // The button came up this frame: `total_drag_delta()` is
            // no longer available, so converge from the stored press
            // origin and the release position.
            match (drag.origin_px, pointer_axis) {
                (Some(origin), Some(pos)) => (pos - origin) - drag.applied_px,
                _ => 0.0,
            }
        } else {
            0.0
        };

        // Arrow-key resizing (5% steps) when focused. `has_focus` takes the
        // context lock itself, so it must run outside `ui.input`.
        let focused = response.has_focus();
        let key_px: f32 = if focused {
            // RSZ-01: lock the arrow keys to this handle so stepping never
            // moves focus. egui decides the focus direction at the START of
            // the pass from the raw input (`Focus::begin_pass`), so merely
            // consuming the key mid-pass cannot stop it — the focused widget
            // must carry an `EventFilter` (the same mechanism `Slider` uses
            // in `slider_core::keyboard_steps`). Both axes are locked: the
            // on-axis arrows resize, and a wrong-axis arrow must leave both
            // the line AND the focus unchanged (S8). The filter only takes
            // effect once the handle had focus last frame, so it is set on
            // every focused pass; an arrow arriving in the very first pass
            // after focus is gained can still move focus (the same one-frame
            // hole egui's own `Slider`/`TextEdit` have).
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                );
            });
            let step = 0.05 * st.panels_px();
            ui.input_mut(|i| {
                let (prev_key, next_key) = if horizontal_group {
                    (egui::Key::ArrowLeft, egui::Key::ArrowRight)
                } else {
                    (egui::Key::ArrowUp, egui::Key::ArrowDown)
                };
                // Consume the arrows as well, so no other widget in this
                // pass acts on them. The wrong-axis arrows never resize,
                // but they are consumed too — a mistaken keypress must not
                // cost the user their focus.
                let (other_prev, other_next) = if horizontal_group {
                    (egui::Key::ArrowUp, egui::Key::ArrowDown)
                } else {
                    (egui::Key::ArrowLeft, egui::Key::ArrowRight)
                };
                i.consume_key(egui::Modifiers::NONE, other_prev);
                i.consume_key(egui::Modifiers::NONE, other_next);
                if i.consume_key(egui::Modifiers::NONE, prev_key) {
                    -step
                } else if i.consume_key(egui::Modifiers::NONE, next_key) {
                    step
                } else {
                    0.0
                }
            })
        } else {
            0.0
        };

        // Drag and arrow-key steps are applied separately so the drag
        // bookkeeping above credits exactly the px the drag applied
        // (a clamp binds the same either way: the two sequential
        // `adjust()` calls reach the same limit as one combined call).
        let mut repaint = false;
        if dragged_px != 0.0 && st.total_panels >= 2 {
            let d = dragged_px / st.panels_px().max(1.0);
            let applied = adjust(ui, group.id, idx, d) * st.panels_px();
            drag.applied_px += applied;
            repaint |= applied.abs() > 0.01;
        }
        if key_px != 0.0 && st.total_panels >= 2 {
            let d = key_px / st.panels_px().max(1.0);
            let applied = adjust(ui, group.id, idx, d) * st.panels_px();
            repaint |= applied.abs() > 0.01;
        }
        if repaint {
            ui.ctx().request_repaint();
        }

        // Persist the convergence state for the next frame of this
        // drag; any frame that is neither dragging nor the release
        // frame resets it, so the next drag starts from zero.
        if response.dragged() || response.drag_stopped() {
            write_drag_state(&ctx, response.id, drag);
        } else {
            write_drag_state(&ctx, response.id, DragConverge::default());
        }

        response
    }
}

fn color_ring(p: &crate::theme::Palette) -> Color32 {
    p.ring
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset, Style};
    use egui::{Key, Pos2};

    #[test]
    fn equal_split_and_panel_px() {
        let st = GroupState {
            frame: 0,
            panels_drawn: 3,
            handles_drawn: 2,
            total_panels: 3,
            total_handles: 2,
            fractions: vec![],
            limits: vec![],
            total_len: 402.0,
        };
        assert!((st.panels_px() - 400.0).abs() < 1e-4);
        let f = st.fraction(0);
        assert!((f - 1.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn adjust_clamps_to_limits() {
        let ctx = egui::Context::default();
        let id = Id::unique("adjust_test");
        // Two panels 50/50; panel 0 cannot drop below 0.4.
        let st = GroupState {
            frame: ctx.cumulative_frame_nr(),
            panels_drawn: 2,
            handles_drawn: 1,
            total_panels: 2,
            total_handles: 1,
            fractions: vec![0.5, 0.5],
            limits: vec![
                PanelLimit { min: 0.4, max: 1.0 },
                PanelLimit { min: 0.0, max: 1.0 },
            ],
            total_len: 101.0,
        };
        write_state(&ctx, id, st);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("adjust_test_ui"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
            );
            // Try to shrink panel 0 by 0.2 — clamps to 0.4.
            let applied = adjust(ui, id, 0, -0.2);
            assert!((applied + 0.1).abs() < 1e-4);
            let st = read_state(ctx, id).unwrap();
            assert!((st.fractions[0] - 0.4).abs() < 1e-4);
            assert!((st.fractions[1] - 0.6).abs() < 1e-4);
            // Growing panel 0 is unconstrained up to 1.0.
            let applied = adjust(ui, id, 0, 0.7);
            assert!((applied - 0.6).abs() < 1e-4);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn renders_and_drags_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("resizable_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                ResizablePanelGroup::new()
                    .id_salt("render_h")
                    .length(400.0)
                    .show(ui, |ui| {
                        ResizablePanel::new().min(0.2).show(ui, |ui| {
                            ui.label("Left");
                        });
                        ResizableHandle::new().with_handle(true).show(ui);
                        ResizablePanel::new().show(ui, |ui| {
                            ui.label("Right");
                        });
                    });
                ResizablePanelGroup::new()
                    .id_salt("render_v")
                    .orientation(ResizableOrientation::Vertical)
                    .length(220.0)
                    .show(ui, |ui| {
                        ResizablePanel::new().show(ui, |ui| {
                            ui.label("Top");
                        });
                        ResizableHandle::new().show(ui);
                        ResizablePanel::new().show(ui, |ui| {
                            ui.label("Bottom");
                        });
                    });
            });
            out.textures_delta.clear();

            // Keyboard resize on the focused first handle.
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::Key {
                key: Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("resizable_key_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                ResizablePanelGroup::new()
                    .id_salt("render_h")
                    .length(400.0)
                    .show(ui, |ui| {
                        ResizablePanel::new().min(0.2).show(ui, |ui| {
                            ui.label("Left");
                        });
                        ResizableHandle::new().show(ui);
                        ResizablePanel::new().show(ui, |ui| {
                            ui.label("Right");
                        });
                    });
            });
            out.textures_delta.clear();
        }
    }

    /// RSZ-01: arrow keys resize a focused handle repeatedly without moving
    /// focus — including a second press in the same direction, and a
    /// wrong-axis key (which must move neither the line nor the focus).
    ///
    /// Mechanism under test: the handle locks both arrow axes via
    /// `set_focus_lock_filter` (the `Slider` mechanism). The filter only
    /// takes effect once the handle had focus last frame, so the test lets
    /// one idle pass elapse after focusing before pressing any arrow — the
    /// same settling the gallery drivers do. A focusable button sits to the
    /// left of the group so that, pre-fix, the first ArrowLeft has somewhere
    /// to move focus to (without it, `find_widget_in_direction` finds
    /// nothing and the test would pass even unfixed).
    #[test]
    fn arrow_keys_resize_repeatedly_without_losing_focus() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let ui_id = egui::Id::new("rsz01_arrow_focus_test");

            // Renders the button + group for one pass and returns the handle
            // line's x position (from its response rect) and the handle's
            // real interact id (from its response — captured, not derived,
            // so `request_focus` targets the exact widget).
            let render = |ctx: &egui::Context, input: egui::RawInput| -> (f32, egui::Id) {
                let mut handle_x = f32::NAN;
                let mut handle_id = egui::Id::new("rsz01_no_handle_yet");
                let mut out = ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        ui_id,
                        egui::UiBuilder::new()
                            .layout(egui::Layout::left_to_right(egui::Align::Min))
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                    );
                    let _ = ui.button("left");
                    ResizablePanelGroup::new()
                        .id_salt("rsz01")
                        .length(400.0)
                        .show(ui, |ui| {
                            ResizablePanel::new().min(0.2).show(ui, |ui| {
                                ui.label("Left");
                            });
                            let response = ResizableHandle::new().show(ui);
                            handle_x = response.rect.center().x;
                            handle_id = response.id;
                            ResizablePanel::new().show(ui, |ui| {
                                ui.label("Right");
                            });
                        });
                });
                out.textures_delta.clear();
                (handle_x, handle_id)
            };

            let key_event = |key: Key| {
                let mut input = egui::RawInput::default();
                input.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                });
                input
            };

            // Pass 1 (warm-up): establishes the panel totals, so the
            // split is 50/50 from the next pass on (in the very first pass
            // `total_panels` is still 0 while panels lay out, so the left
            // panel takes the full width and the handle sits at the far
            // right). Pass 2 measures the settled 50/50 line; focus the
            // handle.
            let _ = render(&ctx, egui::RawInput::default());
            let (x0, handle_id) = render(&ctx, egui::RawInput::default());
            ctx.memory_mut(|m| m.request_focus(handle_id));
            assert_eq!(ctx.memory(|m| m.focused()), Some(handle_id));
            // Pass 3 (idle): the focus lock filter is set during this
            // pass's show; an arrow pressed before this pass would still
            // move focus (egui's one-frame hole, same as `Slider`).
            let (x1, _) = render(&ctx, egui::RawInput::default());
            assert!((x1 - x0).abs() < 0.01, "idle pass must not move the line");
            assert_eq!(ctx.memory(|m| m.focused()), Some(handle_id));
            // Pass 4: ArrowLeft resizes by one 5% step and keeps focus.
            // The fraction changes in this pass, but the panels were already
            // laid out at the old split, so the line only VISIBLY moves in
            // the next pass (one-frame layout lag — the same lag the drag
            // path has). This pass asserts focus only.
            let step = 0.05 * (400.0 - 1.0);
            let (x2, _) = render(&ctx, key_event(Key::ArrowLeft));
            assert!(
                (x2 - x1).abs() < 0.01,
                "line moves in the next pass, not this one"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(handle_id));
            // Pass 5: the first Left's resize is now visible, and a second
            // ArrowLeft resizes again — focus must still be on the handle.
            let (x3, _) = render(&ctx, key_event(Key::ArrowLeft));
            assert!((x3 - (x1 - step)).abs() < 1.0, "first Left must move -5%");
            assert_eq!(ctx.memory(|m| m.focused()), Some(handle_id));
            // Pass 6: the second Left's resize is now visible. Pre-fix,
            // focus had left the handle after pass 4, so pass 5 resized
            // nothing and this line would be unchanged from pass 5.
            let (x4, _) = render(&ctx, key_event(Key::ArrowLeft));
            assert!(
                (x4 - (x3 - step)).abs() < 1.0,
                "second Left must move -5% again"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(handle_id));
            // Pass 7: wrong-axis ArrowUp must not move focus. (The line
            // still moves one lagged step here — pass 6's resize becoming
            // visible — which is expected, not a failure.)
            let (x5, _) = render(&ctx, key_event(Key::ArrowUp));
            assert!(
                (x5 - (x4 - step)).abs() < 1.0,
                "pass 6 resize becomes visible here"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(handle_id));
            // Pass 8 (idle): settled — the ArrowUp changed nothing, so the
            // line is unmoved from pass 7.
            let (x6, _) = render(&ctx, egui::RawInput::default());
            assert!(
                (x6 - x5).abs() < 0.01,
                "wrong-axis key must not move the line"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(handle_id));
        }
    }

    /// Frozen chip geometry (§2.4, §2.5, §3.3).
    #[test]
    fn chip_size_matches_frozen_geometry() {
        // Base chip: 12×16 on a vertical bar, 16×12 rotated.
        assert_eq!(chip_size(false, true), Vec2::new(12.0, 16.0));
        assert_eq!(chip_size(false, false), Vec2::new(16.0, 12.0));
        // Nova pill: 4×24 on a vertical bar, 24×4 rotated.
        assert_eq!(chip_size(true, true), Vec2::new(4.0, 24.0));
        assert_eq!(chip_size(true, false), Vec2::new(24.0, 4.0));
    }

    #[test]
    fn chip_border_is_foreground_not_border_token() {
        for mode in [Mode::Light, Mode::Dark] {
            let p = Preset::default().build(mode).palette;
            assert_eq!(chip_border_color(&p), p.foreground);
            assert_ne!(chip_border_color(&p), p.border);
        }
    }

    /// `Shape::Rect`s in the output filled with `fill`.
    fn rects_with_fill(out: &egui::FullOutput, fill: Color32) -> Vec<&egui::epaint::RectShape> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.fill == fill => Some(rs),
                _ => None,
            })
            .collect()
    }

    /// `Shape::Rect`s in the output stroked in `color` (non-zero width).
    fn rects_with_stroke(out: &egui::FullOutput, color: Color32) -> Vec<&egui::epaint::RectShape> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.stroke.width > 0.0 && rs.stroke.color == color => {
                    Some(rs)
                }
                _ => None,
            })
            .collect()
    }

    /// All meshes in the output (icon glyphs are meshes).
    fn meshes(out: &egui::FullOutput) -> Vec<&egui::Mesh> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Mesh(mesh) => Some(mesh.as_ref()),
                _ => None,
            })
            .collect()
    }

    /// Renders one horizontal group (length 400, two panels, one handle)
    /// in a fixed 500×500 UI, `frames` times, and returns the last
    /// frame's output plus the handle's (id, interact rect) from that
    /// frame. Two frames settle the learned totals (§9.2).
    fn render_group_frames(
        ctx: &egui::Context,
        frames: usize,
        with_handle: bool,
    ) -> (egui::FullOutput, Id, Rect) {
        let mut out = None;
        let mut handle = None;
        // One stable Ui id for all frames, so group/handle ids (and focus)
        // persist across frames.
        let ui_id = egui::Id::unique("resizable_paint_test_ui");
        for _ in 0..frames {
            let mut frame_out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    ui_id,
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                ResizablePanelGroup::new()
                    .id_salt("paint_test")
                    .length(400.0)
                    .show(ui, |ui| {
                        ResizablePanel::new().min(0.2).show(ui, |ui| {
                            ui.label("Left");
                        });
                        let resp = ResizableHandle::new()
                            .with_handle(with_handle)
                            .id_salt("paint_handle")
                            .show(ui);
                        handle = Some((resp.id, resp.rect));
                        ResizablePanel::new().show(ui, |ui| {
                            ui.label("Right");
                        });
                    });
            });
            frame_out.textures_delta.clear();
            out = Some(frame_out);
        }
        let (id, rect) = handle.expect("handle must render");
        (out.expect("at least one frame"), id, rect)
    }

    #[test]
    fn paints_base_chip_with_foreground_border() {
        let ctx = egui::Context::default();
        let preset = Preset::default().with_style(Style::Vega);
        preset.clone().build(Mode::Light).install(&ctx);
        let p = preset.build(Mode::Light).palette;

        let (out, _, hit_rect) = render_group_frames(&ctx, 2, true);

        // Hit band: 4px centred on the 1px line (§3.2).
        assert!(
            (hit_rect.width() - 4.0).abs() < 1e-4,
            "hit band must be 4px wide, got {}",
            hit_rect.width()
        );

        // Handle line: 1px, `bg-border` fill.
        let border_fills = rects_with_fill(&out, p.border);
        let line = border_fills
            .iter()
            .find(|rs| (rs.rect.width() - 1.0).abs() < 1e-4)
            .expect("1px handle line");
        assert!((hit_rect.center().x - line.rect.center().x).abs() < 1e-4);

        // Base chip: 12×16, `bg-border` fill, 2px radius (§2.4, §3.3).
        let chip = border_fills
            .iter()
            .find(|rs| {
                (rs.rect.width() - 12.0).abs() < 1e-4 && (rs.rect.height() - 16.0).abs() < 1e-4
            })
            .expect("12×16 base chip");
        assert_eq!(chip.corner_radius, CornerRadius::same(2));
        assert!((chip.rect.center().x - line.rect.center().x).abs() < 1e-4);

        // Chip border: 1px stroke in foreground — NOT the `border` token
        // (§0 correction 5).
        let fg_strokes = rects_with_stroke(&out, p.foreground);
        assert_eq!(
            fg_strokes.len(),
            1,
            "expected exactly one foreground stroke (the chip border), got {}",
            fg_strokes.len()
        );
        assert!((fg_strokes[0].stroke.width - 1.0).abs() < 1e-4);
        assert_eq!(fg_strokes[0].rect, chip.rect);
        assert!(
            rects_with_stroke(&out, p.border).is_empty(),
            "no stroke may use the border token"
        );

        // Grip glyph: one 10×10 mesh in `currentColor`, centred on the chip.
        let glyph_meshes = meshes(&out);
        assert_eq!(
            glyph_meshes.len(),
            1,
            "expected exactly one glyph mesh, got {}",
            glyph_meshes.len()
        );
        let glyph_bounds = glyph_meshes[0].calc_bounds();
        assert!(
            (glyph_bounds.width() - 10.0).abs() < 1e-4
                && (glyph_bounds.height() - 10.0).abs() < 1e-4,
            "glyph mesh must be 10×10, got {glyph_bounds:?}"
        );
        assert_eq!(glyph_bounds.center(), chip.rect.center());
        assert!(
            glyph_meshes[0]
                .vertices
                .iter()
                .all(|v| v.color == p.foreground),
            "glyph mesh must be tinted foreground"
        );
    }

    #[test]
    fn paints_nova_pill_borderless_without_glyph() {
        let ctx = egui::Context::default();
        let preset = Preset::default(); // nova
        preset.clone().build(Mode::Light).install(&ctx);
        let p = preset.build(Mode::Light).palette;

        let (out, _, _) = render_group_frames(&ctx, 2, true);

        // Nova pill: 4×24, `bg-border` fill, 2px radius (§2.5, §3.3).
        let border_fills = rects_with_fill(&out, p.border);
        let pill = border_fills
            .iter()
            .find(|rs| {
                (rs.rect.width() - 4.0).abs() < 1e-4 && (rs.rect.height() - 24.0).abs() < 1e-4
            })
            .expect("4×24 nova pill");
        assert_eq!(pill.corner_radius, CornerRadius::same(2));

        // No border on the pill (OPEN-11 frozen value): no stroke in
        // foreground or in the border token, and no glyph (no mesh).
        assert!(
            rects_with_stroke(&out, p.foreground).is_empty(),
            "nova pill must have no border"
        );
        assert!(
            rects_with_stroke(&out, p.border).is_empty(),
            "nova pill must have no border"
        );
        assert!(meshes(&out).is_empty(), "nova pill must have no glyph");
    }

    #[test]
    fn focus_ring_is_1px_ring_with_1px_offset() {
        let ctx = egui::Context::default();
        let preset = Preset::default().with_style(Style::Vega);
        preset.clone().build(Mode::Light).install(&ctx);
        let p = preset.build(Mode::Light).palette;

        // Settle two frames, then focus the handle (no grip chip, so the
        // ring is the only stroke painted).
        let (_, handle_id, _) = render_group_frames(&ctx, 2, false);
        ctx.memory_mut(|mem| mem.request_focus(handle_id));
        let (out, _, _) = render_group_frames(&ctx, 1, false);

        // Handle line: 1px, `bg-border` fill.
        let border_fills = rects_with_fill(&out, p.border);
        let line = border_fills
            .iter()
            .find(|rs| (rs.rect.width() - 1.0).abs() < 1e-4)
            .expect("1px handle line");

        // Ring: 1px, `ring` token, occupying the band 1–2px outside the
        // handle rect (§3.4); no outline is painted.
        let ring_strokes = rects_with_stroke(&out, p.ring);
        assert_eq!(
            ring_strokes.len(),
            1,
            "expected exactly one ring stroke, got {}",
            ring_strokes.len()
        );
        assert!((ring_strokes[0].stroke.width - 1.0).abs() < 1e-4);
        assert_eq!(ring_strokes[0].rect, line.rect.expand(1.0));
    }

    /// RSZ-02: dragging handle 1 past the middle's minimum cascades the
    /// excess into the next panel (S8 §3.1 — the library's cascade, not
    /// the frozen spec's "excess is dropped").
    ///
    /// Replays the S6 R5 sequence on the §3 group (Sidebar min 0.15,
    /// Content max 0.4, Details min 0.2; 440px group, 438px of panels):
    /// h2 +50 (middle to its 0.4 max), h1 +50, then h1 +150 in 12 steps.
    /// The middle can only shrink 125px, so 25px of the last drag is
    /// excess and must cascade into Details: the settled lines must be
    /// l1 = 346.5, l2 = 347.5 (fractions [0.79, 0, 0.21] — gallery
    /// coordinates 519/520, the S8 target).
    ///
    /// Pre-fix this failed at l1 = 321.7, l2 = 336.4: the fractions were
    /// already exact ([0.7333, 0, 0.2667]) but the middle panel rendered
    /// ~14px wide at fraction 0 — `allocate_ui` grows to the content's
    /// `min_rect` — pushing the next handle right. The fix confines each
    /// panel to exactly its fraction width.
    #[test]
    fn drag_cascades_excess_past_middle_min() {
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let ui_id = egui::Id::new("rsz02_drag_test");

        // Renders the §3 group for one pass and returns the two handle
        // line x positions (from their response rect centres, i.e. the
        // 1px line centre — fraction position + 0.5).
        let render = |ctx: &egui::Context, input: egui::RawInput| -> (f32, f32) {
            let mut h1 = f32::NAN;
            let mut h2 = f32::NAN;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    ui_id,
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                ResizablePanelGroup::new()
                    .id_salt("rsz02")
                    .length(440.0)
                    .show(ui, |ui| {
                        ResizablePanel::new().min(0.15).show(ui, |ui| {
                            ui.label("Sidebar");
                        });
                        h1 = ResizableHandle::new().show(ui).rect.center().x;
                        ResizablePanel::new().max(0.4).show(ui, |ui| {
                            ui.label("Content (max 40%)");
                        });
                        h2 = ResizableHandle::new().show(ui).rect.center().x;
                        ResizablePanel::new().min(0.2).show(ui, |ui| {
                            ui.label("Details");
                        });
                    });
            });
            out.textures_delta.clear();
            (h1, h2)
        };

        let moved = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerMoved(pos));
            input
        };
        let press = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            });
            input
        };
        let release = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            });
            input
        };

        // Drags the handle currently at `x0` to `x1` in `steps` equal
        // steps (one frame per step, as the gallery driver does) and
        // settles; returns the settled line positions.
        let mut drag = |ctx: &egui::Context, x0: f32, x1: f32, steps: usize| -> (f32, f32) {
            let y = 10.0;
            let _ = render(ctx, moved(Pos2::new(x0, y)));
            let _ = render(ctx, press(Pos2::new(x0, y)));
            for i in 1..=steps {
                let x = x0 + (x1 - x0) * (i as f32 / steps as f32);
                let _ = render(ctx, moved(Pos2::new(x, y)));
            }
            let _ = render(ctx, release(Pos2::new(x1, y)));
            render(ctx, egui::RawInput::default())
        };

        let close = |got: f32, want: f32, what: &str| {
            assert!(
                (got - want).abs() < 1.0,
                "{what}: got {got:.1}, want {want:.1}"
            );
        };

        // Settle: two frames (learned totals make the split 1/3 each).
        let (l1, l2) = render(&ctx, egui::RawInput::default());
        let (l1, l2) = render(&ctx, egui::RawInput::default());
        close(l1, 146.5, "settled l1");
        close(l2, 293.5, "settled l2");

        // h2 +50: middle grows to its 0.4 max (l2 +29.3); the rest of
        // the drag is dropped, l1 unmoved.
        let (l1, l2) = drag(&ctx, l2, l2 + 50.0, 4);
        close(l1, 146.5, "h2+50 l1");
        close(l2, 322.7, "h2+50 l2");

        // h1 +50: middle shrinks 0.4 -> 0.285; l2 unmoved (no transfer
        // within limits).
        let (l1, l2) = drag(&ctx, l1, l1 + 50.0, 4);
        close(l1, 196.5, "h1+50 l1");
        close(l2, 322.7, "h1+50 l2");

        // h1 +150: middle can only shrink 125px; the 25px excess
        // cascades into Details — l1 = 346.5, l2 = 347.5.
        let (l1, l2) = drag(&ctx, l1, l1 + 150.0, 12);
        close(l1, 346.5, "h1+150 l1 (cascade target)");
        close(l2, 347.5, "h1+150 l2 (cascade target)");
    }

    /// RSZ-03: the non-dragged partner line stays exactly put during a
    /// drag. Before the fix the partner moved by exactly one frame's
    /// drag delta for every frame of the drag (the dragged handle's
    /// `adjust()` mutated the live fractions mid-layout, so panels
    /// after the handle rendered one step ahead of the panels before
    /// it — with 5px steps the partner sat 5px off its settled value
    /// for the whole drag, then settled exactly). The dragged line
    /// still moves and settles exactly; the h2 drags never jittered
    /// the partner either, and still must not.
    #[test]
    fn partner_line_is_stable_during_h1_drag() {
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let ui_id = egui::Id::new("rsz03_partner_test");

        let render = |ctx: &egui::Context, input: egui::RawInput| -> (f32, f32) {
            let mut h1 = f32::NAN;
            let mut h2 = f32::NAN;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    ui_id,
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                ResizablePanelGroup::new()
                    .id_salt("rsz03")
                    .length(440.0)
                    .show(ui, |ui| {
                        ResizablePanel::new().min(0.15).show(ui, |ui| {
                            ui.label("Sidebar");
                        });
                        h1 = ResizableHandle::new().show(ui).rect.center().x;
                        ResizablePanel::new().max(0.4).show(ui, |ui| {
                            ui.label("Content (max 40%)");
                        });
                        h2 = ResizableHandle::new().show(ui).rect.center().x;
                        ResizablePanel::new().min(0.2).show(ui, |ui| {
                            ui.label("Details");
                        });
                    });
            });
            out.textures_delta.clear();
            (h1, h2)
        };

        let moved = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerMoved(pos));
            input
        };
        let press = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            });
            input
        };
        let release = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            });
            input
        };

        // Drags the handle currently at `x0` to `x1` in 10px steps (one
        // frame per step) and asserts the OTHER line stays at its
        // settled value in every frame of the drag. Returns the
        // settled line positions. Steps are 10px so the drag is
        // decided on step 1: egui's click-vs-drag decision waits until
        // the pointer moves past `max_click_dist` (6px), so a smaller
        // first step would drop its delta (that loss is RSZ-04's
        // domain, not this test's).
        let mut drag_check_partner = |ctx: &egui::Context,
                                      x0: f32,
                                      x1: f32,
                                      other_settled: f32,
                                      which: &str|
         -> (f32, f32) {
            let y = 10.0;
            let steps = ((x1 - x0).abs() / 10.0).round() as usize;
            let (mut a, mut b) = render(ctx, moved(Pos2::new(x0, y)));
            assert!(
                ((if which == "h1" { b } else { a }) - other_settled).abs() < 0.01,
                "press frame: partner moved before the drag"
            );
            let _ = render(ctx, press(Pos2::new(x0, y)));
            for i in 1..=steps {
                let x = x0 + (x1 - x0) * (i as f32 / steps as f32);
                (a, b) = render(ctx, moved(Pos2::new(x, y)));
                let partner = if which == "h1" { b } else { a };
                assert!(
                    (partner - other_settled).abs() < 0.01,
                    "{which} drag step {i}: partner moved to {partner:.4}, settled {other_settled:.4}"
                );
            }
            let (a, b) = render(ctx, release(Pos2::new(x1, y)));
            let (a, b) = render(ctx, egui::RawInput::default());
            (a, b)
        };

        let (l1, l2) = render(&ctx, egui::RawInput::default());
        let (l1, l2) = render(&ctx, egui::RawInput::default());
        assert!((l1 - 146.5).abs() < 1.0, "settled l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "settled l2: got {l2:.1}");

        // h1 +50: the dragged line moves, the partner (l2) never moves.
        let (l1, l2) = drag_check_partner(&ctx, l1, l1 + 50.0, l2, "h1");
        assert!((l1 - 196.5).abs() < 1.0, "h1+50 l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "h1+50 l2: got {l2:.1}");

        // h1 -50 restoring drag: same, in the other direction.
        let (l1, l2) = drag_check_partner(&ctx, l1, l1 - 50.0, l2, "h1");
        assert!((l1 - 146.5).abs() < 1.0, "h1-50 l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "h1-50 l2: got {l2:.1}");

        // h2 +20 (within the middle's 0.4 max): the partner is l1 this
        // time; it must not move either.
        let (l1, l2) = drag_check_partner(&ctx, l2, l2 + 20.0, l1, "h2");
        assert!((l1 - 146.5).abs() < 1.0, "h2+20 l1: got {l1:.1}");
        assert!((l2 - 313.5).abs() < 1.0, "h2+20 l2: got {l2:.1}");
    }

    #[test]
    fn restoring_drag_converges_to_pointer() {
        // RSZ-04: restoring drags end 2-5px short of the pointer because
        // per-frame `drag_delta()` drops the movement from frames in
        // which `dragged()` is false (before egui's click-vs-drag
        // decision). Replays the exact S7 R1 sequence (§3 group, 5px
        // steps — the step size at which the RSZ-03 diagnostic saw the
        // restoring drag settle 5px short, 151.5 instead of 146.5) and
        // asserts every settled endpoint is exact, the dragged line
        // moves monotonically toward the pointer in every frame, and
        // the partner never moves.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let ui_id = egui::Id::new("rsz04_converge_test");

        let render = |ctx: &egui::Context, input: egui::RawInput| -> (f32, f32) {
            let mut h1 = f32::NAN;
            let mut h2 = f32::NAN;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    ui_id,
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                ResizablePanelGroup::new()
                    .id_salt("rsz04")
                    .length(440.0)
                    .show(ui, |ui| {
                        ResizablePanel::new().min(0.15).show(ui, |ui| {
                            ui.label("Sidebar");
                        });
                        h1 = ResizableHandle::new().show(ui).rect.center().x;
                        ResizablePanel::new().max(0.4).show(ui, |ui| {
                            ui.label("Content (max 40%)");
                        });
                        h2 = ResizableHandle::new().show(ui).rect.center().x;
                        ResizablePanel::new().min(0.2).show(ui, |ui| {
                            ui.label("Details");
                        });
                    });
            });
            out.textures_delta.clear();
            (h1, h2)
        };

        let moved = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerMoved(pos));
            input
        };
        let press = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            });
            input
        };
        let release = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            });
            input
        };
        // One frame that both moves the pointer and releases the
        // button: the release-frame convergence path (origin +
        // release position) must apply the final step, since
        // `total_drag_delta()` is unavailable once the button is up.
        let move_and_release = |pos: Pos2| {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerMoved(pos));
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            });
            input
        };

        // Drags the handle currently at `x0` to `x1` in 5px steps (one
        // frame per step) and asserts the dragged line moves
        // monotonically toward the pointer, stays in band, and settles
        // exactly at `x1`, while the partner never moves. Returns the
        // settled line positions. 5px steps are deliberate: a 5px
        // first step is below egui's 6px `max_click_dist`, so the
        // click-vs-drag decision has not been made yet and that
        // step's delta is the one RSZ-04 loses.
        let mut drag_check_converge = |ctx: &egui::Context,
                                       x0: f32,
                                       x1: f32,
                                       other_settled: f32,
                                       which: &str|
         -> (f32, f32) {
            let y = 10.0;
            let steps = ((x1 - x0).abs() / 5.0).round() as usize;
            let (mut a, mut b) = render(ctx, moved(Pos2::new(x0, y)));
            assert!(
                ((if which == "h1" { b } else { a }) - other_settled).abs() < 0.01,
                "press frame: partner moved before the drag"
            );
            let _ = render(ctx, press(Pos2::new(x0, y)));
            let lo = x0.min(x1);
            let hi = x0.max(x1);
            let mut prev = x0;
            for i in 1..=steps {
                let x = x0 + (x1 - x0) * (i as f32 / steps as f32);
                (a, b) = render(ctx, moved(Pos2::new(x, y)));
                let (dragged, partner) = if which == "h1" { (a, b) } else { (b, a) };
                assert!(
                    (partner - other_settled).abs() < 0.01,
                    "{which} drag step {i}: partner moved to {partner:.4}, settled {other_settled:.4}"
                );
                assert!(
                    dragged >= lo - 0.01 && dragged <= hi + 0.01,
                    "{which} drag step {i}: dragged line {dragged:.4} outside [{lo:.1}, {hi:.1}]"
                );
                if x1 > x0 {
                    assert!(
                        dragged >= prev - 0.01,
                        "{which} drag step {i}: dragged line moved back {prev:.4} -> {dragged:.4}"
                    );
                } else {
                    assert!(
                        dragged <= prev + 0.01,
                        "{which} drag step {i}: dragged line moved back {prev:.4} -> {dragged:.4}"
                    );
                }
                prev = dragged;
            }
            let (a, b) = render(ctx, release(Pos2::new(x1, y)));
            let (a, b) = render(ctx, egui::RawInput::default());
            assert!(
                ((if which == "h1" { a } else { b }) - x1).abs() < 0.01,
                "{which} drag settled at {:.4}, expected {x1:.1}",
                if which == "h1" { a } else { b }
            );
            (a, b)
        };

        let (l1, l2) = render(&ctx, egui::RawInput::default());
        let (l1, l2) = render(&ctx, egui::RawInput::default());
        assert!((l1 - 146.5).abs() < 1.0, "settled l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "settled l2: got {l2:.1}");

        // h1 +50, then the restoring drag back: the S7 shortfall.
        let (l1, l2) = drag_check_converge(&ctx, l1, l1 + 50.0, l2, "h1");
        assert!((l1 - 196.5).abs() < 1.0, "h1+50 l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "h1+50 l2: got {l2:.1}");
        let (l1, l2) = drag_check_converge(&ctx, l1, l1 - 50.0, l2, "h1");
        assert!((l1 - 146.5).abs() < 1.0, "h1-50 l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "h1-50 l2: got {l2:.1}");

        // h2 +25, then the restoring drag back.
        let (l1, l2) = drag_check_converge(&ctx, l2, l2 + 25.0, l1, "h2");
        assert!((l1 - 146.5).abs() < 1.0, "h2+25 l1: got {l1:.1}");
        assert!((l2 - 318.5).abs() < 1.0, "h2+25 l2: got {l2:.1}");
        let (l1, l2) = drag_check_converge(&ctx, l2, l2 - 25.0, l1, "h2");
        assert!((l1 - 146.5).abs() < 1.0, "h2-25 l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "h2-25 l2: got {l2:.1}");

        // Move-and-release in one frame: h1 from 146.5 to 196.5 in
        // 5px steps, but the final step and the release share one
        // frame, so the release-frame path must apply it.
        let y = 10.0;
        let _ = render(&ctx, moved(Pos2::new(l1, y)));
        let _ = render(&ctx, press(Pos2::new(l1, y)));
        for i in 1..=9 {
            let x = l1 + 5.0 * i as f32;
            let _ = render(&ctx, moved(Pos2::new(x, y)));
        }
        let (l1, l2) = render(&ctx, move_and_release(Pos2::new(l1 + 50.0, y)));
        let (l1, l2) = render(&ctx, egui::RawInput::default());
        assert!((l1 - 196.5).abs() < 1.0, "move+release l1: got {l1:.1}");
        assert!((l2 - 293.5).abs() < 1.0, "move+release l2: got {l2:.1}");
    }

    /// RSZ-05: a horizontal panel's height comes from its content,
    /// while its width stays exactly its fraction. RSZ-02's
    /// `allocate_exact_size` froze every horizontal panel at the
    /// row's seeded 36px height, clipping §1's labels and "Add item"
    /// button invisible. The pin: the left panel's height equals its
    /// content height (the "Add item" button's bottom), the button
    /// sits inside the panel rect, and the handle spans the full
    /// group height — while the width stays exactly the fraction
    /// (219.5px at the equal split of 440, RSZ-02's requirement).
    /// Single-label panels take their label's height (18px); the row
    /// itself stays 36px because the handles allocate the seeded row
    /// height — the pre-RSZ-02 behaviour, verified against the old
    /// `allocate_ui` arm headlessly (§1 135px, §3 panels 18px,
    /// handles 36px).
    #[test]
    fn horizontal_panel_height_comes_from_content() {
        use crate::components::button::{Button, ButtonSize};
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let ui_id = egui::Id::new("rsz05_height_test");
        let mut left = Rect::NOTHING;
        let mut right = Rect::NOTHING;
        let mut h = Rect::NOTHING;
        let mut btn = Rect::NOTHING;
        let mut s3 = [Rect::NOTHING; 3];
        let mut s3h = [Rect::NOTHING; 2];
        let mut s3_label = Rect::NOTHING;
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    ui_id,
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                ResizablePanelGroup::new()
                    .id_salt("rsz05_h")
                    .length(440.0)
                    .show(ui, |ui| {
                        let r = ResizablePanel::new().min(0.2).max(0.7).show(ui, |ui| {
                            ui.heading("Left panel");
                            ui.label("Item 1");
                            ui.label("Item 2");
                            ui.label("Item 3");
                            btn = Button::new("Add item").size(ButtonSize::Sm).show(ui).rect;
                        });
                        left = r.response.rect;
                        h = ResizableHandle::new().with_handle(true).show(ui).rect;
                        let r2 = ResizablePanel::new().show(ui, |ui| {
                            ui.heading("Right panel");
                            ui.label("Drag the divider between us.");
                            ui.small("Panel sizes persist across frames (per group id).");
                        });
                        right = r2.response.rect;
                    });
                ui.add_space(12.0);
                ResizablePanelGroup::new()
                    .id_salt("rsz05_3")
                    .length(440.0)
                    .show(ui, |ui| {
                        for (i, text) in ["Sidebar", "Content (max 40%)", "Details"]
                            .iter()
                            .enumerate()
                        {
                            let r = ResizablePanel::new().show(ui, |ui| {
                                let lr = ui.label(*text).rect;
                                if i == 0 {
                                    s3_label = lr;
                                }
                            });
                            s3[i] = r.response.rect;
                            if i < 2 {
                                s3h[i] = ResizableHandle::new().show(ui).rect;
                            }
                        }
                    });
            });
            out.textures_delta.clear();
        }

        // §1: the panel is exactly as tall as its content (the button
        // is the last widget), not the 36px row seed.
        let content_h = btn.bottom() - left.top();
        assert!(
            content_h > 100.0,
            "§1 content must be taller than the 36px seed, got {content_h:.1}"
        );
        assert!(
            (left.height() - content_h).abs() < 1.0,
            "left panel height must equal its content height: got {:.1}, want {:.1}",
            left.height(),
            content_h
        );
        assert!(
            left.contains_rect(btn),
            "the Add item button must sit inside the left panel rect: panel {left:?}, button {btn:?}"
        );
        assert!(
            (left.width() - 219.5).abs() < 1.0,
            "left panel width must stay exactly its fraction: got {:.1}, want 219.5",
            left.width()
        );
        assert!(
            (h.height() - left.height()).abs() < 1.0,
            "handle must span the full group height: got {:.1}, want {:.1}",
            h.height(),
            left.height()
        );
        // The right panel is shorter than the left (its content ends
        // higher) but shares the group's top edge.
        assert!(
            right.top() == left.top() && right.bottom() < left.bottom(),
            "right panel must start at the group top and end above the left panel's bottom: {right:?} vs {left:?}"
        );

        // §3: single-label panels take their label's height, and the
        // row stays 36px because the handles allocate the seeded row
        // height.
        let label_h = s3_label.bottom() - s3[0].top();
        for (i, p) in s3.iter().enumerate() {
            // Tolerance 1.5px: `allocate_rect` rounds the panel rect to
            // whole pixels (`round_ui`), so the panel can sit up to 1px
            // above the label's exact height.
            assert!(
                (p.height() - label_h).abs() <= 1.5,
                "§3 panel {i} height must equal its label height: got {:.1}, want {:.1}",
                p.height(),
                label_h
            );
        }
        for (i, handle) in s3h.iter().enumerate() {
            assert!(
                (handle.height() - 36.0).abs() < 1.0,
                "§3 handle {i} must span the seeded 36px row height: got {:.1}",
                handle.height()
            );
        }
    }
}
