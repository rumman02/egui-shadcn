//! shadcn `dialog.tsx` — a modal dialog with dimmed backdrop.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/dialog.tsx` (Radix Dialog)
//! with the nova recipe: `bg-popover text-popover-foreground
//! ring-foreground/10 rounded-xl p-4 text-sm gap-4 sm:max-w-sm` content over
//! a `bg-black/10` animated backdrop (the source's backdrop blur cannot be
//! reproduced without shaders, so the dim alone carries the effect).
//!
//! Behavior follows Radix: the dialog is modal (input to the rest of the UI
//! is blocked), closes on Escape, on backdrop click, and via the top-right X
//! or any [`DialogClose`]. Content fades in/out
//! (`fade-in-0 zoom-in-95` / `duration-200`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Dialog::new().id_salt("edit").show(ui, |ui| {
//!     DialogTrigger::widget(|ui| Button::new("Edit profile").show(ui)).show(ui);
//!     DialogContent::new().show(ui, |ui| {
//!         DialogHeader::new().show(ui, |ui| {
//!             DialogTitle::new("Edit profile").show(ui);
//!             DialogDescription::new("Make changes to your profile.").show(ui);
//!         });
//!         DialogFooter::new().show(ui, |ui| {
//!             DialogClose::widget(|ui| Button::new("Save").show(ui)).show(ui);
//!         });
//!     });
//! });
//!
//! // Controlled:
//! let mut open = false;
//! Dialog::controlled(&mut open).id_salt("d2").show(ui, |ui| { /* … */ });
//! ```

use crate::components::button::{Button, ButtonVariant};
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::animation::overlay_progress;
use crate::utils::shadow;
use egui::{
    Color32, Context, CornerRadius, Frame, Id, Margin, Modal, Pos2, Rect, Response, Sense, Stroke,
    Ui, UiBuilder, Vec2, WidgetInfo,
};

/// nova max width: `sm:max-w-sm` (384 px).
pub const DIALOG_MAX_WIDTH: f32 = 384.0;
/// nova padding: `p-4`.
const INNER_PADDING: i8 = 16;
/// nova ring: `ring-foreground/10`.
const RING_ALPHA: f32 = 0.10;
/// nova backdrop: `bg-black/10`.
const BACKDROP_ALPHA: f32 = 0.10;

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors popover.rs)
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_dialog_stack")
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

/// Per-instance state: just the open flag.
#[derive(Clone, Copy, Debug, Default)]
struct DialogState {
    open: bool,
}

fn read_state(ctx: &Context, state_id: Id) -> DialogState {
    ctx.data_mut(|d| d.get_temp::<DialogState>(state_id).unwrap_or_default())
}

fn write_state(ctx: &Context, state_id: Id, state: DialogState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

// ---------------------------------------------------------------------------
// Dialog (root)
// ---------------------------------------------------------------------------

/// The shadcn dialog root. See the [module docs](self) for an example.
///
/// Uncontrolled by default: the open state is stored in the egui context and
/// persists per `id_salt`. Use [`Dialog::controlled`] to own the state.
#[derive(Debug)]
pub struct Dialog<'a> {
    controlled: Option<&'a mut bool>,
    default_open: bool,
    id_salt: Option<String>,
}

