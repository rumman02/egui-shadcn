//! shadcn `menubar.tsx` — a horizontal bar of dropdown menus.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/menubar.tsx` (Radix Menubar)
//! with the nova recipe: bar `h-8 gap-0.5 rounded-lg border bg-background
//! p-[3px] shadow-xs`, triggers `rounded-sm px-1.5 py-[2px] text-sm
//! font-medium hover:bg-muted aria-expanded:bg-muted`, content `bg-popover
//! text-popover-foreground min-w-36 rounded-lg p-1 shadow-md ring-1
//! ring-foreground/10` (`align="start"`, `sideOffset = 8`, `alignOffset = -4`).
//! Nova puts the checkbox/radio indicator on the left (`pl-7`, indicator
//! `left-1.5`) and the section label in the foreground (`text-sm font-medium`).
//!
//! Items, sub-menus, checkbox/radio items, shortcuts and keyboard navigation
//! are shared with [`DropdownMenu`](crate::components::DropdownMenu).
//! Behavior mirrors Radix: click a trigger to open, hover another trigger
//! while one is open to switch, ArrowLeft/Right switch menus (or close/enter
//! sub-menus when one is open), Escape closes (deepest sub-menu first),
//! clicks outside close.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut autosave = true;
//! let mut theme = Some(0);
//!
//! let menubar = Menubar::new().id_salt("app").show(ui, |bar| {
//!     bar.menu("File", |m| {
//!         m.item(MenuItem::new("New Tab").shortcut("⌘T"));
//!         m.item(MenuItem::new("New Window").shortcut("⌘N"));
//!         m.item(MenuItem::new("New Incognito Window").disabled(true));
//!         m.separator();
//!         m.sub_menu("Share", |m| {
//!             m.item(MenuItem::new("Email link"));
//!             m.item(MenuItem::new("Messages"));
//!         });
//!     });
//!     bar.menu("View", |m| {
//!         m.checkbox_item(CheckboxItem::new("Always Show Bookmarks Bar", &mut autosave));
//!         m.radio_group(&mut theme, |g| g.radio_labels(["Light", "Dark"]));
//!     });
//! });
//! if menubar.activated("New Tab") { /* … */ }
//! ```

use crate::components::dropdown_menu::{
    apply_activation, handle_menu_escape, handle_menu_keys, handle_outside_click, render_level,
    IndicatorSide, LevelShared, MenuActivated, MenuBuilder, MenuEntry, MenuSpec, MenuState,
};
use crate::theme::ShadcnTheme;
use crate::utils::shadow;
use egui::emath::RectAlign;
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Key, Margin, Pos2, Rect, Response, Sense,
    Stroke, Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Geometry constants (nova recipe)
// ---------------------------------------------------------------------------

/// Bar height: `h-8` (32 px).
pub const MENUBAR_HEIGHT: f32 = 32.0;
/// Bar padding: `p-[3px]`.
pub const MENUBAR_PADDING: f32 = 3.0;
/// Gap between triggers: `gap-0.5` (2 px).
pub const MENUBAR_GAP: f32 = 2.0;
/// Trigger horizontal padding: `px-1.5` (6 px).
pub const MENUBAR_TRIGGER_PAD_X: f32 = 6.0;
/// Content side offset: `sideOffset = 8`.
pub const MENUBAR_SIDE_OFFSET: f32 = 8.0;
/// Content align offset: `alignOffset = -4`.
pub const MENUBAR_ALIGN_OFFSET: f32 = -4.0;
/// Content min width: nova `min-w-36` (144 px).
pub const MENUBAR_MIN_WIDTH: f32 = 144.0;

/// The height of a trigger inside the bar (bar height minus padding).
pub const MENUBAR_TRIGGER_HEIGHT: f32 = MENUBAR_HEIGHT - 2.0 * MENUBAR_PADDING;

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// One collected menu of the bar.
type MenuClosure<'m> = Box<dyn FnOnce(&mut MenuBuilder<'m>) + 'm>;

/// Collects the menus of a [`Menubar`] inside its `show` closure.
pub struct MenubarBuilder<'m> {
    menus: Vec<(String, MenuClosure<'m>)>,
}

impl Default for MenubarBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'m> MenubarBuilder<'m> {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self { menus: Vec::new() }
    }

    /// Appends one menu with its item list.
    pub fn menu(
        &mut self,
        label: impl Into<String>,
        build: impl FnOnce(&mut MenuBuilder<'m>) + 'm,
    ) {
        self.menus.push((label.into(), Box::new(build)));
    }
}

