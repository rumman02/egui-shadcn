//! shadcn `message-scroller.tsx` — an auto-scrolling chat container.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/message-scroller.tsx`: a
//! `MessageScroller` viewport (`flex flex-col overflow-y-auto`) whose
//! content stacks with `gap-8`, that pins to the bottom while new messages
//! stream in and backs off the moment the reader scrolls up. A
//! `MessageScrollerButton` (`size="icon-sm"`, `rounded-md`, `bottom-4`,
//! arrow down — or `top-4`/arrow up for `direction="start"`) appears while
//! scrolled away from its end; one click jumps back and re-engages
//! auto-follow.
//!
//! Web-only details are approximated: egui's `stick_to_bottom` handles the
//! pin/unpin bookkeeping, the smooth (animated) scroll is not portable, and
//! `MessageScrollerItem`'s `scrollAnchor` turn-anchoring is not ported
//! (noted in PROGRESS.md). Scroll fades are egui-native (`spacing.scroll.fade`,
//! strengthened here to match the source's `scroll-fade-b`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! MessageScroller::new().height(400.0).show(ui, |ui| {
//!     MessageGroup::new().show(ui, |ui| { /* bubbles ... */ });
//! });
//! ```

use crate::components::button::ButtonSize;
use crate::icons::Icon;
use crate::theme::ShadcnTheme;
use egui::{Align, InnerResponse, Pos2, Rect, Ui, Vec2};

/// Per-instance scroll bookkeeping, keyed by the scroller [`Id`].
#[derive(Clone, Copy, Debug)]
struct ScrollerState {
    /// Auto-follow the bottom (egui `stick_to_bottom`).
    follow: bool,
    /// A jump-to-bottom requested by the scroll button (executed next pass).
    pending_jump: bool,
    /// Vertical offset from the previous pass (detects user scroll-up).
    last_offset: f32,
}

impl Default for ScrollerState {
    fn default() -> Self {
        Self {
            follow: true,
            pending_jump: false,
            last_offset: 0.0,
        }
    }
}

/// Is the viewport pinned to the bottom of the content? Pure and testable.
pub(crate) fn is_at_bottom(offset_y: f32, content_height: f32, viewport_height: f32) -> bool {
    let max_offset = (content_height - viewport_height).max(0.0);
    offset_y >= max_offset - 1.0
}

/// Re-engages auto-follow when at the bottom, disengages when the user
/// scrolled up. Pure and testable.
pub(crate) fn next_follow(follow: bool, at_bottom: bool, scrolled_up: bool) -> bool {
    if at_bottom {
        true
    } else if scrolled_up {
        false
    } else {
        follow
    }
}

/// Which end the jump button targets (the source `direction` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageScrollerDirection {
    /// Jump to the newest message (`bottom-4`, arrow down; the default).
    #[default]
    End,
    /// Jump to the oldest message (`top-4`, arrow up).
    Start,
}

/// Is the viewport at the top of the content? Pure and testable.
pub(crate) fn is_at_top(offset_y: f32) -> bool {
    offset_y <= 1.0
}

/// The floating jump button (`MessageScrollerButton`): a `rounded-md`
/// `icon-sm` button (`bg-background border-border hover:bg-muted`) that the
/// [`MessageScroller`] anchors at its viewport's `bottom-4` (End) or `top-4`
/// (Start) center. Show/hide animates over 200ms (`scale-95` + fade, sliding
/// one height past its edge while hidden, like the source's
/// `translate-y-full`).
#[derive(Clone, Debug)]
pub struct MessageScrollerButton {
    direction: MessageScrollerDirection,
    active: bool,
    id_salt: Option<String>,
}

impl Default for MessageScrollerButton {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageScrollerButton {
    /// Creates the scroll-to-end button (shown while `active`).
    pub fn new() -> Self {
        Self {
            direction: MessageScrollerDirection::End,
            active: true,
            id_salt: None,
        }
    }

