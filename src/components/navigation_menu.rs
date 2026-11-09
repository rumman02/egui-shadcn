//! shadcn `navigation-menu.tsx` — a site-navigation bar with panels.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/navigation-menu.tsx`
//! (Radix NavigationMenu): triggers are `h-9 px-4 py-2 rounded-md text-sm
//! font-medium bg-background hover:bg-accent hover:text-accent-foreground
//! focus:bg-accent focus:text-accent-foreground
//! data-[state=open]:bg-accent/50 data-[state=open]:text-accent-foreground`
//! (nova: `rounded-lg px-2.5 py-1.5`, muted hovers) with a `size-3` chevron
//! that rotates when open, an animated viewport below the bar
//! (`mt-1.5 rounded-md border bg-popover shadow`, nova `rounded-lg ring-1
//! ring-foreground/10`) plus the little `bg-border` diamond indicator under
//! the open trigger, and links styled `p-2 rounded-sm hover:bg-accent
//! data-[active=true]:bg-accent/50` with a `text-muted-foreground`
//! description.
//!
//! A `viewport(false)` menu renders each panel next to its trigger instead
//! of in the shared viewport (the source's inline mode). An `orientation`
//! of [`NavOrientation::Vertical`] stacks the triggers and floats panels to
//! the side.
//!
//! Opening: click a trigger or hover it (hover also switches while open);
//! closing: click the trigger again, Escape, or any click outside. Content
//! panels are closures, mirroring the JSX composition.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let res = NavigationMenu::new().id_salt("nav").show(ui, |nav| {
//!     nav.link("Introduction");
//!     nav.trigger("Learn", |c| {
//!         c.link("Library", "Structured access to collection data.");
//!         c.link("Components", "Building blocks for the web.");
//!     });
//!     nav.trigger("Examples", |c| {
//!         c.link("Email", "Send and receive email.");
//!     });
//!     nav.trigger_disabled("Enterprise");
//! });
//! if let Some(title) = res.link_clicked { /* navigate */ }
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use crate::utils::animation::overlay_progress;
use crate::utils::shadow;
use egui::emath::RectAlign;
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Key, Margin, Pos2, Rect, Response, Sense,
    Stroke, Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Geometry constants
// ---------------------------------------------------------------------------

/// Trigger height: `h-9` (36 px).
pub const NAV_TRIGGER_HEIGHT: f32 = crate::theme::tokens::spacing::CONTROL;
/// Trigger horizontal padding: `px-4` (base) / `px-2.5` (nova) — use 16 px.
pub const NAV_TRIGGER_PAD_X: f32 = 16.0;
/// Gap between triggers: `gap-1` (4 px).
pub const NAV_GAP: f32 = 4.0;
/// Viewport offset below the bar: `mt-1.5` (6 px).
pub const NAV_VIEWPORT_OFFSET: f32 = 6.0;
/// Content padding: `p-2` (8 px, `pr-2.5` on the right).
const CONTENT_PAD: f32 = 8.0;
/// Link padding: `p-2` (8 px).
const LINK_PAD: f32 = 8.0;
/// Link row spacing: `gap-1` inside a link, `gap-3` between links.
const LINK_GAP: f32 = 4.0;
/// Chevron icon size: `size-3` (12 px).
const CHEVRON_SIZE: f32 = 12.0;
/// Viewport max width cap (docs grids are `w-[400px]`).
const CONTENT_MAX_WIDTH: f32 = 400.0;
/// Viewport min width.
const CONTENT_MIN_WIDTH: f32 = 200.0;
/// Diamond indicator size: `w-2 h-2` (8 px square, rotated 45°).
const INDICATOR_SIZE: f32 = 8.0;

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// A content-panel builder closure (`NavigationMenuContent`).
pub type NavContentWidget<'a> = Box<dyn FnOnce(&mut NavContentBuilder) + 'a>;

/// One link inside a content panel (`NavigationMenuLink`).
#[derive(Clone, Debug, Default)]
pub struct NavLink {
    /// The link title (`text-sm font-medium`).
    pub title: String,
    /// The muted description under the title (empty = title only).
    pub description: String,
}

impl NavLink {
    /// Creates a link with a title and description.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
        }
    }
}

/// Builds the content of one [`NavigationMenuBuilder::trigger`] panel.
#[derive(Debug, Default)]
pub struct NavContentBuilder {
    pub(crate) links: Vec<NavLink>,
}

impl NavContentBuilder {
    /// Creates an empty content builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a link row with a title and description.
    pub fn link(&mut self, title: impl Into<String>, description: impl Into<String>) {
        self.links.push(NavLink::new(title, description));
    }

    /// Appends a title-only link row.
    pub fn link_plain(&mut self, title: impl Into<String>) {
        self.links.push(NavLink::new(title, ""));
    }
}

