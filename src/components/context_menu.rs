//! shadcn `context-menu.tsx` — the right-click menu.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/context-menu.tsx` (Radix
//! ContextMenu) with the nova recipe: content `bg-popover text-popover-foreground
//! min-w-36 rounded-lg p-1 shadow-md ring-1 ring-foreground/10`, the same
//! item rows as the dropdown-menu (nova puts the checkbox/radio indicator on
//! the right: `pr-8`, indicator `right-2`), labels `text-muted-foreground
//! text-xs`, destructive items `text-destructive` with
//! `focus:bg-destructive/10` (dark: `/20`). The nova sub-content differs
//! from the root: `min-w-32` with a `border`-token stroke and `shadow-lg`
//! (`.cn-context-menu-sub-content`).
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
    render_level, write_menu_state, menu_label_height, IndicatorSide, LevelShared, MenuActivated,
    MenuBuilder, MenuSpec, MenuState, TriggerInfo,
};
use crate::theme::{ShadcnTheme, Style};
use egui::emath::RectAlign;
use egui::{Id, Rect, Sense, Ui};

/// Content min width: nova `min-w-36` (144 px).
pub const CONTEXT_MENU_MIN_WIDTH: f32 = 144.0;

/// Sub-menu content min width: nova `min-w-32` (128 px,
/// `.cn-context-menu-sub-content`).
pub const CONTEXT_MENU_SUB_MIN_WIDTH: f32 = 128.0;

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
    /// Radix closes the menu on select by default; `false` keeps it open
    /// (the shared renderer's opt-out, exposed here for parity with
    /// [`DropdownMenu`](crate::components::DropdownMenu)).
    close_on_select: bool,
}

