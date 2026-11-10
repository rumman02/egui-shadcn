//! shadcn `context-menu.tsx` — the right-click menu.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/context-menu.tsx` (Radix
//! ContextMenu) with the nova recipe: content `bg-popover text-popover-foreground
//! min-w-36 rounded-lg p-1 shadow-md ring-1 ring-foreground/10`, the same
//! item rows as the dropdown-menu (nova puts the checkbox/radio indicator on
//! the right: `pr-8`, indicator `right-2`), labels `text-muted-foreground
//! text-xs`, destructive items `text-destructive` with
//! `focus:bg-destructive/10` (dark: `/20`).
//!
//! The trigger is the region laid out so far in the current [`Ui`] (the
//! manual analog of `response.context_menu`): a right-click anywhere in it
//! opens the menu at the pointer. Items, sub-menus, checkbox/radio items,
//! shortcuts and keyboard navigation are shared with
//! [`DropdownMenu`](crate::components::DropdownMenu).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ui.group(|ui| {
//!     ui.label("Right-click me");
//! });
//! let ctx_menu = ContextMenu::new().id_salt("demo").show(ui, |m| {
//!     m.item(MenuItem::new("Back").shortcut("⌘["));
//!     m.separator();
//!     m.item(MenuItem::new("Save").shortcut("⌘S"));
//!     m.sub_menu("Share", |m| {
//!         m.item(MenuItem::new("Email"));
//!         m.item(MenuItem::new("Message"));
//!     });
//!     m.separator();
//!     m.item(MenuItem::new("Delete").destructive(true).icon(Icon::Trash));
//!     m.item(MenuItem::new("Disabled entry").disabled(true));
//! });
//! if ctx_menu.clicked("Delete") { /* … */ }
//!
//! // Controlled open state:
//! let mut open = false;
//! ContextMenu::controlled(&mut open).id_salt("c2").show(ui, |m| {
//!     m.item(MenuItem::new("Hello"));
//! });
//! ```

use crate::components::dropdown_menu::{
    apply_activation, handle_menu_escape, handle_menu_keys, handle_outside_click, read_menu_state,
    render_level, write_menu_state, IndicatorSide, LevelShared, MenuActivated, MenuBuilder,
    MenuSpec, MenuState, TriggerInfo, MENU_LABEL_HEIGHT, SUB_MENU_MIN_WIDTH,
};
use crate::theme::ShadcnTheme;
use egui::emath::RectAlign;
use egui::{Rect, Sense, Ui};

/// Content min width: nova `min-w-36` (144 px).
pub const CONTEXT_MENU_MIN_WIDTH: f32 = 144.0;

/// What [`ContextMenu::show`] returns.
#[derive(Clone, Debug)]
pub struct ContextMenuResponse {
    /// Whether the menu is open after this frame.
    pub open: bool,
    /// The item activated this frame (click or keyboard), if any.
    pub activated: Option<MenuActivated>,
    /// True when the menu was open at frame start and is closed now.
    pub closed: bool,
}

impl ContextMenuResponse {
    /// Whether the item with the given id (or label) was activated this frame.
    pub fn clicked(&self, id: &str) -> bool {
        self.activated.as_ref().is_some_and(|a| a.id == id)
    }
}

/// The shadcn context menu. See the [module docs](self) for an example.
///
/// Uncontrolled by default (open state persists per `id_salt`); use
/// [`ContextMenu::controlled`] to own the open state.
#[derive(Debug)]
pub struct ContextMenu<'a> {
    controlled: Option<&'a mut bool>,
    id_salt: Option<String>,
}

