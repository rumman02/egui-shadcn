//! shadcn `select.tsx` — a fully keyboard-navigable single-value select.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/select.tsx` (Radix Select)
//! to the frozen Pilot Wave S0 spec, in both recipes (base + nova, LAYER
//! composition — a property the nova rule does not mention keeps its base
//! value):
//!
//! - **Trigger:** base `h-9 rounded-md border border-input bg-transparent
//!   px-3 text-sm shadow-xs` (sm: `h-8`); nova `h-8 rounded-lg pl-2.5 pr-2`
//!   (sm: `h-7`, `rounded-[min(var(--radius-md),10px)]` = 8px). No hover
//!   fill in light mode; dark `bg-input/30` resting, `bg-input/50` hovered.
//!   Focus: 1px `border-ring` + 3px `ring-ring/50`; invalid: destructive
//!   border + ring (under nova the invalid ring shows at rest). The chevron
//!   is `size-4 text-muted-foreground`, at 50% opacity under base
//!   (`opacity-50`); under nova its opacity is OPEN-1 and the pre-existing
//!   full-opacity render is kept pending the S4 reference capture. The
//!   value clamps to one line with an ellipsis (`line-clamp-1`).
//! - **Content:** base `min-w-[8rem] rounded-md border shadow-md bg-popover`;
//!   nova `min-w-36 rounded-lg` + 1px `ring-foreground/10`. The popup is
//!   content-sized — it grows and shrinks with the row set, clamped only by
//!   the available height; no fixed ceiling, no reserved empty area below
//!   the last item. Scroll buttons (24px rows, 16px chevrons) show per
//!   scroll direction and auto-repeat while hovered (mechanics OPEN-5).
//! - **Items:** base 32px rows (`py-1.5 pl-2 rounded-sm`), nova 28px rows
//!   (`py-1 pl-1.5 rounded-md`); highlight fill follows focus only
//!   (`focus:bg-accent`) — hover alone never fills a row. The check
//!   indicator sits at `right-2` on the selected item. Labels and
//!   separators follow their recipes; groups render with 4px padding under
//!   nova (`.cn-select-group`).
//! - **Keyboard (Radix docs + spec §11):** click/Enter/Space/ArrowUp/Down
//!   open the focused trigger — Space highlights the selected item, Enter
//!   highlights the first item (the K1/K2 asymmetry is preserved);
//!   ArrowUp/Down move the highlight (skipping disabled items, wrapping),
//!   Home/End jump, Enter/Space select and close, Tab and Escape dismiss,
//!   and typing performs typeahead. Highlight scroll-follow keeps the
//!   highlighted row in view (OPEN-7, pending S6 confirmation).
//!
//! The value is controlled through `&mut Option<String>` or `&mut String`;
//! pass neither and use [`Select::uncontrolled`] to keep the selection inside
//! the egui state (read it from [`SelectResponse::value`]).
//!
//! Platform limits / proposed deviations (spec §15, Expert Reviewer
//! approval pending): **D1** no ARIA tree (keyboard/focus behaviour fully
//! implemented, semantics omitted); **D2** popper-only positioning — the
//! default `position="item-aligned"` geometry is Radix-internal (OPEN-2)
//! and cannot be built faithfully from the snapshot, so the popup renders
//! popper-style below/above the trigger; **D3** the enter/exit scale
//! (0.95) is omitted (fade + slide retained); **D4** `name`/`required`
//! form integration has no egui equivalent and is absent from the Rust API.
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
//!         s.group(|g| {
//!             g.label("Frameworks");
//!             g.item(SelectItem::new("Next.js"));
//!             g.item(SelectItem::new("SvelteKit"));
//!             g.item(SelectItem::new("Astro").disabled(true));
//!         });
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

use crate::components::button::mix_color;
use crate::components::direction;
use crate::components::dropdown_menu::{TypeaheadState, MENU_TYPEAHEAD_TIMEOUT_SECS};
use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use crate::utils::animation::{
    nova_overlay_progress, transition_alpha, transition_color_to, transition_progress,
    TransitionSpec,
};
use crate::utils::focus::paint_ring;
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::emath::RectAlign;
use egui::{
    Align, Align2, Color32, Context, CornerRadius, FontId, Id, Key, LayerId, Layout, Pos2, Rect,
    Response, Sense, Stroke, Ui, UiBuilder, Vec2,
};
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Geometry constants
// ---------------------------------------------------------------------------

/// Content min width: nova `min-w-36` (144 px). The base recipe uses
/// `min-w-[8rem]` (128 px); see [`content_min_width`].
pub const SELECT_MIN_WIDTH: f32 = 144.0;
/// Content min width: base `min-w-[8rem]` (128 px).
const CONTENT_MIN_WIDTH_BASE: f32 = 128.0;
/// Trigger height: base `data-[size=default]:h-9` (36 px).
pub const SELECT_TRIGGER_HEIGHT_DEFAULT: f32 = 36.0;
/// Small trigger height: base `data-[size=sm]:h-8` (32 px).
pub const SELECT_TRIGGER_HEIGHT_SM: f32 = 32.0;
/// Nova trigger height: `data-[size=default]:h-8` (32 px).
const TRIGGER_HEIGHT_NOVA_DEFAULT: f32 = 32.0;
/// Nova small trigger height: `data-[size=sm]:h-7` (28 px).
const TRIGGER_HEIGHT_NOVA_SM: f32 = 28.0;
/// Content padding: `p-1` (4 px), on the viewport in both recipes.
const CONTENT_PADDING: f32 = 4.0;
/// Check/chevron icon size: `size-4` (16 px).
const ICON_SIZE: f32 = 16.0;
/// Separator total height: `my-1` top and bottom + 1 px line (both recipes).
const SEPARATOR_HEIGHT: f32 = 9.0;
/// Scroll-button row height: 16 px chevron + `py-1` (24 px, §6.1).
const SCROLL_BUTTON_HEIGHT: f32 = 24.0;
/// Group padding under nova: `.cn-select-group` `p-1` (4 px, top + bottom).
const GROUP_PAD: f32 = 4.0;
/// Popper gap: `sideOffset` 0 + the 4 px popper translate (`S:65-66`).
const POPUP_GAP: f32 = 4.0;
/// Enter slide distance: `slide-in-from-*-2` (8 px, §4.5).
const SLIDE_DISTANCE: f32 = 8.0;
/// Collision padding: the Radix default `collisionPadding = 10` (§4.4).
const COLLISION_PADDING: f32 = 10.0;
/// Scroll margin for highlight scroll-follow: popper `scroll-my-1` (4 px).
const SCROLL_MARGIN: f32 = 4.0;
/// Scroll-button auto-repeat speed in points per second. The step/speed is
/// OPEN-5 (Radix-internal); this value is a working default for S6 to
/// verify, kept in one place so it can be re-tuned without a rework.
const SCROLL_BUTTON_SPEED: f32 = 240.0;

// ---------------------------------------------------------------------------
// Recipe-resolved geometry (pure, unit-tested)
// ---------------------------------------------------------------------------

/// Trigger height for the size under the active recipe.
fn trigger_height(size: SelectSize, nova: bool) -> f32 {
    match (size, nova) {
        (SelectSize::Default, false) => SELECT_TRIGGER_HEIGHT_DEFAULT,
        (SelectSize::Sm, false) => SELECT_TRIGGER_HEIGHT_SM,
        (SelectSize::Default, true) => TRIGGER_HEIGHT_NOVA_DEFAULT,
        (SelectSize::Sm, true) => TRIGGER_HEIGHT_NOVA_SM,
    }
}

/// Trigger corner radius: base `rounded-md` both sizes; nova `rounded-lg`
/// default and `rounded-[min(var(--radius-md),10px)]` sm — the two nova
/// sizes MUST differ (§3.4). The token is resolved, not hardcoded, so a
/// raised `--radius` still caps the sm radius at 10 px.
fn trigger_radius(size: SelectSize, nova: bool, radius: &RadiusTokens) -> u8 {
    match (size, nova) {
        (_, false) => radius.md_u8(),
        (SelectSize::Default, true) => radius.lg_u8(),
        (SelectSize::Sm, true) => radius.md_u8().min(10),
    }
}

/// Trigger horizontal padding (left, right): base `px-3`, nova `pl-2.5 pr-2`.
fn trigger_padding(nova: bool) -> (f32, f32) {
    if nova {
        (10.0, 8.0)
    } else {
        (12.0, 12.0)
    }
}

/// Value–chevron gap: base `gap-2`, nova `gap-1.5`.
fn trigger_gap(nova: bool) -> f32 {
    if nova {
        6.0
    } else {
        8.0
    }
}

/// Item row height: base 32 px (`py-1.5` + one `text-sm` line), nova 28 px
/// (`py-1`).
fn item_height(nova: bool) -> f32 {
    if nova {
        28.0
    } else {
        32.0
    }
}