/// One entry of the menu bar.
pub(crate) enum NavEntry<'a> {
    /// A plain `NavigationMenuLink` item on the bar.
    Link(String),
    /// A `NavigationMenuTrigger` with its `NavigationMenuContent` panel.
    Trigger {
        label: String,
        disabled: bool,
        content: Option<NavContentWidget<'a>>,
    },
}

/// Collects the bar entries inside the [`NavigationMenu::show`] closure.
#[derive(Default)]
pub struct NavigationMenuBuilder<'a> {
    pub(crate) entries: Vec<NavEntry<'a>>,
}

impl<'a> NavigationMenuBuilder<'a> {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a plain link item on the bar.
    pub fn link(&mut self, label: impl Into<String>) {
        self.entries.push(NavEntry::Link(label.into()));
    }

    /// Appends a trigger with a content panel.
    pub fn trigger(
        &mut self,
        label: impl Into<String>,
        content: impl FnOnce(&mut NavContentBuilder) + 'a,
    ) {
        self.entries.push(NavEntry::Trigger {
            label: label.into(),
            disabled: false,
            content: Some(Box::new(content)),
        });
    }

    /// Appends a trigger without a panel (chevron-less plain item styled as
    /// a trigger).
    pub fn trigger_plain(&mut self, label: impl Into<String>) {
        self.entries.push(NavEntry::Trigger {
            label: label.into(),
            disabled: false,
            content: None,
        });
    }

