//! Tooltip demo: simple, sides, delay and rich content.

use egui_shadcn_ui::prelude::*;

pub fn tooltip_demo(ui: &mut egui::Ui) {
    TooltipProvider::new().delay_ms(300).install(ui.ctx());

    ui.heading("Simple");
    ui.add_space(4.0);
    Tooltip::new().id_salt("demo_simple").show(ui, |ui| {
        TooltipTrigger::widget(|ui| Button::new("Hover me").show(ui)).show(ui);
        TooltipContent::new().show(ui, |ui| {
            ui.label("Simple tooltip");
        });
    });

    ui.add_space(16.0);
    ui.heading("Sides");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Tooltip::new().id_salt("demo_top").show(ui, |ui| {
            TooltipTrigger::widget(|ui| {
                Button::new("Top").variant(ButtonVariant::Outline).show(ui)
            })
            .show(ui);
            TooltipContent::new().side(TooltipSide::Top).show(ui, |ui| {
                ui.label("Above the trigger");
            });
        });
        Tooltip::new().id_salt("demo_bottom").show(ui, |ui| {
            TooltipTrigger::widget(|ui| {
                Button::new("Bottom")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
            })
            .show(ui);
            TooltipContent::new()
                .side(TooltipSide::Bottom)
                .show(ui, |ui| {
                    ui.label("Below the trigger");
                });
        });
        Tooltip::new().id_salt("demo_left").show(ui, |ui| {
            TooltipTrigger::widget(|ui| {
                Button::new("Left").variant(ButtonVariant::Outline).show(ui)
            })
            .show(ui);
            TooltipContent::new()
                .side(TooltipSide::Left)
                .show(ui, |ui| {
                    ui.label("To the left");
                });
        });
        Tooltip::new().id_salt("demo_right").show(ui, |ui| {
            TooltipTrigger::widget(|ui| {
                Button::new("Right")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
            })
            .show(ui);
            TooltipContent::new()
                .side(TooltipSide::Right)
                .show(ui, |ui| {
                    ui.label("To the right");
                });
        });
    });

    ui.add_space(16.0);
    ui.heading("With keyboard shortcut");
    ui.add_space(4.0);
    Tooltip::new().id_salt("demo_kbd").show(ui, |ui| {
        TooltipTrigger::widget(|ui| {
            Button::new("Save")
                .variant(ButtonVariant::Outline)
                .icon_start(Icon::Pencil)
                .show(ui)
        })
        .show(ui);
        TooltipContent::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Save changes");
                Kbd::new("⌘S").show(ui);
            });
        });
    });

    ui.add_space(16.0);
    ui.heading("Slow delay (800ms)");
    ui.add_space(4.0);
    Tooltip::new()
        .delay_ms(800)
        .id_salt("demo_slow")
        .show(ui, |ui| {
            TooltipTrigger::widget(|ui| {
                Button::new("Slow tooltip")
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
            })
            .show(ui);
            TooltipContent::new().show(ui, |ui| {
                ui.label("Waits 800ms before opening");
            });
        });
}
