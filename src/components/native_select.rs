//! shadcn `native-select.tsx` — a styled `<select>` replacement.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/native-select.tsx`: an
//! `h-9 rounded-md border border-input bg-transparent px-3 pr-9 text-sm
//! shadow-xs` trigger (`data-[size=sm]:h-8`) with a chevron-down icon at
//! `right-3.5` (`text-muted-foreground opacity-50`), the shared
//! focus/invalid/disabled styles and `dark:bg-input/30 dark:hover:bg-input/50`.
//!
//! The nova style (`.cn-native-select`) reshapes the trigger to `h-8
//! rounded-lg py-1 pr-8 pl-2.5` (sm: `h-7 rounded-[min(var(--radius-md),10px)] py-0.5`)
//! with the chevron at `right-2.5`. The disabled wrapper rule
//! (`has-[select:disabled]:opacity-50`) composites the WHOLE trigger — fill,
//! border, text, chevron, shadow — at 50% over the backdrop (group fade;
//! there are no separate disabled colours, and disabled text keeps the
//! `foreground` slot — N11).
//!
//! The browser's native dropdown list is replaced by an [`egui::Popup`]
//! (deviation D1 — port design, never original behaviour): `popover` fill,
//! 36px rows, `accent` hover/selected/highlight rows, `shadow-md`, max 8
//! visible rows with internal scrolling, optgroup headers, and trigger-width
//! sizing. The full decided design (12 coordinator values) is recorded in
//! the S1 report; the trigger itself matches the frozen spec exactly.
//!
//! Keyboard (§7.1): Tab focus, click to open, arrows change the selection
//! while closed and move the highlight while open, Home/End/PageUp/PageDown
//! while open, Enter confirms, Escape closes (selection unchanged),
//! type-ahead jumps to the next matching option. Space while open,
//! arrow wrap, and Alt+Arrow behaviour stay OPEN-1 (no guessed behaviour).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut theme = Some(0);
//! NativeSelect::new(&mut theme)
//!     .option("Light")
//!     .option("Dark")
//!     .option("System")
//!     .id_salt("theme")
//!     .show(ui);
//! ```

use crate::components::attachment::mix;
use crate::components::button::mix_color;
use crate::icons::Icon;
use crate::theme::{blend, color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use crate::utils::animation::{
    transition_alpha, transition_color_to, transition_progress, Easing, TransitionSpec,
};
use crate::utils::focus::paint_ring;
use crate::utils::shadow::{
    paint_shadow_layers, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Align2, Color32, Context, CornerRadius, CursorIcon, FontId, Id, Key, Painter, Pos2, Rect,
    Response, Sense, Stroke, Ui, Vec2,
};

/// Port-invented trigger width floor (spec §3.3(4)): no snapshot rule states
/// it; kept from the pre-rebuild code as a named constant per the S1 brief.
pub const NATIVE_SELECT_MIN_WIDTH: f32 = 160.0;
/// Popup row height (D1 decision 2: KEEP 36px rows; gaps-B NS-P2-1 rejected).
const POPUP_ROW_HEIGHT: f32 = 36.0;
/// Popup max visible rows (D1 decision 6: max 8 visible rows + internal
/// `ScrollArea`; also the PageUp/PageDown page size).
const POPUP_MAX_VISIBLE_ROWS: usize = 8;
/// Popup row left text pad (D1 decision 2: KEEP the current 8px pad).
const POPUP_ROW_PAD_LEFT: f32 = 8.0;
/// Popup frame inner margin (D1 decision 11: XS margins).
const POPUP_MARGIN: f32 = 4.0;
/// Type-ahead idle timeout (OPEN-1 working algorithm: 1s, the menu/select
/// precedent; the exact timeout is unsettled).
const TYPEAHEAD_TIMEOUT_SECS: f64 = 1.0;
/// Chevron icon size (`size-4`).
const CHEVRON_SIZE: f32 = 16.0;
/// Trigger/row text size (`text-sm`).
const TEXT_SIZE: f32 = 14.0;

/// Trigger height per size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NativeSelectSize {
    /// `h-9` (36px).
    #[default]
    Default,
    /// `h-8` (32px).
    Sm,
}

impl NativeSelectSize {
    /// Trigger height in points (base recipe `h-9`/`h-8`; the nova style
    /// renders 32px/28px instead).
    pub fn height(self) -> f32 {
        match self {
            NativeSelectSize::Default => crate::theme::tokens::spacing::CONTROL,
            NativeSelectSize::Sm => crate::theme::tokens::spacing::CONTROL_SM,
        }
    }
}

/// Outer height (`h-9`/`h-8` base recipe; nova `.cn-native-select` `h-8`,
/// sm `h-7`).
fn trigger_height(size: NativeSelectSize, style: Style) -> f32 {
    match style {
        Style::Nova => match size {
            NativeSelectSize::Default => 32.0, // h-8
            NativeSelectSize::Sm => 28.0,      // h-7
        },
        _ => size.height(),
    }
}

/// Corner radius (`rounded-md` base recipe; nova `rounded-lg`, sm
/// `rounded-[min(var(--radius-md),10px)]`).
fn trigger_radius(size: NativeSelectSize, style: Style, radius: &RadiusTokens) -> u8 {
    match style {
        Style::Nova => match size {
            NativeSelectSize::Default => radius.lg_u8(),
            NativeSelectSize::Sm => (radius.md().round() as i32).clamp(0, 10) as u8,
        },
        _ => radius.md_u8(),
    }
}

/// Left padding (`pl-3` base recipe; nova `pl-2.5`).
fn padding_left(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0,
        _ => 12.0,
    }
}

/// Right padding (`pr-9` base recipe; nova `pr-8`). The text zone ends at
/// this inset — exactly 6px before the chevron's left edge in both styles
/// (spec §3.2).
fn padding_right(style: Style) -> f32 {
    match style {
        Style::Nova => 32.0,
        _ => 36.0,
    }
}

/// Chevron inset from the right edge (`right-3.5` base recipe; nova
/// `right-2.5`), before the half icon width.
fn chevron_right_inset(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0, // right-2.5
        _ => 14.0,           // right-3.5
    }
}

/// Trigger width padding: `(pr-9 + pl-3)` base recipe; nova `(pr-8 + pl-2.5)`.
fn width_padding(style: Style) -> f32 {
    padding_left(style) + padding_right(style)
}

/// One dropdown option (`NativeSelectOption`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    /// The visible label.
    pub label: String,
    /// `disabled` — the option cannot be picked.
    pub disabled: bool,
}

impl SelectOption {
    /// Creates an enabled option.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    /// Creates a disabled option.
    pub fn disabled(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: true,
        }
    }
}

/// One option group (`NativeSelectOptGroup`, N10). A group's `disabled` flag
/// disables every option in the group (HTML `optgroup disabled` semantics,
/// spec §10); the header itself is never interactive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectOptGroup {
    /// The group header label (required by HTML for the header).
    pub label: String,
    /// `disabled` — disables the whole group.
    pub disabled: bool,
    /// The group's options, in order.
    pub options: Vec<SelectOption>,
}

impl SelectOptGroup {
    /// Creates an empty group with the given header label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            options: Vec::new(),
        }
    }

    /// Creates a group with options.
    pub fn with_options(
        label: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption>,
    ) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            options: options.into_iter().collect(),
        }
    }

    /// Disables the whole group.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Adds an enabled option to the group.
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options.push(SelectOption::new(label));
        self
    }

    /// Adds a disabled option to the group.
    pub fn option_disabled(mut self, label: impl Into<String>) -> Self {
        self.options.push(SelectOption::disabled(label));
        self
    }
}

/// One popup entry: a flat option or a group. Flat selection indices count
/// OPTIONS only, in entry order; headers consume no index.
#[derive(Clone, Debug)]
enum SelectEntry {
    Option(SelectOption),
    Group(SelectOptGroup),
}