/// Item left padding: base `pl-2`, nova `pl-1.5`. Right padding is `pr-8`
/// (32 px) in both recipes.
fn item_pad_x(nova: bool) -> f32 {
    if nova {
        6.0
    } else {
        8.0
    }
}

/// Item corner radius: base `rounded-sm`, nova `rounded-md`.
fn item_radius(nova: bool, radius: &RadiusTokens) -> u8 {
    if nova {
        radius.md_u8()
    } else {
        radius.sm_u8()
    }
}

/// Label row height: base 24 px (`py-1.5` + one `text-xs` line), nova 20 px
/// (`py-1`).
fn label_height(nova: bool) -> f32 {
    if nova {
        20.0
    } else {
        24.0
    }
}

/// Label left padding: base `px-2`, nova `px-1.5`.
fn label_pad_x(nova: bool) -> f32 {
    if nova {
        6.0
    } else {
        8.0
    }
}

/// Content min width: base 128 px, nova 144 px.
fn content_min_width(nova: bool) -> f32 {
    if nova {
        SELECT_MIN_WIDTH
    } else {
        CONTENT_MIN_WIDTH_BASE
    }
}

/// Content corner radius: base `rounded-md`, nova `rounded-lg`.
fn content_radius(nova: bool, radius: &RadiusTokens) -> u8 {
    if nova {
        radius.lg_u8()
    } else {
        radius.md_u8()
    }
}

/// Check-indicator centre offset from the `right-2` edge: the base
/// indicator box is 14 px (`size-3.5`) with the 16 px check centred inside
/// it, so its centre sits 7 px in; the nova box is 16 px, centre 8 px in.
fn indicator_offset(nova: bool) -> f32 {
    if nova {
        8.0
    } else {
        7.0
    }
}

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

/// One row of the select content (item, label, separator, group or group
/// padding). Groups are flattened before rendering (see
/// [`flatten_entries`]); the flat list is what indices and heights run over.
#[derive(Clone, Debug)]
pub(crate) enum SelectEntry {
    Item(SelectItem),
    Label(String),
    Separator,
    /// A `SelectGroup` wrapper: its rows render inline, with 4 px padding
    /// top and bottom under nova (`.cn-select-group` `p-1`).
    Group(Vec<SelectEntry>),
    /// 4 px group padding (nova only; inserted by flattening).
    GroupPad,
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

    /// Appends a `SelectGroup`: the closure adds the group's label and
    /// items. Under nova the group renders with 4 px padding top and bottom;
    /// the base recipe has no group rule, so groups render flat there.
    pub fn group(&mut self, content: impl FnOnce(&mut SelectBuilder)) {
        let mut sub = SelectBuilder::new();
        content(&mut sub);
        self.entries.push(SelectEntry::Group(sub.entries));
    }
}

/// Flattens groups into the render list, inserting [`SelectEntry::GroupPad`]
/// around each group under nova. Returns the input borrowed when it holds
/// no groups, so the common case allocates nothing.
fn flatten_entries<'a>(entries: &'a [SelectEntry], nova: bool) -> Cow<'a, [SelectEntry]> {
    if !entries.iter().any(|e| matches!(e, SelectEntry::Group(_))) {
        return Cow::Borrowed(entries);
    }
    let mut flat = Vec::new();
    flatten_into(entries, nova, &mut flat);
    Cow::Owned(flat)
}

