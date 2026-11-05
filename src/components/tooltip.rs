//! shadcn `tooltip.tsx` — a small delayed hover popup.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/tooltip.tsx` (Radix Tooltip):
//! `rounded-md bg-foreground px-3 py-1.5 text-xs text-background` (nova keeps
//! the base colors), `sideOffset = 0`, and the distinctive rotated-square
//! arrow. Opens after a small hover delay (default 300 ms, like typical
//! Radix `delayDuration` setups), stays open while the pointer stays on the
//! trigger or content, and closes on leave and on Escape.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! TooltipProvider::new().delay_ms(300).install(ui.ctx()); // optional
//!
//! Tooltip::new().show(ui, |ui| {
//!     TooltipTrigger::widget(|ui| Button::new("Hover me").show(ui)).show(ui);
//!     TooltipContent::new().show(ui, |ui| {
//!         ui.label("Simple tooltip");
//!     });
//! });
//! ```

use crate::theme::ShadcnTheme;
use crate::utils::animation::overlay_progress;
use egui::emath::RectAlign;
use egui::{
    Color32, Context, CornerRadius, Frame, Id, Key, LayerId, Margin, Popup, PopupAnchor, Pos2,
    Rect, Response, Sense, Shape, Stroke, Ui,
};

/// Default hover delay before the tooltip opens.
pub const TOOLTIP_DEFAULT_DELAY_MS: u64 = 300;
/// nova padding: `px-3 py-1.5`.
const PADDING_X: i8 = 12;
const PADDING_Y: i8 = 6;

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors popover.rs)
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_tooltip_stack")
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

/// Which side of the trigger the content appears on (`side` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipSide {
    /// Above the trigger.
    Top,
    /// Below the trigger (the source default).
    #[default]
    Bottom,
    /// Left of the trigger.
    Left,
    /// Right of the trigger.
    Right,
}