/// Resolved trigger endpoints for one state (pure, unit-testable). Animated
/// values ride the transition specs in [`show`](NativeSelect::show); the
/// disabled group fade is applied AFTER animation (opacity is in neither
/// transition set, so it snaps).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerEndpoints {
    /// Resting fill: transparent (light) / `input/30` (dark), both styles.
    fill_rest: Color32,
    /// Hover fill: `input/50` (dark only); equals `fill_rest` in light.
    fill_hover: Color32,
    /// Resting border: `input`.
    rest_stroke: Color32,
    /// Active border: `ring` when focused, `destructive` when invalid
    /// (OPEN-2 PROVISIONAL invalid-wins), `destructive/50` for nova-dark
    /// invalid (N5 settled).
    active_stroke: Color32,
    /// Ring colour: `ring/50` when focused, `destructive/20` (light) /
    /// `destructive/40` (dark) when invalid (OPEN-2 PROVISIONAL
    /// invalid-wins).
    ring_target: Color32,
    /// Text colour: `foreground` with a selection (N11: the slot stays
    /// `foreground` when disabled — the fade composites it), `muted-foreground`
    /// for the D4 placeholder.
    text: Color32,
    /// Chevron colour: `muted-foreground` at 50%, both styles (§2.3).
    chevron: Color32,
}

pub(crate) fn resolve_trigger_endpoints(
    p: &Palette,
    dark: bool,
    nova: bool,
    invalid: bool,
    has_selection: bool,
) -> TriggerEndpoints {
    let fill_rest = if dark {
        color_alpha(p.input, 0.3)
    } else {
        Color32::TRANSPARENT
    };
    let fill_hover = if dark {
        color_alpha(p.input, 0.5)
    } else {
        fill_rest
    };
    let active_stroke = if invalid {
        if nova && dark {
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
    TriggerEndpoints {
        fill_rest,
        fill_hover,
        rest_stroke: p.input,
        active_stroke,
        ring_target,
        text: if has_selection {
            p.foreground
        } else {
            p.muted_foreground
        },
        chevron: color_alpha(p.muted_foreground, 0.5),
    }
}

/// Per-property transition specs (coordinator ruling 6, narrow reading —
/// supersedes the broad sentence in spec §6; S4 records the supersession).
/// Base `transition-[color,box-shadow]` reads NARROWLY: text colour +
/// box-shadow (ring) animate, border/fill SNAP. Nova `transition-colors`:
/// colours animate, box-shadow (ring) SNAPS. Duration/easing are the interim
/// framework defaults (150ms, cubic-bezier(0.4,0,0.2,1) — OPEN-3).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerTransitionSpecs {
    /// Text/chevron colour: animates under BOTH styles.
    text: TransitionSpec,
    /// Border/fill colour: SNAP under base, animates under nova.
    fill_border: TransitionSpec,
    /// Ring (box-shadow): animates under base, SNAP under nova.
    ring: TransitionSpec,
}

pub(crate) fn transition_specs(nova: bool) -> TriggerTransitionSpecs {
    // A zero-duration spec snaps instantly while keeping the animator state
    // fresh (toggle pattern).
    let snap = TransitionSpec::secs(0.0, Easing::EaseInOut);
    let live = TransitionSpec::default();
    if nova {
        TriggerTransitionSpecs {
            text: live,
            fill_border: live,
            ring: snap,
        }
    } else {
        TriggerTransitionSpecs {
            text: live,
            fill_border: snap,
            ring: live,
        }
    }
}

/// Whole-element group fade for `disabled:opacity-50` (spec §4.5, ruling 7):
/// the enabled trigger composites at 50% over `backdrop` (the theme page
/// background — exact on the gallery page, where the panel fill is
/// `p.background`; on other backdrops the overlap pixels use the theme
/// background as the fade target, a recorded limitation shared with toggle).
///
/// Per-slot alpha halving would double-fade the overlap pixels (a halved
/// stroke over a halved fill reads 25% at the border band — the CHK-VIS-01
/// defect class). Instead: the fill fades by halving its alpha (it paints
/// over the backdrop alone), while the stroke and text/chevron colours are
/// pre-composited OPAQUE against the backdrop: `0.5 * composite + 0.5 *
/// backdrop`, where `composite` is the enabled colour over the enabled fill
/// over the backdrop. Returns `(fill, stroke, text, chevron)`.
fn disabled_group_fade(
    fill: Color32,
    stroke_color: Color32,
    text: Color32,
    chevron: Color32,
    backdrop: Color32,
) -> (Color32, Color32, Color32, Color32) {
    let faded_fill = color_alpha(fill, 0.5);
    let fill_composite = blend(backdrop, fill);
    let stroke = mix(backdrop, blend(fill_composite, stroke_color), 0.5);
    let text = mix(backdrop, blend(fill_composite, text), 0.5);
    let chevron = mix(backdrop, blend(fill_composite, chevron), 0.5);
    (faded_fill, stroke, text, chevron)
}

/// Next/previous ENABLED flat index from `from`, skipping disabled options
/// and STOPPING at the list ends (NO wrap — OPEN-1 working assumption,
/// recorded for S6/coordinator). `from: None` + down → first enabled;
/// `None` + up → last enabled. Returns `None` when no enabled option exists
/// in the direction (caller keeps the current value).
fn step_enabled(disabled: &[bool], from: Option<usize>, dir: i32) -> Option<usize> {
    let n = disabled.len();
    if n == 0 {
        return None;
    }
    let from = from.filter(|&f| f < n);
    let mut i = match from {
        Some(f) => f as i32 + dir,
        None => {
            if dir > 0 {
                0
            } else {
                n as i32 - 1
            }
        }
    };
    while (0..n as i32).contains(&i) {
        if !disabled[i as usize] {
            return Some(i as usize);
        }
        i += dir;
    }
    None
}

/// First enabled flat index, if any.
fn first_enabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().position(|&d| !d)
}

/// Last enabled flat index, if any.
fn last_enabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().rposition(|&d| !d)
}

/// Page step: up to [`POPUP_MAX_VISIBLE_ROWS`] enabled steps in the
/// direction (the D1 page size = max visible rows), stopping at the ends.
fn page_enabled(disabled: &[bool], from: Option<usize>, dir: i32) -> Option<usize> {
    let mut cur = from;
    for _ in 0..POPUP_MAX_VISIBLE_ROWS {
        match step_enabled(disabled, cur, dir) {
            Some(next) => cur = Some(next),
            None => break,
        }
    }
    cur
}

/// Type-ahead match (OPEN-1 working algorithm): the first ENABLED option
/// AFTER `after` (wrapping once) whose label starts with `query`,
/// case-insensitive. Disabled options are skipped (§10). `after: None`
/// searches from the start.
fn typeahead_match<S: AsRef<str>>(
    labels: &[S],
    disabled: &[bool],
    after: Option<usize>,
    query: &str,
) -> Option<usize> {
    if query.is_empty() || labels.is_empty() {
        return None;
    }
    let q = query.to_lowercase();
    let n = labels.len();
    let start = after.filter(|&a| a < n).map_or(0, |a| (a + 1) % n);
    for offset in 0..n {
        let idx = (start + offset) % n;
        if !disabled[idx] && labels[idx].as_ref().to_lowercase().starts_with(&q) {
            return Some(idx);
        }
    }
    None
}

/// Printable characters typed this frame, excluding Space (Space drives
/// open/confirm, never the type-ahead buffer — OPEN-1) and control
/// characters. The caller gates on trigger focus.
fn typeahead_chars(ctx: &Context) -> Vec<char> {
    let mut chars = Vec::new();
    ctx.input(|i| {
        for event in &i.events {
            if let egui::Event::Text(text) = event {
                chars.extend(text.chars().filter(|c| !c.is_control() && *c != ' '));
            }
        }
    });
    chars
}

/// Per-instance ephemeral state (temp data, keyed by the trigger id).
#[derive(Clone, Debug, Default)]
struct NativeSelectState {
    /// Keyboard highlight while the popup is open (flat option index).
    /// Provisional: the selection commits only on Enter or click (OPEN-1).
    highlight: Option<usize>,
    /// Highlight value the popup last rendered (scroll-follow gating: the
    /// ScrollArea follows only on a highlight change, never on wheel
    /// scrolls).
    last_rendered_highlight: Option<usize>,
    /// Type-ahead buffer + time of the last typed character.
    typeahead_buffer: String,
    typeahead_at: Option<f64>,
}

