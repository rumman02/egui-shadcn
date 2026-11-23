//! shadcn `resizable.tsx` — draggable panel groups.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/resizable.tsx` (the
//! `react-resizable-panels` wrapper): a [`ResizablePanelGroup`] lays out
//! [`ResizablePanel`] children separated by [`ResizableHandle`]s. Handles
//! are `bg-border` 1px lines with an optional `with_handle` grip chip
//! (base: `h-4 w-3 rounded-xs border bg-border` + grip glyph; nova: a plain
//! `h-6 w-1 rounded-lg bg-border` pill); they support pointer dragging and
//! Arrow-key resizing when focused, with per-panel `min`/`max` fraction
//! clamps. Fractions persist in the egui context keyed by the group's id
//! salt.
//!
//! Scope notes: keyboard resize steps by 5% of the group length (an
//! approximation of `react-resizable-panels`' keyboard increments —
//! OPEN-2, not spec truth); sizes are fractional — the source's pixel
//! `defaultSize`/`minSize` and collapsible panels are out of scope
//! (D4). The base grip chip's border is `currentColor` (foreground),
//! per the frozen spec §0 correction 5; the nova pill is borderless
//! (OPEN-11 frozen value).
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
    Color32, CornerRadius, CursorIcon, Id, InnerResponse, Rect, Response, Sense, Stroke, Ui, Vec2,
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

/// Adjusts the split between panels `idx` and `idx + 1` by a fractional
/// delta `d`, clamped to both panels' limits. Returns the applied delta.
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
    let left = st.fractions[idx];
    let right = st.fractions[idx + 1];
    let (lmin, lmax) = get(&st, idx);
    let (rmin, rmax) = get(&st, idx + 1);
    let d_min = (lmin - left).max(right - rmax);
    let d_max = (lmax - left).min(right - rmin);
    let d = d.clamp(d_min.min(d_max), d_max.max(d_min));
    st.fractions[idx] = left + d;
    st.fractions[idx + 1] = right - d;
    write_state(&ctx, id, st);
    d
}

/// A shadcn resizable panel group. See the [module docs](self) for an example.
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
        let frac = st.fraction(idx) * st.panels_px();

        let inner = match group.orientation {
            ResizableOrientation::Horizontal => {
                ui.allocate_ui(Vec2::new(frac, ui.available_height()), |ui| {
                    ui.set_min_width(frac);
                    ui.set_max_width(frac);
                    ui.vertical(content).inner
                })
            }
            ResizableOrientation::Vertical => {
                ui.allocate_ui(Vec2::new(ui.available_width(), frac), |ui| {
                    ui.set_min_height(frac);
                    ui.set_max_height(frac);
                    ui.vertical(content).inner
                })
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

        // Pointer drag adjusts the split.
        let dragged_px = if response.dragged() {
            let delta = response.drag_delta();
            if horizontal_group {
                delta.x
            } else {
                delta.y
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

        let delta_px = dragged_px + key_px;
        if delta_px != 0.0 && st.total_panels >= 2 {
            let d = delta_px / st.panels_px().max(1.0);
            let applied = adjust(ui, group.id, idx, d) * st.panels_px();
            if applied.abs() > 0.01 {
                ui.ctx().request_repaint();
            }
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
}
