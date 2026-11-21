//! shadcn `alert-dialog.tsx` — a modal confirmation dialog.
//!
//! Rebuilt to the frozen Batch E S0 spec
//! (`~/workspace/batch-e/specs/alert-dialog-spec.md`).
//!
//! Panel: `shadow-lg` under BOTH recipes (LAYER — nova is silent on shadow
//! for this slot), base 1px `border` + nova `ring-1 ring-foreground/10`,
//! `bg-background` / `bg-popover`, padding 24px base / 16px nova, radius
//! 10px base / 14px nova, `grid gap-4`. No `text-sm` on Content (unlike
//! dialog — text size is inherited; Title/Description set their own).
//! Widths: Sm 320px at all widths; Default 512px base / 384px nova at
//! `sm+`, viewport−32 base / 320px nova below `sm`.
//!
//! Header: a grid — centred except Default at/above `sm` (left); with media
//! present, Default at/above `sm` is a 2-column grid (media `row-span-2`
//! col 1, title col 2 row 1, description col 2 row 2; column gap 24px base /
//! 16px nova, row gap 6px). Sm + media always stacks. Media box 64px base /
//! 40px nova, icon 32px / 24px, `bg-muted rounded-md`; media is a generic
//! container (any children), not icon-only.
//!
//! Footer: Default stacks reversed below `sm` and is a right-aligned row at
//! `sm+`; Sm is a 2-column equal-cells grid at ALL widths. Nova strip
//! (`bg-muted/50 -mx-4 -mb-4 rounded-b-xl border-t p-4`) on both sizes; the
//! base recipe has no strip.
//!
//! Radix behavior: unlike [`crate::components::dialog::Dialog`], an alert
//! dialog has **no close X** and **backdrop clicks do not dismiss it**; only
//! the Cancel/Action buttons or Escape close it — and Escape closes *without*
//! confirming the action (interim target; OPEN-1 — the definitive Radix
//! source check is still open). Opening auto-focuses Cancel; closing returns
//! focus to the trigger (N-10).
//!
//! Animation is fade-only (D2 — egui widgets do not scale): 100ms nova /
//! 200ms base. Backdrop is flat (D1 — no backdrop blur in egui).
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

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::icons::Icon;
use crate::theme::fonts::{medium, semibold};
use crate::theme::tokens::{spacing, type_scale};
use crate::theme::{color_alpha, ShadcnTheme, Style};
use crate::utils::shadow::{paint_shadow_resolved, ShadowSize, ShadowSlot};
use egui::epaint::StrokeKind;
use egui::{
    Align, Color32, Context, CornerRadius, Frame, Id, Key, Layout, Modal, Modifiers, Pos2, Rect,
    Response, Sense, Stroke, Ui, UiBuilder, Vec2,
};

/// nova `data-[size=default]:sm:max-w-sm` (384 px).
pub const ALERT_DIALOG_MAX_WIDTH: f32 = 384.0;
/// `data-[size=sm]:max-w-xs` (320 px), both recipes.
pub const ALERT_DIALOG_SM_WIDTH: f32 = 320.0;
/// base `data-[size=default]:sm:max-w-lg` (512 px).
pub const ALERT_DIALOG_BASE_MAX_WIDTH: f32 = 512.0;
/// `sm:` breakpoint (640 px, Tailwind v4 default).
const SM_BREAKPOINT: f32 = 640.0;
/// nova `duration-100`.
const ANIM_NOVA_SECS: f32 = 0.1;
/// base `duration-200`.
const ANIM_BASE_SECS: f32 = 0.2;
/// base backdrop: `bg-black/50`.
const BACKDROP_ALPHA_BASE: f32 = 0.50;
/// nova backdrop: `bg-black/10`.
const BACKDROP_ALPHA_NOVA: f32 = 0.10;
/// nova ring: `ring-foreground/10`.
const RING_ALPHA: f32 = 0.10;
/// nova footer strip fill: `bg-muted/50`.
const STRIP_FILL_ALPHA: f32 = 0.5;
/// media box bottom margin: `mb-2` (§4).
const MEDIA_MARGIN_BOTTOM: f32 = 8.0;

/// `data-size` presets on the content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertDialogSize {
    /// Left-aligned text, 384 px max width under nova (the source default).
    #[default]
    Default,
    /// Centered text, 320 px max width.
    Sm,
}

impl AlertDialogSize {
    /// Max content width in points under the nova (default) style.
    ///
    /// The full width rule (§3) also depends on the style and viewport —
    /// see [`panel_max_width`] — so this is the nova cap only.
    pub fn width(self) -> f32 {
        match self {
            AlertDialogSize::Default => ALERT_DIALOG_MAX_WIDTH,
            AlertDialogSize::Sm => ALERT_DIALOG_SM_WIDTH,
        }
    }
}