/// The shared [`MenuSpec`] for this component: nova content 144 px
/// (`min-w-36`, §3.1), sub-content 128 px with a `border`-token stroke —
/// **not** the dropdown-menu ring (§3.4: `.cn-context-menu-sub-content`
/// restates `border`, no `ring-1`), label height 24 px nova / 32 px base
/// (§3.2, OPEN-6).
fn menu_spec(theme: &ShadcnTheme) -> MenuSpec {
    MenuSpec {
        min_width: CONTEXT_MENU_MIN_WIDTH,
        indicator: IndicatorSide::Right,
        label_muted: true,
        label_height: menu_label_height(theme.style == Style::Nova),
        sub_min_width: CONTEXT_MENU_SUB_MIN_WIDTH,
        // Nova sub-content strokes with the `border` token
        // (`.cn-context-menu-sub-content`: `border … shadow-lg`),
        // while the root content stays `ring-foreground/10`.
        sub_border_is_ring: false,
    }
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
            close_on_select: true,
        }
    }

    /// Creates a context menu whose open state is owned by the caller.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            id_salt: None,
            close_on_select: true,
        }
    }

    /// Sets a stable id salt so open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Whether selecting a row closes the menu (Radix default: `true`).
    /// Pass `false` to keep the menu open after activation, e.g. for
    /// multi-select menus — the same opt-out
    /// [`DropdownMenu`](crate::components::DropdownMenu) exposes.
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    /// Registers the trigger region (`ui.min_rect()` — everything laid out so
    /// far in this Ui), and renders the menu popup when open.
    ///
    /// Call it right after building the region's content, e.g. inside a
    /// `ui.group(...)` scope to scope the region to that group. The popup
    /// anchor is refreshed from the region every frame, so a controlled
    /// open renders without a prior right-click and the popup follows a
    /// moving region; a right-click open keeps its 1×1 pointer anchor.
    /// On close (except Tab-away) focus returns to the widget that had it
    /// when the menu opened — the wrapped content's focus id.
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
        // Last frame's open state, before the controlled sync below — the
        // open transition (N2 capture) is judged against this, not the
        // post-sync value.
        let was_open = state.open;
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
        // Whether the live anchor is a 1×1 pointer rect from a right-click
        // (N1): while it is, the per-frame region refresh below is skipped
        // so the popup stays at the pointer; it is reset when the menu
        // closes, so the next open re-anchors from the region.
        let mut pointer_anchor = ctx
            .data_mut(|d| d.get_temp::<bool>(state_id.with("pointer_anchor")))
            .unwrap_or(false);
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
            pointer_anchor = true;
        }
        // N1: refresh the popup anchor from the trigger region every frame,
        // so a controlled open renders without a prior click and the popup
        // follows a moving region (as select does). Right-click opens keep
        // their pointer anchor (see `pointer_anchor` above).
        if !pointer_anchor {
            state.trigger = Some(TriggerInfo {
                rect: region,
                layer_id: ui.layer_id(),
            });
        }
        // N2: remember which widget had focus when the menu opened, so
        // focus can return to it on close. The region itself is hover-only
        // (`Sense::hover()`) and cannot hold focus in egui — `create_widget`
        // calls `ignore_focus` for every non-focusable widget each frame —
        // so the return target is the pre-open focused widget, not the
        // region id; when nothing had focus, nothing is restored (no fake
        // return).
        if !was_open && state.open {
            let before = ctx.memory(|m| m.focused());
            ctx.data_mut(|d| d.insert_temp(state_id.with("focus_before_open"), before));
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
        // source's desktop-style menus). Tab closes (N7); its return is
        // captured so the focus return below can skip the Tab-away case.
        let mut tab_closed = false;
        if state.open {
            tab_closed = handle_menu_keys(&ctx, &entries, &mut state, &mut activation);
            handle_menu_escape(&ctx, &mut state);
        }

        // Popup at the pointer position.
        let popup_id = state_id.with("popup");
        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        if state.open || was_open_last_frame {
            let spec = menu_spec(&theme);
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
                if self.close_on_select {
                    state.open = false;
                }
            }
        }
        let closed = open_at_start && !state.open;
        if closed {
            state.highlight.clear();
        }
        // N2: focus returns to the widget that had it when the menu opened
        // (the wrapped content's focus id), on every close except Tab-away
        // — focus moves on by design there (N7). Runs at close-decision
        // time so it also covers the close-fade path. The stored pre-open
        // focus id is cleared once the menu has closed, whatever the path.
        if closed && !tab_closed {
            let before = ctx
                .data_mut(|d| d.get_temp::<Option<Id>>(state_id.with("focus_before_open")))
                .flatten();
            if let Some(focus_id) = before {
                ctx.memory_mut(|m| m.request_focus(focus_id));
            }
        }
        if closed {
            ctx.data_mut(|d| d.insert_temp(state_id.with("focus_before_open"), None::<Id>));
        }

        let open_now = state.open;
        if let Some(out) = self.controlled {
            *out = open_now;
        }
        // The pointer anchor only lives while a right-click open is live;
        // once the menu closes the next open re-anchors from the region (N1).
        if !open_now {
            pointer_anchor = false;
        }
        ctx.data_mut(|d| d.insert_temp(state_id.with("pointer_anchor"), pointer_anchor));
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
    use crate::theme::{Mode, Preset, Style};
    use egui::{Id, Key, Pos2, Vec2};

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let menu = ContextMenu::controlled(&mut open).id_salt("ctx1");
        assert_eq!(menu.id_salt.as_deref(), Some("ctx1"));
        assert!(menu.controlled.is_some());
        assert!(menu.close_on_select, "default is close-on-select (Radix)");
        let plain = ContextMenu::new();
        assert!(plain.controlled.is_none());
        assert!(plain.close_on_select);
    }

    #[test]
    fn spec_matches_nova_recipes() {
        // §3.1/§3.4: nova content 144 px, sub-content 128 px with a
        // `border` stroke (not the dropdown ring); §3.2: label height
        // 24 px nova / 32 px base (OPEN-6).
        let nova = Preset::default().build(Mode::Light);
        let spec = menu_spec(&nova);
        assert_eq!(spec.min_width, 144.0, "min-w-36");
        assert_eq!(spec.sub_min_width, 128.0, "sub-content min-w-32");
        assert!(
            !spec.sub_border_is_ring,
            "nova sub-content strokes `border`, not ring (§3.4)"
        );
        assert_eq!(spec.label_height, 24.0, "nova label 24 px");
        assert!(matches!(spec.indicator, IndicatorSide::Right));
        assert!(spec.label_muted);
        let base = Preset::default().with_style(Style::Vega).build(Mode::Light);
        assert_eq!(menu_spec(&base).label_height, 32.0, "base label 32 px");
    }

    #[test]
    fn controlled_open_renders_without_click() {
        // N1: a controlled open must render at the trigger/content rect
        // even though the trigger was never right-clicked, and the anchor
        // must follow a moving region.
        let mut h = Harness::new();
        let mut open = true;
        let mut region = Rect::ZERO;
        let mut show = |ui: &mut Ui| {
            ui.label("Right-click target");
            region = ui.min_rect();
            ContextMenu::controlled(&mut open)
                .id_salt("n1")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        };
        h.run(|ui| show(ui));
        // One settle frame (a popup's first pass is sizing-only).
        h.run(|ui| show(ui));
        assert!(h.state("n1").open, "controlled open is honoured");
        assert_eq!(
            h.state("n1").trigger.map(|t| t.rect),
            Some(region),
            "anchor captured without a click"
        );
        let popup_id = Id::unique("context_menu_test_ui").with("n1").with("popup");
        assert!(
            h.ctx.read_response(popup_id).is_some(),
            "popup response exists: the menu rendered"
        );

        // Grow the region by laying out a label above the target: the
        // region (and therefore the anchor) must follow it. Note
        // `add_space` cannot move the region here — the space allocation
        // itself is part of `ui.min_rect()`, so its `min.y` never moves.
        let before = region;
        h.run(|ui| {
            ui.label("Above");
            ui.label("Right-click target");
            region = ui.min_rect();
            ContextMenu::controlled(&mut open)
                .id_salt("n1")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(
            region.max.y > before.max.y,
            "region grew by the added label"
        );
        assert_eq!(
            h.state("n1").trigger.map(|t| t.rect),
            Some(region),
            "anchor follows the moving region"
        );
    }

    /// A show closure with a second focusable widget ("Other") laid out
    /// after the menu, so focus can sit somewhere real before the close
    /// (egui drops focus on an id no widget shows).
    fn show_with_other<'a>(
        salt: &'a str,
        other_id: &'a std::cell::Cell<Id>,
    ) -> impl FnMut(&mut Ui) + Copy + 'a {
        move |ui: &mut Ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt(salt).show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
            other_id.set(ui.button("Other").id);
        }
    }

    #[test]
    fn escape_close_returns_focus_to_widget_focused_at_open() {
        // N2: on Escape close, focus returns to the widget that had it
        // when the menu opened (the wrapped content's focus id) — and the
        // return survives a settle frame, i.e. it targets a real focusable
        // widget, not the hover-only region id.
        let mut h = Harness::new();
        let region = layout(&mut h, "n2esc");
        let other_id = std::cell::Cell::new(Id::NULL);
        let show = show_with_other("n2esc", &other_id);
        h.run(show);
        h.ctx.memory_mut(|m| m.request_focus(other_id.get()));
        h.run(show);
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "focus sits on the other button before the open"
        );

        h.click(region.center(), egui::PointerButton::Secondary, show);
        assert!(h.state("n2esc").open, "right-click opens the menu");

        h.key(Key::Escape, show);
        assert!(!h.state("n2esc").open, "Escape must close the menu");
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "focus returns to the pre-open widget on close"
        );
        // Durability: the return must still hold after a settle frame.
        h.run(show);
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "focus return survives a settle frame"
        );
    }

    #[test]
    fn activation_close_returns_focus_to_widget_focused_at_open() {
        // N2: on item-activation close, focus returns to the widget that
        // had it when the menu opened.
        let mut h = Harness::new();
        let region = layout(&mut h, "n2act");
        let other_id = std::cell::Cell::new(Id::NULL);
        let show = show_with_other("n2act", &other_id);
        h.run(show);
        h.ctx.memory_mut(|m| m.request_focus(other_id.get()));
        h.run(show);
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "focus sits on the other button before the open"
        );

        h.click(region.center(), egui::PointerButton::Secondary, show);
        assert!(h.state("n2act").open, "right-click opens the menu");

        h.key(Key::ArrowDown, show);
        assert_eq!(h.state("n2act").highlight, vec![0]);
        h.key(Key::Enter, show);
        assert!(!h.state("n2act").open, "activation closes the menu");
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "focus returns to the pre-open widget on activation close"
        );
        h.run(show);
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "focus return survives a settle frame"
        );
    }

    #[test]
    fn close_with_nothing_focused_restores_nothing() {
        // N2 limit: when no widget had focus when the menu opened, the
        // close restores nothing — the hover-only region id is not a real
        // focus target (egui would drop it one frame later), so no fake
        // return is issued.
        let mut h = Harness::new();
        let region = layout(&mut h, "n2none");
        assert!(h.ctx.memory(|m| m.focused()).is_none());
        let show = |ui: &mut Ui| {
            ui.label("Right-click target");
            ContextMenu::new().id_salt("n2none").show(ui, |m| {
                m.item(MenuItem::new("Only item"));
            });
        };
        h.click(region.center(), egui::PointerButton::Secondary, show);
        assert!(h.state("n2none").open, "right-click opens the menu");
        h.key(Key::Escape, show);
        assert!(!h.state("n2none").open, "Escape must close the menu");
        assert!(
            h.ctx.memory(|m| m.focused()).is_none(),
            "no focus is fabricated on close"
        );
        h.run(show);
        assert!(
            h.ctx.memory(|m| m.focused()).is_none(),
            "focus stays unset after a settle frame"
        );
    }

    #[test]
    fn tab_closes_open_menu_without_returning_focus() {
        // N7: Tab closes an open menu; the close must not force focus back
        // to the pre-open widget (focus moves on by design), and a later
        // frame must not re-open it.
        let mut h = Harness::new();
        let region = layout(&mut h, "n7");
        let other_id = std::cell::Cell::new(Id::NULL);
        let show = show_with_other("n7", &other_id);
        h.run(show);
        h.ctx.memory_mut(|m| m.request_focus(other_id.get()));
        h.run(show);
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "focus sits on the other button before the open"
        );

        h.click(region.center(), egui::PointerButton::Secondary, show);
        assert!(h.state("n7").open, "right-click opens the menu");

        h.key(Key::Tab, show);
        assert!(!h.state("n7").open, "Tab must close the menu");
        assert!(
            h.state("n7").highlight.is_empty(),
            "Tab close clears the highlight"
        );
        assert_ne!(
            h.ctx.memory(|m| m.focused()),
            Some(other_id.get()),
            "Tab-away must not return focus to the pre-open widget"
        );

        // A later frame (after the focus move) must not re-open it.
        h.run(show);
        assert!(!h.state("n7").open, "menu stays closed after Tab-away");
    }

    #[test]
    fn close_on_select_opt_out_keeps_menu_open() {
        // §8 API decision: the shared renderer's `close_on_select(false)`
        // opt-out is exposed for parity with DropdownMenu; the default
        // stays close-on-select (Radix).
        let mut h = Harness::new();
        let region = layout(&mut h, "cos");
        let show = |ui: &mut Ui| {
            ui.label("Right-click target");
            ContextMenu::new()
                .id_salt("cos")
                .close_on_select(false)
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        };
        h.click(region.center(), egui::PointerButton::Secondary, show);
        assert!(h.state("cos").open, "right-click opens the menu");
        h.run(show);
        h.key(Key::ArrowDown, show);
        assert_eq!(h.state("cos").highlight, vec![0]);
        h.key(Key::Enter, show);
        assert!(
            h.state("cos").open,
            "close_on_select(false) keeps the menu open after activation"
        );
    }

    #[test]
    fn min_width_matches_nova() {
        assert_eq!(CONTEXT_MENU_MIN_WIDTH, 144.0, "min-w-36");
        assert_eq!(CONTEXT_MENU_SUB_MIN_WIDTH, 128.0, "sub-content min-w-32");
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
