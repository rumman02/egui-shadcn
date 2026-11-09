//! shadcn `message-scroller.tsx` — an auto-scrolling chat container.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/message-scroller.tsx`: a
//! `MessageScroller` viewport (`flex flex-col overflow-y-auto`) whose
//! content stacks with `gap-8`, that pins to the bottom while new messages
//! stream in and backs off the moment the reader scrolls up. A
//! `MessageScrollerButton` (`variant="secondary" size="icon-sm"`,
//! `bottom-4`, arrow-down) appears while scrolled away; one click jumps back
//! to the newest message and re-engages auto-follow.
//!
//! Web-only details are approximated: egui's `stick_to_bottom` handles the
//! pin/unpin bookkeeping, the smooth scroll animations and scroll fades are
//! not portable, and `MessageScrollerItem`'s `scrollAnchor` turn-anchoring
//! is not ported (noted in PROGRESS.md).
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
use egui::{Align, Id, InnerResponse, Pos2, Rect, Ui, Vec2};

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

/// The floating scroll-to-bottom button (`MessageScrollerButton`); the
/// [`MessageScroller`] positions it at its viewport's bottom-center.
#[derive(Clone, Debug, Default)]
pub struct MessageScrollerButton {
    id_salt: Option<String>,
}

impl MessageScrollerButton {
    /// Creates the scroll-to-end button.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the button centered on `anchor` (the viewport's bottom-center
    /// minus `bottom-4`) on a floating layer. Returns the click response.
    pub fn show_at(self, ui: &Ui, anchor: Pos2) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let side = ButtonSize::IconSm.height(); // size-8
        let rect = Rect::from_center_size(anchor, Vec2::splat(side));
        let id = match self.id_salt.as_deref() {
            Some(salt) => Id::unique("shadcn_message_scroller_button").with(salt),
            None => Id::unique("shadcn_message_scroller_button"),
        };
        // Hover state comes from the previous pass (one-frame lag, standard
        // for deferred container painting).
        let hovered = ui
            .ctx()
            .read_response(id)
            .map(|r| r.hovered())
            .unwrap_or(false);
        let fill = if hovered { p.muted } else { p.background };

        // Paint on a floating middle layer so it overlaps the content.
        let painter = ui
            .ctx()
            .layer_painter(egui::LayerId::new(egui::Order::Middle, id.with("layer")));
        painter.rect_filled(rect, theme.radius.full_u8(), fill);
        painter.rect_stroke(
            rect,
            theme.radius.full_u8(),
            egui::Stroke::new(1.0, p.border),
            egui::epaint::StrokeKind::Inside,
        );
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(16.0));
        Icon::ArrowDown.paint(&painter, icon_rect, p.foreground, 2.0);

        ui.interact(rect, id, egui::Sense::click())
    }
}

/// What [`MessageScroller::show`] reports for the current pass.
#[derive(Clone, Copy, Debug)]
pub struct MessageScrollerResponse {
    /// True when the viewport is pinned to the bottom of the content.
    pub at_bottom: bool,
    /// True when auto-follow is engaged (new content scrolls into view).
    pub following: bool,
}

/// The message scroller. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct MessageScroller {
    height: Option<f32>,
    gap: f32,
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
        let mut jumped = false;

        let laid_out = ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt(id.with("viewport"))
                .layout(egui::Layout::top_down(Align::Min)),
            |ui| {
                // Scoped shadcn scrollbar look (thumb = bg-border).
                ui.style_mut().spacing.scroll =
                    crate::components::scroll_area::ScrollArea::themed_scroll_style();
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
                        ui.scroll_to_cursor(Some(Align::BOTTOM));
                        jumped = true;
                    }
                })
            },
        );
        let out = laid_out.inner;
        let viewport_rect = out.inner_rect;
        let offset_y = out.state.clamped_offset().y;
        let at_bottom = is_at_bottom(offset_y, out.content_size.y, viewport_rect.height());
        let scrolled_up = offset_y < state.last_offset - 1.0 && !at_bottom;

        state.follow = next_follow(state.follow, at_bottom, scrolled_up);
        if jumped {
            state.pending_jump = false;
        }
        state.last_offset = offset_y;

        // The scroll-to-end button appears when scrolled away from the end.
        if !at_bottom {
            let anchor = Pos2::new(
                viewport_rect.center().x,
                viewport_rect.bottom() - 16.0 - ButtonSize::IconSm.height() * 0.5, // bottom-4
            );
            let btn = MessageScrollerButton::new().id_salt(format!("{id:?}"));
            if btn.show_at(ui, anchor).clicked() {
                state.pending_jump = true;
            }
        }

        ui.ctx().data_mut(|d| d.insert_temp(id, state));

        let response = MessageScrollerResponse {
            at_bottom,
            following: state.follow,
        };
        InnerResponse::new(response, laid_out.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

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
                });
                out.textures_delta.clear();
            }
        }
    }
}
