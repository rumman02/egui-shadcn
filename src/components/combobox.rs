//! shadcn `combobox` — the docs composition of Popover + Command.
//!
//! There is no dedicated `ui/combobox.tsx` in the source; the component is
//! documented as a popover trigger (`<Button variant="outline"
//! className="w-[200px] justify-between font-normal">` showing the selected
//! value or a placeholder with a chevron) opening a `w-[200px] p-0`
//! popover that embeds a `Command` list (`CommandInput` + `CommandEmpty` +
//! check-marked `CommandItem`s). The multi-select docs variant
//! (`ComboboxChips`) shows removable chips in the trigger.
//!
//! This port owns the popup (anchored below the trigger, dismiss on Escape
//! and outside click) and reuses the command list internally for the
//! search field, case-insensitive filtering, empty state, check marks and
//! keyboard navigation (ArrowUp/Down cycle, Enter toggles/selects, typing
//! filters). Single-select closes on pick; multi-select stays open and
//! toggles the picked value.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut framework = Some("Next.js".to_string());
//! let res = Combobox::new(&mut framework)
//!     .placeholder("Select a framework...")
//!     .width(200.0)
//!     .id_salt("framework")
//!     .options(["Next.js", "SvelteKit", "Nuxt.js", "Remix", "Astro"])
//!     .show_seeded(ui);
//! if res.changed() { /* res.value was updated */ }
//!
//! // Multi-select with removable chips in the trigger:
//! let mut picked = vec!["Next.js".to_string()];
//! Combobox::multi(&mut picked)
//!     .placeholder("Frameworks...")
//!     .id_salt("frameworks")
//!     .options(["Next.js", "SvelteKit", "Astro"])
//!     .show_seeded(ui);
//!
//! // Plain-String binding (empty = placeholder) and uncontrolled:
//! Combobox::bind(&mut s).options(["A", "B"]).show_seeded(ui);
//! Combobox::uncontrolled().id_salt("solo").option("A").show_seeded(ui);
//!
//! // With a closure (options + separators):
//! Combobox::new(&mut v).show(ui, |c| {
//!     c.option("Next.js");
//!     c.option_disabled("Astro");
//!     c.separator();
//!     c.option("Other");
//! });
//! ```

use crate::components::command::{Command, CommandInput, CommandItem};
use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use crate::utils::shadow;
use egui::emath::RectAlign;
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Key, LayerId, Pos2, Rect, Response, Sense,
    Stroke, Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Geometry constants
// ---------------------------------------------------------------------------

/// Trigger and popup width: the docs `w-[200px]`.
pub const COMBOBOX_DEFAULT_WIDTH: f32 = 200.0;
/// Trigger height (`h-9`).
pub const COMBOBOX_TRIGGER_HEIGHT: f32 = crate::theme::tokens::spacing::CONTROL;
/// Popup list max height.
pub const COMBOBOX_LIST_MAX_HEIGHT: f32 = 300.0;
/// Chevron icon size (`size-4`).
const ICON_SIZE: f32 = 16.0;
/// Chip height inside the multi trigger (`py-0.5` + `text-xs`).
const CHIP_HEIGHT: f32 = 20.0;
/// Chip horizontal padding + gap.
const CHIP_PAD_X: f32 = 6.0;

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// One combobox option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxOption {
    /// The visible label (also the value by default).
    pub label: String,
    /// The value written into the bound state.
    pub value: String,
    /// `disabled` — cannot be picked, skipped by keyboard navigation.
    pub disabled: bool,
}

impl ComboboxOption {
    /// Creates an enabled option whose value is its label.
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            value: label.clone(),
            label,
            disabled: false,
        }
    }

    /// Overrides the value written into the bound state.
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

/// One entry of the option list (item or separator).
#[derive(Clone, Debug)]
pub(crate) enum ComboEntry {
    Item(ComboboxOption),
    Separator,
}

/// Collects options inside the [`Combobox::show`] closure.
#[derive(Debug, Default)]
pub struct ComboboxBuilder {
    pub(crate) entries: Vec<ComboEntry>,
}

