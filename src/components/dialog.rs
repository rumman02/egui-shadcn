//! shadcn `dialog.tsx` — a modal dialog with a dimmed backdrop.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/dialog.tsx` (Radix Dialog),
//! both recipes: the base recipe (`bg-background`, `rounded-lg`, `p-6`,
//! `sm:max-w-lg`, `bg-black/50` backdrop, 200ms) and the nova recipe
//! (`bg-popover text-popover-foreground ring-foreground/10 rounded-xl p-4
//! text-sm gap-4 sm:max-w-sm` over a `bg-black/10` backdrop, 100ms). The
//! panel keeps `shadow-lg` under BOTH recipes: nova is silent on shadow
//! for this slot, so the base value survives (the settled LAYER ruling).
//!
//! Behavior follows the Radix defaults the wrapper does not override
//! (frozen S0 spec §10): the dialog is modal, focus moves into the panel
//! on open and returns to the trigger on close, Tab cycles inside, and it
//! closes on Escape, on backdrop click, and via the top-right X or any
//! [`DialogClose`]. Backdrop and panel fade together (fade-only: egui
//! widgets do not scale — proposed deviation D2); the close X is a 16px
//! icon box with opacity-only hover, not a ghost button. The backdrop has
//! no blur (proposed deviation D1 — the original's own
//! `supports-backdrop-filter:` guard renders the same flat backdrop where
//! blur is unsupported).
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
use crate::theme::fonts::{medium, semibold};
use crate::theme::tokens::{spacing, type_scale};
use crate::theme::{color_alpha, ShadcnTheme, Style};
use crate::utils::shadow::{paint_shadow_resolved, ShadowSize, ShadowSlot};
use egui::epaint::StrokeKind;
use egui::{
    Align, Color32, Context, CornerRadius, Direction, Frame, Id, Layout, Modal, Pos2, Rect,
    Response, Sense, Stroke, Ui, UiBuilder, Vec2, WidgetInfo,
};

/// nova max width: `sm:max-w-sm` (384 px).
pub const DIALOG_MAX_WIDTH: f32 = 384.0;
/// base max width: `sm:max-w-lg` (512 px).
pub const DIALOG_BASE_MAX_WIDTH: f32 = 512.0;
/// The `sm:` breakpoint (Tailwind v4 default `--breakpoint-sm: 40rem`).
/// Panel width, header alignment, and footer direction all change at this
/// viewport width (§3–§4; deviation D4 keys it off the window width).
const SM_BREAKPOINT: f32 = 640.0;
/// nova ring: `ring-foreground/10`.
const RING_ALPHA: f32 = 0.10;
/// nova backdrop: `bg-black/10`.
const BACKDROP_ALPHA_NOVA: f32 = 0.10;
/// base backdrop: `bg-black/50`.
const BACKDROP_ALPHA_BASE: f32 = 0.50;
/// Close X box: the bare `size-4` icon box (§7).
const CLOSE_X_SIZE: f32 = 16.0;
/// Close X resting opacity: `opacity-70`.
const CLOSE_X_OPACITY: f32 = 0.70;
/// nova animation duration: `duration-100`.
const ANIM_NOVA_SECS: f32 = 0.1;
/// base content animation duration: `duration-200`.
const ANIM_BASE_SECS: f32 = 0.2;
/// Floor for the panel width so degenerate viewports still lay out.
const MIN_PANEL_WIDTH: f32 = 80.0;

/// Panel width (§3): below `sm` the panel is the viewport minus 2rem with
/// no other cap; at/above `sm` it is capped by the recipe max width. An
/// explicit `width` override caps at every width (it is the caller's
/// `className` max-width, not an `sm:` rule).
fn panel_width(nova: bool, width_override: Option<f32>, screen_w: f32) -> f32 {
    let avail = (screen_w - 32.0).max(MIN_PANEL_WIDTH);
    match width_override {
        Some(w) => w.min(avail).max(MIN_PANEL_WIDTH),
        None if screen_w < SM_BREAKPOINT => avail,
        None => {
            let cap = if nova {
                DIALOG_MAX_WIDTH
            } else {
                DIALOG_BASE_MAX_WIDTH
            };
            cap.min(avail).max(MIN_PANEL_WIDTH)
        }
    }
}

