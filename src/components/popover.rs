//! shadcn `popover.tsx` — an anchored, non-modal popup.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/popover.tsx` (Radix Popover)
//! with the nova recipe: `bg-popover text-popover-foreground ring-foreground/10
//! rounded-lg p-2.5 shadow-md`, default width `w-72` (288 px) and
//! `sideOffset = 4`.
//!
//! The root owns the open state — uncontrolled (persisted per `id_salt`) or
//! controlled through a `&mut bool` with [`Popover::controlled`]. The trigger
//! toggles on click; the content closes on Escape, on a click outside, and
//! fades/slides in like the source's `data-[state=open]:animate-in
//! fade-in-0 zoom-in-95 slide-in-from-top-2`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Popover::new().id_salt("dimensions").show(ui, |ui| {
//!     PopoverTrigger::new("Open popover").show(ui);
//!     PopoverContent::new().show(ui, |ui| {
//!         PopoverTitle::new("Dimensions").show(ui);
//!         PopoverDescription::new("Set the width and height").show(ui);
//!     });
//! });
//!
//! // Controlled:
//! let mut open = false;
//! Popover::controlled(&mut open).id_salt("p2").show(ui, |ui| {
//!     PopoverTrigger::widget(|ui| Button::new("Click").show(ui)).show(ui);
//!     PopoverContent::new().show(ui, |ui| { /* … */ });
//! });
//! ```

use crate::components::button::{Button, ButtonVariant};
use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::animation::overlay_progress;
use crate::utils::shadow;
use egui::emath::RectAlign;
use egui::{
    Context, CornerRadius, Frame, Id, Key, LayerId, Margin, Popup, PopupAnchor, Rect, Response,
    Sense, Stroke, Ui,
};

/// A custom trigger widget (`PopoverTrigger asChild`): returns the widget's
/// response for anchoring and toggling.
pub type TriggerWidget<'a> = Box<dyn FnOnce(&mut Ui) -> Response + 'a>;

/// Default content width, matching the source `w-72`.
pub const POPOVER_DEFAULT_WIDTH: f32 = 288.0;
/// Default distance between the trigger and the content (`sideOffset = 4`).
pub const POPOVER_SIDE_OFFSET: f32 = 4.0;
/// nova padding: `p-2.5`.
const INNER_PADDING: i8 = 10;

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors collapsible.rs): children find the active root.
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_popover_stack")
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

/// Where the content aligns against the trigger (`align = "start" | "center" |
/// "end"` in the source).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    /// Aligned to the trigger's left edge (`align="start"`).
    Start,
    /// Centered on the trigger (`align="center"`, the source default).
    #[default]
    Center,
    /// Aligned to the trigger's right edge (`align="end"`).
    End,
}

impl PopoverAlign {
    fn rect_align(self) -> RectAlign {
        match self {
            PopoverAlign::Start => RectAlign::BOTTOM_START,
            PopoverAlign::Center => RectAlign::BOTTOM,
            PopoverAlign::End => RectAlign::BOTTOM_END,
        }
    }
}

/// Per-instance state: open flag plus the last trigger geometry.
#[derive(Clone, Copy, Debug, Default)]
struct PopoverState {
    open: bool,
    trigger: Option<TriggerInfo>,
}

#[derive(Clone, Copy, Debug)]
struct TriggerInfo {
    rect: Rect,
    id: Id,
    layer_id: LayerId,
}

fn read_state(ctx: &Context, state_id: Id) -> PopoverState {
    ctx.data_mut(|d| d.get_temp::<PopoverState>(state_id).unwrap_or_default())
}

/// The default trigger look: the shadcn docs demos use
/// `<Button variant="outline">` inside `PopoverTrigger asChild`.
fn outline_trigger(label: &Option<String>, disabled: bool, id_salt: Option<String>) -> Button {
    let mut b = Button::new(label.clone().unwrap_or_default())
        .variant(ButtonVariant::Outline)
        .disabled(disabled);
    if let Some(salt) = id_salt {
        b = b.id_salt(salt);
    }
    b
}

fn write_state(ctx: &Context, state_id: Id, state: PopoverState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

// ---------------------------------------------------------------------------
// Popover (root)
// ---------------------------------------------------------------------------

/// The shadcn popover root. See the [module docs](self) for an example.
///
/// Uncontrolled by default: the open state is stored in the egui context and
/// persists per `id_salt`. Use [`Popover::controlled`] to own the state.
#[derive(Debug)]
pub struct Popover<'a> {
    controlled: Option<&'a mut bool>,
    default_open: bool,
    id_salt: Option<String>,
}