// ---------------------------------------------------------------------------
// Root state
// ---------------------------------------------------------------------------

/// Per-instance state: which menu is open, the highlight path inside it and
/// the menu playing its close animation (if any).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MenubarState {
    /// The currently open menu index, if any.
    pub open_index: Option<usize>,
    /// Highlight path inside the open menu (length > depth = open sub-menu).
    pub highlight: Vec<usize>,
    /// The index of the menu playing its close animation.
    pub fade_index: Option<usize>,
    /// The trigger a press started on this interaction: `(index, was_open)`
    /// captured at press time, so a release after a hover-switch keeps the
    /// switched-to menu open instead of toggling it closed.
    pub press: Option<(usize, bool)>,
    /// Per-trigger hover state from the previous frame; hover-switching only
    /// fires when the pointer *enters* a trigger (Radix behavior).
    pub hovered: Vec<bool>,
}

fn read_menubar_state(ctx: &Context, state_id: Id) -> MenubarState {
    ctx.data_mut(|d| d.get_temp::<MenubarState>(state_id).unwrap_or_default())
}

fn write_menubar_state(ctx: &Context, state_id: Id, state: MenubarState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

/// Next open menu index, wrapping at the ends (pure, unit-tested).
pub(crate) fn next_open(index: usize, delta: isize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (index as isize + delta).rem_euclid(count as isize) as usize
}

// ---------------------------------------------------------------------------
// Menubar (root)
// ---------------------------------------------------------------------------

/// What [`Menubar::show`] returns.
#[derive(Clone, Debug)]
pub struct MenubarResponse {
    /// The menu that is open after this frame, if any.
    pub open_index: Option<usize>,
    /// The item activated this frame (click or keyboard), if any.
    pub activated: Option<MenuActivated>,
    /// True when a menu was open at frame start and all are closed now.
    pub closed: bool,
}

impl MenubarResponse {
    /// Whether the item with the given id (or label) was activated this frame.
    pub fn activated(&self, id: &str) -> bool {
        self.activated.as_ref().is_some_and(|a| a.id == id)
    }

    /// Alias for [`MenubarResponse::activated`] (dropdown-style naming).
    pub fn clicked(&self, id: &str) -> bool {
        self.activated(id)
    }
}

/// The shadcn menubar. See the [module docs](self) for an example.
///
/// Uncontrolled by default; use [`Menubar::controlled`] to own which menu is
/// open.
#[derive(Debug)]
pub struct Menubar<'a> {
    controlled: Option<&'a mut Option<usize>>,
    id_salt: Option<String>,
}

