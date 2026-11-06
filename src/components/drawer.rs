//! shadcn `drawer.tsx` — a bottom sheet with drag-to-dismiss (vaul-style).
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/drawer.tsx` (vaul) with the
//! nova recipe: `bg-popover text-popover-foreground rounded-t-xl border-t`,
//! the `bg-muted h-1 w-[100px] rounded-full` drag handle at `mt-4`, and a
//! `bg-black/10` animated backdrop.
//!
//! Behavior: the panel can be dragged down with the pointer; releasing past
//! 25% of the panel height (or 80 px) dismisses it, otherwise it snaps back —
//! vaul's default dismissible drawer. Backdrop clicks and Escape also
//! dismiss. Content fades and slides like the source.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Drawer::new().id_salt("cart").show(ui, |ui| {
//!     DrawerTrigger::widget(|ui| Button::new("Open cart").show(ui)).show(ui);
//!     DrawerContent::new().show(ui, |ui| {
//!         DrawerHeader::new().show(ui, |ui| {
//!             DrawerTitle::new("Cart").show(ui);
//!             DrawerDescription::new("2 items").show(ui);
//!         });
//!         DrawerFooter::new().show(ui, |ui| {
//!             DrawerClose::widget(|ui| Button::new("Checkout").show(ui)).show(ui);
//!         });
//!     });
//! });
//! ```

use crate::components::button::{Button, ButtonVariant};
use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::shadow;
use egui::{
    Align2, Color32, Context, CornerRadius, Frame, Id, Modal, Pos2, Rect, Response, Sense, Ui, Vec2,
};

/// The vaul default dismiss threshold: 25% of the panel height (with a
/// 80 px floor so tiny panels stay usable).
const DISMISS_THRESHOLD_FRACTION: f32 = 0.25;
const DISMISS_THRESHOLD_MIN: f32 = 80.0;
/// Headroom the panel rests above the bottom edge (`mt-24` in the source
/// keeps the top 96 px free; we mirror it as the resting offset budget).
const HEADROOM: f32 = 24.0;
/// nova backdrop: `bg-black/10`.
const BACKDROP_ALPHA: f32 = 0.10;
/// Slide duration (vaul's springy ~300ms).
const SLIDE_SECS: f32 = 0.3;
/// Header/footer padding: `p-4`.
const PAD: f32 = 16.0;

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors dialog.rs)
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_drawer_stack")
}

fn push_scope(ui: &Ui, state_id: Id) {
    let ctx = ui.ctx().clone();
    let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
    stack.push(state_id);
    ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
}

fn pop_scope(ui: &Ui) {
    let ctx = ui.ctx().clone();
    let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
    stack.pop();
    ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
}

fn current_scope(ui: &Ui) -> Option<Id> {
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| {
        let stack: Vec<Id> = d.get_temp(stack_id())?;
        stack.last().copied()
    })
}

/// Per-instance state: open flag plus the live drag.
#[derive(Clone, Copy, Debug, Default)]
struct DrawerState {
    open: bool,
    dragging: bool,
    /// Pixels the panel was dragged down this gesture.
    drag_offset: f32,
    /// Pointer y when the gesture started; the offset is measured from it so
    /// pointer jumps between widgets never pollute the drag.
    grab_y: f32,
}

fn read_state(ctx: &Context, state_id: Id) -> DrawerState {
    ctx.data_mut(|d| d.get_temp::<DrawerState>(state_id).unwrap_or_default())
}

fn write_state(ctx: &Context, state_id: Id, state: DrawerState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

fn set_open(ctx: &Context, state_id: Id, open: bool) {
    let mut state = read_state(ctx, state_id);
    state.open = open;
    if !open {
        state.dragging = false;
        state.drag_offset = 0.0;
    }
    write_state(ctx, state_id, state);
}

// ---------------------------------------------------------------------------
// Drawer (root)
// ---------------------------------------------------------------------------

/// The shadcn drawer root. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Drawer<'a> {
    controlled: Option<&'a mut bool>,
    id_salt: Option<String>,
}