impl Default for Popover<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Popover<'a> {
    /// Creates an uncontrolled popover (starts closed).
    pub fn new() -> Self {
        Self {
            controlled: None,
            default_open: false,
            id_salt: None,
        }
    }

    /// Creates a popover whose open state is owned by the caller. The value
    /// is read before and written back after each frame, so clicks,
    /// Escape and click-outside all update the caller's bool.
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

    /// Lays out the trigger and content, then renders the popup if open.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_popover"),
        };

        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        // Controlled mode wins over whatever is stored.
        if let Some(open) = self.controlled.as_deref() {
            state.open = *open;
            write_state(&ctx, state_id, state);
        } else {
            let exists = ctx.data_mut(|d| d.get_temp::<PopoverState>(state_id).is_some());
            if !exists {
                state.open = self.default_open;
                write_state(&ctx, state_id, state);
            }
        }

        push_scope(ui, state_id);
        let inner = content(ui);
        pop_scope(ui);

        // Controlled write-back: pick up toggles and dismissals.
        if let Some(out) = self.controlled {
            *out = read_state(&ctx, state_id).open;
        }
        inner
    }
}

// ---------------------------------------------------------------------------
// PopoverTrigger
// ---------------------------------------------------------------------------

/// Toggles the innermost [`Popover`] when clicked and anchors its content.
///
/// By default it renders like the shadcn docs demos (`<Button variant="outline">`);
/// pass a custom widget with [`PopoverTrigger::widget`].
pub struct PopoverTrigger<'a> {
    label: Option<String>,
    widget: Option<TriggerWidget<'a>>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for PopoverTrigger<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> PopoverTrigger<'a> {
    /// Creates a trigger rendering an outline button with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            widget: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Creates a trigger rendering a custom widget (the Radix `asChild`
    /// pattern). The widget's response is used for anchoring and toggling.
    pub fn widget(w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        Self {
            label: None,
            widget: Some(Box::new(w) as TriggerWidget<'a>),
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

    /// Renders the trigger. The returned response is `changed()` whenever the
    /// popover open state flipped.
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Popover: render an inert outline button.
            return outline_trigger(&self.label, self.disabled, self.id_salt.clone()).show(ui);
        };

        let mut response = match self.widget {
            Some(w) => w(ui),
            None => outline_trigger(&self.label, self.disabled, self.id_salt.clone()).show(ui),
        };

        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        if response.clicked() && !self.disabled {
            state.open = !state.open;
            response.mark_changed();
        }
        state.trigger = Some(TriggerInfo {
            rect: response.rect,
            id: response.id,
            layer_id: response.layer_id,
        });
        write_state(&ctx, state_id, state);
        response
    }
}

// ---------------------------------------------------------------------------
// PopoverContent
// ---------------------------------------------------------------------------

/// The popup body of a [`Popover`]: `w-72 rounded-lg p-2.5 bg-popover
/// ring-foreground/10 shadow-md` (nova) with a fade + slide-in animation.
///
/// Renders nothing when the popover is closed. Closes on Escape and on any
/// click outside the content or trigger.
#[derive(Clone, Debug)]
pub struct PopoverContent {
    width: Option<f32>,
    side_offset: Option<f32>,
    align: PopoverAlign,
    id_salt: Option<String>,
}

impl Default for PopoverContent {
    fn default() -> Self {
        Self::new()
    }
}

impl PopoverContent {
    /// Creates popover content with the source defaults (`w-72`, `align =
    /// "center"`, `sideOffset = 4`).
    pub fn new() -> Self {
        Self {
            width: None,
            side_offset: None,
            align: PopoverAlign::default(),
            id_salt: None,
        }
    }

    /// Overrides the content width (default: [`POPOVER_DEFAULT_WIDTH`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides the trigger-to-content gap (default: [`POPOVER_SIDE_OFFSET`]).
    pub fn side_offset(mut self, offset: f32) -> Self {
        self.side_offset = Some(offset);
        self
    }

    /// Sets the horizontal alignment against the trigger.
    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.align = align;
        self
    }

    /// Sets a stable id salt (kept for API symmetry; the content follows the
    /// root's state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the content, or nothing when closed. Returns the popup area
    /// response while visible.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Option<Response> {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Popover: render statically so mistakes stay visible.
            content(ui);
            return None;
        };
        let ctx = ui.ctx().clone();
        let state = read_state(&ctx, state_id);
        let open = state.open;
        let trigger = state.trigger?;

        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let popup_id = state_id.with("popup");
        let anim_id = state_id.with("anim");

        // `data-[state=open]:fade-in-0 zoom-in-95` (zoom approximated by the
        // fade) plus `slide-in-from-top-2` on the opening side.
        let progress = overlay_progress(&ctx, anim_id, open);
        if !(open || progress.alpha > 0.002) {
            return None;
        }

        let width = self.width.unwrap_or(POPOVER_DEFAULT_WIDTH);
        let margin = Margin::same(INNER_PADDING);
        let gap = self.side_offset.unwrap_or(POPOVER_SIDE_OFFSET) + (1.0 - progress.alpha) * 8.0;

        let frame = Frame::new()
            .fill(p.popover)
            .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
            .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
            .inner_margin(margin)
            .shadow(shadow::shadow_md(dark));

        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        let popup = Popup::new(
            popup_id,
            ctx.clone(),
            PopupAnchor::ParentRect(trigger.rect),
            trigger.layer_id,
        )
        .open(true)
        .align(self.align.rect_align())
        .gap(gap)
        .width(width)
        .frame(Frame::new())
        .anchor_widget(trigger.id);

        let inner = popup.show(|ui| {
            ui.set_opacity(progress.alpha);
            frame.show(ui, |ui| {
                ui.set_min_width(width - margin.left as f32 - margin.right as f32);
                content(ui);
            });
        });

        // Dismissal: Escape or a click outside the popup (and outside the
        // trigger — clicking the trigger toggles instead). Skipped on the
        // opening frame and while closing, and when a modal sits on top so
        // dialogs keep priority on Escape.
        if let Some(ref r) = inner {
            let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
            let esc = !modal_open && ctx.input(|i| i.key_pressed(Key::Escape));
            let outside = r.response.clicked_elsewhere();
            if open && was_open_last_frame && (outside || esc) {
                let mut state = read_state(&ctx, state_id);
                state.open = false;
                write_state(&ctx, state_id, state);
            }
        }
        inner.map(|r| r.response)
    }
}

