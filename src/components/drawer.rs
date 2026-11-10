//! shadcn `drawer.tsx` — an edge-docked sheet with drag-to-dismiss
//! (vaul-style).
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/drawer.tsx` (vaul) with the
//! nova recipe: `bg-popover text-popover-foreground` plus per-direction
//! corners and a single border edge on the screen-facing side — bottom
//! `rounded-t-xl border-t`, top `rounded-b-xl border-b`, left
//! `rounded-r-xl border-r`, right `rounded-l-xl border-l` — the
//! `bg-muted h-1 w-[100px] rounded-full` drag handle at `mt-4` (bottom
//! drawer only), and a `bg-black/10` animated backdrop.
//!
//! Behavior: the panel can be dragged out of the screen with the pointer;
//! releasing past 25% of the panel size (or 80 px) dismisses it, otherwise
//! it snaps back — vaul's default dismissible drawer. Backdrop clicks and
//! Escape also dismiss. Content fades and slides like the source.
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
use egui::{
    Align, Align2, Color32, Context, CornerRadius, Frame, Id, Modal, Painter, Pos2, Rect, Response,
    Sense, Stroke, Ui, Vec2,
};

/// The vaul default dismiss threshold: 25% of the panel size along the drag
/// axis (with a 80 px floor so tiny panels stay usable).
const DISMISS_THRESHOLD_FRACTION: f32 = 0.25;
const DISMISS_THRESHOLD_MIN: f32 = 80.0;
/// Resting budget for the closed slide position. The source's `mt-24` (96px)
/// is a no-op for a bottom-anchored panel; a small budget keeps the slide
/// math positive without affecting the resting layout.
const HEADROOM: f32 = 24.0;
/// nova backdrop: `bg-black/10`.
const BACKDROP_ALPHA: f32 = 0.10;
/// Slide duration (vaul's springy ~300ms).
const SLIDE_SECS: f32 = 0.3;
/// Header/footer padding: `p-4`.
const PAD: f32 = 16.0;
/// Side panels: `w-3/4 sm:max-w-sm` = min(384px, 75% of the screen).
const SIDE_MAX_WIDTH: f32 = 384.0;
/// Side panels cap at `sm:max-w-sm` from the `sm` breakpoint up.
const SM_BREAKPOINT: f32 = 640.0;
/// Header/description centering applies below the `md` breakpoint only
/// (`md:text-left` overrides the direction's `text-center`).
const MD_BREAKPOINT: f32 = 768.0;

/// Which screen edge the drawer docks against (vaul
/// `data-[vaul-drawer-direction]`, set on the root).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerDirection {
    /// Docked to the bottom edge (vaul default): full width,
    /// `rounded-t-xl border-t`, draggable down.
    #[default]
    Bottom,
    /// Docked to the top edge: full width, `rounded-b-xl border-b`,
    /// draggable up.
    Top,
    /// Docked to the left edge: full height, `rounded-r-xl border-r`,
    /// `w-3/4 sm:max-w-sm`, draggable left.
    Left,
    /// Docked to the right edge: full height, `rounded-l-xl border-l`,
    /// `w-3/4 sm:max-w-sm`, draggable right.
    Right,
}

impl DrawerDirection {
    /// `true` for the top/bottom (vertically docked) directions.
    pub fn is_vertical(self) -> bool {
        matches!(self, DrawerDirection::Bottom | DrawerDirection::Top)
    }

    /// The egui anchor for the resting panel position.
    fn anchor(self) -> Align2 {
        match self {
            DrawerDirection::Bottom => Align2::CENTER_BOTTOM,
            DrawerDirection::Top => Align2::CENTER_TOP,
            DrawerDirection::Left => Align2::LEFT_CENTER,
            DrawerDirection::Right => Align2::RIGHT_CENTER,
        }
    }

    /// Anchor offset for a `offset`-point displacement out of the screen.
    fn slide_offset(self, offset: f32) -> Vec2 {
        match self {
            DrawerDirection::Bottom => Vec2::new(0.0, offset),
            DrawerDirection::Top => Vec2::new(0.0, -offset),
            DrawerDirection::Left => Vec2::new(-offset, 0.0),
            DrawerDirection::Right => Vec2::new(offset, 0.0),
        }
    }

    /// Positive drag displacement from `grab` to `pos` along the dismiss
    /// axis (0 when dragged the wrong way).
    fn drag_delta(self, grab: Pos2, pos: Pos2) -> f32 {
        match self {
            DrawerDirection::Bottom => (pos.y - grab.y).max(0.0),
            DrawerDirection::Top => (grab.y - pos.y).max(0.0),
            DrawerDirection::Left => (grab.x - pos.x).max(0.0),
            DrawerDirection::Right => (pos.x - grab.x).max(0.0),
        }
    }

    /// nova corner recipe: only the corners facing into the screen are
    /// rounded (`rounded-t-xl` bottom, `rounded-b-xl` top, `rounded-r-xl`
    /// left, `rounded-l-xl` right).
    fn corner_radius(self, r: u8) -> CornerRadius {
        match self {
            DrawerDirection::Bottom => CornerRadius {
                nw: r,
                ne: r,
                sw: 0,
                se: 0,
            },
            DrawerDirection::Top => CornerRadius {
                nw: 0,
                ne: 0,
                sw: r,
                se: r,
            },
            DrawerDirection::Left => CornerRadius {
                nw: 0,
                ne: r,
                sw: 0,
                se: r,
            },
            DrawerDirection::Right => CornerRadius {
                nw: r,
                ne: 0,
                sw: r,
                se: 0,
            },
        }
    }

    /// Paints the single border edge on the screen-facing side
    /// (`border-t`/`border-b`/`border-r`/`border-l`).
    fn paint_border(self, painter: &Painter, panel: Rect, color: Color32) {
        let stroke = Stroke::new(1.0, color);
        match self {
            DrawerDirection::Bottom => painter.line_segment(
                [
                    Pos2::new(panel.left(), panel.top() + 0.5),
                    Pos2::new(panel.right(), panel.top() + 0.5),
                ],
                stroke,
            ),
            DrawerDirection::Top => painter.line_segment(
                [
                    Pos2::new(panel.left(), panel.bottom() - 0.5),
                    Pos2::new(panel.right(), panel.bottom() - 0.5),
                ],
                stroke,
            ),
            DrawerDirection::Left => painter.line_segment(
                [
                    Pos2::new(panel.right() - 0.5, panel.top()),
                    Pos2::new(panel.right() - 0.5, panel.bottom()),
                ],
                stroke,
            ),
            DrawerDirection::Right => painter.line_segment(
                [
                    Pos2::new(panel.left() + 0.5, panel.top()),
                    Pos2::new(panel.left() + 0.5, panel.bottom()),
                ],
                stroke,
            ),
        };
    }
}

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
    /// Docking direction (vaul sets it on the root; children read it).
    direction: DrawerDirection,
    /// Pixels the panel was dragged out of the screen this gesture.
    drag_offset: f32,
    /// Pointer position when the gesture started; the offset is measured
    /// from it so pointer jumps between widgets never pollute the drag.
    grab_x: f32,
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

/// The docking direction of the innermost drawer (bottom outside one).
fn scope_direction(ui: &Ui) -> DrawerDirection {
    match current_scope(ui) {
        Some(state_id) => read_state(&ui.ctx().clone(), state_id).direction,
        None => DrawerDirection::Bottom,
    }
}

// ---------------------------------------------------------------------------
// Drawer (root)
// ---------------------------------------------------------------------------

/// The shadcn drawer root. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Drawer<'a> {
    controlled: Option<&'a mut bool>,
    id_salt: Option<String>,
    direction: DrawerDirection,
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
            direction: DrawerDirection::default(),
        }
    }

    /// Creates a drawer whose open state is owned by the caller. Trigger,
    /// close button, drag-dismiss, backdrop clicks and Escape all write back.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            id_salt: None,
            direction: DrawerDirection::default(),
        }
    }

    /// Sets a stable id salt so the open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Sets which screen edge the drawer docks against
    /// (vaul `data-[vaul-drawer-direction]`). Defaults to
    /// [`DrawerDirection::Bottom`].
    pub fn direction(mut self, direction: DrawerDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Lays out the trigger and content, showing the drawer while open.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_drawer"),
        };

        let ctx = ui.ctx().clone();
        {
            let mut state = read_state(&ctx, state_id);
            state.direction = self.direction;
            if let Some(open) = self.controlled.as_deref() {
                state.open = *open;
            }
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

/// The docked panel: `bg-popover` with the nova per-direction corner recipe
/// and the single screen-facing border edge, plus the drag handle for the
/// bottom drawer, over a dimmed animated backdrop. Draggable to dismiss.
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

    /// The handle strip renders for the bottom drawer only
    /// (`group-data-[vaul-drawer-direction=bottom]:block`).
    fn shows_handle(&self, direction: DrawerDirection) -> bool {
        self.show_handle && direction == DrawerDirection::Bottom
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
        let direction = state.direction;

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let modal_id = state_id.with("modal");
        let offset_anim_id = state_id.with("offset");
        let size_key = state_id.with("panel_size");
        let panel_size: f32 = ctx
            .data_mut(|d| d.get_temp::<f32>(size_key).unwrap_or(0.0))
            .max(1.0);
        let rest_offset = panel_size + HEADROOM;

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
            .corner_radius(direction.corner_radius(theme.radius.xl_u8()));

        let area = Modal::default_area(modal_id)
            .anchor(direction.anchor(), direction.slide_offset(offset));
        let modal = Modal::new(modal_id)
            .area(area)
            .backdrop_color(backdrop)
            .frame(Frame::new());

        let drag_id = state_id.with("drag");
        let handle_id = state_id.with("handle");
        let content_rect = ctx.content_rect();

        let modal_resp = modal.show(&ctx, |ui| {
            ui.set_opacity(alpha);
            ui.style_mut().visuals.override_text_color = Some(p.popover_foreground);

            // Drag layer under the content: catches drags on empty areas;
            // interactive widgets on top keep their clicks.
            let drag_region = ui.max_rect();
            let drag = ui.interact(drag_region, drag_id, Sense::drag());
            let handle = if self.shows_handle(direction) {
                Some(ui.interact(
                    Rect::from_min_size(
                        Pos2::new(drag_region.center().x - 60.0, drag_region.top()),
                        Vec2::new(120.0, 20.0),
                    ),
                    handle_id,
                    Sense::drag(),
                ))
            } else {
                None
            };

            if direction.is_vertical() {
                // `inset-x-0`: full-width top/bottom panel.
                let screen_w = content_rect.width();
                ui.set_min_width(screen_w);
                ui.set_max_width(screen_w);
            } else {
                // `inset-y-0`: full-height side panel.
                let screen_h = content_rect.height();
                ui.set_min_height(screen_h);
                ui.set_max_height(screen_h);
            }

            frame.show(ui, |ui| {
                if direction.is_vertical() {
                    let screen_w = content_rect.width();
                    ui.set_min_width(screen_w);
                    ui.set_max_width(screen_w);
                    // `max-h-[80vh]` (drawer.tsx:59-60).
                    ui.set_max_height(content_rect.height() * 0.8);
                } else {
                    // `w-3/4 sm:max-w-sm`: 75% of the screen, capped at 384px
                    // from the `sm` breakpoint up.
                    let side_w = if content_rect.width() < SM_BREAKPOINT {
                        content_rect.width() * 0.75
                    } else {
                        (content_rect.width() * 0.75).min(SIDE_MAX_WIDTH)
                    };
                    ui.set_min_width(side_w);
                    ui.set_max_width(side_w);
                    let screen_h = content_rect.height();
                    ui.set_min_height(screen_h);
                    ui.set_max_height(screen_h);
                }
                if self.shows_handle(direction) {
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

            // The single border edge on the screen-facing side
            // (`border-t`/`border-b`/`border-r`/`border-l`).
            direction.paint_border(ui.painter(), ui.min_rect(), p.border);

            // Drag bookkeeping (after layout so the interact regions exist).
            // The offset is measured from the grab point (not accumulated
            // deltas) so pointer jumps between widgets never pollute it.
            let mut state = read_state(&ctx, state_id);
            let gesture_pos = drag
                .interact_pointer_pos()
                .or_else(|| handle.as_ref().and_then(|h| h.interact_pointer_pos()));
            let started = drag.drag_started() || handle.as_ref().is_some_and(|h| h.drag_started());
            let stopped = drag.drag_stopped() || handle.as_ref().is_some_and(|h| h.drag_stopped());
            if started {
                state.dragging = true;
                if let Some(pos) = gesture_pos {
                    state.grab_x = pos.x;
                    state.grab_y = pos.y;
                }
                state.drag_offset = 0.0;
            }
            if let Some(pos) = gesture_pos.filter(|_| state.dragging) {
                state.drag_offset =
                    direction.drag_delta(Pos2::new(state.grab_x, state.grab_y), pos);
            }
            if stopped && state.dragging {
                state.dragging = false;
                let threshold =
                    (panel_size * DISMISS_THRESHOLD_FRACTION).max(DISMISS_THRESHOLD_MIN);
                if state.drag_offset > threshold {
                    state.open = false;
                }
                state.drag_offset = 0.0;
            }
            write_state(&ctx, state_id, state);
        });

        // Cache the measured panel size along the drag axis for the slide
        // math (height for top/bottom, width for left/right).
        let axis_size = if direction.is_vertical() {
            modal_resp.response.rect.height()
        } else {
            modal_resp.response.rect.width()
        };
        ctx.data_mut(|d| d.insert_temp(size_key, axis_size));

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

/// nova drawer header: `gap-0.5 p-4`. The base direction classes center the
/// text for the top/bottom drawers, but `md:text-left` wins from the `md`
/// breakpoint up (drawer.tsx:79); side drawers stay left-aligned.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawerHeader {
    _private: (),
}

impl DrawerHeader {
    /// Creates a drawer header.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the header (centered on small screens for top/bottom drawers).
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        let direction = scope_direction(ui);
        let centered = direction.is_vertical() && ui.ctx().content_rect().width() < MD_BREAKPOINT;
        let align = if centered { Align::Center } else { Align::LEFT };

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
            .with_layout(egui::Layout::top_down(align), content)
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

    /// Renders the description (centered on small screens for top/bottom
    /// drawers, like the header).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let font = egui::FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout(self.text, font, p.muted_foreground, ui.available_width());
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            let direction = scope_direction(ui);
            let centered =
                direction.is_vertical() && ui.ctx().content_rect().width() < MD_BREAKPOINT;
            let origin = if centered {
                Pos2::new(ui.max_rect().center().x - galley.size().x * 0.5, rect.min.y)
            } else {
                rect.min
            };
            ui.painter().galley(origin, galley, p.muted_foreground);
        }
        response
    }
}

/// nova drawer footer: `mt-auto flex flex-col gap-2 p-4` — a left-aligned
/// column (`gap-2` = 8px) that flows after the content.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawerFooter {
    _private: (),
}

impl DrawerFooter {
    /// Creates a drawer footer.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the footer as a left-aligned column.
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
        child.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::SM;
        let real = child
            .with_layout(egui::Layout::top_down(Align::LEFT), content)
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
