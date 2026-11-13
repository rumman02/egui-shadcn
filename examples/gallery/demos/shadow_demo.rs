//! Shadow demo: the layered, texture-cached shadow system in
//! `utils::shadow`.
//!
//! Every card below is painted with exactly one call to `paint_shadow` (or
//! `paint_shadow_resolved` in the resolution row): the shadow is a cached
//! nine-slice texture, rasterized once per (layers, radii, dark, scale) key
//! and reused for every widget that shares it. The panel behind the cards is
//! the theme's muted fill, so the faint outer falloff of each layer reads
//! against a contrasting background. Multi-layer sizes (sm/md/lg/xl) show
//! both layers: a tight dark core plus a wide soft halo.

use egui_shadcn_ui::prelude::*;
use egui_shadcn_ui::utils::shadow::{
    paint_shadow, paint_shadow_resolved, resolve_shadow, ShadowSize, ShadowSlot,
};

/// One scale card: shadow, then the card fill, then the size name.
fn scale_card(ui: &mut egui::Ui, rect: egui::Rect, size: ShadowSize, name: &str) {
    let theme = ShadcnTheme::of_ui(ui);
    let dark = theme.mode.is_dark();
    let corner = egui::CornerRadius::same(8);
    paint_shadow(ui.painter(), rect, corner, size, dark);
    ui.painter().rect_filled(rect, corner, theme.palette.card);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::proportional(14.0),
        theme.palette.card_foreground,
    );
}

/// One resolution card: the shadow is resolved for the active style, and the
/// label names the size the LAYER rule produced.
fn resolved_card(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    base: ShadowSize,
    slot: ShadowSlot,
    slot_name: &str,
) {
    let theme = ShadcnTheme::of_ui(ui);
    let dark = theme.mode.is_dark();
    let resolved = resolve_shadow(base, theme.style, slot);
    let corner = egui::CornerRadius::same(8);
    paint_shadow_resolved(ui.painter(), rect, corner, base, theme.style, slot, dark);
    ui.painter().rect_filled(rect, corner, theme.palette.card);
    let label = format!("{slot_name}: {resolved:?}");
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        theme.palette.card_foreground,
    );
}

pub fn shadow_demo(ui: &mut egui::Ui) {
    let theme = ShadcnTheme::of_ui(ui);

    ui.label("Shadow scale — Tailwind v4 defaults, layered and cached");
    ui.add_space(4.0);
    ui.label(
        "Identical cards, one shadow size each. shadow-sm/md/lg/xl carry two \
         layers; shadow-2xl is a single strong layer; shadow-none paints \
         nothing and allocates no texture.",
    );
    ui.add_space(8.0);

    let panel_w = ui.available_width().min(980.0);
    let (panel, _) = ui.allocate_exact_size(egui::vec2(panel_w, 320.0), egui::Sense::hover());
    ui.painter().rect_filled(panel, 8.0, theme.palette.muted);

    let margin = 28.0;
    let gap = 28.0;
    let card_w = (panel_w - 2.0 * margin - 3.0 * gap) / 4.0;
    let card_h = 72.0;
    let col = |i: usize| panel.min.x + margin + i as f32 * (card_w + gap);
    let row1_y = panel.min.y + 36.0;
    let row2_y = row1_y + card_h + 40.0;
    let card_rect = |i: usize, y: f32| {
        egui::Rect::from_min_size(egui::pos2(col(i), y), egui::vec2(card_w, card_h))
    };
    scale_card(ui, card_rect(0, row1_y), ShadowSize::Xs, "shadow-xs");
    scale_card(ui, card_rect(1, row1_y), ShadowSize::Sm, "shadow-sm");
    scale_card(ui, card_rect(2, row1_y), ShadowSize::Md, "shadow-md");
    scale_card(ui, card_rect(3, row1_y), ShadowSize::Lg, "shadow-lg");
    scale_card(ui, card_rect(0, row2_y), ShadowSize::Xl, "shadow-xl");
    scale_card(ui, card_rect(1, row2_y), ShadowSize::Xxl, "shadow-2xl");
    scale_card(ui, card_rect(2, row2_y), ShadowSize::None, "shadow-none");

    ui.add_space(8.0);
    ui.label("Layer values (offset-x offset-y blur spread color):");
    ui.monospace("xs:  0 1px 2px 0 rgb(0 0 0 / 0.05)");
    ui.monospace("sm:  0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)");
    ui.monospace("md:  0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)");
    ui.monospace("lg:  0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)");
    ui.monospace("xl:  0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)");
    ui.monospace("2xl: 0 25px 50px -12px rgb(0 0 0 / 0.25)");

    ui.add_space(16.0);
    ui.label(format!(
        "Style resolution (LAYER) — active style: {}",
        theme.style.name()
    ));
    ui.add_space(4.0);
    ui.label(
        "Silence keeps the base shadow; an explicit style override wins. \
         Switch styles in the gallery header and watch the resolved sizes \
         change: the outline button keeps shadow-xs everywhere, while \
         popover/dialog/command slots follow their style rules.",
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.set_min_height(56.0);
        let (r1, _) = ui.allocate_exact_size(egui::vec2(210.0, 56.0), egui::Sense::hover());
        resolved_card(
            ui,
            r1,
            ShadowSize::Xs,
            ShadowSlot::ButtonOutline,
            "ButtonOutline",
        );
        ui.add_space(20.0);
        let (r2, _) = ui.allocate_exact_size(egui::vec2(210.0, 56.0), egui::Sense::hover());
        resolved_card(
            ui,
            r2,
            ShadowSize::Md,
            ShadowSlot::PopoverContent,
            "PopoverContent",
        );
        ui.add_space(20.0);
        let (r3, _) = ui.allocate_exact_size(egui::vec2(210.0, 56.0), egui::Sense::hover());
        resolved_card(
            ui,
            r3,
            ShadowSize::Md,
            ShadowSlot::DialogContent,
            "DialogContent",
        );
        ui.add_space(20.0);
        let (r4, _) = ui.allocate_exact_size(egui::vec2(250.0, 56.0), egui::Sense::hover());
        resolved_card(
            ui,
            r4,
            ShadowSize::Xs,
            ShadowSlot::InputGroupInCommand,
            "InputGroupInCommand",
        );
    });

    ui.add_space(12.0);
    ui.label(
        "Dark mode multiplies every layer alpha by 1.6 (the crate's legacy \
         convention, kept by the new system) — toggle the theme to compare.",
    );
}
