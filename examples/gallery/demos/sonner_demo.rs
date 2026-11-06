//! Sonner demo: one button per toast variant, plus a rich toast.
//!
//! The toaster itself renders at the bottom-right of the window (sonner's
//! default position); these buttons just enqueue toasts.

use egui_shadcn_ui::prelude::*;

pub fn sonner_demo(ui: &mut egui::Ui) {
    ui.heading("Variants");
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        if Button::new("Default").show(ui).clicked() {
            Sonner::toast(ui.ctx(), Toast::new("Event has been created."));
        }
        if Button::new("Success")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            Sonner::toast(
                ui.ctx(),
                Toast::new("Event has been created.")
                    .description("Monday, January 6th at 12:01pm")
                    .variant(ToastVariant::Success),
            );
        }
        if Button::new("Error")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            Sonner::toast(
                ui.ctx(),
                Toast::new("Uh oh! Something went wrong.")
                    .description("There was a problem with your request.")
                    .variant(ToastVariant::Error),
            );
        }
        if Button::new("Warning")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            Sonner::toast(
                ui.ctx(),
                Toast::new("Heads up!")
                    .description("You're running low on disk space.")
                    .variant(ToastVariant::Warning),
            );
        }
        if Button::new("Info")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            Sonner::toast(
                ui.ctx(),
                Toast::new("Did you know?")
                    .description("You can dismiss a toast by clicking it.")
                    .variant(ToastVariant::Info),
            );
        }
        if Button::new("Loading")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            Sonner::toast(
                ui.ctx(),
                Toast::new("Uploading file…")
                    .description("report-q4.pdf")
                    .variant(ToastVariant::Loading),
            );
        }
    });

    ui.add_space(16.0);
    ui.heading("Timing");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if Button::new("Sticky (60s)")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            Sonner::toast(
                ui.ctx(),
                Toast::new("I'll stick around")
                    .description("Click me to dismiss.")
                    .duration(60.0),
            );
        }
        if Button::new("Short (1s)")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            Sonner::toast(ui.ctx(), Toast::new("Gone in a second").duration(1.0));
        }
        if Button::new("Clear all")
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            Sonner::clear(ui.ctx());
        }
    });

    // The toaster itself: bottom-right stack, newest at the bottom.
    Sonner::show(ui);
}