fn flatten_into(entries: &[SelectEntry], nova: bool, out: &mut Vec<SelectEntry>) {
    for entry in entries {
        if let SelectEntry::Group(inner) = entry {
            if nova {
                out.push(SelectEntry::GroupPad);
            }
            flatten_into(inner, nova, out);
            if nova {
                out.push(SelectEntry::GroupPad);
            }
        } else {
            out.push(entry.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Number of keyboard-navigable entries (items; labels/separators/group
/// padding skipped).
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

/// Height of one entry row under the active recipe. Groups are flattened
/// before measuring, so a group contributes nothing here.
fn entry_height(entry: &SelectEntry, nova: bool) -> f32 {
    match entry {
        SelectEntry::Item(_) => item_height(nova),
        SelectEntry::Label(_) => label_height(nova),
        SelectEntry::Separator => SEPARATOR_HEIGHT,
        SelectEntry::GroupPad => GROUP_PAD,
        SelectEntry::Group(_) => 0.0,
    }
}

/// Total height of the row list (the viewport content height).
fn rows_height(entries: &[SelectEntry], nova: bool) -> f32 {
    entries.iter().map(|e| entry_height(e, nova)).sum()
}

/// Resolved popup layout: which scroll buttons show and how tall the
/// viewport is. Content-sized: the popup grows and shrinks with the row set
/// and is clamped only by the available height (§5) — no fixed ceiling, no
/// reserved empty area below the last item.
#[derive(Clone, Copy, Debug, PartialEq)]
struct PopupLayout {
    show_up: bool,
    show_down: bool,
    viewport_height: f32,
    total_height: f32,
}

fn popup_layout(rows_h: f32, available: f32, offset_y: f32) -> PopupLayout {
    let pad = 2.0 * CONTENT_PADDING;
    let scrollable = rows_h > available - pad;
    // Worst-case viewport (both buttons) for the max-offset estimate; the
    // per-direction visibility below is stable against the one-button
    // viewport (a button appearing only shrinks the viewport, which keeps
    // the list scrollable — see the layout tests).
    let viewport_both = (available - 2.0 * SCROLL_BUTTON_HEIGHT - pad).max(0.0);
    let max_offset = (rows_h - viewport_both).max(0.0);
    let show_up = scrollable && offset_y > 0.5;
    let show_down = scrollable && offset_y < max_offset - 0.5;
    let buttons_h = (show_up as u8 + show_down as u8) as f32 * SCROLL_BUTTON_HEIGHT;
    let viewport_height = rows_h.min(available - buttons_h - pad).max(0.0);
    PopupLayout {
        show_up,
        show_down,
        viewport_height,
        total_height: viewport_height + pad + buttons_h,
    }
}

/// Last frame's scroll-viewport snapshot, for scroll-button visibility and
/// the auto-repeat drive. Persisted in temp data under the select's id.
#[derive(Clone, Copy, Debug, Default)]
struct ScrollSnapshot {
    offset_y: f32,
    content_h: f32,
    viewport_h: f32,
}

/// Truncates `text` with an ellipsis to fit `max_width` (the trigger value's
/// `line-clamp-1`), via binary search over char count. Returns the text
/// unchanged when it already fits.
fn truncate_with_ellipsis(
    text: &str,
    max_width: f32,
    painter: &egui::Painter,
    font: &FontId,
    color: Color32,
) -> String {
    if max_width <= 0.0 {
        return String::new();
    }
    if painter
        .layout_no_wrap(text.to_owned(), font.clone(), color)
        .size()
        .x
        <= max_width
    {
        return text.to_owned();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lo = 0usize;
    let mut hi = chars.len();
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        let candidate: String = chars[..mid].iter().collect::<String>() + "…";
        if painter
            .layout_no_wrap(candidate, font.clone(), color)
            .size()
            .x
            <= max_width
        {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mut out: String = chars[..lo].iter().collect();
    out.push('…');
    out
}

// ---------------------------------------------------------------------------
// Trigger colors (pure, unit-tested)
// ---------------------------------------------------------------------------

/// Resolved trigger colours for one state, before animation. The fill is an
/// alpha over `p.input` (light: transparent; dark: `/30` resting, `/50`
/// hovered); the stroke has rest/active endpoints so the focus/invalid
/// change animates; the ring target is wrapped once and faded by an alpha
/// ramp (the button ring pattern).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerEndpoints {
    /// Fill alpha over `p.input` at rest.
    pub fill_rest_alpha: f32,
    /// Fill alpha over `p.input` hovered.
    pub fill_hover_alpha: f32,
    /// Border colour at rest (unfocused, valid): `border-input`.
    pub rest_stroke: Color32,
    /// Border colour when focused or invalid (invalid wins).
    pub active_stroke: Color32,
    /// Focus/invalid ring colour (alpha already applied).
    pub ring_target: Color32,
    /// Value/placeholder text colour.
    pub text: Color32,
    /// Chevron colour.
    pub chevron: Color32,
}

/// Mirrors the trigger recipes (§3.3): `border-input`, the dark fills,
/// `dark:aria-invalid:border-destructive/50` (nova only), the focus/invalid
/// ring styles, and `data-[placeholder]:text-muted-foreground`. The base
/// chevron is `opacity-50`; under nova its opacity is OPEN-1, so the
/// pre-existing full-opacity render is kept pending the S4 capture.
pub(crate) fn resolve_trigger_endpoints(
    p: &Palette,
    dark: bool,
    nova: bool,
    invalid: bool,
    has_selection: bool,
) -> TriggerEndpoints {
    let (fill_rest_alpha, fill_hover_alpha) = if dark { (0.3, 0.5) } else { (0.0, 0.0) };
    let active_stroke = if invalid {
        if dark && nova {
            color_alpha(p.destructive, 0.5)
        } else {
            p.destructive
        }
    } else {
        p.ring
    };
    let ring_target = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    let text = if has_selection {
        p.foreground
    } else {
        p.muted_foreground
    };
    let chevron = if nova {
        p.muted_foreground
    } else {
        color_alpha(p.muted_foreground, 0.5)
    };
    TriggerEndpoints {
        fill_rest_alpha,
        fill_hover_alpha,
        rest_stroke: p.input,
        active_stroke,
        ring_target,
        text,
        chevron,
    }
}

// ---------------------------------------------------------------------------
// Sizes
// ---------------------------------------------------------------------------

/// Trigger size (`data-size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectSize {
    /// `data-[size=default]:h-9` base (36 px), `h-8` nova (32 px).
    #[default]
    Default,
    /// `data-[size=sm]:h-8` base (32 px), `h-7` nova (28 px).
    Sm,
}

impl SelectSize {
    /// Trigger height in points under the base recipe.
    pub fn height(self) -> f32 {
        match self {
            SelectSize::Default => SELECT_TRIGGER_HEIGHT_DEFAULT,
            SelectSize::Sm => SELECT_TRIGGER_HEIGHT_SM,
        }
    }

    /// Trigger height in points under the given style (nova renders shorter
    /// triggers).
    pub fn height_for(self, style: Style) -> f32 {
        trigger_height(self, style == Style::Nova)
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

/// A shadcn select (`SelectTrigger` + `SelectContent`). See the [module
/// docs](self) for an example.
///
/// The trigger is `w-fit` — sized to the widest option (or the placeholder)
/// — with no minimum-width floor (§3.2); the content enforces its own min
/// width.
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
    /// (`w-fit`), with no minimum-width floor (§3.2).
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

    fn show_with(self, ui: &mut Ui, entries_in: Vec<SelectEntry>) -> SelectResponse {
        let mut binding = self.binding;
        let theme = ShadcnTheme::of_ui(ui);
        let nova = theme.style == Style::Nova;
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        // Groups flatten into the render list (nova-only group padding);
        // all indices and heights below run over this flat list.
        let flat = flatten_entries(&entries_in, nova);
        let entries: &[SelectEntry] = &flat;

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

        // The trigger shows the selected item's ItemText content (§3.5);
        // when the bound value matches no item (OPEN-16), the raw value
        // is shown in foreground, preserving the pre-existing behaviour.
        let selected_label = value_before.as_deref().and_then(|v| {
            entries.iter().find_map(|e| match e {
                SelectEntry::Item(it) if it.value == v => Some(it.label.as_str()),
                _ => None,
            })
        });
        let display = selected_label.or(value_before.as_deref());
        let label = display
            .map(str::to_owned)
            .unwrap_or_else(|| self.placeholder.clone().unwrap_or_default());

        // Trigger geometry: `w-fit` fits the widest label (or the
        // placeholder), with no minimum-width floor (§3.2).
        let height = trigger_height(self.size, nova);
        let (pad_l, pad_r) = trigger_padding(nova);
        let gap = trigger_gap(nova);
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
        let width = self
            .width
            .unwrap_or(widest + pad_l + gap + ICON_SIZE + pad_r);

        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (_, rect) = ui.allocate_space(Vec2::new(width, height));
        let trigger_id = state_id.with("trigger");
        let response = ui.interact(rect, trigger_id, sense);

        // Claim Escape + vertical arrows on the trigger's focus lock while
        // focused: Escape closes the popup without surrendering focus (K7)
        // and arrows drive the highlight/open (K3/K4) instead of moving
        // focus spatially (the button focus-lock pattern).
        if !self.disabled && response.has_focus() {
            ctx.memory_mut(|m| {
                m.set_focus_lock_filter(
                    trigger_id,
                    egui::EventFilter {
                        tab: false,
                        horizontal_arrows: false,
                        vertical_arrows: true,
                        escape: true,
                    },
                );
            });
        }

        if ui.is_rect_visible(rect) {
            let spec = TransitionSpec::default();
            let opacity = if self.disabled { 0.5 } else { 1.0 };
            let endpoints =
                resolve_trigger_endpoints(&p, dark, nova, self.invalid, value_before.is_some());
            let focused = response.has_focus();
            let hovered = response.hovered() && !self.disabled;
            let radius = trigger_radius(self.size, nova, &theme.radius);
            let corner = CornerRadius::same(radius);
            let painter = ui.painter();

            // `shadow-xs` (LAYER: survives under nova). The whole trigger
            // fades to 50% when disabled, shadow included, so the resolved
            // layers ride the same opacity.
            let shadow = resolve_shadow(ShadowSize::Xs, theme.style, ShadowSlot::SelectTrigger);
            if opacity >= 1.0 {
                paint_shadow_layers(painter, rect, corner, shadow.layers(), dark);
            } else {
                let faded: Vec<ShadowLayer> = shadow
                    .layers()
                    .iter()
                    .map(|l| ShadowLayer {
                        alpha: l.alpha * opacity,
                        ..*l
                    })
                    .collect();
                paint_shadow_layers(painter, rect, corner, &faded, dark);
            }

            // Fill: dark `input/30` resting, `input/50` hovered; transparent
            // in light. The hover leg animates the alpha (the colour
            // transitions of §3.3); theme switches snap the endpoints (the
            // dialog OBS-D1 precedent) rather than legging across the
            // transparent boundary (the button BTN-FLK-02 lesson).
            let hover_t = transition_progress(&ctx, trigger_id.with("fill_hover"), hovered, spec).t;
            let fill_alpha = (endpoints.fill_rest_alpha
                + (endpoints.fill_hover_alpha - endpoints.fill_rest_alpha) * hover_t)
                * opacity;
            if fill_alpha > 0.0 {
                painter.rect_filled(rect, corner, color_alpha(p.input, fill_alpha));
            }

            // Border: `input` at rest, `ring` focused, `destructive` invalid
            // (invalid wins). Both endpoints ride the endpoint-change
            // animator so theme switches animate; the focus/invalid mix is
            // the boundary-safe `mix_color` (button pattern).
            let rest_stroke = transition_color_to(
                &ctx,
                trigger_id.with("border_rest"),
                endpoints.rest_stroke,
                spec,
            )
            .color;
            let active_stroke = transition_color_to(
                &ctx,
                trigger_id.with("border_active"),
                endpoints.active_stroke,
                spec,
            )
            .color;
            let t_border = transition_progress(
                &ctx,
                trigger_id.with("border"),
                focused || self.invalid,
                spec,
            )
            .t;
            let stroke_color =
                color_alpha(mix_color(rest_stroke, active_stroke, t_border), opacity);
            painter.rect_stroke(
                rect,
                corner,
                Stroke::new(1.0, stroke_color),
                egui::epaint::StrokeKind::Inside,
            );

            // Text + chevron ride the endpoint-change animator (theme
            // switches animate; the disabled fade snaps — `transition-colors`
            // does not cover opacity).
            let text_color = color_alpha(
                transition_color_to(&ctx, trigger_id.with("text"), endpoints.text, spec).color,
                opacity,
            );
            let chevron_color = color_alpha(
                transition_color_to(&ctx, trigger_id.with("chevron"), endpoints.chevron, spec)
                    .color,
                opacity,
            );

            // Value: `line-clamp-1` — truncate with an ellipsis under an
            // explicit narrow width; the chevron never shrinks (E11).
            let chev_rect = Rect::from_center_size(
                Pos2::new(rect.max.x - pad_r - ICON_SIZE * 0.5, rect.center().y),
                Vec2::splat(ICON_SIZE),
            );
            let value_zone = Rect::from_min_max(
                Pos2::new(rect.min.x + pad_l, rect.min.y),
                Pos2::new(chev_rect.min.x - gap, rect.max.y),
            );
            let shown =
                truncate_with_ellipsis(&label, value_zone.width(), painter, &font, text_color);
            painter.with_clip_rect(value_zone).text(
                Pos2::new(rect.min.x + pad_l, rect.center().y),
                Align2::LEFT_CENTER,
                shown,
                font,
                text_color,
            );

            // Chevron: 16px, `muted-foreground` (50% under base), never
            // rotates on open (§3.6).
            Icon::ChevronDown.paint(painter, chev_rect, chevron_color, 2.0 * ICON_SIZE / 24.0);

            // Focus ring: 3px `ring/50`, or the destructive ring when
            // invalid. Under nova the invalid ring shows at rest
            // (`aria-invalid:ring-3` ungated); under base it needs focus.
            // The target rides the endpoint animator; visibility is a pure
            // alpha ramp (button ring pattern). Painted on the rect directly
            // (adjacent ring — the button BTN-VIS-02 lesson).
            let ring_shown = focused || (self.invalid && nova);
            let ring_target = transition_color_to(
                &ctx,
                trigger_id.with("ring_target"),
                endpoints.ring_target,
                spec,
            )
            .color;
            let ring_alpha =
                transition_alpha(&ctx, trigger_id.with("ring"), 0.0, 1.0, ring_shown, spec).alpha;
            if ring_alpha > 0.0 {
                paint_ring(
                    painter,
                    rect,
                    color_alpha(ring_target, ring_alpha * opacity),
                    3.0,
                    radius as f32,
                );
            }
        }

        // Trigger input: click (or keyboard Enter/Space on the focused
        // trigger, which egui surfaces as a click) toggles; ArrowUp/Down
        // open while focused (Radix behavior). While the popup is already
        // open, a keyboard click must NOT toggle — Enter/Space select the
        // highlighted item (K6) via the keyboard block below; only pointer
        // clicks toggle the open popup closed (M1/M2). Without this gate
        // the focus-lock filter (which keeps the trigger focused across
        // arrow navigation) would turn every Enter-select into a toggle.
        let mut just_opened = false;
        let keyboard_click = ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
        if !self.disabled && response.clicked() && !(open_at_start && keyboard_click) {
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

        // Opening focus: K1/K2 asymmetry (Radix docs) — Space opens on the
        // selected item, Enter opens on the first item. Mouse/arrow opens
        // keep the selection highlight (OPEN-12: the arrow-open target is
        // unsettled, so the pre-existing behaviour stands).
        let mut highlight: Option<usize> = ctx
            .data_mut(|d| d.get_temp::<Option<usize>>(state_id.with("highlight")))
            .unwrap_or(None);
        if just_opened {
            let (enter, space) =
                ctx.input(|i| (i.key_pressed(Key::Enter), i.key_pressed(Key::Space)));
            highlight = open_highlight(entries, value_before.as_deref(), enter, space);
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
                let mut found = typeahead_action_index(entries, start, &query);
                if found.is_none() && query.chars().count() > 1 {
                    state.typeahead.buffer.clear();
                    state.typeahead.buffer.push(ch);
                    query = state.typeahead.buffer.clone();
                    found = typeahead_action_index(entries, 0, &query);
                }
                if let Some(action) = found {
                    if state.open && open_at_start {
                        highlight = Some(action);
                    } else if !state.open && response.has_focus() {
                        if let Some(item_i) = item_index_at(entries, action) {
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
        let mut popup_rect: Option<Rect> = None;
        let popup_id = state_id.with("popup");
        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        // Scroll snapshot from last frame (scroll-button visibility +
        // auto-repeat drive) and the last rendered highlight (highlight
        // scroll-follow, OPEN-7/N3 — only scroll when the highlight moved).
        let mut snapshot: ScrollSnapshot = ctx
            .data_mut(|d| d.get_temp(state_id.with("scroll")))
            .unwrap_or_default();
        let rendered_highlight: Option<usize> = ctx
            .data_mut(|d| d.get_temp(state_id.with("rendered_highlight")))
            .flatten();
        let scroll_to_highlight = highlight != rendered_highlight;
        if state.open || was_open_last_frame {
            if let Some(trigger) = state.trigger {
                // Nova `.cn-select-content` runs its in/out animation at
                // `duration-100`.
                let progress = nova_overlay_progress(&ctx, popup_id.with("anim"), state.open);
                if state.open || progress.alpha > 0.002 {
                    render_popup(
                        &ctx,
                        entries,
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
                        &mut snapshot,
                        scroll_to_highlight,
                        &mut popup_rect,
                        &mut activated,
                    );
                    ctx.data_mut(|d| {
                        d.insert_temp(state_id.with("rendered_highlight"), highlight);
                    });
                }
            }
        }
        ctx.data_mut(|d| {
            d.insert_temp(state_id.with("scroll"), snapshot);
        });

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
                move_highlight(entries, &mut highlight, 1);
            }
            if up {
                move_highlight(entries, &mut highlight, -1);
            }
            if home || end {
                if let Some(idx) = edge_actionable(entries, end) {
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
                    if let Some(item_i) = item_index_at(entries, action) {
                        if let Some(SelectEntry::Item(item)) = entries.get(item_i) {
                            if !item.disabled {
                                activated = Some(item.value.clone());
                            }
                        }
                    }
                }
            }
        }

        // Dismiss on a click outside the popup and the trigger (only once
        // the popup was actually visible).
        if state.open {
            let trigger_rect = state.trigger.map(|t| t.rect).unwrap_or(rect);
            if crate::components::dropdown_menu::handle_outside_click(
                &ctx,
                popup_rect.as_slice(),
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

/// Opening highlight (K1/K2, Radix docs): Enter opens on the first item,
/// Space (and mouse/arrow opens) on the selected item.
fn open_highlight(
    entries: &[SelectEntry],
    selected_value: Option<&str>,
    enter: bool,
    space: bool,
) -> Option<usize> {
    if enter && !space {
        edge_actionable(entries, false)
    } else {
        selected_value.and_then(|v| action_index_of_value(entries, v))
    }
}

// ---------------------------------------------------------------------------
// Popup rendering
// ---------------------------------------------------------------------------

/// Renders the popper popup below/above the trigger (D2: popper-only — the
/// default `position="item-aligned"` geometry is Radix-internal, OPEN-2).
/// The popup is content-sized (§5): its height is the row list clamped only
/// by the available height, with 24px scroll buttons outside the viewport
/// when the list scrolls (§6).
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
    snapshot: &mut ScrollSnapshot,
    scroll_to_highlight: bool,
    popup_rect: &mut Option<Rect>,
    activated: &mut Option<String>,
) {
    let nova = theme.style == Style::Nova;
    let content_width = content_min_width(nova).max(anchor.width());
    let corner = CornerRadius::same(content_radius(nova, &theme.radius));

    // Available height: the space around the trigger (Radix
    // `--radix-select-content-available-height`), collision padding 10.
    // No fixed ceiling (§5): the old 300 px cap is gone.
    let screen_h = ctx.content_rect().height();
    let below = screen_h - anchor.max.y - POPUP_GAP - COLLISION_PADDING;
    let above = anchor.min.y - POPUP_GAP - COLLISION_PADDING;
    let available = below.max(above).max(0.0);

    let rows_h = rows_height(entries, nova);
    let layout = popup_layout(rows_h, available, snapshot.offset_y);

    // Slide: entering slides 8px from the trigger side
    // (`slide-in-from-top-2` for a bottom popup); the exit animation has
    // no slide (§4.5). The scale (0.95) is omitted (D3).
    let gap = if open {
        POPUP_GAP - (1.0 - alpha) * SLIDE_DISTANCE
    } else {
        POPUP_GAP
    };

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
        let total = Vec2::new(content_width, layout.total_height);
        let (_, rect) = ui.allocate_space(total);

        // Shadow first (behind the body), then fill + strokes: 1px
        // `border` inside, plus the nova 1px `ring-foreground/10` outside
        // (the dialog pattern; the effective rendered stroke is OPEN-19).
        paint_shadow_resolved(
            ui.painter(),
            rect,
            corner,
            ShadowSize::Md,
            theme.style,
            ShadowSlot::SelectContent,
            dark,
        );
        ui.painter().rect_filled(rect, corner, p.popover);
        ui.painter().rect_stroke(
            rect,
            corner,
            Stroke::new(1.0, p.border),
            egui::epaint::StrokeKind::Inside,
        );
        if nova {
            ui.painter().rect_stroke(
                rect,
                corner,
                Stroke::new(1.0, color_alpha(p.foreground, 0.10)),
                egui::epaint::StrokeKind::Outside,
            );
        }

        // Scroll-button hover from last frame drives this frame's
        // auto-repeat (OPEN-5 mechanics; one frame of lag, symmetric for
        // both buttons). Inert while the popup is closing.
        let up_id = popup_id.with("scroll_up");
        let down_id = popup_id.with("scroll_down");
        let up_hovered = ctx.read_response(up_id).is_some_and(|r| r.hovered());
        let down_hovered = ctx.read_response(down_id).is_some_and(|r| r.hovered());
        let mut offset_override: Option<f32> = None;
        if open && (up_hovered || down_hovered) {
            let dt = ctx.input(|i| i.unstable_dt).clamp(0.0, 0.05);
            let max_offset = (rows_h - layout.viewport_height).max(0.0);
            let mut offset = snapshot.offset_y;
            if up_hovered {
                offset -= SCROLL_BUTTON_SPEED * dt;
            }
            if down_hovered {
                offset += SCROLL_BUTTON_SPEED * dt;
            }
            offset_override = Some(offset.clamp(0.0, max_offset));
            ctx.request_repaint();
        }

        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::top_down(Align::Min)),
        );
        if layout.show_up {
            paint_scroll_button(&mut child, up_id, true, p, nova, corner);
        }
        // Viewport band: the `p-1` padding lives on the viewport, while the
        // scroll buttons span the full content width (§6.1).
        let band_h = layout.viewport_height + 2.0 * CONTENT_PADDING;
        let (_, band) = child.allocate_space(Vec2::new(content_width, band_h));
        let mut rows_ui = child.new_child(
            UiBuilder::new()
                .max_rect(band.shrink(CONTENT_PADDING))
                .layout(Layout::top_down(Align::Min)),
        );
        let output = render_rows(
            &mut rows_ui,
            entries,
            popup_id,
            layout.viewport_height,
            offset_override,
            p,
            theme,
            selected_value,
            highlight,
            open,
            scroll_to_highlight,
            activated,
        );
        snapshot.offset_y = output.state.clamped_offset().y;
        snapshot.content_h = output.content_size.y;
        snapshot.viewport_h = output.inner_rect.height();
        if layout.show_down {
            paint_scroll_button(&mut child, down_id, false, p, nova, corner);
        }
    });
    if let Some(inner) = inner {
        *popup_rect = Some(inner.response.rect);
    }
}

/// Renders the scrollable row list: a bare vertical `ScrollArea` (not the
/// themed wrapper — the viewport has no visible scrollbar, §2.8),
/// `AlwaysHidden` scrollbar, no shrinking. The scroll offset is overridden
/// only on scroll-button frames, so wheel scrolling keeps working between
/// them.
///
/// The direction flags are load-bearing (SEL-INT-02/SEL-FLK-01): the area
/// MUST be vertical-only (`[false, true]`). With `[true, false]` the
/// vertical axis is disabled, so egui expands the inner rect to the full
/// content height, `max_offset.y` is 0, and the clamped offset is stuck at
/// 0 — the auto-repeat override re-applies the same single step every
/// frame and never accumulates, the up button never appears, and wheel
/// deltas are consumed by the horizontal axis instead of scrolling.
#[allow(clippy::too_many_arguments)]
fn render_rows(
    ui: &mut Ui,
    entries: &[SelectEntry],
    popup_id: Id,
    max_height: f32,
    offset_override: Option<f32>,
    p: &Palette,
    theme: &ShadcnTheme,
    selected_value: Option<&str>,
    highlight: Option<usize>,
    open: bool,
    scroll_to_highlight: bool,
    activated: &mut Option<String>,
) -> egui::containers::scroll_area::ScrollAreaOutput<()> {
    let mut area = egui::ScrollArea::new([false, true])
        .auto_shrink([false, false])
        .max_height(max_height)
        .scroll_bar_visibility(egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .id_salt(popup_id.with("scroll"));
    if let Some(offset) = offset_override {
        area = area.vertical_scroll_offset(offset);
    }
    area.show(ui, |ui| {
        // Rows stack with zero inter-row spacing: the frame height model
        // (`rows_height` / `popup_layout`) sums pure entry heights (§5),
        // so the theme's global 8px `item_spacing` must not leak between
        // rows here (SEL-VIS-02). The paint functions below allocate one
        // rect each and never read the spacing, so only the pitch changes.
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut action_idx = 0usize;
        for entry in entries.iter() {
            match entry {
                SelectEntry::Separator => paint_separator(ui, p),
                SelectEntry::Label(text) => paint_label(ui, text, p, theme),
                SelectEntry::GroupPad => {
                    ui.allocate_space(Vec2::new(ui.available_width(), GROUP_PAD));
                }
                SelectEntry::Group(_) => {
                    // Flattened before rendering; unreachable.
                }
                SelectEntry::Item(item) => {
                    let highlighted = open && highlight == Some(action_idx);
                    let selected = selected_value.is_some_and(|v| v == item.value);
                    if let Some(value) = paint_item(
                        ui,
                        popup_id.with(("row", action_idx)),
                        item,
                        highlighted,
                        selected,
                        p,
                        theme,
                        open,
                        scroll_to_highlight,
                    ) {
                        *activated = Some(value);
                    }
                    action_idx += 1;
                }
            }
        }
    })
}

/// A scroll button (§6): 24px row, full content width, OUTSIDE the viewport,
/// 16px chevron in `popover-foreground`, `cursor-default`. Under nova the
/// row carries the `popover` fill + `z-10` (painted over the viewport);
/// under base it is transparent. The outer corners match the frame radius.
fn paint_scroll_button(
    ui: &mut Ui,
    id: Id,
    up: bool,
    p: &Palette,
    nova: bool,
    frame_corner: CornerRadius,
) {
    let width = ui.available_width();
    let (_, rect) = ui.allocate_space(Vec2::new(width, SCROLL_BUTTON_HEIGHT));
    let response = ui.interact(rect, id, Sense::hover());
    if ui.is_rect_visible(rect) {
        let corner = if up {
            CornerRadius {
                nw: frame_corner.nw,
                ne: frame_corner.ne,
                sw: 0,
                se: 0,
            }
        } else {
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: frame_corner.sw,
                se: frame_corner.se,
            }
        };
        if nova {
            ui.painter().rect_filled(rect, corner, p.popover);
        }
        let icon = if up {
            Icon::ChevronUp
        } else {
            Icon::ChevronDown
        };
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(ICON_SIZE));
        icon.paint(
            ui.painter(),
            icon_rect,
            p.popover_foreground,
            2.0 * ICON_SIZE / 24.0,
        );
    }
    let _ = response;
}

/// A `SelectItem` row: highlight fill follows focus only — `focus:bg-accent`
/// with `accent-foreground` text — hover alone never fills a row (§8.1).
/// Disabled rows render at 50% opacity and ignore clicks. The check
/// indicator renders iff the item is selected. The label is clipped to the
/// text zone (the right 32px stays clear for the check); long labels clip
/// rather than ellipsize (E1/OPEN-17 — defined behaviour for S5 to verify).
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
    scroll_into_view: bool,
) -> Option<String> {
    let nova = theme.style == Style::Nova;
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), item_height(nova)));
    let sense = if item.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(rect, row_id, sense);
    let fill = if highlighted && !item.disabled {
        p.accent
    } else {
        Color32::TRANSPARENT
    };
    let fg = if highlighted {
        p.accent_foreground
    } else {
        p.popover_foreground
    };
    let fg = if item.disabled {
        color_alpha(fg, 0.5)
    } else {
        fg
    };
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if fill != Color32::TRANSPARENT {
            painter.rect_filled(
                rect,
                CornerRadius::same(item_radius(nova, &theme.radius)),
                fill,
            );
        }
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let pad_x = item_pad_x(nova);
        let zone = Rect::from_min_max(
            Pos2::new(rect.min.x + pad_x, rect.min.y),
            Pos2::new(rect.max.x - 32.0, rect.max.y),
        );
        painter.with_clip_rect(zone).text(
            Pos2::new(rect.min.x + pad_x, rect.center().y),
            Align2::LEFT_CENTER,
            item.label.clone(),
            font,
            fg,
        );
        if selected {
            let centre = Pos2::new(rect.max.x - 8.0 - indicator_offset(nova), rect.center().y);
            let check_rect = Rect::from_center_size(centre, Vec2::splat(ICON_SIZE));
            Icon::Check.paint(painter, check_rect, fg, 2.0 * ICON_SIZE / 24.0);
        }
    }
    // Highlight scroll-follow (OPEN-7/N3): keep the highlighted row in
    // view with the 4px popper scroll margin. Gated on a highlight change
    // so wheel exploration is not stomped.
    if scroll_into_view && highlighted {
        ui.scroll_to_rect(rect.expand2(Vec2::new(0.0, SCROLL_MARGIN)), None);
    }
    if open && response.clicked() && !item.disabled {
        Some(item.value.clone())
    } else {
        None
    }
}

/// A `SelectLabel` row: `text-xs text-muted-foreground`, never
/// keyboard-focusable (§2.12).
fn paint_label(ui: &mut Ui, text: &str, p: &Palette, theme: &ShadcnTheme) {
    let nova = theme.style == Style::Nova;
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), label_height(nova)));
    if ui.is_rect_visible(rect) {
        let font = FontId::proportional(crate::theme::tokens::type_scale::XS);
        ui.painter().text(
            Pos2::new(rect.min.x + label_pad_x(nova), rect.center().y),
            Align2::LEFT_CENTER,
            text.to_owned(),
            font,
            p.muted_foreground,
        );
    }
}