    /// Sets which end the button jumps to (and its anchor edge).
    pub fn direction(mut self, direction: MessageScrollerDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Whether the button is currently shown (`data-active`). Pass `false`
    /// (every frame) so the 200ms hide animation can play instead of
    /// popping the button out.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the button centered on `anchor` (the *active* position) on a
    /// floating layer. Call it every frame so the show/hide animation can
    /// play. Returns the click response (not clickable while fading out).
    pub fn show_at(self, ui: &Ui, anchor: Pos2) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let side = ButtonSize::IconSm.height(); // size-8
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            // Scoped to the enclosing ui instead of a global unique id, so
            // two salt-less buttons never collide.
            None => ui.scope_id().with("shadcn_message_scroller_button"),
        };
        // `data-[active=false]:scale-95 opacity-0 duration-200`.
        let t = crate::utils::animation::animate_value(
            ui.ctx(),
            id.with("visibility"),
            if self.active { 1.0 } else { 0.0 },
            crate::utils::animation::Duration::Md,
        );

        // Paint on a floating middle layer so it overlaps the content.
        let painter = ui
            .ctx()
            .layer_painter(egui::LayerId::new(egui::Order::Middle, id.with("layer")));
        if t > 0.002 {
            // Hover state comes from the previous pass (one-frame lag,
            // standard for deferred container painting).
            let hovered = ui
                .ctx()
                .read_response(id)
                .map(|r| r.hovered())
                .unwrap_or(false);
            let fill = if hovered { p.muted } else { p.background };
            let slide = (1.0 - t)
                * side
                * match self.direction {
                    MessageScrollerDirection::End => 1.0,
                    MessageScrollerDirection::Start => -1.0,
                };
            let scale = 0.95 + 0.05 * t;
            let rect =
                Rect::from_center_size(anchor + Vec2::new(0.0, slide), Vec2::splat(side * scale));
            let fade = |c: egui::Color32| c.gamma_multiply(t);
            // The source renders a real `<Button size="icon-sm">` whose base
            // is `rounded-md` (not a circle).
            let radius = theme.radius.md_u8();
            painter.rect_filled(rect, radius, fade(fill));
            painter.rect_stroke(
                rect,
                radius,
                egui::Stroke::new(1.0, fade(p.border)),
                egui::epaint::StrokeKind::Inside,
            );
            let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(16.0 * scale));
            let icon = match self.direction {
                MessageScrollerDirection::End => Icon::ArrowDown,
                // `data-[direction=start]:[&_svg]:rotate-180`.
                MessageScrollerDirection::Start => Icon::ArrowUp,
            };
            icon.paint(&painter, icon_rect, fade(p.foreground), 2.0);
        }

        // Clickable only while mostly visible (`pointer-events-none` while
        // hidden). `Sense::click()` is focusable in egui, matching the
        // source's real focusable `<button>`.
        let (rect, sense) = if t > 0.5 {
            (
                Rect::from_center_size(anchor, Vec2::splat(side)),
                egui::Sense::click(),
            )
        } else {
            (Rect::NOTHING, egui::Sense::hover())
        };
        let response = ui.interact(rect, id, sense);
        // Button base `focus-visible:ring-3 focus-visible:ring-ring/50`.
        if response.has_focus() {
            crate::utils::focus::paint_ring(
                &painter,
                rect.expand(1.0),
                crate::theme::color_alpha(p.ring, 0.5),
                3.0,
                theme.radius.md() + 1.0,
            );
        }
        response
    }
}

/// What [`MessageScroller::show`] reports for the current pass.
#[derive(Clone, Copy, Debug)]
pub struct MessageScrollerResponse {
    /// True when the viewport is pinned to the bottom of the content.
    pub at_bottom: bool,
    /// True when the viewport is at the top of the content.
    pub at_top: bool,
    /// True when auto-follow is engaged (new content scrolls into view).
    pub following: bool,
}