/// Panel max width (§3): Sm 320px at all widths; Default 512px base /
/// 384px nova at `sm+`, viewport−32 base / 320px nova below `sm`.
/// Always capped by `max-w-[calc(100%-2rem)]` (viewport−32).
fn panel_max_width(size: AlertDialogSize, nova: bool, screen_w: f32) -> f32 {
    let cap = match size {
        AlertDialogSize::Sm => ALERT_DIALOG_SM_WIDTH,
        AlertDialogSize::Default => {
            if screen_w >= SM_BREAKPOINT {
                if nova {
                    ALERT_DIALOG_MAX_WIDTH
                } else {
                    ALERT_DIALOG_BASE_MAX_WIDTH
                }
            } else if nova {
                ALERT_DIALOG_SM_WIDTH
            } else {
                // Base below `sm`: no `sm:` max-width applies — the panel is
                // viewport−32 (the `max-w-[calc(100%-2rem)]` cap below).
                screen_w - 32.0
            }
        }
    };
    cap.min(screen_w - 32.0).max(80.0)
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
    /// Widget that had focus when the dialog opened; focus returns here on
    /// close (N-10). `None` once the restore has stuck or was abandoned.
    return_focus: Option<Id>,
    /// Whether the modal was rendered on the previous frame (settle
    /// bookkeeping for the focus return).
    was_rendered: bool,
    /// Whether the dialog was open on the previous frame (opening-edge
    /// detection).
    was_open: bool,
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

/// Animates the open/closed alpha with an explicit duration (mirrors
/// `overlay_alpha` in dialog.rs).
fn overlay_alpha(ctx: &Context, id: Id, open: bool, duration_secs: f32) -> f32 {
    let target = if open { 1.0 } else { 0.0 };
    ctx.animate_value_with_time(id, target, duration_secs)
}

/// Settles the pending focus return (N-10; mirrors `settle_focus_return` in
/// dialog.rs, DLG-INT-01). The modal layer wipes a focus request issued while
/// it is still up, so the restore is (re-)issued on every fully-closed frame
/// until it sticks across a full frame boundary (which also covers callers
/// that show the content before the trigger, where the first restore is
/// wiped later in the same frame), and abandoned if focus demonstrably moved
/// elsewhere after the dialog fully closed — never fight the user's Tab.
/// Nested/stacked dialogs are out of scope (OPEN-11).
fn settle_focus_return(ctx: &Context, state_id: Id, rendered: bool) {
    let mut state = read_state(ctx, state_id);
    let was_rendered = state.was_rendered;
    state.was_rendered = rendered;
    if rendered {
        // Modal still up (open or fading): wait until it is gone.
        write_state(ctx, state_id, state);
        return;
    }
    let Some(id) = state.return_focus else {
        write_state(ctx, state_id, state);
        return;
    };
    let current = ctx.memory(|m| m.focused());
    if current == Some(id) {
        // The restore survived a full frame boundary: it stuck.
        state.return_focus = None;
    } else if !was_rendered && current.is_some() {
        // Focus moved elsewhere after the dialog fully closed: the user
        // (or another widget) owns it now; don't yank it back.
        state.return_focus = None;
    } else {
        ctx.memory_mut(|m| m.request_focus(id));
    }
    write_state(ctx, state_id, state);
}

// ---------------------------------------------------------------------------
// AlertDialog (root)
// ---------------------------------------------------------------------------

/// The shadcn alert-dialog root. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct AlertDialog<'a> {
    controlled: Option<&'a mut bool>,
    default_open: bool,
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
            default_open: false,
            id_salt: None,
        }
    }

    /// Creates an alert dialog whose open state is owned by the caller.
    /// Cancel, Action and Escape all write back.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            default_open: false,
            id_salt: None,
        }
    }

    /// Initial open state (uncontrolled mode only; N-13, Radix `defaultOpen`).
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
            None => ui.auto_id_with("shadcn_alert_dialog"),
        };

        let ctx = ui.ctx().clone();
        if let Some(open) = self.controlled.as_deref() {
            // Controlled: sync the open flag only, preserving the
            // transition bookkeeping (`was_open`, `return_focus`) that
            // `AlertDialogContent` maintains.
            let mut state = read_state(&ctx, state_id);
            state.open = *open;
            write_state(&ctx, state_id, state);
        } else {
            let exists = ctx.data_mut(|d| d.get_temp::<AlertDialogState>(state_id).is_some());
            if !exists {
                write_state(
                    &ctx,
                    state_id,
                    AlertDialogState {
                        open: self.default_open,
                        ..Default::default()
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

/// The Sm-footer grid protocol (§7, A-P2-2).
///
/// The Sm footer is a `grid grid-cols-2`, but the content closure is opaque,
/// so the footer cannot split it across cells itself. Instead it runs the
/// closure once per button with a "run" recorded in temp data, and each
/// `AlertDialogAction`/`AlertDialogCancel` renders for real only when its
/// index matches the run's target (otherwise it allocates nothing). A
/// measuring run renders every button for real in an invisible child and
/// records heights. `Off` means no grid run is active (normal rendering).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmGridRun {
    Off,
    /// Measuring run: render for real, record heights.
    Measure,
    /// Paint run for the button with this index.
    Paint(usize),
}

/// The confirm button (`variant = "default"`): closes the dialog when
/// clicked. Check the returned response's `clicked()` to run the action.
pub struct AlertDialogAction<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    variant: ButtonVariant,
    size: ButtonSize,
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
            size: ButtonSize::Default,
            id_salt: None,
        }
    }

    /// Creates an action rendering a custom widget (Radix `asChild`).
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w)),
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            id_salt: None,
        }
    }

    /// Overrides the button variant (default: [`ButtonVariant::Default`]).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the button size (default: [`ButtonSize::Default`], N-15).
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
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
            return action_button(&self.label, self.variant, self.size, self.id_salt.clone())
                .show(ui);
        };

        let ctx = ui.ctx().clone();
        let run = ctx
            .data_mut(|d| d.get_temp::<SmGridRun>(state_id.with("sm_grid_run")))
            .unwrap_or(SmGridRun::Off);
        let mut grid: Option<SmGridRun> = None;
        if run != SmGridRun::Off {
            let index = ctx.data_mut(|d| {
                let index = d
                    .get_temp::<usize>(state_id.with("sm_grid_index"))
                    .unwrap_or(0);
                d.insert_temp(state_id.with("sm_grid_index"), index + 1);
                index
            });
            if let SmGridRun::Paint(target) = run {
                if target != index {
                    // Another button's cell: allocate nothing.
                    return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
                }
            }
            grid = Some(run);
        }

        let mut response = match self.widget {
            Some(w) => w(ui),
            None => {
                action_button(&self.label, self.variant, self.size, self.id_salt.clone()).show(ui)
            }
        };

        if grid == Some(SmGridRun::Measure) {
            // Record the natural height for the grid row sizing.
            let h = response.rect.height();
            ctx.data_mut(|d| {
                let mut heights: Vec<f32> = d
                    .get_temp(state_id.with("sm_grid_heights"))
                    .unwrap_or_default();
                heights.push(h);
                d.insert_temp(state_id.with("sm_grid_heights"), heights);
            });
        }

        if response.clicked() {
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
    size: ButtonSize,
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
            size: ButtonSize::Default,
            id_salt: None,
        }
    }

    /// Creates a cancel control rendering a custom widget.
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w)),
            variant: ButtonVariant::Outline,
            size: ButtonSize::Default,
            id_salt: None,
        }
    }

    /// Overrides the button variant (default: [`ButtonVariant::Outline`]).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the button size (default: [`ButtonSize::Default`], N-15).
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
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
            return action_button(&self.label, self.variant, self.size, self.id_salt.clone())
                .show(ui);
        };

        let ctx = ui.ctx().clone();
        let run = ctx
            .data_mut(|d| d.get_temp::<SmGridRun>(state_id.with("sm_grid_run")))
            .unwrap_or(SmGridRun::Off);
        let mut grid: Option<SmGridRun> = None;
        if run != SmGridRun::Off {
            let index = ctx.data_mut(|d| {
                let index = d
                    .get_temp::<usize>(state_id.with("sm_grid_index"))
                    .unwrap_or(0);
                d.insert_temp(state_id.with("sm_grid_index"), index + 1);
                index
            });
            if let SmGridRun::Paint(target) = run {
                if target != index {
                    // Another button's cell: allocate nothing.
                    return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
                }
            }
            grid = Some(run);
        }

        let mut response = match self.widget {
            Some(w) => w(ui),
            None => {
                action_button(&self.label, self.variant, self.size, self.id_salt.clone()).show(ui)
            }
        };

        if grid == Some(SmGridRun::Measure) {
            // Record the natural height for the grid row sizing.
            let h = response.rect.height();
            ctx.data_mut(|d| {
                let mut heights: Vec<f32> = d
                    .get_temp(state_id.with("sm_grid_heights"))
                    .unwrap_or_default();
                heights.push(h);
                d.insert_temp(state_id.with("sm_grid_heights"), heights);
            });
        }

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
    size: ButtonSize,
    id_salt: Option<String>,
) -> Button {
    let mut b = Button::new(label.clone().unwrap_or_default())
        .variant(variant)
        .size(size);
    if let Some(salt) = id_salt {
        b = b.id_salt(salt);
    }
    b
}

// ---------------------------------------------------------------------------
// AlertDialogContent
// ---------------------------------------------------------------------------