impl Default for Drawer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Drawer<'a> {
    /// Creates an uncontrolled drawer (starts closed).
    pub fn new() -> Self {
        Self {
            controlled: None,
            id_salt: None,
        }
    }

    /// Creates a drawer whose open state is owned by the caller. Trigger,
    /// close button, drag-dismiss, backdrop clicks and Escape all write back.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            id_salt: None,
        }
    }

    /// Sets a stable id salt so the open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Lays out the trigger and content, showing the drawer while open.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_drawer"),
        };

        let ctx = ui.ctx().clone();
        if let Some(open) = self.controlled.as_deref() {
            let mut state = read_state(&ctx, state_id);
            state.open = *open;
            write_state(&ctx, state_id, state);
        }

        push_scope(ui, state_id);
        let inner = content(ui);
        pop_scope(ui);

        if let Some(out) = self.controlled {
            *out = read_state(&ctx, state_id).open;
        }
        inner
    }
}

// ---------------------------------------------------------------------------
// DrawerTrigger / DrawerClose
// ---------------------------------------------------------------------------

/// Opens the innermost [`Drawer`] when clicked. Renders an outline button by
/// default or a custom widget via [`DrawerTrigger::widget`].
pub struct DrawerTrigger<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for DrawerTrigger<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> DrawerTrigger<'a> {
    /// Creates a trigger rendering an outline button.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            widget: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Creates a trigger rendering a custom widget (Radix `asChild`).
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w)),
            disabled: false,
            id_salt: None,
        }
    }

    /// Disables the trigger.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the trigger; `changed()` when the open state flipped.
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(state_id) = current_scope(ui) else {
            return outline_button(&self.label, self.disabled, self.id_salt.clone()).show(ui);
        };

        let mut response = match self.widget {
            Some(w) => w(ui),
            None => outline_button(&self.label, self.disabled, self.id_salt.clone()).show(ui),
        };

        if response.clicked() && !self.disabled {
            let ctx = ui.ctx().clone();
            let mut state = read_state(&ctx, state_id);
            state.open = !state.open;
            write_state(&ctx, state_id, state);
            response.mark_changed();
        }
        response
    }
}

/// Closes the innermost [`Drawer`] when clicked. Renders an outline button by
/// default or a custom widget via [`DrawerClose::widget`].
pub struct DrawerClose<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    id_salt: Option<String>,
}

impl Default for DrawerClose<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> DrawerClose<'a> {
    /// Creates a close control rendering an outline button.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            widget: None,
            id_salt: None,
        }
    }

    /// Creates a close control rendering a custom widget.
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w)),
            id_salt: None,
        }
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the close control; clicking dismisses the drawer.
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(state_id) = current_scope(ui) else {
            return outline_button(&self.label, false, self.id_salt.clone()).show(ui);
        };

        let mut response = match self.widget {
            Some(w) => w(ui),
            None => outline_button(&self.label, false, self.id_salt.clone()).show(ui),
        };

        if response.clicked() {
            let ctx = ui.ctx().clone();
            set_open(&ctx, state_id, false);
            response.mark_changed();
        }
        response
    }
}

fn outline_button(label: &Option<String>, disabled: bool, id_salt: Option<String>) -> Button {
    let mut b = Button::new(label.clone().unwrap_or_default())
        .variant(ButtonVariant::Outline)
        .disabled(disabled);
    if let Some(salt) = id_salt {
        b = b.id_salt(salt);
    }
    b
}

// ---------------------------------------------------------------------------
// DrawerContent
// ---------------------------------------------------------------------------

/// The bottom panel: `bg-popover rounded-t-xl border-t` with the drag handle,
/// over a dimmed animated backdrop. Draggable to dismiss.
#[derive(Clone, Copy, Debug)]
pub struct DrawerContent {
    show_handle: bool,
}

impl Default for DrawerContent {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawerContent {
    /// Creates drawer content (the vaul default bottom drawer with handle).
    pub fn new() -> Self {
        Self { show_handle: true }
    }

    /// Hides the drag handle (vaul `handle` prop).
    pub fn show_handle(mut self, show: bool) -> Self {
        self.show_handle = show;
        self
    }