/// Fade progress 0..1 for the open/close animation at the recipe duration.
/// Mirrors `nova_overlay_progress` (same egui value animator, `alpha = t`)
/// with a recipe duration; the foundation helper is nova-only (100ms).
/// OPEN-8: easing is the animator's default. OPEN-9: the base overlay
/// duration is unresolved, so the overlay runs at the content duration to
/// preserve the §9 rule that backdrop and panel animate together.
fn overlay_alpha(ctx: &Context, id: Id, open: bool, duration_secs: f32) -> f32 {
    let target = if open { 1.0 } else { 0.0 };
    ctx.animate_value_with_time(id, target, duration_secs)
}

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

/// Per-instance state: the open flag, the previous frame's open flag (for
/// open/close transition handling), and the widget that had focus before
/// the dialog opened (for focus return on close, §10).
#[derive(Clone, Copy, Debug, Default)]
struct DialogState {
    open: bool,
    was_open: bool,
    return_focus: Option<Id>,
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
            // Controlled: sync the open flag only, preserving the
            // transition bookkeeping (`was_open`, `return_focus`) that
            // `DialogContent` maintains.
            let mut state = read_state(&ctx, state_id);
            state.open = *open;
            write_state(&ctx, state_id, state);
        } else {
            let exists = ctx.data_mut(|d| d.get_temp::<DialogState>(state_id).is_some());
            if !exists {
                write_state(
                    &ctx,
                    state_id,
                    DialogState {
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

    /// Disables the trigger. A disabled trigger does not open the dialog.
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
    disabled: bool,
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
            disabled: false,
            id_salt: None,
        }
    }

    /// Creates a close control rendering a custom widget (Radix `asChild`).
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w)),
            disabled: false,
            id_salt: None,
        }
    }

    /// Disables the close control (`disabled:pointer-events-none`): clicks
    /// on it do not close the dialog.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the close control; `changed()` when the dialog closed.
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