/// The modal panel: centered, `shadow-lg` under both recipes, base 1px
/// `border` + nova `ring-1 ring-foreground/10`, `bg-background` /
/// `bg-popover`, padding 24px base / 16px nova, radius 10px base / 14px
/// nova. No close X; backdrop clicks are ignored; Escape dismisses without
/// confirming. Opening auto-focuses the Cancel button, like Radix; closing
/// returns focus to the trigger.
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
    pub fn show(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui)) -> Option<Response> {
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
        let opening = open && !state.was_open;
        if opening {
            // Remember who had focus so it can be restored on close (N-10).
            state.return_focus = ctx.memory(|m| m.focused());
        }
        state.was_open = open;
        write_state(&ctx, state_id, state);

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let nova = theme.style == Style::Nova;
        let dark = theme.mode.is_dark();

        let modal_id = state_id.with("modal");
        let anim_id = state_id.with("anim");

        // Fade-only animation (D2 — egui widgets do not scale): 100ms nova /
        // 200ms base, the same duration for backdrop and panel so they
        // animate together (§9).
        let duration = if nova { ANIM_NOVA_SECS } else { ANIM_BASE_SECS };
        let alpha = overlay_alpha(&ctx, anim_id, open, duration);
        let rendered = open || alpha > 0.002;
        // Settle the pending focus return now that the modal-render decision
        // is known (mirrors dialog.rs DLG-INT-01): the restore runs only on
        // fully-closed frames, after the modal layer is gone.
        settle_focus_return(&ctx, state_id, rendered);
        if !rendered {
            return None;
        }

        let pad = if nova { spacing::LG } else { spacing::XL };
        let screen_w = ctx.content_rect().width();
        let max_width = panel_max_width(self.size, nova, screen_w);
        let radius = CornerRadius::same(if nova {
            theme.radius.xl_u8()
        } else {
            theme.radius.lg_u8()
        });
        let fill = if nova { p.popover } else { p.background };
        let text_color = if nova {
            p.popover_foreground
        } else {
            p.foreground
        };
        let backdrop_alpha = if nova {
            BACKDROP_ALPHA_NOVA
        } else {
            BACKDROP_ALPHA_BASE
        };
        // Flat backdrop, no blur: D1 (egui has no backdrop blur; the
        // original's own `supports-backdrop-filter:` guard renders the same
        // flat backdrop where blur is unsupported).
        let backdrop = color_alpha(Color32::BLACK, backdrop_alpha * alpha);

        let modal = Modal::new(modal_id)
            .backdrop_color(backdrop)
            .frame(Frame::new());

        let modal_resp = modal.show(&ctx, |ui| {
            ui.set_opacity(alpha);

            // Measure the content height with an invisible pass so the
            // shadow, fill, border, and ring can be painted behind the real
            // content with the exact panel rect (same pattern as dialog.rs:
            // widgets in the measuring pass are disabled, so no callbacks
            // can double-fire).
            let inner_w = max_width - pad * 2.0;
            let measure_rect =
                Rect::from_min_size(ui.cursor().min, Vec2::new(inner_w, f32::INFINITY));
            let mut measure = ui.new_child(UiBuilder::new().max_rect(measure_rect).invisible());
            measure.spacing_mut().item_spacing.y = spacing::LG;
            measure.style_mut().visuals.override_text_color = Some(text_color);
            content(&mut measure);
            let content_h = measure.min_rect().height();

            let panel_h = content_h + pad * 2.0;
            let (_, panel_rect) = ui.allocate_space(Vec2::new(max_width, panel_h));

            // Shadow first, behind the body. `shadow-lg` under both recipes
            // (LAYER: nova is silent for this slot, §6).
            paint_shadow_resolved(
                ui.painter(),
                panel_rect,
                radius,
                ShadowSize::Lg,
                theme.style,
                ShadowSlot::AlertDialogContent,
                dark,
            );
            ui.painter().rect_filled(panel_rect, radius, fill);
            // Base `border` (1px, border token) survives under nova (LAYER);
            // nova adds `ring-1 ring-foreground/10` just outside it (§6).
            ui.painter().rect_stroke(
                panel_rect,
                radius,
                Stroke::new(1.0, p.border),
                StrokeKind::Inside,
            );
            if nova {
                ui.painter().rect_stroke(
                    panel_rect,
                    radius,
                    Stroke::new(1.0, color_alpha(p.foreground, RING_ALPHA)),
                    StrokeKind::Outside,
                );
            }

            // Real content pass inside the panel padding. `grid gap-4`
            // between direct children, both recipes (§3).
            let inner_rect = panel_rect.shrink(pad);
            let mut child = ui.new_child(UiBuilder::new().max_rect(inner_rect));
            child.set_min_width(inner_w);
            child.set_max_width(inner_w);
            child.spacing_mut().item_spacing.y = spacing::LG;
            child.style_mut().visuals.override_text_color = Some(text_color);
            content(&mut child);

            // Radix auto-focuses the cancel button when the dialog opens; the
            // guard makes it fire once per open, and resets while closed
            // (during the fade-out, while the modal still renders).
            let focus_guard = state_id.with("cancel_focus_done");
            let already_focused =
                ctx.data_mut(|d| d.get_temp::<bool>(focus_guard).unwrap_or(false));
            if !open {
                ctx.data_mut(|d| d.insert_temp(focus_guard, false));
            }
            if open && !already_focused {
                ctx.data_mut(|d| d.insert_temp(focus_guard, true));
                if let Some(cancel_id) =
                    ctx.data_mut(|d| d.get_temp::<Id>(state_id.with("cancel_resp")))
                {
                    ctx.memory_mut(|m| m.request_focus(cancel_id));
                }
            }
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

/// Header alignment (§4, N-02): centred except Default at/above `sm`.
fn header_align(ui: &Ui) -> Align {
    let size = current_scope(ui)
        .map(|id| read_state(ui.ctx(), id).size)
        .unwrap_or_default();
    let wide = ui.ctx().content_rect().width() >= SM_BREAKPOINT;
    if size == AlertDialogSize::Default && wide {
        Align::LEFT
    } else {
        Align::Center
    }
}

/// Which pass of the header grid is running (§4, N-03). The header runs its
/// content closure up to three times (measure, right column, left column);
/// the parts filter themselves by this flag. `Off` means no grid pass is
/// active (stacked render or no header).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeaderPass {
    Off,
    /// Measuring run (invisible): detect media presence only.
    Measure,
    /// Real run: the right column (title/description) in grid mode.
    Col2,
    /// Real run: the left column (media) in grid mode.
    Col1,
}

fn header_pass(ui: &Ui) -> HeaderPass {
    current_scope(ui)
        .and_then(|id| {
            ui.ctx()
                .data_mut(|d| d.get_temp::<HeaderPass>(id.with("header_pass")))
        })
        .unwrap_or(HeaderPass::Off)
}

/// `grid place-items-center gap-1.5 text-center` header; centred except for
/// the `default` size at/above `sm` (left-aligned, §4 N-02). With media
/// present, Default at/above `sm` is a 2-column grid (§4 N-03).
#[derive(Clone, Copy, Debug, Default)]
pub struct AlertDialogHeader {
    _private: (),
}

impl AlertDialogHeader {
    /// Creates an alert-dialog header.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the header, aligning against the active content size and
    /// breakpoint. The content closure runs more than once (measure pass,
    /// then the real pass(es)); it must be `FnMut`, as for the footer.
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        let size = current_scope(ui)
            .map(|id| read_state(ui.ctx(), id).size)
            .unwrap_or_default();
        let wide = ui.ctx().content_rect().width() >= SM_BREAKPOINT;
        let align = header_align(ui);
        let Some(state_id) = current_scope(ui) else {
            // No scope: stacked render (no grid possible without the size
            // state).
            return ui
                .with_layout(Layout::top_down(align), |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = spacing::XS_HALF;
                    content(ui)
                })
                .inner;
        };
        let ctx = ui.ctx().clone();
        let avail_w = ui.available_width();
        let flow_min = ui.cursor().min;

        // Pass A (measure, invisible): detect media presence (§4).
        ctx.data_mut(|d| {
            d.insert_temp(state_id.with("header_media_present"), false);
            d.insert_temp(state_id.with("header_pass"), HeaderPass::Measure);
        });
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(avail_w, f32::INFINITY));
        let mut measure = ui.new_child(UiBuilder::new().max_rect(measure_rect).invisible());
        measure.with_layout(Layout::top_down(align), |ui| {
            ui.spacing_mut().item_spacing.y = spacing::XS_HALF;
            content(ui)
        });
        let media_present = ctx
            .data_mut(|d| d.get_temp::<bool>(state_id.with("header_media_present")))
            .unwrap_or(false);
        let grid = media_present && size == AlertDialogSize::Default && wide;

        if !grid {
            // Stacked: media / title / description, 6px row gaps.
            ctx.data_mut(|d| d.insert_temp(state_id.with("header_pass"), HeaderPass::Off));
            let result = ui
                .with_layout(Layout::top_down(align), |ui| {
                    ui.set_min_width(avail_w);
                    ui.spacing_mut().item_spacing.y = spacing::XS_HALF;
                    content(ui)
                })
                .inner;
            ctx.data_mut(|d| d.insert_temp(state_id.with("header_pass"), HeaderPass::Off));
            return result;
        }

        // Grid (§4, N-03): media `row-span-2` col 1, title col 2 row 1,
        // description col 2 row 2. Column gap 24px base / 16px nova; row
        // gap 6px.
        let theme = ShadcnTheme::of_ui(ui);
        let nova = theme.style == Style::Nova;
        let media_w = if nova { 40.0 } else { 64.0 };
        let gap_x = if nova { spacing::LG } else { spacing::XL };
        let gap_y = spacing::XS_HALF;
        let col2_x = flow_min.x + media_w + gap_x;
        let col2_w = (avail_w - media_w - gap_x).max(0.0);

        // Pass B (real, right column): title + description, left-aligned.
        ctx.data_mut(|d| d.insert_temp(state_id.with("header_pass"), HeaderPass::Col2));
        let col2_rect = Rect::from_min_size(
            Pos2::new(col2_x, flow_min.y),
            Vec2::new(col2_w, f32::INFINITY),
        );
        let mut col2 = ui.new_child(UiBuilder::new().max_rect(col2_rect));
        let _ = col2.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = gap_y;
            content(ui)
        });
        let right_h = col2.min_rect().height();

        // The media's margin-box (box + `mb-2`) is vertically centred in
        // the two-row area (`place-items-center`).
        let media_box_h = media_w + MEDIA_MARGIN_BOTTOM;
        let total_h = right_h.max(media_box_h);
        let media_top = flow_min.y + (total_h - media_box_h) / 2.0;

        // Pass C (real, left column): the media, vertically centred.
        ctx.data_mut(|d| d.insert_temp(state_id.with("header_pass"), HeaderPass::Col1));
        let col1_rect = Rect::from_min_size(
            Pos2::new(flow_min.x, media_top),
            Vec2::new(media_w, media_box_h),
        );
        let mut col1 = ui.new_child(UiBuilder::new().max_rect(col1_rect));
        let result = col1
            .with_layout(Layout::top_down(Align::Min), |ui| content(ui))
            .inner;
        ctx.data_mut(|d| d.insert_temp(state_id.with("header_pass"), HeaderPass::Off));
        ui.allocate_space(Vec2::new(avail_w, total_h));
        result
    }
}

