//! shadcn `dropdown-menu.tsx` — a button-triggered menu with items, labels,
//! separators, shortcuts, checkbox/radio items and sub-menus.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx` (Radix
//! DropdownMenu) with the nova recipe: content `bg-popover text-popover-foreground
//! min-w-32 rounded-lg p-1 shadow-md ring-1 ring-foreground/10`, items
//! `rounded-md px-1.5 py-1 text-sm focus:bg-accent focus:text-accent-foreground`,
//! checkbox/radio items with the indicator on the right (`pr-8`, indicator
//! `right-2`), labels `text-xs font-medium text-muted-foreground`, destructive
//! items `text-destructive` with `focus:bg-destructive/10` (dark: `/20`).
//!
//! The menu is declarative: the content closure builds the item list, and
//! `show` renders the trigger plus the anchored popup. Keyboard support
//! mirrors Radix: ArrowUp/Down move the highlight (skipping disabled items,
//! wrapping), ArrowRight opens a sub-menu, ArrowLeft closes it, Enter/Space
//! activate, Home/End jump, Escape closes (deepest sub-menu first).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut show_status_bar = true;
//! let mut theme = Some(0);
//!
//! let menu = DropdownMenu::new("Options")
//!     .align(MenuAlign::Start)
//!     .width(224.0)
//!     .show(ui, |m| {
//!         m.label("Appearance");
//!         m.separator();
//!         m.checkbox_item(CheckboxItem::new("Status Bar", &mut show_status_bar));
//!         m.radio_group(&mut theme, |g| {
//!             g.radio_labels(["Light", "Dark", "System"]);
//!         });
//!         m.sub_menu("Share", |m| {
//!             m.item(MenuItem::new("Email"));
//!             m.item(MenuItem::new("Message"));
//!         });
//!         m.separator();
//!         m.item(MenuItem::new("Log out").shortcut("⇧⌘Q"));
//!         m.item(MenuItem::new("Delete account").destructive(true));
//!         m.item(MenuItem::new("API").disabled(true));
//!     });
//! if menu.clicked("Log out") { /* … */ }
//!
//! // Controlled open state:
//! let mut open = false;
//! DropdownMenu::controlled("Open", &mut open).show(ui, |m| {
//!     m.item(MenuItem::new("Hi"));
//! });
//! ```
//!
//! The menu model ([`MenuBuilder`], [`MenuItem`], [`CheckboxItem`],
//! [`RadioItem`]) is shared with the context-menu and menubar components,
//! which reuse the level renderer and keyboard helpers from this module.

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use crate::utils::animation::overlay_progress;
use crate::utils::shadow;
use egui::emath::RectAlign;
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Key, LayerId, Margin, Painter, Popup,
    PopupAnchor, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Geometry constants (nova recipe)
// ---------------------------------------------------------------------------

/// Content min width: `min-w-32` (128 px).
pub const DROPDOWN_MIN_WIDTH: f32 = 128.0;
/// Sub-menu content min width: `min-w-[96px]` (96 px).
pub const SUB_MENU_MIN_WIDTH: f32 = 96.0;
/// Content padding: `p-1` (4 px).
pub(crate) const MENU_CONTENT_PADDING: f32 = 4.0;
/// Item row height: `py-1` + one `text-sm` line (min-h-7).
pub(crate) const MENU_ROW_HEIGHT: f32 = 28.0;
/// Label row height: `py-1` + one `text-xs` line.
pub(crate) const MENU_LABEL_HEIGHT: f32 = 20.0;
/// Item horizontal padding: `px-1.5` (6 px).
pub(crate) const MENU_ROW_PAD_X: f32 = 6.0;
/// Checkbox/radio trailing space for the indicator: `pr-8` (32 px).
pub(crate) const MENU_INDICATOR_SPACE: f32 = 32.0;
/// Leading space for `inset` rows: `pl-7` (28 px).
pub(crate) const MENU_INSET_PAD: f32 = 28.0;
/// Gap between an item icon and its label: `gap-1.5` (6 px).
const MENU_ICON_GAP: f32 = 6.0;
/// Item icon size: `size-4` (16 px).
pub(crate) const MENU_ICON_SIZE: f32 = 16.0;
/// Separator total height: `my-1` (4 px top and bottom) + 1 px line.
pub(crate) const MENU_SEPARATOR_HEIGHT: f32 = 9.0;

// ---------------------------------------------------------------------------
// Item model
// ---------------------------------------------------------------------------

/// Where a checkbox/radio indicator sits (nova: dropdown/context right,
/// menubar left).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IndicatorSide {
    /// Indicator at the left edge (menubar: `pl-7`, indicator `left-1.5`).
    Left,
    /// Indicator at the right edge (dropdown/context: `pr-8`, indicator `right-2`).
    Right,
}

/// Per-component visual tweaks of the shared menu rows (nova recipe).
#[derive(Clone, Copy, Debug)]
pub(crate) struct MenuSpec {
    /// Content min width in px.
    pub min_width: f32,
    /// Checkbox/radio indicator side.
    pub indicator: IndicatorSide,
    /// `true` for dropdown/context (`text-muted-foreground text-xs` label),
    /// `false` for menubar (`text-sm` foreground label).
    pub label_muted: bool,
}

/// A plain menu entry (`DropdownMenuItem`).
#[derive(Clone, Debug, Default)]
pub struct MenuItem {
    pub(crate) label: String,
    pub(crate) id: Option<String>,
    pub(crate) shortcut: Option<String>,
    pub(crate) icon: Option<Icon>,
    pub(crate) disabled: bool,
    pub(crate) destructive: bool,
    pub(crate) inset: bool,
}

impl MenuItem {
    /// Creates an enabled item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    /// Explicit activation id (defaults to the label).
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Right-aligned `DropdownMenuShortcut` text (`⌘P`).
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Leading lucide icon (`size-4`, muted unless destructive).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// `data-[disabled]:opacity-50` — not clickable and skipped by keyboard nav.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `data-[variant=destructive]` — red text, red-tinted highlight.
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    /// `data-[inset]:pl-8` — extra leading space (aligns with checkable rows).
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }
}

/// A checkable menu entry (`DropdownMenuCheckboxItem`). The bound `bool` is
/// toggled when the row is activated.
#[derive(Debug)]
pub struct CheckboxItem<'m> {
    pub(crate) item: MenuItem,
    pub(crate) checked: &'m mut bool,
}

impl<'m> CheckboxItem<'m> {
    /// Creates a checkbox item bound to the caller's flag.
    pub fn new(label: impl Into<String>, checked: &'m mut bool) -> Self {
        Self {
            item: MenuItem::new(label),
            checked,
        }
    }

    /// Explicit activation id (defaults to the label).
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.item = self.item.id(id);
        self
    }

    /// Right-aligned shortcut text.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.item = self.item.shortcut(shortcut);
        self
    }

    /// Disables the row.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.item = self.item.disabled(disabled);
        self
    }

    /// Destructive variant.
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.item = self.item.destructive(destructive);
        self
    }

    /// Extra leading space.
    pub fn inset(mut self, inset: bool) -> Self {
        self.item = self.item.inset(inset);
        self
    }
}

/// A radio entry inside a [`MenuBuilder::radio_group`] (`DropdownMenuRadioItem`).
#[derive(Clone, Debug, Default)]
pub struct RadioItem {
    pub(crate) item: MenuItem,
}

impl RadioItem {
    /// Creates a radio item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            item: MenuItem::new(label),
        }
    }

    /// Explicit activation id (defaults to the label).
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.item = self.item.id(id);
        self
    }

    /// Right-aligned shortcut text.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.item = self.item.shortcut(shortcut);
        self
    }

    /// Disables the row.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.item = self.item.disabled(disabled);
        self
    }

    /// Extra leading space.
    pub fn inset(mut self, inset: bool) -> Self {
        self.item = self.item.inset(inset);
        self
    }
}

