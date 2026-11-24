//! Accordion demo: single (collapsible), multiple (controlled), the
//! non-collapsible + disabled-trigger combination from the source demo,
//! long/wrapping labels (N6), and uncontrolled multiple via `new_multiple`
//! + `default_open_many` (N3).

use egui_shadcn_ui::prelude::*;

pub fn accordion_demo(ui: &mut egui::Ui) {
    let multi_store = egui::Id::unique("accordion_demo_multi");
    let mut multi: Vec<bool> = ui
        .ctx()
        .data_mut(|d| d.get_temp(multi_store).unwrap_or(vec![true, false, false]));
    let activations_store = egui::Id::unique("accordion_demo_uncontrolled_multi_activations");
    let mut activations: u32 = ui
        .ctx()
        .data_mut(|d| d.get_temp(activations_store).unwrap_or(0));

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
        "Long label — wraps with 16px clearance before the chevron (N6). Narrow column \
         (440px, sidebar width) so the 79-character label reaches the chevron, as in the S1 test.",
    );
    ui.add_space(4.0);
    ui.scope(|ui| {
        ui.set_max_width(440.0);
        Accordion::new()
            .id_salt("accordion_demo_long_label")
            .default_open(0)
            .collapsible(true)
            .show(ui, |ui| {
                AccordionItem::new().show(ui, |ui| {
                    AccordionTrigger::new(
                        "What is the return policy for items purchased during the holiday sale period?",
                    )
                    .show(ui);
                    AccordionContent::new().show(ui, |ui| {
                        ui.label(
                            "The label wraps inside its own width and never paints under the chevron; \
                         the row grows to contain the wrapped text.",
                        );
                        // The source demo content is `flex flex-col gap-4`.
                        ui.add_space(8.0);
                        ui.label("The chevron stays centred on the first line of the wrapped label.");
                    });
                });
            });
    });

    ui.add_space(12.0);
    ui.label(
        "Wrapping label — multi-line, the row grows (N6). Same narrow column; the chevron \
         stays centred on the first line box.",
    );
    ui.add_space(4.0);
    ui.scope(|ui| {
        ui.set_max_width(440.0);
        Accordion::new()
            .id_salt("accordion_demo_wrapping_label")
            .default_open(0)
            .collapsible(true)
            .show(ui, |ui| {
                AccordionItem::new().show(ui, |ui| {
                    AccordionTrigger::new(
                        "How do I change the delivery address on an order that has already shipped, \
                     and can I still add a gift message after checkout?",
                    )
                    .show(ui);
                    AccordionContent::new().show(ui, |ui| {
                        ui.label(
                            "A very long label wraps to several lines; the row grows with it.",
                        );
                    });
                });
            });
    });

    ui.add_space(12.0);
    ui.label(
        "Uncontrolled multiple — items 0 and 2 open by default (N3, `new_multiple` + \
         `default_open_many([0, 2])`)",
    );
    ui.add_space(4.0);
    Accordion::new_multiple()
        .id_salt("accordion_demo_uncontrolled_multi")
        .default_open_many([0, 2])
        .show(ui, |ui| {
            AccordionItem::new().show(ui, |ui| {
                if AccordionTrigger::new("Getting started").show(ui).clicked() {
                    activations += 1;
                }
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Open by default via `default_open_many([0, 2])`.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                if AccordionTrigger::new("Billing").show(ui).clicked() {
                    activations += 1;
                }
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Closed by default; click to open alongside the others.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                if AccordionTrigger::new("API access").show(ui).clicked() {
                    activations += 1;
                }
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Open by default via `default_open_many([0, 2])`.");
                });
            });
        });
    ui.small(format!("trigger activations: {activations}"));

    ui.add_space(12.0);
    ui.label(
        "Keyboard: ArrowUp/Down (or Left/Right) move between triggers, Home/End jump; \
         Enter or Space toggles. Arrow keys stop at the ends (no wrap — OPEN-2, unresolved); \
         whether focus lands on a disabled trigger is unresolved (OPEN-3) — S6 settles both \
         from runtime evidence.",
    );

    ui.ctx().data_mut(|d| d.insert_temp(multi_store, multi));
    ui.ctx()
        .data_mut(|d| d.insert_temp(activations_store, activations));
}
