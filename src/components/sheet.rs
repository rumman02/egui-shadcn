//! shadcn `sheet.tsx` — a side drawer over a dimmed backdrop.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/sheet.tsx` (Radix Dialog on
//! the side) with the nova recipe: `bg-popover text-popover-foreground
//! shadow-lg text-sm`, `w-3/4 sm:max-w-sm` for the left/right sides (384 px),
//! full-width auto-height for top/bottom, a single border on the inner edge,
//! and the slide-in/out animation from the source
//! (`data-[state=open]:duration-500`, `data-[state=closed]:duration-300`).
//!
//! Backdrop: `bg-black/10` (nova), fading with the panel; clicking it or
//! pressing Escape dismisses the sheet; a ghost X sits at `top-3 right-3`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Sheet::new().id_salt("menu").show(ui, |ui| {
//!     SheetTrigger::widget(|ui| Button::new("Open").show(ui)).show(ui);
//!     SheetContent::new().side(SheetSide::Right).show(ui, |ui| {
//!         SheetHeader::new().show(ui, |ui| {
//!             SheetTitle::new("Menu").show(ui);
//!             SheetDescription::new("Navigation").show(ui);
//!         });
//!         SheetFooter::new().show(ui, |ui| {
//!             SheetClose::widget(|ui| Button::new("Close").show(ui)).show(ui);
//!         });
//!     });
//! });
//! ```

use crate::components::button::{Button, ButtonVariant};
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::shadow;
use egui::{
    Align2, Color32, Context, CornerRadius, Frame, Id, Modal, Pos2, Rect, Response, Sense, Stroke,
    Ui, Vec2, WidgetInfo,
};

/// nova `data-[side=left|right]:sm:max-w-sm` (384 px).
pub const SHEET_MAX_WIDTH: f32 = 384.0;
/// Header/footer padding: `p-4`.
const PAD: f32 = 16.0;
/// nova ring: `ring-foreground/10` (used for the inner-edge border).
const BORDER_ALPHA: f32 = 0.10;
/// nova backdrop: `bg-black/10`.
const BACKDROP_ALPHA: f32 = 0.10;
/// Open duration: `data-[state=open]:duration-500`.
const OPEN_SECS: f32 = 0.5;
/// Close duration: `data-[state=closed]:duration-300`.
const CLOSE_SECS: f32 = 0.3;

/// Which side the sheet docks to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetSide {
    /// Full-height panel on the left edge.
    Left,
    /// Full-height panel on the right edge (the source default).
    #[default]
    Right,
    /// Full-width panel on the top edge.
    Top,
    /// Full-width panel on the bottom edge.
    Bottom,
}

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors dialog.rs)
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_sheet_stack")
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

#[derive(Clone, Copy, Debug, Default)]
struct SheetState {
    open: bool,
    side: SheetSide,
}

fn read_state(ctx: &Context, state_id: Id) -> SheetState {
    ctx.data_mut(|d| d.get_temp::<SheetState>(state_id).unwrap_or_default())
}

fn write_state(ctx: &Context, state_id: Id, state: SheetState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

fn set_open(ctx: &Context, state_id: Id, open: bool) {
    let mut state = read_state(ctx, state_id);
    state.open = open;
    write_state(ctx, state_id, state);
}

/// Slide progress: `data-[state=open]:duration-500`,
/// `data-[state=closed]:duration-300`.
fn sheet_progress(ctx: &Context, anim_id: Id, open: bool) -> f32 {
    ctx.animate_value_with_time(
        anim_id,
        if open { 1.0 } else { 0.0 },
        if open { OPEN_SECS } else { CLOSE_SECS },
    )
}

// ---------------------------------------------------------------------------
// Sheet (root)
// ---------------------------------------------------------------------------

/// The shadcn sheet root. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Sheet<'a> {
    controlled: Option<&'a mut bool>,
    id_salt: Option<String>,
}

impl Default for Sheet<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Sheet<'a> {
    /// Creates an uncontrolled sheet (starts closed).
    pub fn new() -> Self {
        Self {
            controlled: None,
            id_salt: None,
        }
    }

    /// Creates a sheet whose open state is owned by the caller. Trigger,
    /// close button, backdrop clicks and Escape all write back.
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

    /// Lays out the trigger and content, showing the sheet while open.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_sheet"),
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
// SheetTrigger / SheetClose
// ---------------------------------------------------------------------------

/// Opens the innermost [`Sheet`] when clicked. Renders an outline button by
/// default or a custom widget via [`SheetTrigger::widget`].
pub struct SheetTrigger<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for SheetTrigger<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> SheetTrigger<'a> {
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

/// Closes the innermost [`Sheet`] when clicked. Renders an outline button by
/// default or a custom widget via [`SheetClose::widget`].
pub struct SheetClose<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    id_salt: Option<String>,
}

