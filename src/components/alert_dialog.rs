//! shadcn `alert-dialog.tsx` — a modal confirmation dialog.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/alert-dialog.tsx` (Radix
//! AlertDialog) with the nova recipe: `bg-popover text-popover-foreground
//! ring-foreground/10 rounded-xl p-4 gap-4 ring-1 duration-100` with sizes
//! `default → sm:max-w-sm` (384 px) and `sm → max-w-xs` (320 px), a
//! `bg-muted/50` full-bleed footer strip, and an optional
//! `bg-muted size-10 rounded-md` media slot.
//!
//! Radix behavior: unlike [`crate::components::dialog::Dialog`], an alert
//! dialog has **no close X** and **backdrop clicks do not dismiss it**; only
//! the Cancel/Action buttons or Escape close it — and Escape closes *without*
//! confirming the action.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! AlertDialog::new().id_salt("delete").show(ui, |ui| {
//!     AlertDialogTrigger::widget(|ui| {
//!         Button::new("Delete").variant(ButtonVariant::Destructive).show(ui)
//!     })
//!     .show(ui);
//!     AlertDialogContent::new().show(ui, |ui| {
//!         AlertDialogHeader::new().show(ui, |ui| {
//!             AlertDialogTitle::new("Delete this file?").show(ui);
//!             AlertDialogDescription::new("This cannot be undone.").show(ui);
//!         });
//!         AlertDialogFooter::new().show(ui, |ui| {
//!             if AlertDialogCancel::new("Cancel").show(ui).clicked() {
//!                 // dismissed
//!             }
//!             if AlertDialogAction::new("Delete").show(ui).clicked() {
//!                 // confirmed — do the destructive thing
//!             }
//!         });
//!     });
//! });
//! ```

use crate::components::button::{Button, ButtonVariant};
use crate::components::dialog::footer_strip;
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::animation::nova_overlay_progress;
use egui::{
    Align, Color32, Context, CornerRadius, Frame, Id, Key, Layout, Margin, Modal, Modifiers, Pos2,
    Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2,
};

/// nova `data-[size=default]:sm:max-w-sm` (384 px).
pub const ALERT_DIALOG_MAX_WIDTH: f32 = 384.0;
/// nova `data-[size=sm]:max-w-xs` (320 px).
pub const ALERT_DIALOG_SM_WIDTH: f32 = 320.0;
/// nova padding: `p-4`.
const INNER_PADDING: i8 = 16;
/// nova ring: `ring-foreground/10`.
const RING_ALPHA: f32 = 0.10;
/// nova backdrop: `bg-black/10`.
const BACKDROP_ALPHA: f32 = 0.10;

/// `data-size` presets on the content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertDialogSize {
    /// Left-aligned text, 384 px max width (the source default).
    #[default]
    Default,
    /// Centered text, 320 px max width.
    Sm,
}

impl AlertDialogSize {
    /// Max content width in points.
    pub fn width(self) -> f32 {
        match self {
            AlertDialogSize::Default => ALERT_DIALOG_MAX_WIDTH,
            AlertDialogSize::Sm => ALERT_DIALOG_SM_WIDTH,
        }
    }
}

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors dialog.rs)
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_alert_dialog_stack")
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
struct AlertDialogState {
    open: bool,
    size: AlertDialogSize,
}

fn read_state(ctx: &Context, state_id: Id) -> AlertDialogState {
    ctx.data_mut(|d| d.get_temp::<AlertDialogState>(state_id).unwrap_or_default())
}

fn write_state(ctx: &Context, state_id: Id, state: AlertDialogState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

fn set_open(ctx: &Context, state_id: Id, open: bool) {
    let mut state = read_state(ctx, state_id);
    state.open = open;
    write_state(ctx, state_id, state);
}

// ---------------------------------------------------------------------------
// AlertDialog (root)
// ---------------------------------------------------------------------------

/// The shadcn alert-dialog root. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct AlertDialog<'a> {
    controlled: Option<&'a mut bool>,
    id_salt: Option<String>,
}

impl Default for AlertDialog<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> AlertDialog<'a> {
    /// Creates an uncontrolled alert dialog (starts closed).
    pub fn new() -> Self {
        Self {
            controlled: None,
            id_salt: None,
        }
    }

    /// Creates an alert dialog whose open state is owned by the caller.
    /// Cancel, Action and Escape all write back.
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

    /// Lays out the trigger and content, showing the modal while open.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_alert_dialog"),
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
// AlertDialogTrigger / AlertDialogAction / AlertDialogCancel
// ---------------------------------------------------------------------------