fn read_ns_state(ctx: &Context, state_id: Id) -> NativeSelectState {
    ctx.data_mut(|d| {
        d.get_temp::<NativeSelectState>(state_id)
            .unwrap_or_default()
    })
}

fn write_ns_state(ctx: &Context, state_id: Id, state: &NativeSelectState) {
    ctx.data_mut(|d| d.insert_temp(state_id, state.clone()));
}

/// A shadcn native select. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct NativeSelect<'a> {
    selected: &'a mut Option<usize>,
    entries: Vec<SelectEntry>,
    placeholder: Option<String>,
    size: NativeSelectSize,
    width: Option<f32>,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> NativeSelect<'a> {
    /// Creates a select bound to the selected index (`None` = no selection).
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self {
            selected,
            entries: Vec::new(),
            placeholder: None,
            size: NativeSelectSize::Default,
            width: None,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Adds an enabled option.
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.entries
            .push(SelectEntry::Option(SelectOption::new(label)));
        self
    }

    /// Adds a disabled option.
    pub fn option_disabled(mut self, label: impl Into<String>) -> Self {
        self.entries
            .push(SelectEntry::Option(SelectOption::disabled(label)));
        self
    }

    /// Adds several options at once.
    pub fn options(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.entries.extend(
            labels
                .into_iter()
                .map(SelectOption::new)
                .map(SelectEntry::Option),
        );
        self
    }

    /// Adds an option group (`NativeSelectOptGroup`, N10). Flat and grouped
    /// options may mix in insertion order; selection indices count options
    /// only, across groups.
    pub fn optgroup(mut self, group: SelectOptGroup) -> Self {
        self.entries.push(SelectEntry::Group(group));
        self
    }

    /// Adds several option groups at once.
    pub fn optgroups(mut self, groups: impl IntoIterator<Item = SelectOptGroup>) -> Self {
        self.entries
            .extend(groups.into_iter().map(SelectEntry::Group));
        self
    }

    /// Shows faint text on the trigger when nothing is selected (D4 decided
    /// convention: placeholder text in `muted-foreground`; no placeholder
    /// supplied → empty trigger, geometry unchanged).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the trigger size (`data-size`).
    pub fn size(mut self, size: NativeSelectSize) -> Self {
        self.size = size;
        self
    }

    /// Sets an explicit trigger width. The default fits the widest option.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Disables the select (`has-[select:disabled]:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the select aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so popup open state and focus persist per
    /// instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Effective disabled flags in flat option order (a group `disabled`
    /// disables every option in the group).
    fn flat_disabled(&self) -> Vec<bool> {
        let mut out = Vec::new();
        for entry in &self.entries {
            match entry {
                SelectEntry::Option(o) => out.push(o.disabled),
                SelectEntry::Group(g) => {
                    out.extend(g.options.iter().map(|o| o.disabled || g.disabled));
                }
            }
        }
        out
    }

    /// Option labels in flat option order (owned: the keyboard block
    /// mutates the selection while the labels are live).
    fn flat_labels(&self) -> Vec<String> {
        let mut out = Vec::new();
        for entry in &self.entries {
            match entry {
                SelectEntry::Option(o) => out.push(o.label.clone()),
                SelectEntry::Group(g) => {
                    out.extend(g.options.iter().map(|o| o.label.clone()));
                }
            }
        }
        out
    }

    /// The option at a flat index, with its effective disabled flag.
    fn flat_option(&self, index: usize) -> Option<(&SelectOption, bool)> {
        let mut flat = 0;
        for entry in &self.entries {
            match entry {
                SelectEntry::Option(o) => {
                    if flat == index {
                        return Some((o, o.disabled));
                    }
                    flat += 1;
                }
                SelectEntry::Group(g) => {
                    for o in &g.options {
                        if flat == index {
                            return Some((o, o.disabled || g.disabled));
                        }
                        flat += 1;
                    }
                }
            }
        }
        None
    }

    /// The trigger label for the current selection ("" when the index is
    /// out of range).
    fn label_for(&self, index: usize) -> String {
        self.flat_option(index)
            .map(|(o, _)| o.label.clone())
            .unwrap_or_default()
    }

    /// Type-ahead (OPEN-1 working algorithm): printable characters
    /// accumulate in a 1s buffer; the search jumps to the next option whose
    /// label starts with the buffer (case-insensitive, wrapping once,
    /// skipping disabled options). A buffer of one repeated character
    /// cycles through matches on that character. While the popup is open
    /// the HIGHLIGHT moves; while closed the SELECTION commits directly.
    fn apply_typeahead(
        &mut self,
        ctx: &Context,
        labels: &[String],
        disabled: &[bool],
        state: &mut NativeSelectState,
        open: bool,
    ) {
        let chars = typeahead_chars(ctx);
        if chars.is_empty() {
            return;
        }
        let now = ctx.input(|i| i.time);
        for ch in chars {
            if state
                .typeahead_at
                .is_some_and(|t| now - t > TYPEAHEAD_TIMEOUT_SECS)
            {
                state.typeahead_buffer.clear();
            }
            state.typeahead_buffer.push(ch);
            state.typeahead_at = Some(now);
            // Repeated single character: cycle on that character alone.
            let query = if state.typeahead_buffer.chars().all(|c| c == ch) {
                ch.to_string()
            } else {
                state.typeahead_buffer.clone()
            };
            let after = if open {
                state.highlight
            } else {
                *self.selected
            };
            if let Some(idx) = typeahead_match(labels, disabled, after, &query) {
                if open {
                    state.highlight = Some(idx);
                } else {
                    *self.selected = Some(idx);
                }
            }
        }
    }

    /// Renders the select trigger and popup.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let style = theme.style;
        let nova = style == Style::Nova;
        let height = trigger_height(self.size, style);
        let ctx = ui.ctx().clone();

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_native_select"),
        };
        let state_id = id.with("ns_state");

        // Trigger width: widest option/placeholder label + horizontal
        // padding, floored at NATIVE_SELECT_MIN_WIDTH (§3.3 + port floor).
        // The placeholder joins the width computation (pre-rebuild
        // behaviour, kept so a long placeholder is not clipped).
        let font = FontId::proportional(TEXT_SIZE);
        let widest = self
            .flat_labels()
            .iter()
            .map(|label| {
                ui.painter()
                    .layout_no_wrap(label.to_string(), font.clone(), Color32::WHITE)
                    .size()
                    .x
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
            .unwrap_or_else(|| (widest + width_padding(style)).max(NATIVE_SELECT_MIN_WIDTH));

        // A disabled select is skipped by Tab (Sense::hover is not
        // focusable) and receives no pointer events (§4.5).
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (_, rect) = ui.allocate_space(Vec2::new(width, height));
        let mut response = ui.interact(rect, id, sense);

        let popup_id = egui::Popup::default_response_id(&response);
        let open_at_start = egui::Popup::is_id_open(&ctx, popup_id);
        let mut state = read_ns_state(&ctx, state_id);

        // Focus lock: while the trigger holds focus, vertical arrows +
        // Escape belong to the select, not to egui focus navigation
        // (select pattern). Tab is NOT claimed: it moves focus normally.
        if !self.disabled && response.has_focus() {
            ctx.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        tab: false,
                        horizontal_arrows: false,
                        vertical_arrows: true,
                        escape: true,
                    },
                );
            });
        }

        let disabled_flags = self.flat_disabled();
        let labels = self.flat_labels();
        let mut close_requested = false;
        let mut refocus_trigger = false;

        // ---- Keyboard (§7.1 must-implement rows; OPEN-1 rows are recorded,
        // not guessed). Modified keys (Ctrl/Cmd/Alt) never drive the
        // select: Alt+ArrowDown/Up stays OPEN-1.
        if !self.disabled && response.has_focus() {
            let modifiers = ctx.input(|i| i.modifiers);
            let plain = modifiers.is_none();
            let (down, up, home, end, page_down, page_up, enter, tab, escape) = ctx.input(|i| {
                (
                    plain && i.key_pressed(Key::ArrowDown),
                    plain && i.key_pressed(Key::ArrowUp),
                    plain && i.key_pressed(Key::Home),
                    plain && i.key_pressed(Key::End),
                    plain && i.key_pressed(Key::PageDown),
                    plain && i.key_pressed(Key::PageUp),
                    plain && i.key_pressed(Key::Enter),
                    !modifiers.ctrl
                        && !modifiers.command
                        && !modifiers.alt
                        && i.key_pressed(Key::Tab),
                    plain && i.key_pressed(Key::Escape),
                )
            });
            if open_at_start {
                // Open: arrows/Home/End/PageUp/PageDown move the HIGHLIGHT
                // only; the selection commits on Enter or click
                // (provisional-selection semantics stay OPEN-1).
                if down {
                    if let Some(next) = step_enabled(&disabled_flags, state.highlight, 1) {
                        state.highlight = Some(next);
                    }
                }
                if up {
                    if let Some(next) = step_enabled(&disabled_flags, state.highlight, -1) {
                        state.highlight = Some(next);
                    }
                }
                if home {
                    state.highlight = first_enabled(&disabled_flags);
                }
                if end {
                    state.highlight = last_enabled(&disabled_flags);
                }
                if page_down {
                    if let Some(next) = page_enabled(&disabled_flags, state.highlight, 1) {
                        state.highlight = Some(next);
                    }
                }
                if page_up {
                    if let Some(next) = page_enabled(&disabled_flags, state.highlight, -1) {
                        state.highlight = Some(next);
                    }
                }
                if enter {
                    if let Some(h) = state.highlight {
                        if !disabled_flags.get(h).copied().unwrap_or(true) {
                            *self.selected = Some(h);
                        }
                    }
                    close_requested = true;
                    refocus_trigger = true;
                }
                // Space (open): OPEN-1 — no port behaviour implemented.
                // The keyboard-click toggle is suppressed below, so Space
                // neither confirms nor closes; S6/coordinator decides.
                if escape {
                    // Selection unchanged: the highlight is provisional, so
                    // revert-vs-keep (OPEN-1) is moot — Escape keeps the
                    // pre-open value.
                    close_requested = true;
                    refocus_trigger = true;
                }
                if tab {
                    // OPEN-9: close the list; focus moves per egui default.
                    close_requested = true;
                }
                self.apply_typeahead(&ctx, &labels, &disabled_flags, &mut state, true);
            } else {
                // Closed: arrows commit the SELECTION immediately (N3 core).
                if down {
                    if let Some(next) = step_enabled(&disabled_flags, *self.selected, 1) {
                        *self.selected = Some(next);
                    }
                }
                if up {
                    if let Some(next) = step_enabled(&disabled_flags, *self.selected, -1) {
                        *self.selected = Some(next);
                    }
                }
                // Home/End/PageUp/PageDown (closed): OPEN-1 — no behaviour.
                // Space/Enter (closed): open via the click toggle below
                // (egui surfaces them as clicks on the focused trigger).
                // Escape (closed): nothing (the focus lock keeps focus).
                self.apply_typeahead(&ctx, &labels, &disabled_flags, &mut state, false);
            }
        }

        // ---- Open command (select gate): a keyboard click while open must
        // NOT toggle the popup (Enter confirms above; Space does nothing).
        // A pointer click toggles in both states. Disabled forces close.
        let keyboard_click = ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
        let open_cmd = if self.disabled || close_requested {
            Some(egui::SetOpenCommand::Bool(false))
        } else if response.clicked() && !(open_at_start && keyboard_click) {
            Some(egui::SetOpenCommand::Toggle)
        } else {
            None
        };
        let will_be_open = match open_cmd {
            Some(egui::SetOpenCommand::Bool(b)) => b,
            Some(egui::SetOpenCommand::Toggle) => !open_at_start,
            None => open_at_start,
        };
        if will_be_open && !open_at_start {
            // Just opened: highlight the current selection (when enabled),
            // else the first enabled option; reset the scroll-follow gate
            // so the first open frame scrolls the highlight into view.
            state.highlight = (*self.selected)
                .filter(|&i| !disabled_flags.get(i).copied().unwrap_or(true))
                .or_else(|| first_enabled(&disabled_flags));
            state.last_rendered_highlight = None;
        }
        if !will_be_open {
            state.last_rendered_highlight = None;
        }

        // Click focuses the trigger (egui `interact` does not move focus on
        // pointer press — select pattern; §7.1 "focuses the trigger").
        if !self.disabled && response.clicked() {
            response.request_focus();
        }
        if refocus_trigger {
            response.request_focus();
        }

        if ui.is_rect_visible(rect) {
            let specs = transition_specs(nova);
            let endpoints =
                resolve_trigger_endpoints(&p, dark, nova, self.invalid, self.selected.is_some());
            let focused = response.has_focus();
            let hovered = response.hovered() && !self.disabled;
            let radius = trigger_radius(self.size, style, &theme.radius);
            let corner = CornerRadius::same(radius);
            let painter = ui.painter();

            // `shadow-xs` via the resolve layer (LAYER: survives under
            // nova), in every state including disabled (halved layers).
            paint_trigger_shadow(painter, rect, corner, style, dark, self.disabled);

            // Fill: rest/hover endpoints ride the fill/border spec (snap
            // under base, animate under nova — ruling 6).
            let fill_target = if hovered {
                endpoints.fill_hover
            } else {
                endpoints.fill_rest
            };
            let mut fill =
                transition_color_to(&ctx, id.with("fill"), fill_target, specs.fill_border).color;
            // Border: rest/active endpoints + the focus/invalid mix ride
            // the same spec; the mix is boundary-safe (button pattern).
            let rest_stroke = transition_color_to(
                &ctx,
                id.with("border_rest"),
                endpoints.rest_stroke,
                specs.fill_border,
            )
            .color;
            let active_stroke = transition_color_to(
                &ctx,
                id.with("border_active"),
                endpoints.active_stroke,
                specs.fill_border,
            )
            .color;
            let t_border = transition_progress(
                &ctx,
                id.with("border"),
                focused || self.invalid,
                specs.fill_border,
            )
            .t;
            let mut stroke_color = mix_color(rest_stroke, active_stroke, t_border);
            // Text + chevron ride the text spec (animate under both styles).
            let mut text_color =
                transition_color_to(&ctx, id.with("text"), endpoints.text, specs.text).color;
            let mut chevron_color =
                transition_color_to(&ctx, id.with("chevron"), endpoints.chevron, specs.text).color;
            // Ring: target + alpha ramp ride the ring spec (animate under
            // base, snap under nova — ruling 6). Ring shows on any egui
            // focus (D2) or as the nova invalid rest ring (§4.4).
            let ring_shown = focused || (self.invalid && nova);
            let ring_target = transition_color_to(
                &ctx,
                id.with("ring_target"),
                endpoints.ring_target,
                specs.ring,
            )
            .color;
            let ring_alpha =
                transition_alpha(&ctx, id.with("ring"), 0.0, 1.0, ring_shown, specs.ring).alpha;
            let mut ring_color = color_alpha(ring_target, ring_alpha);

            if self.disabled {
                // Group fade AFTER animation: opacity snaps (it is in
                // neither transition set).
                let (f, s, t, c) = disabled_group_fade(
                    fill,
                    stroke_color,
                    text_color,
                    chevron_color,
                    p.background,
                );
                fill = f;
                stroke_color = s;
                text_color = t;
                chevron_color = c;
                ring_color = color_alpha(ring_color, 0.5);
            }

            if fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, corner, fill);
            }
            painter.rect_stroke(
                rect,
                corner,
                Stroke::new(1.0, stroke_color),
                egui::epaint::StrokeKind::Inside,
            );

            // Label: clipped to the text zone (S1 decision D-NS2: clip, no
            // ellipsis — §10 leaves the truncation behaviour to S1).
            let label = (*self.selected)
                .map(|i| self.label_for(i))
                .unwrap_or_else(|| self.placeholder.clone().unwrap_or_default());
            let chev_left = rect.max.x - chevron_right_inset(style) - CHEVRON_SIZE;
            let text_zone = Rect::from_min_max(
                Pos2::new(rect.min.x + padding_left(style), rect.min.y),
                Pos2::new(chev_left - 6.0, rect.max.y),
            );
            painter.with_clip_rect(text_zone).text(
                Pos2::new(rect.min.x + padding_left(style), rect.center().y),
                Align2::LEFT_CENTER,
                label,
                font,
                text_color,
            );

            // Chevron: 16px, centred, `muted-foreground` at 50%; never
            // rotates or changes on open (§4.7).
            let chev_rect = Rect::from_center_size(
                Pos2::new(chev_left + CHEVRON_SIZE * 0.5, rect.center().y),
                Vec2::splat(CHEVRON_SIZE),
            );
            Icon::ChevronDown.paint(painter, chev_rect, chevron_color, 2.0 * CHEVRON_SIZE / 24.0);

            // Focus ring: 3px, OUTSIDE the border box, on the unexpanded
            // rect (button BTN-VIS-02 lesson).
            if ring_alpha > 0.0 {
                paint_ring(painter, rect, ring_color, 3.0, radius as f32);
            }
        }

        // ---- Popup (D1 port design; all 12 values decided by the
        // coordinator — see the S1 report). Always shown so a programmatic
        // close applies even when the widget becomes disabled mid-open.
        let popup = egui::Popup::from_response(&response)
            .open_memory(open_cmd)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .width(rect.width())
            .frame(egui::Frame::new());
        let mut row_clicked = false;
        popup.show(|ui| {
            row_clicked = self.render_popup(ui, &p, dark, id, rect.width(), &mut state);
        });
        if row_clicked {
            // Clicking an option selects it, closes the list, and focus
            // stays on the trigger (§7.1).
            response.request_focus();
        }
        write_ns_state(&ctx, state_id, &state);

        if self.disabled {
            response = response.on_hover_cursor(CursorIcon::NotAllowed);
        }
        response
    }
}