impl Default for SheetClose<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> SheetClose<'a> {
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

    /// Renders the close control; clicking dismisses the sheet.
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
// SheetContent
// ---------------------------------------------------------------------------

/// The drawer panel docked to a side, with a close X at `top-3 right-3`, over
/// a dimmed animated backdrop.
#[derive(Clone, Copy, Debug)]
pub struct SheetContent {
    side: SheetSide,
    width: Option<f32>,
    show_close_button: bool,
}

impl Default for SheetContent {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetContent {
    /// Creates sheet content (source default `side = "right"`).
    pub fn new() -> Self {
        Self {
            side: SheetSide::default(),
            width: None,
            show_close_button: true,
        }
    }

    /// Sets the docking side.
    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    /// Overrides the panel width for the left/right sides (default:
    /// `min([`SHEET_MAX_WIDTH`], 75% of the window)`).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Hides the corner close button (`showCloseButton = false`).
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Renders the sheet, or nothing while closed. Returns the modal area
    /// response while visible.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Option<Response> {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Sheet: render statically so mistakes stay visible.
            content(ui);
            return None;
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        state.side = self.side;
        let open = state.open;
        write_state(&ctx, state_id, state);

        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let modal_id = state_id.with("modal");
        let anim_id = state_id.with("anim");

        let progress = sheet_progress(&ctx, anim_id, open);
        if !(open || progress > 0.001) {
            return None;
        }

        let screen = ctx.content_rect();
        let side_width = self
            .width
            .unwrap_or(SHEET_MAX_WIDTH)
            .min(screen.width() * 0.75)
            .max(80.0);
        let vertical = matches!(self.side, SheetSide::Left | SheetSide::Right);

        // Measure the panel height for top/bottom sheets (auto height).
        let height_key = state_id.with("panel_h");
        let measured_h: f32 = ctx.data_mut(|d| d.get_temp(height_key).unwrap_or(0.0));
        let panel_h = if vertical {
            screen.height()
        } else {
            measured_h.max(1.0)
        };

        // Slide-in offset: start just off the docked edge.
        let slide = (1.0 - progress) * if vertical { side_width } else { panel_h };
        let (anchor, offset) = match self.side {
            SheetSide::Right => (Align2::RIGHT_CENTER, Vec2::new(slide, 0.0)),
            SheetSide::Left => (Align2::LEFT_CENTER, Vec2::new(-slide, 0.0)),
            SheetSide::Top => (Align2::CENTER_TOP, Vec2::new(0.0, -slide)),
            SheetSide::Bottom => (Align2::CENTER_BOTTOM, Vec2::new(0.0, slide)),
        };

        let backdrop = color_alpha(Color32::from_black_alpha(255), BACKDROP_ALPHA * progress);

        let panel_frame = Frame::new()
            .fill(p.popover)
            .corner_radius(CornerRadius::same(0))
            .shadow(shadow::shadow_lg(dark));

        let area = Modal::default_area(modal_id).anchor(anchor, offset);
        let modal = Modal::new(modal_id)
            .area(area)
            .backdrop_color(backdrop)
            .frame(Frame::new());

        let modal_resp = modal.show(&ctx, |ui| {
            ui.set_opacity(progress);
            if vertical {
                ui.set_min_width(side_width);
                ui.set_max_width(side_width);
                ui.set_min_height(screen.height());
            } else {
                ui.set_min_width(screen.width());
                ui.set_max_width(screen.width());
            }
            panel_frame.show(ui, |ui| {
                if vertical {
                    ui.set_min_width(side_width);
                    ui.set_max_width(side_width);
                }
                ui.style_mut().visuals.override_text_color = Some(p.popover_foreground);
                content(ui);
            });

            // Panel rect in this frame's coordinates.
            let panel = ui.min_rect();
            let painter = ui.painter();

            // Single border on the inner edge (`border-l` for right, …).
            let border = Stroke::new(1.0, color_alpha(p.foreground, BORDER_ALPHA));
            match self.side {
                SheetSide::Right => {
                    painter.line_segment([panel.left_top(), panel.left_bottom()], border);
                }
                SheetSide::Left => {
                    painter.line_segment([panel.right_top(), panel.right_bottom()], border);
                }
                SheetSide::Top => {
                    painter.line_segment([panel.left_bottom(), panel.right_bottom()], border);
                }
                SheetSide::Bottom => {
                    painter.line_segment([panel.left_top(), panel.right_top()], border);
                }
            }

            // `absolute top-3 right-3` ghost X close button.
            if self.show_close_button {
                let x_rect = Rect::from_min_size(
                    Pos2::new(panel.right() - 12.0 - 24.0, panel.top() + 12.0),
                    Vec2::splat(24.0),
                );
                let x_id = state_id.with("sheet_close_x");
                let x_resp = ui.interact(x_rect, x_id, Sense::click());
                x_resp.widget_info(|| {
                    WidgetInfo::labeled(egui::Role::Button, true, "Close".to_owned())
                });
                let fill = if x_resp.hovered() {
                    p.secondary
                } else {
                    Color32::TRANSPARENT
                };
                let fg = if x_resp.hovered() {
                    p.secondary_foreground
                } else {
                    color_alpha(p.foreground, 0.7)
                };
                painter.rect_filled(x_rect, theme.radius.sm_u8(), fill);
                let icon_rect = Rect::from_center_size(x_rect.center(), Vec2::splat(16.0));
                Icon::X.paint(painter, icon_rect, fg, 16.0 / 24.0);
                if x_resp.clicked() {
                    set_open(&ctx, state_id, false);
                }
            }
        });

