//! Sidebar demo: collapsible rails (offcanvas + icon), variants, menu
//! structures (group/menu actions, skeletons), trigger/rail/keyboard
//! toggles.

use egui_shadcn_ui::prelude::*;

fn sidebar_menu(ui: &mut egui::Ui, salt: &str) {
    SidebarContent::new().show(ui, |ui| {
        SidebarGroup::new().show(ui, |ui| {
            SidebarGroupLabel::new("Platform").show(ui);
            SidebarGroupAction::new(Icon::Plus).show(ui);
            SidebarGroupContent::new().show(ui, |ui| {
                SidebarMenu::new().show(ui, |ui| {
                    SidebarMenuItem::new().show(ui, |ui| {
                        SidebarMenuButton::new("History", Icon::Message)
                            .tooltip("History")
                            .is_active(true)
                            .id_salt(format!("{salt}_history"))
                            .show(ui);
                        SidebarMenuAction::new(Icon::MoreVertical)
                            .show_on_hover(true)
                            .show(ui);
                    });
                    SidebarMenuItem::new().show(ui, |ui| {
                        SidebarMenuButton::new("Starred", Icon::Sparkles)
                            .tooltip("Starred")
                            .badge("6")
                            .id_salt(format!("{salt}_starred"))
                            .show(ui);
                    });
                    SidebarMenuItem::new().show(ui, |ui| {
                        SidebarMenuButton::new("Outline", Icon::Pencil)
                            .tooltip("Outline")
                            .variant(SidebarMenuButtonVariant::Outline)
                            .id_salt(format!("{salt}_outline"))
                            .show(ui);
                        SidebarMenuAction::new(Icon::MoreVertical).show(ui);
                    });
                    SidebarMenuItem::new().show(ui, |ui| {
                        SidebarMenuButton::new("Disabled", Icon::CircleAlert)
                            .tooltip("Disabled")
                            .disabled(true)
                            .id_salt(format!("{salt}_disabled"))
                            .show(ui);
                    });
                    SidebarMenuSub::new().show(ui, |ui| {
                        SidebarMenuSubButton::new("Drafts", None::<Icon>)
                            .is_active(true)
                            .show(ui);
                        SidebarMenuSubButton::new("Sent", None::<Icon>)
                            .size(SidebarMenuSubSize::Sm)
                            .show(ui);
                    });
                });
            });
        });
        SidebarSeparator::new().show(ui);
        SidebarGroup::new().show(ui, |ui| {
            SidebarGroupLabel::new("Sizes").show(ui);
            SidebarMenu::new().show(ui, |ui| {
                SidebarMenuItem::new().show(ui, |ui| {
                    SidebarMenuButton::new("Small", Icon::Copy)
                        .size(SidebarMenuButtonSize::Sm)
                        .show(ui);
                });
                SidebarMenuItem::new().show(ui, |ui| {
                    SidebarMenuButton::new("Large", Icon::CalendarDays)
                        .size(SidebarMenuButtonSize::Lg)
                        .show(ui);
                });
            });
        });
        SidebarSeparator::new().show(ui);
        SidebarGroup::new().show(ui, |ui| {
            SidebarGroupLabel::new("Loading").show(ui);
            SidebarMenu::new().show(ui, |ui| {
                SidebarMenuSkeleton::new().show_icon(true).show(ui);
                SidebarMenuSkeleton::new().show_icon(true).show(ui);
                SidebarMenuSkeleton::new().show(ui);
            });
        });
    });
}

pub fn run(ui: &mut egui::Ui) {
    ui.label(
        "Collapsible = icon (Ctrl/Cmd+B or the trigger toggles; Escape collapses when focused; \
         hover a collapsed button for its tooltip)",
    );
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Sidebar::new()
            .id_salt("demo_icon")
            .variant(SidebarVariant::Sidebar)
            .collapsible(SidebarCollapsible::Icon)
            .show(ui, |ui| {
                SidebarHeader::new().show(ui, |ui| {
                    SidebarTrigger::new().show(ui);
                });
                sidebar_menu(ui, "icon");
                SidebarFooter::new().show(ui, |ui| {
                    SidebarMenuButton::new("Account", Icon::User)
                        .tooltip("Account")
                        .show(ui);
                });
                SidebarRail::new().show(ui);
            });
        SidebarInset::new().show(ui, |ui| {
            ui.label("Main content (inset variant pairs the sidebar with a rounded card)");
        });
    });

    ui.add_space(16.0);
    ui.label("Collapsible = offcanvas (default), floating variant");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Sidebar::new()
            .id_salt("demo_floating")
            .variant(SidebarVariant::Floating)
            .show(ui, |ui| {
                SidebarHeader::new().show(ui, |ui| {
                    SidebarTrigger::new().show(ui);
                });
                sidebar_menu(ui, "floating");
                SidebarRail::new().show(ui);
            });
        ui.vertical(|ui| {
            ui.label("Main content (floating sidebar)");
        });
    });

    ui.add_space(16.0);
    ui.label("Right side + inset variant");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Sidebar::new()
            .id_salt("demo_inset")
            .side(SidebarSide::Right)
            .variant(SidebarVariant::Inset)
            .collapsible(SidebarCollapsible::Icon)
            .show(ui, |ui| {
                SidebarHeader::new().show(ui, |ui| {
                    SidebarTrigger::new().show(ui);
                });
                sidebar_menu(ui, "inset");
            });
        SidebarInset::new().show(ui, |ui| {
            ui.label("Main content (right-side inset)");
        });
    });

    ui.add_space(16.0);
    ui.label("Non-collapsible (trigger does nothing) + external trigger");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        SidebarTrigger::with_salt("demo_fixed").show(ui);
        ui.label("External SidebarTrigger::with_salt toggles the sidebar below.");
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Sidebar::new()
            .id_salt("demo_fixed")
            .collapsible(SidebarCollapsible::Offcanvas)
            .show(ui, |ui| {
                SidebarHeader::new().show(ui, |ui| {
                    SidebarTrigger::new().show(ui);
                });
                sidebar_menu(ui, "fixed");
            });
        ui.vertical(|ui| {
            ui.label("Main content (non-collapsible)");
        });
    });

    ui.add_space(16.0);
    ui.label("Controlled open state");
    ui.add_space(4.0);
    let store = egui::Id::unique("sidebar_demo_controlled");
    let mut open: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(store).unwrap_or(true));
    ui.horizontal(|ui| {
        let r = Sidebar::controlled(&mut open)
            .id_salt("demo_controlled")
            .collapsible(SidebarCollapsible::Icon)
            .show(ui, |ui| {
                SidebarHeader::new().show(ui, |ui| {
                    SidebarTrigger::new().show(ui);
                });
                SidebarContent::new().show(ui, |ui| {
                    SidebarMenu::new().show(ui, |ui| {
                        SidebarMenuItem::new().show(ui, |ui| {
                            SidebarMenuButton::new("Only item", Icon::CircleCheck).show(ui);
                        });
                    });
                });
            });
        if r.controlled_updated {
            ui.ctx().data_mut(|d| d.insert_temp(store, r.open));
        }
        ui.vertical(|ui| {
            ui.label(format!("Controlled open: {}", r.open));
        });
    });
    ui.ctx().data_mut(|d| d.insert_temp(store, open));
}