/// One rendered popup row: a group header or an option (flat index).
#[derive(Clone, Debug)]
enum PopupRow {
    Header(String),
    Option {
        flat: usize,
        label: String,
        disabled: bool,
    },
}

impl<'a> NativeSelect<'a> {
    /// Popup rows in entry order (headers consume no flat index).
    fn popup_rows(&self) -> Vec<PopupRow> {
        let mut rows = Vec::new();
        let mut flat = 0;
        for entry in &self.entries {
            match entry {
                SelectEntry::Option(o) => {
                    rows.push(PopupRow::Option {
                        flat,
                        label: o.label.clone(),
                        disabled: o.disabled,
                    });
                    flat += 1;
                }
                SelectEntry::Group(g) => {
                    rows.push(PopupRow::Header(g.label.clone()));
                    for o in &g.options {
                        rows.push(PopupRow::Option {
                            flat,
                            label: o.label.clone(),
                            disabled: o.disabled || g.disabled,
                        });
                        flat += 1;
                    }
                }
            }
        }
        rows
    }

    /// Popup content: shadow + fill + stroke on the outer rect, rows (or
    /// the empty state) inside an internally-scrolling area. Returns true
    /// when a row click selected an option. D1 decisions implemented here:
    /// (1) `popover`/`popover-foreground` tokens; (2) 36px rows + 8px left
    /// pad; (3) `accent` hover/selected/highlight rows; (4) `shadow-md`
    /// direct call (no resolve-layer slot exists for the popup); (6) max 8
    /// visible rows + internal `ScrollArea`; (7) 36px non-interactive
    /// headers, `muted-foreground` 14px; (8) per-slot halving inside popup
    /// rows (NOT group fade); (9) empty list → one "No options" row;
    /// (10) trigger width; (11) radius md + XS margins + 1px `border`
    /// stroke. Positioning/flip (5) and the disabled-trigger rule (12) live
    /// in [`show`](NativeSelect::show).
    fn render_popup(
        &mut self,
        ui: &mut Ui,
        p: &Palette,
        dark: bool,
        base_id: Id,
        trigger_width: f32,
        state: &mut NativeSelectState,
    ) -> bool {
        let rows = self.popup_rows();
        // Empty list (no entries at all) → the single "No options" row. A
        // list of empty groups renders its headers (§10); only a list with
        // no entries at all takes the empty state.
        let row_count = rows.len().max(1);
        let total_rows_h = row_count as f32 * POPUP_ROW_HEIGHT;
        let content_h = total_rows_h.min(POPUP_MAX_VISIBLE_ROWS as f32 * POPUP_ROW_HEIGHT);
        let (_, outer) =
            ui.allocate_space(Vec2::new(trigger_width, content_h + 2.0 * POPUP_MARGIN));
        let theme = ShadcnTheme::of_ui(ui);
        let corner = CornerRadius::same(theme.radius.md_u8());
        paint_shadow_layers(ui.painter(), outer, corner, ShadowSize::Md.layers(), dark);
        ui.painter().rect_filled(outer, corner, p.popover);
        ui.painter().rect_stroke(
            outer,
            corner,
            Stroke::new(1.0, p.border),
            egui::epaint::StrokeKind::Inside,
        );

        let selected = *self.selected;
        let highlight = state.highlight;
        let follow = highlight != state.last_rendered_highlight;
        let mut clicked_flat: Option<usize> = None;
        let inner = outer.shrink(POPUP_MARGIN);
        // `new_child`: the outer rect is already allocated — the child Ui
        // must not allocate again (alert-dialog pattern).
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        egui::ScrollArea::vertical()
            .max_height(POPUP_MAX_VISIBLE_ROWS as f32 * POPUP_ROW_HEIGHT)
            .show(&mut child, |ui| {
                if rows.is_empty() {
                    paint_label_row(ui, "No options", p);
                } else {
                    for row in &rows {
                        match row {
                            PopupRow::Header(label) => {
                                paint_label_row(ui, label, p);
                            }
                            PopupRow::Option {
                                flat,
                                label,
                                disabled,
                            } => {
                                if render_option_row(
                                    ui,
                                    base_id.with("row").with(*flat),
                                    label,
                                    *disabled,
                                    selected == Some(*flat),
                                    highlight == Some(*flat),
                                    follow,
                                    p,
                                ) {
                                    clicked_flat = Some(*flat);
                                }
                            }
                        }
                    }
                }
            });
        state.last_rendered_highlight = highlight;
        if let Some(flat) = clicked_flat {
            *self.selected = Some(flat);
            true
        } else {
            false
        }
    }
}

