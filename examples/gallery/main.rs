//! Gallery app shell: sidebar with every component demo, light/dark toggle
//! and preset picker. Demos are added per component as they are ported.

#![allow(dead_code)]

use eframe::egui;
use egui_shadcn_ui::prelude::*;

mod demos;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("shadcn/ui for egui — gallery"),
        ..Default::default()
    };
    eframe::run_native(
        "gallery",
        options,
        Box::new(|cc| {
            // Install the default shadcn preset before the first frame.
            Preset::default().install(&cc.egui_ctx);
            Ok(Box::new(GalleryApp::default()))
        }),
    )
}

/// One selectable demo page.
struct DemoPage {
    name: &'static str,
    /// Set once the component's demo module exists.
    show: Option<fn(&mut egui::Ui)>,
}

#[derive(Default)]
struct GalleryApp {
    selected: usize,
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.top_bar(ui);

        egui::Panel::left("gallery_sidebar")
            .resizable(false)
            .default_size(200.0)
            .show(ui, |ui| {
                ui.add_space(8.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, page) in demo_pages().iter().enumerate() {
                        let selected = i == self.selected;
                        if ui.selectable_label(selected, page.name).clicked() {
                            self.selected = i;
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let page = &demo_pages()[self.selected];
                ui.heading(page.name);
                ui.add_space(12.0);
                match page.show {
                    Some(show) => show(ui),
                    None => {
                        ui.label("Demo not ported yet.");
                    }
                }
            });
        });
    }
}

impl GalleryApp {
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("gallery_topbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let theme = ShadcnTheme::of(ui);
                ui.label("shadcn/ui — egui edition");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Light / dark toggle.
                    let label = if theme.mode.is_dark() {
                        "🌙 Dark"
                    } else {
                        "☀️ Light"
                    };
                    if ui.button(label).clicked() {
                        ShadcnTheme::set_mode(ui.ctx(), theme.mode.toggle());
                    }

                    // Preset picker: base color selector.
                    let mut preset = theme.preset.clone();
                    egui::ComboBox::from_id_salt("preset_base_color")
                        .selected_text(format!("base: {}", preset.base_color.name()))
                        .show_ui(ui, |ui| {
                            for color in [
                                BaseColor::Neutral,
                                BaseColor::Stone,
                                BaseColor::Zinc,
                                BaseColor::Mauve,
                                BaseColor::Olive,
                                BaseColor::Mist,
                                BaseColor::Taupe,
                            ] {
                                ui.selectable_value(&mut preset.base_color, color, color.name());
                            }
                        });
                    if preset != theme.preset {
                        preset.install(ui.ctx());
                    }

                    // Style picker.
                    let mut preset = theme.preset.clone();
                    egui::ComboBox::from_id_salt("preset_style")
                        .selected_text(format!("style: {}", preset.style.name()))
                        .show_ui(ui, |ui| {
                            for style in [
                                Style::Nova,
                                Style::Vega,
                                Style::Maia,
                                Style::Lyra,
                                Style::Mira,
                                Style::Luma,
                                Style::Sera,
                                Style::Rhea,
                            ] {
                                ui.selectable_value(&mut preset.style, style, style.name());
                            }
                        });
                    if preset != theme.preset {
                        preset.install(ui.ctx());
                    }
                });
            });
        });
    }
}