    /// Renders the drawer, or nothing while closed. Returns the modal area
    /// response while visible.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Option<Response> {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Drawer: render statically so mistakes stay visible.
            content(ui);
            return None;
        };
        let ctx = ui.ctx().clone();
        let state = read_state(&ctx, state_id);
        let open = state.open;

        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let modal_id = state_id.with("modal");
        let offset_anim_id = state_id.with("offset");
        let height_key = state_id.with("panel_h");
        let panel_h: f32 = ctx
            .data_mut(|d| d.get_temp::<f32>(height_key).unwrap_or(0.0))
            .max(1.0);
        let rest_offset = panel_h + HEADROOM;

        // The slide offset: direct while dragging, animated otherwise. With
        // duration 0 the animated value jumps to the live drag position so
        // the release continues smoothly from wherever the gesture ended.
        let target_offset = if state.dragging {
            state.drag_offset
        } else if open {
            0.0
        } else {
            rest_offset
        };
        let duration = if state.dragging { 0.0 } else { SLIDE_SECS };
        let offset = ctx.animate_value_with_time(offset_anim_id, target_offset, duration);

        let visible = open || state.dragging || offset < rest_offset - 0.5;
        if !visible {
            return None;
        }

        // Backdrop dims with the slide.
        let alpha = (1.0 - offset / rest_offset).clamp(0.0, 1.0);
        let backdrop = color_alpha(Color32::from_black_alpha(255), BACKDROP_ALPHA * alpha);

        let frame = Frame::new()
            .fill(p.popover)
            .corner_radius(CornerRadius {
                nw: theme.radius.xl_u8(),
                ne: theme.radius.xl_u8(),
                sw: 0,
                se: 0,
            })
            .shadow(shadow::shadow_lg(dark));

        let area =
            Modal::default_area(modal_id).anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, offset));
        let modal = Modal::new(modal_id)
            .area(area)
            .backdrop_color(backdrop)
            .frame(Frame::new());

        let drag_id = state_id.with("drag");
        let handle_id = state_id.with("handle");

        let modal_resp = modal.show(&ctx, |ui| {
            ui.set_opacity(alpha);
            ui.style_mut().visuals.override_text_color = Some(p.popover_foreground);

            let screen_w = ctx.content_rect().width();
            ui.set_min_width(screen_w);
            ui.set_max_width(screen_w);

            // Drag layer under the content: catches drags on empty areas;
            // interactive widgets on top keep their clicks.
            let drag_region = ui.max_rect();
            let drag = ui.interact(drag_region, drag_id, Sense::drag());
            let handle = ui.interact(
                Rect::from_min_size(
                    Pos2::new(drag_region.center().x - 60.0, drag_region.top()),
                    Vec2::new(120.0, 20.0),
                ),
                handle_id,
                Sense::drag(),
            );

            frame.show(ui, |ui| {
                ui.set_min_width(screen_w);
                ui.set_max_width(screen_w);
                if self.show_handle {
                    // `bg-muted h-1 w-[100px] rounded-full mx-auto mt-4`.
                    let (_, strip) = ui.allocate_space(Vec2::new(ui.available_width(), 16.0 + 4.0));
                    let bar = Rect::from_min_size(
                        Pos2::new(strip.center().x - 50.0, strip.top() + 16.0),
                        Vec2::new(100.0, 4.0),
                    );
                    ui.painter()
                        .rect_filled(bar, CornerRadius::same(2), p.muted);
                }
                content(ui);
            });

            // Drag bookkeeping (after layout so the interact regions exist).
            // The offset is measured from the grab point (not accumulated
            // deltas) so pointer jumps between widgets never pollute it.
            let mut state = read_state(&ctx, state_id);
            let gesture_y = drag
                .interact_pointer_pos()
                .or_else(|| handle.interact_pointer_pos())
                .map(|p| p.y);
            let started = drag.drag_started() || handle.drag_started();
            let stopped = drag.drag_stopped() || handle.drag_stopped();
            if started {
                state.dragging = true;
                state.grab_y = gesture_y.unwrap_or(state.grab_y);
                state.drag_offset = 0.0;
            }
            if let Some(y) = gesture_y.filter(|_| state.dragging) {
                state.drag_offset = (y - state.grab_y).max(0.0);
            }
            if stopped && state.dragging {
                state.dragging = false;
                let threshold = (panel_h * DISMISS_THRESHOLD_FRACTION).max(DISMISS_THRESHOLD_MIN);
                if state.drag_offset > threshold {
                    state.open = false;
                }
                state.drag_offset = 0.0;
            }
            write_state(&ctx, state_id, state);
        });

        // Cache the measured panel height for the slide math.
        ctx.data_mut(|d| d.insert_temp(height_key, modal_resp.response.rect.height()));

        // vaul drawers dismiss on backdrop clicks and Escape too.
        if modal_resp.should_close() && open {
            set_open(&ctx, state_id, false);
        }

        Some(modal_resp.response)
    }
}