/// Opens the innermost [`AlertDialog`] when clicked. Renders an outline
/// button by default or a custom widget via [`AlertDialogTrigger::widget`].
pub struct AlertDialogTrigger<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for AlertDialogTrigger<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> AlertDialogTrigger<'a> {
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

/// The confirm button (`variant = "default"`): closes the dialog when
/// clicked. Check the returned response's `clicked()` to run the action.
pub struct AlertDialogAction<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    variant: ButtonVariant,
    id_salt: Option<String>,
}

impl Default for AlertDialogAction<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> AlertDialogAction<'a> {
    /// Creates an action button with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            widget: None,
            variant: ButtonVariant::Default,
            id_salt: None,
        }
    }

    /// Creates an action rendering a custom widget (Radix `asChild`).
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w)),
            variant: ButtonVariant::Default,
            id_salt: None,
        }
    }

    /// Overrides the button variant (default: [`ButtonVariant::Default`]).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the action; `clicked()` also dismisses the dialog.
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(state_id) = current_scope(ui) else {
            return action_button(&self.label, self.variant, self.id_salt.clone()).show(ui);
        };

        let mut response = match self.widget {
            Some(w) => w(ui),
            None => action_button(&self.label, self.variant, self.id_salt.clone()).show(ui),
        };

        if response.clicked() {
            let ctx = ui.ctx().clone();
            set_open(&ctx, state_id, false);
            response.mark_changed();
        }
        response
    }
}

/// The dismiss button (`variant = "outline"`): closes the dialog without
/// confirming.
pub struct AlertDialogCancel<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    variant: ButtonVariant,
    id_salt: Option<String>,
}