    /// Appends a disabled trigger (`disabled:opacity-50`).
    pub fn trigger_disabled(&mut self, label: impl Into<String>) {
        self.entries.push(NavEntry::Trigger {
            label: label.into(),
            disabled: true,
            content: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Menu orientation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavOrientation {
    /// Horizontal bar with the viewport below (default).
    #[default]
    Horizontal,
    /// Stacked triggers with panels floating beside them.
    Vertical,
}

/// Resolved trigger colors: `hover:bg-accent focus:bg-accent
/// data-[state=open]:bg-accent/50` (nova uses muted tints) over
/// `bg-background`, `text-sm font-medium`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerColors {
    pub fill: Color32,
    pub text: Color32,
    pub chevron: Color32,
    pub ring: Color32,
}

pub(crate) fn resolve_trigger_colors(
    p: &Palette,
    open: bool,
    hovered: bool,
    disabled: bool,
) -> TriggerColors {
    let mut fill = Color32::TRANSPARENT;
    let mut text = p.foreground;
    if open {
        fill = color_alpha(p.accent, 0.5);
        text = p.accent_foreground;
    } else if hovered {
        fill = p.accent;
        text = p.accent_foreground;
    }
    if disabled {
        fill = color_alpha(fill, 0.5);
        text = color_alpha(text, 0.5);
    }
    TriggerColors {
        fill,
        text,
        chevron: text,
        ring: color_alpha(p.ring, 0.5),
    }
}

/// Resolved link-row colors: `hover:bg-accent focus:bg-accent
/// data-[active=true]:bg-accent/50`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LinkColors {
    pub fill: Color32,
    pub title: Color32,
    pub description: Color32,
}

pub(crate) fn resolve_link_colors(p: &Palette, active: bool, hovered: bool) -> LinkColors {
    if hovered {
        LinkColors {
            fill: p.accent,
            title: p.accent_foreground,
            description: color_alpha(p.accent_foreground, 0.7),
        }
    } else if active {
        LinkColors {
            fill: color_alpha(p.accent, 0.5),
            title: p.accent_foreground,
            description: color_alpha(p.accent_foreground, 0.7),
        }
    } else {
        LinkColors {
            fill: Color32::TRANSPARENT,
            title: p.popover_foreground,
            description: p.muted_foreground,
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Per-instance state: which trigger is open and the captured bar geometry.
#[derive(Clone, Debug, Default)]
struct NavState {
    /// Open trigger index (into the entries list).
    open: Option<usize>,
    /// Whether the open state came from hover (hover switches targets).
    hover_open: bool,
    /// Trigger rects captured this frame (per entry index).
    trigger_rects: Vec<Option<Rect>>,
    /// The whole bar/root rect captured this frame.
    root_rect: Option<Rect>,
}

fn read_state(ctx: &Context, id: Id) -> NavState {
    ctx.data_mut(|d| d.get_temp::<NavState>(id).unwrap_or_default())
}

fn write_state(ctx: &Context, id: Id, state: NavState) {
    ctx.data_mut(|d| d.insert_temp(id, state));
}

/// What [`NavigationMenu::show`] returns.
#[derive(Clone, Debug, Default)]
pub struct NavigationMenuResponse {
    /// The label of the trigger that is open after this frame, if any.
    pub open: Option<String>,
    /// The title of a link (bar item or panel row) clicked this frame.
    pub link_clicked: Option<String>,
    /// Whether the menu was open at frame start and is closed now.
    pub closed: bool,
}

// ---------------------------------------------------------------------------
// NavigationMenu (root)
// ---------------------------------------------------------------------------

/// A shadcn navigation menu. See the [module docs](self) for an example.
pub struct NavigationMenu<'a> {
    orientation: NavOrientation,
    /// `viewport(false)` renders panels next to their trigger instead of in
    /// the shared viewport below the bar.
    viewport: bool,
    id_salt: Option<String>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl Default for NavigationMenu<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> NavigationMenu<'a> {
    /// Creates a horizontal navigation menu with a shared viewport.
    pub fn new() -> Self {
        Self {
            orientation: NavOrientation::Horizontal,
            viewport: true,
            id_salt: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Sets the orientation (default [`NavOrientation::Horizontal`]).
    pub fn orientation(mut self, orientation: NavOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Whether the shared viewport renders below the bar (default `true`);
    /// `false` floats each panel next to its trigger (`viewport=false`).
    pub fn viewport(mut self, viewport: bool) -> Self {
        self.viewport = viewport;
        self
    }

    /// Sets a stable id salt so open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the menu bar and the open panel.
    pub fn show(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut NavigationMenuBuilder<'a>),
    ) -> NavigationMenuResponse {
        let mut builder = NavigationMenuBuilder::new();
        content(&mut builder);
        self.show_builder(ui, builder.entries)
    }

    fn show_builder(self, ui: &mut Ui, mut entries: Vec<NavEntry<'a>>) -> NavigationMenuResponse {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_navigation_menu"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        let open_at_start = state.open;
        let open_label_at_start =
            open_at_start.and_then(|i| entry_label(&entries, i).map(str::to_owned));

        // ------------------------------------------------------------------
        // The bar (triggers + plain links), capturing geometry.
        // ------------------------------------------------------------------
        let horizontal = self.orientation == NavOrientation::Horizontal;
        let mut link_clicked: Option<String> = None;
        let mut hover_switch: Option<usize> = None;
        let mut toggle: Option<usize> = None;
        state.trigger_rects = vec![None; entries.len()];

        let bar = ui.scope(|ui| {
            if horizontal {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = NAV_GAP;
                    for (i, entry) in entries.iter().enumerate() {
                        match entry {
                            NavEntry::Link(label) => {
                                let res = paint_bar_item(
                                    ui, state_id, i, label, None, false, false, &p, dark,
                                );
                                if res.clicked() {
                                    link_clicked = Some(label.clone());
                                }
                            }
                            NavEntry::Trigger {
                                label, disabled, ..
                            } => {
                                let is_open = state.open == Some(i);
                                let res = paint_bar_item(
                                    ui,
                                    state_id,
                                    i,
                                    label,
                                    Some(is_open),
                                    *disabled,
                                    true,
                                    &p,
                                    dark,
                                );
                                state.trigger_rects[i] = Some(res.rect);
                                if *disabled {
                                    continue;
                                }
                                if res.hovered() {
                                    hover_switch = Some(i);
                                }
                                if res.clicked() {
                                    toggle = Some(i);
                                }
                            }
                        }
                    }
                });
            } else {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = NAV_GAP;
                    for (i, entry) in entries.iter().enumerate() {
                        match entry {
                            NavEntry::Link(label) => {
                                let res = paint_bar_item(
                                    ui, state_id, i, label, None, false, false, &p, dark,
                                );
                                if res.clicked() {
                                    link_clicked = Some(label.clone());
                                }
                            }
                            NavEntry::Trigger {
                                label, disabled, ..
                            } => {
                                let is_open = state.open == Some(i);
                                let res = paint_bar_item(
                                    ui,
                                    state_id,
                                    i,
                                    label,
                                    Some(is_open),
                                    *disabled,
                                    true,
                                    &p,
                                    dark,
                                );
                                state.trigger_rects[i] = Some(res.rect);
                                if *disabled {
                                    continue;
                                }
                                if res.hovered() {
                                    hover_switch = Some(i);
                                }
                                if res.clicked() {
                                    toggle = Some(i);
                                }
                            }
                        }
                    }
                });
            }
        });
        state.root_rect = Some(bar.response.rect);

        // ------------------------------------------------------------------
        // Opening logic: click toggles; hover opens/switches.
        // ------------------------------------------------------------------
        let mut closed = false;
        if let Some(i) = toggle {
            if state.open == Some(i) {
                if state.hover_open {
                    // The menu was hover-opened by this press; a click
                    // adopts it into click-mode instead of closing.
                    state.hover_open = false;
                } else {
                    state.open = None;
                    state.hover_open = false;
                }
            } else {
                state.open = Some(i);
                state.hover_open = false;
            }
        } else if let Some(i) = hover_switch {
            if state.open.is_none() {
                state.open = Some(i);
                state.hover_open = true;
            } else if state.hover_open && state.open != Some(i) {
                state.open = Some(i);
            }
        }

        // ArrowLeft/Right (horizontal) switch the open trigger.
        if state.open.is_some() && horizontal {
            let (left, right) = ctx.input(|i| {
                (
                    i.key_pressed(Key::ArrowLeft),
                    i.key_pressed(Key::ArrowRight),
                )
            });
            let delta = if right {
                1
            } else if left {
                -1
            } else {
                0
            };
            if delta != 0 {
                let next = next_trigger_index(&entries, state.open, delta);
                if next != state.open {
                    state.open = next;
                }
            }
        }

        // Escape closes (dialogs keep priority).
        if state.open.is_some() {
            let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
            let esc = !modal_open && ctx.input(|i| i.key_pressed(Key::Escape));
            if esc {
                state.open = None;
            }
        }

        // ------------------------------------------------------------------
        // Panels: the shared viewport (below the bar) or per-trigger floats.
        // ------------------------------------------------------------------
        let popup_rects = if horizontal && self.viewport {
            render_viewport(
                ui,
                &ctx,
                &mut entries,
                &state,
                state_id,
                &p,
                dark,
                &theme,
                &mut link_clicked,
            )
        } else {
            render_float_panels(
                ui,
                &ctx,
                &mut entries,
                &state,
                state_id,
                horizontal,
                &p,
                dark,
                &theme,
                &mut link_clicked,
            )
        };

        // Click outside any panel or trigger closes.
        if state.open.is_some()
            && crate::components::dropdown_menu::handle_outside_click(
                &ctx,
                &popup_rects,
                &[bar.response.rect],
                open_at_start.is_some(),
            )
        {
            state.open = None;
        }

        // Expire hover-open when the pointer rests on neither trigger nor
        // panel (Radix keeps menus open on the viewport, so only close when
        // the pointer left everything AND a click happened — handled above).
        let open_now = state.open;
        let open_label = open_now.and_then(|i| entry_label(&entries, i).map(str::to_owned));
        closed = closed || (open_at_start.is_some() && open_now.is_none());
        let _ = open_label_at_start;
        write_state(&ctx, state_id, state);

        NavigationMenuResponse {
            open: open_label,
            link_clicked,
            closed,
        }
    }
}

/// The label of entry `i`, if it is a trigger.
fn entry_label<'a>(entries: &'a [NavEntry<'a>], i: usize) -> Option<&'a str> {
    match entries.get(i) {
        Some(NavEntry::Trigger { label, .. }) => Some(label),
        _ => None,
    }
}

/// Next (or previous) trigger index wrapping around, skipping disabled ones.
pub(crate) fn next_trigger_index(
    entries: &[NavEntry<'_>],
    current: Option<usize>,
    delta: isize,
) -> Option<usize> {
    let triggers: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            matches!(
                e,
                NavEntry::Trigger {
                    disabled: false,
                    ..
                }
            )
        })
        .map(|(i, _)| i)
        .collect();
    if triggers.is_empty() {
        return None;
    }
    let start = match current.and_then(|i| triggers.iter().position(|&t| t == i)) {
        Some(pos) => pos as isize,
        None => {
            if delta > 0 {
                -1
            } else {
                triggers.len() as isize
            }
        }
    };
    let next = (start + delta).rem_euclid(triggers.len() as isize) as usize;
    triggers.get(next).copied()
}

// ---------------------------------------------------------------------------
// Painters
// ---------------------------------------------------------------------------

/// Paints one bar item (trigger or plain link). Returns its response.
#[allow(clippy::too_many_arguments)]
fn paint_bar_item(
    ui: &mut Ui,
    _state_id: Id,
    index: usize,
    label: &str,
    open: Option<bool>,
    disabled: bool,
    with_chevron: bool,
    p: &Palette,
    _dark: bool,
) -> Response {
    let theme = ShadcnTheme::of_ui(ui);
    let id = ui.scope_id().with(("nav_item", index));
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);

    // Measure: px-4 + text + (ml-1 size-3 chevron).
    let text_w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x;
    let width = NAV_TRIGGER_PAD_X * 2.0
        + text_w
        + if with_chevron {
            4.0 + CHEVRON_SIZE
        } else {
            0.0
        };

    let (_, rect) = ui.allocate_space(Vec2::new(width, NAV_TRIGGER_HEIGHT));
    let sense = if disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(rect, id, sense);

    if ui.is_rect_visible(rect) {
        let c = resolve_trigger_colors(p, open.unwrap_or(false), response.hovered(), disabled);
        let painter = ui.painter();
        if c.fill != Color32::TRANSPARENT {
            painter.rect_filled(rect, CornerRadius::same(theme.radius.md_u8()), c.fill);
        }
        let text_x = rect.min.x + NAV_TRIGGER_PAD_X;
        painter.text(
            Pos2::new(text_x, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font,
            c.text,
        );
        if with_chevron {
            // `size-3 ml-1`, rotated when open.
            let chev_rect = Rect::from_center_size(
                Pos2::new(
                    rect.max.x - NAV_TRIGGER_PAD_X - CHEVRON_SIZE * 0.5,
                    rect.center().y,
                ),
                Vec2::splat(CHEVRON_SIZE),
            );
            let chev_icon = if open.unwrap_or(false) {
                Icon::ChevronUp
            } else {
                Icon::ChevronDown
            };
            chev_icon.paint(painter, chev_rect, c.chevron, 2.0 * CHEVRON_SIZE / 24.0);
        }
        if response.has_focus() && !disabled {
            let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
            crate::utils::focus::paint_ring(painter, ring_rect, c.ring, 3.0, theme.radius.md());
        }
    }
    response
}

/// The link rows inside a panel (`NavigationMenuLink`); returns the clicked
/// title.
fn paint_links(
    ui: &mut Ui,
    links: &[NavLink],
    panel_width: f32,
    p: &Palette,
    theme: &ShadcnTheme,
) -> Option<String> {
    let mut clicked = None;
    for (i, link) in links.iter().enumerate() {
        let row_id = ui.scope_id().with(("nav_link", i));
        // Measure the description wrap at the panel width.
        let desc_font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let inner_w = panel_width - 2.0 * CONTENT_PAD - 2.0 * LINK_PAD;
        let desc_galley = if link.description.is_empty() {
            None
        } else {
            Some(ui.painter().layout(
                link.description.clone(),
                desc_font.clone(),
                Color32::WHITE,
                inner_w.max(40.0),
            ))
        };
        let title_font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let title_galley =
            ui.painter()
                .layout_no_wrap(link.title.clone(), title_font.clone(), Color32::WHITE);
        let row_h = LINK_PAD * 2.0
            + title_galley.size().y
            + desc_galley
                .as_ref()
                .map(|g| LINK_GAP + g.size().y)
                .unwrap_or(0.0);

        let (_, rect) = ui.allocate_space(Vec2::new(panel_width - 2.0 * CONTENT_PAD, row_h));
        let response = ui.interact(rect, row_id, Sense::click());
        let hovered = response.hovered();
        let active = hovered;
        let c = resolve_link_colors(p, active, hovered);
        let radius = theme.radius.sm_u8();

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            if c.fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, CornerRadius::same(radius), c.fill);
            }
            let mut y = rect.min.y + LINK_PAD;
            painter.text(
                Pos2::new(rect.min.x + LINK_PAD, y + title_galley.size().y * 0.5),
                Align2::LEFT_CENTER,
                link.title.clone(),
                title_font,
                c.title,
            );
            y += title_galley.size().y;
            if let Some(g) = desc_galley {
                y += LINK_GAP;
                painter.galley(Pos2::new(rect.min.x + LINK_PAD, y), g, c.description);
            }
        }
        if response.clicked() {
            clicked = Some(link.title.clone());
        }
    }
    clicked
}