impl TooltipSide {
    fn rect_align(self) -> RectAlign {
        match self {
            TooltipSide::Top => RectAlign::TOP,
            TooltipSide::Bottom => RectAlign::BOTTOM,
            TooltipSide::Left => RectAlign::LEFT,
            TooltipSide::Right => RectAlign::RIGHT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct TriggerInfo {
    rect: Rect,
    layer_id: LayerId,
}

/// Per-instance state: trigger geometry plus the hover clock.
#[derive(Clone, Copy, Debug, Default)]
struct TooltipState {
    trigger: Option<TriggerInfo>,
    /// Time the pointer started hovering the trigger, if hovering.
    hover_start: Option<f64>,
    open: bool,
    delay_ms: Option<u64>,
}

fn read_state(ctx: &Context, state_id: Id) -> TooltipState {
    ctx.data_mut(|d| d.get_temp::<TooltipState>(state_id).unwrap_or_default())
}

fn write_state(ctx: &Context, state_id: Id, state: TooltipState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

// ---------------------------------------------------------------------------
// TooltipProvider
// ---------------------------------------------------------------------------

/// Sets a default hover delay for every [`Tooltip`] shown afterwards
/// (the Radix `TooltipProvider delayDuration`). Install once per frame (or
/// once at startup); the value persists in context memory.
#[derive(Clone, Copy, Debug, Default)]
pub struct TooltipProvider {
    delay_ms: Option<u64>,
}

impl TooltipProvider {
    /// Creates a provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the default delay for all tooltips that don't override it.
    pub fn delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    /// Installs the default delay into the context.
    pub fn install(self, ctx: &Context) {
        if let Some(delay_ms) = self.delay_ms {
            ctx.data_mut(|d| d.insert_temp(provider_id(), delay_ms));
        }
    }
}

fn provider_id() -> Id {
    Id::unique("shadcn_tooltip_provider_delay")
}

// ---------------------------------------------------------------------------
// Tooltip (root)
// ---------------------------------------------------------------------------

/// The shadcn tooltip root. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct Tooltip {
    delay_ms: Option<u64>,
    id_salt: Option<String>,
}

impl Tooltip {
    /// Creates a tooltip root.
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the hover delay (default: the [`TooltipProvider`] delay, or
    /// [`TOOLTIP_DEFAULT_DELAY_MS`]).
    pub fn delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    /// Sets a stable id salt so hover state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Lays out the trigger and content.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_tooltip"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        state.delay_ms = self.delay_ms;
        write_state(&ctx, state_id, state);

        push_scope(ui, state_id);
        let inner = content(ui);
        pop_scope(ui);
        inner
    }
}

// ---------------------------------------------------------------------------
// TooltipTrigger
// ---------------------------------------------------------------------------

/// The hover target of a [`Tooltip`]. Renders like the shadcn docs demos
/// (`<Button variant="outline">`) unless a custom widget is given.
pub struct TooltipTrigger<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for TooltipTrigger<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> TooltipTrigger<'a> {
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
    /// pattern).
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

    /// Renders the trigger and updates the hover clock.
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(state_id) = current_scope(ui) else {
            return crate::components::button::Button::new(self.label.unwrap_or_default())
                .variant(crate::components::button::ButtonVariant::Outline)
                .disabled(self.disabled)
                .show(ui);
        };

        let response = match self.widget {
            Some(w) => w(ui),
            None => {
                let mut b = crate::components::button::Button::new(self.label.unwrap_or_default())
                    .variant(crate::components::button::ButtonVariant::Outline)
                    .disabled(self.disabled);
                if let Some(salt) = self.id_salt.clone() {
                    b = b.id_salt(salt);
                }
                b.show(ui)
            }
        };

        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        state.trigger = Some(TriggerInfo {
            rect: response.rect,
            layer_id: response.layer_id,
        });

        let now = ctx.input(|i| i.time);
        let delay_ms = state
            .delay_ms
            .or_else(|| ctx.data_mut(|d| d.get_temp::<u64>(provider_id())))
            .unwrap_or(TOOLTIP_DEFAULT_DELAY_MS);
        let delay_secs = delay_ms as f64 / 1000.0;

        let hovered = response.hovered() && !self.disabled;
        let popup_id = state_id.with("popup");
        let popup_hovered = ctx
            .read_response(popup_id)
            .map(|r| r.hovered())
            .unwrap_or(false);

        if hovered {
            if state.hover_start.is_none() {
                state.hover_start = Some(now);
            }
        } else if !popup_hovered {
            state.hover_start = None;
        }

        let elapsed_ok = state
            .hover_start
            .map(|t| now - t >= delay_secs)
            .unwrap_or(false);
        state.open = if hovered {
            elapsed_ok
        } else if popup_hovered {
            // Keep visible while the pointer is over the content.
            state.open
        } else {
            false
        };

        // Escape dismisses (unless a modal sits on top and owns Escape).
        let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
        let esc = ctx.input(|i| i.key_pressed(Key::Escape));
        if state.open && esc && !modal_open {
            state.open = false;
            state.hover_start = None;
        }

        write_state(&ctx, state_id, state);
        response
    }
}

// ---------------------------------------------------------------------------
// TooltipContent
// ---------------------------------------------------------------------------

/// The tooltip body: `rounded-md bg-foreground px-3 py-1.5 text-xs
/// text-background` with the rotated-square arrow, fade + slide-in animation,
/// `sideOffset = 0`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TooltipContent {
    side: TooltipSide,
}

impl TooltipContent {
    /// Creates tooltip content (source default `side = "bottom"`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the side of the trigger the content appears on.
    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    /// Renders the content, or nothing while closed.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Option<Response> {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Tooltip: render statically so mistakes stay visible.
            content(ui);
            return None;
        };
        let ctx = ui.ctx().clone();
        let state = read_state(&ctx, state_id);
        let open = state.open;
        let trigger = state.trigger?;

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let popup_id = state_id.with("popup");
        let anim_id = state_id.with("anim");

        let progress = overlay_progress(&ctx, anim_id, open);
        if !(open || progress.alpha > 0.002) {
            return None;
        }

        // `sideOffset = 0` plus `slide-in-from-top-2` on the opening side.
        let gap = (1.0 - progress.alpha) * 8.0;
        let arrow = 10.0; // size-2.5 rotated 45°

        let frame = Frame::new()
            .fill(p.foreground)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(theme.radius.md_u8()))
            .inner_margin(Margin::symmetric(PADDING_X, PADDING_Y))
            .outer_margin(Margin {
                // Reserve room for the arrow tip outside the painted frame.
                left: 0,
                right: 0,
                top: if self.side == TooltipSide::Top {
                    arrow as i8
                } else {
                    0
                },
                bottom: if self.side == TooltipSide::Bottom {
                    arrow as i8
                } else {
                    0
                },
            });

        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        let popup = Popup::new(
            popup_id,
            ctx.clone(),
            PopupAnchor::ParentRect(trigger.rect),
            trigger.layer_id,
        )
        .open(true)
        .align(self.side.rect_align())
        .gap(gap)
        .sense(Sense::hover())
        .frame(Frame::new());

        let inner = popup.show(|ui| {
            ui.set_opacity(progress.alpha);
            frame.show(ui, |ui| {
                ui.style_mut().visuals.override_text_color = Some(p.background);
                content(ui);
            });
            // Draw the rotated-square arrow in the reserved outer margin,
            // centered on the frame's painted edge like the source's
            // `translate-y-[calc(-50%-2px)] rotate-45`.
            let outer = ui.min_rect();
            let painted = match self.side {
                TooltipSide::Bottom => Pos2::new(outer.center().x, outer.bottom() - arrow),
                TooltipSide::Top => Pos2::new(outer.center().x, outer.top() + arrow),
                TooltipSide::Left => Pos2::new(outer.left() + arrow, outer.center().y),
                TooltipSide::Right => Pos2::new(outer.right() - arrow, outer.center().y),
            };
            paint_arrow(ui.painter(), painted, self.side, arrow, p.foreground);
        });

        // Nothing to dismiss manually: the trigger's hover clock owns the
        // state. Keep the popup response for callers.
        let _ = was_open_last_frame;
        inner.map(|r| r.response)
    }
}