/// The full list of demo pages; filled in as components are ported.
fn demo_pages() -> Vec<DemoPage> {
    vec![
        DemoPage {
            name: "button",
            show: Some(demos::button_demo),
        },
        DemoPage {
            name: "badge",
            show: Some(demos::badge_demo),
        },
        DemoPage {
            name: "label",
            show: Some(demos::label_demo),
        },
        DemoPage {
            name: "separator",
            show: Some(demos::separator_demo),
        },
        DemoPage {
            name: "kbd",
            show: Some(demos::kbd_demo),
        },
        DemoPage {
            name: "spinner",
            show: Some(demos::spinner_demo),
        },
        DemoPage {
            name: "skeleton",
            show: Some(demos::skeleton_demo),
        },
        DemoPage {
            name: "typography",
            show: Some(demos::typography_demo),
        },
        DemoPage {
            name: "aspect-ratio",
            show: Some(demos::aspect_ratio_demo),
        },
        DemoPage {
            name: "marker",
            show: Some(demos::marker_demo),
        },
        DemoPage {
            name: "button-group",
            show: Some(demos::button_group_demo),
        },
        DemoPage {
            name: "input",
            show: Some(demos::input_demo),
        },
        DemoPage {
            name: "textarea",
            show: Some(demos::textarea_demo),
        },
        DemoPage {
            name: "checkbox",
            show: Some(demos::checkbox_demo),
        },
        DemoPage {
            name: "radio-group",
            show: Some(demos::radio_group_demo),
        },
        DemoPage {
            name: "switch",
            show: Some(demos::switch_demo),
        },
        DemoPage {
            name: "slider",
            show: Some(demos::slider_demo),
        },
        DemoPage {
            name: "toggle",
            show: Some(demos::toggle_demo),
        },
        DemoPage {
            name: "toggle-group",
            show: Some(demos::toggle_group_demo),
        },
        DemoPage {
            name: "input-otp",
            show: Some(demos::input_otp_demo),
        },
        DemoPage {
            name: "native-select",
            show: None,
        },
        DemoPage {
            name: "field",
            show: None,
        },
        DemoPage {
            name: "input-group",
            show: None,
        },
        DemoPage {
            name: "card",
            show: None,
        },
        DemoPage {
            name: "alert",
            show: None,
        },
        DemoPage {
            name: "avatar",
            show: None,
        },
        DemoPage {
            name: "progress",
            show: None,
        },
        DemoPage {
            name: "table",
            show: None,
        },
        DemoPage {
            name: "empty",
            show: None,
        },
        DemoPage {
            name: "item",
            show: None,
        },
        DemoPage {
            name: "scroll-area",
            show: None,
        },
        DemoPage {
            name: "resizable",
            show: None,
        },
        DemoPage {
            name: "accordion",
            show: None,
        },
        DemoPage {
            name: "collapsible",
            show: None,
        },
        DemoPage {
            name: "tabs",
            show: None,
        },
        DemoPage {
            name: "breadcrumb",
            show: None,
        },
        DemoPage {
            name: "pagination",
            show: None,
        },
        DemoPage {
            name: "dialog",
            show: None,
        },
        DemoPage {
            name: "alert-dialog",
            show: None,
        },
        DemoPage {
            name: "sheet",
            show: None,
        },
        DemoPage {
            name: "drawer",
            show: None,
        },
        DemoPage {
            name: "popover",
            show: None,
        },
        DemoPage {
            name: "tooltip",
            show: None,
        },
        DemoPage {
            name: "hover-card",
            show: None,
        },
        DemoPage {
            name: "sonner",
            show: None,
        },
        DemoPage {
            name: "dropdown-menu",
            show: None,
        },
        DemoPage {
            name: "context-menu",
            show: None,
        },
        DemoPage {
            name: "menubar",
            show: None,
        },
        DemoPage {
            name: "navigation-menu",
            show: None,
        },
        DemoPage {
            name: "select",
            show: None,
        },
        DemoPage {
            name: "combobox",
            show: None,
        },
        DemoPage {
            name: "command",
            show: None,
        },
        DemoPage {
            name: "calendar",
            show: None,
        },
        DemoPage {
            name: "date-picker",
            show: None,
        },
        DemoPage {
            name: "carousel",
            show: None,
        },
        DemoPage {
            name: "chart",
            show: None,
        },
        DemoPage {
            name: "data-table",
            show: None,
        },
        DemoPage {
            name: "form",
            show: None,
        },
        DemoPage {
            name: "sidebar",
            show: None,
        },
        DemoPage {
            name: "attachment",
            show: None,
        },
        DemoPage {
            name: "bubble",
            show: None,
        },
        DemoPage {
            name: "message",
            show: None,
        },
        DemoPage {
            name: "message-scroller",
            show: None,
        },
        DemoPage {
            name: "direction",
            show: None,
        },
    ]
}
