//! Sheet demo: all four sides with slide-in panels.

use egui_shadcn_ui::prelude::*;

pub fn sheet_demo(ui: &mut egui::Ui) {
    ui.heading("Sides");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        sheet_trigger(
            ui,
            "demo_right",
            SheetSide::Right,
            "Right",
            "The right sheet (source default).",
        );
        sheet_trigger(ui, "demo_left", SheetSide::Left, "Left", "The left sheet.");
        sheet_trigger(
            ui,
            "demo_top",
            SheetSide::Top,
            "Top",
            "The top sheet slides down.",
        );
        sheet_trigger(
            ui,
            "demo_bottom",
            SheetSide::Bottom,
            "Bottom",
            "The bottom sheet slides up.",
        );
    });

    ui.add_space(16.0);
    ui.heading("With footer");
    ui.add_space(4.0);
    Sheet::new().id_salt("demo_footer").show(ui, |ui| {
        SheetTrigger::widget(|ui| Button::new("Right sheet with footer").show(ui)).show(ui);
        SheetContent::new().side(SheetSide::Right).show(ui, |ui| {
            SheetHeader::new().show(ui, |ui| {
                SheetTitle::new("Edit profile").show(ui);
                SheetDescription::new("Make changes to your profile.").show(ui);
            });
            ui.label("Name");
            let mut name = scratch(ui, "name");
            ui.add(egui::TextEdit::singleline(&mut name).desired_width(f32::INFINITY));
            set_scratch(ui, "name", name);
            SheetFooter::new().show(ui, |ui| {
                SheetClose::widget(|ui| {
                    Button::new("Cancel")
                        .variant(ButtonVariant::Outline)
                        .show(ui)
                })
                .show(ui);
                SheetClose::widget(|ui| Button::new("Save").show(ui)).show(ui);
            });
        });
    });

    ui.add_space(16.0);
    ui.heading("Without close button");
    ui.add_space(4.0);
    Sheet::new().id_salt("demo_noclose").show(ui, |ui| {
        SheetTrigger::widget(|ui| {
            Button::new("Left, no X")
                .variant(ButtonVariant::Outline)
                .show(ui)
        })
        .show(ui);
        SheetContent::new()
            .side(SheetSide::Left)
            .show_close_button(false)
            .show(ui, |ui| {
                SheetHeader::new().show(ui, |ui| {
                    SheetTitle::new("Dismiss with Escape or backdrop").show(ui);
                    SheetDescription::new("showCloseButton = false hides the X.").show(ui);
                });
            });
    });
}

fn sheet_trigger(ui: &mut egui::Ui, salt: &str, side: SheetSide, label: &str, description: &str) {
    Sheet::new().id_salt(salt).show(ui, |ui| {
        SheetTrigger::widget(|ui| Button::new(label).variant(ButtonVariant::Outline).show(ui))
            .show(ui);
        SheetContent::new().side(side).show(ui, |ui| {
            SheetHeader::new().show(ui, |ui| {
                SheetTitle::new(label).show(ui);
                SheetDescription::new(description).show(ui);
            });
            SheetFooter::new().show(ui, |ui| {
                SheetClose::widget(|ui| Button::new("Close").show(ui)).show(ui);
            });
        });
    });
}

fn scratch(ui: &egui::Ui, key: &str) -> String {
    let id = egui::Id::unique(("sheet_demo_scratch", key));
    ui.ctx()
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_default()
}

fn set_scratch(ui: &egui::Ui, key: &str, value: String) {
    let id = egui::Id::unique(("sheet_demo_scratch", key));
    ui.ctx().data_mut(|d| d.insert_temp(id, value));
}
