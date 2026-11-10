//! shadcn `hover-card.tsx` — a richer delayed hover panel.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/hover-card.tsx` (Radix
//! HoverCard) with the nova recipe: `bg-popover text-popover-foreground
//! ring-foreground/10 w-64 rounded-lg p-2.5 shadow-md`, `sideOffset = 4`.
//!
//! Hover behavior follows the Radix defaults: the card opens after a delay
//! (default 700 ms) and stays open while the pointer is over the trigger or
//! the card itself, closing after a shorter grace period (default 300 ms)
//! once the pointer leaves both. Escape closes immediately.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! HoverCard::new().id_salt("author").show(ui, |ui| {
//!     HoverCardTrigger::new("@nextjs").show(ui);
//!     HoverCardContent::new().show(ui, |ui| {
//!         ui.label("The React Framework — created and maintained by Vercel.");
//!     });
//! });
//! ```

use crate::components::popover::PopoverAlign;
use crate::theme::ShadcnTheme;
use crate::utils::animation::nova_overlay_progress;
use crate::utils::shadow;
use egui::emath::RectAlign;
use egui::{
    Context, CornerRadius, FontId, Frame, Id, Key, LayerId, Margin, Popup, PopupAnchor, Pos2, Rect,
    Response, Sense, Stroke, Ui,
};

/// Radix HoverCard default open delay.
pub const HOVER_CARD_DEFAULT_OPEN_DELAY_MS: u64 = 700;
/// Radix HoverCard default close delay.
pub const HOVER_CARD_DEFAULT_CLOSE_DELAY_MS: u64 = 300;
/// Default content width, matching the source `w-64`.
pub const HOVER_CARD_DEFAULT_WIDTH: f32 = 256.0;
/// Default distance between the trigger and the content (`sideOffset = 4`).
pub const HOVER_CARD_SIDE_OFFSET: f32 = 4.0;
/// nova padding: `p-2.5`.
const INNER_PADDING: i8 = 10;

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors popover.rs)
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_hover_card_stack")
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

#[derive(Clone, Copy, Debug)]
struct TriggerInfo {
    rect: Rect,
    layer_id: LayerId,
}

/// Per-instance state: trigger geometry plus the hover clock.
#[derive(Clone, Copy, Debug, Default)]
struct HoverCardState {
    trigger: Option<TriggerInfo>,
    hover_start: Option<f64>,
    close_start: Option<f64>,
    open: bool,
    open_delay_ms: Option<u64>,
    close_delay_ms: Option<u64>,
}

fn read_state(ctx: &Context, state_id: Id) -> HoverCardState {
    ctx.data_mut(|d| d.get_temp::<HoverCardState>(state_id).unwrap_or_default())
}

fn write_state(ctx: &Context, state_id: Id, state: HoverCardState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

// ---------------------------------------------------------------------------
// HoverCard (root)
// ---------------------------------------------------------------------------

/// The shadcn hover-card root. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct HoverCard {
    open_delay_ms: Option<u64>,
    close_delay_ms: Option<u64>,
    id_salt: Option<String>,
}

impl HoverCard {
    /// Creates a hover card root.
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the hover-open delay (Radix default 700 ms).
    pub fn open_delay_ms(mut self, delay_ms: u64) -> Self {
        self.open_delay_ms = Some(delay_ms);
        self
    }

    /// Overrides the close grace period (Radix default 300 ms).
    pub fn close_delay_ms(mut self, delay_ms: u64) -> Self {
        self.close_delay_ms = Some(delay_ms);
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
            None => ui.auto_id_with("shadcn_hover_card"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        state.open_delay_ms = self.open_delay_ms;
        state.close_delay_ms = self.close_delay_ms;
        write_state(&ctx, state_id, state);

        push_scope(ui, state_id);
        let inner = content(ui);
        pop_scope(ui);
        inner
    }
}

// ---------------------------------------------------------------------------
// HoverCardTrigger
// ---------------------------------------------------------------------------

/// The hover target of a [`HoverCard`]. By default renders as an inline
/// link-style label (the shadcn docs demos use `<a>`), or a custom widget.
pub struct HoverCardTrigger<'a> {
    label: Option<String>,
    widget: Option<crate::components::popover::TriggerWidget<'a>>,
    id_salt: Option<String>,
}