/// Moves the content closure of trigger `idx` out of the entries (the
/// builder rebuilds them every frame, so consuming one is fine).
fn take_panel_content<'x>(
    entries: &'x mut [NavEntry<'_>],
    idx: usize,
) -> Option<NavContentWidget<'x>> {
    match entries.get_mut(idx) {
        Some(NavEntry::Trigger { content, .. }) => content.take(),
        _ => None,
    }
}

/// Builds and measures one panel's content (calling the closure once).
fn panel_content(content: Option<NavContentWidget<'_>>) -> Vec<NavLink> {
    let mut builder = NavContentBuilder::new();
    if let Some(build) = content {
        build(&mut builder);
    }
    builder.links
}

/// Panel width from its links (clamped to the docs' 400 px grid).
fn panel_width(links: &[NavLink], ui: &Ui) -> f32 {
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let mut w = CONTENT_MIN_WIDTH;
    for link in links {
        let title_w = ui
            .painter()
            .layout_no_wrap(link.title.clone(), font.clone(), Color32::WHITE)
            .size()
            .x;
        let desc_w = if link.description.is_empty() {
            0.0
        } else {
            ui.painter()
                .layout(
                    link.description.clone(),
                    font.clone(),
                    Color32::WHITE,
                    CONTENT_MAX_WIDTH - 2.0 * CONTENT_PAD - 2.0 * LINK_PAD,
                )
                .size()
                .x
        };
        w = w.max(title_w.max(desc_w) + 2.0 * CONTENT_PAD + 2.0 * LINK_PAD);
    }
    w.min(CONTENT_MAX_WIDTH)
}