impl Default for AlertDialogCancel<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> AlertDialogCancel<'a> {
    /// Creates a cancel button with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            widget: None,
            variant: ButtonVariant::Outline,
            id_salt: None,
        }
    }

    /// Creates a cancel control rendering a custom widget.
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w)),
            variant: ButtonVariant::Outline,
            id_salt: None,
        }
    }

    /// Overrides the button variant (default: [`ButtonVariant::Outline`]).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the cancel button; clicking dismisses the dialog.
    ///
    /// The rendered button's id is recorded so the content can give it the
    /// initial focus (Radix auto-focuses Cancel on open).
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(state_id) = current_scope(ui) else {
            return action_button(&self.label, self.variant, self.id_salt.clone()).show(ui);
        };

        let mut response = match self.widget {
            Some(w) => w(ui),
            None => action_button(&self.label, self.variant, self.id_salt.clone()).show(ui),
        };

        let ctx = ui.ctx().clone();
        ctx.data_mut(|d| d.insert_temp(state_id.with("cancel_resp"), response.id));

        if response.clicked() {
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

fn action_button(
    label: &Option<String>,
    variant: ButtonVariant,
    id_salt: Option<String>,
) -> Button {
    let mut b = Button::new(label.clone().unwrap_or_default()).variant(variant);
    if let Some(salt) = id_salt {
        b = b.id_salt(salt);
    }
    b
}

// ---------------------------------------------------------------------------
// AlertDialogContent
// ---------------------------------------------------------------------------

/// The modal panel: centered, `bg-popover ring-foreground/10 rounded-xl p-4`
/// (nova, ring-only), sizes `default`/`sm`. No close X; backdrop clicks are
/// ignored; Escape dismisses without confirming. Opening auto-focuses the
/// Cancel button, like Radix.
#[derive(Clone, Debug)]
pub struct AlertDialogContent {
    size: AlertDialogSize,
    id_salt: Option<String>,
}

impl Default for AlertDialogContent {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertDialogContent {
    /// Creates alert-dialog content (source default `size = "default"`).
    pub fn new() -> Self {
        Self {
            size: AlertDialogSize::default(),
            id_salt: None,
        }
    }

    /// Sets the content size preset.
    pub fn size(mut self, size: AlertDialogSize) -> Self {
        self.size = size;
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
            // Outside an AlertDialog: render statically so mistakes stay
            // visible.
            content(ui);
            return None;
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        state.size = self.size;
        let open = state.open;
        write_state(&ctx, state_id, state);

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let modal_id = state_id.with("modal");
        let anim_id = state_id.with("anim");

        // nova `duration-100`: fade in/out at 100ms in both directions.
        let progress = nova_overlay_progress(&ctx, anim_id, open);
        if !(open || progress.alpha > 0.002) {
            return None;
        }

        let pad = INNER_PADDING as f32;
        let screen_w = ctx.content_rect().width();
        // nova `data-[size=default]:max-w-xs` below the 640px breakpoint.
        let size_width = if screen_w < 640.0 {
            AlertDialogSize::Sm.width()
        } else {
            self.size.width()
        };
        let max_width = size_width.min(screen_w - 32.0).max(80.0);

        // `bg-black/10` on the backdrop, fading with the dialog.
        let backdrop = color_alpha(
            Color32::from_black_alpha(255),
            BACKDROP_ALPHA * progress.alpha,
        );

        // nova `.cn-alert-dialog-content` is ring-only: no shadow.
        let frame = Frame::new()
            .fill(p.popover)
            .stroke(Stroke::new(1.0, color_alpha(p.foreground, RING_ALPHA)))
            .corner_radius(CornerRadius::same(theme.radius.xl_u8()))
            .inner_margin(Margin::same(INNER_PADDING));

        // Radix auto-focuses the cancel button when the dialog opens; the
        // guard makes it fire once per open, and resets while closed.
        let focus_guard = state_id.with("cancel_focus_done");
        let already_focused = ctx.data_mut(|d| d.get_temp::<bool>(focus_guard).unwrap_or(false));
        if !open {
            ctx.data_mut(|d| d.insert_temp(focus_guard, false));
        }

        let modal = Modal::new(modal_id)
            .backdrop_color(backdrop)
            .frame(Frame::new());

        let modal_resp = modal.show(&ctx, |ui| {
            ui.set_opacity(progress.alpha);
            frame.show(ui, |ui| {
                // Pin the content width so the footer bleed doesn't grow the
                // container (see dialog.rs).
                let inner_w = max_width - pad * 2.0;
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);
                ui.style_mut().visuals.override_text_color = Some(p.popover_foreground);
                // nova `.cn-alert-dialog-content`: `grid gap-4` column flow.
                ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::LG;
                content(ui);
                if open && !already_focused {
                    ctx.data_mut(|d| d.insert_temp(focus_guard, true));
                    if let Some(cancel_id) =
                        ctx.data_mut(|d| d.get_temp::<Id>(state_id.with("cancel_resp")))
                    {
                        ctx.memory_mut(|m| m.request_focus(cancel_id));
                    }
                }
            });
        });

        // Escape dismisses without confirming (topmost modal only). Unlike
        // Dialog, backdrop clicks do NOT dismiss an alert dialog, so
        // `should_close()` (which includes the backdrop) is deliberately not
        // used here.
        let esc = modal_resp.is_top_modal
            && !modal_resp.any_popup_open
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if esc && open {
            set_open(&ctx, state_id, false);
        }

        Some(modal_resp.response)
    }
}

// ---------------------------------------------------------------------------
// Header / Title / Description / Media / Footer
// ---------------------------------------------------------------------------

/// `grid place-items-center gap-1.5 text-center` header; centered for the
/// `sm` size, left-aligned for `default` (per the nova
/// `sm:group-data-[size=default]:place-items-start` selectors).
#[derive(Clone, Copy, Debug, Default)]
pub struct AlertDialogHeader {
    _private: (),
}

impl AlertDialogHeader {
    /// Creates an alert-dialog header.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the header, aligning against the active content size.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let size = current_scope(ui)
            .map(|id| read_state(ui.ctx(), id).size)
            .unwrap_or_default();
        let align = match size {
            AlertDialogSize::Default => Align::LEFT,
            AlertDialogSize::Sm => Align::Center,
        };
        ui.with_layout(Layout::top_down(align), |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::XS_HALF;
            content(ui)
        })
        .inner
    }
}

/// nova alert-dialog title: `text-base font-medium`.
#[derive(Clone, Debug)]
pub struct AlertDialogTitle {
    text: String,
}

