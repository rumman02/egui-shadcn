//! Popover demo: basic, controlled, alignments and custom width.

use egui_shadcn_ui::prelude::*;

pub fn popover_demo(ui: &mut egui::Ui) {
    ui.heading("Basic");
    ui.add_space(4.0);
    Popover::new().id_salt("demo_basic").show(ui, |ui| {
        PopoverTrigger::new("Open popover").show(ui);
        PopoverContent::new().show(ui, |ui| {
            PopoverHeader::new().show(ui, |ui| {
                PopoverTitle::new("Dimensions").show(ui);
                PopoverDescription::new("Set the dimensions for the layer.").show(ui);
            });
            ui.add_space(4.0);
            let mut width = scratch(ui, "w").unwrap_or_default();
            let mut height = scratch(ui, "h").unwrap_or_default();
            ui.label("Width");
            ui.add(egui::TextEdit::singleline(&mut width).desired_width(120.0));
            ui.label("Height");
            ui.add(egui::TextEdit::singleline(&mut height).desired_width(120.0));
            set_scratch(ui, "w", width);
            set_scratch(ui, "h", height);
            ui.add_space(6.0);
            Button::new("Save").show(ui);
        });
    });

    ui.add_space(16.0);
    ui.heading("Controlled");
    ui.add_space(4.0);
    let mut open = open_state(ui, "demo_controlled");
    ui.horizontal(|ui| {
        Popover::controlled(&mut open)
            .id_salt("demo_controlled")
            .show(ui, |ui| {
                PopoverTrigger::widget(|ui| Button::new("Controlled popover").show(ui)).show(ui);
                PopoverContent::new().show(ui, |ui| {
                    PopoverTitle::new("Controlled").show(ui);
                    PopoverDescription::new(
                        "Open state is owned by the caller; Escape and \
                         click-outside write back into the bool.",
                    )
                    .show(ui);
                });
            });
        if Button::new(open_label(open))
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            set_open_state(ui, "demo_controlled", !open);
        }
    });

    ui.add_space(16.0);
    ui.heading("Alignments");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Popover::new().id_salt("demo_start").show(ui, |ui| {
            PopoverTrigger::widget(|ui| {
                Button::new("Align start")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
            })
            .show(ui);
            PopoverContent::new()
                .align(PopoverAlign::Start)
                .show(ui, |ui| {
                    ui.label("Aligned to the trigger's left edge (align=\"start\").");
                });
        });
        Popover::new().id_salt("demo_end").show(ui, |ui| {
            PopoverTrigger::widget(|ui| {
                Button::new("Align end")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
            })
            .show(ui);
            PopoverContent::new()
                .align(PopoverAlign::End)
                .show(ui, |ui| {
                    ui.label("Aligned to the trigger's right edge (align=\"end\").");
                });
        });
    });

    ui.add_space(16.0);
    ui.heading("Custom width");
    ui.add_space(4.0);
    Popover::new().id_salt("demo_wide").show(ui, |ui| {
        PopoverTrigger::widget(|ui| Button::new("Wide (400px)").show(ui)).show(ui);
        PopoverContent::new().width(400.0).show(ui, |ui| {
            PopoverTitle::new("Extra content").show(ui);
            PopoverDescription::new(
                "Content wider than the w-72 default; text wraps to the \
                 full width of the popover.",
            )
            .show(ui);
        });
    });
}

fn scratch(ui: &egui::Ui, key: &str) -> Option<String> {
    let id = egui::Id::unique(("popover_demo_scratch", key));
    ui.ctx().data_mut(|d| d.get_temp::<String>(id))
}

fn set_scratch(ui: &egui::Ui, key: &str, value: String) {
    let id = egui::Id::unique(("popover_demo_scratch", key));
    ui.ctx().data_mut(|d| d.insert_temp(id, value));
}

fn open_state(ui: &egui::Ui, salt: &str) -> bool {
    let id = egui::Id::unique(("popover_demo_open", salt));
    ui.ctx()
        .data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false))
}

fn set_open_state(ui: &egui::Ui, salt: &str, open: bool) {
    let id = egui::Id::unique(("popover_demo_open", salt));
    ui.ctx().data_mut(|d| d.insert_temp(id, open));
}

fn open_label(open: bool) -> &'static str {
    if open {
        "open"
    } else {
        "closed"
    }
}