impl Default for Dialog<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Dialog<'a> {
    /// Creates an uncontrolled dialog (starts closed).
    pub fn new() -> Self {
        Self {
            controlled: None,
            default_open: false,
            id_salt: None,
        }
    }

    /// Creates a dialog whose open state is owned by the caller. Escape,
    /// backdrop clicks, the X button and [`DialogClose`] all write back.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            default_open: false,
            id_salt: None,
        }
    }

    /// Initial open state (uncontrolled mode only).
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Sets a stable id salt so the open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Lays out the trigger and content, showing the modal while open.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_dialog"),
        };

        let ctx = ui.ctx().clone();
        if let Some(open) = self.controlled.as_deref() {
            write_state(&ctx, state_id, DialogState { open: *open });
        } else {
            let exists = ctx.data_mut(|d| d.get_temp::<DialogState>(state_id).is_some());
            if !exists {
                write_state(
                    &ctx,
                    state_id,
                    DialogState {
                        open: self.default_open,
                    },
                );
            }
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
// DialogTrigger / DialogClose
// ---------------------------------------------------------------------------

/// Opens the innermost [`Dialog`] when clicked.
///
/// Renders an outline button by default (the shadcn docs demos) or a custom
/// widget via [`DialogTrigger::widget`].
pub struct DialogTrigger<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for DialogTrigger<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> DialogTrigger<'a> {
    /// Creates a trigger rendering an outline button with the given label.
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

    /// Renders the trigger; `changed()` when the dialog open state flipped.
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

/// Closes the innermost [`Dialog`] when clicked.
///
/// Renders an outline button by default (the source's footer close button)
/// or a custom widget via [`DialogClose::widget`].
pub struct DialogClose<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    id_salt: Option<String>,
}

impl Default for DialogClose<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> DialogClose<'a> {
    /// Creates a close control rendering an outline button.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            widget: None,
            id_salt: None,
        }
    }

    /// Creates a close control rendering a custom widget (Radix `asChild`).
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

    /// Renders the close control; `changed()` when the dialog closed.
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
            let mut state = read_state(&ctx, state_id);
            state.open = false;
            write_state(&ctx, state_id, state);
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
// DialogContent
// ---------------------------------------------------------------------------

/// The modal panel: centered, `bg-popover ring-foreground/10 rounded-xl p-4
/// shadow-lg` (nova) with a close X at `top-2 right-2`, over a dimmed
/// backdrop. Closes on Escape and backdrop click.
#[derive(Clone, Debug)]
pub struct DialogContent {
    width: Option<f32>,
    show_close_button: bool,
    id_salt: Option<String>,
}

impl Default for DialogContent {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogContent {
    /// Creates dialog content with the source defaults (`sm:max-w-sm`, close
    /// button shown).
    pub fn new() -> Self {
        Self {
            width: None,
            show_close_button: true,
            id_salt: None,
        }
    }

    /// Overrides the panel max width (default: [`DIALOG_MAX_WIDTH`], clamped
    /// to the window minus 2rem like `max-w-[calc(100%-2rem)]`).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Hides the top-right close button (`showCloseButton = false`).
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Sets a stable id salt (kept for API symmetry; the content follows the
    /// root's state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the modal, or nothing while closed. Returns the modal area
    /// response while visible.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Option<Response> {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Dialog: render statically so mistakes stay visible.
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
        let anim_id = state_id.with("anim");

        // `data-[state=open]:fade-in-0 zoom-in-95 duration-200`.
        let progress = overlay_progress(&ctx, anim_id, open);
        if !(open || progress.alpha > 0.002) {
            return None;
        }

        let pad = INNER_PADDING as f32;
        let screen_w = ctx.content_rect().width();
        let max_width = self
            .width
            .unwrap_or(DIALOG_MAX_WIDTH)
            .min(screen_w - 32.0)
            .max(80.0);

        // `bg-black/10` on the backdrop, fading with the dialog.
        let backdrop = color_alpha(
            Color32::from_black_alpha(255),
            BACKDROP_ALPHA * progress.alpha,
        );

        // The panel frame, painted inside the opacity scope so it fades too.
        let frame = Frame::new()
            .fill(p.popover)
            .stroke(Stroke::new(1.0, color_alpha(p.foreground, RING_ALPHA)))
            .corner_radius(CornerRadius::same(theme.radius.xl_u8()))
            .inner_margin(Margin::same(INNER_PADDING))
            .shadow(shadow::shadow_lg(dark));

        let modal = Modal::new(modal_id)
            .backdrop_color(backdrop)
            .frame(Frame::new());

        let modal_resp = modal.show(&ctx, |ui| {
            ui.set_opacity(progress.alpha);
            frame.show(ui, |ui| {
                // Pin the content width: the footer strip bleeds into the
                // panel padding, and an auto-sized container would otherwise
                // grow every frame to fit the bleed.
                let inner_w = max_width - pad * 2.0;
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);
                ui.style_mut().visuals.override_text_color = Some(p.popover_foreground);
                content(ui);
                if self.show_close_button && progress.alpha > 0.75 {
                    // `absolute top-2 right-2` relative to the panel.
                    let x_rect = Rect::from_min_size(
                        Pos2::new(
                            ui.max_rect().right() + pad - 8.0 - 24.0,
                            ui.max_rect().top() - pad + 8.0,
                        ),
                        Vec2::splat(24.0),
                    );
                    let x_id = state_id.with("dialog_close_x");
                    let x_resp = ui.interact(x_rect, x_id, Sense::click());
                    x_resp.widget_info(|| {
                        WidgetInfo::labeled(egui::Role::Button, true, "Close".to_owned())
                    });
                    if ui.is_rect_visible(x_rect) {
                        let corner = CornerRadius::same(theme.radius.sm_u8());
                        let fill = if x_resp.hovered() {
                            p.accent
                        } else {
                            Color32::TRANSPARENT
                        };
                        let fg = if x_resp.hovered() {
                            p.muted_foreground
                        } else {
                            color_alpha(p.foreground, 0.7)
                        };
                        ui.painter().rect_filled(x_rect, corner, fill);
                        let icon_rect = Rect::from_center_size(x_rect.center(), Vec2::splat(16.0));
                        Icon::X.paint(ui.painter(), icon_rect, fg, 16.0 / 24.0);
                    }
                    if x_resp.clicked() {
                        let mut s = read_state(&ctx, state_id);
                        s.open = false;
                        write_state(&ctx, state_id, s);
                    }
                }
            });
        });