impl AlertDialogTitle {
    /// Creates an alert-dialog title.
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

/// `text-sm text-muted-foreground` description, wrapping.
#[derive(Clone, Debug)]
pub struct AlertDialogDescription {
    text: String,
}

impl AlertDialogDescription {
    /// Creates an alert-dialog description.
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

/// nova `bg-muted size-10 rounded-md` media slot with an icon
/// (`*:[svg]:size-6`).
#[derive(Clone, Copy, Debug)]
pub struct AlertDialogMedia {
    icon: Icon,
}

impl AlertDialogMedia {
    /// Creates a media slot showing the given icon.
    pub fn new(icon: Icon) -> Self {
        Self { icon }
    }

    /// Renders the media box.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let side = 40.0;
        let icon_side = 24.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // nova: the header grid is `place-items-start` for the `default`
            // size on desktop; only `sm` stays centered.
            let size = current_scope(ui)
                .map(|id| read_state(ui.ctx(), id).size)
                .unwrap_or_default();
            let rect = match size {
                AlertDialogSize::Default => rect,
                AlertDialogSize::Sm => {
                    let center_x = ui.max_rect().center().x;
                    Rect::from_center_size(
                        Pos2::new(center_x.max(rect.center().x), rect.center().y),
                        Vec2::splat(side),
                    )
                }
            };
            painter.rect_filled(rect, theme.radius.md_u8(), p.muted);
            let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_side));
            self.icon
                .paint(painter, icon_rect, p.popover_foreground, icon_side / 24.0);
        }
        response
    }
}

/// nova alert-dialog footer: same full-bleed `bg-muted/50 -mx-4 -mb-4
/// rounded-b-xl border-t p-4` strip as the dialog footer.
#[derive(Clone, Copy, Debug, Default)]
pub struct AlertDialogFooter {
    _private: (),
}

