//! Icon demo: every delivered Lucide icon at several sizes and tints.
//!
//! All icons are real Lucide data (see `icons::data`), rasterized once
//! into cached alpha-mask textures and tinted per paint. The grid uses
//! `Icon::paint` directly; the sizes row uses `Icon::show` (theme
//! foreground, standard stroke). The colors row proves one cached mask
//! serves every tint.

use egui_shadcn_ui::prelude::*;

fn icon_cell(ui: &mut egui::Ui, icon: Icon, size: f32, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());
    icon.paint(ui.painter(), rect, color, 2.0 * size / 24.0);
}

pub fn icon_demo(ui: &mut egui::Ui) {
    let theme = ShadcnTheme::of_ui(ui);

    ui.label("All 44 icons (16px, theme foreground)");
    ui.add_space(4.0);
    egui::Grid::new("icon-grid")
        .num_columns(8)
        .spacing([8.0, 10.0])
        .show(ui, |ui| {
            for (i, icon) in Icon::all().iter().enumerate() {
                // Fixed-width cells keep the columns even; the variant name
                // fits on one line at 10px in 104px.
                ui.vertical_centered(|ui| {
                    ui.set_min_width(104.0);
                    ui.set_max_width(104.0);
                    icon_cell(ui, *icon, 16.0, theme.palette.foreground);
                    ui.label(egui::RichText::new(format!("{icon:?}")).size(10.0));
                });
                if i % 8 == 7 {
                    ui.end_row();
                }
            }
        });

    ui.add_space(16.0);
    ui.label("Sizes (Icon::show: 12 / 16 / 24 / 32 / 48px)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for size in [12.0, 16.0, 24.0, 32.0, 48.0] {
            Icon::Check.show(ui, size);
            ui.add_space(8.0);
        }
        Icon::Sparkles.show(ui, 32.0);
        ui.add_space(8.0);
        Icon::LoaderCircle.show(ui, 32.0);
    });

    ui.add_space(16.0);
    ui.label("Colors (one cached mask, four tints)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for color in [
            theme.palette.foreground,
            theme.palette.primary,
            theme.palette.destructive,
            theme.palette.muted_foreground,
        ] {
            icon_cell(ui, Icon::Search, 24.0, color);
            ui.add_space(12.0);
        }
    });
}