/// One entry of the shared menu model (used by the dropdown, context-menu
/// and menubar components).
#[derive(Debug)]
pub(crate) enum EntryKind<'m> {
    Item {
        item: MenuItem,
    },
    Checkbox {
        item: MenuItem,
        checked: &'m mut bool,
    },
    RadioGroup {
        selection: &'m mut Option<usize>,
        items: Vec<MenuEntry<'m>>,
    },
    Label {
        text: String,
        inset: bool,
    },
    Separator,
    Sub {
        label: String,
        items: Vec<MenuEntry<'m>>,
        disabled: bool,
        inset: bool,
    },
}

/// A menu tree node.
#[derive(Debug)]
pub(crate) struct MenuEntry<'m> {
    pub kind: EntryKind<'m>,
}

/// Collects menu entries inside a `show` closure. The same builder type is
/// used for the root content, [`MenuBuilder::sub_menu`] bodies and radio
/// groups (via [`RadioGroupBuilder`]).
pub struct MenuBuilder<'m> {
    pub(crate) entries: Vec<MenuEntry<'m>>,
}

impl Default for MenuBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'m> MenuBuilder<'m> {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Appends a plain item.
    pub fn item(&mut self, item: MenuItem) {
        self.entries.push(MenuEntry {
            kind: EntryKind::Item { item },
        });
    }

    /// Appends a checkbox item bound to `checked`.
    pub fn checkbox_item(&mut self, item: CheckboxItem<'m>) {
        self.entries.push(MenuEntry {
            kind: EntryKind::Checkbox {
                item: item.item,
                checked: item.checked,
            },
        });
    }