/// Alert-dialog title: `text-lg font-semibold` base (18px/600, 28px
/// line-height), `text-base font-medium` nova (16px/500, 24px line-height).
/// Neither recipe sets a `leading-*` class, so the Tailwind default
/// line-heights ARE the spec (§5) — do NOT copy dialog's `leading-none`.
/// The embedded Inter weights supply the 500/600 faces (same mechanism as
/// dialog's title).
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
        // Header grid protocol (§4): the title lives in the right column;
        // in the left-column pass it allocates nothing.
        if header_pass(ui) == HeaderPass::Col1 {
            return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
        }
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let nova = theme.style == Style::Nova;
        let (font, color, line_height) = if nova {
            (
                medium(type_scale::BASE),
                p.popover_foreground,
                24.0, // text-base default
            )
        } else {
            (
                semibold(type_scale::LG),
                p.foreground,
                28.0, // text-lg default
            )
        };
        let format = egui::TextFormat {
            font_id: font,
            color,
            line_height: Some(line_height),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = ui.available_width();
        job.halign = header_align(ui);
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, color);
        }
        response
    }
}

/// `text-sm text-muted-foreground` description (14px, 20px default
/// line-height), wrapping. Nova adds link styles and `text-balance` /
/// `md:text-pretty` (§5); plain-text descriptions render the same in both
/// recipes (link/balance handling is a wrapping nicety, not a separate
/// layout rule).
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
        // Header grid protocol (§4): the description lives in the right
        // column; in the left-column pass it allocates nothing.
        if header_pass(ui) == HeaderPass::Col1 {
            return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
        }
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let format = egui::TextFormat {
            font_id: egui::FontId::proportional(type_scale::SM),
            color: p.muted_foreground,
            line_height: Some(20.0), // text-sm default
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = ui.available_width();
        job.halign = header_align(ui);
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.muted_foreground);
        }
        response
    }
}

/// `bg-muted rounded-md` media slot (generic container, N-16).
///
/// `AlertDialogMedia::new(icon)` renders an icon; `with_content` renders
/// arbitrary children inside the box. The box is 64px (base) / 40px (nova)
/// with an 8px bottom margin (`mb-2`); the icon inside is 32px / 24px.
pub struct AlertDialogMedia<'a> {
    content: Box<dyn FnOnce(&mut Ui) + 'a>,
}

impl<'a> AlertDialogMedia<'a> {
    /// Creates a media slot showing the given icon.
    pub fn new(icon: Icon) -> Self {
        Self {
            content: Box::new(move |ui: &mut Ui| {
                let theme = ShadcnTheme::of_ui(ui);
                let p = theme.palette;
                let nova = theme.style == Style::Nova;
                let icon_side = if nova { 24.0 } else { 32.0 };
                let rect = ui.max_rect();
                if ui.is_rect_visible(rect) {
                    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_side));
                    icon.paint(
                        ui.painter(),
                        icon_rect,
                        p.popover_foreground,
                        icon_side / 24.0,
                    );
                }
            }),
        }
    }

    /// Creates a media slot rendering arbitrary children (N-16).
    pub fn with_content(content: impl FnOnce(&mut Ui) + 'a) -> Self {
        Self {
            content: Box::new(content),
        }
    }

    /// Renders the media box.
    pub fn show(self, ui: &mut Ui) -> Response {
        // Header grid protocol (§4): the media lives in the left column; in
        // the right-column pass it allocates nothing.
        if header_pass(ui) == HeaderPass::Col2 {
            return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
        }
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let nova = theme.style == Style::Nova;
        let side = if nova { 40.0 } else { 64.0 };
        // `mb-2` (8px) bottom margin on the media box (§4): the allocated
        // height is the margin-box.
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(side, side + MEDIA_MARGIN_BOTTOM), Sense::hover());
        // Record presence for the header grid decision (§4).
        if let Some(state_id) = current_scope(ui) {
            ui.ctx()
                .data_mut(|d| d.insert_temp(state_id.with("header_media_present"), true));
        }
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // Centring rule (§4, N-02): the media is centred whenever the
            // header is centred — everywhere except Default at/above `sm`.
            // In the left-column (grid) pass the box is already positioned
            // by the column rect, so no adjustment is needed there.
            let box_rect = if header_align(ui) == Align::Center {
                let center_x = ui.max_rect().center().x;
                Rect::from_min_size(
                    Pos2::new(center_x.max(rect.center().x) - side / 2.0, rect.top()),
                    Vec2::splat(side),
                )
            } else {
                Rect::from_min_size(rect.min, Vec2::splat(side))
            };
            painter.rect_filled(box_rect, theme.radius.md_u8(), p.muted);
            let mut child = ui.new_child(UiBuilder::new().max_rect(box_rect));
            (self.content)(&mut child);
        }
        response
    }
}

/// Alert-dialog footer (§7): Default stacks reversed below `sm` and is a
/// right-aligned row at `sm+` (N-04); Sm is a 2-column equal-cells grid at
/// ALL widths (A-P2-2). Nova strip on both sizes; base has no strip.
#[derive(Clone, Copy, Debug, Default)]
pub struct AlertDialogFooter {
    _private: (),
}

impl AlertDialogFooter {
    /// Creates an alert-dialog footer.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the footer.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnMut(&mut Ui) -> R) -> R {
        let size = current_scope(ui)
            .map(|id| read_state(ui.ctx(), id).size)
            .unwrap_or_default();
        match size {
            AlertDialogSize::Default => alert_dialog_default_footer(ui, content),
            AlertDialogSize::Sm => alert_dialog_sm_footer(ui, content),
        }
    }
}

/// The Default footer layout: recipe strip (nova only) + responsive
/// direction (§7, N-04). Mirrors `dialog_footer` in dialog.rs — the Default
/// footer rules are the same as dialog's — but the shared `footer_strip`
/// painter is row-only, always paints the strip, and measures with a
/// top-down child, so it cannot express the `sm:` direction switch or the
/// base no-strip rule; alert-dialog therefore carries its own copy here
/// (dialog.rs is out of scope for this line — see the S1 report).
fn alert_dialog_default_footer<R>(ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let nova = theme.style == Style::Nova;
    let wide = ui.ctx().content_rect().width() >= SM_BREAKPOINT;
    let pad = spacing::LG; // p-4: strip padding/bleed (nova)
    let gap = spacing::SM; // gap-2

    // Measure the content with the same layout the real pass uses, so the
    // strip (nova) and the in-flow reservation match the rendered content.
    // Widgets in the measuring pass are disabled, so no callbacks can
    // double-fire.
    let flow_min = ui.cursor().min;
    let flow_w = ui.available_width();
    let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
    let mut measure = ui.new_child(UiBuilder::new().max_rect(measure_rect).invisible());
    if wide {
        // NOTE: cross-align Min, not Center: `advance_after_rects` expands
        // `min_rect` to the frame rect, and a Center cross-align makes the
        // frame span the full (infinite) available height, so the measured
        // height would be infinite. Min keeps the frame widget-sized; the
        // row width (the main direction) is unaffected, and the real pass
        // below still centers vertically.
        measure.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = gap;
            content(ui)
        });
    } else {
        // Stack height is direction-independent; measure top-down, with the
        // same cross-axis justify as the real pass so wrapping children
        // measure at their real (stretched) width.
        measure.with_layout(
            Layout::top_down(Align::Min).with_cross_justify(true),
            |ui| {
                ui.spacing_mut().item_spacing.y = gap;
                content(ui)
            },
        );
    }
    let measured = measure.min_rect();
    let content_w = measured.width();
    let content_h = measured.height();

    if nova {
        // Full-bleed strip: it stretches to the panel's side edges and hangs
        // into the bottom padding; the in-flow reservation is the strip
        // height minus the bleed.
        let strip = Rect::from_min_size(
            Pos2::new(measure_rect.left() - pad, measure_rect.top()),
            Vec2::new(flow_w + pad * 2.0, content_h + pad * 2.0),
        );
        if ui.is_rect_visible(strip) {
            let painter = ui.painter();
            let corner = CornerRadius {
                nw: 0,
                ne: 0,
                sw: theme.radius.xl_u8(),
                se: theme.radius.xl_u8(),
            };
            painter.rect_filled(strip, corner, color_alpha(p.muted, STRIP_FILL_ALPHA));
            // border-t
            painter.line_segment(
                [strip.left_top(), strip.right_top()],
                Stroke::new(1.0, p.border),
            );
        }
        let inner = Rect::from_min_size(
            Pos2::new(strip.left() + pad, strip.top() + pad),
            Vec2::new(strip.width() - pad * 2.0, content_h),
        );
        let real = layout_footer_content(ui, inner, wide, gap, content_w, content);
        ui.allocate_space(Vec2::new(flow_w, (strip.height() - pad).max(0.0)));
        real
    } else {
        // Base: no strip — plain flex layout, no bleed, no extra padding.
        let inner = Rect::from_min_size(flow_min, Vec2::new(flow_w, content_h));
        let real = layout_footer_content(ui, inner, wide, gap, content_w, content);
        ui.allocate_space(Vec2::new(flow_w, content_h));
        real
    }
}

