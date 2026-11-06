//! Alert-dialog demo: destructive confirm, media slot, small size.

use egui_shadcn_ui::prelude::*;

pub fn alert_dialog_demo(ui: &mut egui::Ui) {
    ui.heading("Destructive confirmation");
    ui.add_space(4.0);
    let deleted = confirm_state(ui, "deleted");
    AlertDialog::new().id_salt("demo_delete").show(ui, |ui| {
        AlertDialogTrigger::widget(|ui| {
            Button::new("Delete file")
                .variant(ButtonVariant::Destructive)
                .show(ui)
        })
        .show(ui);
        AlertDialogContent::new().show(ui, |ui| {
            AlertDialogHeader::new().show(ui, |ui| {
                AlertDialogTitle::new("Are you absolutely sure?").show(ui);
                AlertDialogDescription::new(
                    "This action cannot be undone. This will permanently \
                     delete your account and remove your data from our servers.",
                )
                .show(ui);
            });
            AlertDialogFooter::new().show(ui, |ui| {
                AlertDialogCancel::new("Cancel").show(ui);
                if AlertDialogAction::new("Delete account")
                    .variant(ButtonVariant::Destructive)
                    .show(ui)
                    .clicked()
                {
                    set_confirm(ui, "deleted", true);
                }
            });
        });
    });
    ui.add_space(4.0);
    ui.weak(if deleted {
        "Account deleted (action confirmed)."
    } else {
        "Nothing confirmed yet."
    });
    set_confirm(ui, "deleted", deleted);

    ui.add_space(16.0);
    ui.heading("With media icon");
    ui.add_space(4.0);
    AlertDialog::new().id_salt("demo_media").show(ui, |ui| {
        AlertDialogTrigger::widget(|ui| Button::new("Reset settings").show(ui)).show(ui);
        AlertDialogContent::new().show(ui, |ui| {
            AlertDialogHeader::new().show(ui, |ui| {
                AlertDialogMedia::new(Icon::CircleAlert).show(ui);
                AlertDialogTitle::new("Reset all settings?").show(ui);
                AlertDialogDescription::new("Your preferences will return to their defaults.")
                    .show(ui);
            });
            AlertDialogFooter::new().show(ui, |ui| {
                AlertDialogCancel::new("Cancel").show(ui);
                AlertDialogAction::new("Reset").show(ui);
            });
        });
    });

    ui.add_space(16.0);
    ui.heading("Small size (sm)");
    ui.add_space(4.0);
    AlertDialog::new().id_salt("demo_sm").show(ui, |ui| {
        AlertDialogTrigger::widget(|ui| {
            Button::new("Small alert")
                .variant(ButtonVariant::Outline)
                .show(ui)
        })
        .show(ui);
        AlertDialogContent::new()
            .size(AlertDialogSize::Sm)
            .show(ui, |ui| {
                AlertDialogHeader::new().show(ui, |ui| {
                    AlertDialogTitle::new("Small dialog").show(ui);
                    AlertDialogDescription::new(
                        "Centered text with a 320px max width and a \
                         two-column footer.",
                    )
                    .show(ui);
                });
                AlertDialogFooter::new().show(ui, |ui| {
                    AlertDialogCancel::new("Cancel").show(ui);
                    AlertDialogAction::new("Continue").show(ui);
                });
            });
    });
}

fn confirm_state(ui: &egui::Ui, salt: &str) -> bool {
    let id = egui::Id::unique(("alert_dialog_demo", salt));
    ui.ctx()
        .data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false))
}

fn set_confirm(ui: &egui::Ui, salt: &str, value: bool) {
    let id = egui::Id::unique(("alert_dialog_demo", salt));
    ui.ctx().data_mut(|d| d.insert_temp(id, value));
}
