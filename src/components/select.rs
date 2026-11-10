//! shadcn `select.tsx` — a fully keyboard-navigable single-value select.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/select.tsx` (Radix Select)
//! with the nova recipe (`style-nova.css .cn-select-*`): an `h-8 rounded-lg
//! border border-input bg-transparent pl-2.5 pr-2 text-sm shadow-xs`
//! trigger (`data-[size=sm]:h-7` with `rounded-[min(var(--radius-md),10px)]`)
//! with a full-opacity `size-4 text-muted-foreground` chevron-down,
//! `data-[placeholder]:text-muted-foreground`,
//! `dark:bg-input/30 dark:hover:bg-input/50` and
//! `dark:aria-invalid:border-destructive/50`, and a `min-w-36 rounded-lg
//! ring-1 ring-foreground/10 shadow-md bg-popover p-1` content popup whose
//! items follow `rounded-md py-1 pr-8 pl-1.5 text-sm focus:bg-accent
//! focus:text-accent-foreground data-[disabled]:opacity-50` (28 px rows)
//! with the check indicator at `right-2`. Labels are `px-1.5 py-1 text-xs
//! text-muted-foreground`, separators are `-mx-1 my-1 h-px bg-border`.
//!
//! The value is controlled through `&mut Option<String>` or `&mut String`;
//! pass neither and use [`Select::uncontrolled`] to keep the selection inside
//! the egui state (read it from [`SelectResponse::value`]). The popup
//! supports labels/separators/disabled options, a scrolling list, and
//! Radix-style keyboard behavior: the trigger opens on click, Enter, Space
//! or ArrowUp/Down while focused; ArrowUp/Down move the highlight (skipping
//! disabled options, wrapping), Home/End jump, Enter/Space select and close,
//! Tab and Escape dismiss, and typing performs the Radix typeahead —
//! jumping the highlight to matching options while open and selecting them
//! directly on the focused trigger while closed. Opening highlights the
//! current selection.
//!
//! Known gaps vs the source (no egui equivalent yet): the
//! `SelectScrollUp/DownButton` hover auto-repeat rows (the list relies on
//! the scroll area) and Radix's `position="item-aligned"` popup alignment.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut framework = Some("Next.js".to_string());
//!
//! let res = Select::new(&mut framework)
//!     .placeholder("Select a framework")
//!     .id_salt("framework")
//!     .size(SelectSize::Default)
//!     .show(ui, |s| {
//!         s.label("Frameworks");
//!         s.item(SelectItem::new("Next.js"));
//!         s.item(SelectItem::new("SvelteKit"));
//!         s.item(SelectItem::new("Astro").disabled(true));
//!         s.separator();
//!         s.item(SelectItem::new("Other").value("other"));
//!     });
//! if res.changed() { /* res.value was updated */ }
//!
//! // Bound to a plain String (empty = placeholder) or uncontrolled:
//! let mut fruit = String::new();
//! Select::bind(&mut fruit).option("Apple").option("Pear").show_seeded(ui);
//! Select::uncontrolled().id_salt("solo").option("A").show_seeded(ui);
//! ```

use crate::components::direction;
use crate::components::dropdown_menu::{TypeaheadState, MENU_TYPEAHEAD_TIMEOUT_SECS};
use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use crate::utils::animation::nova_overlay_progress;
use crate::utils::shadow;
use egui::emath::RectAlign;
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Key, LayerId, Margin, Pos2, Rect, Response,
    Sense, Stroke, Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Geometry constants
// ---------------------------------------------------------------------------

/// Content min width: nova `min-w-36` (144 px).
pub const SELECT_MIN_WIDTH: f32 = 144.0;
/// Trigger height: nova `data-[size=default]:h-8` (32 px).
pub const SELECT_TRIGGER_HEIGHT_DEFAULT: f32 = 32.0;
/// Small trigger height: nova `data-[size=sm]:h-7` (28 px).
pub const SELECT_TRIGGER_HEIGHT_SM: f32 = 28.0;
/// Trigger text padding: nova `pl-2.5` (10 px).
const TRIGGER_PAD_X: f32 = 10.0;
/// Trigger right padding: nova `pr-2` (8 px).
const TRIGGER_PAD_RIGHT: f32 = 8.0;
/// Gap between the value and the chevron: nova `gap-1.5` (6 px).
const TRIGGER_ICON_GAP: f32 = 6.0;
/// Content padding: `p-1` (4 px).
const CONTENT_PADDING: f32 = 4.0;
/// Item row height: nova `py-1` + one `text-sm` line (28 px).
const ITEM_HEIGHT: f32 = 28.0;
/// Item horizontal padding: nova `pl-1.5` (6 px).
const ITEM_PAD_X: f32 = 6.0;
/// Check/chevron icon size: `size-4` (16 px).
const ICON_SIZE: f32 = 16.0;
/// Label row height: nova `py-1` + one `text-xs` line (20 px).
const LABEL_HEIGHT: f32 = 20.0;
/// Separator total height: `my-1` top and bottom + 1 px line.
const SEPARATOR_HEIGHT: f32 = 9.0;
/// Popup max viewport height (clamped further to the screen space around
/// the trigger; the source clamps to
/// `--radix-select-content-available-height`).
const MAX_CONTENT_HEIGHT: f32 = 300.0;
/// Margin kept between the popup and the window edge when deriving the
/// available height.
const AVAILABLE_SLACK: f32 = 16.0;

// ---------------------------------------------------------------------------
// Items and entries
// ---------------------------------------------------------------------------

/// One selectable option (`SelectItem`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectItem {
    /// The value written into the bound state when picked.
    pub value: String,
    /// The visible label (defaults to the value).
    pub label: String,
    /// `data-[disabled]` — not selectable, skipped by keyboard navigation.
    pub disabled: bool,
}

impl SelectItem {
    /// Creates an enabled option whose value is its label.
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            value: label.clone(),
            label,
            disabled: false,
        }
    }

    /// Overrides the value written into the bound state (`value="…"`).
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Disables the option.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// One row of the select content (item, label or separator).
#[derive(Clone, Debug)]
pub(crate) enum SelectEntry {
    Item(SelectItem),
    Label(String),
    Separator,
}

/// Collects the content rows inside the [`Select::show`] closure.
pub struct SelectBuilder {
    pub(crate) entries: Vec<SelectEntry>,
}