/// Lays out footer content in the measured rect: a right-aligned row in DOM
/// order at/above `sm`, a bottom-up stack (reverse DOM order) below `sm`
/// (mirrors dialog.rs). The stack justifies its cross axis:
/// `flex-col-reverse` carries the CSS `align-items: stretch` default, so
/// every child spans the content width. The row does not justify: cross-axis
/// stretch there is vertical and all children share the row height.
fn layout_footer_content<R>(
    ui: &mut Ui,
    inner: Rect,
    wide: bool,
    gap: f32,
    content_w: f32,
    mut content: impl FnMut(&mut Ui) -> R,
) -> R {
    if wide {
        // Right-aligned row: the child rect hugs the right edge of the
        // strip/panel, and DOM order runs left to right inside it.
        let row = Rect::from_min_size(
            Pos2::new(inner.right() - content_w, inner.top()),
            Vec2::new(content_w, inner.height()),
        );
        let mut child = ui.new_child(UiBuilder::new().max_rect(row));
        child
            .with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                content(ui)
            })
            .inner
    } else {
        // Column-reverse: the first DOM child lands at the bottom.
        // `with_cross_justify(true)` is the egui form of the flex
        // `align-items: stretch` default: the placer expands each child to
        // the frame width, so buttons (which paint their whole allocated
        // rect, content centred) span the content width.
        let mut child = ui.new_child(UiBuilder::new().max_rect(inner));
        child
            .with_layout(
                Layout::bottom_up(Align::Min).with_cross_justify(true),
                |ui| {
                    ui.spacing_mut().item_spacing.y = gap;
                    content(ui)
                },
            )
            .inner
    }
}