/// The modal panel: centered, `bg-background` (base) / `bg-popover` (nova)
/// with a 1px border plus `ring-foreground/10` under nova and `shadow-lg`
/// under both recipes, a close X at the recipe inset, over a dimmed
/// backdrop. Closes on Escape and backdrop click; focus moves into the
/// panel on open and returns to the trigger on close.
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
    /// Creates dialog content with the source defaults (recipe max width,
    /// close button shown).
    pub fn new() -> Self {
        Self {
            width: None,
            show_close_button: true,
            id_salt: None,
        }
    }

    /// Overrides the panel max width (default: the recipe cap —
    /// [`DIALOG_MAX_WIDTH`] under nova, [`DIALOG_BASE_MAX_WIDTH`] under
    /// base — clamped to the window minus 2rem like
    /// `max-w-[calc(100%-2rem)]`). The override is the caller's `className`
    /// max-width, so it caps at every viewport width, unlike the recipe
    /// cap which only applies at/above `sm` (§3).
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
    pub fn show(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui)) -> Option<Response> {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Dialog: render statically so mistakes stay visible.
            content(ui);
            return None;
        };
        let ctx = ui.ctx().clone();

        // Focus transitions (§10). Handled before the early return so the
        // close transition fires even after the closing animation finished.
        let mut state = read_state(&ctx, state_id);
        let open = state.open;
        let opening = open && !state.was_open;
        let closing = !open && state.was_open;
        if opening {
            // Remember who had focus so it can be restored on close.
            state.return_focus = ctx.memory(|m| m.focused());
        }
        if closing {
            if let Some(id) = state.return_focus.take() {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
        state.was_open = open;
        write_state(&ctx, state_id, state);

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let nova = theme.style == Style::Nova;
        let dark = theme.mode.is_dark();

        let modal_id = state_id.with("modal");
        let anim_id = state_id.with("anim");
        let panel_id = state_id.with("panel");

        // Fade-only animation (proposed deviation D2 — egui widgets do not
        // scale): backdrop and panel share the recipe duration and the same
        // progress value, so they animate together (§9).
        let duration = if nova { ANIM_NOVA_SECS } else { ANIM_BASE_SECS };
        let alpha = overlay_alpha(&ctx, anim_id, open, duration);
        if !(open || alpha > 0.002) {
            return None;
        }

        let pad = if nova { spacing::LG } else { spacing::XL };
        let screen_w = ctx.content_rect().width();
        let max_width = panel_width(nova, self.width, screen_w);
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
        // Flat backdrop, no blur: proposed deviation D1 (egui has no
        // backdrop blur; the original's own `supports-backdrop-filter:`
        // guard renders the same flat backdrop where blur is unsupported).
        let backdrop = color_alpha(Color32::BLACK, backdrop_alpha * alpha);
        let x_inset = if nova { spacing::SM } else { spacing::LG };

        let modal = Modal::new(modal_id)
            .backdrop_color(backdrop)
            .frame(Frame::new());

        let show_close = self.show_close_button;
        let modal_resp = modal.show(&ctx, |ui| {
            ui.set_opacity(alpha);

            // Measure the content height with an invisible pass so the
            // shadow, fill, border, and ring can be painted behind the real
            // content with the exact panel rect (same pattern as
            // `footer_strip`: widgets in the measuring pass are disabled,
            // so no callbacks can double-fire).
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
            // The panel holds focus while open (§10): focus moves into the
            // dialog on open without presuming which descendant Radix
            // targets (OPEN-1 — the Content-element resolution). Focusable
            // but not interactive, with no visible outline (`outline-none`,
            // §3/§8). This interact runs first, so Tab order starts here.
            // Tab cannot leave the dialog: egui only offers focus to
            // widgets on or above the top modal layer (OPEN-2).
            let _panel_resp = ui.interact(panel_rect, panel_id, Sense::focusable_noninteractive());

            // Shadow first, behind the body. The faded painter fades it in
            // lockstep with the panel. `shadow-lg` under both recipes
            // (LAYER: nova is silent for this slot, §6/§14).
            paint_shadow_resolved(
                ui.painter(),
                panel_rect,
                radius,
                ShadowSize::Lg,
                theme.style,
                ShadowSlot::DialogContent,
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

            if show_close {
                // `absolute top-2 right-2` (nova) / `top-4 right-4`
                // (base) relative to the panel: the 16px icon box (§7).
                // No alpha gate — the X fades WITH the panel (§9); gating
                // it on animation progress was the staging defect.
                let x_rect = Rect::from_min_size(
                    Pos2::new(
                        panel_rect.right() - x_inset - CLOSE_X_SIZE,
                        panel_rect.top() + x_inset,
                    ),
                    Vec2::splat(CLOSE_X_SIZE),
                );
                let x_id = state_id.with("dialog_close_x");
                let x_resp = ui.interact(x_rect, x_id, Sense::click());
                x_resp.widget_info(|| {
                    WidgetInfo::labeled(egui::Role::Button, true, "Close".to_owned())
                });
                if ui.is_rect_visible(x_rect) {
                    // `focus:ring-2 focus:ring-ring focus:ring-offset-2`:
                    // 2px ring with a 2px offset in the background token,
                    // `rounded-xs`. Painted on any focus (egui has no
                    // focus-visible distinction).
                    if x_resp.has_focus() {
                        let corner = CornerRadius::same(2);
                        ui.painter().rect_stroke(
                            x_rect,
                            corner,
                            Stroke::new(4.0, p.ring),
                            StrokeKind::Outside,
                        );
                        ui.painter().rect_stroke(
                            x_rect,
                            corner,
                            Stroke::new(2.0, p.background),
                            StrokeKind::Outside,
                        );
                    }
                    // Opacity-only hover (`opacity-70`, `hover:opacity-100`,
                    // `transition-opacity`): no hover fill in either recipe.
                    let fg = if x_resp.hovered() {
                        p.foreground
                    } else {
                        color_alpha(p.foreground, CLOSE_X_OPACITY)
                    };
                    Icon::X.paint(ui.painter(), x_rect, fg, 2.0 * CLOSE_X_SIZE / 24.0);
                }
                if x_resp.clicked() {
                    let mut s = read_state(&ctx, state_id);
                    s.open = false;
                    write_state(&ctx, state_id, s);
                }
            }
        });

        if opening {
            // Focus moves into the dialog (§10) — unless a widget inside
            // already claimed it this frame (e.g. an autofocus target),
            // which takes precedence over the panel default.
            let current = ctx.memory(|m| m.focused());
            if current == state.return_focus {
                ctx.memory_mut(|m| m.request_focus(panel_id));
            }
        }

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

/// Horizontal text alignment for header children: the header's `text-center`
/// below `sm` / `sm:text-left` rule (§4) is carried by the header layout's
/// cross alignment, which title/description read back here. Only a
/// top-down layout's cross alignment is a horizontal alignment.
fn header_halign(ui: &Ui) -> Align {
    let layout = ui.layout();
    if layout.main_dir == Direction::TopDown && layout.cross_align == Align::Center {
        Align::Center
    } else {
        Align::LEFT
    }
}

/// `flex flex-col gap-2` header inside a [`DialogContent`]: text centered
/// below `sm`, left-aligned at/above 640px (§4).
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
        let wide = ui.ctx().content_rect().width() >= SM_BREAKPOINT;
        let align = if wide { Align::Min } else { Align::Center };
        ui.with_layout(Layout::top_down(align), |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = spacing::SM;
            content(ui)
        })
        .inner
    }
}