/// A `SelectSeparator` row: `-mx-1 my-1 h-px bg-border` (9px, both recipes).
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
    fn trigger_endpoints_light_base() {
        let p = palette(Mode::Light);
        // Placeholder, valid: transparent fill, input border, ring target,
        // muted text, 50% chevron (base `opacity-50`).
        let c = resolve_trigger_endpoints(&p, false, false, false, false);
        assert_eq!(c.fill_rest_alpha, 0.0);
        assert_eq!(c.fill_hover_alpha, 0.0, "no light hover fill");
        assert_eq!(c.rest_stroke, p.input);
        assert_eq!(c.active_stroke, p.ring);
        assert_eq!(c.ring_target, color_alpha(p.ring, 0.5));
        assert_eq!(c.text, p.muted_foreground, "placeholder look");
        assert_eq!(c.chevron, color_alpha(p.muted_foreground, 0.5));
        // Selected: foreground text.
        let sel = resolve_trigger_endpoints(&p, false, false, false, true);
        assert_eq!(sel.text, p.foreground);
        // Invalid: full destructive border + /20 ring.
        let inv = resolve_trigger_endpoints(&p, false, false, true, true);
        assert_eq!(inv.active_stroke, p.destructive);
        assert_eq!(inv.ring_target, color_alpha(p.destructive, 0.2));
    }

    #[test]
    fn trigger_endpoints_light_nova() {
        let p = palette(Mode::Light);
        let c = resolve_trigger_endpoints(&p, false, true, false, false);
        assert_eq!(c.fill_rest_alpha, 0.0);
        assert_eq!(c.fill_hover_alpha, 0.0);
        assert_eq!(
            c.chevron, p.muted_foreground,
            "nova chevron is full-opacity muted (OPEN-1)"
        );
        let inv = resolve_trigger_endpoints(&p, false, true, true, true);
        assert_eq!(inv.active_stroke, p.destructive, "light: no /50 variant");
        assert_eq!(inv.ring_target, color_alpha(p.destructive, 0.2));
    }

    #[test]
    fn trigger_endpoints_dark_fills_and_invalid() {
        let p = palette(Mode::Dark);
        let c = resolve_trigger_endpoints(&p, true, true, false, true);
        assert_eq!(c.fill_rest_alpha, 0.3, "dark:bg-input/30");
        assert_eq!(c.fill_hover_alpha, 0.5, "dark:hover:bg-input/50");
        // Nova dark invalid: /50 border + /40 ring.
        let inv = resolve_trigger_endpoints(&p, true, true, true, true);
        assert_eq!(
            inv.active_stroke,
            color_alpha(p.destructive, 0.5),
            "nova dark:aria-invalid:border-destructive/50"
        );
        assert_eq!(inv.ring_target, color_alpha(p.destructive, 0.4));
        // Base dark invalid: the /50 border is nova-only.
        let inv_base = resolve_trigger_endpoints(&p, true, false, true, true);
        assert_eq!(inv_base.active_stroke, p.destructive);
        assert_eq!(inv_base.ring_target, color_alpha(p.destructive, 0.4));
    }

    #[test]
    fn geometry_helpers_match_both_recipes() {
        let r = RadiusTokens::new(10.0);
        // Trigger heights: base h-9/h-8, nova h-8/h-7.
        assert_eq!(SelectSize::Default.height(), 36.0);
        assert_eq!(SelectSize::Sm.height(), 32.0);
        assert_eq!(SelectSize::Default.height_for(Style::Nova), 32.0);
        assert_eq!(SelectSize::Sm.height_for(Style::Nova), 28.0);
        assert_eq!(SelectSize::Default.height_for(Style::Vega), 36.0);
        assert_eq!(SELECT_TRIGGER_HEIGHT_DEFAULT, 36.0);
        assert_eq!(SELECT_TRIGGER_HEIGHT_SM, 32.0);
        // Trigger radii: base rounded-md both sizes; nova rounded-lg
        // default, min(md, 10px) sm — the two nova sizes MUST differ.
        assert_eq!(trigger_radius(SelectSize::Default, false, &r), 8);
        assert_eq!(trigger_radius(SelectSize::Sm, false, &r), 8);
        assert_eq!(trigger_radius(SelectSize::Default, true, &r), 10);
        assert_eq!(trigger_radius(SelectSize::Sm, true, &r), 8);
        // The sm cap resolves the token: a raised --radius still caps at 10.
        let big = RadiusTokens::new(16.0);
        assert_eq!(trigger_radius(SelectSize::Sm, true, &big), 10);
        assert_eq!(trigger_radius(SelectSize::Default, true, &big), 16);
        // Padding + gap: base px-3/gap-2, nova pl-2.5 pr-2/gap-1.5.
        assert_eq!(trigger_padding(false), (12.0, 12.0));
        assert_eq!(trigger_padding(true), (10.0, 8.0));
        assert_eq!(trigger_gap(false), 8.0);
        assert_eq!(trigger_gap(true), 6.0);
        // Items: base 32px/pl-2/rounded-sm, nova 28px/pl-1.5/rounded-md.
        assert_eq!(item_height(false), 32.0);
        assert_eq!(item_height(true), 28.0);
        assert_eq!(item_pad_x(false), 8.0);
        assert_eq!(item_pad_x(true), 6.0);
        assert_eq!(item_radius(false, &r), 6);
        assert_eq!(item_radius(true, &r), 8);
        // Labels: base 24px/px-2, nova 20px/px-1.5.
        assert_eq!(label_height(false), 24.0);
        assert_eq!(label_height(true), 20.0);
        assert_eq!(label_pad_x(false), 8.0);
        assert_eq!(label_pad_x(true), 6.0);
        // Content: base min-w-[8rem]/rounded-md, nova min-w-36/rounded-lg.
        assert_eq!(SELECT_MIN_WIDTH, 144.0, "nova min-w-36");
        assert_eq!(content_min_width(false), 128.0);
        assert_eq!(content_min_width(true), 144.0);
        assert_eq!(content_radius(false, &r), 8);
        assert_eq!(content_radius(true, &r), 10);
        // Check indicator centre: 7px in (base 14px box), 8px (nova 16px).
        assert_eq!(indicator_offset(false), 7.0);
        assert_eq!(indicator_offset(true), 8.0);
        assert_eq!(SEPARATOR_HEIGHT, 9.0);
        assert_eq!(SCROLL_BUTTON_HEIGHT, 24.0);
    }

    #[test]
    fn flatten_entries_inserts_nova_group_padding_only() {
        let grouped = vec![SelectEntry::Group(vec![
            SelectEntry::Label("G".to_string()),
            SelectEntry::Item(SelectItem::new("A")),
        ])];
        // No groups: borrowed, no allocation.
        let plain = entries();
        assert!(matches!(flatten_entries(&plain, true), Cow::Borrowed(_)));
        // Nova: 4px pads around the group.
        let flat = flatten_entries(&grouped, true);
        assert_eq!(flat.len(), 4);
        assert!(matches!(flat[0], SelectEntry::GroupPad));
        assert!(matches!(flat[1], SelectEntry::Label(_)));
        assert!(matches!(flat[2], SelectEntry::Item(_)));
        assert!(matches!(flat[3], SelectEntry::GroupPad));
        // Base: no group rule, renders flat.
        let flat_base = flatten_entries(&grouped, false);
        assert_eq!(flat_base.len(), 2);
        assert!(matches!(flat_base[0], SelectEntry::Label(_)));
        assert!(matches!(flat_base[1], SelectEntry::Item(_)));
        // Groups still count one actionable item for navigation.
        assert_eq!(count_actionable(&flat), 1);
    }

    #[test]
    fn rows_height_counts_recipe_rows_and_group_pads() {
        let grouped = vec![SelectEntry::Group(vec![
            SelectEntry::Label("G".to_string()),
            SelectEntry::Item(SelectItem::new("A")),
            SelectEntry::Item(SelectItem::new("B")),
        ])];
        let flat_nova = flatten_entries(&grouped, true);
        assert_eq!(
            rows_height(&flat_nova, true),
            4.0 + 20.0 + 28.0 + 28.0 + 4.0
        );
        let flat_base = flatten_entries(&grouped, false);
        assert_eq!(rows_height(&flat_base, false), 24.0 + 32.0 + 32.0);
        // Separators are 9px in both recipes.
        let sep = vec![SelectEntry::Separator];
        assert_eq!(rows_height(&sep, true), 9.0);
        assert_eq!(rows_height(&sep, false), 9.0);
    }

    #[test]
    fn rendered_row_pitch_matches_height_model() {
        // SEL-VIS-02: the harness installs the theme, so the global 8px
        // `item_spacing` is live — exactly the pre-fix conditions. The
        // ScrollArea content height must equal the pure-height model
        // (`rows_height`), in both recipes, with every entry kind
        // present (label, items, separator, nova group pads). Any leaked
        // spacing would add 8px per entry and fail this assertion.
        for (style, nova) in [(Style::Nova, true), (Style::Vega, false)] {
            let theme = Preset::default().with_style(style).build(Mode::Light);
            let grouped = vec![
                SelectEntry::Group(vec![
                    SelectEntry::Label("G".to_string()),
                    SelectEntry::Item(SelectItem::new("A")),
                    SelectEntry::Item(SelectItem::new("B")),
                ]),
                SelectEntry::Item(SelectItem::new("C")),
                SelectEntry::Separator,
                SelectEntry::Item(SelectItem::new("D")),
            ];
            let flat = flatten_entries(&grouped, nova).into_owned();
            let expected = rows_height(&flat, nova);
            let mut harness = Harness::new();
            let mut content_h = f32::NAN;
            let mut first_bottom = f32::NAN;
            let mut second_top = f32::NAN;
            let popup_id = Id::unique("vis02_pitch");
            // Two frames: row rects are registered by `paint_item` under
            // `popup_id.with(("row", action_idx))` and readable via
            // `read_response` only on the following frame.
            for frame in 0..2 {
                harness.run(|ui| {
                    let mut activated = None;
                    let output = render_rows(
                        ui,
                        &flat,
                        popup_id,
                        2000.0,
                        None,
                        &theme.palette,
                        &theme,
                        None,
                        None,
                        true,
                        false,
                        &mut activated,
                    );
                    content_h = output.content_size.y;
                    if frame == 1 {
                        // The first two actionable rows must be adjacent
                        // (no 8px gap between them).
                        if let Some(r) = ui.ctx().read_response(popup_id.with(("row", 0))) {
                            first_bottom = r.rect.bottom();
                        }
                        if let Some(r) = ui.ctx().read_response(popup_id.with(("row", 1))) {
                            second_top = r.rect.top();
                        }
                    }
                });
            }
            assert_eq!(
                content_h, expected,
                "content height must equal the height model ({style:?})"
            );
            assert!(
                first_bottom.is_finite() && second_top.is_finite(),
                "row responses must be readable ({style:?})"
            );
            assert_eq!(
                second_top, first_bottom,
                "consecutive rows must be adjacent, no inter-row gap ({style:?})"
            );
        }
    }

    #[test]
    fn scroll_offset_override_accumulates_across_frames() {
        // SEL-INT-02 / SEL-FLK-01: the scroll-button auto-repeat drive
        // applies `previous clamped offset + one step` as an override
        // every frame. The clamped offset must therefore advance by one
        // step per frame and the viewport must stay clamped to its max
        // height. With a horizontal-only ScrollArea the inner rect
        // expands to the full content height, `max_offset.y` is 0, and
        // the clamped offset is stuck at 0 — the single-step symptom.
        for (style, nova) in [(Style::Nova, true), (Style::Vega, false)] {
            let theme = Preset::default().with_style(style).build(Mode::Light);
            let flat: Vec<SelectEntry> = (0..40)
                .map(|i| SelectEntry::Item(SelectItem::new(format!("Option {i}"))))
                .collect();
            let rows_h = rows_height(&flat, nova);
            let max_height = 200.0;
            assert!(rows_h > max_height, "list must overflow the viewport");
            let mut harness = Harness::new();
            let popup_id = Id::unique("int02_accumulate");
            let mut prev = 0.0f32;
            for step in 1..=6 {
                let mut clamped = f32::NAN;
                let mut inner_h = f32::NAN;
                let override_offset = prev + 12.0;
                harness.run(|ui| {
                    let mut activated = None;
                    let output = render_rows(
                        ui,
                        &flat,
                        popup_id,
                        max_height,
                        Some(override_offset),
                        &theme.palette,
                        &theme,
                        None,
                        None,
                        true,
                        false,
                        &mut activated,
                    );
                    clamped = output.state.clamped_offset().y;
                    inner_h = output.inner_rect.height();
                });
                assert!(
                    inner_h <= max_height + 1.0,
                    "viewport must stay clamped to max_height ({style:?}): inner {inner_h}"
                );
                assert_eq!(
                    clamped, override_offset,
                    "clamped offset must track the override on frame {step} ({style:?})"
                );
                prev = clamped;
            }
            assert_eq!(prev, 72.0, "offset must accumulate over frames");
        }
    }

    #[test]
    fn wheel_scrolls_the_viewport() {
        // SEL-INT-02 (M8): a wheel event over the viewport must move the
        // vertical offset — down increases it, up decreases it. With a
        // horizontal-only ScrollArea the wheel delta is consumed by the
        // horizontal axis and the vertical offset never moves.
        for (style, _nova) in [(Style::Nova, true), (Style::Vega, false)] {
            let theme = Preset::default().with_style(style).build(Mode::Light);
            let flat: Vec<SelectEntry> = (0..40)
                .map(|i| SelectEntry::Item(SelectItem::new(format!("Option {i}"))))
                .collect();
            let mut harness = Harness::new();
            let popup_id = Id::unique("int02_wheel");
            let render = |harness: &mut Harness, offset: &mut f32| {
                harness.run(|ui| {
                    let mut activated = None;
                    let output = render_rows(
                        ui,
                        &flat,
                        popup_id,
                        200.0,
                        None,
                        &theme.palette,
                        &theme,
                        None,
                        None,
                        true,
                        false,
                        &mut activated,
                    );
                    *offset = output.state.clamped_offset().y;
                });
            };
            // Settle at a mid offset via the override, so both wheel
            // directions have room to move.
            let mut offset = f32::NAN;
            harness.run(|ui| {
                let mut activated = None;
                let output = render_rows(
                    ui,
                    &flat,
                    popup_id,
                    200.0,
                    Some(100.0),
                    &theme.palette,
                    &theme,
                    None,
                    None,
                    true,
                    false,
                    &mut activated,
                );
                offset = output.state.clamped_offset().y;
            });
            assert_eq!(offset, 100.0, "override must set the offset ({style:?})");
            // Wheel down (negative point delta) over the viewport.
            harness.events(
                vec![
                    egui::Event::PointerMoved(Pos2::new(100.0, 100.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: Vec2::new(0.0, -120.0),
                        phase: egui::TouchPhase::Move,
                        source: egui::MouseWheelSource::Wheel,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
                |ui| {
                    let mut activated = None;
                    let output = render_rows(
                        ui,
                        &flat,
                        popup_id,
                        200.0,
                        None,
                        &theme.palette,
                        &theme,
                        None,
                        None,
                        true,
                        false,
                        &mut activated,
                    );
                    offset = output.state.clamped_offset().y;
                },
            );
            // Let kinetic scrolling settle over a few frames.
            for _ in 0..3 {
                render(&mut harness, &mut offset);
            }
            assert!(
                offset > 100.0,
                "wheel down must increase the offset ({style:?}): {offset}"
            );
            let after_down = offset;
            // Wheel up over the viewport.
            harness.events(
                vec![
                    egui::Event::PointerMoved(Pos2::new(100.0, 100.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: Vec2::new(0.0, 120.0),
                        phase: egui::TouchPhase::Move,
                        source: egui::MouseWheelSource::Wheel,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
                |ui| {
                    let mut activated = None;
                    let output = render_rows(
                        ui,
                        &flat,
                        popup_id,
                        200.0,
                        None,
                        &theme.palette,
                        &theme,
                        None,
                        None,
                        true,
                        false,
                        &mut activated,
                    );
                    offset = output.state.clamped_offset().y;
                },
            );
            for _ in 0..3 {
                render(&mut harness, &mut offset);
            }
            assert!(
                offset < after_down,
                "wheel up must decrease the offset ({style:?}): {offset} after {after_down}"
            );
        }
    }

    #[test]
    fn popup_layout_is_content_sized_without_a_ceiling() {
        // One nova option: 28px row + 8px padding, no buttons.
        let one = popup_layout(28.0, 554.0, 0.0);
        assert!(!one.show_up && !one.show_down);
        assert_eq!(one.viewport_height, 28.0);
        assert_eq!(one.total_height, 36.0);
        // Empty list: padding only (OPEN-15 — defined, no crash).
        let empty = popup_layout(0.0, 554.0, 0.0);
        assert_eq!(empty.total_height, 8.0);
        // 40 nova options at the top: down button only, viewport clamped
        // by the available height — no fixed 300px ceiling.
        let top = popup_layout(1120.0, 554.0, 0.0);
        assert!(!top.show_up && top.show_down);
        assert_eq!(top.viewport_height, 522.0);
        assert_eq!(top.total_height, 554.0);
        // Scrolled to the middle: both buttons.
        let mid = popup_layout(1120.0, 554.0, 100.0);
        assert!(mid.show_up && mid.show_down);
        assert_eq!(mid.viewport_height, 498.0);
        assert_eq!(mid.total_height, 554.0);
        // Scrolled to the bottom: up button only.
        let bottom = popup_layout(1120.0, 554.0, 622.0);
        assert!(bottom.show_up && !bottom.show_down);
        assert_eq!(bottom.viewport_height, 522.0);
        assert_eq!(bottom.total_height, 554.0);
        // Just fits: no buttons even at the boundary.
        let fits = popup_layout(546.0, 554.0, 0.0);
        assert!(!fits.show_up && !fits.show_down);
        assert_eq!(fits.total_height, 554.0);
    }

    #[test]
    fn open_highlight_keeps_the_k1_k2_asymmetry() {
        let e = entries();
        // K2: Enter opens on the first item, even with a selection.
        assert_eq!(open_highlight(&e, Some("carrot"), true, false), Some(0));
        // K1: Space opens on the selected item.
        assert_eq!(open_highlight(&e, Some("carrot"), false, true), Some(3));
        // Mouse/arrow opens keep the selection highlight (OPEN-12).
        assert_eq!(open_highlight(&e, Some("carrot"), false, false), Some(3));
        // No selection: Enter opens on the first item, Space on nothing.
        assert_eq!(open_highlight(&e, None, true, false), Some(0));
        assert_eq!(open_highlight(&e, None, false, true), None);
        // Both keys: Space wins (the selection).
        assert_eq!(open_highlight(&e, Some("carrot"), true, true), Some(3));
        // Enter skips a disabled first item.
        let disabled_first = vec![
            SelectEntry::Item(SelectItem::new("A").disabled(true)),
            SelectEntry::Item(SelectItem::new("B")),
        ];
        assert_eq!(open_highlight(&disabled_first, None, true, false), Some(1));
        let empty: Vec<SelectEntry> = Vec::new();
        assert_eq!(open_highlight(&empty, None, true, false), None);
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

        let mut g = SelectBuilder::new();
        g.group(|sub| {
            sub.label("G");
            sub.item(SelectItem::new("I"));
        });
        assert_eq!(g.entries.len(), 1);
        assert!(matches!(g.entries[0], SelectEntry::Group(_)));
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
                screen_rect: Some(SCREEN),
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
                    screen_rect: Some(SCREEN),
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
                screen_rect: Some(SCREEN),
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
                screen_rect: Some(SCREEN),
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

        /// Runs one frame with the given raw events (pointer moves, wheel,
        /// …), for input-driven tests such as wheel scrolling.
        fn events(&mut self, events: Vec<egui::Event>, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(SCREEN),
                events,
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

        fn popup_rect(&self, salt: &str) -> Rect {
            let popup_id = Id::unique("select_test_ui").with(salt).with("popup");
            self.ctx
                .read_response(popup_id)
                .map(|r| r.rect)
                .unwrap_or(Rect::NOTHING)
        }
    }

    const OPTS: [&str; 1] = ["Only option"];

    /// The simulated 600×600 screen: `ctx.content_rect()` (which the popup
    /// clamps against) follows `RawInput::screen_rect`, so the harness must
    /// declare the same screen its Ui claims.
    const SCREEN: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(600.0, 600.0));

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

    #[test]
    fn enter_opens_on_the_first_item() {
        let mut h = Harness::new();
        let mut value = Some("Beta".to_string());
        h.run(|ui| {
            Select::new(&mut value)
                .id_salt("k2")
                .options(["Alpha", "Beta", "Cherry"])
                .show_seeded(ui);
        });
        let trigger_id = Id::unique("select_test_ui").with("k2").with("trigger");
        h.ctx.memory_mut(|m| m.request_focus(trigger_id));
        h.key(Key::Enter, |ui| {
            Select::new(&mut value)
                .id_salt("k2")
                .options(["Alpha", "Beta", "Cherry"])
                .show_seeded(ui);
        });
        assert!(h.state("k2").open, "Enter opens the focused trigger");
        assert_eq!(
            h.highlight("k2"),
            Some(0),
            "K2: Enter highlights the first item, not the selection"
        );
    }

    #[test]
    fn space_opens_on_the_selected_item() {
        let mut h = Harness::new();
        let mut value = Some("Beta".to_string());
        h.run(|ui| {
            Select::new(&mut value)
                .id_salt("k1")
                .options(["Alpha", "Beta", "Cherry"])
                .show_seeded(ui);
        });
        let trigger_id = Id::unique("select_test_ui").with("k1").with("trigger");
        h.ctx.memory_mut(|m| m.request_focus(trigger_id));
        h.key(Key::Space, |ui| {
            Select::new(&mut value)
                .id_salt("k1")
                .options(["Alpha", "Beta", "Cherry"])
                .show_seeded(ui);
        });
        assert!(h.state("k1").open, "Space opens the focused trigger");
        assert_eq!(
            h.highlight("k1"),
            Some(1),
            "K1: Space highlights the selected item"
        );
    }

    #[test]
    fn escape_closes_and_keeps_trigger_focus() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "escf");
        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("escf")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("escf").open);
        // Focus the trigger and let the focus-lock filter arm: it only
        // stores once the widget had focus last frame too.
        let trigger_id = Id::unique("select_test_ui").with("escf").with("trigger");
        h.ctx.memory_mut(|m| m.request_focus(trigger_id));
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("escf")
                .options(OPTS)
                .show_seeded(ui);
        });
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("escf")
                .options(OPTS)
                .show_seeded(ui);
        });
        h.key(Key::Escape, |ui| {
            Select::uncontrolled()
                .id_salt("escf")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("escf").open, "Escape must close the select");
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger_id),
            "K7: Escape keeps focus on the trigger"
        );
    }

    #[test]
    fn popup_is_content_sized_for_one_option() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "sz1");
        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("sz1")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("sz1").open);
        // Let the popup settle (sizing pass + 100ms enter animation).
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("sz1")
                .options(OPTS)
                .show_seeded(ui);
        });
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("sz1")
                .options(OPTS)
                .show_seeded(ui);
        });
        let popup = h.popup_rect("sz1");
        assert!(popup != Rect::NOTHING, "popup must be on screen");
        assert!(
            (popup.height() - 36.0).abs() < 3.0,
            "one nova option: 28px row + 8px padding, no empty area (height={})",
            popup.height()
        );
        assert!(
            ((popup.min.y - rect.max.y) - 4.0).abs() < 2.0,
            "rest gap is the 4px popper gap (gap={})",
            popup.min.y - rect.max.y
        );
    }

    #[test]
    fn popup_clamps_to_available_height_for_long_lists() {
        let mut h = Harness::new();
        let options: Vec<String> = (0..40).map(|i| format!("Option {i}")).collect();
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = Select::uncontrolled()
                .id_salt("sz40")
                .options(options.clone())
                .show_seeded(ui)
                .trigger
                .rect;
        });
        h.run(|ui| {
            rect = Select::uncontrolled()
                .id_salt("sz40")
                .options(options.clone())
                .show_seeded(ui)
                .trigger
                .rect;
        });
        h.click(rect.center(), |ui| {
            Select::uncontrolled()
                .id_salt("sz40")
                .options(options.clone())
                .show_seeded(ui);
        });
        assert!(h.state("sz40").open);
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("sz40")
                .options(options.clone())
                .show_seeded(ui);
        });
        h.run(|ui| {
            Select::uncontrolled()
                .id_salt("sz40")
                .options(options.clone())
                .show_seeded(ui);
        });
        let popup = h.popup_rect("sz40");
        assert!(popup != Rect::NOTHING, "popup must be on screen");
        // Available: 600 - 32 (trigger) - 4 (gap) - 10 (collision) = 554.
        assert!(
            (popup.height() - 554.0).abs() < 3.0,
            "long list clamps to the available height, not a fixed ceiling (height={})",
            popup.height()
        );
        assert!(
            popup.height() > 300.0,
            "the old 300px cap is gone (height={})",
            popup.height()
        );
        assert!(
            popup.max.y <= 590.0 + 1.0,
            "popup respects the 10px collision padding (max.y={})",
            popup.max.y
        );
    }

    #[test]
    fn group_popup_height_counts_nova_group_padding() {
        let mut h = Harness::new();
        let show = |ui: &mut Ui| {
            Select::uncontrolled().id_salt("grp").show(ui, |s| {
                s.group(|g| {
                    g.label("Frameworks");
                    g.item(SelectItem::new("Next.js"));
                    g.item(SelectItem::new("Astro"));
                });
            })
        };
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = show(ui).trigger.rect;
        });
        h.run(|ui| {
            rect = show(ui).trigger.rect;
        });
        h.click(rect.center(), |ui| {
            show(ui);
        });
        assert!(h.state("grp").open);
        h.run(|ui| {
            show(ui);
        });
        h.run(|ui| {
            show(ui);
        });
        let popup = h.popup_rect("grp");
        assert!(popup != Rect::NOTHING, "popup must be on screen");
        // Rows: 4 (group pad) + 20 (label) + 28 + 28 (items) + 4 (pad) =
        // 84, plus 8px viewport padding.
        assert!(
            (popup.height() - 92.0).abs() < 3.0,
            "group pads count in the content-sized height (height={})",
            popup.height()
        );
    }
}