/// The Sm footer: `grid grid-cols-2` (§7, A-P2-2) — two equal-width cells
/// at ALL widths, a third child wrapping to a second row, `gap-2` (8px).
/// The content closure is opaque, so the footer runs it once per button
/// with the [`SmGridRun`] protocol: a measuring run records each button's
/// height, then one paint run per button renders it in its cell. Only
/// `AlertDialogAction`/`AlertDialogCancel` children participate; arbitrary
/// widgets in an Sm footer cannot be cell-filtered (see the S1 report).
fn alert_dialog_sm_footer<R>(ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let nova = theme.style == Style::Nova;
    let pad = spacing::LG; // p-4 strip padding/bleed (nova)
    let gap = spacing::SM; // gap-2
    let flow_min = ui.cursor().min;
    let flow_w = ui.available_width();
    let Some(state_id) = current_scope(ui) else {
        // No scope: no cell protocol possible; render the content plainly.
        return content(ui);
    };
    let ctx = ui.ctx().clone();

    // Measuring run: render every button for real in an invisible child and
    // record heights (widgets are disabled there, so no callbacks fire).
    ctx.data_mut(|d| {
        d.insert_temp(state_id.with("sm_grid_heights"), Vec::<f32>::new());
        d.insert_temp(state_id.with("sm_grid_index"), 0usize);
        d.insert_temp(state_id.with("sm_grid_run"), SmGridRun::Measure);
    });
    let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
    let mut measure = ui.new_child(UiBuilder::new().max_rect(measure_rect).invisible());
    measure.with_layout(Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = gap;
        content(ui)
    });
    let heights: Vec<f32> = ctx
        .data_mut(|d| d.get_temp(state_id.with("sm_grid_heights")))
        .unwrap_or_default();
    let n = heights.len();
    eprintln!("SMFOOT measure: n={n} heights={heights:?} flow_min={flow_min:?} flow_w={flow_w}");

    // Rows of two cells; each row is as tall as its tallest button.
    let rows = n.div_ceil(2).max(1);
    let mut row_heights = Vec::with_capacity(rows);
    for r in 0..rows {
        let h0 = heights.get(2 * r).copied().unwrap_or(0.0);
        let h1 = heights.get(2 * r + 1).copied().unwrap_or(0.0);
        row_heights.push(h0.max(h1));
    }
    let content_h = row_heights.iter().sum::<f32>() + gap * (rows.saturating_sub(1) as f32);

    // Strip (nova only) + inner rect, as in the Default footer.
    let (strip_h, inner) = if nova {
        let strip = Rect::from_min_size(
            Pos2::new(measure_rect.left() - pad, measure_rect.top()),
            Vec2::new(flow_w + pad * 2.0, content_h + pad * 2.0),
        );
        if ui.is_rect_visible(strip) {
            let painter = ui.painter();
            let corner = CornerRadius {
                nw: 0,
                ne: 0,
                sw: theme.radius.xl_u8(),
                se: theme.radius.xl_u8(),
            };
            painter.rect_filled(strip, corner, color_alpha(p.muted, STRIP_FILL_ALPHA));
            painter.line_segment(
                [strip.left_top(), strip.right_top()],
                Stroke::new(1.0, p.border),
            );
        }
        let inner = Rect::from_min_size(
            Pos2::new(strip.left() + pad, strip.top() + pad),
            Vec2::new(strip.width() - pad * 2.0, content_h),
        );
        (strip.height(), inner)
    } else {
        let inner = Rect::from_min_size(flow_min, Vec2::new(flow_w, content_h));
        (content_h, inner)
    };

    // One paint run per button: each button renders for real only in its
    // own cell; the others allocate nothing. (With no buttons, a single run
    // renders the empty strip and returns the closure's value.)
    let runs = n.max(1);
    let cell_w = (inner.width() - gap) / 2.0;
    let mut result: Option<R> = None;
    for target in 0..runs {
        let row = target / 2;
        let col = target % 2;
        let cell_y = inner.top() + row_heights[..row].iter().sum::<f32>() + gap * row as f32;
        let cell_x = inner.left() + col as f32 * (cell_w + gap);
        let cell = Rect::from_min_size(
            Pos2::new(cell_x, cell_y),
            Vec2::new(cell_w, row_heights[row]),
        );
        ctx.data_mut(|d| {
            d.insert_temp(state_id.with("sm_grid_index"), 0usize);
            d.insert_temp(state_id.with("sm_grid_run"), SmGridRun::Paint(target));
        });
        eprintln!(
            "SMFOOT paint: target={target} row={row} col={col} cell={cell:?} inner={inner:?}"
        );
        let mut child = ui.new_child(UiBuilder::new().max_rect(cell));
        let r = child
            .with_layout(
                Layout::top_down(Align::Min).with_cross_justify(true),
                |ui| content(ui),
            )
            .inner;
        result = Some(r);
    }
    // Clear the protocol so later parts render normally.
    ctx.data_mut(|d| d.insert_temp(state_id.with("sm_grid_run"), SmGridRun::Off));
    let reservation = if nova {
        (strip_h - pad).max(0.0)
    } else {
        content_h
    };
    ui.allocate_space(Vec2::new(flow_w, reservation));
    result.expect("at least one paint run")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset, Style};
    use crate::utils::shadow::resolve_shadow;
    use egui::Pos2;

    struct Harness {
        ctx: egui::Context,
        time: f64,
        screen_w: f32,
        screen_h: f32,
    }

    fn test_ui_id() -> Id {
        Id::unique("alert_dialog_test_ui")
    }

    impl Harness {
        fn new() -> Self {
            Self::with_preset(Preset::default())
        }

        fn with_preset(preset: Preset) -> Self {
            let ctx = egui::Context::default();
            preset.install(&ctx);
            Self {
                ctx,
                time: 0.0,
                screen_w: 800.0,
                screen_h: 600.0,
            }
        }

        fn set_screen(&mut self, w: f32, h: f32) {
            self.screen_w = w;
            self.screen_h = h;
        }

        fn screen(&self) -> Rect {
            Rect::from_min_size(Pos2::ZERO, Vec2::new(self.screen_w, self.screen_h))
        }

        fn raw_input(&self, events: Vec<egui::Event>) -> egui::RawInput {
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(self.screen()),
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
                    test_ui_id(),
                    UiBuilder::new().max_rect(self.screen()),
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
                        test_ui_id(),
                        UiBuilder::new().max_rect(self.screen()),
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
                    test_ui_id(),
                    UiBuilder::new().max_rect(self.screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> AlertDialogState {
            let id = test_ui_id().with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<AlertDialogState>(id))
                .unwrap_or_default()
        }

        fn focused(&self) -> Option<Id> {
            self.ctx.memory(|m| m.focused())
        }
    }

    #[test]
    fn constants_match_recipes() {
        assert_eq!(ALERT_DIALOG_MAX_WIDTH, 384.0); // nova sm:max-w-sm
        assert_eq!(ALERT_DIALOG_BASE_MAX_WIDTH, 512.0); // base sm:max-w-lg
        assert_eq!(ALERT_DIALOG_SM_WIDTH, 320.0); // max-w-xs
        assert_eq!(AlertDialogSize::Default.width(), 384.0);
        assert_eq!(AlertDialogSize::Sm.width(), 320.0);
        assert!((SM_BREAKPOINT - 640.0).abs() < f32::EPSILON);
        assert!((RING_ALPHA - 0.10).abs() < f32::EPSILON);
        assert!((BACKDROP_ALPHA_NOVA - 0.10).abs() < f32::EPSILON); // bg-black/10
        assert!((BACKDROP_ALPHA_BASE - 0.50).abs() < f32::EPSILON); // bg-black/50
        assert!((ANIM_NOVA_SECS - 0.1).abs() < f32::EPSILON); // duration-100
        assert!((ANIM_BASE_SECS - 0.2).abs() < f32::EPSILON); // duration-200
        assert!((STRIP_FILL_ALPHA - 0.5).abs() < f32::EPSILON); // bg-muted/50
        assert!((MEDIA_MARGIN_BOTTOM - 8.0).abs() < f32::EPSILON); // mb-2
    }

    #[test]
    fn panel_width_rules() {
        // Sm: 320 at all widths, both recipes.
        assert!((panel_max_width(AlertDialogSize::Sm, true, 800.0) - 320.0).abs() < f32::EPSILON);
        assert!((panel_max_width(AlertDialogSize::Sm, false, 800.0) - 320.0).abs() < f32::EPSILON);
        assert!((panel_max_width(AlertDialogSize::Sm, true, 500.0) - 320.0).abs() < f32::EPSILON);
        assert!((panel_max_width(AlertDialogSize::Sm, false, 500.0) - 320.0).abs() < f32::EPSILON);
        // Default at/above sm: 384 nova / 512 base.
        assert!(
            (panel_max_width(AlertDialogSize::Default, true, 800.0) - 384.0).abs() < f32::EPSILON
        );
        assert!(
            (panel_max_width(AlertDialogSize::Default, false, 800.0) - 512.0).abs() < f32::EPSILON
        );
        assert!(
            (panel_max_width(AlertDialogSize::Default, true, 640.0) - 384.0).abs() < f32::EPSILON
        );
        assert!(
            (panel_max_width(AlertDialogSize::Default, false, 640.0) - 512.0).abs() < f32::EPSILON
        );
        // Default below sm: 320 nova / viewport−32 base.
        assert!(
            (panel_max_width(AlertDialogSize::Default, true, 500.0) - 320.0).abs() < f32::EPSILON
        );
        assert!(
            (panel_max_width(AlertDialogSize::Default, false, 500.0) - 468.0).abs() < f32::EPSILON
        );
        assert!(
            (panel_max_width(AlertDialogSize::Default, false, 416.0) - 384.0).abs() < f32::EPSILON
        );
        // The viewport−32 cap binds on narrow screens.
        assert!(
            (panel_max_width(AlertDialogSize::Default, true, 300.0) - 268.0).abs() < f32::EPSILON
        );
    }

    #[test]
    fn shadow_lg_survives_under_both_recipes() {
        // LAYER: nova is silent on shadow for this slot, so the base
        // `shadow-lg` survives under both recipes.
        assert_eq!(
            resolve_shadow(ShadowSize::Lg, Style::Nova, ShadowSlot::AlertDialogContent),
            ShadowSize::Lg
        );
        assert_eq!(
            resolve_shadow(ShadowSize::Lg, Style::Vega, ShadowSlot::AlertDialogContent),
            ShadowSize::Lg
        );
        // And the paint path runs (no panic) under both recipes.
        for preset in [Preset::default(), Preset::default().with_style(Style::Vega)] {
            let mut h = Harness::with_preset(preset);
            let mut open = true;
            for _ in 0..3 {
                h.run(|ui| {
                    AlertDialog::controlled(&mut open)
                        .id_salt("sh1")
                        .show(ui, |ui| {
                            AlertDialogContent::new().show(ui, |ui| {
                                AlertDialogTitle::new("Title").show(ui);
                            });
                        });
                });
            }
            assert!(h.state("sh1").open);
        }
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
    fn focus_returns_to_trigger_on_close() {
        let mut h = Harness::new();
        let trigger_rect = std::cell::Cell::new(Rect::ZERO);
        let cancel_rect = std::cell::Cell::new(Rect::ZERO);
        let mut show = |ui: &mut Ui| {
            AlertDialog::new().id_salt("foc1").show(ui, |ui| {
                trigger_rect.set(
                    AlertDialogTrigger::new("Delete")
                        .id_salt("trig")
                        .show(ui)
                        .rect,
                );
                AlertDialogContent::new().show(ui, |ui| {
                    AlertDialogTitle::new("Title").show(ui);
                    AlertDialogFooter::new().show(ui, |ui| {
                        cancel_rect.set(AlertDialogCancel::new("Cancel").show(ui).rect);
                    });
                });
            });
        };
        h.run(&mut show);
        assert!(!h.state("foc1").open);
        let pos = trigger_rect.get().center();
        h.click(pos, &mut show);
        assert!(h.state("foc1").open, "trigger click opens");

        // Warm up: focus moves to Cancel, the modal settles.
        for _ in 0..3 {
            h.run(&mut show);
        }

        // Close via Cancel.
        let cpos = cancel_rect.get().center();
        h.click(cpos, &mut show);
        assert!(!h.state("foc1").open, "cancel click closes");

        // The restore lands on the first fully-closed frame (after the
        // fade-out), not on the closing frame.
        h.run(&mut show);
        h.run(&mut show);
        h.run(&mut show);
        let trigger_id = test_ui_id().with("trig");
        assert_eq!(
            h.focused(),
            Some(trigger_id),
            "focus returns to the trigger on close"
        );
        // The return must SURVIVE later frames.
        h.run(&mut show);
        h.run(&mut show);
        assert_eq!(
            h.focused(),
            Some(trigger_id),
            "focus return survives later frames"
        );
    }

    #[test]
    fn default_open_initial_state() {
        // Uncontrolled + default_open(true) starts open (N-13).
        let mut h = Harness::new();
        h.run(|ui| {
            AlertDialog::new()
                .default_open(true)
                .id_salt("dflt")
                .show(ui, |ui| {
                    AlertDialogContent::new().show(ui, |ui| {
                        AlertDialogTitle::new("Title").show(ui);
                    });
                });
        });
        assert!(h.state("dflt").open, "default_open(true) starts open");
        // Uncontrolled + default (no default_open) starts closed.
        let mut h = Harness::new();
        h.run(|ui| {
            AlertDialog::new().id_salt("dflt2").show(ui, |ui| {
                AlertDialogContent::new().show(ui, |ui| {
                    AlertDialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(!h.state("dflt2").open, "default starts closed");
    }

    #[test]
    fn action_cancel_size_forwards() {
        // The size builder is stored and forwarded to the Button: a larger
        // size renders taller (N-15).
        let a = AlertDialogAction::new("go").size(ButtonSize::Lg);
        assert_eq!(a.size, ButtonSize::Lg);
        let c = AlertDialogCancel::new("no").size(ButtonSize::Sm);
        assert_eq!(c.size, ButtonSize::Sm);
        let mut h = Harness::new();
        let mut sm_h = 0.0f32;
        let mut lg_h = 0.0f32;
        h.run(|ui| {
            sm_h = AlertDialogCancel::new("no")
                .size(ButtonSize::Sm)
                .show(ui)
                .rect
                .height();
            lg_h = AlertDialogAction::new("go")
                .size(ButtonSize::Lg)
                .show(ui)
                .rect
                .height();
        });
        assert!(
            lg_h > sm_h,
            "Lg renders taller than Sm: lg {lg_h} sm {sm_h}"
        );
    }

    #[test]
    fn title_typography_by_recipe() {
        // nova: 16px text, 24px default line-height (§5).
        let mut h = Harness::new();
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = AlertDialogTitle::new("Hello").show(ui).rect;
        });
        assert!(
            (rect.height() - 24.0).abs() < 0.01,
            "nova title height: {rect:?}"
        );

        // base: 18px text, 28px default line-height (§5).
        let mut h = Harness::with_preset(Preset::default().with_style(Style::Vega));
        h.run(|ui| {
            rect = AlertDialogTitle::new("Hello").show(ui).rect;
        });
        assert!(
            (rect.height() - 28.0).abs() < 0.01,
            "base title height: {rect:?}"
        );
    }

    #[test]
    fn description_line_height() {
        // text-sm, 20px default line-height (§5), both recipes.
        let mut h = Harness::new();
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = AlertDialogDescription::new("Hello").show(ui).rect;
        });
        assert!(
            (rect.height() - 20.0).abs() < 0.01,
            "description height: {rect:?}"
        );
    }

    #[test]
    fn media_box_size_by_recipe() {
        // nova: 40px box (+8px `mb-2` margin → 48px tall).
        let mut h = Harness::new();
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = AlertDialogMedia::new(Icon::CircleAlert).show(ui).rect;
        });
        assert!(
            (rect.width() - 40.0).abs() < 0.01,
            "nova media width: {rect:?}"
        );
        assert!(
            (rect.height() - 48.0).abs() < 0.01,
            "nova media margin-box height: {rect:?}"
        );

        // base: 64px box (+8px margin → 72px tall).
        let mut h = Harness::with_preset(Preset::default().with_style(Style::Vega));
        h.run(|ui| {
            rect = AlertDialogMedia::new(Icon::CircleAlert).show(ui).rect;
        });
        assert!(
            (rect.width() - 64.0).abs() < 0.01,
            "base media width: {rect:?}"
        );
        assert!(
            (rect.height() - 72.0).abs() < 0.01,
            "base media margin-box height: {rect:?}"
        );
    }

    #[test]
    fn media_generic_children() {
        // N-16: media is a generic container, not icon-only.
        let mut h = Harness::new();
        let mut media_rect = Rect::ZERO;
        let mut child_rect = Rect::ZERO;
        h.run(|ui| {
            media_rect = AlertDialogMedia::with_content(|ui| {
                child_rect = ui.label("X").rect;
            })
            .show(ui)
            .rect;
        });
        assert!(media_rect.width() > 0.0, "media renders");
        assert!(
            media_rect.contains_rect(child_rect),
            "child renders inside the media box: media {media_rect:?} child {child_rect:?}"
        );
    }

    #[test]
    fn header_alignment_six_cases() {
        // (screen_w, size, media, centred)
        let cases = [
            (500.0, AlertDialogSize::Default, false, true),
            (800.0, AlertDialogSize::Default, false, false),
            (800.0, AlertDialogSize::Sm, false, true),
            (500.0, AlertDialogSize::Default, true, true),
            (800.0, AlertDialogSize::Default, true, false),
            (800.0, AlertDialogSize::Sm, true, true),
        ];
        for (screen_w, size, media, centred) in cases {
            let mut h = Harness::new();
            h.set_screen(screen_w, 600.0);
            let mut open = true;
            let mut title_rect = Rect::ZERO;
            for _ in 0..3 {
                h.run(|ui| {
                    AlertDialog::controlled(&mut open)
                        .id_salt("hdr")
                        .show(ui, |ui| {
                            AlertDialogContent::new().size(size).show(ui, |ui| {
                                AlertDialogHeader::new().show(ui, |ui| {
                                    if media {
                                        AlertDialogMedia::new(Icon::CircleAlert).show(ui);
                                    }
                                    title_rect = AlertDialogTitle::new("Title").show(ui).rect;
                                });
                            });
                        });
                });
            }
            if centred {
                assert!(
                    (title_rect.center().x - screen_w / 2.0).abs() < 1.0,
                    "centred case w={screen_w} size={size:?} media={media}: {title_rect:?}"
                );
            } else {
                // Left-aligned: title starts at the panel inner left edge, or
                // (grid case, media present) at the right-column left edge.
                let panel_w = panel_max_width(size, true, screen_w);
                let inner_left = (screen_w - panel_w) / 2.0 + spacing::LG;
                let expected_left = if media {
                    inner_left + 40.0 + spacing::LG
                } else {
                    inner_left
                };
                assert!(
                    (title_rect.left() - expected_left).abs() < 1.0,
                    "left case w={screen_w} size={size:?} media={media}: {title_rect:?} vs {expected_left}"
                );
            }
        }
    }

    #[test]
    fn media_grid_default_wide_vs_sm_stacked() {
        // Default, ≥640, media: 2-column grid — media left of title.
        let mut h = Harness::new();
        let mut open = true;
        let mut media_rect = Rect::ZERO;
        let mut title_rect = Rect::ZERO;
        let mut desc_rect = Rect::ZERO;
        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("mg1")
                    .show(ui, |ui| {
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogHeader::new().show(ui, |ui| {
                                media_rect = AlertDialogMedia::new(Icon::CircleAlert).show(ui).rect;
                                title_rect = AlertDialogTitle::new("Title").show(ui).rect;
                                desc_rect = AlertDialogDescription::new("Description text.")
                                    .show(ui)
                                    .rect;
                            });
                        });
                    });
            });
        }
        assert!(
            media_rect.right() <= title_rect.left(),
            "grid: media left of title: media {media_rect:?} title {title_rect:?}"
        );
        assert!(
            (title_rect.left() - media_rect.right() - 16.0).abs() < 1.0,
            "grid: column gap 16px nova: media {media_rect:?} title {title_rect:?}"
        );
        assert!(
            (desc_rect.left() - title_rect.left()).abs() < 1.0,
            "grid: description in column 2: desc {desc_rect:?} title {title_rect:?}"
        );
        let block_top = title_rect.top();
        let block_bottom = desc_rect.bottom();
        assert!(
            (media_rect.center().y - (block_top + block_bottom) / 2.0).abs() < 1.0,
            "grid: media vertically centred on the block: media {media_rect:?} block {block_top}..{block_bottom}"
        );
        assert!(
            (media_rect.width() - 40.0).abs() < 0.01,
            "grid: media box 40px nova: {media_rect:?}"
        );

        // Sm, ≥640, media: stacked — media above the title, centred.
        let mut h = Harness::new();
        let mut open = true;
        let mut media_rect = Rect::ZERO;
        let mut title_rect = Rect::ZERO;
        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("mg2")
                    .show(ui, |ui| {
                        AlertDialogContent::new()
                            .size(AlertDialogSize::Sm)
                            .show(ui, |ui| {
                                AlertDialogHeader::new().show(ui, |ui| {
                                    media_rect =
                                        AlertDialogMedia::new(Icon::CircleAlert).show(ui).rect;
                                    title_rect = AlertDialogTitle::new("Title").show(ui).rect;
                                });
                            });
                    });
            });
        }
        assert!(
            media_rect.bottom() <= title_rect.top(),
            "sm: media above title: media {media_rect:?} title {title_rect:?}"
        );
        assert!(
            (media_rect.center().x - 400.0).abs() < 1.0,
            "sm: media centred: {media_rect:?}"
        );
    }

    #[test]
    fn sm_footer_equal_cells_narrow_and_wide() {
        // The Sm grid applies at ALL widths (§7, A-P2-2).
        for screen_w in [500.0, 800.0] {
            let mut h = Harness::new();
            h.set_screen(screen_w, 600.0);
            let mut open = true;
            let mut cancel_rect = Rect::ZERO;
            let mut action_rect = Rect::ZERO;
            for _ in 0..3 {
                h.run(|ui| {
                    AlertDialog::controlled(&mut open)
                        .id_salt("smf")
                        .show(ui, |ui| {
                            AlertDialogContent::new()
                                .size(AlertDialogSize::Sm)
                                .show(ui, |ui| {
                                    AlertDialogFooter::new().show(ui, |ui| {
                                        // The footer runs the closure once
                                        // per cell; dummy passes allocate a
                                        // zero rect, so only record real
                                        // renders.
                                        let r = AlertDialogCancel::new("Cancel").show(ui).rect;
                                        if r.width() > 1.0 {
                                            cancel_rect = r;
                                        }
                                        let r = AlertDialogAction::new("Continue").show(ui).rect;
                                        if r.width() > 1.0 {
                                            action_rect = r;
                                        }
                                    });
                                });
                        });
                });
            }
            assert!(
                (cancel_rect.width() - action_rect.width()).abs() < 1.0,
                "sm cells equal at {screen_w}: cancel {cancel_rect:?} action {action_rect:?}"
            );
            assert!(
                cancel_rect.left() < action_rect.left(),
                "sm: cancel in the left cell at {screen_w}: cancel {cancel_rect:?} action {action_rect:?}"
            );
            assert!(
                (action_rect.left() - cancel_rect.right() - 8.0).abs() < 1.0,
                "sm: column gap 8px at {screen_w}: cancel {cancel_rect:?} action {action_rect:?}"
            );
            assert!(
                (cancel_rect.top() - action_rect.top()).abs() < 1.0,
                "sm: same row at {screen_w}: cancel {cancel_rect:?} action {action_rect:?}"
            );
            let panel_w = panel_max_width(AlertDialogSize::Sm, true, screen_w);
            let inner_left = (screen_w - panel_w) / 2.0 + spacing::LG;
            let inner_right = (screen_w + panel_w) / 2.0 - spacing::LG;
            assert!(
                (cancel_rect.left() - inner_left).abs() < 1.0,
                "sm: cancel at inner left at {screen_w}: {cancel_rect:?} vs {inner_left}"
            );
            assert!(
                (action_rect.right() - inner_right).abs() < 1.0,
                "sm: action at inner right at {screen_w}: {action_rect:?} vs {inner_right}"
            );
        }
    }

    #[test]
    fn sm_footer_third_child_wraps_to_second_row() {
        let mut h = Harness::new();
        let mut open = true;
        let mut r1 = Rect::ZERO;
        let mut r2 = Rect::ZERO;
        let mut r3 = Rect::ZERO;
        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("sm3")
                    .show(ui, |ui| {
                        AlertDialogContent::new()
                            .size(AlertDialogSize::Sm)
                            .show(ui, |ui| {
                                AlertDialogFooter::new().show(ui, |ui| {
                                    let r = AlertDialogCancel::new("One").show(ui).rect;
                                    if r.width() > 1.0 {
                                        r1 = r;
                                    }
                                    let r = AlertDialogAction::new("Two").show(ui).rect;
                                    if r.width() > 1.0 {
                                        r2 = r;
                                    }
                                    let r = AlertDialogAction::new("Three").show(ui).rect;
                                    if r.width() > 1.0 {
                                        r3 = r;
                                    }
                                });
                            });
                    });
            });
        }
        assert!(
            (r1.top() - r2.top()).abs() < 1.0,
            "first row: one {r1:?} two {r2:?}"
        );
        assert!(
            r3.top() > r2.bottom(),
            "third wraps to a second row: three {r3:?} two {r2:?}"
        );
        assert!(
            (r3.left() - r1.left()).abs() < 1.0,
            "third in the left cell: three {r3:?} one {r1:?}"
        );
        assert!(
            (r3.width() - r1.width()).abs() < 1.0,
            "equal cells: three {r3:?} one {r1:?}"
        );
    }

    #[test]
    fn default_footer_stacked_below_sm_row_above() {
        // Below sm (500px): stacked reversed — Action above Cancel, both
        // full width.
        let mut h = Harness::new();
        h.set_screen(500.0, 600.0);
        let mut open = true;
        let mut cancel_rect = Rect::ZERO;
        let mut action_rect = Rect::ZERO;
        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("dff")
                    .show(ui, |ui| {
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogFooter::new().show(ui, |ui| {
                                cancel_rect = AlertDialogCancel::new("Cancel").show(ui).rect;
                                action_rect = AlertDialogAction::new("Delete").show(ui).rect;
                            });
                        });
                    });
            });
        }
        assert!(
            action_rect.bottom() <= cancel_rect.top(),
            "stacked: action above cancel: action {action_rect:?} cancel {cancel_rect:?}"
        );
        let panel_w = panel_max_width(AlertDialogSize::Default, true, 500.0);
        let inner_left = (500.0 - panel_w) / 2.0 + spacing::LG;
        let inner_right = (500.0 + panel_w) / 2.0 - spacing::LG;
        assert!(
            (cancel_rect.left() - inner_left).abs() < 1.0,
            "stacked: cancel spans inner width: {cancel_rect:?} vs {inner_left}"
        );
        assert!(
            (cancel_rect.right() - inner_right).abs() < 1.0,
            "stacked: cancel spans inner width: {cancel_rect:?} vs {inner_right}"
        );
        assert!(
            (action_rect.left() - inner_left).abs() < 1.0,
            "stacked: action spans inner width: {action_rect:?} vs {inner_left}"
        );
        assert!(
            (action_rect.right() - inner_right).abs() < 1.0,
            "stacked: action spans inner width: {action_rect:?} vs {inner_right}"
        );

        // At/above sm (800px): right-aligned row — Cancel left of Action.
        let mut h = Harness::new();
        let mut open = true;
        let mut cancel_rect = Rect::ZERO;
        let mut action_rect = Rect::ZERO;
        for _ in 0..3 {
            h.run(|ui| {
                AlertDialog::controlled(&mut open)
                    .id_salt("dff2")
                    .show(ui, |ui| {
                        AlertDialogContent::new().show(ui, |ui| {
                            AlertDialogFooter::new().show(ui, |ui| {
                                cancel_rect = AlertDialogCancel::new("Cancel").show(ui).rect;
                                action_rect = AlertDialogAction::new("Delete").show(ui).rect;
                            });
                        });
                    });
            });
        }
        assert!(
            (cancel_rect.top() - action_rect.top()).abs() < 1.0,
            "row: same row: cancel {cancel_rect:?} action {action_rect:?}"
        );
        assert!(
            cancel_rect.right() <= action_rect.left(),
            "row: cancel left of action: cancel {cancel_rect:?} action {action_rect:?}"
        );
        let panel_w = panel_max_width(AlertDialogSize::Default, true, 800.0);
        let inner_right = (800.0 + panel_w) / 2.0 - spacing::LG;
        assert!(
            (action_rect.right() - inner_right).abs() < 1.0,
            "row: action at inner right: {action_rect:?} vs {inner_right}"
        );
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let d = AlertDialog::controlled(&mut open).id_salt("x");
        assert_eq!(d.id_salt.as_deref(), Some("x"));
        let dflt = AlertDialog::new().default_open(true);
        assert!(dflt.default_open);
        let c = AlertDialogContent::new()
            .size(AlertDialogSize::Sm)
            .id_salt("c");
        assert_eq!(c.size, AlertDialogSize::Sm);
        let t = AlertDialogTrigger::new("hi").disabled(true).id_salt("t");
        assert!(t.disabled);
        let a = AlertDialogAction::new("go")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Lg)
            .id_salt("a");
        assert_eq!(a.variant, ButtonVariant::Destructive);
        assert_eq!(a.size, ButtonSize::Lg);
        let c2 = AlertDialogCancel::new("no")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .id_salt("n");
        assert_eq!(c2.variant, ButtonVariant::Ghost);
        assert_eq!(c2.size, ButtonSize::Sm);
        let _ = AlertDialogHeader::new();
        let _ = AlertDialogFooter::new();
        let _ = AlertDialogTitle::new("T");
        let _ = AlertDialogDescription::new("D");
        let _ = AlertDialogMedia::new(Icon::CircleAlert);
        let _ = AlertDialogMedia::with_content(|ui: &mut Ui| {
            ui.label("hi");
        });
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