impl Default for Menubar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Menubar<'a> {
    /// Creates an uncontrolled menubar.
    pub fn new() -> Self {
        Self {
            controlled: None,
            id_salt: None,
        }
    }

    /// Creates a menubar whose open menu index is owned by the caller.
    pub fn controlled(open_index: &'a mut Option<usize>) -> Self {
        Self {
            controlled: Some(open_index),
            id_salt: None,
        }
    }

    /// Sets a stable id salt so open state and focus persist per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the bar and the popup of the open menu, if any.
    pub fn show(self, ui: &mut Ui, build: impl FnOnce(&mut MenubarBuilder<'a>)) -> MenubarResponse {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_menubar"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_menubar_state(&ctx, state_id);
        if let Some(open_index) = self.controlled.as_deref() {
            state.open_index = *open_index;
        }
        let open_at_start = state.open_index.is_some();

        // Collect the menus and build every item tree eagerly, so activation
        // side effects (checkbox toggles, radio selection) can be applied
        // after rendering. The closures re-run every frame with fresh `&mut`
        // bindings, so their effects flow back to the caller.
        let mut builder = MenubarBuilder::new();
        build(&mut builder);
        let mut menus: Vec<(String, Vec<MenuEntry<'a>>)> = builder
            .menus
            .into_iter()
            .map(|(label, closure)| {
                let mut mb = MenuBuilder::new();
                closure(&mut mb);
                (label, mb.entries)
            })
            .collect();
        let count = menus.len();

        let mut nav_update: Option<Vec<usize>> = None;
        let mut activation: Option<Vec<usize>> = None;
        let mut popup_rects: Vec<Rect> = Vec::new();
        let mut trigger_rects: Vec<Rect> = Vec::new();

        // Keyboard handling for the open menu — once per frame (moving it
        // into the loop would run it once per open-menu transition).
        if let Some(i) = state.open_index {
            let entries = &menus[i].1;
            let before_len = state.highlight.len();
            let (left, right) = ctx.input(|i| {
                (
                    i.key_pressed(Key::ArrowLeft),
                    i.key_pressed(Key::ArrowRight),
                )
            });
            let mut tmp = MenuState {
                open: true,
                trigger: None,
                highlight: std::mem::take(&mut state.highlight),
            };
            handle_menu_keys(&ctx, entries, &mut tmp, &mut activation);
            state.highlight = tmp.highlight;
            // ArrowRight switches to the next menu unless it opened a
            // sub-menu; ArrowLeft switches to the previous one unless it
            // closed one.
            let old_index = state.open_index;
            if right && state.highlight.len() == before_len && count > 1 {
                state.open_index = Some(next_open(i, 1, count));
                state.highlight.clear();
            }
            if left && state.highlight.len() == before_len && before_len <= 1 && count > 1 {
                state.open_index = Some(next_open(i, -1, count));
                state.highlight.clear();
            }
            if state.open_index != old_index {
                state.fade_index = old_index;
            }

            // Escape: pop one sub-menu level, or close the whole menu.
            let hl_before = state.highlight.len();
            let mut tmp = MenuState {
                open: true,
                trigger: None,
                highlight: std::mem::take(&mut state.highlight),
            };
            let esc = handle_menu_escape(&ctx, &mut tmp);
            state.highlight = tmp.highlight;
            if esc && state.highlight.len() == hl_before {
                state.fade_index = state.open_index;
                state.open_index = None;
                state.highlight.clear();
            }
        }

        // The bar: `h-8 gap-0.5 rounded-lg border bg-background p-[3px] shadow-xs`.
        let radius = ShadcnTheme::of_ui(ui).radius;
        egui::Frame::new()
            .fill(ShadcnTheme::of_ui(ui).palette.background)
            .stroke(Stroke::new(1.0, ShadcnTheme::of_ui(ui).palette.border))
            .corner_radius(CornerRadius::same(radius.lg_u8()))
            .inner_margin(Margin::same(MENUBAR_PADDING as i8))
            .shadow(shadow::shadow_xs(ShadcnTheme::of_ui(ui).mode.is_dark()))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = MENUBAR_GAP;

                    for (i, (label, entries)) in menus.iter().enumerate() {
                        let is_open = state.open_index == Some(i);
                        let is_fading = state.fade_index == Some(i) && !is_open;

                        // Trigger: `rounded-sm px-1.5 py-[2px] text-sm font-medium
                        // hover:bg-muted aria-expanded:bg-muted focus:bg-accent`.
                        let trigger =
                            paint_trigger(ui, state_id.with(("trigger", i)), label, is_open);

                        trigger_rects.push(trigger.rect);

                        // Capture the open state at press time, so a release
                        // after a hover-switch keeps the menu open.
                        let primary_pressed = ui.input(|input| input.pointer.primary_pressed());
                        if primary_pressed && trigger.contains_pointer() && state.press.is_none() {
                            state.press = Some((i, is_open));
                        }

                        if trigger.clicked() {
                            let was_open = state
                                .press
                                .is_some_and(|(pi, was_open)| pi == i && was_open);
                            if was_open {
                                state.fade_index = Some(i);
                                state.open_index = None;
                                state.highlight.clear();
                            } else {
                                state.open_index = Some(i);
                                state.highlight.clear();
                                state.fade_index = None;
                            }
                        }
                        // Hover-with-open switches menus on pointer *enter*
                        // (Radix behavior; continuous hover would undo
                        // keyboard menu switching).
                        let hovered_now = trigger.hovered();
                        let entered = hovered_now && state.hovered.get(i).copied() != Some(true);
                        if state.open_index.is_some() && !is_open && entered {
                            state.open_index = Some(i);
                            state.highlight.clear();
                            state.fade_index = None;
                        }

                        let popup_id = state_id.with(("menupopup", i));
                        if state.open_index == Some(i) {
                            render_menubar_level(
                                ui,
                                &ctx,
                                entries,
                                popup_id,
                                trigger.rect,
                                true,
                                &mut nav_update,
                                &mut activation,
                                &mut popup_rects,
                                &mut state,
                            );
                        } else if is_fading {
                            // Close animation: render once more with open=false.
                            let fade_done = render_menubar_level(
                                ui,
                                &ctx,
                                entries,
                                popup_id,
                                trigger.rect,
                                false,
                                &mut nav_update,
                                &mut activation,
                                &mut popup_rects,
                                &mut state,
                            );
                            if fade_done {
                                state.fade_index = None;
                            }
                        }
                    }
                });
            });

        // Record hover state for next frame's enter detection.
        state.hovered = trigger_rects
            .iter()
            .map(|rect| {
                ctx.input(|input| {
                    input
                        .pointer
                        .latest_pos()
                        .is_some_and(|pos| rect.contains(pos))
                })
            })
            .collect();

        // The pointer interaction ended; drop the press capture.
        if ctx.input(|i| i.pointer.any_click()) {
            state.press = None;
        }

        // Dismissal: click outside every popup and trigger.
        if state.open_index.is_some()
            && handle_outside_click(&ctx, &popup_rects, &trigger_rects, true)
        {
            state.fade_index = state.open_index;
            state.open_index = None;
            state.highlight.clear();
        }

        // Apply navigation updates from hover.
        if let Some(path) = nav_update.take() {
            state.highlight = path;
        }

        // Apply activation side effects (checkbox toggle, radio selection).
        let mut activated = None;
        if let (Some(path), Some(i)) = (activation.take(), state.open_index) {
            let id = apply_activation(&mut menus[i].1, &path);
            if let Some(id) = id {
                activated = Some(MenuActivated { id, path });
            }
            // Radix closes the menu after a select.
            state.fade_index = state.open_index;
            state.open_index = None;
            state.highlight.clear();
        }

        let closed = open_at_start && state.open_index.is_none();
        if closed {
            state.highlight.clear();
        }

        let open_index = state.open_index;
        if let Some(out) = self.controlled {
            *out = open_index;
        }
        write_menubar_state(&ctx, state_id, state);

        MenubarResponse {
            open_index,
            activated,
            closed,
        }
    }
}

