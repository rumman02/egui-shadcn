//! Drawer demo: bottom sheet with drag handle, footer, and no-handle mode.

use egui_shadcn_ui::prelude::*;

pub fn drawer_demo(ui: &mut egui::Ui) {
    ui.heading("Bottom sheet with drag handle");
    ui.add_space(4.0);
    ui.weak("Drag the panel down to dismiss; release past the threshold to close.");
    ui.add_space(4.0);
    Drawer::new().id_salt("demo_basic").show(ui, |ui| {
        DrawerTrigger::widget(|ui| Button::new("Open drawer").show(ui)).show(ui);
        DrawerContent::new().show(ui, |ui| {
            DrawerHeader::new().show(ui, |ui| {
                DrawerTitle::new("Drawer with handle").show(ui);
                DrawerDescription::new(
                    "The vaul-style bottom drawer. Drag it down or press Escape.",
                )
                .show(ui);
            });
            DrawerFooter::new().show(ui, |ui| {
                DrawerClose::widget(|ui| Button::new("Close").show(ui)).show(ui);
            });
        });
    });

    ui.add_space(16.0);
    ui.heading("Controlled");
    ui.add_space(4.0);
    let mut open = open_state(ui, "demo_controlled");
    ui.horizontal(|ui| {
        Drawer::controlled(&mut open)
            .id_salt("demo_controlled")
            .show(ui, |ui| {
                DrawerTrigger::widget(|ui| Button::new("Open controlled").show(ui)).show(ui);
                DrawerContent::new().show(ui, |ui| {
                    DrawerHeader::new().show(ui, |ui| {
                        DrawerTitle::new("Controlled drawer").show(ui);
                        DrawerDescription::new(
                            "Open state is owned by the caller; drag-dismiss \
                             writes back into the bool.",
                        )
                        .show(ui);
                    });
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
    ui.heading("Without handle");
    ui.add_space(4.0);
    Drawer::new().id_salt("demo_nohandle").show(ui, |ui| {
        DrawerTrigger::widget(|ui| {
            Button::new("No handle")
                .variant(ButtonVariant::Outline)
                .show(ui)
        })
        .show(ui);
        DrawerContent::new().show_handle(false).show(ui, |ui| {
            DrawerHeader::new().show(ui, |ui| {
                DrawerTitle::new("Handle hidden").show(ui);
                DrawerDescription::new("show_handle(false) removes the grip bar.").show(ui);
            });
            DrawerFooter::new().show(ui, |ui| {
                DrawerClose::widget(|ui| Button::new("Dismiss").show(ui)).show(ui);
            });
        });
    });
}

fn open_state(ui: &egui::Ui, salt: &str) -> bool {
    let id = egui::Id::unique(("drawer_demo_open", salt));
    ui.ctx()
        .data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false))
}

fn set_open_state(ui: &egui::Ui, salt: &str, open: bool) {
    let id = egui::Id::unique(("drawer_demo_open", salt));
    ui.ctx().data_mut(|d| d.insert_temp(id, open));
}

fn open_label(open: bool) -> &'static str {
    if open {
        "open"
    } else {
        "closed"
    }
}