// ---------------------------------------------------------------------------
// DrawerHeader / DrawerTitle / DrawerDescription / DrawerFooter
// ---------------------------------------------------------------------------

/// nova drawer header: `gap-0.5 p-4`, centered for the bottom drawer.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawerHeader {
    _private: (),
}

impl DrawerHeader {
    /// Creates a drawer header.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the header (centered, like the source's bottom drawer).
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        let flow_min = ui.cursor().min;
        let flow_w = ui.available_width();
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
        let mut measure = ui.new_child(egui::UiBuilder::new().max_rect(measure_rect).invisible());
        let _ = content(&mut measure);
        let content_h = measure.min_rect().height();

        let inner = Rect::from_min_size(
            Pos2::new(flow_min.x + PAD, flow_min.y + PAD),
            Vec2::new(flow_w - PAD * 2.0, content_h),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::HALF;
        let real = child
            .with_layout(egui::Layout::top_down(egui::Align::Center), content)
            .inner;
        ui.allocate_space(Vec2::new(flow_w, content_h + PAD * 2.0));
        real
    }
}

/// nova drawer title: `text-base font-medium`.
#[derive(Clone, Debug)]
pub struct DrawerTitle {
    text: String,
}

impl DrawerTitle {
    /// Creates a drawer title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let font = egui::FontId::proportional(crate::theme::tokens::type_scale::BASE);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text, font, p.popover_foreground);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.popover_foreground);
        }
        response
    }
}

/// `text-sm text-muted-foreground` drawer description, wrapping.
#[derive(Clone, Debug)]
pub struct DrawerDescription {
    text: String,
}

impl DrawerDescription {
    /// Creates a drawer description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description (centered under the bottom-drawer title).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let font = egui::FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout(self.text, font, p.muted_foreground, ui.available_width());
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            let center = Rect::from_center_size(
                Pos2::new(ui.max_rect().center().x, rect.center().y),
                galley.size(),
            );
            ui.painter().galley(center.min, galley, p.muted_foreground);
        }
        response
    }
}

/// nova drawer footer: `gap-2 p-4` (flows after the content).
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawerFooter {
    _private: (),
}

impl DrawerFooter {
    /// Creates a drawer footer.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the footer, centered (bottom drawer).
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        let flow_min = ui.cursor().min;
        let flow_w = ui.available_width();
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
        let mut measure = ui.new_child(egui::UiBuilder::new().max_rect(measure_rect).invisible());
        let _ = content(&mut measure);
        let content_h = measure.min_rect().height();