        // Cache the measured panel height for top/bottom slide math.
        let panel_h = modal_resp.response.rect.height();
        ctx.data_mut(|d| d.insert_temp(height_key, panel_h));

        // Sheet is Dialog-based: Escape and backdrop clicks both dismiss.
        if modal_resp.should_close() && open {
            set_open(&ctx, state_id, false);
        }

        Some(modal_resp.response)
    }
}

// ---------------------------------------------------------------------------
// SheetHeader / SheetTitle / SheetDescription / SheetFooter
// ---------------------------------------------------------------------------

/// nova sheet header: `gap-0.5 p-4`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SheetHeader {
    _private: (),
}

impl SheetHeader {
    /// Creates a sheet header.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the header with 16 px padding and a 2 px column gap.
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        // Measure with an invisible pass (widgets disabled, no double
        // callbacks), then lay out for real inside the padded box.
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
        let real = content(&mut child);
        ui.allocate_space(Vec2::new(flow_w, content_h + PAD * 2.0));
        real
    }
}

/// nova sheet title: `text-base font-medium`.
#[derive(Clone, Debug)]
pub struct SheetTitle {
    text: String,
}

impl SheetTitle {
    /// Creates a sheet title.
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

/// `text-sm text-muted-foreground` sheet description, wrapping.
#[derive(Clone, Debug)]
pub struct SheetDescription {
    text: String,
}

impl SheetDescription {
    /// Creates a sheet description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let font = egui::FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout(self.text, font, p.muted_foreground, ui.available_width());
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.muted_foreground);
        }
        response
    }
}

/// nova sheet footer: `mt-auto gap-2 p-4` — bottom-anchored in the panel for
/// the left/right sides (full height), flowing after the content for
/// top/bottom.
#[derive(Clone, Copy, Debug, Default)]
pub struct SheetFooter {
    _private: (),
}

impl SheetFooter {
    /// Creates a sheet footer.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the footer, right-aligned.
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        let side = current_scope(ui)
            .map(|id| read_state(ui.ctx(), id).side)
            .unwrap_or_default();

        // Measure the content height with an invisible pass.
        let flow_min = ui.cursor().min;
        let flow_w = ui.available_width();
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
        let mut measure = ui.new_child(egui::UiBuilder::new().max_rect(measure_rect).invisible());
        let _ = content(&mut measure);
        let content_h = measure.min_rect().height();

        let full_height_panel = matches!(side, SheetSide::Left | SheetSide::Right);
        if full_height_panel {
            // Push the footer to the bottom of the panel (`mt-auto`).
            let panel_bottom = ui.max_rect().bottom();
            let footer_top = (panel_bottom - content_h - PAD * 2.0).max(flow_min.y);
            if footer_top > flow_min.y {
                ui.allocate_space(Vec2::new(flow_w, footer_top - flow_min.y));
            }
        }