/// Renders the shared viewport below the bar (plus the diamond indicator).
#[allow(clippy::too_many_arguments)]
fn render_viewport(
    ui: &mut Ui,
    ctx: &Context,
    entries: &mut [NavEntry<'_>],
    state: &NavState,
    state_id: Id,
    p: &Palette,
    dark: bool,
    theme: &ShadcnTheme,
    link_clicked: &mut Option<String>,
) -> Vec<Rect> {
    let mut rects = Vec::new();
    let Some(open_idx) = state.open else {
        return rects;
    };
    if !matches!(entries.get(open_idx), Some(NavEntry::Trigger { .. })) {
        return rects;
    }
    let Some(root_rect) = state.root_rect else {
        return rects;
    };
    let links = panel_content(take_panel_content(entries, open_idx));
    if links.is_empty() {
        return rects;
    }
    let width = panel_width(&links, ui);

    let viewport_id = state_id.with("viewport");
    let progress = overlay_progress(ctx, viewport_id.with("anim"), true);
    if progress.alpha <= 0.002 {
        return rects;
    }

    // Diamond indicator under the open trigger (`bg-border rotate-45`).
    if let Some(Some(trigger_rect)) = state.trigger_rects.get(open_idx) {
        let center = Pos2::new(trigger_rect.center().x, root_rect.max.y + 3.0);
        let painter = ui.painter();
        let color = p.border;
        let half = INDICATOR_SIZE * 0.5;
        let pts = [
            Pos2::new(center.x, center.y - half),
            Pos2::new(center.x + half, center.y),
            Pos2::new(center.x, center.y + half),
            Pos2::new(center.x - half, center.y),
        ];
        painter.add(egui::Shape::convex_polygon(
            pts.to_vec(),
            color,
            Stroke::NONE,
        ));
    }

    let frame = egui::Frame::new()
        .fill(p.popover)
        .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
        .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
        .inner_margin(Margin::same(CONTENT_PAD as i8))
        .shadow(shadow::shadow_md(dark));

    let popup = egui::Popup::new(
        viewport_id,
        ctx.clone(),
        egui::PopupAnchor::ParentRect(Rect::from_min_size(
            Pos2::new(root_rect.min.x, root_rect.max.y),
            egui::vec2(root_rect.width(), 0.0),
        )),
        ui.layer_id(),
    )
    .open(true)
    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
    .align(RectAlign::BOTTOM_START)
    .gap(NAV_VIEWPORT_OFFSET + (1.0 - progress.alpha) * 8.0)
    .width(width)
    .frame(egui::Frame::new());

    let inner = popup.show(|ui| {
        ui.set_opacity(progress.alpha);
        frame.show(ui, |ui| {
            ui.set_min_width(width - 2.0 * CONTENT_PAD);
            ui.set_max_width(width - 2.0 * CONTENT_PAD);
            if let Some(title) = paint_links(ui, &links, width, p, theme) {
                *link_clicked = Some(title);
            }
        });
    });
    if let Some(ref r) = inner {
        rects.push(r.response.rect);
    }
    rects
}

/// Renders panels floating next to their trigger (`viewport=false`, or the
/// vertical orientation).
#[allow(clippy::too_many_arguments)]
fn render_float_panels(
    ui: &mut Ui,
    ctx: &Context,
    entries: &mut [NavEntry<'_>],
    state: &NavState,
    state_id: Id,
    horizontal: bool,
    p: &Palette,
    dark: bool,
    theme: &ShadcnTheme,
    link_clicked: &mut Option<String>,
) -> Vec<Rect> {
    let mut rects = Vec::new();
    let Some(open_idx) = state.open else {
        return rects;
    };
    if !matches!(entries.get(open_idx), Some(NavEntry::Trigger { .. })) {
        return rects;
    }
    let Some(Some(trigger_rect)) = state.trigger_rects.get(open_idx) else {
        return rects;
    };
    let links = panel_content(take_panel_content(entries, open_idx));
    if links.is_empty() {
        return rects;
    }
    let width = panel_width(&links, ui);

    let popup_id = state_id.with(("panel", open_idx));
    let progress = overlay_progress(ctx, popup_id.with("anim"), true);
    if progress.alpha <= 0.002 {
        return rects;
    }

    let frame = egui::Frame::new()
        .fill(p.popover)
        .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
        .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
        .inner_margin(Margin::same(CONTENT_PAD as i8))
        .shadow(shadow::shadow_md(dark));

    let align = if horizontal {
        RectAlign::BOTTOM_START
    } else {
        RectAlign::RIGHT_START
    };

    let popup = egui::Popup::new(
        popup_id,
        ctx.clone(),
        egui::PopupAnchor::ParentRect(*trigger_rect),
        ui.layer_id(),
    )
    .open(true)
    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
    .align(align)
    .gap(crate::components::dropdown_menu::DROPDOWN_SIDE_OFFSET + (1.0 - progress.alpha) * 8.0)
    .width(width)
    .frame(egui::Frame::new());

    let inner = popup.show(|ui| {
        ui.set_opacity(progress.alpha);
        frame.show(ui, |ui| {
            ui.set_min_width(width - 2.0 * CONTENT_PAD);
            ui.set_max_width(width - 2.0 * CONTENT_PAD);
            if let Some(title) = paint_links(ui, &links, width, p, theme) {
                *link_clicked = Some(title);
            }
        });
    });
    if let Some(ref r) = inner {
        rects.push(r.response.rect);
    }
    rects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    fn sample_entries() -> Vec<NavEntry<'static>> {
        vec![
            NavEntry::Link("Introduction".to_owned()),
            NavEntry::Trigger {
                label: "Learn".to_owned(),
                disabled: false,
                content: Some(Box::new(|c: &mut NavContentBuilder| {
                    c.link("Library", "Structured access.");
                    c.link("Components", "Building blocks.");
                })),
            },
            NavEntry::Trigger {
                label: "Disabled".to_owned(),
                disabled: true,
                content: None,
            },
            NavEntry::Trigger {
                label: "Examples".to_owned(),
                disabled: false,
                content: Some(Box::new(|c: &mut NavContentBuilder| {
                    c.link_plain("Email");
                })),
            },
        ]
    }

    #[test]
    fn trigger_indices_skip_links_and_disabled() {
        let e = sample_entries();
        assert_eq!(
            next_trigger_index(&e, None, 1),
            Some(1),
            "first enabled trigger"
        );
        assert_eq!(
            next_trigger_index(&e, Some(1), 1),
            Some(3),
            "skips disabled"
        );
        assert_eq!(next_trigger_index(&e, Some(3), 1), Some(1), "wraps");
        assert_eq!(next_trigger_index(&e, Some(3), -1), Some(1));
        assert_eq!(next_trigger_index(&e, Some(1), -1), Some(3), "wraps back");
        let empty: Vec<NavEntry<'static>> = Vec::new();
        assert_eq!(next_trigger_index(&empty, None, 1), None);
    }

    #[test]
    fn trigger_colors_follow_state() {
        let p = palette(Mode::Light);
        let rest = resolve_trigger_colors(&p, false, false, false);
        assert_eq!(rest.fill, Color32::TRANSPARENT);
        assert_eq!(rest.text, p.foreground);
        let open = resolve_trigger_colors(&p, true, false, false);
        assert_eq!(open.fill, color_alpha(p.accent, 0.5));
        assert_eq!(open.text, p.accent_foreground);
        let hovered = resolve_trigger_colors(&p, false, true, false);
        assert_eq!(hovered.fill, p.accent);
        let disabled = resolve_trigger_colors(&p, false, false, true);
        assert_eq!(disabled.text, color_alpha(p.foreground, 0.5));
    }

    #[test]
    fn link_colors_follow_state() {
        let p = palette(Mode::Light);
        let rest = resolve_link_colors(&p, false, false);
        assert_eq!(rest.fill, Color32::TRANSPARENT);
        assert_eq!(rest.title, p.popover_foreground);
        assert_eq!(rest.description, p.muted_foreground);
        let hovered = resolve_link_colors(&p, false, true);
        assert_eq!(hovered.fill, p.accent);
        assert_eq!(hovered.title, p.accent_foreground);
        let active = resolve_link_colors(&p, true, false);
        assert_eq!(active.fill, color_alpha(p.accent, 0.5));
    }

    #[test]
    fn geometry_constants_match_source() {
        assert_eq!(NAV_TRIGGER_HEIGHT, 36.0, "h-9");
        assert_eq!(NAV_TRIGGER_PAD_X, 16.0, "px-4");
        assert_eq!(NAV_GAP, 4.0, "gap-1");
        assert_eq!(NAV_VIEWPORT_OFFSET, 6.0, "mt-1.5");
        assert_eq!(CONTENT_MAX_WIDTH, 400.0, "w-[400px]");
        assert_eq!(CHEVRON_SIZE, 12.0, "size-3");
    }

    #[test]
    fn content_builder_collects_links() {
        let mut c = NavContentBuilder::new();
        c.link("Library", "Structured access.");
        c.link_plain("Email");
        assert_eq!(c.links.len(), 2);
        assert_eq!(c.links[0].title, "Library");
        assert_eq!(c.links[0].description, "Structured access.");
        assert_eq!(c.links[1].description, "");
    }

    /// Advances time so animations make progress across frames.
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
                    Id::unique("nav_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn click(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.time += 0.1;
                let input = egui::RawInput {
                    time: Some(self.time),
                    events: vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    ..Default::default()
                };
                let mut out = self.ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("nav_test_ui"),
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
                    Id::unique("nav_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> NavState {
            let id = Id::unique("nav_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<NavState>(id))
                .unwrap_or_default()
        }
    }

    fn menu_ui(ui: &mut Ui, salt: &str) -> NavigationMenuResponse {
        NavigationMenu::new().id_salt(salt).show(ui, |nav| {
            nav.link("Introduction");
            nav.trigger("Learn", |c| {
                c.link("Library", "Structured access.");
            });
        })
    }

    fn trigger_rect(h: &mut Harness, salt: &str, index: usize) -> Rect {
        let mut rect = Rect::ZERO;
        let ctx = h.ctx.clone();
        h.run(|ui| {
            let _ = menu_ui(ui, salt);
            let st: NavState = ctx
                .data_mut(|d| d.get_temp(Id::unique("nav_test_ui").with(salt)))
                .unwrap_or_default();
            rect = st
                .trigger_rects
                .get(index)
                .copied()
                .flatten()
                .unwrap_or(Rect::ZERO);
        });
        h.run(|ui| {
            let _ = menu_ui(ui, salt);
            let st: NavState = ctx
                .data_mut(|d| d.get_temp(Id::unique("nav_test_ui").with(salt)))
                .unwrap_or_default();
            rect = st
                .trigger_rects
                .get(index)
                .copied()
                .flatten()
                .unwrap_or(Rect::ZERO);
        });
        rect
    }

    #[test]
    fn trigger_click_toggles_open_state() {
        let mut h = Harness::new();
        let rect = trigger_rect(&mut h, "t1", 1);
        assert!(h.state("t1").open.is_none(), "default is closed");

        h.click(rect.center(), |ui| {
            let _ = menu_ui(ui, "t1");
        });
        assert_eq!(h.state("t1").open, Some(1), "clicking the trigger opens it");

        h.click(rect.center(), |ui| {
            let _ = menu_ui(ui, "t1");
        });
        assert!(h.state("t1").open.is_none(), "clicking again closes it");
    }

    #[test]
    fn escape_closes_open_menu() {
        let mut h = Harness::new();
        let rect = trigger_rect(&mut h, "esc", 1);
        h.click(rect.center(), |ui| {
            let _ = menu_ui(ui, "esc");
        });
        assert_eq!(h.state("esc").open, Some(1));
        h.run(|ui| {
            let _ = menu_ui(ui, "esc");
        });
        h.key(Key::Escape, |ui| {
            let _ = menu_ui(ui, "esc");
        });
        assert!(h.state("esc").open.is_none(), "Escape must close the menu");
    }

    #[test]
    fn arrow_keys_switch_open_trigger() {
        let mut h = Harness::new();
        let ctx = h.ctx.clone();
        let mut r1 = Rect::ZERO;
        let read_r1 = |ctx: &egui::Context| -> Rect {
            let st: NavState = ctx
                .data_mut(|d| d.get_temp(Id::unique("nav_test_ui").with("arrows")))
                .unwrap_or_default();
            st.trigger_rects[0].unwrap_or(Rect::ZERO)
        };
        h.run(|ui| {
            let _ = NavigationMenu::new().id_salt("arrows").show(ui, |nav| {
                nav.trigger("One", |c| c.link_plain("A"));
                nav.trigger("Two", |c| c.link_plain("B"));
            });
            r1 = read_r1(&ctx);
        });
        h.run(|ui| {
            let _ = NavigationMenu::new().id_salt("arrows").show(ui, |nav| {
                nav.trigger("One", |c| c.link_plain("A"));
                nav.trigger("Two", |c| c.link_plain("B"));
            });
            r1 = read_r1(&ctx);
        });

        h.click(r1.center(), |ui| {
            let _ = NavigationMenu::new().id_salt("arrows").show(ui, |nav| {
                nav.trigger("One", |c| c.link_plain("A"));
                nav.trigger("Two", |c| c.link_plain("B"));
            });
        });
        assert_eq!(h.state("arrows").open, Some(0));

        h.key(Key::ArrowRight, |ui| {
            let _ = NavigationMenu::new().id_salt("arrows").show(ui, |nav| {
                nav.trigger("One", |c| c.link_plain("A"));
                nav.trigger("Two", |c| c.link_plain("B"));
            });
        });
        assert_eq!(h.state("arrows").open, Some(1), "ArrowRight switches");
    }

    #[test]
    fn hover_opens_when_closed() {
        let mut h = Harness::new();
        let rect = trigger_rect(&mut h, "hov", 1);
        // Hover (pointer move) opens a closed menu.
        h.time += 0.1;
        let input = egui::RawInput {
            time: Some(h.time),
            events: vec![egui::Event::PointerMoved(rect.center())],
            ..Default::default()
        };
        let mut out = h.ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("nav_test_ui"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
            );
            let _ = menu_ui(ui, "hov");
        });
        out.textures_delta.clear();
        assert_eq!(h.state("hov").open, Some(1), "hover opens the menu");
    }

    #[test]
    fn clicking_a_panel_link_reports_it() {
        let mut h = Harness::new();
        let rect = trigger_rect(&mut h, "lnk", 1);
        h.click(rect.center(), |ui| {
            let _ = menu_ui(ui, "lnk");
        });
        assert_eq!(h.state("lnk").open, Some(1));

        // Settle one frame so the popup rects exist, then click the row.
        h.run(|ui| {
            let _ = menu_ui(ui, "lnk");
        });
        let ctx = h.ctx.clone();
        let mut hit = false;
        let popup_pos = Pos2::new(rect.center().x, rect.max.y + 40.0);
        h.click(popup_pos, |ui| {
            let res = menu_ui(ui, "lnk");
            if res.link_clicked.is_some() {
                hit = true;
            }
            let _ = ctx.clone();
        });
        assert!(hit, "clicking the panel link reports link_clicked");
    }
}