        let inner = Rect::from_min_size(
            Pos2::new(flow_min.x + PAD, flow_min.y + PAD),
            Vec2::new(flow_w - PAD * 2.0, content_h),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        let real = child
            .with_layout(egui::Layout::top_down(egui::Align::Center), content)
            .inner;
        ui.allocate_space(Vec2::new(flow_w, content_h + PAD * 2.0));
        real
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, UiBuilder};

    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    fn screen() -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn raw_input(&self, events: Vec<egui::Event>) -> egui::RawInput {
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(screen()),
                events,
                ..Default::default()
            }
        }

        fn run(&mut self, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = self.raw_input(vec![]);
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("drawer_test_ui"),
                    UiBuilder::new().max_rect(screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn click(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.time += 0.1;
                let input = self.raw_input(vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                }]);
                let mut out = self.ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("drawer_test_ui"),
                        UiBuilder::new().max_rect(screen()),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        /// Drags from `from` to `to` across press, move, move, release frames.
        fn drag(&mut self, from: Pos2, to: Pos2, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let press = self.raw_input(vec![egui::Event::PointerButton {
                pos: from,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }]);
            let mut out = self.ctx.run_ui(press, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("drawer_test_ui"),
                    UiBuilder::new().max_rect(screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();

            for pos in [
                from,
                Pos2::new((from.x + to.x) * 0.5, (from.y + to.y) * 0.5),
            ] {
                self.time += 0.1;
                let input = self.raw_input(vec![egui::Event::PointerMoved(pos)]);
                let mut out = self.ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("drawer_test_ui"),
                        UiBuilder::new().max_rect(screen()),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }

            self.time += 0.1;
            let release = self.raw_input(vec![
                egui::Event::PointerMoved(to),
                egui::Event::PointerButton {
                    pos: to,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
            ]);
            let mut out = self.ctx.run_ui(release, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("drawer_test_ui"),
                    UiBuilder::new().max_rect(screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> DrawerState {
            let id = Id::unique("drawer_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<DrawerState>(id))
                .unwrap_or_default()
        }
    }

    #[test]
    fn constants_match_source() {
        assert!((DISMISS_THRESHOLD_FRACTION - 0.25).abs() < f32::EPSILON);
        assert_eq!(DISMISS_THRESHOLD_MIN, 80.0);
        assert!((BACKDROP_ALPHA - 0.10).abs() < f32::EPSILON); // bg-black/10
        assert_eq!(PAD, 16.0); // p-4
    }

    #[test]
    fn trigger_opens_and_close_widget_dismisses() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;
        let mut close_rect = Rect::ZERO;

        for _ in 0..4 {
            h.run(|ui| {
                Drawer::controlled(&mut open).id_salt("w1").show(ui, |ui| {
                    trigger_rect = DrawerTrigger::new("Open").show(ui).rect;
                    DrawerContent::new().show(ui, |ui| {
                        DrawerHeader::new().show(ui, |ui| {
                            DrawerTitle::new("Cart").show(ui);
                        });
                        DrawerFooter::new().show(ui, |ui| {
                            close_rect = DrawerClose::new("Close").show(ui).rect;
                        });
                    });
                });
            });
        }
        assert!(!open);

        h.click(trigger_rect.center(), |ui| {
            Drawer::controlled(&mut open).id_salt("w1").show(ui, |ui| {
                DrawerTrigger::new("Open").show(ui);
                DrawerContent::new().show(ui, |ui| {
                    DrawerHeader::new().show(ui, |ui| {
                        DrawerTitle::new("Cart").show(ui);
                    });
                    DrawerFooter::new().show(ui, |ui| {
                        DrawerClose::new("Close").show(ui);
                    });
                });
            });
        });
        assert!(open, "trigger click opens the drawer");

        // Settle the slide.
        for _ in 0..6 {
            h.run(|ui| {
                Drawer::controlled(&mut open).id_salt("w1").show(ui, |ui| {
                    DrawerTrigger::new("Open").show(ui);
                    DrawerContent::new().show(ui, |ui| {
                        DrawerHeader::new().show(ui, |ui| {
                            DrawerTitle::new("Cart").show(ui);
                        });
                        DrawerFooter::new().show(ui, |ui| {
                            close_rect = DrawerClose::new("Close").show(ui).rect;
                        });
                    });
                });
            });
        }

        h.click(close_rect.center(), |ui| {
            Drawer::controlled(&mut open).id_salt("w1").show(ui, |ui| {
                DrawerTrigger::new("Open").show(ui);
                DrawerContent::new().show(ui, |ui| {
                    DrawerHeader::new().show(ui, |ui| {
                        DrawerTitle::new("Cart").show(ui);
                    });
                    DrawerFooter::new().show(ui, |ui| {
                        DrawerClose::new("Close").show(ui);
                    });
                });
            });
        });
        assert!(!open, "DrawerClose must dismiss the drawer");
    }

    #[test]
    fn drag_beyond_threshold_dismisses_and_snap_back_keeps_open() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;
        let mut panel_rect = Rect::ZERO;

        for _ in 0..3 {
            h.run(|ui| {
                Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                    trigger_rect = DrawerTrigger::new("Open").show(ui).rect;
                    DrawerContent::new().show(ui, |ui| {
                        DrawerTitle::new("Cart").show(ui);
                    });
                });
            });
        }

        h.click(trigger_rect.center(), |ui| {
            Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                DrawerTrigger::new("Open").show(ui);
                DrawerContent::new().show(ui, |ui| {
                    DrawerTitle::new("Cart").show(ui);
                });
            });
        });
        assert!(open);

        // Settle fully open and grab the resting panel rect.
        for _ in 0..8 {
            h.run(|ui| {
                Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                    DrawerTrigger::new("Open").show(ui);
                    if let Some(r) = DrawerContent::new().show(ui, |ui| {
                        DrawerTitle::new("Cart").show(ui);
                    }) {
                        panel_rect = r.rect;
                    }
                });
            });
        }
        assert!(open);

        // Drag the handle down past the dismiss threshold
        // (max(25% of the panel height, 80px)) → dismiss.
        let from = Pos2::new(panel_rect.center().x, panel_rect.top() + 8.0);
        let to = Pos2::new(from.x, from.y + 200.0);
        h.drag(from, to, |ui| {
            Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                DrawerTrigger::new("Open").show(ui);
                DrawerContent::new().show(ui, |ui| {
                    DrawerTitle::new("Cart").show(ui);
                });
            });
        });
        assert!(
            !h.state("d1").open,
            "releasing past the dismiss threshold must close the drawer"
        );

        // Let the close animation finish (the backdrop swallows clicks while
        // the panel slides out), then reopen and drag only a little → snap
        // back, stays open.
        for _ in 0..8 {
            h.run(|ui| {
                Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                    DrawerTrigger::new("Open").show(ui);
                    DrawerContent::new().show(ui, |ui| {
                        DrawerTitle::new("Cart").show(ui);
                    });
                });
            });
        }
        h.click(trigger_rect.center(), |ui| {
            Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                DrawerTrigger::new("Open").show(ui);
                DrawerContent::new().show(ui, |ui| {
                    DrawerTitle::new("Cart").show(ui);
                });
            });
        });
        for _ in 0..6 {
            h.run(|ui| {
                Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                    DrawerTrigger::new("Open").show(ui);
                    DrawerContent::new().show(ui, |ui| {
                        DrawerTitle::new("Cart").show(ui);
                    });
                });
            });
        }
        let from = Pos2::new(panel_rect.center().x, panel_rect.top() + 8.0);
        let to = Pos2::new(from.x, from.y + 20.0);
        h.drag(from, to, |ui| {
            Drawer::controlled(&mut open).id_salt("d1").show(ui, |ui| {
                DrawerTrigger::new("Open").show(ui);
                DrawerContent::new().show(ui, |ui| {
                    DrawerTitle::new("Cart").show(ui);
                });
            });
        });
        assert!(
            h.state("d1").open,
            "releasing below the threshold must snap back open"
        );
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let d = Drawer::controlled(&mut open).id_salt("x");
        assert_eq!(d.id_salt.as_deref(), Some("x"));
        let c = DrawerContent::new().show_handle(false);
        assert!(!c.show_handle);
        let t = DrawerTrigger::new("hi").disabled(true).id_salt("t");
        assert!(t.disabled);
        let _ = DrawerClose::new("bye").id_salt("bye");
        let _ = DrawerHeader::new();
        let _ = DrawerFooter::new();
        let _ = DrawerTitle::new("T");
        let _ = DrawerDescription::new("D");
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