/// Trigger `shadow-xs` via the resolve layer (`ShadowSlot::NativeSelect` —
/// LAYER: nova silence keeps the base value). Paints in every state; the
/// disabled group fade halves the layer alphas (toggle pattern).
fn paint_trigger_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::NativeSelect).layers();
    if disabled {
        let faded: Vec<ShadowLayer> = layers
            .iter()
            .map(|l| ShadowLayer {
                alpha: l.alpha * 0.5,
                ..*l
            })
            .collect();
        paint_shadow_layers(painter, rect, corner, &faded, dark);
    } else {
        paint_shadow_layers(painter, rect, corner, layers, dark);
    }
}

/// A non-interactive label row: an optgroup header or the "No options" empty
/// state. 36px, `muted-foreground` 14px, same left pad as option rows.
fn paint_label_row(ui: &mut Ui, label: &str, p: &Palette) {
    let (_, row_rect) = ui.allocate_space(Vec2::new(ui.available_width(), POPUP_ROW_HEIGHT));
    if ui.is_rect_visible(row_rect) {
        ui.painter().text(
            Pos2::new(row_rect.min.x + POPUP_ROW_PAD_LEFT, row_rect.center().y),
            Align2::LEFT_CENTER,
            label.to_owned(),
            FontId::proportional(TEXT_SIZE),
            p.muted_foreground,
        );
    }
}