impl AlertDialogFooter {
    /// Creates an alert-dialog footer.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the footer strip. For the `sm` size the content fills a
    /// 2-column grid (Cancel left, Action right); otherwise it is
    /// right-aligned.
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        let size = current_scope(ui)
            .map(|id| read_state(ui.ctx(), id).size)
            .unwrap_or_default();
        match size {
            AlertDialogSize::Default => footer_strip(ui, content),
            AlertDialogSize::Sm => {
                // Source `group-data-[size=sm]:grid grid-cols-2`: two
                // equal-width cells (Cancel left, Action right). An egui
                // layout cannot split an opaque closure across cells, so the
                // row is spread across the full inner width instead: the
                // measured natural row width pins the first child to the
                // left edge and the last to the right edge, matching the
                // grid's outer geometry (more than two children simply
                // spread tighter, without grid row wrapping).
                let theme = ShadcnTheme::of_ui(ui);
                let p = theme.palette;
                let pad = crate::theme::tokens::spacing::LG;
                let bleed = pad;
                let radius = theme.radius.xl_u8();
                let gap_x = ui.spacing().item_spacing.x;

                let flow_min = ui.cursor().min;
                let flow_w = ui.available_width();
                let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
                let mut measure = ui.new_child(
                    UiBuilder::new()
                        .max_rect(measure_rect)
                        .layout(Layout::left_to_right(Align::Center))
                        .invisible(),
                );
                let _ = content(&mut measure);
                let content_h = measure.min_rect().height();
                let natural_w = (measure.min_rect().width() - gap_x).max(0.0);

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
                    painter.line_segment(
                        [strip.left_top(), strip.right_top()],
                        Stroke::new(1.0, p.border),
                    );
                }
                let inner_rect = Rect::from_min_size(
                    Pos2::new(strip.left() + pad, strip.top() + pad),
                    Vec2::new(strip.width() - pad * 2.0, content_h),
                );
                let spread = (inner_rect.width() - natural_w).max(0.0);
                let mut child = ui.new_child(UiBuilder::new().max_rect(inner_rect));
                let real = child
                    .with_layout(Layout::left_to_right(Align::Center), |c| {
                        c.spacing_mut().item_spacing.x = spread;
                        content(c)
                    })
                    .inner;
                ui.allocate_space(Vec2::new(flow_w, (strip.height() - bleed).max(0.0)));
                real
            }
        }
    }
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
                    Id::unique("alert_dialog_test_ui"),
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
                        Id::unique("alert_dialog_test_ui"),
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
                key: Key::Escape,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
                physical_key: None,
            }]);
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("alert_dialog_test_ui"),
                    UiBuilder::new().max_rect(screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> AlertDialogState {
            let id = Id::unique("alert_dialog_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<AlertDialogState>(id))
                .unwrap_or_default()
        }
    }

    #[test]
    fn constants_match_nova_source() {
        assert_eq!(ALERT_DIALOG_MAX_WIDTH, 384.0); // sm:max-w-sm
        assert_eq!(ALERT_DIALOG_SM_WIDTH, 320.0); // max-w-xs
        assert_eq!(AlertDialogSize::Default.width(), 384.0);
        assert_eq!(AlertDialogSize::Sm.width(), 320.0);
        assert_eq!(INNER_PADDING, 16); // p-4
        assert!((BACKDROP_ALPHA - 0.10).abs() < f32::EPSILON); // bg-black/10
    }

    #[test]
    fn trigger_opens_and_cancel_dismisses() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;
        let mut cancel_rect = Rect::ZERO;

        // Warm up so the modal settles, then capture rects.
        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("a1")
                    .show(ui, |ui| {
                        trigger_rect = AlertDialogTrigger::new("Delete").show(ui).rect;
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogTitle::new("Delete?").show(ui);
                            AlertDialogFooter::new().show(ui, |ui| {
                                cancel_rect = AlertDialogCancel::new("Cancel").show(ui).rect;
                            });
                        });
                    });
            });
        }
        assert!(!open);

        h.click(trigger_rect.center(), |ui| {
            AlertDialog::controlled(&mut open)
                .id_salt("a1")
                .show(ui, |ui| {
                    AlertDialogTrigger::new("Delete").show(ui);
                    AlertDialogContent::new().show(ui, |ui| {
                        AlertDialogTitle::new("Delete?").show(ui);
                        AlertDialogFooter::new().show(ui, |ui| {
                            AlertDialogCancel::new("Cancel").show(ui);
                        });
                    });
                });
        });
        assert!(open, "trigger click opens the alert dialog");

        // Warm up so the modal's backdrop is fully registered (it starts
        // blocking clicks a couple of frames after appearing).
        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("a1")
                    .show(ui, |ui| {
                        AlertDialogTrigger::new("Delete").show(ui);
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogTitle::new("Delete?").show(ui);
                            AlertDialogFooter::new().show(ui, |ui| {
                                AlertDialogCancel::new("Cancel").show(ui);
                            });
                        });
                    });
            });
        }

        // A click away from any widget lands on the backdrop, which must NOT
        // dismiss an alert dialog (Radix behavior).
        h.click(egui::Pos2::new(750.0, 550.0), |ui| {
            AlertDialog::controlled(&mut open)
                .id_salt("a1")
                .show(ui, |ui| {
                    AlertDialogTrigger::new("Delete").show(ui);
                    AlertDialogContent::new().show(ui, |ui| {
                        AlertDialogTitle::new("Delete?").show(ui);
                        AlertDialogFooter::new().show(ui, |ui| {
                            AlertDialogCancel::new("Cancel").show(ui);
                        });
                    });
                });
        });
        assert!(
            h.state("a1").open,
            "backdrop clicks must NOT dismiss an alert dialog"
        );

        // Let the pointer settle after the backdrop click.
        for _ in 0..2 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("a1")
                    .show(ui, |ui| {
                        AlertDialogTrigger::new("Delete").show(ui);
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogTitle::new("Delete?").show(ui);
                            AlertDialogFooter::new().show(ui, |ui| {
                                cancel_rect = AlertDialogCancel::new("Cancel").show(ui).rect;
                            });
                        });
                    });
            });
        }

        h.click(cancel_rect.center(), |ui| {
            AlertDialog::controlled(&mut open)
                .id_salt("a1")
                .show(ui, |ui| {
                    AlertDialogTrigger::new("Delete").show(ui);
                    AlertDialogContent::new().show(ui, |ui| {
                        AlertDialogTitle::new("Delete?").show(ui);
                        AlertDialogFooter::new().show(ui, |ui| {
                            AlertDialogCancel::new("Cancel").show(ui);
                        });
                    });
                });
        });
        assert!(!open, "Cancel must dismiss the dialog");
    }

    #[test]
    fn action_confirms_and_closes() {
        let mut h = Harness::new();
        let mut open = true;
        let mut action_rect = Rect::ZERO;
        let mut confirmations = 0;

        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("a2")
                    .show(ui, |ui| {
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogTitle::new("Delete?").show(ui);
                            AlertDialogFooter::new().show(ui, |ui| {
                                action_rect = AlertDialogAction::new("Delete").show(ui).rect;
                            });
                        });
                    });
            });
        }

        h.click(action_rect.center(), |ui| {
            AlertDialog::controlled(&mut open)
                .id_salt("a2")
                .show(ui, |ui| {
                    AlertDialogContent::new().show(ui, |ui| {
                        AlertDialogTitle::new("Delete?").show(ui);
                        AlertDialogFooter::new().show(ui, |ui| {
                            if AlertDialogAction::new("Delete").show(ui).clicked() {
                                confirmations += 1;
                            }
                        });
                    });
                });
        });
        assert!(confirmations > 0, "Action click must be observable");
        assert!(!open, "Action click must close the dialog");
    }

    #[test]
    fn escape_closes_without_confirming() {
        let mut h = Harness::new();
        let mut open = true;
        let mut confirmations = 0;

        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("a3")
                    .show(ui, |ui| {
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogFooter::new().show(ui, |ui| {
                                AlertDialogAction::new("Delete").show(ui);
                            });
                        });
                    });
            });
        }

        h.press_escape(|ui| {
            AlertDialog::controlled(&mut open)
                .id_salt("a3")
                .show(ui, |ui| {
                    AlertDialogContent::new().show(ui, |ui| {
                        AlertDialogFooter::new().show(ui, |ui| {
                            if AlertDialogAction::new("Delete").show(ui).clicked() {
                                confirmations += 1;
                            }
                        });
                    });
                });
        });
        assert!(!open, "Escape must close the alert dialog");
        assert_eq!(confirmations, 0, "Escape must not confirm the action");
    }

    #[test]
    fn opening_focuses_the_cancel_button() {
        let mut h = Harness::new();
        let mut open = false;
        let mut cancel_id: Option<Id> = None;

        // Closed warm-up frame; the content closure does not run while
        // closed, so nothing is focused.
        h.run(|ui| {
            AlertDialog::controlled(&mut open)
                .id_salt("a4")
                .show(ui, |ui| {
                    AlertDialogContent::new().show(ui, |ui| {
                        AlertDialogFooter::new().show(ui, |ui| {
                            cancel_id = Some(AlertDialogCancel::new("Cancel").show(ui).id);
                        });
                    });
                });
        });
        assert!(cancel_id.is_none(), "closed dialogs render no content");

        // Opening auto-focuses Cancel (Radix behavior): the first open frame
        // requests focus, the next frame grants it.
        open = true;
        for _ in 0..2 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("a4")
                    .show(ui, |ui| {
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogFooter::new().show(ui, |ui| {
                                cancel_id = Some(AlertDialogCancel::new("Cancel").show(ui).id);
                            });
                        });
                    });
            });
        }
        assert!(
            h.ctx.memory(|m| m.focused() == cancel_id),
            "the cancel button must receive the initial focus"
        );

        // Focus is released once the dialog closes and the fade-out removes
        // the button from the screen.
        open = false;
        for _ in 0..2 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("a4")
                    .show(ui, |ui| {
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogFooter::new().show(ui, |ui| {
                                cancel_id = Some(AlertDialogCancel::new("Cancel").show(ui).id);
                            });
                        });
                    });
            });
        }
        assert_ne!(
            h.ctx.memory(|m| m.focused()),
            cancel_id,
            "focus must be released after closing"
        );
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let d = AlertDialog::controlled(&mut open).id_salt("x");
        assert_eq!(d.id_salt.as_deref(), Some("x"));
        let c = AlertDialogContent::new()
            .size(AlertDialogSize::Sm)
            .id_salt("c");
        assert_eq!(c.size, AlertDialogSize::Sm);
        let t = AlertDialogTrigger::new("hi").disabled(true).id_salt("t");
        assert!(t.disabled);
        let a = AlertDialogAction::new("go")
            .variant(ButtonVariant::Destructive)
            .id_salt("a");
        assert_eq!(a.variant, ButtonVariant::Destructive);
        let c2 = AlertDialogCancel::new("no")
            .variant(ButtonVariant::Ghost)
            .id_salt("n");
        assert_eq!(c2.variant, ButtonVariant::Ghost);
        let _ = AlertDialogHeader::new();
        let _ = AlertDialogFooter::new();
        let _ = AlertDialogTitle::new("T");
        let _ = AlertDialogDescription::new("D");
        let _ = AlertDialogMedia::new(Icon::CircleAlert);
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