    /// Appends a radio group (`DropdownMenuRadioGroup`). The selection index
    /// is relative to the group.
    pub fn radio_group(
        &mut self,
        selection: &'m mut Option<usize>,
        build: impl FnOnce(&mut RadioGroupBuilder<'m>),
    ) {
        let mut group = RadioGroupBuilder::new();
        build(&mut group);
        self.entries.push(MenuEntry {
            kind: EntryKind::RadioGroup {
                selection,
                items: group.builder.entries,
            },
        });
    }

    /// Appends a sub-menu with a nested item list.
    pub fn sub_menu(&mut self, label: impl Into<String>, build: impl FnOnce(&mut MenuBuilder<'m>)) {
        let mut inner = MenuBuilder::new();
        build(&mut inner);
        self.entries.push(MenuEntry {
            kind: EntryKind::Sub {
                label: label.into(),
                items: inner.entries,
                disabled: false,
                inset: false,
            },
        });
    }

    /// Appends a disabled sub-menu trigger.
    pub fn sub_menu_disabled(
        &mut self,
        label: impl Into<String>,
        build: impl FnOnce(&mut MenuBuilder<'m>),
    ) {
        let mut inner = MenuBuilder::new();
        build(&mut inner);
        self.entries.push(MenuEntry {
            kind: EntryKind::Sub {
                label: label.into(),
                items: inner.entries,
                disabled: true,
                inset: false,
            },
        });
    }

    /// Appends a non-interactive section label.
    pub fn label(&mut self, text: impl Into<String>) {
        self.entries.push(MenuEntry {
            kind: EntryKind::Label {
                text: text.into(),
                inset: false,
            },
        });
    }

    /// Appends an inset section label (extra leading space).
    pub fn label_inset(&mut self, text: impl Into<String>) {
        self.entries.push(MenuEntry {
            kind: EntryKind::Label {
                text: text.into(),
                inset: true,
            },
        });
    }

    /// Appends a full-bleed separator line.
    pub fn separator(&mut self) {
        self.entries.push(MenuEntry {
            kind: EntryKind::Separator,
        });
    }
}

/// Builder for the entries of one radio group.
pub struct RadioGroupBuilder<'m> {
    builder: MenuBuilder<'m>,
}

impl Default for RadioGroupBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'m> RadioGroupBuilder<'m> {
    /// Creates an empty radio group builder.
    pub fn new() -> Self {
        Self {
            builder: MenuBuilder::new(),
        }
    }

    /// Appends one radio item.
    pub fn radio_item(&mut self, item: RadioItem) {
        self.builder.entries.push(MenuEntry {
            kind: EntryKind::Item { item: item.item },
        });
    }

    /// Appends several radio items.
    pub fn radio_items(&mut self, items: impl IntoIterator<Item = RadioItem>) {
        for item in items {
            self.radio_item(item);
        }
    }

    /// Appends radio items from plain labels.
    pub fn radio_labels(&mut self, labels: impl IntoIterator<Item = impl Into<String>>) {
        for label in labels {
            self.radio_item(RadioItem::new(label));
        }
    }
}

// ---------------------------------------------------------------------------
// Pure tree helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Number of keyboard-navigable entries at one level (items, checkbox items,
/// radio items and sub triggers; labels and separators are skipped, radio
/// groups contribute their items).
pub(crate) fn count_actionable(entries: &[MenuEntry<'_>]) -> usize {
    entries
        .iter()
        .map(|e| match &e.kind {
            EntryKind::Item { .. } | EntryKind::Checkbox { .. } | EntryKind::Sub { .. } => 1,
            EntryKind::RadioGroup { items, .. } => count_actionable(items),
            EntryKind::Label { .. } | EntryKind::Separator => 0,
        })
        .sum()
}

/// The entry selected by `action_idx` at this level (radio groups are
/// transparent: their items count at the same level).
fn entry_at_level<'a, 'm>(
    entries: &'a [MenuEntry<'m>],
    action_idx: usize,
) -> Option<&'a MenuEntry<'m>> {
    let mut seen = 0usize;
    for entry in entries {
        match &entry.kind {
            EntryKind::Item { .. } | EntryKind::Checkbox { .. } | EntryKind::Sub { .. } => {
                if seen == action_idx {
                    return Some(entry);
                }
                seen += 1;
            }
            EntryKind::RadioGroup { items, .. } => {
                if action_idx < seen + items.len() {
                    return items.get(action_idx - seen);
                }
                seen += items.len();
            }
            EntryKind::Label { .. } | EntryKind::Separator => {}
        }
    }
    None
}

/// Resolves a highlight/activation path to the entry it points at.
/// `path = [a0, a1, …]`: `a0` indexes the root level; when that entry is a
/// sub-menu and the path continues, the next index applies to its items.
pub(crate) fn resolve_action<'a, 'm>(
    entries: &'a [MenuEntry<'m>],
    path: &[usize],
) -> Option<&'a MenuEntry<'m>> {
    let (first, rest) = path.split_first()?;
    let entry = entry_at_level(entries, *first)?;
    match rest.first() {
        Some(&next) => match &entry.kind {
            EntryKind::Sub { items, .. } => resolve_action(items, &[next]),
            _ => None,
        },
        None => Some(entry),
    }
}

/// Whether the entry is keyboard-skippable (disabled).
pub(crate) fn entry_disabled(entry: &MenuEntry<'_>) -> bool {
    match &entry.kind {
        EntryKind::Item { item } => item.disabled,
        EntryKind::Checkbox { item, .. } => item.disabled,
        EntryKind::Sub { disabled, .. } => *disabled,
        EntryKind::RadioGroup { .. } | EntryKind::Label { .. } | EntryKind::Separator => false,
    }
}

/// Next actionable index starting from `from` in direction `delta`, wrapping
/// once and skipping disabled entries.
fn next_actionable(entries: &[MenuEntry<'_>], from: usize, delta: isize) -> Option<usize> {
    let count = count_actionable(entries);
    if count == 0 {
        return None;
    }
    let mut idx = from as isize;
    for _ in 0..count {
        idx = (idx + delta).rem_euclid(count as isize);
        if let Some(entry) = entry_at_level(entries, idx as usize) {
            if !entry_disabled(entry) {
                return Some(idx as usize);
            }
        }
    }
    None
}

/// First (or last, when `from_end`) navigable index at a level.
pub(crate) fn edge_actionable(entries: &[MenuEntry<'_>], from_end: bool) -> Option<usize> {
    let count = count_actionable(entries);
    if count == 0 {
        return None;
    }
    let range: Box<dyn Iterator<Item = usize>> = if from_end {
        Box::new((0..count).rev())
    } else {
        Box::new(0..count)
    };
    range
        .into_iter()
        .find(|i| entry_at_level(entries, *i).is_some_and(|e| !entry_disabled(e)))
}

/// Moves the highlight path by `delta` (ArrowDown/ArrowUp), skipping disabled
/// entries and wrapping at the ends. An empty path starts at the first (or
/// last, for negative delta) entry of the current level.
pub(crate) fn move_highlight(entries: &[MenuEntry<'_>], path: &mut Vec<usize>, delta: isize) {
    if path.is_empty() {
        if let Some(idx) = edge_actionable(entries, delta < 0) {
            path.push(idx);
        }
        return;
    }
    // The level the last path element lives in: the root for single-element
    // paths, otherwise the items of the sub-menu referenced by the prefix.
    let level: &[MenuEntry<'_>] = match path.len() {
        1 => entries,
        _ => match resolve_action(entries, &path[..path.len() - 1]) {
            Some(MenuEntry {
                kind: EntryKind::Sub { items, .. },
            }) => items,
            _ => entries,
        },
    };
    let last = path.len() - 1;
    if let Some(next) = next_actionable(level, path[last], delta) {
        path[last] = next;
    }
}

/// First navigable index of a (sub-)menu level, for opening it with
/// ArrowRight / hover.
pub(crate) fn first_nav_child(entries: &[MenuEntry<'_>]) -> Option<usize> {
    edge_actionable(entries, false)
}

/// The activation id of an entry (explicit id or its label).
fn item_id(item: &MenuItem) -> String {
    item.id.clone().unwrap_or_else(|| item.label.clone())
}

/// Applies the side effects of activating the entry at `path`: toggles
/// checkbox items, stores the radio group selection, and returns the
/// activation id of the entry (sub triggers yield `None`).
pub(crate) fn apply_activation(entries: &mut [MenuEntry<'_>], path: &[usize]) -> Option<String> {
    let (first, rest) = path.split_first()?;
    let mut seen = 0usize;
    for entry in entries.iter_mut() {
        match &mut entry.kind {
            EntryKind::Item { item } => {
                if seen == *first && rest.is_empty() {
                    return Some(item_id(item));
                }
                seen += 1;
            }
            EntryKind::Checkbox { item, checked } => {
                if seen == *first && rest.is_empty() {
                    **checked = !**checked;
                    return Some(item_id(item));
                }
                seen += 1;
            }
            EntryKind::Sub { items, .. } => {
                if seen == *first {
                    return match rest.is_empty() {
                        true => None,
                        false => apply_activation(items, rest),
                    };
                }
                seen += 1;
            }
            EntryKind::RadioGroup { selection, items } => {
                if *first < seen + items.len() {
                    let radio_idx = *first - seen;
                    let id = match &items[radio_idx].kind {
                        EntryKind::Item { item } => item_id(item),
                        _ => return None,
                    };
                    **selection = Some(radio_idx);
                    return Some(id);
                }
                seen += items.len();
            }
            EntryKind::Label { .. } | EntryKind::Separator => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Row colors (pure, unit-tested)
// ---------------------------------------------------------------------------

/// Resolved colors for one menu row.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RowColors {
    pub fill: Color32,
    pub text: Color32,
    pub shortcut: Color32,
    pub icon: Color32,
    pub indicator: Color32,
}

/// Mirrors the source row recipes: `focus:bg-accent focus:text-accent-foreground`,
/// `data-[variant=destructive]:text-destructive focus:bg-destructive/10
/// dark:focus:bg-destructive/20`, `[&_svg]:text-muted-foreground`, shortcut
/// `text-muted-foreground`, disabled `opacity-50`.
pub(crate) fn resolve_row_colors(
    p: &Palette,
    dark: bool,
    highlighted: bool,
    destructive: bool,
    disabled: bool,
) -> RowColors {
    let mut c = RowColors {
        fill: Color32::TRANSPARENT,
        text: if destructive {
            p.destructive
        } else {
            p.popover_foreground
        },
        shortcut: p.muted_foreground,
        icon: if destructive {
            p.destructive
        } else {
            p.muted_foreground
        },
        indicator: if destructive {
            p.destructive
        } else {
            p.popover_foreground
        },
    };
    if highlighted {
        if destructive {
            c.fill = color_alpha(p.destructive, if dark { 0.2 } else { 0.1 });
            c.text = p.destructive;
            c.shortcut = p.destructive;
            c.icon = p.destructive;
            c.indicator = p.destructive;
        } else {
            c.fill = p.accent;
            c.text = p.accent_foreground;
            c.shortcut = p.accent_foreground;
            c.icon = p.accent_foreground;
            c.indicator = p.accent_foreground;
        }
    }
    if disabled {
        c.fill = color_alpha(c.fill, 0.5);
        c.text = color_alpha(c.text, 0.5);
        c.shortcut = color_alpha(c.shortcut, 0.5);
        c.icon = color_alpha(c.icon, 0.5);
        c.indicator = color_alpha(c.indicator, 0.5);
    }
    c
}

// ---------------------------------------------------------------------------
// Shared level rendering
// ---------------------------------------------------------------------------

/// Per-frame channels shared by all rendered levels of one menu.
pub(crate) struct LevelShared<'s> {
    pub p: Palette,
    pub dark: bool,
    /// Whether the root menu is open (rows and sub-menus are inert while the
    /// root fades out).
    pub root_open: bool,
    /// Current highlight path (read-only snapshot; hover writes go to
    /// `nav_update`).
    pub highlight: Vec<usize>,
    /// Hover / click-open navigation updates from this frame.
    pub nav_update: &'s mut Option<Vec<usize>>,
    /// Click activations from this frame.
    pub activation: &'s mut Option<Vec<usize>>,
    /// All rendered popup rects (for click-outside dismissal).
    pub popup_rects: &'s mut Vec<Rect>,
    pub spec: MenuSpec,
}

/// Renders the popup of one menu level and returns its response while
/// visible. `prefix` is the actionable path leading into this level (empty
/// for the root level).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_level(
    ui: &mut Ui,
    ctx: &Context,
    entries: &[MenuEntry<'_>],
    prefix: &[usize],
    popup_id: Id,
    anchor_rect: Rect,
    parent_layer: LayerId,
    align: RectAlign,
    gap: f32,
    shadow_lg: bool,
    open: bool,
    shared: &mut LevelShared<'_>,
) -> Option<Response> {
    let progress = overlay_progress(ctx, popup_id.with("anim"), open);
    if !(open || progress.alpha > 0.002) {
        return None;
    }

    let p = shared.p;
    let radius = ShadcnTheme::of_ui(ui).radius;
    let frame = egui::Frame::new()
        .fill(p.popover)
        .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
        .corner_radius(CornerRadius::same(radius.lg_u8()))
        .inner_margin(Margin::same(MENU_CONTENT_PADDING as i8))
        .shadow(if shadow_lg {
            shadow::shadow_lg(shared.dark)
        } else {
            shadow::shadow_md(shared.dark)
        });

    let popup = Popup::new(
        popup_id,
        ctx.clone(),
        PopupAnchor::ParentRect(anchor_rect),
        parent_layer,
    )
    .open(true)
    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
    .align(align)
    .gap(gap + (1.0 - progress.alpha) * 8.0)
    .width(shared.spec.min_width)
    .frame(egui::Frame::new());

    let inner = popup.show(|ui| {
        ui.set_opacity(progress.alpha);
        frame.show(ui, |ui| {
            ui.set_min_width(shared.spec.min_width - 2.0 * MENU_CONTENT_PADDING);
            render_rows(ui, entries, prefix, popup_id, shared);
        });
    });

    if let Some(ref r) = inner {
        shared.popup_rects.push(r.response.rect);
    }
    inner.map(|r| r.response)
}

/// Renders the rows of one level (items, labels, separators, sub-menus).
fn render_rows(
    ui: &mut Ui,
    entries: &[MenuEntry<'_>],
    prefix: &[usize],
    level_id: Id,
    shared: &mut LevelShared<'_>,
) {
    let mut action_idx = 0usize;
    let mut row_counter = 0usize;
    for entry in entries.iter() {
        match &entry.kind {
            EntryKind::Separator => {
                paint_separator(ui, shared.p);
            }
            EntryKind::Label { text, inset } => {
                paint_label(ui, text, *inset, shared);
            }
            EntryKind::Item { item } => {
                let path = push_path(prefix, action_idx);
                action_idx += 1;
                let colors = row_colors_for(shared, &path, item.disabled, item.destructive);
                let row_id = level_id.with(("shadcn_menu_row", row_counter));
                row_counter += 1;
                let response = paint_item_row(
                    ui,
                    row_id,
                    &RowPaint {
                        label: &item.label,
                        shortcut: item.shortcut.as_deref(),
                        icon: item.icon,
                        disabled: item.disabled,
                        inset: item.inset,
                        indicator: None,
                        colors,
                    },
                );
                handle_row_input(&response, &path, None, item.disabled, shared);
            }
            EntryKind::Checkbox { item, checked } => {
                let path = push_path(prefix, action_idx);
                action_idx += 1;
                let colors = row_colors_for(shared, &path, item.disabled, item.destructive);
                let checked_now = **checked;
                let row_id = level_id.with(("shadcn_menu_row", row_counter));
                row_counter += 1;
                let response = paint_item_row(
                    ui,
                    row_id,
                    &RowPaint {
                        label: &item.label,
                        shortcut: item.shortcut.as_deref(),
                        icon: None,
                        disabled: item.disabled,
                        inset: item.inset,
                        indicator: Some((shared.spec.indicator, checked_now, false)),
                        colors,
                    },
                );
                handle_row_input(&response, &path, None, item.disabled, shared);
            }
            EntryKind::RadioGroup { selection, items } => {
                let selected = **selection;
                for (radio_idx, radio) in items.iter().enumerate() {
                    let EntryKind::Item { item } = &radio.kind else {
                        continue;
                    };
                    let path = push_path(prefix, action_idx);
                    action_idx += 1;
                    let colors = row_colors_for(shared, &path, item.disabled, false);
                    let row_id = level_id.with(("shadcn_menu_row", row_counter));
                    row_counter += 1;
                    let response = paint_item_row(
                        ui,
                        row_id,
                        &RowPaint {
                            label: &item.label,
                            shortcut: item.shortcut.as_deref(),
                            icon: None,
                            disabled: item.disabled,
                            inset: item.inset,
                            indicator: Some((
                                shared.spec.indicator,
                                selected == Some(radio_idx),
                                true,
                            )),
                            colors,
                        },
                    );
                    handle_row_input(&response, &path, None, item.disabled, shared);
                }
            }
            EntryKind::Sub {
                label,
                items,
                disabled,
                inset,
            } => {
                let path = push_path(prefix, action_idx);
                action_idx += 1;
                let colors = row_colors_for(shared, &path, *disabled, false);
                let open_now =
                    shared.highlight.starts_with(&path) && shared.highlight.len() > path.len();
                let row_id = level_id.with(("shadcn_menu_row", row_counter));
                row_counter += 1;
                let response = paint_item_row(
                    ui,
                    row_id,
                    &RowPaint {
                        label,
                        shortcut: None,
                        icon: None,
                        disabled: *disabled,
                        inset: *inset,
                        indicator: None,
                        colors,
                    },
                );
                // Trailing chevron: `ChevronRightIcon class="ml-auto size-4"`.
                if ui.is_rect_visible(response.rect) {
                    let chev_rect = Rect::from_min_size(
                        Pos2::new(
                            response.rect.max.x - MENU_ROW_PAD_X - MENU_ICON_SIZE,
                            response.rect.center().y - MENU_ICON_SIZE * 0.5,
                        ),
                        Vec2::splat(MENU_ICON_SIZE),
                    );
                    let chev_color = if *disabled {
                        color_alpha(shared.p.muted_foreground, 0.5)
                    } else if open_now || colors.text == shared.p.accent_foreground {
                        shared.p.accent_foreground
                    } else {
                        shared.p.muted_foreground
                    };
                    Icon::ChevronRight.paint(
                        ui.painter(),
                        chev_rect,
                        chev_color,
                        2.0 * MENU_ICON_SIZE / 24.0,
                    );
                }
                let child = if *disabled {
                    None
                } else {
                    first_nav_child(items)
                };
                handle_row_input(&response, &path, child, *disabled, shared);

                // Child popup while the highlight path extends into this
                // sub-menu (a length > depth encodes "open").
                if open_now && child.is_some() && shared.root_open {
                    let sub_popup_id = level_id.with(("shadcn_menu_sub", action_idx - 1));
                    let parent_layer = ui.layer_id();
                    let sub_ctx = ui.ctx().clone();
                    let mut child_prefix = path.clone();
                    child_prefix.push(child.unwrap_or_default());
                    let _ = render_level(
                        ui,
                        &sub_ctx,
                        items,
                        &child_prefix,
                        sub_popup_id,
                        response.rect,
                        parent_layer,
                        RectAlign::RIGHT_START,
                        4.0,
                        true,
                        true,
                        shared,
                    );
                }
            }
        }
    }
}

fn push_path(prefix: &[usize], idx: usize) -> Vec<usize> {
    let mut path = prefix.to_vec();
    path.push(idx);
    path
}

fn row_colors_for(
    shared: &LevelShared<'_>,
    path: &[usize],
    disabled: bool,
    destructive: bool,
) -> RowColors {
    let highlighted = shared.highlight.starts_with(path);
    resolve_row_colors(&shared.p, shared.dark, highlighted, destructive, disabled)
}

/// Hover and click handling for one actionable row. Sub rows open their
/// child level on hover or click (Radix sub-trigger behavior); plain rows
/// move the highlight on hover and activate on click.
fn handle_row_input(
    response: &Response,
    path: &[usize],
    sub_child: Option<usize>,
    disabled: bool,
    shared: &mut LevelShared<'_>,
) {
    if disabled || !shared.root_open {
        return;
    }
    let open_now = shared.highlight.starts_with(path) && shared.highlight.len() > path.len();
    if let Some(child) = sub_child {
        if !open_now && (response.hovered() || response.clicked()) {
            let mut open_path = path.to_vec();
            open_path.push(child);
            *shared.nav_update = Some(open_path);
        }
    } else {
        if response.hovered() {
            *shared.nav_update = Some(path.to_vec());
        }
        if response.clicked() {
            *shared.activation = Some(path.to_vec());
        }
    }
}

// ---------------------------------------------------------------------------
// Row painters
// ---------------------------------------------------------------------------

/// Everything [`paint_item_row`] needs to draw one row.
struct RowPaint<'a> {
    label: &'a str,
    shortcut: Option<&'a str>,
    icon: Option<Icon>,
    disabled: bool,
    inset: bool,
    /// `(side, checked, is_radio)` for checkbox/radio rows.
    indicator: Option<(IndicatorSide, bool, bool)>,
    colors: RowColors,
}

fn paint_item_row(ui: &mut Ui, row_id: Id, row: &RowPaint<'_>) -> Response {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), MENU_ROW_HEIGHT));
    let sense = if row.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(rect, row_id, sense);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if row.colors.fill != Color32::TRANSPARENT {
            let r = ShadcnTheme::of_ui(ui).radius.md_u8();
            painter.rect_filled(rect, CornerRadius::same(r), row.colors.fill);
        }

        // Leading icon / left indicator / text, honoring `inset`.
        let mut text_x = if row.inset {
            rect.min.x + MENU_INSET_PAD
        } else {
            rect.min.x + MENU_ROW_PAD_X
        };
        if let Some(icon) = row.icon {
            let icon_rect = Rect::from_min_size(
                Pos2::new(text_x, rect.center().y - MENU_ICON_SIZE * 0.5),
                Vec2::splat(MENU_ICON_SIZE),
            );
            icon.paint(
                painter,
                icon_rect,
                row.colors.icon,
                2.0 * MENU_ICON_SIZE / 24.0,
            );
            text_x += MENU_ICON_SIZE + MENU_ICON_GAP;
        }
        if let Some((IndicatorSide::Left, checked, radio)) = row.indicator {
            paint_indicator(
                painter,
                rect,
                IndicatorSide::Left,
                checked,
                radio,
                row.colors.indicator,
            );
        }

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        painter.text(
            Pos2::new(text_x, rect.center().y),
            Align2::LEFT_CENTER,
            row.label,
            font,
            row.colors.text,
        );

        // Shortcut: `ml-auto text-xs tracking-widest text-muted-foreground`.
        let right_pad = match row.indicator {
            Some((IndicatorSide::Right, ..)) => MENU_INDICATOR_SPACE,
            _ => MENU_ROW_PAD_X,
        };
        if let Some(shortcut) = row.shortcut {
            let sfont = FontId::proportional(crate::theme::tokens::type_scale::XS);
            painter.text(
                Pos2::new(rect.max.x - right_pad, rect.center().y),
                Align2::RIGHT_CENTER,
                shortcut,
                sfont,
                row.colors.shortcut,
            );
        }

        // Checkbox/radio indicator on the right (dropdown/context).
        if let Some((side @ IndicatorSide::Right, checked, radio)) = row.indicator {
            paint_indicator(painter, rect, side, checked, radio, row.colors.indicator);
        }
    }
    response
}

fn paint_indicator(
    painter: &Painter,
    rect: Rect,
    side: IndicatorSide,
    checked: bool,
    radio: bool,
    color: Color32,
) {
    if !checked {
        return;
    }
    let center_y = rect.center().y;
    let center_x = match side {
        IndicatorSide::Right => rect.max.x - 8.0 - MENU_ICON_SIZE * 0.5, // right-2 + half icon
        IndicatorSide::Left => rect.min.x + 6.0 + MENU_ICON_SIZE * 0.5,  // left-1.5 + half icon
    };
    if radio {
        // `CircleIcon class="size-2 fill-current"` — an 8 px filled dot.
        painter.circle_filled(Pos2::new(center_x, center_y), 4.0, color);
    } else {
        // `CheckIcon class="size-4"`.
        let icon_rect =
            Rect::from_center_size(Pos2::new(center_x, center_y), Vec2::splat(MENU_ICON_SIZE));
        Icon::Check.paint(painter, icon_rect, color, 2.0 * MENU_ICON_SIZE / 24.0);
    }
}

fn paint_label(ui: &mut Ui, text: &str, inset: bool, shared: &LevelShared<'_>) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), MENU_LABEL_HEIGHT));
    if ui.is_rect_visible(rect) {
        let (color, size) = if shared.spec.label_muted {
            (
                shared.p.muted_foreground,
                crate::theme::tokens::type_scale::XS,
            )
        } else {
            (
                shared.p.popover_foreground,
                crate::theme::tokens::type_scale::SM,
            )
        };
        let font = FontId::proportional(size);
        let x = if inset {
            rect.min.x + MENU_INSET_PAD
        } else {
            rect.min.x + MENU_ROW_PAD_X
        };
        ui.painter().text(
            Pos2::new(x, rect.center().y),
            Align2::LEFT_CENTER,
            text,
            font,
            color,
        );
    }
}

fn paint_separator(ui: &mut Ui, p: Palette) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), MENU_SEPARATOR_HEIGHT));
    if ui.is_rect_visible(rect) {
        // `-mx-1` — the line bleeds 4 px into the content padding on both sides.
        let y = rect.center().y;
        ui.painter().line_segment(
            [
                Pos2::new(rect.min.x - MENU_CONTENT_PADDING, y),
                Pos2::new(rect.max.x + MENU_CONTENT_PADDING, y),
            ],
            Stroke::new(1.0, p.border),
        );
    }
}

// ---------------------------------------------------------------------------
// Alignment
// ---------------------------------------------------------------------------

/// Where the content aligns against the trigger (`align` in the source).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuAlign {
    /// Aligned to the trigger's left edge.
    Start,
    /// Centered on the trigger.
    #[default]
    Center,
    /// Aligned to the trigger's right edge.
    End,
}

impl MenuAlign {
    /// The egui rect alignment for a bottom-anchored popup.
    pub fn rect_align(self) -> RectAlign {
        match self {
            MenuAlign::Start => RectAlign::BOTTOM_START,
            MenuAlign::Center => RectAlign::BOTTOM,
            MenuAlign::End => RectAlign::BOTTOM_END,
        }
    }
}

// ---------------------------------------------------------------------------
// Root state
// ---------------------------------------------------------------------------

/// The content popup side offset (`sideOffset = 4`).
pub const DROPDOWN_SIDE_OFFSET: f32 = 4.0;

/// Per-instance state: open flag, trigger geometry and the highlight path.
#[derive(Clone, Debug, Default)]
pub(crate) struct MenuState {
    pub open: bool,
    pub trigger: Option<TriggerInfo>,
    /// Highlight path; a length > depth encodes open sub-menus.
    pub highlight: Vec<usize>,
}

/// Trigger geometry captured the frame the trigger was drawn.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerInfo {
    pub rect: Rect,
    pub layer_id: LayerId,
}