/// Renders one menu's popup. Returns `true` when a closing popup finished
/// its fade-out.
#[allow(clippy::too_many_arguments)]
fn render_menubar_level(
    ui: &mut Ui,
    ctx: &Context,
    entries: &[MenuEntry<'_>],
    popup_id: Id,
    trigger_rect: Rect,
    open: bool,
    nav_update: &mut Option<Vec<usize>>,
    activation: &mut Option<Vec<usize>>,
    popup_rects: &mut Vec<Rect>,
    state: &mut MenubarState,
) -> bool {
    let theme = ShadcnTheme::of_ui(ui);
    let spec = MenuSpec {
        min_width: MENUBAR_MIN_WIDTH,
        indicator: IndicatorSide::Left,
        label_muted: false,
    };
    let mut shared = LevelShared {
        p: theme.palette,
        dark: theme.mode.is_dark(),
        root_open: open,
        highlight: state.highlight.clone(),
        nav_update,
        activation,
        popup_rects,
        spec,
    };

    // `align="start"`, `alignOffset = -4`, `sideOffset = 8`.
    let anchor = trigger_rect.translate(egui::vec2(MENUBAR_ALIGN_OFFSET, 0.0));
    let response = render_level(
        ui,
        ctx,
        entries,
        &[],
        popup_id,
        anchor,
        ui.layer_id(),
        RectAlign::BOTTOM_START,
        MENUBAR_SIDE_OFFSET,
        false,
        open,
        &mut shared,
    );

    // Done fading when the close animation finished.
    if !open {
        let progress = crate::utils::animation::overlay_progress(ctx, popup_id.with("anim"), false);
        return progress.alpha <= 0.002;
    }
    response.is_some()
}

/// One menubar trigger button: `rounded-sm px-1.5 py-[2px] text-sm
/// font-medium` with `hover:bg-muted`, `aria-expanded:bg-muted` and
/// `focus:bg-accent focus:text-accent-foreground`.
fn paint_trigger(ui: &mut Ui, id: Id, label: &str, open: bool) -> Response {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let radius = theme.radius;
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x;
    let width = text_width + 2.0 * MENUBAR_TRIGGER_PAD_X;
    let (_, rect) = ui.allocate_space(Vec2::new(width, MENUBAR_TRIGGER_HEIGHT));
    let response = ui.interact(rect, id, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let fill = if open || response.hovered() {
            p.muted
        } else {
            Color32::TRANSPARENT
        };
        if fill != Color32::TRANSPARENT {
            painter.rect_filled(rect, CornerRadius::same(radius.sm_u8()), fill);
        }
        let text_color = if response.has_focus() {
            p.accent_foreground
        } else {
            p.foreground
        };
        painter.text(
            Pos2::new(rect.center().x, rect.center().y),
            Align2::CENTER_CENTER,
            label,
            font,
            text_color,
        );
        if response.has_focus() {
            crate::utils::focus::paint_focus_ring(ui, &response, radius.sm(), true);
        }
    }
    response
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::dropdown_menu::{CheckboxItem, MenuItem};
    use crate::theme::{Mode, Preset};
    use egui::Key;

    #[test]
    fn next_open_wraps() {
        assert_eq!(next_open(0, 1, 3), 1);
        assert_eq!(next_open(2, 1, 3), 0, "wraps forward");
        assert_eq!(next_open(0, -1, 3), 2, "wraps backward");
        assert_eq!(next_open(1, -1, 3), 0);
        assert_eq!(next_open(0, 1, 0), 0, "no menus");
    }

    #[test]
    fn geometry_constants_match_source() {
        assert_eq!(MENUBAR_HEIGHT, 32.0, "h-8");
        assert_eq!(MENUBAR_PADDING, 3.0, "p-[3px]");
        assert_eq!(MENUBAR_GAP, 2.0, "gap-0.5");
        assert_eq!(MENUBAR_TRIGGER_PAD_X, 6.0, "px-1.5");
        assert_eq!(MENUBAR_SIDE_OFFSET, 8.0, "sideOffset=8");
        assert_eq!(MENUBAR_ALIGN_OFFSET, -4.0, "alignOffset=-4");
        assert_eq!(MENUBAR_MIN_WIDTH, 144.0, "min-w-36");
        assert_eq!(MENUBAR_TRIGGER_HEIGHT, 26.0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = None;
        let bar = Menubar::controlled(&mut open).id_salt("mb1");
        assert_eq!(bar.id_salt.as_deref(), Some("mb1"));
        assert!(bar.controlled.is_some());
        let plain = Menubar::new();
        assert!(plain.controlled.is_none());

        let mut checked = false;
        let mut b = MenubarBuilder::new();
        b.menu("File", |m| m.item(MenuItem::new("New Tab")));
        b.menu("Edit", |m| {
            m.checkbox_item(CheckboxItem::new("Autosave", &mut checked));
        });
        assert_eq!(b.menus.len(), 2);
        assert_eq!(b.menus[0].0, "File");
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

        fn run_with(&mut self, events: Vec<egui::Event>, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("menubar_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn run(&mut self, f: impl FnMut(&mut Ui)) {
            self.run_with(vec![], f);
        }

        fn click(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.run_with(
                    vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    &mut f,
                );
            }
        }

        fn key(&mut self, key: Key, mut f: impl FnMut(&mut Ui)) {
            self.run_with(
                vec![egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                    physical_key: None,
                }],
                &mut f,
            );
        }

        fn state(&self, salt: &str) -> MenubarState {
            let id = Id::unique("menubar_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<MenubarState>(id))
                .unwrap_or_default()
        }

        fn trigger_rects(&self, salt: &str) -> Vec<Rect> {
            let state_id = Id::unique("menubar_test_ui").with(salt);
            (0..2)
                .filter_map(|i| self.ctx.read_response(state_id.with(("trigger", i))))
                .map(|r| r.rect)
                .collect()
        }
    }

    fn draw(h: &mut Harness, salt: &str) -> Vec<Rect> {
        h.run(|ui| {
            Menubar::new().id_salt(salt).show(ui, |bar| {
                bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
            });
        });
        h.trigger_rects(salt)
    }

    fn draw_click(h: &mut Harness, salt: &str, pos: Pos2) -> Vec<Rect> {
        h.click(pos, |ui| {
            Menubar::new().id_salt(salt).show(ui, |bar| {
                bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
            });
        });
        h.trigger_rects(salt)
    }

    fn draw_key(h: &mut Harness, salt: &str, key: Key) {
        h.key(key, |ui| {
            Menubar::new().id_salt(salt).show(ui, |bar| {
                bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
            });
        });
    }

    #[test]
    fn trigger_click_opens_and_toggles() {
        let mut h = Harness::new();
        let rects = draw(&mut h, "mb1");
        assert_eq!(rects.len(), 2, "two triggers");
        assert!(h.state("mb1").open_index.is_none());

        let rects = draw_click(&mut h, "mb1", rects[0].center());
        assert_eq!(h.state("mb1").open_index, Some(0), "clicking File opens it");

        let _ = draw_click(&mut h, "mb1", rects[0].center());
        assert!(
            h.state("mb1").open_index.is_none(),
            "clicking again closes it"
        );
    }

    #[test]
    fn clicking_another_trigger_switches_menus() {
        let mut h = Harness::new();
        let rects = draw(&mut h, "mb2");
        let rects = draw_click(&mut h, "mb2", rects[0].center());
        assert_eq!(h.state("mb2").open_index, Some(0));

        let _ = draw_click(&mut h, "mb2", rects[1].center());
        assert_eq!(
            h.state("mb2").open_index,
            Some(1),
            "clicking Edit while File is open switches"
        );
    }

    #[test]
    fn click_outside_closes_open_menu() {
        let mut h = Harness::new();
        let rects = draw(&mut h, "mb3");
        let _ = draw_click(&mut h, "mb3", rects[0].center());
        assert_eq!(h.state("mb3").open_index, Some(0));

        h.run(|ui| {
            Menubar::new().id_salt("mb3").show(ui, |bar| {
                bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
            });
        });
        h.click(egui::Pos2::new(550.0, 550.0), |ui| {
            Menubar::new().id_salt("mb3").show(ui, |bar| {
                bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
            });
        });
        assert!(
            h.state("mb3").open_index.is_none(),
            "click outside closes it"
        );
    }

    #[test]
    fn escape_closes_open_menu() {
        let mut h = Harness::new();
        let rects = draw(&mut h, "mb4");
        let _ = draw_click(&mut h, "mb4", rects[0].center());
        assert_eq!(h.state("mb4").open_index, Some(0));

        h.run(|ui| {
            Menubar::new().id_salt("mb4").show(ui, |bar| {
                bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
            });
        });
        draw_key(&mut h, "mb4", Key::Escape);
        assert!(h.state("mb4").open_index.is_none(), "Escape closes it");
    }

    #[test]
    fn arrow_right_switches_menus_while_open() {
        let mut h = Harness::new();
        let rects = draw(&mut h, "mb5");
        let _ = draw_click(&mut h, "mb5", rects[0].center());
        assert_eq!(h.state("mb5").open_index, Some(0));

        h.run(|ui| {
            Menubar::new().id_salt("mb5").show(ui, |bar| {
                bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
            });
        });
        draw_key(&mut h, "mb5", Key::ArrowRight);
        assert_eq!(
            h.state("mb5").open_index,
            Some(1),
            "ArrowRight switches to Edit"
        );
    }

    #[test]
    fn controlled_mode_write_back() {
        let mut h = Harness::new();
        let mut open_index: Option<usize> = None;
        let mut rects = Vec::new();
        for _ in 0..2 {
            h.run(|ui| {
                Menubar::controlled(&mut open_index)
                    .id_salt("mb6")
                    .show(ui, |bar| {
                        bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                        bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
                    });
            });
            rects = h.trigger_rects("mb6");
        }

        let _ = draw_click_controlled(&mut h, &mut open_index, rects[0].center());
        assert_eq!(open_index, Some(0), "click write-back reaches the caller");

        // The caller closes it from the outside.
        open_index = None;
        h.run(|ui| {
            Menubar::controlled(&mut open_index)
                .id_salt("mb6")
                .show(ui, |bar| {
                    bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                    bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
                });
        });
        assert_eq!(open_index, None);
    }

    fn draw_click_controlled(
        h: &mut Harness,
        open_index: &mut Option<usize>,
        pos: Pos2,
    ) -> Vec<Rect> {
        h.click(pos, |ui| {
            Menubar::controlled(open_index)
                .id_salt("mb6")
                .show(ui, |bar| {
                    bar.menu("File", |m| m.item(MenuItem::new("New Tab")));
                    bar.menu("Edit", |m| m.item(MenuItem::new("Undo")));
                });
        });
        h.trigger_rects("mb6")
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
