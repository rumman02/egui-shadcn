//! shadcn `sonner.tsx` — a global toast notification system.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/sonner.tsx`, which configures
//! the Sonner toaster with CSS variables
//! `--normal-bg: popover`, `--normal-text: popover-foreground`,
//! `--normal-border: border`, `--border-radius: radius` and lucide icons for
//! success/info/warning/error/loading. The source uses the library defaults
//! for everything else: bottom-right position, 4s auto-dismiss, click to
//! dismiss, newest toast at the bottom of the stack, slide+fade animation.
//! Without Sonner's `richColors` mode the variant icons render in the normal
//! text color (monochrome), which is what this port does.
//!
//! Usage: fire toasts from anywhere with [`Sonner::toast`], then render the
//! toaster once per frame with [`Sonner::show`] (e.g. at the end of your
//! central panel).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! // anywhere (e.g. on a button click):
//! Sonner::toast(ui.ctx(), Toast::new("Event has been created"));
//! Sonner::toast(
//!     ui.ctx(),
//!     Toast::new("Uh oh! Something went wrong.")
//!         .description("There was a problem with your request.")
//!         .variant(ToastVariant::Error),
//! );
//!
//! // once per frame:
//! Sonner::show(ui);
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Order, Pos2, Rect, Sense, Stroke, Ui, Vec2,
};

/// sonner's default toast width.
pub const SONNER_TOAST_WIDTH: f32 = 356.0;
/// sonner's default auto-dismiss duration (seconds).
pub const SONNER_DEFAULT_DURATION_SECS: f32 = 4.0;
/// sonner's default gap between toasts.
pub const SONNER_GAP: f32 = 14.0;
/// Out-animation duration.
const LEAVE_SECS: f32 = 0.2;

/// The toast variants the source configures icons for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastVariant {
    /// No icon.
    #[default]
    Default,
    /// `CircleCheckIcon`.
    Success,
    /// `OctagonXIcon` (rendered as the lucide circle-x in this port's icon
    /// set).
    Error,
    /// `TriangleAlertIcon` (rendered as the lucide circle-alert).
    Warning,
    /// `InfoIcon`.
    Info,
    /// Spinning `Loader2Icon`.
    Loading,
}

impl ToastVariant {
    fn icon(self) -> Option<Icon> {
        match self {
            ToastVariant::Default => None,
            ToastVariant::Success => Some(Icon::CircleCheck),
            ToastVariant::Error => Some(Icon::CircleX),
            ToastVariant::Warning => Some(Icon::CircleAlert),
            ToastVariant::Info => Some(Icon::Info),
            ToastVariant::Loading => Some(Icon::LoaderCircle),
        }
    }
}

/// A toast message. See the [module docs](self) for the full flow.
#[derive(Clone, Debug)]
pub struct Toast {
    title: String,
    description: Option<String>,
    variant: ToastVariant,
    duration_secs: f32,
}

impl Toast {
    /// Creates a toast with the given title and sonner's defaults.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            variant: ToastVariant::default(),
            duration_secs: SONNER_DEFAULT_DURATION_SECS,
        }
    }

    /// Sets the secondary description line.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the variant (icon).
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the auto-dismiss duration in seconds.
    pub fn duration(mut self, secs: f32) -> Self {
        self.duration_secs = secs;
        self
    }
}

#[derive(Clone, Debug)]
struct ToastEntry {
    id: u64,
    toast: Toast,
    created_at: f64,
    leaving: bool,
    leave_at: f64,
}

fn queue_id() -> Id {
    Id::unique("shadcn_sonner_toasts")
}

fn next_id(ctx: &Context) -> u64 {
    ctx.data_mut(|d| {
        let next: u64 = d.get_temp(next_counter_id()).unwrap_or(1);
        d.insert_temp(next_counter_id(), next.wrapping_add(1));
        next
    })
}

fn next_counter_id() -> Id {
    Id::unique("shadcn_sonner_counter")
}

/// The sonner toaster facade.
pub struct Sonner;