// ---------------------------------------------------------------------------
// PopoverHeader / PopoverTitle / PopoverDescription
// ---------------------------------------------------------------------------

/// `flex flex-col gap-0.5 text-sm` header inside a [`PopoverContent`].
#[derive(Clone, Debug, Default)]
pub struct PopoverHeader {
    _private: (),
}

impl PopoverHeader {
    /// Creates a popover header.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the header with a 2 px column gap.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        ui.vertical(|ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::HALF;
            content(ui)
        })
        .inner
    }
}

/// `font-medium` popover title (`text-sm`, nova).
#[derive(Clone, Debug)]
pub struct PopoverTitle {
    text: String,
}

impl PopoverTitle {
    /// Creates a popover title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let font = egui::FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text, font, theme.palette.popover_foreground);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter()
                .galley(rect.min, galley, theme.palette.popover_foreground);
        }
        response
    }
}

/// `text-muted-foreground` popover description, wrapping.
#[derive(Clone, Debug)]
pub struct PopoverDescription {
    text: String,
}

impl PopoverDescription {
    /// Creates a popover description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let font = egui::FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui.painter().layout(
            self.text,
            font,
            theme.palette.muted_foreground,
            ui.available_width(),
        );
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter()
                .galley(rect.min, galley, theme.palette.muted_foreground);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::button::Button;
    use crate::theme::{Mode, Preset};
    use egui::Vec2;

    /// Advances time so animations make progress across frames.
    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn run(&mut self, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("popover_test_ui"),
                    egui::UiBuilder::new().max_rect(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(600.0, 600.0),
                    )),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        /// Presses and releases the primary button at `pos` over two frames.
        fn click(&mut self, pos: egui::Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.time += 0.1;
                let input = egui::RawInput {
                    time: Some(self.time),
                    events: vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    ..Default::default()
                };
                let mut out = self.ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("popover_test_ui"),
                        egui::UiBuilder::new().max_rect(Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(600.0, 600.0),
                        )),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        fn state(&self, salt: &str) -> PopoverState {
            let id = Id::unique("popover_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<PopoverState>(id))
                .unwrap_or_default()
        }
    }

    #[test]
    fn trigger_click_toggles_open_state() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            Popover::new().id_salt("t1").show(ui, |ui| {
                trigger_rect = PopoverTrigger::new("Open").show(ui).rect;
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(!h.state("t1").open, "default is closed");

        h.click(trigger_rect.center(), |ui| {
            Popover::new().id_salt("t1").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(h.state("t1").open, "clicking the trigger opens it");

        h.click(trigger_rect.center(), |ui| {
            Popover::new().id_salt("t1").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(!h.state("t1").open, "clicking again closes it");
    }

    #[test]
    fn controlled_mode_write_back() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            Popover::controlled(&mut open).id_salt("c1").show(ui, |ui| {
                trigger_rect = PopoverTrigger::new("Open").show(ui).rect;
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(!open);

        h.click(trigger_rect.center(), |ui| {
            Popover::controlled(&mut open).id_salt("c1").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(open, "trigger click write-back reaches the caller");

        // Caller closes it from the outside.
        open = false;
        h.run(|ui| {
            Popover::controlled(&mut open).id_salt("c1").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(!open);
    }

    #[test]
    fn click_outside_closes_open_popover() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        // Layout frame so the trigger is registered before any click.
        h.run(|ui| {
            Popover::new().id_salt("out").show(ui, |ui| {
                trigger_rect = PopoverTrigger::new("Open").show(ui).rect;
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });

        // Open it.
        h.click(trigger_rect.center(), |ui| {
            Popover::new().id_salt("out").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(h.state("out").open);

        // One more frame so the popup was visible last frame, then click far
        // away from both trigger and popup.
        h.run(|ui| {
            Popover::new().id_salt("out").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        h.click(egui::Pos2::new(550.0, 550.0), |ui| {
            Popover::new().id_salt("out").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(!h.state("out").open, "clicking outside must close it");
    }

    #[test]
    fn escape_closes_open_popover() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        // Layout frame so the trigger is registered before any click.
        h.run(|ui| {
            Popover::new().id_salt("esc").show(ui, |ui| {
                trigger_rect = PopoverTrigger::new("Open").show(ui).rect;
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });

        h.click(trigger_rect.center(), |ui| {
            Popover::new().id_salt("esc").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        assert!(h.state("esc").open);

        h.run(|ui| {
            Popover::new().id_salt("esc").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });

        h.time += 0.1;
        let input = egui::RawInput {
            time: Some(h.time),
            events: vec![egui::Event::Key {
                key: Key::Escape,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
                physical_key: None,
            }],
            ..Default::default()
        };
        let mut out = h.ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("popover_test_ui"),
                egui::UiBuilder::new().max_rect(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(600.0, 600.0),
                )),
            );
            Popover::new().id_salt("esc").show(ui, |ui| {
                PopoverTrigger::new("Open").show(ui);
                PopoverContent::new().show(ui, |ui| {
                    ui.label("body");
                });
            });
        });
        out.textures_delta.clear();
        assert!(!h.state("esc").open, "Escape must close the popover");
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let root = Popover::controlled(&mut open)
            .default_open(true)
            .id_salt("x");
        assert_eq!(root.id_salt.as_deref(), Some("x"));
        assert!(root.default_open);
        let t = PopoverTrigger::new("hi").disabled(true).id_salt("t");
        assert!(t.disabled);
        let c = PopoverContent::new()
            .width(100.0)
            .side_offset(2.0)
            .align(PopoverAlign::End)
            .id_salt("c");
        assert_eq!(c.width, Some(100.0));
        assert_eq!(c.side_offset, Some(2.0));
        assert_eq!(c.align, PopoverAlign::End);
    }

    #[test]
    fn geometry_constants_match_source() {
        // w-72 = 288px, sideOffset = 4
        assert_eq!(POPOVER_DEFAULT_WIDTH, 288.0);
        assert_eq!(POPOVER_SIDE_OFFSET, 4.0);
    }

    #[test]
    fn align_maps_to_rect_aligns() {
        assert_eq!(PopoverAlign::Start.rect_align(), RectAlign::BOTTOM_START);
        assert_eq!(PopoverAlign::Center.rect_align(), RectAlign::BOTTOM);
        assert_eq!(PopoverAlign::End.rect_align(), RectAlign::BOTTOM_END);
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }

    #[test]
    fn trigger_outside_popover_renders_button() {
        let mut h = Harness::new();
        h.run(|ui| {
            let r = PopoverTrigger::new("lonely").show(ui);
            assert!(!r.changed(), "no popover in scope, must be inert");
            let _ = Button::new("noop");
        });
    }
}