/// One option row. Highlighted rows (keyboard) render with the same `accent`
/// treatment as hover/selected rows — the highlight must be visible, and no
/// separate highlight colour was decided (recorded S1 detail for S8).
/// Returns true when an enabled row was clicked.
#[allow(clippy::too_many_arguments)]
fn render_option_row(
    ui: &mut Ui,
    row_id: Id,
    label: &str,
    row_disabled: bool,
    selected: bool,
    highlighted: bool,
    scroll_follow: bool,
    p: &Palette,
) -> bool {
    let (_, row_rect) = ui.allocate_space(Vec2::new(ui.available_width(), POPUP_ROW_HEIGHT));
    let sense = if row_disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(row_rect, row_id, sense);
    let hovered = response.hovered() && !row_disabled;
    let active = selected || hovered || highlighted;
    let fill = if active {
        p.accent
    } else {
        Color32::TRANSPARENT
    };
    let fg = if active {
        p.accent_foreground
    } else {
        p.popover_foreground
    };
    // Disabled rows: per-slot halving (D1 decision 8) — NOT the trigger's
    // group fade (ruling 7 explicitly separates the two).
    let (fill, fg) = if row_disabled {
        (color_alpha(fill, 0.5), color_alpha(fg, 0.5))
    } else {
        (fill, fg)
    };
    if ui.is_rect_visible(row_rect) {
        let painter = ui.painter();
        if fill != Color32::TRANSPARENT {
            painter.rect_filled(row_rect, CornerRadius::same(theme_radius_sm(ui)), fill);
        }
        painter.text(
            Pos2::new(row_rect.min.x + POPUP_ROW_PAD_LEFT, row_rect.center().y),
            Align2::LEFT_CENTER,
            label.to_owned(),
            FontId::proportional(TEXT_SIZE),
            fg,
        );
    }
    // Highlight scroll-follow: minimal scroll, only on a highlight change
    // (wheel exploration is never stomped).
    if scroll_follow && highlighted {
        ui.scroll_to_rect(row_rect, None);
    }
    response.clicked() && !row_disabled
}