impl Sonner {
    /// Enqueues a toast. Call from anywhere; render the stack once per frame
    /// with [`Sonner::show`].
    pub fn toast(ctx: &Context, toast: Toast) {
        let now = ctx.input(|i| i.time);
        // Compute the id outside the data lock: nested `data_mut` calls
        // would deadlock.
        let id = next_id(ctx);
        ctx.data_mut(|d| {
            let mut queue: Vec<ToastEntry> = d.get_temp(queue_id()).unwrap_or_default();
            queue.push(ToastEntry {
                id,
                toast,
                created_at: now,
                leaving: false,
                leave_at: 0.0,
            });
            d.insert_temp(queue_id(), queue);
        });
    }

    /// Dismisses every toast immediately (skips the exit animation).
    pub fn clear(ctx: &Context) {
        ctx.data_mut(|d| {
            d.insert_temp(queue_id(), Vec::<ToastEntry>::new());
        });
    }

    /// Renders the toast stack (bottom-right, newest at the bottom). Call
    /// once per frame.
    pub fn show(ui: &mut Ui) {
        show_at(ui, Align2::RIGHT_BOTTOM, Vec2::new(-16.0, -16.0));
    }
}

fn show_at(ui: &mut Ui, anchor: Align2, offset: Vec2) {
    let ctx = ui.ctx().clone();
    let theme = ShadcnTheme::of_ui(ui);
    let dark = theme.mode.is_dark();
    let now = ctx.input(|i| i.time);

    // Lifecycle: auto-dismiss, animate out, then drop.
    let mut queue: Vec<ToastEntry> = ctx.data_mut(|d| d.get_temp(queue_id()).unwrap_or_default());
    for entry in &mut queue {
        if !entry.leaving {
            let elapsed = (now - entry.created_at) as f32;
            if elapsed >= entry.toast.duration_secs {
                entry.leaving = true;
                entry.leave_at = now;
            }
        }
    }
    queue.retain(|e| !e.leaving || ((now - e.leave_at) as f32) < LEAVE_SECS + 0.05);
    ctx.data_mut(|d| d.insert_temp(queue_id(), queue.clone()));

    if queue.is_empty() {
        return;
    }

    egui::Area::new(Id::unique("shadcn_sonner_area"))
        .order(Order::Foreground)
        .anchor(anchor, offset)
        .interactable(true)
        .show(&ctx, |ui| {
            // bottom_up: the first child sits at the bottom, so enqueue order
            // (oldest first) puts the newest toast at the bottom, like
            // sonner's default position.
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.spacing_mut().item_spacing.y = SONNER_GAP;
                ui.set_max_width((SONNER_TOAST_WIDTH).min(ctx.content_rect().width() - 32.0));
                for entry in &queue {
                    render_toast(ui, entry, now, &theme, dark);
                }
            });
        });
}

/// The stable interact id of a rendered toast (used by tests and callers
/// that need to locate a toast on screen).
pub fn toast_id(entry_id: u64) -> Id {
    Id::unique(("shadcn_sonner_toast", entry_id))
}