/// Dialog title: base `text-lg leading-none font-semibold`, nova
/// `text-base leading-none font-medium` (§5). Long titles wrap (§12).
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
        let nova = theme.style == Style::Nova;
        // Colour inherits the panel text (no colour class on Title itself).
        let (size, font, color) = if nova {
            (
                type_scale::BASE,
                medium(type_scale::BASE),
                p.popover_foreground,
            )
        } else {
            (type_scale::LG, semibold(type_scale::LG), p.foreground)
        };
        let format = egui::TextFormat {
            font_id: font,
            color,
            line_height: Some(size),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = ui.available_width();
        job.halign = header_halign(ui);
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            // Align the galley's own rect with the allocated rect: with a
            // centered halign the galley rect is centered on the origin,
            // so painting at `rect.min` would shift the text by half its
            // width.
            let pos = rect.min - galley.rect.min.to_vec2();
            ui.painter().galley(pos, galley, color);
        }
        response
    }
}

/// `text-sm text-muted-foreground` dialog description, wrapping, default
/// line-height, both recipes (§5). (The nova link styling applies to links
/// inside the description; this plain-text API renders no links.)
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
        let font = egui::FontId::proportional(type_scale::SM);
        let format = egui::TextFormat {
            font_id: font,
            color: p.muted_foreground,
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = ui.available_width();
        job.halign = header_halign(ui);
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            let pos = rect.min - galley.rect.min.to_vec2();
            ui.painter().galley(pos, galley, p.muted_foreground);
        }
        response
    }
}

/// Dialog footer: `flex flex-col-reverse gap-2` below `sm` (children stack
/// vertically in reverse DOM order), `sm:flex-row sm:justify-end` at/above
/// 640px (right-aligned row in DOM order). Under nova the footer is a
/// full-bleed strip (`bg-muted/50 -mx-4 -mb-4 rounded-b-xl border-t p-4`);
/// the base recipe has no strip styling (§4).
///
/// Like the source, [`DialogFooter::show_close_button`] appends an outline
/// "Close" button (a [`DialogClose`], so clicking it closes the dialog)
/// AFTER the children in DOM order: at the top of the stacked footer below
/// `sm`, at the right end of the row at/above `sm`.
#[derive(Clone, Debug, Default)]
pub struct DialogFooter {
    show_close_button: bool,
}

impl DialogFooter {
    /// Creates a dialog footer.
    pub fn new() -> Self {
        Self {
            show_close_button: false,
        }
    }

    /// Appends an outline "Close" button that closes the dialog (the source's
    /// `showCloseButton` prop, default `false`).
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Renders the footer.
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
        let show_close = self.show_close_button;
        dialog_footer(ui, |ui| {
            let r = content(ui);
            if show_close {
                DialogClose::new("Close").show(ui);
            }
            r
        })
    }
}