impl Default for SelectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Appends an enabled option.
    pub fn item(&mut self, item: SelectItem) {
        self.entries.push(SelectEntry::Item(item));
    }

    /// Appends several enabled options from labels.
    pub fn items(&mut self, labels: impl IntoIterator<Item = impl Into<String>>) {
        for label in labels {
            self.item(SelectItem::new(label));
        }
    }

    /// Appends a disabled option.
    pub fn disabled_item(&mut self, label: impl Into<String>) {
        self.entries
            .push(SelectEntry::Item(SelectItem::new(label).disabled(true)));
    }

    /// Appends a `SelectLabel` section header.
    pub fn label(&mut self, text: impl Into<String>) {
        self.entries.push(SelectEntry::Label(text.into()));
    }

    /// Appends a `SelectSeparator`.
    pub fn separator(&mut self) {
        self.entries.push(SelectEntry::Separator);
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Number of keyboard-navigable entries (items; labels/separators skipped).
pub(crate) fn count_actionable(entries: &[SelectEntry]) -> usize {
    entries
        .iter()
        .filter(|e| matches!(e, SelectEntry::Item(_)))
        .count()
}

/// The entry index (into `entries`) selected by `action_idx`, or `None` when
/// the index falls on a label/separator or out of bounds.
pub(crate) fn item_index_at(entries: &[SelectEntry], action_idx: usize) -> Option<usize> {
    let mut seen = 0usize;
    for (i, entry) in entries.iter().enumerate() {
        if let SelectEntry::Item(_) = entry {
            if seen == action_idx {
                return Some(i);
            }
            seen += 1;
        }
    }
    None
}

/// Whether the entry is keyboard-skippable (a disabled item).
pub(crate) fn entry_disabled(entries: &[SelectEntry], index: usize) -> bool {
    matches!(entries.get(index), Some(SelectEntry::Item(it)) if it.disabled)
}

/// Next actionable action index from `from` in direction `delta`, wrapping
/// once and skipping disabled items.
fn next_actionable(entries: &[SelectEntry], from: usize, delta: isize) -> Option<usize> {
    let count = count_actionable(entries);
    if count == 0 {
        return None;
    }
    let mut idx = from as isize;
    for _ in 0..count {
        idx = (idx + delta).rem_euclid(count as isize);
        let i = idx as usize;
        if let Some(entry_i) = item_index_at(entries, i) {
            if !entry_disabled(entries, entry_i) {
                return Some(i);
            }
        }
    }
    None
}

/// Moves the highlight by `delta` (ArrowDown/ArrowUp); an empty highlight
/// starts at the first (or last, for negative delta) enabled item.
pub(crate) fn move_highlight(entries: &[SelectEntry], highlight: &mut Option<usize>, delta: isize) {
    if count_actionable(entries) == 0 {
        return;
    }
    let from = match *highlight {
        Some(idx) => idx,
        None => {
            *highlight = edge_actionable(entries, delta < 0);
            return;
        }
    };
    if let Some(next) = next_actionable(entries, from, delta) {
        *highlight = Some(next);
    }
}

/// First (or last) enabled action index, for Home/End.
pub(crate) fn edge_actionable(entries: &[SelectEntry], from_end: bool) -> Option<usize> {
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
        .find(|i| item_index_at(entries, *i).is_some_and(|ei| !entry_disabled(entries, ei)))
}

/// The action index of the item with the given value, for highlighting the
/// selection when the popup opens.
pub(crate) fn action_index_of_value(entries: &[SelectEntry], value: &str) -> Option<usize> {
    let mut seen = 0usize;
    for entry in entries {
        if let SelectEntry::Item(item) = entry {
            if item.value == value {
                return Some(seen);
            }
            seen += 1;
        }
    }
    None
}

/// First actionable item whose label starts with `query` (case-insensitive),
/// searching from action index `start` (exclusive) and wrapping once — the
/// Radix select typeahead match. Disabled items are skipped.
pub(crate) fn typeahead_action_index(
    entries: &[SelectEntry],
    start: usize,
    query: &str,
) -> Option<usize> {
    let count = count_actionable(entries);
    if count == 0 || query.is_empty() {
        return None;
    }
    let q = query.to_lowercase();
    for offset in 0..count {
        let idx = (start + offset) % count;
        if let Some(entry_i) = item_index_at(entries, idx) {
            if let Some(SelectEntry::Item(item)) = entries.get(entry_i) {
                if !item.disabled && item.label.to_lowercase().starts_with(&q) {
                    return Some(idx);
                }
            }
        }
    }
    None
}

/// Collects the printable characters typed this frame (Radix `useTypeahead`
/// feeds from raw text events; a focused text edit elsewhere suppresses it —
/// unlike the menus, the select itself holds egui focus while closed, so a
/// plain `egui_wants_keyboard_input` check would swallow every keystroke).
pub(crate) fn typeahead_chars(ctx: &Context) -> Vec<char> {
    if ctx.text_edit_focused() {
        return Vec::new();
    }
    let mut chars = Vec::new();
    ctx.input(|i| {
        for event in &i.events {
            if let egui::Event::Text(text) = event {
                chars.extend(text.chars().filter(|c| !c.is_control()));
            }
        }
    });
    chars
}

// ---------------------------------------------------------------------------
// Trigger colors (pure, unit-tested)
// ---------------------------------------------------------------------------

/// Resolved trigger colors for one state.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerColors {
    pub fill: Color32,
    pub stroke: Stroke,
    /// Focus ring color (alpha already applied).
    pub ring: Color32,
    pub text: Color32,
    pub chevron: Color32,
}

/// Mirrors the nova trigger recipe: `border-input bg-transparent pl-2.5
/// pr-2 text-sm`, `dark:bg-input/30 dark:hover:bg-input/50`,
/// `dark:aria-invalid:border-destructive/50`, the focus/invalid ring styles,
/// `data-[placeholder]:text-muted-foreground` and the full-opacity
/// `text-muted-foreground` chevron (`opacity-50` only when disabled).
pub(crate) fn resolve_trigger_colors(
    p: &Palette,
    dark: bool,
    focused: bool,
    invalid: bool,
    hovered: bool,
    disabled: bool,
    has_selection: bool,
) -> TriggerColors {
    let fill = if dark {
        if hovered && !disabled {
            color_alpha(p.input, 0.5)
        } else {
            color_alpha(p.input, 0.3)
        }
    } else {
        Color32::TRANSPARENT
    };
    let stroke_color = if invalid {
        if dark {
            color_alpha(p.destructive, 0.5)
        } else {
            p.destructive
        }
    } else if focused {
        p.ring
    } else {
        p.input
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    let text = if disabled || !has_selection {
        color_alpha(p.muted_foreground, if disabled { 0.5 } else { 1.0 })
    } else {
        p.foreground
    };
    TriggerColors {
        fill,
        stroke: Stroke::new(1.0, stroke_color),
        ring,
        text,
        chevron: if disabled {
            color_alpha(p.muted_foreground, 0.5)
        } else {
            p.muted_foreground
        },
    }
}

// ---------------------------------------------------------------------------
// Sizes
// ---------------------------------------------------------------------------

/// Trigger size (`data-size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectSize {
    /// `data-[size=default]:h-8` (32 px).
    #[default]
    Default,
    /// `data-[size=sm]:h-7` (28 px).
    Sm,
}