pub(crate) fn read_menu_state(ctx: &Context, state_id: Id) -> MenuState {
    ctx.data_mut(|d| d.get_temp::<MenuState>(state_id).unwrap_or_default())
}

pub(crate) fn write_menu_state(ctx: &Context, state_id: Id, state: MenuState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

/// Root-level keyboard handling shared by dropdown, context-menu and
/// menubar: ArrowUp/Down/Right/Left, Home/End, Enter/Space. Escape is handled
/// by the caller (it closes the deepest level first).
pub(crate) fn handle_menu_keys(
    ctx: &Context,
    entries: &[MenuEntry<'_>],
    state: &mut MenuState,
    activation: &mut Option<Vec<usize>>,
) {
    let (down, up, left, right, home, end, enter, space) = ctx.input(|i| {
        (
            i.key_pressed(Key::ArrowDown),
            i.key_pressed(Key::ArrowUp),
            i.key_pressed(Key::ArrowLeft),
            i.key_pressed(Key::ArrowRight),
            i.key_pressed(Key::Home),
            i.key_pressed(Key::End),
            i.key_pressed(Key::Enter),
            i.key_pressed(Key::Space),
        )
    });
    if down {
        move_highlight(entries, &mut state.highlight, 1);
    }
    if up {
        move_highlight(entries, &mut state.highlight, -1);
    }
    if home || end {
        if let Some(idx) = deepest_level_edge(entries, &state.highlight, end) {
            if state.highlight.is_empty() {
                state.highlight.push(idx);
            } else {
                let last = state.highlight.len() - 1;
                state.highlight[last] = idx;
            }
        }
    }
    if right {
        open_highlighted_sub(entries, &mut state.highlight);
    }
    if left && state.highlight.len() > 1 {
        state.highlight.pop();
    }
    if enter || space {
        let activates = open_highlighted_sub(entries, &mut state.highlight).is_none()
            && !state.highlight.is_empty()
            && resolve_action(entries, &state.highlight)
                .is_some_and(|entry| !entry_disabled(entry));
        if activates {
            *activation = Some(state.highlight.clone());
        }
    }
}

/// First/last navigable index of the level the highlight path currently
/// points into (for Home/End).
fn deepest_level_edge(entries: &[MenuEntry<'_>], path: &[usize], from_end: bool) -> Option<usize> {
    let level: &[MenuEntry<'_>] = match path.len() {
        0 => entries,
        _ => match resolve_action(entries, &path[..path.len() - 1]) {
            Some(MenuEntry {
                kind: EntryKind::Sub { items, .. },
            }) => items,
            _ => entries,
        },
    };
    edge_actionable(level, from_end)
}

/// Opens the highlighted entry if it is an enabled sub-menu; returns the
/// pushed child index.
fn open_highlighted_sub(entries: &[MenuEntry<'_>], highlight: &mut Vec<usize>) -> Option<usize> {
    let entry = resolve_action(entries, highlight)?;
    if let MenuEntry {
        kind:
            EntryKind::Sub {
                items,
                disabled: false,
                ..
            },
    } = entry
    {
        let child = first_nav_child(items)?;
        highlight.push(child);
        Some(child)
    } else {
        None
    }
}

/// Escape handling shared by all three menu components: pop one level when a
/// sub-menu is open, close the root otherwise. Returns `true` when Escape
/// was consumed.
pub(crate) fn handle_menu_escape(ctx: &Context, state: &mut MenuState) -> bool {
    let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
    let esc = !modal_open && ctx.input(|i| i.key_pressed(Key::Escape));
    if !esc {
        return false;
    }
    if state.highlight.len() > 1 {
        state.highlight.pop();
    } else {
        state.open = false;
        state.highlight.clear();
    }
    true
}

/// Click-outside dismissal shared by all three menu components: a click that
/// lands outside every popup rect and outside `extra` (e.g. the trigger)
/// closes the menu. Only runs when the popup was visible last frame.
pub(crate) fn handle_outside_click(
    ctx: &Context,
    popup_rects: &[Rect],
    extra: &[Rect],
    was_open_last_frame: bool,
) -> bool {
    if !was_open_last_frame {
        return false;
    }
    let any_click = ctx.input(|i| i.pointer.any_click());
    if !any_click {
        return false;
    }
    let pos = ctx.input(|i| i.pointer.latest_pos());
    let Some(pos) = pos else {
        return false;
    };
    let inside = popup_rects
        .iter()
        .chain(extra.iter())
        .any(|r| r.contains(pos));
    !inside
}

// ---------------------------------------------------------------------------
// DropdownMenu (root)
// ---------------------------------------------------------------------------

/// A custom trigger widget (`DropdownMenuTrigger asChild`): returns the
/// widget's response for anchoring and toggling.
pub type TriggerWidget<'a> = Box<dyn FnOnce(&mut Ui) -> Response + 'a>;

/// What [`DropdownMenu::show`] returns.
#[derive(Clone, Debug)]
pub struct DropdownMenuResponse {
    /// The trigger widget's response.
    pub trigger: Response,
    /// Whether the menu is open after this frame.
    pub open: bool,
    /// The item activated this frame (click or keyboard), if any.
    pub activated: Option<MenuActivated>,
    /// True when the menu was open at frame start and is closed now.
    pub closed: bool,
}

impl DropdownMenuResponse {
    /// Whether the item with the given id (or label) was activated this frame.
    pub fn clicked(&self, id: &str) -> bool {
        self.activated.as_ref().is_some_and(|a| a.id == id)
    }
}

/// One activated menu item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuActivated {
    /// Activation id (explicit `id` or the item label).
    pub id: String,
    /// Actionable path from the root level to this item.
    pub path: Vec<usize>,
}

/// The shadcn dropdown menu. See the [module docs](self) for an example.
///
/// The trigger renders as an outline button (the shadcn docs demos use
/// `<Button variant="outline">`); override it with
/// [`DropdownMenu::trigger_widget`]. Uncontrolled by default — use
/// [`DropdownMenu::controlled`] to own the open state.
pub struct DropdownMenu<'a> {
    label: Option<String>,
    trigger_widget: Option<TriggerWidget<'a>>,
    controlled: Option<&'a mut bool>,
    align: MenuAlign,
    width: Option<f32>,
    side_offset: Option<f32>,
    close_on_select: bool,
    id_salt: Option<String>,
}

impl Default for DropdownMenu<'_> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> DropdownMenu<'a> {
    /// Creates an uncontrolled menu with an outline-button trigger.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            trigger_widget: None,
            controlled: None,
            align: MenuAlign::default(),
            width: None,
            side_offset: None,
            close_on_select: true,
            id_salt: None,
        }
    }

    /// Creates a menu whose open state is owned by the caller.
    pub fn controlled(label: impl Into<String>, open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            ..Self::new(label)
        }
    }

    /// Replaces the trigger with a custom widget (the Radix `asChild`
    /// pattern).
    pub fn trigger_widget(mut self, w: impl FnOnce(&mut Ui) -> Response + 'a) -> Self {
        self.trigger_widget = Some(Box::new(w) as TriggerWidget<'a>);
        self
    }

    /// Sets the horizontal alignment against the trigger (default `Center`;
    /// the shadcn docs demos use `Start`).
    pub fn align(mut self, align: MenuAlign) -> Self {
        self.align = align;
        self
    }

    /// Sets an explicit content width (default: [`DROPDOWN_MIN_WIDTH`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides the trigger-to-content gap (default: [`DROPDOWN_SIDE_OFFSET`]).
    pub fn side_offset(mut self, offset: f32) -> Self {
        self.side_offset = Some(offset);
        self
    }

    /// Whether activating an item closes the menu (Radix default: `true`).
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    /// Sets a stable id salt so open state and focus persist per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the trigger, and the menu popup when open.
    pub fn show(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut MenuBuilder<'a>),
    ) -> DropdownMenuResponse {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_dropdown_menu"),
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

        // Build the item tree.
        let mut builder = MenuBuilder::new();
        content(&mut builder);
        let mut entries = builder.entries;

        // Trigger.
        let trigger_response = match self.trigger_widget {
            Some(w) => w(ui),
            None => crate::components::button::Button::new(self.label.clone().unwrap_or_default())
                .variant(crate::components::button::ButtonVariant::Outline)
                .id_salt(state_id.value().to_string())
                .show(ui),
        };
        if trigger_response.clicked() {
            state.open = !state.open;
            if state.open {
                state.highlight.clear();
            }
            state.trigger = Some(TriggerInfo {
                rect: trigger_response.rect,
                layer_id: trigger_response.layer_id,
            });
        }

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();

        let mut nav_update: Option<Vec<usize>> = None;
        let mut activation: Option<Vec<usize>> = None;
        let mut popup_rects: Vec<Rect> = Vec::new();

        // Keyboard navigation while open.
        if state.open {
            handle_menu_keys(&ctx, &entries, &mut state, &mut activation);
            handle_menu_escape(&ctx, &mut state);
        }

        // Popup (kept up during the close fade like the source's
        // `data-[state=closed]:animate-out`).
        let popup_id = state_id.with("popup");
        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        if state.open || was_open_last_frame {
            let spec = MenuSpec {
                min_width: self.width.unwrap_or(DROPDOWN_MIN_WIDTH),
                indicator: IndicatorSide::Right,
                label_muted: true,
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
                    self.align.rect_align(),
                    self.side_offset.unwrap_or(DROPDOWN_SIDE_OFFSET),
                    false,
                    state.open,
                    &mut shared,
                );
            }
        }

        // Dismissal: click outside every popup and the trigger.
        let trigger_rect = state.trigger.map(|t| t.rect).unwrap_or(Rect::NOTHING);
        if state.open
            && handle_outside_click(&ctx, &popup_rects, &[trigger_rect], was_open_last_frame)
        {
            state.open = false;
        }

        // Apply navigation updates from hover.
        if let Some(path) = nav_update.take() {
            state.highlight = path;
        }

        // Apply activation side effects (only while genuinely open — not
        // during the close fade).
        let mut activated = None;
        if state.open {
            if let Some(path) = activation.take() {
                let id = apply_activation(&mut entries, &path);
                if let Some(id) = id {
                    activated = Some(MenuActivated { id, path });
                }
                if self.close_on_select {
                    state.open = false;
                }
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

        DropdownMenuResponse {
            trigger: trigger_response,
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
    use crate::theme::{Mode, Preset};

    /// A representative menu: label, items, separator, disabled item,
    /// sub-menu and radio group. `checked` and `selection` are leaked so the
    /// static tree can hold them.
    fn entries() -> Vec<MenuEntry<'static>> {
        let selection: &'static mut Option<usize> = Box::leak(Box::new(None));
        let mut b = MenuBuilder::new();
        b.label("My Account");
        b.item(MenuItem::new("Profile").shortcut("⇧⌘P"));
        b.item(MenuItem::new("Billing"));
        b.separator();
        b.item(MenuItem::new("API").disabled(true));
        b.sub_menu("Invite users", |m| {
            m.item(MenuItem::new("Email"));
            m.item(MenuItem::new("Message").disabled(true));
            m.item(MenuItem::new("More"));
        });
        b.radio_group(selection, |g| {
            g.radio_labels(["Light", "Dark"]);
        });
        b.entries
    }

    #[test]
    fn counts_actionable_entries() {
        let e = entries();
        // Profile, Billing, API, Sub, Light, Dark = 6; label + separator skipped.
        assert_eq!(count_actionable(&e), 6);
    }

    #[test]
    fn resolves_paths_through_submenus_and_radio_groups() {
        let e = entries();
        assert!(matches!(
            resolve_action(&e, &[1]).unwrap().kind,
            EntryKind::Item { .. }
        ));
        assert!(entry_disabled(resolve_action(&e, &[2]).unwrap()));
        assert!(matches!(
            resolve_action(&e, &[3]).unwrap().kind,
            EntryKind::Sub { .. }
        ));
        match &resolve_action(&e, &[3, 2]).unwrap().kind {
            EntryKind::Item { item } => assert_eq!(item.label, "More"),
            other => panic!("expected item, got {other:?}"),
        }
        match &resolve_action(&e, &[4]).unwrap().kind {
            EntryKind::Item { item } => assert_eq!(item.label, "Light"),
            other => panic!("expected radio item, got {other:?}"),
        }
    }

    #[test]
    fn move_highlight_skips_disabled_and_wraps() {
        let e = entries();
        let mut path = Vec::new();
        move_highlight(&e, &mut path, 1);
        assert_eq!(path, vec![0], "starts at the first entry");
        move_highlight(&e, &mut path, 1);
        assert_eq!(path, vec![1]);
        move_highlight(&e, &mut path, 1);
        assert_eq!(path, vec![3], "skips the disabled API (2)");
        move_highlight(&e, &mut path, -1);
        assert_eq!(path, vec![1], "backward also skips disabled");
        let mut path = vec![5];
        move_highlight(&e, &mut path, 1);
        assert_eq!(path, vec![0], "wraps forward to the start");
        let mut path = vec![0];
        move_highlight(&e, &mut path, -1);
        assert_eq!(path, vec![5], "wraps backward to the end");
    }

    #[test]
    fn move_highlight_navigates_inside_open_submenu() {
        let e = entries();
        let mut path = vec![3];
        assert!(open_highlighted_sub(&e, &mut path).is_some());
        assert_eq!(path, vec![3, 0], "sub-menu opens on its first child");
        // Email (0) -> Message is disabled (1) -> More (2).
        move_highlight(&e, &mut path, 1);
        assert_eq!(path, vec![3, 2]);
        move_highlight(&e, &mut path, 1);
        assert_eq!(path, vec![3, 0], "wraps inside the sub-menu");
        // ArrowLeft closes it.
        path.pop();
        assert_eq!(path, vec![3]);
    }

    #[test]
    fn empty_menus_do_not_panic() {
        let e: Vec<MenuEntry<'static>> = Vec::new();
        let mut path = Vec::new();
        move_highlight(&e, &mut path, 1);
        assert!(path.is_empty());
        assert!(open_highlighted_sub(&e, &mut path).is_none());
        assert!(resolve_action(&e, &[0]).is_none());
        assert!(edge_actionable(&e, false).is_none());
    }

    #[test]
    fn activation_toggles_checkboxes_and_sets_radio_selection() {
        let mut checked = false;
        let mut b = MenuBuilder::new();
        b.checkbox_item(CheckboxItem::new("Status Bar", &mut checked).id("status"));
        let mut e = b.entries;
        let checked_state = |e: &[MenuEntry<'_>]| match &resolve_action(e, &[0]).unwrap().kind {
            EntryKind::Checkbox { checked, .. } => **checked,
            _ => panic!("expected checkbox"),
        };
        assert_eq!(apply_activation(&mut e, &[0]).as_deref(), Some("status"));
        assert!(checked_state(&e), "checkbox toggled on");
        assert_eq!(apply_activation(&mut e, &[0]).as_deref(), Some("status"));
        assert!(!checked_state(&e), "checkbox toggled off");

        let selection: &'static mut Option<usize> = Box::leak(Box::new(None));
        let mut b = MenuBuilder::new();
        b.radio_group(selection, |g| g.radio_labels(["Light", "Dark"]));
        let mut e = b.entries;
        assert_eq!(apply_activation(&mut e, &[1]).as_deref(), Some("Dark"));
        assert_eq!(*selection, Some(1));

        let mut b = MenuBuilder::new();
        b.sub_menu("Share", |m| m.item(MenuItem::new("Email").id("mail")));
        let mut e = b.entries;
        assert_eq!(
            apply_activation(&mut e, &[0]).as_deref(),
            None,
            "sub triggers do not activate"
        );
        assert_eq!(apply_activation(&mut e, &[0, 0]).as_deref(), Some("mail"));
    }

    #[test]
    fn row_colors_follow_state() {
        let p = Preset::default().build(Mode::Light).palette;
        let c = resolve_row_colors(&p, false, false, false, false);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.text, p.popover_foreground);
        assert_eq!(c.shortcut, p.muted_foreground);

        let h = resolve_row_colors(&p, false, true, false, false);
        assert_eq!(h.fill, p.accent);
        assert_eq!(h.text, p.accent_foreground);
        assert_eq!(h.icon, p.accent_foreground);

        let d = resolve_row_colors(&p, false, false, true, false);
        assert_eq!(d.text, p.destructive);
        let hd = resolve_row_colors(&p, false, true, true, false);
        assert_eq!(hd.fill, color_alpha(p.destructive, 0.1));
        assert_eq!(hd.text, p.destructive);
        let hdd = resolve_row_colors(&p, true, true, true, false);
        assert_eq!(hdd.fill, color_alpha(p.destructive, 0.2), "dark mode /20");

        let dis = resolve_row_colors(&p, false, false, false, true);
        assert_eq!(
            dis.text,
            color_alpha(p.popover_foreground, 0.5),
            "opacity-50"
        );
    }

    #[test]
    fn geometry_constants_match_source() {
        assert_eq!(DROPDOWN_MIN_WIDTH, 128.0, "min-w-32");
        assert_eq!(SUB_MENU_MIN_WIDTH, 96.0, "min-w-[96px]");
        assert_eq!(DROPDOWN_SIDE_OFFSET, 4.0, "sideOffset=4");
        assert_eq!(MENU_ROW_HEIGHT, 28.0, "py-1 + text-sm");
        assert_eq!(MENU_INSET_PAD, 28.0, "pl-7");
        assert_eq!(MENU_INDICATOR_SPACE, 32.0, "pr-8");
    }

    #[test]
    fn align_maps_to_rect_aligns() {
        assert_eq!(MenuAlign::Start.rect_align(), RectAlign::BOTTOM_START);
        assert_eq!(MenuAlign::Center.rect_align(), RectAlign::BOTTOM);
        assert_eq!(MenuAlign::End.rect_align(), RectAlign::BOTTOM_END);
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let mut checked = true;
        let menu = DropdownMenu::controlled("Open", &mut open)
            .align(MenuAlign::End)
            .width(240.0)
            .side_offset(8.0)
            .close_on_select(false)
            .id_salt("m1");
        assert!(!menu.close_on_select);
        assert_eq!(menu.width, Some(240.0));
        assert_eq!(menu.side_offset, Some(8.0));
        assert_eq!(menu.align, MenuAlign::End);

        let item = MenuItem::new("X")
            .id("x")
            .shortcut("⌘X")
            .icon(Icon::Copy)
            .disabled(true)
            .destructive(true)
            .inset(true);
        assert_eq!(item.label, "X");
        assert!(item.disabled && item.destructive && item.inset);

        let cb = CheckboxItem::new("C", &mut checked).disabled(true);
        assert!(cb.item.disabled);
        let ri = RadioItem::new("R").shortcut("⌘R");
        assert_eq!(ri.item.shortcut.as_deref(), Some("⌘R"));

        let mut b = MenuBuilder::new();
        b.item(MenuItem::new("A"));
        b.checkbox_item(CheckboxItem::new("B", &mut checked));
        b.sub_menu("S", |m| m.item(MenuItem::new("S1")));
        b.label("L");
        b.label_inset("LI");
        b.separator();
        assert_eq!(b.entries.len(), 6);
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
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
                    Id::unique("dropdown_test_ui"),
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
                        Id::unique("dropdown_test_ui"),
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
                    Id::unique("dropdown_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> MenuState {
            let id = Id::unique("dropdown_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<MenuState>(id))
                .unwrap_or_default()
        }
    }

    fn layout(h: &mut Harness, salt: &str) -> Rect {
        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            DropdownMenu::new("Open")
                .id_salt(salt)
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        // Re-run once to grab the stored trigger rect (the first frame may
        // not have laid out the button yet).
        h.run(|ui| {
            trigger_rect = DropdownMenu::new("Open")
                .id_salt(salt)
                .show(ui, |m| m.item(MenuItem::new("Only item")))
                .trigger
                .rect;
        });
        trigger_rect
    }

    #[test]
    fn trigger_click_toggles_open_state() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "t1");
        assert!(!h.state("t1").open, "default is closed");

        h.click(rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Open")
                .id_salt("t1")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(h.state("t1").open, "clicking the trigger opens it");

        h.click(rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Open")
                .id_salt("t1")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(!h.state("t1").open, "clicking again closes it");
    }

    #[test]
    fn click_outside_closes_open_menu() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "out");
        h.click(rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Open")
                .id_salt("out")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(h.state("out").open);

        // One more frame so the popup was visible last frame, then click far away.
        h.run(|ui| {
            DropdownMenu::new("Open")
                .id_salt("out")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        h.click(
            Pos2::new(550.0, 550.0),
            egui::PointerButton::Primary,
            |ui| {
                DropdownMenu::new("Open")
                    .id_salt("out")
                    .show(ui, |m| m.item(MenuItem::new("Only item")));
            },
        );
        assert!(!h.state("out").open, "clicking outside must close it");
    }

    #[test]
    fn escape_closes_open_menu() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "esc");
        h.click(rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Open")
                .id_salt("esc")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(h.state("esc").open);
        h.run(|ui| {
            DropdownMenu::new("Open")
                .id_salt("esc")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        h.key(Key::Escape, |ui| {
            DropdownMenu::new("Open")
                .id_salt("esc")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(!h.state("esc").open, "Escape must close the menu");
    }

    #[test]
    fn clicking_an_item_activates_it_and_closes() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "act");
        h.click(rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Open")
                .id_salt("act")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(h.state("act").open);

        // Let the popup settle one frame (egui hit-tests against the
        // previous pass's widget rects; a popup's first pass is sizing-only).
        h.run(|ui| {
            DropdownMenu::new("Open")
                .id_salt("act")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });

        // Click the item row (the popup opens below the trigger).
        let popup_pos = Pos2::new(rect.center().x, rect.max.y + 20.0);
        let ctx = h.ctx.clone();
        h.click(popup_pos, egui::PointerButton::Primary, |ui| {
            let res = DropdownMenu::new("Open")
                .id_salt("act")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
            if res.clicked("Only item") {
                // Record through the shared state store.
                ctx.data_mut(|d| d.insert_temp(Id::unique("act_hit"), true));
            }
        });
        let hit: bool = h
            .ctx
            .data_mut(|d| d.get_temp(Id::unique("act_hit")).unwrap_or(false));
        assert!(hit, "clicking the item must activate it");
        assert!(!h.state("act").open, "activation closes the menu");
    }

    #[test]
    fn keyboard_activates_first_item() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "kb");
        h.click(rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Open")
                .id_salt("kb")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(h.state("kb").open);

        h.key(Key::ArrowDown, |ui| {
            DropdownMenu::new("Open")
                .id_salt("kb")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert_eq!(
            h.state("kb").highlight,
            vec![0],
            "ArrowDown highlights the first item"
        );

        let ctx = h.ctx.clone();
        h.key(Key::Enter, |ui| {
            let res = DropdownMenu::new("Open")
                .id_salt("kb")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
            if res.clicked("Only item") {
                ctx.data_mut(|d| d.insert_temp(Id::unique("kb_hit"), true));
            }
        });
        let hit: bool = h
            .ctx
            .data_mut(|d| d.get_temp(Id::unique("kb_hit")).unwrap_or(false));
        assert!(hit, "Enter activates the highlighted item");
        assert!(!h.state("kb").open, "activation closes the menu");
    }

    #[test]
    fn controlled_mode_write_back() {
        let mut h = Harness::new();
        let mut open = false;
        let mut trigger_rect = Rect::ZERO;

        h.run(|ui| {
            trigger_rect = DropdownMenu::controlled("Open", &mut open)
                .id_salt("c1")
                .show(ui, |m| m.item(MenuItem::new("Only item")))
                .trigger
                .rect;
        });
        assert!(!open);

        h.click(trigger_rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::controlled("Open", &mut open)
                .id_salt("c1")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(open, "trigger click write-back reaches the caller");

        // The caller closes it from the outside.
        open = false;
        h.run(|ui| {
            DropdownMenu::controlled("Open", &mut open)
                .id_salt("c1")
                .show(ui, |m| m.item(MenuItem::new("Only item")));
        });
        assert!(!open);
    }

    #[test]
    fn checkbox_toggle_through_ui() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "cb2");
        let mut checked = false;
        h.click(rect.center(), egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Checkboxes")
                .id_salt("cb2")
                .width(224.0)
                .show(ui, |m| {
                    m.checkbox_item(CheckboxItem::new("Status Bar", &mut checked));
                });
        });
        assert!(h.state("cb2").open);

        // Let the popup settle one frame (see clicking_an_item test).
        h.run(|ui| {
            DropdownMenu::new("Checkboxes")
                .id_salt("cb2")
                .width(224.0)
                .show(ui, |m| {
                    m.checkbox_item(CheckboxItem::new("Status Bar", &mut checked));
                });
        });

        let popup_pos = Pos2::new(rect.center().x, rect.max.y + 20.0);
        h.click(popup_pos, egui::PointerButton::Primary, |ui| {
            DropdownMenu::new("Checkboxes")
                .id_salt("cb2")
                .width(224.0)
                .show(ui, |m| {
                    m.checkbox_item(CheckboxItem::new("Status Bar", &mut checked));
                });
        });
        assert!(checked, "clicking the checkbox row toggles the flag");
    }
}