impl Default for HoverCardTrigger<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> HoverCardTrigger<'a> {
    /// Creates a trigger rendering an inline link-style label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            widget: None,
            id_salt: None,
        }
    }

    /// Creates a trigger rendering a custom widget.
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

    /// Renders the trigger and updates the hover clock.
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(state_id) = current_scope(ui) else {
            return link_label(
                ui,
                self.label.as_deref().unwrap_or(""),
                self.id_salt.clone(),
            );
        };

        let response = match self.widget {
            Some(w) => w(ui),
            None => link_label(
                ui,
                self.label.as_deref().unwrap_or(""),
                self.id_salt.clone(),
            ),
        };

        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        state.trigger = Some(TriggerInfo {
            rect: response.rect,
            layer_id: response.layer_id,
        });

        let now = ctx.input(|i| i.time);
        let open_delay = state
            .open_delay_ms
            .unwrap_or(HOVER_CARD_DEFAULT_OPEN_DELAY_MS) as f64
            / 1000.0;
        let close_delay = state
            .close_delay_ms
            .unwrap_or(HOVER_CARD_DEFAULT_CLOSE_DELAY_MS) as f64
            / 1000.0;

        let hovered = response.hovered();
        let popup_id = state_id.with("popup");
        let popup_hovered = ctx
            .read_response(popup_id)
            .map(|r| r.hovered())
            .unwrap_or(false);

        if hovered {
            state.close_start = None;
            if state.hover_start.is_none() {
                state.hover_start = Some(now);
            }
            if now - state.hover_start.unwrap_or(now) >= open_delay {
                state.open = true;
            }
        } else if popup_hovered {
            // Over the card: stay open, keep the return path warm.
            state.close_start = None;
        } else {
            // Left both: close after the grace period.
            state.hover_start = None;
            if state.open {
                let close_start = *state.close_start.get_or_insert(now);
                if now - close_start >= close_delay {
                    state.open = false;
                    state.close_start = None;
                }
            }
        }

        // Escape dismisses immediately (unless a modal owns Escape).
        let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
        let esc = ctx.input(|i| i.key_pressed(Key::Escape));
        if state.open && esc && !modal_open {
            state.open = false;
            state.hover_start = None;
            state.close_start = None;
        }

        write_state(&ctx, state_id, state);
        response
    }
}

/// Renders `@handle`-style inline link text like the demo's
/// `<Button variant="link">`: `text-primary`, fading to 80% and underlining
/// only while hovered (`hover:underline`).
fn link_label(ui: &mut Ui, text: &str, id_salt: Option<String>) -> Response {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, p.primary);
    let id = match id_salt {
        Some(salt) => ui.scope_id().with(salt),
        None => ui.auto_id_with("shadcn_hover_card_trigger"),
    };
    let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
    let response = ui.interact(rect, id, Sense::click() | Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Link, true, text.to_owned()));
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let color = if hovered {
            crate::theme::color_alpha(p.primary, 0.8)
        } else {
            p.primary
        };
        let galley = ui.painter().layout_no_wrap(
            text.to_owned(),
            FontId::proportional(crate::theme::tokens::type_scale::SM),
            color,
        );
        ui.painter().galley(rect.min, galley, color);
        if hovered {
            // `hover:underline`
            let y = rect.bottom() + 1.0;
            ui.painter().line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                egui::Stroke::new(1.0, color),
            );
        }
    }
    response
}

// ---------------------------------------------------------------------------
// HoverCardContent
// ---------------------------------------------------------------------------

/// Maps the shared [`PopoverAlign`] onto the hover card's bottom-edge
/// alignment (the source `align = "start" | "center" | "end"`).
fn hover_card_rect_align(align: PopoverAlign) -> RectAlign {
    match align {
        PopoverAlign::Start => RectAlign::BOTTOM_START,
        PopoverAlign::Center => RectAlign::BOTTOM,
        PopoverAlign::End => RectAlign::BOTTOM_END,
    }
}

/// The card body: `w-64 rounded-lg p-2.5 bg-popover ring-foreground/10
/// shadow-md` (nova) with a fade + slide-in animation, `sideOffset = 4`.
///
/// Stays open while the pointer is over the card, so content can contain
/// interactive widgets. The card opens below the trigger by default with the
/// source's `align = "center"`; see [`HoverCardContent::align`].
#[derive(Clone, Debug)]
pub struct HoverCardContent {
    width: Option<f32>,
    side_offset: Option<f32>,
    align: PopoverAlign,
    id_salt: Option<String>,
}

