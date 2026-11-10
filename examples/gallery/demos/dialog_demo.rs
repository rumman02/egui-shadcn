//! Dialog demo: basic form dialog, controlled, no-close-button, footer
//! close button, custom width.

use egui_shadcn_ui::prelude::*;

pub fn dialog_demo(ui: &mut egui::Ui) {
    ui.heading("Form dialog");
    ui.add_space(4.0);
    let mut name = scratch(ui, "name");
    Dialog::new().id_salt("demo_form").show(ui, |ui| {
        DialogTrigger::widget(|ui| Button::new("Edit profile").show(ui)).show(ui);
        DialogContent::new().show(ui, |ui| {
            DialogHeader::new().show(ui, |ui| {
                DialogTitle::new("Edit profile").show(ui);
                DialogDescription::new(
                    "Make changes to your profile here. Click save when you're done.",
                )
                .show(ui);
            });
            ui.label("Name");
            ui.add(egui::TextEdit::singleline(&mut name).desired_width(f32::INFINITY));
            ui.label("Email");
            let mut email = scratch(ui, "email");
            ui.add(
                egui::TextEdit::singleline(&mut email)
                    .hint_text("you@example.com")
                    .desired_width(f32::INFINITY),
            );
            set_scratch(ui, "email", email);
            DialogFooter::new().show(ui, |ui| {
                DialogClose::widget(|ui| {
                    Button::new("Cancel")
                        .variant(ButtonVariant::Outline)
                        .show(ui)
                })
                .show(ui);
                DialogClose::widget(|ui| Button::new("Save changes").show(ui)).show(ui);
            });
        });
    });
    set_scratch(ui, "name", name);

    ui.add_space(16.0);
    ui.heading("Controlled");
    ui.add_space(4.0);
    let mut open = open_state(ui, "demo_controlled");
    ui.horizontal(|ui| {
        Dialog::controlled(&mut open)
            .id_salt("demo_controlled")
            .show(ui, |ui| {
                DialogTrigger::widget(|ui| Button::new("Open controlled").show(ui)).show(ui);
                DialogContent::new().show(ui, |ui| {
                    DialogHeader::new().show(ui, |ui| {
                        DialogTitle::new("Controlled dialog").show(ui);
                        DialogDescription::new(
                            "The open state is owned by the caller. Escape, \
                             backdrop click and the X button all write back.",
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
    ui.heading("Without close button");
    ui.add_space(4.0);
    Dialog::new().id_salt("demo_noclose").show(ui, |ui| {
        DialogTrigger::widget(|ui| {
            Button::new("No X button")
                .variant(ButtonVariant::Outline)
                .show(ui)
        })
        .show(ui);
        DialogContent::new()
            .show_close_button(false)
            .show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("Dismiss with Escape or backdrop").show(ui);
                    DialogDescription::new("showCloseButton = false hides the X.").show(ui);
                });
                DialogFooter::new().show(ui, |ui| {
                    DialogClose::widget(|ui| Button::new("Got it").show(ui)).show(ui);
                });
            });
    });

    ui.add_space(16.0);
    ui.heading("Custom width");
    ui.add_space(4.0);
    Dialog::new().id_salt("demo_wide").show(ui, |ui| {
        DialogTrigger::widget(|ui| Button::new("Wide (560px)").show(ui)).show(ui);
        DialogContent::new().width(560.0).show(ui, |ui| {
            DialogHeader::new().show(ui, |ui| {
                DialogTitle::new("Wide dialog").show(ui);
                DialogDescription::new(
                    "Content wider than the sm:max-w-sm default; the panel \
                     is clamped to the window minus 2rem.",
                )
                .show(ui);
            });
        });
    });

    ui.add_space(16.0);
    ui.heading("Footer close button");
    ui.add_space(4.0);
    Dialog::new().id_salt("demo_footer_close").show(ui, |ui| {
        DialogTrigger::widget(|ui| Button::new("Footer showCloseButton").show(ui)).show(ui);
        DialogContent::new().show(ui, |ui| {
            DialogHeader::new().show(ui, |ui| {
                DialogTitle::new("Confirm action").show(ui);
                DialogDescription::new(
                    "The footer appends an outline Close button \
                     (showCloseButton = true).",
                )
                .show(ui);
            });
            DialogFooter::new().show_close_button(true).show(ui, |ui| {
                DialogClose::widget(|ui| Button::new("Continue").show(ui)).show(ui);
            });
        });
    });
}

fn scratch(ui: &egui::Ui, key: &str) -> String {
    let id = egui::Id::unique(("dialog_demo_scratch", key));
    ui.ctx()
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_default()
}

fn set_scratch(ui: &egui::Ui, key: &str, value: String) {
    let id = egui::Id::unique(("dialog_demo_scratch", key));
    ui.ctx().data_mut(|d| d.insert_temp(id, value));
}

fn open_state(ui: &egui::Ui, salt: &str) -> bool {
    let id = egui::Id::unique(("dialog_demo_open", salt));
    ui.ctx()
        .data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false))
}

fn set_open_state(ui: &egui::Ui, salt: &str, open: bool) {
    let id = egui::Id::unique(("dialog_demo_open", salt));
    ui.ctx().data_mut(|d| d.insert_temp(id, open));
}

fn open_label(open: bool) -> &'static str {
    if open {
        "open"
    } else {
        "closed"
    }
}
