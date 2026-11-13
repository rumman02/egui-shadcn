//! Hover-card demo: link trigger, custom widget trigger and delays.

use egui_shadcn_ui::prelude::*;

pub fn hover_card_demo(ui: &mut egui::Ui) {
    ui.heading("Basic");
    ui.add_space(4.0);
    HoverCard::new().id_salt("demo_basic").show(ui, |ui| {
        HoverCardTrigger::new("@nextjs").show(ui);
        HoverCardContent::new().width(320.0).show(ui, |ui| {
            // Official demo parity: `w-80`, title, description, muted date.
            let p = ShadcnTheme::of_ui(ui).palette;
            ui.label(egui::RichText::new("@nextjs").size(type_scale::SM).strong());
            ui.label(
                egui::RichText::new(
                    "The React Framework \u{2013} created and maintained by @vercel.",
                )
                .size(type_scale::SM),
            );
            ui.label(
                egui::RichText::new("Joined December 2021")
                    .size(type_scale::XS)
                    .color(p.muted_foreground),
            );
        });
    });

    ui.add_space(16.0);
    ui.heading("With a button trigger");
    ui.add_space(4.0);
    HoverCard::new().id_salt("demo_button").show(ui, |ui| {
        HoverCardTrigger::widget(|ui| {
            Button::new("Hover for details")
                .variant(ButtonVariant::Outline)
                .show(ui)
        })
        .show(ui);
        HoverCardContent::new().show(ui, |ui| {
            ui.strong("Calendar");
            ui.weak("Schedules the meeting for you.");
        });
    });

    ui.add_space(16.0);
    ui.heading("Fast delays (150ms open / 100ms close)");
    ui.add_space(4.0);
    HoverCard::new()
        .open_delay_ms(150)
        .close_delay_ms(100)
        .id_salt("demo_fast")
        .show(ui, |ui| {
            HoverCardTrigger::new("@shadcn").show(ui);
            HoverCardContent::new().show(ui, |ui| {
                ui.weak("Snappy hover card with custom delays.");
            });
        });
}