impl SelectSize {
    /// Trigger height in points.
    pub fn height(self) -> f32 {
        match self {
            SelectSize::Default => SELECT_TRIGGER_HEIGHT_DEFAULT,
            SelectSize::Sm => SELECT_TRIGGER_HEIGHT_SM,
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Per-instance popup state (open flag + trigger geometry + typeahead).
#[derive(Clone, Debug, Default)]
struct SelectState {
    open: bool,
    trigger: Option<TriggerInfo>,
    /// Radix-style typeahead buffer (printable-char search).
    typeahead: TypeaheadState,
}

#[derive(Clone, Copy, Debug)]
struct TriggerInfo {
    rect: Rect,
    layer_id: LayerId,
}

fn read_state(ctx: &Context, state_id: Id) -> SelectState {
    ctx.data_mut(|d| d.get_temp::<SelectState>(state_id).unwrap_or_default())
}

fn write_state(ctx: &Context, state_id: Id, state: SelectState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

/// What the select writes into: `Option<String>`, `String` (empty =
/// placeholder) or the internal uncontrolled store.
enum Binding<'a> {
    Opt(&'a mut Option<String>),
    Str(&'a mut String),
}

impl Binding<'_> {
    fn get(&self) -> Option<String> {
        match self {
            Binding::Opt(v) => (*v).clone(),
            Binding::Str(s) => {
                if s.is_empty() {
                    None
                } else {
                    Some(s.as_str().to_owned())
                }
            }
        }
    }

    fn set(&mut self, value: Option<String>) {
        match self {
            Binding::Opt(v) => **v = value,
            Binding::Str(s) => **s = value.unwrap_or_default(),
        }
    }
}

impl std::fmt::Debug for Binding<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::Opt(v) => f.debug_tuple("Opt").field(v).finish(),
            Binding::Str(s) => f.debug_tuple("Str").field(s).finish(),
        }
    }
}

/// What [`Select::show`] returns.
#[derive(Clone, Debug)]
pub struct SelectResponse {
    /// The trigger's response.
    pub trigger: Response,
    /// The value after this frame (bound or uncontrolled).
    pub value: Option<String>,
    /// Whether the value changed this frame.
    pub changed: bool,
    /// Whether the popup was open at frame start and is closed now.
    pub closed: bool,
}

// ---------------------------------------------------------------------------
// Select (root)
// ---------------------------------------------------------------------------

/// A shadcn select. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Select<'a> {
    binding: Option<Binding<'a>>,
    placeholder: Option<String>,
    size: SelectSize,
    width: Option<f32>,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
    /// Seeded options for the closure-less `show_seeded` call.
    seed: Vec<SelectEntry>,
}

impl<'a> Select<'a> {
    /// Creates a select bound to the caller's value (`None` = placeholder).
    pub fn new(value: &'a mut Option<String>) -> Self {
        Self {
            binding: Some(Binding::Opt(value)),
            ..Self::uncontrolled()
        }
    }

    /// Creates a select bound to a plain `String` (an empty string shows the
    /// placeholder).
    pub fn bind(value: &'a mut String) -> Self {
        Self {
            binding: Some(Binding::Str(value)),
            ..Self::uncontrolled()
        }
    }

    /// Creates an uncontrolled select: the value lives in the egui state
    /// keyed by [`Select::id_salt`]; read it from
    /// [`SelectResponse::value`].
    pub fn uncontrolled() -> Self {
        Self {
            binding: None,
            placeholder: None,
            size: SelectSize::Default,
            width: None,
            disabled: false,
            invalid: false,
            id_salt: None,
            seed: Vec::new(),
        }
    }