        let inner = Rect::from_min_size(
            Pos2::new(ui.cursor().min.x + PAD, ui.cursor().min.y + PAD),
            Vec2::new(flow_w - PAD * 2.0, content_h),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        let real = child
            .with_layout(egui::Layout::right_to_left(egui::Align::Center), content)
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
                    Id::unique("sheet_test_ui"),
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
                        Id::unique("sheet_test_ui"),
                        UiBuilder::new().max_rect(screen()),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        fn press_escape(&mut self, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = self.raw_input(vec![egui::Event::Key {
                key: egui::Key::Escape,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
                physical_key: None,
            }]);
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("sheet_test_ui"),
                    UiBuilder::new().max_rect(screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn constants_match_nova_source() {
        assert_eq!(SHEET_MAX_WIDTH, 384.0); // sm:max-w-sm
        assert_eq!(PAD, 16.0); // p-4
        assert!((BACKDROP_ALPHA - 0.10).abs() < f32::EPSILON); // bg-black/10
        assert!((OPEN_SECS - 0.5).abs() < f32::EPSILON); // duration-500
        assert!((CLOSE_SECS - 0.3).abs() < f32::EPSILON); // duration-300
    }

    #[test]
    fn trigger_opens_and_close_widget_dismisses() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;
        let mut close_rect = Rect::ZERO;

        for _ in 0..4 {
            h.run(|ui| {
                Sheet::controlled(&mut open).id_salt("s1").show(ui, |ui| {
                    trigger_rect = SheetTrigger::new("Open").show(ui).rect;
                    SheetContent::new().side(SheetSide::Right).show(ui, |ui| {
                        SheetHeader::new().show(ui, |ui| {
                            SheetTitle::new("Menu").show(ui);
                        });
                        SheetFooter::new().show(ui, |ui| {
                            close_rect = SheetClose::new("Close").show(ui).rect;
                        });
                    });
                });
            });
        }
        assert!(!open);

        h.click(trigger_rect.center(), |ui| {
            Sheet::controlled(&mut open).id_salt("s1").show(ui, |ui| {
                SheetTrigger::new("Open").show(ui);
                SheetContent::new().side(SheetSide::Right).show(ui, |ui| {
                    SheetHeader::new().show(ui, |ui| {
                        SheetTitle::new("Menu").show(ui);
                    });
                    SheetFooter::new().show(ui, |ui| {
                        SheetClose::new("Close").show(ui);
                    });
                });
            });
        });
        assert!(open, "trigger click opens the sheet");

        // Let the sheet fully slide in and settle.
        for _ in 0..6 {
            h.run(|ui| {
                Sheet::controlled(&mut open).id_salt("s1").show(ui, |ui| {
                    SheetTrigger::new("Open").show(ui);
                    SheetContent::new().side(SheetSide::Right).show(ui, |ui| {
                        SheetHeader::new().show(ui, |ui| {
                            SheetTitle::new("Menu").show(ui);
                        });
                        SheetFooter::new().show(ui, |ui| {
                            close_rect = SheetClose::new("Close").show(ui).rect;
                        });
                    });
                });
            });
        }

        h.click(close_rect.center(), |ui| {
            Sheet::controlled(&mut open).id_salt("s1").show(ui, |ui| {
                SheetTrigger::new("Open").show(ui);
                SheetContent::new().side(SheetSide::Right).show(ui, |ui| {
                    SheetHeader::new().show(ui, |ui| {
                        SheetTitle::new("Menu").show(ui);
                    });
                    SheetFooter::new().show(ui, |ui| {
                        SheetClose::new("Close").show(ui);
                    });
                });
            });
        });
        assert!(!open, "SheetClose must dismiss the sheet");
    }

    #[test]
    fn escape_closes_sheet() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;

        for _ in 0..3 {
            h.run(|ui| {
                Sheet::controlled(&mut open).id_salt("s2").show(ui, |ui| {
                    trigger_rect = SheetTrigger::new("Open").show(ui).rect;
                    SheetContent::new().show(ui, |ui| {
                        SheetTitle::new("Menu").show(ui);
                    });
                });
            });
        }

        h.click(trigger_rect.center(), |ui| {
            Sheet::controlled(&mut open).id_salt("s2").show(ui, |ui| {
                SheetTrigger::new("Open").show(ui);
                SheetContent::new().show(ui, |ui| {
                    SheetTitle::new("Menu").show(ui);
                });
            });
        });
        assert!(open);

        h.press_escape(|ui| {
            Sheet::controlled(&mut open).id_salt("s2").show(ui, |ui| {
                SheetTrigger::new("Open").show(ui);
                SheetContent::new().show(ui, |ui| {
                    SheetTitle::new("Menu").show(ui);
                });
            });
        });
        assert!(!open, "Escape must dismiss the sheet");
    }

    #[test]
    fn side_panel_geometry() {
        // Right sheet: full height, width min(384, 75% of screen), docked to
        // the right edge.
        let screen = screen();
        let side_width = SHEET_MAX_WIDTH.min(screen.width() * 0.75);
        assert_eq!(side_width, 384.0);
        // Narrow screens clamp to 75%.
        let narrow = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0));
        assert_eq!(SHEET_MAX_WIDTH.min(narrow.width() * 0.75), 300.0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let s = Sheet::controlled(&mut open).id_salt("x");
        assert_eq!(s.id_salt.as_deref(), Some("x"));
        let c = SheetContent::new()
            .side(SheetSide::Bottom)
            .width(300.0)
            .show_close_button(false);
        assert_eq!(c.side, SheetSide::Bottom);
        assert_eq!(c.width, Some(300.0));
        assert!(!c.show_close_button);
        let t = SheetTrigger::new("hi").disabled(true).id_salt("t");
        assert!(t.disabled);
        let _ = SheetClose::new("bye").id_salt("bye");
        let _ = SheetHeader::new();
        let _ = SheetFooter::new();
        let _ = SheetTitle::new("T");
        let _ = SheetDescription::new("D");
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