impl ComboboxBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an enabled option.
    pub fn option(&mut self, label: impl Into<String>) {
        self.entries
            .push(ComboEntry::Item(ComboboxOption::new(label)));
    }

    /// Appends a disabled option.
    pub fn option_disabled(&mut self, label: impl Into<String>) {
        self.entries
            .push(ComboEntry::Item(ComboboxOption::new(label).disabled(true)));
    }

    /// Appends an option with an explicit value.
    pub fn item(&mut self, item: ComboboxOption) {
        self.entries.push(ComboEntry::Item(item));
    }

    /// Appends a separator line (rendered while the group has any visible
    /// items).
    pub fn separator(&mut self) {
        self.entries.push(ComboEntry::Separator);
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested)
// ---------------------------------------------------------------------------

/// What the combobox writes into.
enum Binding<'a> {
    /// Single select, `None` = placeholder.
    Opt(&'a mut Option<String>),
    /// Single select on a plain string (empty = placeholder).
    Str(&'a mut String),
    /// Multi select.
    Multi(&'a mut Vec<String>),
}

impl Binding<'_> {
    fn is_multi(&self) -> bool {
        matches!(self, Binding::Multi(_))
    }

    /// The single value (multi returns `None`).
    fn single(&self) -> Option<String> {
        match self {
            Binding::Opt(v) => (*v).clone(),
            Binding::Str(s) => {
                if s.is_empty() {
                    None
                } else {
                    Some(s.as_str().to_owned())
                }
            }
            Binding::Multi(_) => None,
        }
    }

    /// Whether `value` is picked (multi checks membership).
    fn contains(&self, value: &str) -> bool {
        match self {
            Binding::Multi(v) => v.iter().any(|s| s == value),
            _ => self.single().as_deref() == Some(value),
        }
    }

    fn set_single(&mut self, value: Option<String>) {
        match self {
            Binding::Opt(v) => **v = value,
            Binding::Str(s) => **s = value.unwrap_or_default(),
            Binding::Multi(_) => {}
        }
    }

    /// Toggles `value` in a multi binding; returns the new membership.
    fn toggle_multi(&mut self, value: &str) -> bool {
        if let Binding::Multi(v) = self {
            if let Some(pos) = v.iter().position(|s| s == value) {
                v.remove(pos);
                false
            } else {
                v.push(value.to_owned());
                true
            }
        } else {
            self.contains(value)
        }
    }
}

/// Resolved trigger colors (mirrors the outline button the docs use).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerColors {
    pub fill: Color32,
    pub text: Color32,
    pub chevron: Color32,
    pub stroke: Stroke,
    pub ring: Color32,
}

