//! Motion demo: the color/opacity transition system in `utils::animation`.
//!
//! Hover the swatches. Colors fade through OKLab (no muddy sRGB midpoints),
//! opacity fades linearly, and every fade follows its easing curve. The 600ms
//! timings on the hero and easing rows are slowed for visibility only — the
//! system default is the original's 150ms `transition-colors` timing, shown
//! in the default-spec row.

use egui_shadcn_ui::prelude::*;
use egui_shadcn_ui::utils::animation::{
    mix_oklab, transition_alpha, transition_color, transition_progress, Easing, TransitionSpec,
};

/// One hoverable swatch: a rounded rect whose fill and label color follow
/// one transition progress (the canonical wave pattern).
fn swatch(
    ui: &mut egui::Ui,
    id: &str,
    size: egui::Vec2,
    from: egui::Color32,
    to: egui::Color32,
    spec: TransitionSpec,
    label: &str,
) {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = &ShadcnTheme::of_ui(ui).palette;
    let progress = transition_progress(ui.ctx(), egui::Id::unique(id), response.hovered(), spec);
    let fill = mix_oklab(from, to, progress.t);
    let label_color = mix_oklab(p.muted_foreground, p.primary_foreground, progress.t);
    ui.painter().rect_filled(rect, 8.0, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        label_color,
    );
}

pub fn motion_demo(ui: &mut egui::Ui) {
    let theme = ShadcnTheme::of_ui(ui);
    let p = &theme.palette;

    ui.label(
        "Hover fade — one progress drives fill + text (600ms ease-out, slowed for visibility)",
    );
    ui.add_space(4.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(360.0, 64.0), egui::Sense::hover());
    let progress = transition_progress(
        ui.ctx(),
        egui::Id::unique("motion_hero"),
        response.hovered(),
        TransitionSpec::secs(0.6, Easing::EaseOut),
    );
    let fill = mix_oklab(p.muted, p.primary, progress.t);
    let text = mix_oklab(p.muted_foreground, p.primary_foreground, progress.t);
    ui.painter().rect_filled(rect, 8.0, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!(
            "t = {:.2}{}",
            progress.t,
            if progress.animating { " …" } else { "" }
        ),
        egui::FontId::proportional(14.0),
        text,
    );

    ui.add_space(16.0);
    ui.label("Default spec — 150ms, Tailwind default curve (the original's transition-colors)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        swatch(
            ui,
            "motion_default_bg",
            egui::vec2(140.0, 48.0),
            p.secondary,
            p.primary,
            TransitionSpec::default(),
            "fill",
        );
        // Border fade: the stroke color transitions while the fill stays put.
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(140.0, 48.0), egui::Sense::hover());
        let border = transition_color(
            ui.ctx(),
            egui::Id::unique("motion_default_border"),
            p.border,
            p.ring,
            response.hovered(),
            TransitionSpec::default(),
        );
        ui.painter().rect_filled(rect, 8.0, p.background);
        ui.painter().rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(2.0, border.color),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "border",
            egui::FontId::proportional(14.0),
            p.foreground,
        );
        // Text fade: the label color transitions on the static fill.
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(140.0, 48.0), egui::Sense::hover());
        let fg = transition_color(
            ui.ctx(),
            egui::Id::unique("motion_default_text"),
            p.muted_foreground,
            p.foreground,
            response.hovered(),
            TransitionSpec::default(),
        );
        ui.painter().rect_filled(rect, 8.0, p.muted);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "text",
            egui::FontId::proportional(14.0),
            fg.color,
        );
    });

    ui.add_space(16.0);
    ui.label("Opacity fade — transition-opacity over stripes (600ms ease-out)");
    ui.add_space(4.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(360.0, 64.0), egui::Sense::hover());
    // Striped backdrop so the fading alpha is visible.
    let stripe = 24.0;
    let mut x = rect.left();
    let mut dark = false;
    while x < rect.right() {
        let band = egui::Rect::from_min_max(
            egui::pos2(x, rect.top()),
            egui::pos2((x + stripe).min(rect.right()), rect.bottom()),
        );
        ui.painter()
            .rect_filled(band, 0.0, if dark { p.muted } else { p.background });
        dark = !dark;
        x += stripe;
    }
    let fade = transition_alpha(
        ui.ctx(),
        egui::Id::unique("motion_opacity"),
        0.25,
        1.0,
        response.hovered(),
        TransitionSpec::secs(0.6, Easing::EaseOut),
    );
    let overlay = p.primary.gamma_multiply(fade.alpha);
    ui.painter().rect_filled(rect, 8.0, overlay);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("alpha = {:.2}", fade.alpha),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );

    ui.add_space(16.0);
    ui.label("Easing comparison — same 600ms, different curves (hover each)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        swatch(
            ui,
            "motion_ease_linear",
            egui::vec2(140.0, 48.0),
            p.secondary,
            p.primary,
            TransitionSpec::secs(0.6, Easing::Linear),
            "linear",
        );
        swatch(
            ui,
            "motion_ease_out",
            egui::vec2(140.0, 48.0),
            p.secondary,
            p.primary,
            TransitionSpec::secs(0.6, Easing::EaseOut),
            "ease-out",
        );
        swatch(
            ui,
            "motion_ease_expo",
            egui::vec2(140.0, 48.0),
            p.secondary,
            p.primary,
            TransitionSpec::secs(0.6, Easing::EaseOutExpo),
            "ease-out-expo",
        );
    });

    ui.add_space(16.0);
    ui.label("Settled transitions request no repaints; hover away to watch the reverse fade.");
}
