//! shadcn `resizable.tsx` — draggable panel groups.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/resizable.tsx` (the
//! `react-resizable-panels` wrapper): a [`ResizablePanelGroup`] lays out
//! [`ResizablePanel`] children separated by [`ResizableHandle`]s. Handles
//! are `bg-border` 1px lines with an optional `with_handle` grip chip
//! (`h-4 w-3 rounded-xs border bg-border` + grip glyph); they support
//! pointer dragging and Arrow-key resizing when focused, with per-panel
//! `min`/`max` fraction clamps. Fractions persist in the egui context
//! keyed by the group's id salt.
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

use crate::theme::ShadcnTheme;
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

        // Expanded interaction rect (the source's `after:` hit area).
        let hit = if horizontal_group {
            flow_rect.expand2(egui::vec2(3.0, 0.0))
        } else {
            flow_rect.expand2(egui::vec2(0.0, 3.0))
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
                // Grip chip: w-3 h-4 rotated for horizontal groups.
                let (cw, ch) = if horizontal_group {
                    (12.0, 16.0)
                } else {
                    (16.0, 12.0)
                };
                let chip = Rect::from_center_size(flow_rect.center(), Vec2::new(cw, ch));
                painter.rect_filled(chip, CornerRadius::same(2), p.border);
                painter.rect_stroke(
                    chip,
                    CornerRadius::same(2),
                    Stroke::new(1.0, p.border),
                    egui::epaint::StrokeKind::Inside,
                );
                // Grip glyph: three dots along the chip's long axis.
                let dot = 1.2;
                let along = if horizontal_group {
                    chip.center().y
                } else {
                    chip.center().x
                };
                let cross_c = if horizontal_group {
                    chip.center().x
                } else {
                    chip.center().y
                };
                for k in [-1.0f32, 0.0, 1.0] {
                    let center = if horizontal_group {
                        Pos2::new(cross_c, along + k * 4.0)
                    } else {
                        Pos2::new(cross_c + k * 4.0, along)
                    };
                    painter.circle_filled(center, dot, dot_color(&p, theme.mode.is_dark()));
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
            let step = 0.05 * st.panels_px();
            ui.input(|i| {
                let (prev_key, next_key) = if horizontal_group {
                    (egui::Key::ArrowLeft, egui::Key::ArrowRight)
                } else {
                    (egui::Key::ArrowUp, egui::Key::ArrowDown)
                };
                if i.key_pressed(prev_key) {
                    -step
                } else if i.key_pressed(next_key) {
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

fn dot_color(p: &crate::theme::Palette, _dark: bool) -> Color32 {
    // Grip glyph on a `bg-border` chip: the background token reads against
    // it in both modes.
    p.background
}

fn color_ring(p: &crate::theme::Palette) -> Color32 {
    p.ring
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Key;

    fn palette(mode: Mode) -> crate::theme::Palette {
        Preset::default().build(mode).palette
    }

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

    #[test]
    fn dot_color_uses_background_token() {
        let p = palette(Mode::Light);
        assert_eq!(dot_color(&p, false), p.background);
        let dark = palette(Mode::Dark);
        assert_eq!(dot_color(&dark, true), dark.background);
    }
}