pub(crate) fn resolve_trigger_colors(
    p: &Palette,
    dark: bool,
    hovered: bool,
    disabled: bool,
    invalid: bool,
    has_value: bool,
) -> TriggerColors {
    let (fill, text) = if hovered && !disabled {
        if dark {
            (color_alpha(p.input, 0.5), p.foreground)
        } else {
            (p.accent, p.accent_foreground)
        }
    } else if dark {
        (color_alpha(p.input, 0.3), p.foreground)
    } else {
        (p.background, p.foreground)
    };
    let text = if disabled {
        color_alpha(text, 0.5)
    } else if !has_value {
        p.muted_foreground
    } else {
        text
    };
    let stroke_color = if invalid {
        p.destructive
    } else if dark {
        p.input
    } else {
        p.border
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    TriggerColors {
        fill,
        text,
        chevron: color_alpha(p.muted_foreground, 0.5),
        stroke: Stroke::new(1.0, stroke_color),
        ring,
    }
}

/// The chip look for multi-select triggers (`bg-secondary
/// text-secondary-foreground`, docs `ComboboxChip`).
pub(crate) fn chip_colors(p: &Palette) -> (Color32, Color32) {
    (p.secondary, p.secondary_foreground)
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Per-instance popup state.
#[derive(Clone, Debug, Default)]
struct ComboState {
    open: bool,
    trigger: Option<TriggerInfo>,
}

#[derive(Clone, Copy, Debug)]
struct TriggerInfo {
    rect: Rect,
    layer_id: LayerId,
}

fn read_state(ctx: &Context, id: Id) -> ComboState {
    ctx.data_mut(|d| d.get_temp::<ComboState>(id).unwrap_or_default())
}

fn write_state(ctx: &Context, id: Id, state: ComboState) {
    ctx.data_mut(|d| d.insert_temp(id, state));
}

/// What [`Combobox::show`] returns.
#[derive(Clone, Debug)]
pub struct ComboboxResponse {
    /// The trigger's response.
    pub trigger: Response,
    /// Single-select value after this frame (multi: `None`).
    pub value: Option<String>,
    /// Multi-select: the value toggled this frame, if any.
    pub toggled: Option<String>,
    /// Whether the bound state changed this frame.
    pub changed: bool,
    /// Whether the popup is open after this frame.
    pub open: bool,
    /// Whether the popup was open at frame start and is closed now.
    pub closed: bool,
}

// ---------------------------------------------------------------------------
// Combobox (root)
// ---------------------------------------------------------------------------

/// A shadcn combobox (popover trigger + command filter list). See the
/// [module docs](self) for an example.
pub struct Combobox<'a> {
    binding: Option<Binding<'a>>,
    options: Vec<ComboEntry>,
    placeholder: Option<String>,
    empty_text: String,
    search_placeholder: String,
    width: f32,
    list_max_height: f32,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> Combobox<'a> {
    /// Creates a single-select combobox bound to the caller's value.
    pub fn new(value: &'a mut Option<String>) -> Self {
        Self {
            binding: Some(Binding::Opt(value)),
            ..Self::template()
        }
    }

    /// Creates a single-select combobox bound to a plain `String` (empty =
    /// placeholder).
    pub fn bind(value: &'a mut String) -> Self {
        Self {
            binding: Some(Binding::Str(value)),
            ..Self::template()
        }
    }

    /// Creates a multi-select combobox bound to the caller's values; the
    /// trigger shows removable chips.
    pub fn multi(values: &'a mut Vec<String>) -> Self {
        Self {
            binding: Some(Binding::Multi(values)),
            ..Self::template()
        }
    }

    /// Creates an uncontrolled combobox: the value lives in the egui state
    /// keyed by [`Combobox::id_salt`]; read it from
    /// [`ComboboxResponse::value`].
    pub fn uncontrolled() -> Self {
        Self {
            binding: None,
            ..Self::template()
        }
    }

    fn template() -> Self {
        Self {
            binding: None,
            options: Vec::new(),
            placeholder: None,
            empty_text: "No items found.".to_owned(),
            search_placeholder: "Search...".to_owned(),
            width: COMBOBOX_DEFAULT_WIDTH,
            list_max_height: COMBOBOX_LIST_MAX_HEIGHT,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Adds an enabled option (shorthand for the closure-less `show`).
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options
            .push(ComboEntry::Item(ComboboxOption::new(label)));
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
        self.options
            .push(ComboEntry::Item(ComboboxOption::new(label).disabled(true)));
        self
    }

    /// Shows faint text on the trigger while nothing is picked.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Overrides the popup's empty-state text (default `"No items found."`).
    pub fn empty_text(mut self, text: impl Into<String>) -> Self {
        self.empty_text = text.into();
        self
    }

    /// Overrides the search field placeholder (default `"Search..."`).
    pub fn search_placeholder(mut self, text: impl Into<String>) -> Self {
        self.search_placeholder = text.into();
        self
    }

    /// Sets the trigger and popup width (default [`COMBOBOX_DEFAULT_WIDTH`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Caps the popup list height (default [`COMBOBOX_LIST_MAX_HEIGHT`]).
    pub fn list_max_height(mut self, height: f32) -> Self {
        self.list_max_height = height;
        self
    }

    /// Disables the combobox.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the trigger `aria-invalid` (destructive border + ring).
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

    /// Renders the trigger and, when open, the anchored command popup.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut ComboboxBuilder)) -> ComboboxResponse {
        let mut builder = ComboboxBuilder::new();
        content(&mut builder);
        if builder.entries.is_empty() {
            builder.entries = self.options.clone();
        }
        self.show_with(ui, builder.entries)
    }

    /// Renders with only the seeded `.option(…)`s (no closure).
    pub fn show_seeded(self, ui: &mut Ui) -> ComboboxResponse {
        let seed = self.options.clone();
        self.show_with(ui, seed)
    }

    fn show_with(mut self, ui: &mut Ui, entries: Vec<ComboEntry>) -> ComboboxResponse {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let width = self.width.max(COMBOBOX_DEFAULT_WIDTH);

        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_combobox"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);
        let open_at_start = state.open;

        // Resolve the current value(s).
        let mut stored: Option<String> = ctx
            .data_mut(|d| d.get_temp::<Option<String>>(state_id.with("value")))
            .unwrap_or(None);
        let mut stored_multi: Vec<String> = ctx
            .data_mut(|d| d.get_temp::<Vec<String>>(state_id.with("value_multi")))
            .unwrap_or_default();
        let is_multi = self.binding.as_ref().map(|b| b.is_multi()).unwrap_or(false);
        let value_before = match &self.binding {
            Some(b) => match b {
                Binding::Multi(_) => String::new(),
                _ => b.single().unwrap_or_default(),
            },
            None => {
                if is_multi {
                    String::new()
                } else {
                    stored.clone().unwrap_or_default()
                }
            }
        };
        let multi_before: Vec<String> = match &self.binding {
            Some(Binding::Multi(v)) => (**v).clone(),
            _ => stored_multi.clone(),
        };
        let has_value = if is_multi {
            !multi_before.is_empty()
        } else {
            !value_before.is_empty()
        };

        // Trigger: outline button, `justify-between font-normal`, h-9.
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (_, rect) = ui.allocate_space(Vec2::new(width, COMBOBOX_TRIGGER_HEIGHT));
        let trigger_id = state_id.with("trigger");
        let response = ui.interact(rect, trigger_id, sense);

        // Multi: removable chips drawn over the trigger.
        let mut chip_remove: Option<String> = None;
        if ui.is_rect_visible(rect) {
            let c = resolve_trigger_colors(
                &p,
                dark,
                response.hovered(),
                self.disabled,
                self.invalid,
                has_value,
            );
            let radius = theme.radius.md_u8();
            let corner = CornerRadius::same(radius);
            let painter = ui.painter();

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
            painter.rect_filled(rect, corner, c.fill);
            painter.rect_stroke(rect, corner, c.stroke, egui::epaint::StrokeKind::Inside);

            // Chevron at the right edge (`justify-between`).
            let chev_rect = Rect::from_center_size(
                Pos2::new(
                    rect.max.x - crate::theme::tokens::spacing::MD - ICON_SIZE * 0.5,
                    rect.center().y,
                ),
                Vec2::splat(ICON_SIZE),
            );
            Icon::ChevronDown.paint(painter, chev_rect, c.chevron, 2.0 * ICON_SIZE / 24.0);

            let right_edge = chev_rect.min.x - crate::theme::tokens::spacing::SM;
            if is_multi && !multi_before.is_empty() {
                // Chips: as many as fit, then a `+N` chip.
                let font = FontId::proportional(crate::theme::tokens::type_scale::XS);
                let (chip_bg, chip_fg) = chip_colors(&p);
                let mut x = rect.min.x + CHIP_PAD_X;
                let mut shown = 0usize;
                for value in &multi_before {
                    let galley = ui
                        .painter()
                        .layout_no_wrap(value.clone(), font.clone(), chip_fg);
                    let chip_w = galley.size().x + 2.0 * CHIP_PAD_X + ICON_SIZE + 4.0;
                    if x + chip_w > right_edge {
                        break;
                    }
                    let chip_rect = Rect::from_min_size(
                        Pos2::new(x, rect.center().y - CHIP_HEIGHT * 0.5),
                        Vec2::new(chip_w, CHIP_HEIGHT),
                    );
                    painter.rect_filled(
                        chip_rect,
                        CornerRadius::same(theme.radius.sm_u8()),
                        chip_bg,
                    );
                    painter.galley(
                        Pos2::new(
                            chip_rect.min.x + CHIP_PAD_X,
                            chip_rect.center().y - galley.size().y * 0.5,
                        ),
                        galley,
                        chip_fg,
                    );
                    // Remove X (interactive; on top of the trigger).
                    let x_rect = Rect::from_center_size(
                        Pos2::new(
                            chip_rect.max.x - CHIP_PAD_X - ICON_SIZE * 0.5,
                            chip_rect.center().y,
                        ),
                        Vec2::splat(ICON_SIZE),
                    );
                    let x_id = trigger_id.with(("chip_x", value.clone()));
                    let x_resp = ui.interact(x_rect, x_id, Sense::click());
                    let x_color = if x_resp.hovered() {
                        chip_fg
                    } else {
                        color_alpha(chip_fg, 0.6)
                    };
                    Icon::X.paint(painter, x_rect, x_color, 2.0 * ICON_SIZE / 24.0);
                    if x_resp.clicked() {
                        chip_remove = Some(value.clone());
                    }
                    x = chip_rect.max.x + CHIP_PAD_X;
                    shown += 1;
                }
                let hidden = multi_before.len() - shown;
                if hidden > 0 {
                    let label = format!("+{hidden}");
                    let galley = ui.painter().layout_no_wrap(label, font.clone(), chip_fg);
                    let chip_w = galley.size().x + 2.0 * CHIP_PAD_X;
                    let chip_rect = Rect::from_min_size(
                        Pos2::new(
                            x.min(right_edge - chip_w),
                            rect.center().y - CHIP_HEIGHT * 0.5,
                        ),
                        Vec2::new(chip_w, CHIP_HEIGHT),
                    );
                    painter.rect_filled(
                        chip_rect,
                        CornerRadius::same(theme.radius.sm_u8()),
                        chip_bg,
                    );
                    painter.galley(
                        Pos2::new(
                            chip_rect.min.x + CHIP_PAD_X,
                            chip_rect.center().y - galley.size().y * 0.5,
                        ),
                        galley,
                        chip_fg,
                    );
                }
            } else {
                let label = if is_multi {
                    self.placeholder.clone().unwrap_or_default()
                } else if has_value {
                    value_before.clone()
                } else {
                    self.placeholder.clone().unwrap_or_default()
                };
                let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
                painter.text(
                    Pos2::new(
                        rect.min.x + crate::theme::tokens::spacing::MD,
                        rect.center().y,
                    ),
                    Align2::LEFT_CENTER,
                    label,
                    font,
                    c.text,
                );
            }

            // Focus ring.
            if response.has_focus() && !self.disabled {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, c.ring, 3.0, radius as f32);
            }
        }

        // Trigger toggles (unless the click hit a chip's remove X).
        if !self.disabled && response.clicked() && chip_remove.is_none() {
            state.open = !state.open;
        }

        // Apply chip removal (multi).
        let mut changed = false;
        let mut toggled: Option<String> = None;
        if let Some(value) = chip_remove {
            let removed = match &mut self.binding {
                Some(b) => !b.toggle_multi(&value),
                None => {
                    if let Some(pos) = stored_multi.iter().position(|s| *s == value) {
                        stored_multi.remove(pos);
                        true
                    } else {
                        false
                    }
                }
            };
            let _ = removed;
            changed = true;
            toggled = Some(value);
        }

        if !state.open {
            state.trigger = Some(TriggerInfo {
                rect,
                layer_id: response.layer_id,
            });
        }

        // Popup.
        let popup_id = state_id.with("popup");
        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        let just_opened = state.open && !was_open_last_frame;
        let mut popup_rects: Vec<Rect> = Vec::new();
        let mut activated: Option<String> = None;

        if state.open || was_open_last_frame {
            if let Some(trigger) = state.trigger {
                let progress = crate::utils::animation::overlay_progress(
                    &ctx,
                    popup_id.with("anim"),
                    state.open,
                );
                if state.open || progress.alpha > 0.002 {
                    let popup_width = width.max(SELECT_MIN_WIDTH_FLOOR);
                    let frame = egui::Frame::new()
                        .fill(p.popover)
                        .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
                        .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
                        .inner_margin(egui::Margin::same(0))
                        .shadow(shadow::shadow_md(dark));

                    let popup = egui::Popup::new(
                        popup_id,
                        ctx.clone(),
                        egui::PopupAnchor::ParentRect(trigger.rect),
                        trigger.layer_id,
                    )
                    .open(true)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .align(RectAlign::BOTTOM_START)
                    .gap(4.0 + (1.0 - progress.alpha) * 8.0)
                    .width(popup_width)
                    .frame(egui::Frame::new());

                    let inner = popup.show(|ui| {
                        ui.set_opacity(progress.alpha);
                        frame.show(ui, |ui| {
                            ui.set_min_width(popup_width);
                            ui.set_max_width(popup_width);
                            let command_salt = format!("{}_cmd", state_id.value());
                            let mut multi_now = multi_before.clone();
                            let res = Command::new()
                                .chrome(false)
                                .width(popup_width)
                                .autofocus(true)
                                .reset_autofocus(just_opened)
                                .force_keyboard(true)
                                .id_salt(command_salt)
                                .show(ui, |c| {
                                    c.input(
                                        CommandInput::new()
                                            .placeholder(self.search_placeholder.clone()),
                                    );
                                    c.empty_text(self.empty_text.clone());
                                    c.group_bare(|g| {
                                        for entry in &entries {
                                            match entry {
                                                ComboEntry::Item(opt) => {
                                                    let checked = selected_now(
                                                        opt,
                                                        is_multi,
                                                        &value_before,
                                                        &multi_now,
                                                    );
                                                    g.item(
                                                        CommandItem::new(opt.label.clone())
                                                            .id(opt.value.clone())
                                                            .checked(checked)
                                                            .disabled(opt.disabled),
                                                    );
                                                }
                                                ComboEntry::Separator => {
                                                    g.separator();
                                                }
                                            }
                                        }
                                    });
                                });
                            if let Some(a) = res.activated {
                                activated = Some(a.id);
                            }
                            let _ = &mut multi_now;
                        });
                    });
                    if let Some(ref r) = inner {
                        popup_rects.push(r.response.rect);
                    }
                }
            }
        }

        // Escape closes (dialogs keep priority).
        if state.open {
            let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
            let esc = !modal_open && ctx.input(|i| i.key_pressed(Key::Escape));
            if esc {
                state.open = false;
            }
            // Click outside closes.
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

        // Apply the activation: single selects + closes, multi toggles.
        if state.open || activated.is_some() {
            if let Some(value) = activated.take() {
                let enabled = entries.iter().any(|e| match e {
                    ComboEntry::Item(it) => it.value == value && !it.disabled,
                    _ => false,
                });
                if enabled {
                    changed = true;
                    toggled = Some(value.clone());
                    if is_multi {
                        match &mut self.binding {
                            Some(b) => {
                                b.toggle_multi(&value);
                            }
                            None => {
                                if let Some(pos) = stored_multi.iter().position(|s| *s == value) {
                                    stored_multi.remove(pos);
                                } else {
                                    stored_multi.push(value.clone());
                                }
                            }
                        }
                    } else {
                        match &mut self.binding {
                            Some(b) => b.set_single(Some(value.clone())),
                            None => stored = Some(value.clone()),
                        }
                        if state.open {
                            state.open = false;
                        }
                    }
                }
            }
        }

        // Write the uncontrolled store.
        if self.binding.is_none() {
            ctx.data_mut(|d| {
                d.insert_temp(state_id.with("value"), stored.clone());
                d.insert_temp(state_id.with("value_multi"), stored_multi.clone());
            });
        }

        let closed = open_at_start && !state.open;
        let open_now = state.open;
        write_state(&ctx, state_id, state);

        let value_after = if is_multi {
            None
        } else {
            match &self.binding {
                Some(b) => b.single(),
                None => stored.clone(),
            }
        };

        ComboboxResponse {
            trigger: response,
            value: value_after,
            toggled,
            changed,
            open: open_now,
            closed,
        }
    }
}

/// Whether the option is currently picked (for the check mark).
fn selected_now(
    opt: &ComboboxOption,
    is_multi: bool,
    single_value: &str,
    multi_values: &[String],
) -> bool {
    if is_multi {
        multi_values.iter().any(|v| v == &opt.value)
    } else {
        single_value == opt.value
    }
}

/// The popup never shrinks below the trigger width floor.
const SELECT_MIN_WIDTH_FLOOR: f32 = COMBOBOX_DEFAULT_WIDTH;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn trigger_colors_follow_state() {
        let p = palette(Mode::Light);
        let c = resolve_trigger_colors(&p, false, false, false, false, false);
        assert_eq!(c.fill, p.background);
        assert_eq!(c.text, p.muted_foreground, "placeholder look");
        assert_eq!(c.stroke.color, p.border);
        let valued = resolve_trigger_colors(&p, false, false, false, false, true);
        assert_eq!(valued.text, p.foreground);
        let hovered = resolve_trigger_colors(&p, false, true, false, false, true);
        assert_eq!(hovered.fill, p.accent);
        let invalid = resolve_trigger_colors(&p, false, false, false, true, true);
        assert_eq!(invalid.stroke.color, p.destructive);
        assert_eq!(invalid.ring, color_alpha(p.destructive, 0.2));
        let disabled = resolve_trigger_colors(&p, false, false, true, false, true);
        assert_eq!(disabled.text, color_alpha(p.foreground, 0.5));
    }

    #[test]
    fn dark_mode_uses_input_fills() {
        let p = palette(Mode::Dark);
        let resting = resolve_trigger_colors(&p, true, false, false, false, true);
        assert_eq!(resting.fill, color_alpha(p.input, 0.3));
        assert_eq!(resting.stroke.color, p.input);
        let hovered = resolve_trigger_colors(&p, true, true, false, false, true);
        assert_eq!(hovered.fill, color_alpha(p.input, 0.5));
    }

    #[test]
    fn options_keep_values_and_disabled() {
        let o = ComboboxOption::new("Next.js");
        assert_eq!(o.value, "Next.js");
        let o = ComboboxOption::new("Other").value("other").disabled(true);
        assert_eq!(o.value, "other");
        assert!(o.disabled);
    }

    #[test]
    fn builders_keep_entries() {
        let c = Combobox::uncontrolled()
            .option("A")
            .options(["B", "C"])
            .option_disabled("D")
            .placeholder("Pick")
            .empty_text("Nothing")
            .search_placeholder("Type")
            .width(240.0)
            .list_max_height(200.0)
            .disabled(true)
            .invalid(true)
            .id_salt("x");
        assert_eq!(c.options.len(), 4);
        assert!(matches!(&c.options[3], ComboEntry::Item(o) if o.disabled));
        assert_eq!(c.placeholder.as_deref(), Some("Pick"));
        assert_eq!(c.empty_text, "Nothing");
        assert_eq!(c.search_placeholder, "Type");
        assert_eq!(c.width, 240.0);
        assert_eq!(c.list_max_height, 200.0);
        assert!(c.disabled && c.invalid);
        assert_eq!(c.id_salt.as_deref(), Some("x"));

        let mut b = ComboboxBuilder::new();
        b.option("A");
        b.option_disabled("B");
        b.item(ComboboxOption::new("C").value("c"));
        b.separator();
        assert_eq!(b.entries.len(), 4);
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
                    Id::unique("combobox_test_ui"),
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
                        Id::unique("combobox_test_ui"),
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
                    Id::unique("combobox_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> ComboState {
            let id = Id::unique("combobox_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<ComboState>(id))
                .unwrap_or_default()
        }
    }

    const OPTS: [&str; 2] = ["Alpha", "Beta"];

    fn layout(h: &mut Harness, salt: &str) -> Rect {
        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            trigger_rect = Combobox::uncontrolled()
                .id_salt(salt)
                .options(OPTS)
                .show_seeded(ui)
                .trigger
                .rect;
        });
        h.run(|ui| {
            trigger_rect = Combobox::uncontrolled()
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
            Combobox::uncontrolled()
                .id_salt("t1")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("t1").open, "clicking the trigger opens it");

        h.click(rect.center(), |ui| {
            Combobox::uncontrolled()
                .id_salt("t1")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("t1").open, "clicking again closes it");
    }

    #[test]
    fn escape_closes_open_combobox() {
        let mut h = Harness::new();
        let rect = layout(&mut h, "esc");
        h.click(rect.center(), |ui| {
            Combobox::uncontrolled()
                .id_salt("esc")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("esc").open);
        h.run(|ui| {
            Combobox::uncontrolled()
                .id_salt("esc")
                .options(OPTS)
                .show_seeded(ui);
        });
        h.key(Key::Escape, |ui| {
            Combobox::uncontrolled()
                .id_salt("esc")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("esc").open, "Escape must close the combobox");
    }

    #[test]
    fn selecting_an_item_updates_the_value_and_closes() {
        let mut h = Harness::new();
        let mut value: Option<String> = None;
        let rect = layout(&mut h, "sel");

        h.click(rect.center(), |ui| {
            value = None;
            Combobox::new(&mut value)
                .id_salt("sel")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("sel").open);

        // Let the popup settle one frame (egui hit-tests against the
        // previous pass's rects).
        h.run(|ui| {
            Combobox::new(&mut value)
                .id_salt("sel")
                .options(OPTS)
                .show_seeded(ui);
        });

        let popup_pos = Pos2::new(rect.center().x, rect.max.y + 60.0);
        h.click(popup_pos, |ui| {
            Combobox::new(&mut value)
                .id_salt("sel")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(!h.state("sel").open, "single select closes the popup");
        assert_eq!(value.as_deref(), Some("Alpha"), "first item picked");
    }

    #[test]
    fn multi_toggle_keeps_the_popup_open() {
        let mut h = Harness::new();
        let mut picked: Vec<String> = Vec::new();
        let rect = layout(&mut h, "multi");

        h.click(rect.center(), |ui| {
            Combobox::multi(&mut picked)
                .id_salt("multi")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("multi").open);

        h.run(|ui| {
            Combobox::multi(&mut picked)
                .id_salt("multi")
                .options(OPTS)
                .show_seeded(ui);
        });

        let popup_pos = Pos2::new(rect.center().x, rect.max.y + 60.0);
        h.click(popup_pos, |ui| {
            Combobox::multi(&mut picked)
                .id_salt("multi")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(h.state("multi").open, "multi stays open after picking");
        assert_eq!(picked, vec!["Alpha".to_string()]);

        // Clicking the same row again toggles it off.
        h.click(popup_pos, |ui| {
            Combobox::multi(&mut picked)
                .id_salt("multi")
                .options(OPTS)
                .show_seeded(ui);
        });
        assert!(picked.is_empty(), "second click toggles the value off");
    }

    #[test]
    fn disabled_combobox_ignores_input() {
        let mut h = Harness::new();
        let mut value: Option<String> = None;
        let mut rect = Rect::ZERO;
        h.run(|ui| {
            rect = Combobox::new(&mut value)
                .id_salt("dis")
                .options(OPTS)
                .disabled(true)
                .show_seeded(ui)
                .trigger
                .rect;
        });
        h.click(rect.center(), |ui| {
            Combobox::new(&mut value)
                .id_salt("dis")
                .options(OPTS)
                .disabled(true)
                .show_seeded(ui);
        });
        assert!(!h.state("dis").open, "a disabled combobox cannot open");
        assert_eq!(value, None);
    }
}