    /// Adds an enabled option (shorthand for the closure-less `show_seeded`).
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.seed.push(SelectEntry::Item(SelectItem::new(label)));
        self
    }

    /// Adds several enabled options.
    pub fn options(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for label in labels {
            self = self.option(label);
        }
        self
    }

    /// Adds a disabled option.
    pub fn option_disabled(mut self, label: impl Into<String>) -> Self {
        self.seed
            .push(SelectEntry::Item(SelectItem::new(label).disabled(true)));
        self
    }

    /// Shows faint text on the trigger while no value is selected
    /// (`SelectValue placeholder="…"`).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the trigger size (`data-size`).
    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
    }

    /// Sets an explicit trigger width. The default fits the widest option
    /// (`w-fit`), at least [`SELECT_MIN_WIDTH`].
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Disables the select (`disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the select `aria-invalid` (destructive border + ring).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so popup state (and the uncontrolled value)
    /// persist per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the trigger, and the option list popup when open. The closure
    /// adds labels, separators and grouped items through the
    /// [`SelectBuilder`].
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut SelectBuilder)) -> SelectResponse {
        let mut builder = SelectBuilder::new();
        content(&mut builder);
        if builder.entries.is_empty() {
            builder.entries = self.seed.clone();
        }
        self.show_with(ui, builder.entries)
    }

    /// Renders with only the seeded `.option(…)`s (no closure).
    pub fn show_seeded(self, ui: &mut Ui) -> SelectResponse {
        let seed = self.seed.clone();
        self.show_with(ui, seed)
    }

    fn show_with(self, ui: &mut Ui, entries: Vec<SelectEntry>) -> SelectResponse {
        let binding = self.binding;
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_select"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        let open_at_start = state.open;

        // Resolve the current value (controlled bindings win; uncontrolled
        // keeps its own stored value).
        let stored: Option<String> = ctx
            .data_mut(|d| d.get_temp::<Option<String>>(state_id.with("value")))
            .unwrap_or(None);
        let value_before = match &binding {
            Some(b) => b.get(),
            None => stored.clone(),
        };

        // Trigger geometry: `w-fit` fits the widest label, min `min-w-36`.
        let height = self.size.height();
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let widest = entries
            .iter()
            .map(|e| match e {
                SelectEntry::Item(it) => {
                    ui.painter()
                        .layout_no_wrap(it.label.clone(), font.clone(), Color32::WHITE)
                        .size()
                        .x
                }
                _ => 0.0,
            })
            .fold(0.0f32, f32::max)
            .max(
                self.placeholder
                    .as_deref()
                    .map(|ph| {
                        ui.painter()
                            .layout_no_wrap(ph.to_owned(), font.clone(), Color32::WHITE)
                            .size()
                            .x
                    })
                    .unwrap_or(0.0),
            );
        let width = self.width.unwrap_or_else(|| {
            (widest + TRIGGER_PAD_X + TRIGGER_ICON_GAP + ICON_SIZE + TRIGGER_PAD_RIGHT)
                .max(SELECT_MIN_WIDTH)
        });

        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (_, rect) = ui.allocate_space(Vec2::new(width, height));
        let trigger_id = state_id.with("trigger");
        let response = ui.interact(rect, trigger_id, sense);

        if ui.is_rect_visible(rect) {
            let c = resolve_trigger_colors(
                &p,
                dark,
                response.has_focus(),
                self.invalid,
                response.hovered(),
                self.disabled,
                value_before.is_some(),
            );
            let radius = theme.radius.lg_u8();
            let corner = CornerRadius::same(radius);
            let painter = ui.painter();

            // `shadow-xs`
            if !self.disabled {
                let sh = shadow::shadow_xs(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        corner,
                        sh.color,
                    )
                    .with_blur_width(2.0),
                );
            }

            if c.fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, corner, c.fill);
            }
            painter.rect_stroke(rect, corner, c.stroke, egui::epaint::StrokeKind::Inside);

            // Label: `pl-2.5 text-sm` (nova `gap-1.5` to the chevron).
            let label = value_before
                .clone()
                .unwrap_or_else(|| self.placeholder.clone().unwrap_or_default());
            let text_pos = Pos2::new(rect.min.x + TRIGGER_PAD_X, rect.center().y);
            painter.text(text_pos, Align2::LEFT_CENTER, label, font, c.text);

            // Chevron: `size-4 text-muted-foreground` at `pr-2`.
            let chev_rect = Rect::from_center_size(
                Pos2::new(
                    rect.max.x - TRIGGER_PAD_RIGHT - ICON_SIZE * 0.5,
                    rect.center().y,
                ),
                Vec2::splat(ICON_SIZE),
            );
            Icon::ChevronDown.paint(painter, chev_rect, c.chevron, 2.0 * ICON_SIZE / 24.0);

            // `focus-visible:ring-[3px] focus-visible:ring-ring/50`
            if response.has_focus() && !self.disabled {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, c.ring, 3.0, radius as f32);
            }
        }

        // Trigger input: click (or keyboard Enter/Space on the focused
        // trigger, which egui surfaces as a click) toggles; ArrowUp/Down
        // open while focused (Radix behavior).
        let mut just_opened = false;
        if !self.disabled && response.clicked() {
            state.open = !state.open;
            just_opened = state.open;
        }
        if !self.disabled && !state.open && response.has_focus() {
            let open_keys =
                ctx.input(|i| i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowUp));
            if open_keys {
                state.open = true;
                just_opened = true;
            }
        }

        // Opening highlights the current selection (Radix scroll/highlight).
        let mut highlight: Option<usize> = ctx
            .data_mut(|d| d.get_temp::<Option<usize>>(state_id.with("highlight")))
            .unwrap_or(None);
        if just_opened {
            highlight = value_before
                .as_deref()
                .and_then(|v| action_index_of_value(&entries, v));
        }
        if !state.open {
            state.trigger = Some(TriggerInfo {
                rect,
                layer_id: response.layer_id,
            });
        }

        // Radix typeahead: printable characters jump to matching options
        // while the popup is open, and select them directly on the focused
        // trigger while it is closed (the trigger value updates live). If
        // the buffered prefix matches nothing, the buffer restarts from the
        // last typed character.
        let mut typeahead_selected: Option<String> = None;
        let chars = typeahead_chars(&ctx);
        if !chars.is_empty() && !self.disabled {
            let now = ctx.input(|i| i.time);
            for ch in chars {
                if state
                    .typeahead
                    .at
                    .is_some_and(|t| now - t > MENU_TYPEAHEAD_TIMEOUT_SECS)
                {
                    state.typeahead.buffer.clear();
                }
                state.typeahead.buffer.push(ch);
                state.typeahead.at = Some(now);
                let mut query = state.typeahead.buffer.clone();
                let start = if state.open && open_at_start {
                    highlight.map_or(0, |i| i + 1)
                } else {
                    0
                };
                let mut found = typeahead_action_index(&entries, start, &query);
                if found.is_none() && query.chars().count() > 1 {
                    state.typeahead.buffer.clear();
                    state.typeahead.buffer.push(ch);
                    query = state.typeahead.buffer.clone();
                    found = typeahead_action_index(&entries, 0, &query);
                }
                if let Some(action) = found {
                    if state.open && open_at_start {
                        highlight = Some(action);
                    } else if !state.open && response.has_focus() {
                        if let Some(item_i) = item_index_at(&entries, action) {
                            if let Some(SelectEntry::Item(item)) = entries.get(item_i) {
                                typeahead_selected = Some(item.value.clone());
                            }
                        }
                    }
                }
            }
        }

        // Popup (kept up during the close fade like the source's
        // `data-[state=closed]:animate-out`).
        let mut activated: Option<String> = None;
        let mut popup_rects: Vec<Rect> = Vec::new();
        let popup_id = state_id.with("popup");
        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        if state.open || was_open_last_frame {
            if let Some(trigger) = state.trigger {
                // Nova `.cn-select-content` runs its in/out animation at
                // `duration-100`.
                let progress = nova_overlay_progress(&ctx, popup_id.with("anim"), state.open);
                if state.open || progress.alpha > 0.002 {
                    render_popup(
                        &ctx,
                        &entries,
                        popup_id,
                        trigger.rect,
                        trigger.layer_id,
                        // RTL mirrors the popup to hug the trigger's right
                        // edge (Radix flips inline alignment).
                        direction::popup_align(ui, RectAlign::BOTTOM_START, RectAlign::BOTTOM_END),
                        &p,
                        dark,
                        &theme,
                        value_before.as_deref(),
                        highlight,
                        state.open,
                        progress.alpha,
                        &mut popup_rects,
                        &mut activated,
                    );
                }
            }
        }

        // Keyboard behavior while the popup is already open (the frame it
        // opened, Radix only highlights).
        if state.open && open_at_start {
            let (down, up, home, end, enter, space, tab) = ctx.input(|i| {
                (
                    i.key_pressed(Key::ArrowDown),
                    i.key_pressed(Key::ArrowUp),
                    i.key_pressed(Key::Home),
                    i.key_pressed(Key::End),
                    i.key_pressed(Key::Enter),
                    i.key_pressed(Key::Space),
                    i.key_pressed(Key::Tab),
                )
            });
            if down {
                move_highlight(&entries, &mut highlight, 1);
            }
            if up {
                move_highlight(&entries, &mut highlight, -1);
            }
            if home || end {
                if let Some(idx) = edge_actionable(&entries, end) {
                    highlight = Some(idx);
                }
            }
            let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
            let esc = !modal_open && ctx.input(|i| i.key_pressed(Key::Escape));
            // Radix closes the popup on Tab as well.
            if esc || tab {
                state.open = false;
            }
            if (enter || space) && !esc && !tab {
                if let Some(action) = highlight {
                    if let Some(item_i) = item_index_at(&entries, action) {
                        if let Some(SelectEntry::Item(item)) = entries.get(item_i) {
                            if !item.disabled {
                                activated = Some(item.value.clone());
                            }
                        }
                    }
                }
            }
        }

        // Dismiss on a click outside every popup and the trigger (only once
        // the popup was actually visible).
        if state.open {
            let trigger_rect = state.trigger.map(|t| t.rect).unwrap_or(rect);
            if crate::components::dropdown_menu::handle_outside_click(
                &ctx,
                &popup_rects,
                &[trigger_rect],
                was_open_last_frame,
            ) {
                state.open = false;
            }
        }

        // Apply activation (click or keyboard; only set while genuinely
        // open — the fade renders inert rows). Selecting always closes.
        let mut value_after = value_before.clone();
        let mut changed = false;
        if let Some(value) = activated.take() {
            value_after = Some(value);
            changed = true;
            state.open = false;
        }
        // Closed-trigger typeahead selects directly (Radix updates the value
        // live on the trigger).
        if let Some(value) = typeahead_selected.take() {
            value_after = Some(value);
            changed = true;
        }
        if changed {
            let mut binding = binding;
            if let Some(b) = binding.as_mut() {
                b.set(value_after.clone());
            }
        }
        ctx.data_mut(|d| {
            d.insert_temp(state_id.with("value"), value_after.clone());
            d.insert_temp(state_id.with("highlight"), highlight);
        });

        let closed = open_at_start && !state.open;
        write_state(&ctx, state_id, state);

        SelectResponse {
            trigger: response,
            value: value_after,
            changed,
            closed,
        }
    }
}

/// Renders the options popup (a `min-w-36 rounded-lg ring-1
/// ring-foreground/10 shadow-md bg-popover p-1` list inside an
/// [`egui::Popup`]).
#[allow(clippy::too_many_arguments)]
fn render_popup(
    ctx: &Context,
    entries: &[SelectEntry],
    popup_id: Id,
    anchor: Rect,
    parent_layer: LayerId,
    align: RectAlign,
    p: &Palette,
    dark: bool,
    theme: &ShadcnTheme,
    selected_value: Option<&str>,
    highlight: Option<usize>,
    open: bool,
    alpha: f32,
    popup_rects: &mut Vec<Rect>,
    activated: &mut Option<String>,
) {
    let content_width = SELECT_MIN_WIDTH.max(anchor.width());
    let frame = egui::Frame::new()
        .fill(p.popover)
        .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
        .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
        .inner_margin(Margin::same(CONTENT_PADDING as i8))
        .shadow(shadow::shadow_md(dark));

    // `translate-y-1` on the popper content: the effective gap is the
    // default side offset plus 4 px.
    let gap = 8.0 + (1.0 - alpha) * 8.0;

    // Clamp the list to the screen space around the trigger (the source
    // clamps to `--radix-select-content-available-height`).
    let screen_h = ctx.content_rect().height();
    let below = screen_h - anchor.max.y - gap - AVAILABLE_SLACK;
    let above = anchor.min.y - gap - AVAILABLE_SLACK;
    let available = below.max(above).max(160.0);
    let max_height = MAX_CONTENT_HEIGHT.min(available);

    let popup = egui::Popup::new(
        popup_id,
        ctx.clone(),
        egui::PopupAnchor::ParentRect(anchor),
        parent_layer,
    )
    .open(true)
    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
    .align(align)
    .gap(gap)
    .width(content_width)
    .frame(egui::Frame::new());

    let inner = popup.show(|ui| {
        ui.set_opacity(alpha);
        frame.show(ui, |ui| {
            ui.set_min_width(content_width - 2.0 * CONTENT_PADDING);
            render_rows(
                ui,
                entries,
                popup_id,
                max_height,
                p,
                theme,
                selected_value,
                highlight,
                open,
                activated,
            );
        });
    });
    if let Some(ref r) = inner {
        popup_rects.push(r.response.rect);
    }
}

/// The scrolling row list (`max-h` viewport with `p-1` padding).
#[allow(clippy::too_many_arguments)]
fn render_rows(
    ui: &mut Ui,
    entries: &[SelectEntry],
    popup_id: Id,
    max_height: f32,
    p: &Palette,
    theme: &ShadcnTheme,
    selected_value: Option<&str>,
    highlight: Option<usize>,
    open: bool,
    activated: &mut Option<String>,
) {
    let scroll_id = popup_id.with("scroll");
    crate::components::scroll_area::ScrollArea::new()
        .height(max_height)
        .id_salt(scroll_id.value().to_string())
        .show(ui, |ui| {
            let mut action_idx = 0usize;
            for (i, entry) in entries.iter().enumerate() {
                match entry {
                    SelectEntry::Separator => paint_separator(ui, p),
                    SelectEntry::Label(text) => paint_label(ui, text, p),
                    SelectEntry::Item(item) => {
                        let action = action_idx;
                        action_idx += 1;
                        let row_id = popup_id.with(("select_row", i));
                        let highlighted = open && highlight == Some(action);
                        let is_selected = selected_value.is_some_and(|v| v == item.value);
                        if let Some(value) =
                            paint_item(ui, row_id, item, highlighted, is_selected, p, theme, open)
                        {
                            *activated = Some(value);
                        }
                    }
                }
            }
        });
}

/// Draws one option row; returns the option value when it was clicked.
#[allow(clippy::too_many_arguments)]
fn paint_item(
    ui: &mut Ui,
    row_id: Id,
    item: &SelectItem,
    highlighted: bool,
    selected: bool,
    p: &Palette,
    theme: &ShadcnTheme,
    open: bool,
) -> Option<String> {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), ITEM_HEIGHT));
    let sense = if item.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(rect, row_id, sense);

    let hovered = response.hovered() && !item.disabled;
    let fill = if (highlighted || hovered) && !item.disabled {
        p.accent
    } else {
        Color32::TRANSPARENT
    };
    let mut fg = if highlighted || hovered {
        p.accent_foreground
    } else {
        p.popover_foreground
    };
    let mut check = fg;
    if item.disabled {
        fg = color_alpha(fg, 0.5);
        check = color_alpha(check, 0.5);
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if fill != Color32::TRANSPARENT {
            painter.rect_filled(rect, CornerRadius::same(theme.radius.md_u8()), fill);
        }

        // Label: nova `pl-1.5 text-sm` (`pr-8` before the indicator).
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        painter.text(
            Pos2::new(rect.min.x + ITEM_PAD_X, rect.center().y),
            Align2::LEFT_CENTER,
            item.label.clone(),
            font,
            fg,
        );

        // Check indicator: `absolute right-2 size-4` when selected.
        if selected {
            let ind_rect = Rect::from_center_size(
                Pos2::new(rect.max.x - 8.0 - ICON_SIZE * 0.5, rect.center().y),
                Vec2::splat(ICON_SIZE),
            );
            Icon::Check.paint(painter, ind_rect, check, 2.0 * ICON_SIZE / 24.0);
        }
    }

    if open && response.clicked() && !item.disabled {
        return Some(item.value.clone());
    }
    None
}

