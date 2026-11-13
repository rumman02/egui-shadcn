//! Accordion demo: single (collapsible), multiple (controlled), and the
//! non-collapsible + disabled-trigger combination from the source demo.

use egui_shadcn_ui::prelude::*;

pub fn accordion_demo(ui: &mut egui::Ui) {
    let multi_store = egui::Id::unique("accordion_demo_multi");
    let mut multi: Vec<bool> = ui
        .ctx()
        .data_mut(|d| d.get_temp(multi_store).unwrap_or(vec![true, false, false]));

    ui.label("Single, collapsible (first item open by default)");
    ui.add_space(4.0);
    Accordion::new()
        .id_salt("accordion_demo_single")
        .default_open(0)
        .collapsible(true)
        .show(ui, |ui| {
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Product Information").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label(
                        "Our flagship product combines cutting-edge technology with sleek design. \
                     Built with premium materials, it offers unparalleled performance.",
                    );
                    // The source demo content is `flex flex-col gap-4`.
                    ui.add_space(8.0);
                    ui.label(
                        "Key features include advanced processing capabilities and an intuitive \
                     user interface designed for both beginners and experts.",
                    );
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Shipping Details").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label(
                        "We offer worldwide shipping through trusted courier partners. Standard \
                     delivery takes 3-5 business days, express shipping 1-2 business days.",
                    );
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Return Policy").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label(
                        "We stand behind our products with a comprehensive 30-day return policy. \
                     Free return shipping and full refunds processed within 48 hours.",
                    );
                });
            });
        });

    ui.add_space(12.0);
    ui.label(format!("Multiple (open flags: {multi:?})"));
    ui.add_space(4.0);
    Accordion::multiple(&mut multi)
        .id_salt("accordion_demo_multi")
        .show(ui, |ui| {
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Billing").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Invoices are issued monthly and can be downloaded as PDFs.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Team members").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Invite unlimited teammates on the Pro plan.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("API access").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Generate personal access tokens from the dashboard.");
                });
            });
        });

    ui.add_space(12.0);
    ui.label("Disabled trigger and non-collapsible single");
    ui.add_space(4.0);
    Accordion::new()
        .id_salt("accordion_demo_locked")
        .default_open(0)
        .collapsible(false)
        .show(ui, |ui| {
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Always open (collapsible = false)").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Clicking this trigger keeps the section open.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Unavailable").disabled(true).show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("You cannot open this section.");
                });
            });
        });

    ui.add_space(12.0);
    ui.label(
        "Keyboard: ArrowUp/Down (or Left/Right) move between triggers, Home/End jump; \
         Enter or Space toggles.",
    );

    ui.ctx().data_mut(|d| d.insert_temp(multi_store, multi));
}