fn theme_radius_sm(ui: &Ui) -> u8 {
    (ShadcnTheme::of_ui(ui).radius.sm().round() as u8).max(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Key, PointerButton};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn trigger_endpoints_follow_state() {
        let p = palette(Mode::Light);
        // Resting, light: transparent fill, `input` border, placeholder text.
        let c = resolve_trigger_endpoints(&p, false, false, false, false);
        assert_eq!(c.fill_rest, Color32::TRANSPARENT);
        assert_eq!(c.fill_hover, Color32::TRANSPARENT);
        assert_eq!(c.rest_stroke, p.input);
        assert_eq!(c.text, p.muted_foreground);
        assert_eq!(c.chevron, color_alpha(p.muted_foreground, 0.5));
        // Selection: `foreground` text.
        let sel = resolve_trigger_endpoints(&p, false, false, false, true);
        assert_eq!(sel.text, p.foreground);
        // Focus: `ring` border, `ring/50` ring.
        let focused = resolve_trigger_endpoints(&p, false, false, false, true);
        assert_eq!(focused.active_stroke, p.ring);
        assert_eq!(focused.ring_target, color_alpha(p.ring, 0.5));
        // Invalid (base light): full `destructive` border, `/20` ring.
        let invalid = resolve_trigger_endpoints(&p, false, false, true, true);
        assert_eq!(invalid.active_stroke, p.destructive);
        assert_eq!(invalid.ring_target, color_alpha(p.destructive, 0.2));
        // Dark: fill `input/30`, hover `input/50`, invalid ring `/40`.
        let d = palette(Mode::Dark);
        let dark = resolve_trigger_endpoints(&d, true, false, false, true);
        assert_eq!(dark.fill_rest, color_alpha(d.input, 0.3));
        assert_eq!(dark.fill_hover, color_alpha(d.input, 0.5));
        let dark_invalid = resolve_trigger_endpoints(&d, true, false, true, true);
        assert_eq!(dark_invalid.active_stroke, d.destructive);
        assert_eq!(dark_invalid.ring_target, color_alpha(d.destructive, 0.4));
        // N5 SETTLED: nova dark invalid border is `destructive/50`;
        // nova light invalid border stays full `destructive`.
        let nova_dark = resolve_trigger_endpoints(&d, true, true, true, true);
        assert_eq!(nova_dark.active_stroke, color_alpha(d.destructive, 0.5));
        let nova_light = resolve_trigger_endpoints(&p, false, true, true, true);
        assert_eq!(nova_light.active_stroke, p.destructive);
        // Chevron slot is identical under nova (spec §2.3).
        assert_eq!(nova_light.chevron, color_alpha(p.muted_foreground, 0.5));
        assert_eq!(nova_dark.chevron, color_alpha(d.muted_foreground, 0.5));
    }

    #[test]
    fn disabled_text_keeps_foreground_slot() {
        // N11: the disabled text SLOT is `foreground` when a selection
        // exists — the 50% composite comes from the group fade, never from
        // a `muted-foreground` slot swap. The old code pinned the swap;
        // this test pins the derived rule.
        let p = palette(Mode::Light);
        let c = resolve_trigger_endpoints(&p, false, false, false, true);
        assert_eq!(c.text, p.foreground);
        // Disabled + no selection: the D4 placeholder colour, faded by the
        // group fade (not by a slot change here).
        let c = resolve_trigger_endpoints(&p, false, false, false, false);
        assert_eq!(c.text, p.muted_foreground);
    }

    #[test]
    fn disabled_group_fade_matches_the_differential_model() {
        // Synthetic opaque case, exact arithmetic: backdrop 200, fill 50,
        // stroke 100, text 80, chevron 60. Group fade =
        // 0.5 * composite + 0.5 * 200 (toggle pattern).
        let backdrop = Color32::from_rgb(200, 200, 200);
        let fill = Color32::from_rgb(50, 50, 50);
        let stroke = Color32::from_rgb(100, 100, 100);
        let text = Color32::from_rgb(80, 80, 80);
        let chevron = Color32::from_rgb(60, 60, 60);
        let (f, s, t, c) = disabled_group_fade(fill, stroke, text, chevron, backdrop);
        // Fill: halved alpha over the backdrop → 0.5*50 + 0.5*200 = 125.
        assert_eq!(blend(backdrop, f), Color32::from_rgb(125, 125, 125));
        // Stroke over fill: composite 100 → 150.
        assert_eq!(s, Color32::from_rgb(150, 150, 150));
        // Text over fill: composite 80 → 140.
        assert_eq!(t, Color32::from_rgb(140, 140, 140));
        // Chevron over fill: composite 60 → 130.
        assert_eq!(c, Color32::from_rgb(130, 130, 130));
        // Transparent fill: stroke/text precomposites fade over the
        // backdrop alone.
        let (f, s, _, _) =
            disabled_group_fade(Color32::TRANSPARENT, stroke, text, chevron, backdrop);
        assert_eq!(f, Color32::TRANSPARENT);
        assert_eq!(s, Color32::from_rgb(150, 150, 150));
        // Translucent fill (dark `input/30` shape): the precomposite
        // equals the differential formula within ±1 (double premultiplied
        // rounding; the S5 differential method allows ±1).
        let translucent = color_alpha(Color32::from_rgb(255, 255, 255), 0.3);
        let (f, s, t, _) = disabled_group_fade(translucent, stroke, text, chevron, backdrop);
        let fill_composite = blend(backdrop, translucent);
        assert_within_one(blend(backdrop, f), mix(backdrop, fill_composite, 0.5));
        assert_within_one(s, mix(backdrop, blend(fill_composite, stroke), 0.5));
        assert_within_one(t, mix(backdrop, blend(fill_composite, text), 0.5));
    }

    /// Channel-wise ±1 equality (rounding tolerance for premultiplied
    /// compositing paths).
    fn assert_within_one(a: Color32, b: Color32) {
        for (x, y) in [
            (a.r(), b.r()),
            (a.g(), b.g()),
            (a.b(), b.b()),
            (a.a(), b.a()),
        ] {
            assert!(
                (x as i32 - y as i32).abs() <= 1,
                "{a:?} vs {b:?}: channels differ by more than 1"
            );
        }
    }

    #[test]
    fn transition_specs_follow_ruling_6() {
        // Base: text + ring animate (150ms), border/fill snap.
        let base = transition_specs(false);
        assert_eq!(base.text.duration_secs, 0.15);
        assert_eq!(base.fill_border.duration_secs, 0.0);
        assert_eq!(base.ring.duration_secs, 0.15);
        // Nova: text + border/fill animate, ring (box-shadow) snaps.
        let nova = transition_specs(true);
        assert_eq!(nova.text.duration_secs, 0.15);
        assert_eq!(nova.fill_border.duration_secs, 0.15);
        assert_eq!(nova.ring.duration_secs, 0.0);
    }

    #[test]
    fn trigger_shadow_resolves_xs_under_both_styles() {
        // LAYER: nova silence keeps the base `shadow-xs`.
        assert_eq!(
            resolve_shadow(ShadowSize::Xs, Style::Nova, ShadowSlot::NativeSelect),
            ShadowSize::Xs
        );
        assert_eq!(
            resolve_shadow(ShadowSize::Xs, Style::Vega, ShadowSlot::NativeSelect),
            ShadowSize::Xs
        );
    }

    #[test]
    fn trigger_geometry_follows_style() {
        let radius = Preset::default().build(Mode::Light).radius;
        // Nova `h-8` / sm `h-7`; base recipe `h-9` / `h-8`.
        assert_eq!(trigger_height(NativeSelectSize::Default, Style::Nova), 32.0);
        assert_eq!(trigger_height(NativeSelectSize::Sm, Style::Nova), 28.0);
        assert_eq!(trigger_height(NativeSelectSize::Default, Style::Vega), 36.0);
        assert_eq!(trigger_height(NativeSelectSize::Sm, Style::Vega), 32.0);
        // Nova `rounded-lg`; sm `min(var(--radius-md), 10px)`.
        assert_eq!(
            trigger_radius(NativeSelectSize::Default, Style::Nova, &radius),
            radius.lg_u8()
        );
        assert_eq!(
            trigger_radius(NativeSelectSize::Sm, Style::Nova, &radius),
            (radius.md().round() as i32).clamp(0, 10) as u8
        );
        assert_eq!(padding_left(Style::Nova), 10.0);
        assert_eq!(padding_left(Style::Vega), 12.0);
        assert_eq!(padding_right(Style::Nova), 32.0);
        assert_eq!(padding_right(Style::Vega), 36.0);
        assert_eq!(chevron_right_inset(Style::Nova), 10.0);
        assert_eq!(chevron_right_inset(Style::Vega), 14.0);
        assert_eq!(width_padding(Style::Nova), 42.0);
        assert_eq!(width_padding(Style::Vega), 48.0);
    }

    #[test]
    fn size_heights() {
        assert_eq!(NativeSelectSize::Default.height(), 36.0);
        assert_eq!(NativeSelectSize::Sm.height(), 32.0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut selected = None;
        let s = NativeSelect::new(&mut selected)
            .option("Light")
            .option_disabled("Dark")
            .options(["System"])
            .optgroup(SelectOptGroup::new("More").option("A").option_disabled("B"))
            .placeholder("Pick a theme")
            .size(NativeSelectSize::Sm)
            .width(240.0)
            .disabled(true)
            .invalid(true)
            .id_salt("theme");
        assert_eq!(
            s.flat_labels(),
            vec![
                "Light".to_owned(),
                "Dark".to_owned(),
                "System".to_owned(),
                "A".to_owned(),
                "B".to_owned()
            ]
        );
        assert_eq!(s.flat_disabled(), vec![false, true, false, false, true]);
        assert_eq!(s.placeholder.as_deref(), Some("Pick a theme"));
        assert_eq!(s.size, NativeSelectSize::Sm);
        assert_eq!(s.width, Some(240.0));
        assert!(s.disabled);
        assert!(s.invalid);
        assert_eq!(s.id_salt.as_deref(), Some("theme"));
    }

    #[test]
    fn group_disabled_disables_all_group_options() {
        let mut selected = None;
        let s = NativeSelect::new(&mut selected).optgroup(
            SelectOptGroup::new("G")
                .disabled(true)
                .option("A")
                .option("B"),
        );
        assert_eq!(s.flat_disabled(), vec![true, true]);
        // An enabled group keeps per-option flags.
        let mut selected = None;
        let s = NativeSelect::new(&mut selected)
            .optgroup(SelectOptGroup::new("G").option("A").option_disabled("B"));
        assert_eq!(s.flat_disabled(), vec![false, true]);
    }

    #[test]
    fn popup_rows_mix_headers_and_options() {
        let mut selected = None;
        let s = NativeSelect::new(&mut selected)
            .option("Flat")
            .optgroup(SelectOptGroup::new("G").option("A").option("B"))
            .option("Tail");
        let rows = s.popup_rows();
        assert_eq!(rows.len(), 5);
        assert!(matches!(&rows[0], PopupRow::Option { flat: 0, .. }));
        assert!(matches!(&rows[1], PopupRow::Header(l) if l == "G"));
        assert!(matches!(&rows[2], PopupRow::Option { flat: 1, .. }));
        assert!(matches!(&rows[3], PopupRow::Option { flat: 2, .. }));
        assert!(matches!(&rows[4], PopupRow::Option { flat: 3, .. }));
        // An empty group renders its header and no rows.
        let mut selected = None;
        let s = NativeSelect::new(&mut selected).optgroup(SelectOptGroup::new("Empty"));
        let rows = s.popup_rows();
        assert_eq!(rows.len(), 1);
        assert!(matches!(&rows[0], PopupRow::Header(l) if l == "Empty"));
        assert!(s.flat_labels().is_empty());
    }

    #[test]
    fn label_for_uses_flat_index_across_groups() {
        let mut selected = None;
        let s = NativeSelect::new(&mut selected)
            .option("A")
            .optgroup(SelectOptGroup::new("G").option("B").option("C"));
        assert_eq!(s.label_for(0), "A");
        assert_eq!(s.label_for(1), "B");
        assert_eq!(s.label_for(2), "C");
        assert_eq!(s.label_for(3), "");
    }

    #[test]
    fn step_enabled_skips_disabled_and_stops_at_ends() {
        let d = [false, true, false];
        assert_eq!(step_enabled(&d, Some(0), 1), Some(2));
        assert_eq!(step_enabled(&d, Some(2), 1), None);
        assert_eq!(step_enabled(&d, Some(2), -1), Some(0));
        assert_eq!(step_enabled(&d, Some(0), -1), None);
        assert_eq!(step_enabled(&d, None, 1), Some(0));
        assert_eq!(step_enabled(&d, None, -1), Some(2));
        assert_eq!(step_enabled(&[true, true], Some(0), 1), None);
        assert_eq!(step_enabled(&[], None, 1), None);
        // A stale out-of-range start behaves like None.
        assert_eq!(step_enabled(&d, Some(99), 1), Some(0));
        assert_eq!(step_enabled(&d, Some(99), -1), Some(2));
    }

    #[test]
    fn page_enabled_moves_up_to_eight() {
        let d = [false; 12];
        assert_eq!(page_enabled(&d, Some(0), 1), Some(8));
        assert_eq!(page_enabled(&d, Some(8), 1), Some(11));
        assert_eq!(page_enabled(&d, Some(11), 1), Some(11));
        assert_eq!(page_enabled(&d, None, 1), Some(7));
        assert_eq!(page_enabled(&d, Some(11), -1), Some(3));
        // Disabled options do not consume page steps.
        let d = [false, true, false, false];
        assert_eq!(page_enabled(&d, Some(0), 1), Some(3));
    }

    #[test]
    fn typeahead_match_searches_after_current_and_wraps() {
        let labels = ["Apple", "Banana", "Avocado"];
        let d = [false, false, false];
        assert_eq!(typeahead_match(&labels, &d, Some(0), "a"), Some(2));
        assert_eq!(typeahead_match(&labels, &d, Some(2), "a"), Some(0));
        assert_eq!(typeahead_match(&labels, &d, None, "a"), Some(0));
        assert_eq!(typeahead_match(&labels, &d, None, "AV"), Some(2));
        assert_eq!(typeahead_match(&labels, &d, None, "z"), None);
        assert_eq!(typeahead_match(&labels, &d, None, ""), None);
        // Disabled options are skipped.
        let d = [true, false, false];
        assert_eq!(typeahead_match(&labels, &d, None, "a"), Some(2));
        let d = [true, false, true];
        assert_eq!(typeahead_match(&labels, &d, None, "a"), None);
    }

    // ---- Runtime harness (accordion pattern): frames with pointer/key
    // events drive the real `show` path end to end.

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

        fn step(&mut self, events: Vec<egui::Event>, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("native_select_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn run(&mut self, f: impl FnMut(&mut Ui)) {
            self.step(Vec::new(), f);
        }

        /// Moves the pointer, presses and releases, each on a rendered frame.
        fn click(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            self.step(vec![egui::Event::PointerMoved(pos)], &mut f);
            for pressed in [true, false] {
                self.step(
                    vec![egui::Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    &mut f,
                );
            }
        }

        fn key(&mut self, key: Key, f: impl FnMut(&mut Ui)) {
            self.step(
                vec![egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                    physical_key: None,
                }],
                f,
            );
        }

        fn text(&mut self, text: &str, f: impl FnMut(&mut Ui)) {
            self.step(vec![egui::Event::Text(text.to_owned())], f);
        }

        fn is_popup_open(&self, trigger_id: Id) -> bool {
            egui::Popup::is_id_open(&self.ctx, trigger_id.with("popup"))
        }
    }

    /// Renders one select, capturing its trigger rect + id.
    fn select_ui(ui: &mut Ui, selected: &mut Option<usize>, rect: &mut Rect, trigger_id: &mut Id) {
        let r = NativeSelect::new(selected)
            .id_salt("harness")
            .option("Apple")
            .option_disabled("Banana")
            .option("Avocado")
            .option("Cherry")
            .show(ui);
        *rect = r.rect;
        *trigger_id = r.id;
    }

    #[test]
    fn trigger_height_follows_installed_style() {
        // The default preset is nova: `h-8` (32pt).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("native_select_size_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            rect = NativeSelect::new(&mut selected)
                .options(["A", "B"])
                .show(ui)
                .rect;
        });
        out.textures_delta.clear();
        assert_eq!(rect.height(), 32.0, "nova h-8: {}", rect.height());

        // A non-nova (base recipe) theme keeps `h-9`.
        let ctx = egui::Context::default();
        Preset::default().with_style(Style::Vega).install(&ctx);
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("native_select_size_test_base"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            rect = NativeSelect::new(&mut selected)
                .options(["A", "B"])
                .show(ui)
                .rect;
        });
        out.textures_delta.clear();
        assert_eq!(rect.height(), NativeSelectSize::Default.height());
    }

    #[test]
    fn width_floor_and_explicit_width() {
        let mut h = Harness::new();
        let mut rect = Rect::NOTHING;
        let mut selected = Some(0);
        h.run(|ui| {
            rect = NativeSelect::new(&mut selected)
                .id_salt("floor")
                .option("A")
                .show(ui)
                .rect;
        });
        assert_eq!(rect.width(), NATIVE_SELECT_MIN_WIDTH);
        h.run(|ui| {
            rect = NativeSelect::new(&mut selected)
                .id_salt("explicit")
                .width(240.0)
                .option("A very long option label that exceeds the floor")
                .show(ui)
                .rect;
        });
        assert_eq!(rect.width(), 240.0);
    }

    #[test]
    fn click_opens_popup_and_focuses_trigger() {
        let mut h = Harness::new();
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut trigger_id = Id::NULL;
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        assert!(!h.is_popup_open(trigger_id));
        h.click(rect.center(), |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert!(h.is_popup_open(trigger_id), "click opens the popup");
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        assert!(
            h.ctx.memory(|m| m.focused() == Some(trigger_id)),
            "click focuses the trigger"
        );
        // Clicking the trigger again closes the popup.
        h.click(rect.center(), |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert!(!h.is_popup_open(trigger_id), "second click closes");
    }

    #[test]
    fn closed_arrows_change_selection_skipping_disabled() {
        let mut h = Harness::new();
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut trigger_id = Id::NULL;
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        // Focus via click, then Escape to close (focus stays on trigger).
        // Two settling frames first: egui's focus-lock filter takes effect
        // only after two consecutive focused frames, and real usage always
        // has frames elapse between the click and the keypress.
        h.click(rect.center(), |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert!(h.is_popup_open(trigger_id));
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        h.key(Key::Escape, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert!(!h.is_popup_open(trigger_id));
        assert_eq!(selected, Some(0), "Escape keeps the pre-open value");
        // ArrowDown skips disabled Banana (index 1) → Avocado (2).
        h.key(Key::ArrowDown, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(2));
        assert!(!h.is_popup_open(trigger_id), "closed arrows do not open");
        h.key(Key::ArrowDown, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(3));
        // No wrap at the end (OPEN-1 working assumption).
        h.key(Key::ArrowDown, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(3));
        h.key(Key::ArrowUp, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(2));
    }

    #[test]
    fn open_arrows_move_highlight_and_enter_confirms() {
        let mut h = Harness::new();
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut trigger_id = Id::NULL;
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        h.click(rect.center(), |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert!(h.is_popup_open(trigger_id));
        // Two settling frames: the focus-lock filter (which keeps arrows
        // from moving focus) takes effect only after two consecutive
        // focused frames.
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        // Open arrows move the highlight only — the selection is untouched.
        h.key(Key::ArrowDown, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(0), "open arrows do not commit");
        assert!(h.is_popup_open(trigger_id));
        // Enter confirms the highlight (Avocado, index 2 — Banana skipped).
        h.key(Key::Enter, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(2));
        assert!(!h.is_popup_open(trigger_id), "Enter closes the popup");
        // Focus stays on the trigger after close.
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        assert_eq!(h.ctx.memory(|m| m.focused()), Some(trigger_id));
    }

    #[test]
    fn space_closed_opens_and_space_open_does_nothing() {
        let mut h = Harness::new();
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut trigger_id = Id::NULL;
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        h.ctx.memory_mut(|m| m.request_focus(trigger_id));
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        // Space (closed) opens.
        h.key(Key::Space, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert!(h.is_popup_open(trigger_id), "Space opens when closed");
        // Space (open): OPEN-1 — no behaviour; the popup stays open and
        // the selection is unchanged.
        h.key(Key::Space, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert!(h.is_popup_open(trigger_id), "Space does nothing when open");
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn home_end_and_typeahead_when_open() {
        let mut h = Harness::new();
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut trigger_id = Id::NULL;
        h.run(|ui| select_ui(ui, &mut selected, &mut rect, &mut trigger_id));
        h.click(rect.center(), |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        // End → last option (Cherry); Enter confirms it.
        h.key(Key::End, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        h.key(Key::Enter, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(3));
        // Reopen; type-ahead "a" jumps the highlight to Apple (index 0,
        // searching after the current highlight, wrapping once).
        h.click(rect.center(), |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        h.text("a", |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        h.key(Key::Enter, |ui| {
            select_ui(ui, &mut selected, &mut rect, &mut trigger_id)
        });
        assert_eq!(selected, Some(0), "type-ahead + Enter selects Apple");
    }

    #[test]
    fn disabled_trigger_never_opens() {
        let mut h = Harness::new();
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut trigger_id = Id::NULL;
        h.run(|ui| {
            let r = NativeSelect::new(&mut selected)
                .id_salt("disabled-harness")
                .disabled(true)
                .option("A")
                .option("B")
                .show(ui);
            rect = r.rect;
            trigger_id = r.id;
        });
        h.click(rect.center(), |ui| {
            let r = NativeSelect::new(&mut selected)
                .id_salt("disabled-harness")
                .disabled(true)
                .option("A")
                .option("B")
                .show(ui);
            rect = r.rect;
            trigger_id = r.id;
        });
        assert!(!h.is_popup_open(trigger_id));
    }

    #[test]
    fn empty_list_renders_without_panicking() {
        let mut h = Harness::new();
        let mut selected: Option<usize> = None;
        let mut rect = Rect::NOTHING;
        let mut trigger_id = Id::NULL;
        h.run(|ui| {
            let r = NativeSelect::new(&mut selected)
                .id_salt("empty-harness")
                .placeholder("Nothing here")
                .show(ui);
            rect = r.rect;
            trigger_id = r.id;
        });
        h.click(rect.center(), |ui| {
            let r = NativeSelect::new(&mut selected)
                .id_salt("empty-harness")
                .placeholder("Nothing here")
                .show(ui);
            rect = r.rect;
            trigger_id = r.id;
        });
        assert!(h.is_popup_open(trigger_id), "empty popup still opens");
        h.key(Key::Enter, |ui| {
            NativeSelect::new(&mut selected)
                .id_salt("empty-harness")
                .placeholder("Nothing here")
                .show(ui);
        });
        assert_eq!(selected, None);
        assert!(!h.is_popup_open(trigger_id));
    }
}