fn paint_label(ui: &mut Ui, text: &str, p: &Palette) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), LABEL_HEIGHT));
    if ui.is_rect_visible(rect) {
        let font = FontId::proportional(crate::theme::tokens::type_scale::XS);
        ui.painter().text(
            Pos2::new(rect.min.x + ITEM_PAD_X, rect.center().y),
            Align2::LEFT_CENTER,
            text,
            font,
            p.muted_foreground,
        );
    }
}

fn paint_separator(ui: &mut Ui, p: &Palette) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), SEPARATOR_HEIGHT));
    if ui.is_rect_visible(rect) {
        // `-mx-1` — the line bleeds into the content padding.
        let y = rect.center().y;
        ui.painter().line_segment(
            [
                Pos2::new(rect.min.x - CONTENT_PADDING, y),
                Pos2::new(rect.max.x + CONTENT_PADDING, y),
            ],
            Stroke::new(1.0, p.border),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    /// Label, items, separator — action indices: Apple 0, Banana 1,
    /// Kiwi 2 (disabled), Carrot 3.
    fn entries() -> Vec<SelectEntry> {
        vec![
            SelectEntry::Label("Fruits".to_string()),
            SelectEntry::Item(SelectItem::new("Apple")),
            SelectEntry::Item(SelectItem::new("Banana")),
            SelectEntry::Item(SelectItem::new("Kiwi").disabled(true)),
            SelectEntry::Separator,
            SelectEntry::Item(SelectItem::new("Carrot").value("carrot")),
        ]
    }

    #[test]
    fn counts_only_items() {
        assert_eq!(count_actionable(&entries()), 4);
    }

    #[test]
    fn item_index_skips_labels_and_separators() {
        let e = entries();
        assert_eq!(item_index_at(&e, 0), Some(1));
        assert_eq!(item_index_at(&e, 2), Some(3));
        assert_eq!(item_index_at(&e, 3), Some(5));
        assert_eq!(item_index_at(&e, 4), None);
    }

    #[test]
    fn move_highlight_skips_disabled_and_wraps() {
        let e = entries();
        let mut h = None;
        move_highlight(&e, &mut h, 1);
        assert_eq!(h, Some(0), "starts at the first item");
        move_highlight(&e, &mut h, 1);
        assert_eq!(h, Some(1));
        move_highlight(&e, &mut h, 1);
        assert_eq!(h, Some(3), "skips the disabled Kiwi (2)");
        move_highlight(&e, &mut h, -1);
        assert_eq!(h, Some(1), "backward skips the disabled item too");
        let mut h = None;
        move_highlight(&e, &mut h, -1);
        assert_eq!(h, Some(3), "starting from the end when moving up");
        let mut h = Some(3);
        move_highlight(&e, &mut h, 1);
        assert_eq!(h, Some(0), "wraps forward to the start");
    }

    #[test]
    fn edge_actionable_finds_enabled_ends() {
        let e = entries();
        assert_eq!(edge_actionable(&e, false), Some(0));
        assert_eq!(edge_actionable(&e, true), Some(3));
        let empty: Vec<SelectEntry> = Vec::new();
        assert_eq!(edge_actionable(&empty, false), None);
    }

    #[test]
    fn action_index_of_value_finds_selection() {
        let e = entries();
        assert_eq!(action_index_of_value(&e, "carrot"), Some(3));
        assert_eq!(action_index_of_value(&e, "Apple"), Some(0));
        assert_eq!(action_index_of_value(&e, "missing"), None);
    }

    #[test]
    fn typeahead_finds_labels_from_a_start_index() {
        let e = entries();
        // "b" from the start finds Banana (action 1).
        assert_eq!(typeahead_action_index(&e, 0, "b"), Some(1));
        // From Banana onward it wraps back to Banana itself (no later "b").
        assert_eq!(typeahead_action_index(&e, 2, "b"), Some(1));
        // Disabled Kiwi is skipped: "k" wraps past it to nothing.
        assert_eq!(typeahead_action_index(&e, 0, "k"), None);
        // Prefixes are case-insensitive.
        assert_eq!(typeahead_action_index(&e, 0, "CAR"), Some(3));
        assert_eq!(typeahead_action_index(&e, 0, ""), None);
    }

    #[test]
    fn empty_entries_do_not_panic() {
        let e: Vec<SelectEntry> = Vec::new();
        let mut h = None;
        move_highlight(&e, &mut h, 1);
        assert!(h.is_none());
        assert_eq!(item_index_at(&e, 0), None);
        assert_eq!(action_index_of_value(&e, "x"), None);
    }

    #[test]
    fn trigger_colors_follow_state() {
        let p = palette(Mode::Light);
        let c = resolve_trigger_colors(&p, false, false, false, false, false, false);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.stroke.color, p.input);
        assert_eq!(c.text, p.muted_foreground, "placeholder look");
        assert_eq!(
            c.chevron, p.muted_foreground,
            "nova chevron is full-opacity muted"
        );
        let sel = resolve_trigger_colors(&p, false, false, false, false, false, true);
        assert_eq!(sel.text, p.foreground);
        let focused = resolve_trigger_colors(&p, false, true, false, false, false, true);
        assert_eq!(focused.stroke.color, p.ring);
        assert_eq!(focused.ring, color_alpha(p.ring, 0.5));
        let invalid = resolve_trigger_colors(&p, false, false, true, false, false, true);
        assert_eq!(invalid.stroke.color, p.destructive);
        assert_eq!(invalid.ring, color_alpha(p.destructive, 0.2));
        let disabled = resolve_trigger_colors(&p, false, false, false, false, true, true);
        assert_eq!(disabled.text, color_alpha(p.muted_foreground, 0.5));
        assert_eq!(
            disabled.chevron,
            color_alpha(p.muted_foreground, 0.5),
            "only the disabled chevron dims"
        );
    }

    #[test]
    fn dark_mode_hover_uses_input_50() {
        let p = palette(Mode::Dark);
        let hovered = resolve_trigger_colors(&p, true, false, false, true, false, true);
        assert_eq!(hovered.fill, color_alpha(p.input, 0.5));
        let resting = resolve_trigger_colors(&p, true, false, false, false, false, true);
        assert_eq!(resting.fill, color_alpha(p.input, 0.3));
    }

    #[test]
    fn dark_mode_invalid_border_is_half_destructive() {
        let p = palette(Mode::Dark);
        let invalid = resolve_trigger_colors(&p, true, false, true, false, false, true);
        assert_eq!(
            invalid.stroke.color,
            color_alpha(p.destructive, 0.5),
            "nova dark:aria-invalid:border-destructive/50"
        );
        assert_eq!(invalid.ring, color_alpha(p.destructive, 0.4));
    }

    #[test]
    fn geometry_constants_match_source() {
        assert_eq!(
            SelectSize::Default.height(),
            32.0,
            "nova data-[size=default]:h-8"
        );
        assert_eq!(SelectSize::Sm.height(), 28.0, "nova data-[size=sm]:h-7");
        assert_eq!(SELECT_MIN_WIDTH, 144.0, "nova min-w-36");
        assert_eq!(TRIGGER_PAD_X, 10.0, "pl-2.5");
        assert_eq!(TRIGGER_PAD_RIGHT, 8.0, "pr-2");
        assert_eq!(ITEM_HEIGHT, 28.0, "py-1 + text-sm");
        assert_eq!(ITEM_PAD_X, 6.0, "pl-1.5");
        assert_eq!(LABEL_HEIGHT, 20.0, "py-1 + text-xs");
        assert_eq!(MAX_CONTENT_HEIGHT, 300.0);
    }

    #[test]
    fn items_keep_values_and_disabled() {
        let it = SelectItem::new("Next.js");
        assert_eq!(it.value, "Next.js");
        assert_eq!(it.label, "Next.js");
        let it = SelectItem::new("Other").value("other").disabled(true);
        assert_eq!(it.value, "other");
        assert!(it.disabled);
    }

    #[test]
    fn builders_keep_entries() {
        let s = Select::uncontrolled()
            .option("A")
            .options(["B", "C"])
            .option_disabled("D")
            .placeholder("Pick")
            .size(SelectSize::Sm)
            .width(200.0)
            .disabled(true)
            .invalid(true)
            .id_salt("x");
        assert_eq!(s.seed.len(), 4);
        assert!(matches!(&s.seed[3], SelectEntry::Item(it) if it.disabled));
        assert_eq!(s.placeholder.as_deref(), Some("Pick"));
        assert_eq!(s.size, SelectSize::Sm);
        assert_eq!(s.width, Some(200.0));
        assert!(s.disabled && s.invalid);
        assert_eq!(s.id_salt.as_deref(), Some("x"));

        let mut b = SelectBuilder::new();
        b.label("G");
        b.item(SelectItem::new("I"));
        b.items(["X", "Y"]);
        b.disabled_item("Z");
        b.separator();
        assert_eq!(b.entries.len(), 6);
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
                    Id::unique("select_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        /// Presses and releases the primary button at `pos` over two frames.
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
                        Id::unique("select_test_ui"),
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
                    Id::unique("select_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn text(&mut self, text: &str, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events: vec![egui::Event::Text(text.to_owned())],
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("select_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> SelectState {
            let id = Id::unique("select_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<SelectState>(id))
                .unwrap_or_default()
        }

        fn highlight(&self, salt: &str) -> Option<usize> {
            let id = Id::unique("select_test_ui").with(salt).with("highlight");
            self.ctx
                .data_mut(|d| d.get_temp::<Option<usize>>(id))
                .flatten()
        }

        fn value(&self, salt: &str) -> Option<String> {
            let id = Id::unique("select_test_ui").with(salt).with("value");
            self.ctx
                .data_mut(|d| d.get_temp::<Option<String>>(id))
                .flatten()
        }
    }

    const OPTS: [&str; 1] = ["Only option"];

    fn layout(h: &mut Harness, salt: &str) -> Rect {
        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            trigger_rect = Select::uncontrolled()
                .id_salt(salt)
                .options(OPTS)
                .show_seeded(ui)
                .trigger
                .rect;
        });
        h.run(|ui| {
            trigger_rect = Select::uncontrolled()
                .id_salt(salt)
                .options(OPTS)
                .show_seeded(ui)
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

        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("t1")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("t1").open, "clicking the trigger opens it");

        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("t1")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("t1").open, "clicking again closes it");
    }

    #[test]
    fn click_outside_closes_open_select() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "out");
        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("out")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("out").open);

        // One more frame so the popup was visible last frame.
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("out")
                .options(OPTS)
                .show_seeded(ui);
        });
        h.click(Pos2::new(550.0, 550.0), |ui| {
            Select::uncontrolled()
                .id_salt("out")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("out").open, "clicking outside must close it");
    }

    #[test]
    fn escape_closes_open_select() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "esc");
        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("esc")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("esc").open);
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("esc")
                .options(OPTS)
                .show_seeded(ui);
        });
        h.key(Key::Escape, |ui| {
            Select::uncontrolled()
                .id_salt("esc")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("esc").open, "Escape must close the select");
    }

    #[test]
    fn rtl_scope_mirrors_popup_alignment() {
        use crate::components::direction::DirectionProvider;
        let mut h = Harness::new();
        let mut scope = Id::NULL;
        let mut rect = Rect::ZERO;
        // An 80px trigger keeps it narrower than the popup's 144px min
        // width, so the two alignments land on different rects. Capture
        // the trigger rect and the RTL scope id: the provider's
        // `with_layout` pushes a child scope, so the select state (and the
        // popup id) live one scope below the harness ui.
        h.run(|ui| {
            DirectionProvider::rtl().show(ui, |ui| {
                scope = ui.scope_id();
                rect = Select::uncontrolled()
                    .id_salt("rtl")
                    .width(80.0)
                    .options(OPTS)
                    .show_seeded(ui)
                    .trigger
                    .rect;
            });
        });
        h.click(rect.center(), |ui| {
            DirectionProvider::rtl().show(ui, |ui| {
                Select::uncontrolled()
                    .id_salt("rtl")
                    .width(80.0)
                    .options(OPTS)
                    .show_seeded(ui);
            });
        });
        let state = h
            .ctx
            .data_mut(|d| d.get_temp::<SelectState>(scope.with("rtl")))
            .unwrap_or_default();
        assert!(state.open, "clicking the trigger opens the select");
        // Let the popup settle one frame (a popup's first pass is
        // sizing-only).
        h.run(|ui| {
            DirectionProvider::rtl().show(ui, |ui| {
                Select::uncontrolled()
                    .id_salt("rtl")
                    .width(80.0)
                    .options(OPTS)
                    .show_seeded(ui);
            });
        });
        let popup_id = scope.with("rtl").with("popup");
        let popup_rect = h
            .ctx
            .read_response(popup_id)
            .map(|r| r.rect)
            .unwrap_or(Rect::NOTHING);
        assert!(popup_rect != Rect::NOTHING, "popup must be on screen");
        assert!(
            (popup_rect.max.x - rect.max.x).abs() < 3.0,
            "RTL select hugs the trigger's right edge (popup.max.x={}, \
             trigger.max.x={})",
            popup_rect.max.x,
            rect.max.x
        );
        assert!(
            popup_rect.min.x < rect.min.x,
            "the popup extends leftward from the trigger (popup.min.x={}, \
             trigger.min.x={})",
            popup_rect.min.x,
            rect.min.x
        );
    }

    #[test]
    fn selecting_an_item_stores_the_value() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "sel");
        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("sel")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("sel").open);

        // Let the popup settle one frame (egui hit-tests against the
        // previous pass's rects; a popup's first pass is sizing-only).
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("sel")
                .options(OPTS)
                .show_seeded(ui);
        });

        let popup_pos = Pos2::new(rect.center().x, rect.max.y + 20.0);
        h.click(popup_pos, |ui| {
            Select::uncontrolled()
                .id_salt("sel")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("sel").open, "activation closes the popup");
        assert_eq!(
            h.value("sel").as_deref(),
            Some("Only option"),
            "the uncontrolled store keeps the value"
        );
    }

    #[test]
    fn keyboard_moves_highlight_and_selects() {
        let mut h = Harness::new();
        let mut value: Option<String> = None;
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = Select::new(&mut value)
                .id_salt("kb")
                .option("Alpha")
                .option("Beta")
                .show_seeded(ui)
                .trigger
                .rect;
        });
        h.run(|ui| {
            Select::new(&mut value)
                .id_salt("kb")
                .option("Alpha")
                .option("Beta")
                .show_seeded(ui);
        });

        // Open with a click, then drive the popup from the keyboard.
        h.click(rect.center(), |ui| {
            Select::new(&mut value)
                .id_salt("kb")
                .option("Alpha")
                .option("Beta")
                .show_seeded(ui);
        });
        assert!(h.state("kb").open);

        h.key(Key::ArrowDown, |ui| {
            Select::new(&mut value)
                .id_salt("kb")
                .option("Alpha")
                .option("Beta")
                .show_seeded(ui);
        });
        h.key(Key::ArrowDown, |ui| {
            Select::new(&mut value)
                .id_salt("kb")
                .option("Alpha")
                .option("Beta")
                .show_seeded(ui);
        });
        h.key(Key::Enter, |ui| {
            Select::new(&mut value)
                .id_salt("kb")
                .option("Alpha")
                .option("Beta")
                .show_seeded(ui);
        });
        assert!(!h.state("kb").open, "Enter closes after selecting");
        assert_eq!(value.as_deref(), Some("Beta"), "second item selected");
    }

    #[test]
    fn focused_trigger_opens_on_arrow_down() {
        let mut h = Harness::new();
        let mut value: Option<String> = None;
        h.run(|ui| {
            Select::new(&mut value)
                .id_salt("focus")
                .option("Alpha")
                .show_seeded(ui);
        });
        // Focus the trigger (its id is `<scope>.with("trigger")`).
        let trigger_id = Id::unique("select_test_ui").with("focus").with("trigger");
        h.ctx.memory_mut(|m| m.request_focus(trigger_id));
        h.key(Key::ArrowDown, |ui| {
            Select::new(&mut value)
                .id_salt("focus")
                .option("Alpha")
                .show_seeded(ui);
        });
        assert!(h.state("focus").open, "ArrowDown opens a focused select");
    }

    #[test]
    fn disabled_select_ignores_input_and_keeps_value() {
        let mut h = Harness::new();
        let mut fruit = String::from("Apple");
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = Select::bind(&mut fruit)
                .id_salt("str")
                .option("Apple")
                .option("Pear")
                .disabled(true)
                .show_seeded(ui)
                .trigger
                .rect;
        });
        h.click(rect.center(), |ui| {
            Select::bind(&mut fruit)
                .id_salt("str")
                .option("Apple")
                .option("Pear")
                .disabled(true)
                .show_seeded(ui);
        });
        assert!(!h.state("str").open, "a disabled select cannot open");
        assert_eq!(fruit, "Apple", "value untouched");
    }

    #[test]
    fn typeahead_moves_highlight_while_open() {
        let mut h = Harness::new();
        let mut value: Option<String> = None;
        let rect = layout(&mut h, "ta_open");
        h.click(rect.center(), |ui| {
            Select::new(&mut value)
                .id_salt("ta_open")
                .options(["Apple", "Banana", "Cherry"])
                .show_seeded(ui);
        });
        assert!(h.state("ta_open").open);

        h.text("b", |ui| {
            Select::new(&mut value)
                .id_salt("ta_open")
                .options(["Apple", "Banana", "Cherry"])
                .show_seeded(ui);
        });
        assert_eq!(
            h.highlight("ta_open"),
            Some(1),
            "typing b highlights Banana"
        );
    }

    #[test]
    fn typeahead_selects_on_the_focused_closed_trigger() {
        let mut h = Harness::new();
        let mut value: Option<String> = None;
        h.run(|ui| {
            Select::new(&mut value)
                .id_salt("ta_closed")
                .options(["Apple", "Banana"])
                .show_seeded(ui);
        });
        // Focus the trigger, then type.
        let trigger_id = Id::unique("select_test_ui")
            .with("ta_closed")
            .with("trigger");
        h.ctx.memory_mut(|m| m.request_focus(trigger_id));
        h.text("b", |ui| {
            Select::new(&mut value)
                .id_salt("ta_closed")
                .options(["Apple", "Banana"])
                .show_seeded(ui);
        });
        assert!(!h.state("ta_closed").open, "stays closed");
        assert_eq!(
            value.as_deref(),
            Some("Banana"),
            "closed typeahead selects the match directly"
        );
    }

    #[test]
    fn tab_closes_the_open_select() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "tab");
        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("tab")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("tab").open);
        h.key(Key::Tab, |ui| {
            Select::uncontrolled()
                .id_salt("tab")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("tab").open, "Tab dismisses the popup");
    }
}