impl Default for ContextMenu<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ContextMenu<'a> {
    /// Creates an uncontrolled context menu.
    pub fn new() -> Self {
        Self {
            controlled: None,
            id_salt: None,
        }
    }

    /// Creates a context menu whose open state is owned by the caller.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            id_salt: None,
        }
    }

    /// Sets a stable id salt so open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Registers the trigger region (`ui.min_rect()` — everything laid out so
    /// far in this Ui), and renders the menu popup when open.
    ///
    /// Call it right after building the region's content, e.g. inside a
    /// `ui.group(...)` scope to scope the region to that group.
    pub fn show(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut MenuBuilder<'a>),
    ) -> ContextMenuResponse {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_context_menu"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_menu_state(&ctx, state_id);
        if let Some(open) = self.controlled.as_deref() {
            state.open = *open;
        } else {
            let exists = ctx.data_mut(|d| d.get_temp::<MenuState>(state_id).is_some());
            if !exists {
                write_menu_state(&ctx, state_id, state.clone());
            }
        }
        let open_at_start = state.open;

        // Trigger region: everything laid out so far in this Ui. Hover-only
        // sensing so the overlay never steals clicks from real widgets.
        let region = ui.min_rect();
        let region_id = state_id.with("region");
        let region_response = ui.interact(region, region_id, Sense::hover());
        let right_clicked =
            region_response.contains_pointer() && ui.input(|i| i.pointer.secondary_clicked());
        if right_clicked {
            let pos = ui
                .input(|i| i.pointer.interact_pos())
                .unwrap_or(region.center());
            state.open = true;
            state.highlight.clear();
            state.trigger = Some(TriggerInfo {
                rect: Rect::from_min_size(pos, egui::Vec2::ONE),
                layer_id: ui.layer_id(),
            });
        }

        // Build the item tree.
        let mut builder = MenuBuilder::new();
        content(&mut builder);
        let mut entries = builder.entries;

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();

        let mut nav_update: Option<Vec<usize>> = None;
        let mut activation: Option<Vec<usize>> = None;
        let mut popup_rects: Vec<Rect> = Vec::new();

        // Keyboard navigation while open (secondary to the pointer, like the
        // source's desktop-style menus).
        if state.open {
            handle_menu_keys(&ctx, &entries, &mut state, &mut activation);
            handle_menu_escape(&ctx, &mut state);
        }

        // Popup at the pointer position.
        let popup_id = state_id.with("popup");
        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        if state.open || was_open_last_frame {
            let spec = MenuSpec {
                min_width: CONTEXT_MENU_MIN_WIDTH,
                indicator: IndicatorSide::Right,
                label_muted: true,
                label_height: MENU_LABEL_HEIGHT,
                sub_min_width: SUB_MENU_MIN_WIDTH,
                sub_border_is_ring: true,
            };
            let mut shared = LevelShared {
                p,
                dark,
                root_open: state.open,
                highlight: state.highlight.clone(),
                nav_update: &mut nav_update,
                activation: &mut activation,
                popup_rects: &mut popup_rects,
                spec,
            };
            if let Some(trigger) = state.trigger {
                let _ = render_level(
                    ui,
                    &ctx,
                    &entries,
                    &[],
                    popup_id,
                    trigger.rect,
                    trigger.layer_id,
                    RectAlign::BOTTOM_START,
                    0.0,
                    false,
                    state.open,
                    None,
                    &mut shared,
                );
            }
        }

        // Dismissal: click (any button) outside every popup and the region.
        if state.open && handle_outside_click(&ctx, &popup_rects, &[region], was_open_last_frame) {
            state.open = false;
        }

        // Apply navigation updates from hover.
        if let Some(path) = nav_update.take() {
            state.highlight = path;
        }

        // Apply activation side effects (only while genuinely open).
        let mut activated = None;
        if state.open {
            if let Some(path) = activation.take() {
                let id = apply_activation(&mut entries, &path);
                if let Some(id) = id {
                    activated = Some(MenuActivated { id, path });
                }
                // Radix closes the menu after a select.
                state.open = false;
            }
        }
        let closed = open_at_start && !state.open;
        if closed {
            state.highlight.clear();
        }

        let open_now = state.open;
        if let Some(out) = self.controlled {
            *out = open_now;
        }
        write_menu_state(&ctx, state_id, state);

        ContextMenuResponse {
            open: open_now,
            activated,
            closed,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::dropdown_menu::{entry_disabled, CheckboxItem, MenuItem};
    use crate::theme::{Mode, Preset};
    use egui::{Id, Key, Pos2, Vec2};

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let menu = ContextMenu::controlled(&mut open).id_salt("ctx1");
        assert_eq!(menu.id_salt.as_deref(), Some("ctx1"));
        assert!(menu.controlled.is_some());
        let plain = ContextMenu::new();
        assert!(plain.controlled.is_none());
    }

    #[test]
    fn min_width_matches_nova() {
        assert_eq!(CONTEXT_MENU_MIN_WIDTH, 144.0, "min-w-36");
    }

    #[test]
    fn content_closure_builds_tree() {
        use crate::components::dropdown_menu::EntryKind;
        let mut b = MenuBuilder::new();
        b.item(MenuItem::new("Back"));
        b.separator();
        b.item(MenuItem::new("Delete").destructive(true));
        assert_eq!(b.entries.len(), 3);
        assert!(!entry_disabled(&b.entries[0]));
        assert!(matches!(b.entries[1].kind, EntryKind::Separator));
        match &b.entries[2].kind {
            EntryKind::Item { item } => assert!(item.destructive),
            other => panic!("expected item, got {other:?}"),
        }
    }

    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn run(&mut self, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("context_menu_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        /// Presses and releases a pointer button at `pos` over two frames.
        fn click(&mut self, pos: Pos2, button: egui::PointerButton, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.time += 0.1;
                let input = egui::RawInput {
                    time: Some(self.time),
                    events: vec![egui::Event::PointerButton {
                        pos,
                        button,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    ..Default::default()
                };
                let mut out = self.ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("context_menu_test_ui"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        fn key(&mut self, key: Key, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events: vec![egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                    physical_key: None,
                }],
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("context_menu_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> MenuState {
            let id = Id::unique("context_menu_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<MenuState>(id))
                .unwrap_or_default()
        }
    }

    fn layout(h: &mut Harness, salt: &str) -> Rect {
        let mut region = Rect::ZERO;
        h.run(|ui| {
            ui.label("Right-click target");
            region = ui.min_rect();
            ContextMenu::new().id_salt(salt).show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        h.run(|ui| {
            ui.label("Right-click target");
            region = ui.min_rect();
            ContextMenu::new().id_salt(salt).show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        region
    }

    #[test]
    fn right_click_opens_and_second_right_click_keeps_open() {
        let mut h = Harness::new();
        let region = layout(&mut h, "rc");
        assert!(!h.state("rc").open, "default is closed");

        h.click(region.center(), egui::PointerButton::Secondary, |ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("rc").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        assert!(h.state("rc").open, "right-click opens the menu");

        // Left click outside closes it.
        h.run(|ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("rc").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        h.click(
            egui::Pos2::new(550.0, 550.0),
            egui::PointerButton::Primary,
            |ui| {
                ui.label("Right-click target");
                ContextMenu::new().id_salt("rc").show(ui, |m| {
                    m.item(MenuItem::new("Only item"));
                });
            },
        );
        assert!(!h.state("rc").open, "click outside closes the menu");
    }

    #[test]
    fn right_click_outside_closes_open_menu() {
        let mut h = Harness::new();
        let region = layout(&mut h, "rc2");
        h.click(region.center(), egui::PointerButton::Secondary, |ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("rc2").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        assert!(h.state("rc2").open);
        h.run(|ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("rc2").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        h.click(
            egui::Pos2::new(560.0, 560.0),
            egui::PointerButton::Secondary,
            |ui| {
                ui.label("Right-click target");
                ContextMenu::new().id_salt("rc2").show(ui, |m| {
                    m.item(MenuItem::new("Only item"));
                });
            },
        );
        assert!(!h.state("rc2").open, "right-click outside closes the menu");
    }

    #[test]
    fn escape_closes_open_menu() {
        let mut h = Harness::new();
        let region = layout(&mut h, "esc");
        h.click(region.center(), egui::PointerButton::Secondary, |ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("esc").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        assert!(h.state("esc").open);
        h.run(|ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("esc").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        h.key(Key::Escape, |ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("esc").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        });
        assert!(!h.state("esc").open, "Escape must close the menu");
    }

    #[test]
    fn keyboard_navigates_items() {
        let mut h = Harness::new();
        let region = layout(&mut h, "kb");
        h.click(region.center(), egui::PointerButton::Secondary, |ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("kb").show(ui, |m| {
                m.item(MenuItem::new("First"));
                m.item(MenuItem::new("Second"));
            });
        });
        assert!(h.state("kb").open);
        h.run(|ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("kb").show(ui, |m| {
                m.item(MenuItem::new("First"));
                m.item(MenuItem::new("Second"));
            });
        });
        h.key(Key::ArrowDown, |ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("kb").show(ui, |m| {
                m.item(MenuItem::new("First"));
                m.item(MenuItem::new("Second"));
            });
        });
        assert_eq!(
            h.state("kb").highlight,
            vec![0],
            "ArrowDown highlights the first item"
        );
        h.key(Key::ArrowDown, |ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("kb").show(ui, |m| {
                m.item(MenuItem::new("First"));
                m.item(MenuItem::new("Second"));
            });
        });
        assert_eq!(h.state("kb").highlight, vec![1]);
    }

    #[test]
    fn controlled_mode_write_back() {
        let mut h = Harness::new();
        let mut open = false;
        h.run(|ui| {
            ui.label("Right-click target");
            ContextMenu::controlled(&mut open)
                .id_salt("c1")
                .show(ui, |m| {
                    m.item(MenuItem::new("Only item"));
                });
        });
        assert!(!open);

        let mut region = Rect::ZERO;
        h.run(|ui| {
            ui.label("Right-click target");
            region = ui.min_rect();
            ContextMenu::controlled(&mut open)
                .id_salt("c1")
                .show(ui, |m| {
                    m.item(MenuItem::new("Only item"));
                });
        });
        h.click(region.center(), egui::PointerButton::Secondary, |ui| {
            ui.label("Right-click target");
            ContextMenu::controlled(&mut open)
                .id_salt("c1")
                .show(ui, |m| {
                    m.item(MenuItem::new("Only item"));
                });
        });
        assert!(open, "right-click write-back reaches the caller");

        // The caller closes it from the outside.
        open = false;
        h.run(|ui| {
            ui.label("Right-click target");
            ContextMenu::controlled(&mut open)
                .id_salt("c1")
                .show(ui, |m| {
                    m.item(MenuItem::new("Only item"));
                });
        });
        assert!(!open);
    }

    #[test]
    fn activation_toggles_checkboxes_and_sets_radio_selection() {
        use crate::components::dropdown_menu::{resolve_action, EntryKind, MenuEntry};
        let mut checked = false;
        let selection: &'static mut Option<usize> = Box::leak(Box::new(None));
        let mut b = MenuBuilder::new();
        b.checkbox_item(CheckboxItem::new("Show grid", &mut checked).id("grid"));
        b.radio_group(selection, |g| g.radio_labels(["Small", "Large"]));
        let mut e = b.entries;
        let checked_state = |e: &[MenuEntry<'_>]| match &resolve_action(e, &[0]).unwrap().kind {
            EntryKind::Checkbox { checked, .. } => **checked,
            _ => panic!("expected checkbox"),
        };
        assert_eq!(apply_activation(&mut e, &[0]).as_deref(), Some("grid"));
        assert!(checked_state(&e));
        assert_eq!(apply_activation(&mut e, &[1]).as_deref(), Some("Small"));
        assert_eq!(*selection, Some(0));
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