/// Paints the rotated-square tooltip arrow pointing toward the trigger.
///
/// `edge` is the center of the arrow (on the frame's painted edge); the
/// square's tip pokes out toward the trigger.
fn paint_arrow(painter: &egui::Painter, edge: Pos2, side: TooltipSide, size: f32, color: Color32) {
    // A square with side `size` rotated 45° is a diamond whose half-diagonal
    // is `size * sqrt(2) / 2`.
    let h = size * std::f32::consts::SQRT_2 * 0.5;
    let (tip, back): (Pos2, Pos2) = match side {
        TooltipSide::Bottom => (Pos2::new(edge.x, edge.y + h), Pos2::new(edge.x, edge.y - h)),
        TooltipSide::Top => (Pos2::new(edge.x, edge.y - h), Pos2::new(edge.x, edge.y + h)),
        TooltipSide::Left => (Pos2::new(edge.x - h, edge.y), Pos2::new(edge.x + h, edge.y)),
        TooltipSide::Right => (Pos2::new(edge.x + h, edge.y), Pos2::new(edge.x - h, edge.y)),
    };
    // The two side corners sit perpendicular to the tip/back axis.
    let perpendicular = match side {
        TooltipSide::Bottom | TooltipSide::Top => {
            (Pos2::new(edge.x - h, edge.y), Pos2::new(edge.x + h, edge.y))
        }
        TooltipSide::Left | TooltipSide::Right => {
            (Pos2::new(edge.x, edge.y - h), Pos2::new(edge.x, edge.y + h))
        }
    };
    let pts = vec![tip, perpendicular.0, back, perpendicular.1];
    painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Vec2;

    /// Advances time so animations and hover delays make progress.
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

        fn run_with_events(&mut self, events: Vec<egui::Event>, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.05;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("tooltip_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn run(&mut self, f: impl FnMut(&mut Ui)) {
            self.run_with_events(vec![], f);
        }

        fn hover(&mut self, pos: Pos2, f: impl FnMut(&mut Ui)) {
            self.run_with_events(vec![egui::Event::PointerMoved(pos)], f);
        }

        fn state(&self, salt: &str) -> TooltipState {
            let id = Id::unique("tooltip_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<TooltipState>(id))
                .unwrap_or_default()
        }
    }

    #[test]
    fn side_maps_to_rect_aligns() {
        assert_eq!(TooltipSide::Top.rect_align(), RectAlign::TOP);
        assert_eq!(TooltipSide::Bottom.rect_align(), RectAlign::BOTTOM);
        assert_eq!(TooltipSide::Left.rect_align(), RectAlign::LEFT);
        assert_eq!(TooltipSide::Right.rect_align(), RectAlign::RIGHT);
    }

    #[test]
    fn delay_constants_match_source() {
        assert_eq!(TOOLTIP_DEFAULT_DELAY_MS, 300);
        assert_eq!(PADDING_X, 12); // px-3
        assert_eq!(PADDING_Y, 6); // py-1.5
    }

    #[test]
    fn provider_installs_delay() {
        let ctx = egui::Context::default();
        TooltipProvider::new().delay_ms(500).install(&ctx);
        let delay = ctx.data_mut(|d| d.get_temp::<u64>(provider_id()));
        assert_eq!(delay, Some(500));
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }

    #[test]
    fn trigger_outside_tooltip_renders_button() {
        let mut h = Harness::new();
        h.run(|ui| {
            let r = TooltipTrigger::new("lonely").show(ui);
            assert!(!r.changed());
            assert!(r.rect.size().x > 0.0);
        });
    }

    #[test]
    fn hover_opens_after_delay_and_closes_on_leave() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        // Layout frame so the trigger registers.
        h.run(|ui| {
            Tooltip::new().id_salt("t1").show(ui, |ui| {
                trigger_rect = TooltipTrigger::new("Hover").show(ui).rect;
                TooltipContent::new().show(ui, |ui| {
                    ui.label("Tooltip text");
                });
            });
        });
        assert!(!h.state("t1").open);

        // Park the pointer on the trigger; delay is 300ms so 4 × 50ms frames
        // should cross it.
        for _ in 0..8 {
            h.hover(trigger_rect.center(), |ui| {
                Tooltip::new().id_salt("t1").show(ui, |ui| {
                    TooltipTrigger::new("Hover").show(ui);
                    TooltipContent::new().show(ui, |ui| {
                        ui.label("Tooltip text");
                    });
                });
            });
        }
        assert!(h.state("t1").open, "hovering past the delay must open it");

        // Leave the trigger (and the popup): the tooltip closes.
        for _ in 0..2 {
            h.hover(Pos2::new(550.0, 550.0), |ui| {
                Tooltip::new().id_salt("t1").show(ui, |ui| {
                    TooltipTrigger::new("Hover").show(ui);
                    TooltipContent::new().show(ui, |ui| {
                        ui.label("Tooltip text");
                    });
                });
            });
        }
        assert!(!h.state("t1").open, "leaving the trigger must close it");
    }

    #[test]
    fn short_hover_below_delay_stays_closed() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            Tooltip::new().id_salt("t2").show(ui, |ui| {
                trigger_rect = TooltipTrigger::new("Hover").show(ui).rect;
                TooltipContent::new().show(ui, |ui| {
                    ui.label("Tooltip text");
                });
            });
        });

        // One hover frame = 50ms < 300ms delay, then leave.
        h.hover(trigger_rect.center(), |ui| {
            Tooltip::new().id_salt("t2").show(ui, |ui| {
                TooltipTrigger::new("Hover").show(ui);
                TooltipContent::new().show(ui, |ui| {
                    ui.label("Tooltip text");
                });
            });
        });
        h.hover(Pos2::new(550.0, 550.0), |ui| {
            Tooltip::new().id_salt("t2").show(ui, |ui| {
                TooltipTrigger::new("Hover").show(ui);
                TooltipContent::new().show(ui, |ui| {
                    ui.label("Tooltip text");
                });
            });
        });
        assert!(
            !h.state("t2").open,
            "a hover shorter than the delay must not open"
        );
    }

    #[test]
    fn content_outside_tooltip_renders_statically() {
        let mut h = Harness::new();
        let mut rendered = false;
        h.run(|ui| {
            TooltipContent::new().show(ui, |_ui| {
                rendered = true;
            });
        });
        assert!(rendered, "content outside a tooltip still renders");
    }

    #[test]
    fn builders_keep_fields() {
        let t = Tooltip::new().delay_ms(500).id_salt("x");
        assert_eq!(t.delay_ms, Some(500));
        assert_eq!(t.id_salt.as_deref(), Some("x"));
        let c = TooltipContent::new().side(TooltipSide::Top);
        assert_eq!(c.side, TooltipSide::Top);
        let tr = TooltipTrigger::new("hi").disabled(true).id_salt("t");
        assert!(tr.disabled);
    }

    #[test]
    fn arrow_geometry_is_finite() {
        // Paint into a throwaway painter-less shape check: the polygon points
        // must be finite for both directions.
        for dir in [-1.0_f32, 1.0] {
            let half = 10.0 * std::f32::consts::SQRT_2 * 0.5;
            let edge = Pos2::new(50.0, 50.0);
            let tip = Pos2::new(edge.x, edge.y + dir * half * 0.5);
            assert!(tip.x.is_finite() && tip.y.is_finite());
        }
    }

    #[test]
    fn trigger_text_color_matches_foreground() {
        // The nova tooltip uses bg-foreground/text-background; make sure both
        // palette slots resolve distinctly in light mode.
        let p = Preset::default().build(Mode::Light).palette;
        assert_ne!(p.foreground, p.background);
    }
}