/// The message scroller. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct MessageScroller {
    height: Option<f32>,
    gap: f32,
    direction: MessageScrollerDirection,
    id_salt: Option<String>,
}

impl Default for MessageScroller {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageScroller {
    /// Creates a scroller that fills the available height.
    pub fn new() -> Self {
        Self {
            height: None,
            gap: 32.0, // gap-8 between content groups
            direction: MessageScrollerDirection::End,
            id_salt: None,
        }
    }

    /// Caps the viewport height (default: available height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the vertical gap between content children (`gap-8` default).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets which end the floating jump button targets (`end` default).
    pub fn direction(mut self, direction: MessageScrollerDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets a stable id salt so scroll state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the scroller around its message content.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<MessageScrollerResponse> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_message_scroller"),
        };
        let mut state: ScrollerState = ui.ctx().data_mut(|d| d.get_temp(id)).unwrap_or_default();

        let gap = self.gap;
        let pending_jump = state.pending_jump;
        let jump_align = match self.direction {
            MessageScrollerDirection::End => Align::BOTTOM,
            MessageScrollerDirection::Start => Align::TOP,
        };
        let mut jumped = false;

        let laid_out = ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt(id.with("viewport"))
                .layout(egui::Layout::top_down(Align::Min)),
            |ui| {
                // Scoped shadcn scrollbar look (thumb = bg-border), with the
                // fade strengthened to match the source's `scroll-fade-b`.
                ui.style_mut().spacing.scroll =
                    crate::components::scroll_area::ScrollArea::themed_scroll_style();
                ui.style_mut().spacing.scroll.fade = egui::style::ScrollFadeStyle {
                    strength: 0.75,
                    size: 24.0,
                };
                ui.style_mut().visuals.extreme_bg_color = p.border;

                let mut area = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(state.follow);
                if let Some(h) = self.height {
                    area = area.max_height(h);
                }
                area.show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = gap;
                    content(ui);
                    if pending_jump {
                        ui.scroll_to_cursor(Some(jump_align));
                        jumped = true;
                    }
                })
            },
        );
        let out = laid_out.inner;
        let viewport_rect = out.inner_rect;
        let offset_y = out.state.clamped_offset().y;
        let at_bottom = is_at_bottom(offset_y, out.content_size.y, viewport_rect.height());
        let at_top = is_at_top(offset_y);
        let scrolled_up = offset_y < state.last_offset - 1.0 && !at_bottom;

        state.follow = next_follow(state.follow, at_bottom, scrolled_up);
        if jumped {
            state.pending_jump = false;
        }
        state.last_offset = offset_y;

        // The jump button appears when scrolled away from its end and
        // animates out when returning — paint it every frame for the fade.
        let active = match self.direction {
            MessageScrollerDirection::End => !at_bottom,
            MessageScrollerDirection::Start => !at_top,
        };
        let side = ButtonSize::IconSm.height();
        let anchor = match self.direction {
            MessageScrollerDirection::End => Pos2::new(
                viewport_rect.center().x,
                viewport_rect.bottom() - 16.0 - side * 0.5, // bottom-4
            ),
            MessageScrollerDirection::Start => Pos2::new(
                viewport_rect.center().x,
                viewport_rect.top() + 16.0 + side * 0.5, // top-4
            ),
        };
        let btn = MessageScrollerButton::new()
            .direction(self.direction)
            .active(active)
            .id_salt(format!("{id:?}"));
        if btn.show_at(ui, anchor).clicked() {
            state.pending_jump = true;
        }

        ui.ctx().data_mut(|d| d.insert_temp(id, state));

        let response = MessageScrollerResponse {
            at_bottom,
            at_top,
            following: state.follow,
        };
        InnerResponse::new(response, laid_out.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Id;

    #[test]
    fn at_bottom_math() {
        // Content 500, viewport 200 → max offset 300.
        assert!(is_at_bottom(300.0, 500.0, 200.0));
        assert!(is_at_bottom(299.5, 500.0, 200.0)); // 1px tolerance
        assert!(!is_at_bottom(250.0, 500.0, 200.0));
        // Content smaller than viewport: always at bottom.
        assert!(is_at_bottom(0.0, 100.0, 200.0));
    }

    #[test]
    fn at_top_math() {
        assert!(is_at_top(0.0));
        assert!(is_at_top(0.5)); // 1px tolerance
        assert!(!is_at_top(2.0));
    }

    #[test]
    fn direction_defaults_to_end() {
        assert_eq!(
            MessageScrollerDirection::default(),
            MessageScrollerDirection::End
        );
        assert_eq!(
            MessageScroller::new()
                .direction(MessageScrollerDirection::Start)
                .direction,
            MessageScrollerDirection::Start
        );
    }

    #[test]
    fn renders_active_button_and_fade() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("scroller_button_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 400.0))),
            );
            // Active: fully painted 32px square.
            let active = MessageScrollerButton::new()
                .id_salt("active")
                .show_at(ui, Pos2::new(150.0, 380.0));
            assert!(
                (active.rect.width() - 32.0).abs() < 1.0,
                "{:?}",
                active.rect
            );
            // Faded out: not clickable (empty hit rect, hover sense only).
            let hidden = MessageScrollerButton::new()
                .id_salt("hidden")
                .active(false)
                .direction(MessageScrollerDirection::Start)
                .show_at(ui, Pos2::new(150.0, 20.0));
            assert!(!hidden.sense.senses_click());
            // The salt-less default id is scoped to the enclosing ui, so two
            // scroller instances never share button state.
            let ui2 = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("scroller_button_test_2"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 400.0))),
            );
            let in_ui2 = MessageScrollerButton::new().show_at(ui2, Pos2::new(150.0, 380.0));
            assert_eq!(
                in_ui2.id,
                ui2.scope_id().with("shadcn_message_scroller_button")
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn follow_transitions() {
        // At the bottom: always follow.
        assert!(next_follow(false, true, false));
        // Scrolled up: disengage.
        assert!(!next_follow(true, false, true));
        // Not at bottom but no upward scroll: keep previous.
        assert!(next_follow(true, false, false));
        assert!(!next_follow(false, false, false));
    }

    #[test]
    fn sticks_to_bottom_from_the_start() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut last: Option<MessageScrollerResponse> = None;
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("scroller_state_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 400.0))),
                );
                let resp = MessageScroller::new()
                    .height(200.0)
                    .id_salt("persist")
                    .show(ui, |ui| {
                        for i in 0..30 {
                            ui.label(format!("message {i}"));
                        }
                    })
                    .inner;
                last = Some(resp);
            });
            out.textures_delta.clear();
        }
        let last = last.expect("response");
        // Overflowing content + auto-follow: pinned to the bottom.
        assert!(last.at_bottom, "stick_to_bottom should pin the start");
        assert!(last.following);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            for frame in 0..3 {
                let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("scroller_render_test"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 400.0))),
                    );
                    MessageScroller::new()
                        .height(200.0)
                        .id_salt("render")
                        .show(ui, |ui| {
                            for i in 0..30 {
                                ui.label(format!("message {i} — frame {frame}"));
                            }
                        });
                    // Second instance with a start-direction button — state
                    // must not bleed between instances.
                    let resp = MessageScroller::new()
                        .height(120.0)
                        .direction(MessageScrollerDirection::Start)
                        .id_salt("render_start")
                        .show(ui, |ui| {
                            for i in 0..10 {
                                ui.label(format!("start message {i} — frame {frame}"));
                            }
                        })
                        .inner;
                    let _ = (resp.at_top, resp.at_bottom, resp.following);
                });
                out.textures_delta.clear();
            }
        }
    }
}