impl Default for HoverCardContent {
    fn default() -> Self {
        Self::new()
    }
}

impl HoverCardContent {
    /// Creates hover-card content with the source defaults (`w-64`,
    /// `align = "center"`, `sideOffset = 4`).
    pub fn new() -> Self {
        Self {
            width: None,
            side_offset: None,
            align: PopoverAlign::default(),
            id_salt: None,
        }
    }

    /// Overrides the content width (default: [`HOVER_CARD_DEFAULT_WIDTH`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides the trigger-to-content gap.
    pub fn side_offset(mut self, offset: f32) -> Self {
        self.side_offset = Some(offset);
        self
    }

    /// Sets the horizontal alignment below the trigger (the source's
    /// `align` prop, default `center`).
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

    /// Renders the card, or nothing while closed. Returns the popup area
    /// response while visible.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Option<Response> {
        let Some(state_id) = current_scope(ui) else {
            // Outside a HoverCard: render statically so mistakes stay visible.
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

        // nova `duration-100` fade (zoom-in-95 approximated by the fade).
        let progress = nova_overlay_progress(&ctx, anim_id, open);
        if !(open || progress.alpha > 0.002) {
            return None;
        }

        let width = self.width.unwrap_or(HOVER_CARD_DEFAULT_WIDTH);
        let margin = Margin::same(INNER_PADDING);
        let gap = self.side_offset.unwrap_or(HOVER_CARD_SIDE_OFFSET) + (1.0 - progress.alpha) * 8.0;

        let frame = Frame::new()
            .fill(p.popover)
            .stroke(Stroke::new(
                1.0,
                crate::theme::color_alpha(p.foreground, 0.10),
            ))
            .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
            .inner_margin(margin)
            .shadow(shadow::shadow_md(dark));

        let popup = Popup::new(
            popup_id,
            ctx.clone(),
            PopupAnchor::ParentRect(trigger.rect),
            trigger.layer_id,
        )
        .open(true)
        .align(hover_card_rect_align(self.align))
        .gap(gap)
        .width(width)
        .frame(Frame::new());

        let inner = popup.show(|ui| {
            ui.set_opacity(progress.alpha);
            frame.show(ui, |ui| {
                ui.set_min_width(width - margin.left as f32 - margin.right as f32);
                content(ui);
            });
        });
        inner.map(|r| r.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Vec2};

    /// Advances time so hover delays make progress across frames.
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
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("hover_card_test_ui"),
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

        fn state(&self, salt: &str) -> HoverCardState {
            let id = Id::unique("hover_card_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<HoverCardState>(id))
                .unwrap_or_default()
        }
    }

    #[test]
    fn constants_match_source() {
        // Radix defaults: openDelay 700, closeDelay 300. w-64 = 256.
        assert_eq!(HOVER_CARD_DEFAULT_OPEN_DELAY_MS, 700);
        assert_eq!(HOVER_CARD_DEFAULT_CLOSE_DELAY_MS, 300);
        assert_eq!(HOVER_CARD_DEFAULT_WIDTH, 256.0);
        assert_eq!(HOVER_CARD_SIDE_OFFSET, 4.0);
    }

    #[test]
    fn hover_opens_after_open_delay() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            HoverCard::new().id_salt("t1").show(ui, |ui| {
                trigger_rect = HoverCardTrigger::new("@nextjs").show(ui).rect;
                HoverCardContent::new().show(ui, |ui| {
                    ui.label("The React Framework");
                });
            });
        });
        assert!(!h.state("t1").open);

        // 700ms open delay at 100ms frames.
        for _ in 0..9 {
            h.hover(trigger_rect.center(), |ui| {
                HoverCard::new().id_salt("t1").show(ui, |ui| {
                    HoverCardTrigger::new("@nextjs").show(ui);
                    HoverCardContent::new().show(ui, |ui| {
                        ui.label("The React Framework");
                    });
                });
            });
        }
        assert!(h.state("t1").open, "hovering past the open delay opens it");
    }

    #[test]
    fn short_hover_stays_closed() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            HoverCard::new().id_salt("t2").show(ui, |ui| {
                trigger_rect = HoverCardTrigger::new("@nextjs").show(ui).rect;
                HoverCardContent::new().show(ui, |ui| {
                    ui.label("The React Framework");
                });
            });
        });

        // Two hover frames = 200ms < 700ms open delay, then leave.
        for _ in 0..2 {
            h.hover(trigger_rect.center(), |ui| {
                HoverCard::new().id_salt("t2").show(ui, |ui| {
                    HoverCardTrigger::new("@nextjs").show(ui);
                    HoverCardContent::new().show(ui, |ui| {
                        ui.label("The React Framework");
                    });
                });
            });
        }
        h.hover(Pos2::new(550.0, 550.0), |ui| {
            HoverCard::new().id_salt("t2").show(ui, |ui| {
                HoverCardTrigger::new("@nextjs").show(ui);
                HoverCardContent::new().show(ui, |ui| {
                    ui.label("The React Framework");
                });
            });
        });
        assert!(
            !h.state("t2").open,
            "hover shorter than the delay stays closed"
        );
    }

    #[test]
    fn closes_after_close_delay_when_left() {
        let mut h = Harness::new();
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            HoverCard::new().id_salt("t3").show(ui, |ui| {
                trigger_rect = HoverCardTrigger::new("@nextjs").show(ui).rect;
                HoverCardContent::new().show(ui, |ui| {
                    ui.label("The React Framework");
                });
            });
        });
        for _ in 0..9 {
            h.hover(trigger_rect.center(), |ui| {
                HoverCard::new().id_salt("t3").show(ui, |ui| {
                    HoverCardTrigger::new("@nextjs").show(ui);
                    HoverCardContent::new().show(ui, |ui| {
                        ui.label("The React Framework");
                    });
                });
            });
        }
        assert!(h.state("t3").open);

        // Leave: 300ms close delay; still open right after leaving, closed
        // after a few frames.
        h.hover(Pos2::new(550.0, 550.0), |ui| {
            HoverCard::new().id_salt("t3").show(ui, |ui| {
                HoverCardTrigger::new("@nextjs").show(ui);
                HoverCardContent::new().show(ui, |ui| {
                    ui.label("The React Framework");
                });
            });
        });
        assert!(h.state("t3").open, "inside the grace period it stays open");
        for _ in 0..4 {
            h.hover(Pos2::new(550.0, 550.0), |ui| {
                HoverCard::new().id_salt("t3").show(ui, |ui| {
                    HoverCardTrigger::new("@nextjs").show(ui);
                    HoverCardContent::new().show(ui, |ui| {
                        ui.label("The React Framework");
                    });
                });
            });
        }
        assert!(!h.state("t3").open, "past the grace period it closes");
    }

    #[test]
    fn builders_keep_fields() {
        let root = HoverCard::new()
            .open_delay_ms(100)
            .close_delay_ms(50)
            .id_salt("x");
        assert_eq!(root.open_delay_ms, Some(100));
        assert_eq!(root.close_delay_ms, Some(50));
        let c = HoverCardContent::new()
            .width(300.0)
            .side_offset(8.0)
            .align(PopoverAlign::End)
            .id_salt("c");
        assert_eq!(c.width, Some(300.0));
        assert_eq!(c.side_offset, Some(8.0));
        assert_eq!(c.align, PopoverAlign::End);
        let t = HoverCardTrigger::new("hi").id_salt("t");
        assert_eq!(t.id_salt.as_deref(), Some("t"));
    }

    #[test]
    fn align_maps_to_rect_aligns() {
        assert_eq!(
            hover_card_rect_align(PopoverAlign::Start),
            RectAlign::BOTTOM_START
        );
        assert_eq!(
            hover_card_rect_align(PopoverAlign::Center),
            RectAlign::BOTTOM
        );
        assert_eq!(
            hover_card_rect_align(PopoverAlign::End),
            RectAlign::BOTTOM_END
        );
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }

    #[test]
    fn content_outside_hover_card_renders_statically() {
        let mut h = Harness::new();
        let mut rendered = false;
        h.run(|ui| {
            HoverCardContent::new().show(ui, |_ui| {
                rendered = true;
            });
        });
        assert!(rendered);
    }
}