fn render_toast(ui: &mut Ui, entry: &ToastEntry, now: f64, theme: &ShadcnTheme, dark: bool) {
    let p = theme.palette;
    let anim_id = Id::unique(("shadcn_sonner_anim", entry.id));
    // Slide up + fade in on enter; fade out on leave.
    let progress = ui.ctx().animate_value_with_time(
        anim_id,
        if entry.leaving { 0.0 } else { 1.0 },
        LEAVE_SECS,
    );

    let width = ui.available_width();
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let desc_font = FontId::proportional(crate::theme::tokens::type_scale::XS);

    // Measure the text block (layout color is irrelevant for sizing).
    let title_galley =
        ui.painter()
            .layout_no_wrap(entry.toast.title.clone(), font.clone(), Color32::WHITE);
    let desc_galley = entry.toast.description.as_ref().map(|d| {
        ui.painter()
            .layout(d.clone(), desc_font.clone(), Color32::WHITE, width - 48.0)
    });
    let text_h = title_galley.size().y + desc_galley.as_ref().map_or(0.0, |g| g.size().y + 4.0);
    let height = (text_h.max(20.0) + 16.0 * 2.0).max(36.0);

    let (_, space) = ui.allocate_space(Vec2::new(width, height));
    let response = ui.interact(space, toast_id(entry.id), Sense::click());

    if ui.is_rect_visible(space) {
        let mut painter = ui.painter().clone();
        // Slide-in: rise 8px while fading (sonner's enter transition).
        let slide = (1.0 - progress) * 8.0;
        let rect = space.translate(Vec2::new(0.0, slide));
        painter.set_opacity(progress);
        let corner = CornerRadius::same(theme.radius.lg_u8());

        // Shadow behind the card, then `--normal-bg` fill and border.
        let sh = crate::utils::shadow::shadow_lg(dark);
        painter.add(
            egui::epaint::RectShape::filled(
                rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                corner,
                sh.color,
            )
            .with_blur_width(3.0),
        );
        painter.rect_filled(rect, corner, p.popover);
        painter.rect_stroke(
            rect,
            corner,
            Stroke::new(1.0, p.border),
            egui::epaint::StrokeKind::Inside,
        );

        let mut cursor_x = rect.left() + 16.0;
        if let Some(icon) = entry.toast.variant.icon() {
            let icon_rect = Rect::from_center_size(
                Pos2::new(cursor_x + 8.0, rect.center().y),
                Vec2::splat(16.0),
            );
            if entry.toast.variant == ToastVariant::Loading {
                // Spin the loader, like the source's `animate-spin`.
                let angle = crate::components::spinner::angle_at(now);
                let pts: Vec<Pos2> = crate::components::spinner::arc_points(angle, 24)
                    .iter()
                    .map(|[x, y]| {
                        Pos2::new(
                            icon_rect.center().x + (x - 12.0) * (16.0 / 24.0),
                            icon_rect.center().y + (y - 12.0) * (16.0 / 24.0),
                        )
                    })
                    .collect();
                painter.add(egui::Shape::line(
                    pts,
                    Stroke::new(2.0 * (16.0 / 24.0), p.popover_foreground),
                ));
            } else {
                icon.paint(&painter, icon_rect, p.popover_foreground, 16.0 / 24.0);
            }
            cursor_x += 24.0;
        }

        let text_x = cursor_x;
        let mut text_y = rect.center().y - text_h * 0.5;
        let title =
            ui.painter()
                .layout_no_wrap(entry.toast.title.clone(), font, p.popover_foreground);
        let title_h = title.size().y;
        painter.galley(Pos2::new(text_x, text_y), title, p.popover_foreground);
        text_y += title_h + 4.0;
        if let Some(desc) = &entry.toast.description {
            let muted = color_alpha(p.popover_foreground, 0.7);
            let galley = ui.painter().layout(
                desc.clone(),
                desc_font,
                muted,
                width - (text_x - rect.left()) - 16.0,
            );
            painter.galley(Pos2::new(text_x, text_y), galley, muted);
        }

        painter.set_opacity(1.0);
    }

    // Click to dismiss (sonner default) — start the exit animation.
    if response.clicked() && !entry.leaving {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|i| i.time);
        ctx.data_mut(|d| {
            let mut queue: Vec<ToastEntry> = d.get_temp(queue_id()).unwrap_or_default();
            if let Some(e) = queue.iter_mut().find(|e| e.id == entry.id) {
                e.leaving = true;
                e.leave_at = now;
            }
            d.insert_temp(queue_id(), queue);
        });
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

        fn run(&mut self, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.5;
            let input = egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(screen()),
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("sonner_test_ui"),
                    egui::UiBuilder::new().max_rect(screen()),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn click_at(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.time += 0.5;
                let input = egui::RawInput {
                    time: Some(self.time),
                    screen_rect: Some(screen()),
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
                        Id::unique("sonner_test_ui"),
                        egui::UiBuilder::new().max_rect(screen()),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        fn queue_len(&self) -> usize {
            self.ctx
                .data_mut(|d| d.get_temp::<Vec<ToastEntry>>(queue_id()))
                .map(|q| q.len())
                .unwrap_or(0)
        }
    }

    #[test]
    fn constants_match_sonner_defaults() {
        assert_eq!(SONNER_TOAST_WIDTH, 356.0);
        assert_eq!(SONNER_DEFAULT_DURATION_SECS, 4.0);
        assert_eq!(SONNER_GAP, 14.0);
    }

    #[test]
    fn variant_icons_match_source_config() {
        // sonner.tsx maps success/info/warning/error/loading to lucide icons.
        assert_eq!(ToastVariant::Success.icon(), Some(Icon::CircleCheck));
        assert_eq!(ToastVariant::Info.icon(), Some(Icon::Info));
        assert_eq!(ToastVariant::Warning.icon(), Some(Icon::CircleAlert));
        assert_eq!(ToastVariant::Error.icon(), Some(Icon::CircleX));
        assert_eq!(ToastVariant::Loading.icon(), Some(Icon::LoaderCircle));
        assert_eq!(ToastVariant::Default.icon(), None);
    }

    #[test]
    fn toast_enqueues_and_auto_dismisses() {
        let mut h = Harness::new();
        h.run(|ui| {
            Sonner::toast(ui.ctx(), Toast::new("Event has been created"));
            Sonner::show(ui);
        });
        assert_eq!(h.queue_len(), 1);

        // Advance past the 4s default (0.5s per frame → 9 frames = 4.5s).
        for _ in 0..10 {
            h.run(|ui| {
                Sonner::show(ui);
            });
        }
        assert_eq!(
            h.queue_len(),
            0,
            "toast must auto-dismiss after its duration"
        );
    }

    #[test]
    fn click_dismisses_toast() {
        let mut h = Harness::new();
        h.run(|ui| {
            Sonner::toast(ui.ctx(), Toast::new("Click me").duration(60.0));
            Sonner::show(ui);
        });
        assert_eq!(h.queue_len(), 1);

        // Warm up until the anchored area has settled, then click the toast.
        for _ in 0..4 {
            h.run(|ui| {
                Sonner::show(ui);
            });
        }
        let toast_rect = h
            .ctx
            .read_response(toast_id(1))
            .map(|r| r.rect)
            .expect("toast must be on screen");
        assert!(toast_rect.is_positive());

        h.click_at(toast_rect.center(), |ui| {
            Sonner::show(ui);
        });
        // The toast enters its leave animation, then leaves the queue.
        assert!(h.queue_len() <= 1);
        for _ in 0..3 {
            h.run(|ui| {
                Sonner::show(ui);
            });
        }
        assert_eq!(
            h.queue_len(),
            0,
            "clicked toast must leave after animating out"
        );
    }

    #[test]
    fn custom_duration_controls_lifetime() {
        let mut h = Harness::new();
        h.run(|ui| {
            Sonner::toast(ui.ctx(), Toast::new("quick").duration(1.0));
            Sonner::show(ui);
        });
        assert_eq!(h.queue_len(), 1);

        // 0.5s/frame: elapsed 0.5s (kept), 1.0s (starts leaving), then gone.
        h.run(|ui| {
            Sonner::show(ui);
        });
        assert_eq!(h.queue_len(), 1);
        h.run(|ui| {
            Sonner::show(ui);
        });
        assert!(h.queue_len() <= 1, "1s duration starts expiring now");
        h.run(|ui| {
            Sonner::show(ui);
        });
        assert_eq!(h.queue_len(), 0, "1s duration expires after ~3 frames");
    }

    #[test]
    fn clear_removes_all_toasts() {
        let mut h = Harness::new();
        h.run(|ui| {
            Sonner::toast(ui.ctx(), Toast::new("a").duration(60.0));
            Sonner::toast(ui.ctx(), Toast::new("b").duration(60.0));
            Sonner::show(ui);
        });
        assert_eq!(h.queue_len(), 2);
        Sonner::clear(&h.ctx);
        assert_eq!(h.queue_len(), 0);
    }

    #[test]
    fn toast_builders_keep_fields() {
        let t = Toast::new("Title")
            .description("Desc")
            .variant(ToastVariant::Success)
            .duration(2.5);
        assert_eq!(t.title, "Title");
        assert_eq!(t.description.as_deref(), Some("Desc"));
        assert_eq!(t.variant, ToastVariant::Success);
        assert!((t.duration_secs - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