/// The dialog footer layout: recipe strip (nova only) + responsive direction
/// (§4). Separate from [`footer_strip`] (kept for `alert_dialog.rs`): the
/// dialog footer needs the base no-strip variant and the `sm:` direction
/// switch, which the shared strip does not provide.
fn dialog_footer<R>(ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> R {
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
        // Stack height is direction-independent; measure top-down.
        measure.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = gap;
            content(ui)
        });
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
            painter.rect_filled(strip, corner, color_alpha(p.muted, 0.5));
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
/// order at/above `sm`, a bottom-up stack (reverse DOM order) below `sm`.
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
        let mut child = ui.new_child(UiBuilder::new().max_rect(inner));
        child
            .with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = gap;
                content(ui)
            })
            .inner
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
    use crate::theme::{Mode, Preset, Style};
    use egui::Pos2;

    struct Harness {
        ctx: egui::Context,
        time: f64,
        screen_w: f32,
        screen_h: f32,
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

        /// The test screen; the modal anchors to the context screen rect, so
        /// the harness pins it to match the test Ui.
        fn screen(&self) -> Rect {
            Rect::from_min_size(Pos2::ZERO, Vec2::new(self.screen_w, self.screen_h))
        }

        fn set_screen(&mut self, w: f32, h: f32) {
            self.screen_w = w;
            self.screen_h = h;
        }

        fn raw_input(&self, events: Vec<egui::Event>) -> egui::RawInput {
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(self.screen()),
                events,
                ..Default::default()
            }
        }

        fn frame(&mut self, events: Vec<egui::Event>, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = self.raw_input(events);
            let screen = self.screen();
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("dialog_test_ui"),
                    UiBuilder::new().max_rect(screen),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn run(&mut self, f: impl FnMut(&mut Ui)) {
            self.frame(vec![], f);
        }

        fn click(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.frame(
                    vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    &mut f,
                );
            }
        }

        /// Sends one key press for one frame.
        fn press_key(&mut self, key: egui::Key, f: impl FnMut(&mut Ui)) {
            self.frame(
                vec![egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                    physical_key: None,
                }],
                f,
            );
        }

        fn focused(&self) -> Option<Id> {
            self.ctx.memory(|m| m.focused())
        }

        fn state(&self, salt: &str) -> DialogState {
            let id = Id::unique("dialog_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<DialogState>(id))
                .unwrap_or_default()
        }

        /// The widget rect from the last pass, if the widget was used.
        fn widget_rect(&self, id: Id) -> Option<Rect> {
            self.ctx.read_response(id).map(|r| r.rect)
        }
    }

    fn test_ui_id() -> Id {
        Id::unique("dialog_test_ui")
    }

    #[test]
    fn constants_match_recipes() {
        assert!((DIALOG_MAX_WIDTH - 384.0).abs() < f32::EPSILON); // sm:max-w-sm
        assert!((DIALOG_BASE_MAX_WIDTH - 512.0).abs() < f32::EPSILON); // sm:max-w-lg
        assert!((SM_BREAKPOINT - 640.0).abs() < f32::EPSILON);
        assert!((RING_ALPHA - 0.10).abs() < f32::EPSILON);
        assert!((BACKDROP_ALPHA_NOVA - 0.10).abs() < f32::EPSILON); // bg-black/10
        assert!((BACKDROP_ALPHA_BASE - 0.50).abs() < f32::EPSILON); // bg-black/50
        assert!((CLOSE_X_SIZE - 16.0).abs() < f32::EPSILON); // size-4
        assert!((CLOSE_X_OPACITY - 0.70).abs() < f32::EPSILON); // opacity-70
        assert!((ANIM_NOVA_SECS - 0.1).abs() < f32::EPSILON); // duration-100
        assert!((ANIM_BASE_SECS - 0.2).abs() < f32::EPSILON); // duration-200
    }

    #[test]
    fn panel_width_rules() {
        // At/above sm: recipe cap.
        assert!((panel_width(true, None, 800.0) - 384.0).abs() < f32::EPSILON);
        assert!((panel_width(false, None, 800.0) - 512.0).abs() < f32::EPSILON);
        assert!((panel_width(true, None, 640.0) - 384.0).abs() < f32::EPSILON);
        assert!((panel_width(false, None, 640.0) - 512.0).abs() < f32::EPSILON);
        // Below sm: viewport minus 2rem, no cap (matrix N-05).
        assert!((panel_width(true, None, 600.0) - 568.0).abs() < f32::EPSILON);
        assert!((panel_width(false, None, 416.0) - 384.0).abs() < f32::EPSILON);
        // Explicit override caps at every width.
        assert!((panel_width(true, Some(560.0), 800.0) - 560.0).abs() < f32::EPSILON);
        assert!((panel_width(true, Some(560.0), 500.0) - 468.0).abs() < f32::EPSILON);
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

        h.press_key(egui::Key::Escape, |ui| {
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
    fn dialog_close_disabled_does_not_close() {
        let mut h = Harness::new();
        let mut open = true;
        let mut close_rect = Rect::ZERO;

        for _ in 0..3 {
            h.run(|ui| {
                Dialog::controlled(&mut open).id_salt("d4b").show(ui, |ui| {
                    DialogContent::new().show(ui, |ui| {
                        DialogTitle::new("Title").show(ui);
                        DialogFooter::new().show(ui, |ui| {
                            close_rect = DialogClose::new("Close").disabled(true).show(ui).rect;
                        });
                    });
                });
            });
        }
        assert!(open);

        h.click(close_rect.center(), |ui| {
            Dialog::controlled(&mut open).id_salt("d4b").show(ui, |ui| {
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                    DialogFooter::new().show(ui, |ui| {
                        DialogClose::new("Close").disabled(true).show(ui);
                    });
                });
            });
        });
        assert!(open, "a disabled DialogClose must not close the dialog");
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
        let cl = DialogClose::new("bye").disabled(true).id_salt("bye");
        assert!(cl.disabled);
        let _ = DialogHeader::new();
        let f = DialogFooter::new().show_close_button(true);
        assert!(f.show_close_button);
        let _ = DialogTitle::new("T");
        let _ = DialogDescription::new("D");
    }

    #[test]
    fn panel_geometry_nova_centered_384() {
        let mut h = Harness::new();
        let mut open = true;
        for _ in 0..4 {
            h.run(|ui| {
                Dialog::controlled(&mut open)
                    .id_salt("geo1")
                    .show(ui, |ui| {
                        DialogContent::new().show(ui, |ui| {
                            DialogTitle::new("Title").show(ui);
                        });
                    });
            });
        }
        let panel_id = test_ui_id().with("geo1").with("panel");
        let rect = h.widget_rect(panel_id).expect("panel widget used");
        assert!((rect.width() - 384.0).abs() < 1.0, "nova width: {rect:?}");
        assert!(
            (rect.center().x - 400.0).abs() < 1.0,
            "centered x: {rect:?}"
        );
        assert!(
            (rect.center().y - 300.0).abs() < 1.0,
            "centered y: {rect:?}"
        );
    }

    #[test]
    fn panel_geometry_base_512() {
        let mut h = Harness::with_preset(Preset::default().with_style(Style::Vega));
        let mut open = true;
        for _ in 0..4 {
            h.run(|ui| {
                Dialog::controlled(&mut open)
                    .id_salt("geo2")
                    .show(ui, |ui| {
                        DialogContent::new().show(ui, |ui| {
                            DialogTitle::new("Title").show(ui);
                        });
                    });
            });
        }
        let panel_id = test_ui_id().with("geo2").with("panel");
        let rect = h.widget_rect(panel_id).expect("panel widget used");
        assert!((rect.width() - 512.0).abs() < 1.0, "base width: {rect:?}");
    }

    #[test]
    fn panel_width_below_sm_is_viewport_minus_2rem() {
        let mut h = Harness::new();
        h.set_screen(500.0, 600.0);
        let mut open = true;
        for _ in 0..4 {
            h.run(|ui| {
                Dialog::controlled(&mut open)
                    .id_salt("geo3")
                    .show(ui, |ui| {
                        DialogContent::new().show(ui, |ui| {
                            DialogTitle::new("Title").show(ui);
                        });
                    });
            });
        }
        let panel_id = test_ui_id().with("geo3").with("panel");
        let rect = h.widget_rect(panel_id).expect("panel widget used");
        assert!((rect.width() - 468.0).abs() < 1.0, "narrow width: {rect:?}");
    }

    #[test]
    fn title_geometry_by_recipe_leading_none() {
        // nova: 16px text, leading-none → 16px box.
        let mut h = Harness::new();
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = DialogTitle::new("Hello").show(ui).rect;
        });
        assert!(
            (rect.height() - 16.0).abs() < 0.01,
            "nova title height: {rect:?}"
        );

        // base: 18px text, leading-none → 18px box.
        let mut h = Harness::with_preset(Preset::default().with_style(Style::Vega));
        h.run(|ui| {
            rect = DialogTitle::new("Hello").show(ui).rect;
        });
        assert!(
            (rect.height() - 18.0).abs() < 0.01,
            "base title height: {rect:?}"
        );
    }

    #[test]
    fn header_alignment_narrow_centers_wide_left() {
        // Below sm: header text centers.
        let mut h = Harness::new();
        h.set_screen(400.0, 600.0);
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            DialogHeader::new().show(ui, |ui| {
                rect = DialogTitle::new("Hi").show(ui).rect;
            });
        });
        assert!(
            (rect.center().x - 200.0).abs() < 1.0,
            "narrow title centers: {rect:?}"
        );

        // At/above sm: header text left-aligns.
        let mut h = Harness::new();
        h.run(|ui| {
            DialogHeader::new().show(ui, |ui| {
                rect = DialogTitle::new("Hi").show(ui).rect;
            });
        });
        assert!(rect.min.x.abs() < 1.0, "wide title left-aligns: {rect:?}");
    }

    #[test]
    fn footer_row_dom_order_and_close_last() {
        let mut h = Harness::new();
        let mut a = Rect::ZERO;
        let mut b = Rect::ZERO;
        for _ in 0..2 {
            h.run(|ui| {
                DialogFooter::new().show_close_button(true).show(ui, |ui| {
                    a = Button::new("A").show(ui).rect;
                    b = Button::new("B").show(ui).rect;
                });
            });
        }
        // DOM order left to right in the row.
        assert!(a.center().x < b.center().x, "A left of B: {a:?} {b:?}");
        // The appended Close button sits right of B: A is not at the right
        // edge of the 800px flow (the old right_to_left order put A
        // rightmost).
        assert!(
            a.right() < 700.0,
            "Close button is right of the content: {a:?} {b:?}"
        );
    }

    #[test]
    fn footer_stack_reverses_dom_order() {
        let mut h = Harness::new();
        h.set_screen(400.0, 600.0);
        let mut a = Rect::ZERO;
        let mut b = Rect::ZERO;
        for _ in 0..2 {
            h.run(|ui| {
                DialogFooter::new().show_close_button(true).show(ui, |ui| {
                    a = Button::new("A").show(ui).rect;
                    b = Button::new("B").show(ui).rect;
                });
            });
        }
        // flex-col-reverse: the first DOM child lands at the bottom.
        assert!(
            a.center().y > b.center().y,
            "A below B in the stack: {a:?} {b:?}"
        );
        // The appended Close button renders at the TOP of the stack, above B.
        assert!(b.min.y > 50.0, "Close button sits above B: {a:?} {b:?}");
    }

    #[test]
    fn close_x_geometry_nova() {
        let mut h = Harness::new();
        let mut open = true;
        for _ in 0..4 {
            h.run(|ui| {
                Dialog::controlled(&mut open)
                    .id_salt("xgeo")
                    .show(ui, |ui| {
                        DialogContent::new().show(ui, |ui| {
                            DialogTitle::new("Title").show(ui);
                        });
                    });
            });
        }
        let panel_id = test_ui_id().with("xgeo").with("panel");
        let x_id = test_ui_id().with("xgeo").with("dialog_close_x");
        let panel = h.widget_rect(panel_id).expect("panel widget used");
        let x = h.widget_rect(x_id).expect("close X widget used");
        assert!((x.width() - 16.0).abs() < 0.01, "X width: {x:?}");
        assert!((x.height() - 16.0).abs() < 0.01, "X height: {x:?}");
        assert!(
            (x.right() - (panel.right() - 8.0)).abs() < 1.0,
            "X 8px from panel right: panel {panel:?} x {x:?}"
        );
        assert!(
            (x.top() - (panel.top() + 8.0)).abs() < 1.0,
            "X 8px from panel top: panel {panel:?} x {x:?}"
        );
    }

    #[test]
    fn focus_moves_into_dialog_on_open() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            Dialog::new().id_salt("foc1").show(ui, |ui| {
                trigger_rect = DialogTrigger::new("Open").show(ui).rect;
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        h.click(trigger_rect.center(), |ui| {
            Dialog::new().id_salt("foc1").show(ui, |ui| {
                DialogTrigger::new("Open").show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        });
        assert!(h.state("foc1").open);
        let panel_id = test_ui_id().with("foc1").with("panel");
        assert_eq!(
            h.focused(),
            Some(panel_id),
            "focus moves into the dialog on open"
        );
    }

    #[test]
    fn focus_returns_to_trigger_on_close() {
        let mut h = Harness::new();
        let trigger_rect = std::cell::Cell::new(Rect::ZERO);
        let mut show = |ui: &mut Ui| {
            Dialog::new().id_salt("foc2").show(ui, |ui| {
                trigger_rect.set(DialogTrigger::new("Open").id_salt("trig").show(ui).rect);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        };
        h.run(&mut show);
        let pos = trigger_rect.get().center();
        h.click(pos, &mut show);
        assert!(h.state("foc2").open);

        h.press_key(egui::Key::Escape, &mut show);
        assert!(!h.state("foc2").open);
        // One settle frame so the close transition's focus return lands.
        h.run(&mut show);
        let trigger_id = test_ui_id().with("trig");
        assert_eq!(
            h.focused(),
            Some(trigger_id),
            "focus returns to the trigger on close"
        );
    }

    #[test]
    fn tab_cycles_inside_dialog() {
        let mut h = Harness::new();
        let trigger_rect = std::cell::Cell::new(Rect::ZERO);
        let outside_id = std::cell::Cell::new(Id::NULL);
        let mut show = |ui: &mut Ui| {
            outside_id.set(Button::new("Outside").show(ui).id);
            Dialog::new().id_salt("foc3").show(ui, |ui| {
                trigger_rect.set(DialogTrigger::new("Open").show(ui).rect);
                DialogContent::new().show(ui, |ui| {
                    Button::new("In1").show(ui);
                    Button::new("In2").show(ui);
                });
            });
        };
        h.run(&mut show);
        let pos = trigger_rect.get().center();
        h.click(pos, &mut show);
        assert!(h.state("foc3").open);

        let x_id = test_ui_id().with("foc3").with("dialog_close_x");
        let mut saw_x = false;
        for _ in 0..8 {
            h.press_key(egui::Key::Tab, &mut show);
            let focus = h.focused();
            assert_ne!(
                focus,
                Some(outside_id.get()),
                "Tab must never leave the dialog"
            );
            if focus == Some(x_id) {
                saw_x = true;
            }
        }
        assert!(saw_x, "Tab cycling reaches the close X");
    }

    #[test]
    fn close_x_keyboard_enter_closes() {
        let mut h = Harness::new();
        let trigger_rect = std::cell::Cell::new(Rect::ZERO);
        let mut show = |ui: &mut Ui| {
            Dialog::new().id_salt("foc4").show(ui, |ui| {
                trigger_rect.set(DialogTrigger::new("Open").show(ui).rect);
                DialogContent::new().show(ui, |ui| {
                    DialogTitle::new("Title").show(ui);
                });
            });
        };
        h.run(&mut show);
        let pos = trigger_rect.get().center();
        h.click(pos, &mut show);
        assert!(h.state("foc4").open);

        // Tab until the close X has focus.
        let x_id = test_ui_id().with("foc4").with("dialog_close_x");
        for _ in 0..10 {
            if h.focused() == Some(x_id) {
                break;
            }
            h.press_key(egui::Key::Tab, &mut show);
        }
        assert_eq!(h.focused(), Some(x_id), "Tab reaches the close X");

        h.press_key(egui::Key::Enter, &mut show);
        assert!(
            !h.state("foc4").open,
            "Enter on the focused close X closes the dialog"
        );
    }
}