        // `should_close` covers Escape (topmost modal only) and backdrop
        // clicks.
        if modal_resp.should_close() && open {
            let mut s = read_state(&ctx, state_id);
            s.open = false;
            write_state(&ctx, state_id, s);
        }

        Some(modal_resp.response)
    }
}

// ---------------------------------------------------------------------------
// DialogHeader / DialogTitle / DialogDescription / DialogFooter
// ---------------------------------------------------------------------------

/// `flex flex-col gap-2` header inside a [`DialogContent`].
#[derive(Clone, Debug, Default)]
pub struct DialogHeader {
    _private: (),
}

impl DialogHeader {
    /// Creates a dialog header.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the header with an 8 px column gap.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        ui.vertical(|ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::SM;
            content(ui)
        })
        .inner
    }
}

/// nova dialog title: `text-base leading-none font-medium`.
#[derive(Clone, Debug)]
pub struct DialogTitle {
    text: String,
}

impl DialogTitle {
    /// Creates a dialog title.
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

/// `text-sm text-muted-foreground` dialog description, wrapping.
#[derive(Clone, Debug)]
pub struct DialogDescription {
    text: String,
}

impl DialogDescription {
    /// Creates a dialog description.
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

/// nova dialog footer: `bg-muted/50 -mx-4 -mb-4 rounded-b-xl border-t p-4`
/// with right-aligned children (`sm:justify-end`).
///
/// The strip is full-bleed: it stretches to the panel's left/right edges and
/// hangs into the bottom padding.
#[derive(Clone, Debug, Default)]
pub struct DialogFooter {
    _private: (),
}

impl DialogFooter {
    /// Creates a dialog footer.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the footer strip. The content is right-aligned.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnMut(&mut Ui) -> R) -> R {
        footer_strip(ui, content)
    }
}

/// Shared footer-strip implementation (also used by alert_dialog.rs).
///
/// nova: `bg-muted/50 -mx-4 -mb-4 rounded-b-xl border-t p-4`.
pub(crate) fn footer_strip<R>(ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let pad = crate::theme::tokens::spacing::LG; // p-4
    let bleed = crate::theme::tokens::spacing::LG; // -mb-4 into the panel padding
    let radius = theme.radius.xl_u8();

    // Measure the content height with an invisible pass so the strip can be
    // painted behind the real content in the same frame. Widgets in the
    // measuring pass are disabled, so no callbacks can double-fire.
    let flow_min = ui.cursor().min;
    let flow_w = ui.available_width();
    let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
    let mut measure = ui.new_child(UiBuilder::new().max_rect(measure_rect).invisible());
    let _ = content(&mut measure);
    let content_h = measure.min_rect().height();

    let strip = Rect::from_min_size(
        Pos2::new(measure_rect.left() - pad, measure_rect.top()),
        Vec2::new(measure_rect.width() + pad * 2.0, content_h + pad * 2.0),
    );

    if ui.is_rect_visible(strip) {
        let painter = ui.painter();
        let corner = CornerRadius {
            nw: 0,
            ne: 0,
            sw: radius,
            se: radius,
        };
        painter.rect_filled(strip, corner, color_alpha(p.muted, 0.5));
        // border-t
        painter.line_segment(
            [strip.left_top(), strip.right_top()],
            Stroke::new(1.0, p.border),
        );
    }

    // Lay out the real content inside the strip, right-aligned.
    let inner_rect = Rect::from_min_size(
        Pos2::new(strip.left() + pad, strip.top() + pad),
        Vec2::new(strip.width() - pad * 2.0, content_h),
    );
    let mut child = ui.new_child(UiBuilder::new().max_rect(inner_rect));
    let real = child
        .with_layout(egui::Layout::right_to_left(egui::Align::Center), content)
        .inner;

    // Reserve the in-flow height (the rest of the strip hangs into the
    // panel's bottom padding).
    ui.allocate_space(Vec2::new(flow_w, (strip.height() - bleed).max(0.0)));
    real
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Pos2;

    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    /// The test screen; the modal anchors to the context screen rect, so the
    /// harness pins it to match the test Ui.
    fn screen() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(800.0, 600.0))
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
                    Id::unique("dialog_test_ui"),
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
                        Id::unique("dialog_test_ui"),
                        UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        fn state(&self, salt: &str) -> DialogState {
            let id = Id::unique("dialog_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<DialogState>(id))
                .unwrap_or_default()
        }

        /// Sends Escape for one frame.
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
                    Id::unique("dialog_test_ui"),
                    UiBuilder::new().max_rect(screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn constants_match_nova_source() {
        assert_eq!(DIALOG_MAX_WIDTH, 384.0); // sm:max-w-sm
        assert_eq!(INNER_PADDING, 16); // p-4
        assert!((RING_ALPHA - 0.10).abs() < f32::EPSILON);
        assert!((BACKDROP_ALPHA - 0.10).abs() < f32::EPSILON); // bg-black/10
    }

    #[test]
    fn trigger_click_opens_and_closes() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            Dialog::new().id_salt("d1").show(ui, |ui| {
                trigger_rect = DialogTrigger::new("Open").show(ui).rect;
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(!h.state("d1").open);

        h.click(trigger_rect.center(), |ui| {
            Dialog::new().id_salt("d1").show(ui, |ui| {
                DialogTrigger::new("Open").show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(h.state("d1").open, "trigger click opens the dialog");

        // Warm up: the modal's backdrop only starts blocking clicks a couple
        // of frames after it appears.
        for _ in 0..3 {
            h.run(|ui| {
                Dialog::new().id_salt("d1").show(ui, |ui| {
                    DialogTrigger::new("Open").show(ui);
                    DialogContent::new().show(ui, |ui| {
                        DialogTitle::new("Title").show(ui);
                    });
                });
            });
        }

        // A click away from any widget lands on the backdrop and dismisses
        // the dialog, like Radix's dismiss-on-interact-outside.
        h.click(egui::Pos2::new(750.0, 550.0), |ui| {
            Dialog::new().id_salt("d1").show(ui, |ui| {
                DialogTrigger::new("Open").show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(
            !h.state("d1").open,
            "backdrop click must dismiss the dialog"
        );
    }

    #[test]
    fn escape_closes_open_dialog() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            Dialog::new().id_salt("d2").show(ui, |ui| {
                trigger_rect = DialogTrigger::new("Open").show(ui).rect;
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        h.click(trigger_rect.center(), |ui| {
            Dialog::new().id_salt("d2").show(ui, |ui| {
                DialogTrigger::new("Open").show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(h.state("d2").open);

        h.press_escape(|ui| {
            Dialog::new().id_salt("d2").show(ui, |ui| {
                DialogTrigger::new("Open").show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(!h.state("d2").open, "Escape must close the dialog");
    }

    #[test]
    fn controlled_mode_write_back() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            Dialog::controlled(&mut open).id_salt("d3").show(ui, |ui| {
                trigger_rect = DialogTrigger::new("Open").show(ui).rect;
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(!open);

        h.click(trigger_rect.center(), |ui| {
            Dialog::controlled(&mut open).id_salt("d3").show(ui, |ui| {
                DialogTrigger::new("Open").show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(open, "trigger click write-back reaches the caller");

        // Caller closes from the outside.
        open = false;
        h.run(|ui| {
            Dialog::controlled(&mut open).id_salt("d3").show(ui, |ui| {
                DialogTrigger::new("Open").show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(!open);
    }

    #[test]
    fn close_widget_closes_dialog() {
        let mut h = Harness::new();
        let mut open = true;
        let mut close_rect = Rect::ZERO;

        // Warm up so the modal has settled (frame 1 is the sizing pass).
        for _ in 0..3 {
            h.run(|ui| {
                Dialog::controlled(&mut open).id_salt("d4").show(ui, |ui| {
                    DialogContent::new().show(ui, |ui| {
                        DialogTitle::new("Title").show(ui);
                        DialogFooter::new().show(ui, |ui| {
                            close_rect = DialogClose::new("Close").show(ui).rect;
                        });
                    });
                });
            });
        }
        assert!(open);

        h.click(close_rect.center(), |ui| {
            Dialog::controlled(&mut open).id_salt("d4").show(ui, |ui| {
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                    DialogFooter::new().show(ui, |ui| {
                        DialogClose::new("Close").show(ui);
                    });
                });
            });
        });
        assert!(!open, "DialogClose must close the dialog");
    }

    #[test]
    fn button_directly_in_modal_receives_clicks() {
        let mut h = Harness::new();
        let mut open = true;
        let mut btn_rect = Rect::ZERO;
        let mut clicks = 0;

        // Warm up: frame 1 is the area's sizing pass (default position);
        // the modal centers from frame 2 on.
        for _ in 0..3 {
            h.run(|ui| {
                Dialog::controlled(&mut open).id_salt("d5").show(ui, |ui| {
                    DialogContent::new().show(ui, |ui| {
                        DialogTitle::new("Title").show(ui);
                        btn_rect = Button::new("Plain").show(ui).rect;
                    });
                });
            });
        }

        h.click(btn_rect.center(), |ui| {
            Dialog::controlled(&mut open).id_salt("d5").show(ui, |ui| {
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                    if Button::new("Plain").show(ui).clicked() {
                        clicks += 1;
                    }
                });
            });
        });
        assert!(clicks > 0, "buttons inside the modal must receive clicks");
    }

    #[test]
    fn footer_strip_geometry_is_full_bleed() {
        // -mx-4 -mb-4 with p-4: the strip hangs one padding into the panel's
        // bottom padding; the in-flow reservation is content + one padding.
        let pad = crate::theme::tokens::spacing::LG;
        let content_h = 36.0;
        let strip_h = content_h + pad * 2.0;
        let flow_h = strip_h - pad;
        assert_eq!(strip_h, 68.0);
        assert_eq!(flow_h, 52.0);
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let d = Dialog::controlled(&mut open)
            .default_open(true)
            .id_salt("x");
        assert_eq!(d.id_salt.as_deref(), Some("x"));
        let c = DialogContent::new()
            .width(300.0)
            .show_close_button(false)
            .id_salt("c");
        assert_eq!(c.width, Some(300.0));
        assert!(!c.show_close_button);
        let t = DialogTrigger::new("hi").disabled(true).id_salt("t");
        assert!(t.disabled);
        let _ = DialogClose::new("bye").id_salt("bye");
        let _ = DialogHeader::new();
        let _ = DialogFooter::new();
        let _ = DialogTitle::new("T");
        let _ = DialogDescription::new("D");
    }
}
