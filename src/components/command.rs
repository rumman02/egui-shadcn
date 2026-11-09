//! shadcn `command.tsx` — the cmdk-style command palette.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/command.tsx` (cmdk): the
//! root is `flex flex-col overflow-hidden rounded-md bg-popover
//! text-popover-foreground` (the docs' standalone demo adds `rounded-lg
//! border shadow-md md:w-[300px]`, which [`Command`] paints when
//! `chrome(true)`), the input wrapper is `flex h-9 items-center gap-2
//! border-b px-3` with a `size-4 opacity-50` search icon, the list is
//! `max-h-[300px]`, groups are `p-1` with `px-2 py-1.5 text-xs font-medium
//! text-muted-foreground` headings, items are `rounded-sm px-2 py-1.5
//! text-sm data-[selected=true]:bg-accent data-[selected=true]:
//! text-accent-foreground data-[disabled=true]:opacity-50`, the shortcut is
//! `ml-auto text-xs tracking-widest text-muted-foreground` and the empty
//! state is `py-6 text-center text-sm`.
//!
//! Items are declared through builder closures like the menu API. Filtering
//! is a case-insensitive substring match on the item label; items with
//! `should_filter(false)` stay visible while searching. Keyboard: the input
//! focuses on open (dialog), ArrowUp/Down cycle through visible enabled
//! items (wrapping), Home/End jump, Enter selects, and Escape closes when
//! rendered inside [`CommandDialog`].
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let res = Command::new().id_salt("palette").show(ui, |c| {
//!     c.input(CommandInput::new().placeholder("Type a command or search..."));
//!     c.group("Suggestions", |g| {
//!         g.item(CommandItem::new("Calendar").icon(Icon::Calendar).shortcut("⌘E"));
//!         g.item(CommandItem::new("Search Emoji"));
//!         g.item(CommandItem::new("Calculator").disabled(true));
//!     });
//!     c.separator();
//!     c.group("Settings", |g| {
//!         g.item(CommandItem::new("Theme").shortcut("⌘T"));
//!     });
//! });
//! if let Some(a) = res.activated { /* a.id was selected */ }
//!
//! // ⌘K-style modal palette:
//! let mut open = false;
//! CommandDialog::controlled(&mut open).id_salt("palette_dlg").show(ui, |c| {
//!     c.input(CommandInput::new().placeholder("Type a command or search..."));
//!     c.group("Actions", |g| { g.item(CommandItem::new("Log out")); });
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, Key, Margin, Pos2, Rect, Response, Sense,
    Stroke, Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Geometry constants
// ---------------------------------------------------------------------------

/// Standalone root width: the docs demo `md:w-[300px]`.
pub const COMMAND_DEFAULT_WIDTH: f32 = 300.0;
/// List max height: `max-h-[300px]`.
pub const COMMAND_LIST_MAX_HEIGHT: f32 = 300.0;
/// Input wrapper height: `h-9`.
pub const COMMAND_INPUT_HEIGHT: f32 = crate::theme::tokens::spacing::CONTROL;
/// Item row height: `py-1.5` + one `text-sm` line.
pub const COMMAND_ITEM_HEIGHT: f32 = 32.0;
/// Item horizontal padding: `px-2` (8 px).
const ITEM_PAD_X: f32 = 8.0;
/// Group heading height: `py-1.5` + one `text-xs` line.
const HEADING_HEIGHT: f32 = 24.0;
/// Group padding: `p-1` (4 px).
const GROUP_PADDING: f32 = 4.0;
/// Empty state height: `py-6` + one `text-sm` line.
const EMPTY_HEIGHT: f32 = 48.0;
/// Icon size: `size-4` (16 px).
const ICON_SIZE: f32 = 16.0;
/// Gap between the icon and the text field: `gap-2` (8 px).
const ICON_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Item model
// ---------------------------------------------------------------------------

/// A command entry (`CommandItem`).
#[derive(Clone, Debug)]
pub struct CommandItem {
    pub(crate) label: String,
    pub(crate) id: Option<String>,
    pub(crate) icon: Option<Icon>,
    pub(crate) shortcut: Option<String>,
    pub(crate) disabled: bool,
    /// `true` (default) hides the row while it does not match the query;
    /// `false` keeps it always visible (cmdk `forceMount`).
    pub(crate) should_filter: bool,
    /// Trailing check indicator (combobox-style items).
    pub(crate) checked: bool,
}

impl Default for CommandItem {
    fn default() -> Self {
        Self {
            label: String::new(),
            id: None,
            icon: None,
            shortcut: None,
            disabled: false,
            should_filter: true,
            checked: false,
        }
    }
}

impl CommandItem {
    /// Creates an enabled, filterable item; the activation id defaults to
    /// the label.
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

    /// Leading lucide icon (`size-4`, muted unless selected).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Right-aligned shortcut text (`ml-auto text-xs tracking-widest`).
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Disables the row (`data-[disabled=true]:opacity-50`, skipped by
    /// keyboard navigation and clicks).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Whether the row participates in query filtering (default `true`;
    /// `false` pins the row visible, cmdk `forceMount`).
    pub fn should_filter(mut self, should_filter: bool) -> Self {
        self.should_filter = should_filter;
        self
    }

    /// Trailing check indicator (combobox items mark the selected value).
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
}

/// The command search field (`CommandInput`).
#[derive(Debug, Default)]
pub struct CommandInput<'a> {
    pub(crate) placeholder: Option<String>,
    /// Binds the query to the caller's string; `None` keeps it internal.
    pub(crate) binding: Option<&'a mut String>,
}

impl<'a> CommandInput<'a> {
    /// Creates the search input with internal query state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds the query to the caller's string.
    pub fn bind(query: &'a mut String) -> Self {
        Self {
            placeholder: None,
            binding: Some(query),
        }
    }

    /// Hint text shown while the query is empty
    /// (`placeholder:text-muted-foreground`).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
}

/// One row of a command group.
#[derive(Debug)]
pub(crate) enum CommandEntry {
    Item(CommandItem),
    Separator,
}

/// A `CommandGroup`: heading + rows, hidden entirely when no row matches.
#[derive(Debug, Default)]
pub(crate) struct CommandGroup {
    pub heading: Option<String>,
    pub entries: Vec<CommandEntry>,
}

/// One block of the command body.
#[derive(Debug)]
pub(crate) enum CommandPart {
    Group(CommandGroup),
    /// Top-level items/separators (cmdk wraps them in an implicit group).
    Loose(CommandGroup),
}

/// Collects the command body inside the `show` closure.
#[derive(Debug, Default)]
pub struct CommandBuilder<'a> {
    pub(crate) input: Option<CommandInput<'a>>,
    pub(crate) parts: Vec<CommandPart>,
    pub(crate) empty_text: String,
}

impl<'a> CommandBuilder<'a> {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self {
            input: None,
            parts: Vec::new(),
            empty_text: "No results found.".to_owned(),
        }
    }

    /// Adds the search input (`CommandInput`); a second call replaces the
    /// first.
    pub fn input(&mut self, input: CommandInput<'a>) {
        self.input = Some(input);
    }

    /// Appends a `CommandGroup` with a `text-muted-foreground` heading.
    pub fn group(
        &mut self,
        heading: impl Into<String>,
        build: impl FnOnce(&mut CommandGroupBuilder<'_>),
    ) {
        let mut g = CommandGroupBuilder::new();
        build(&mut g);
        self.parts.push(CommandPart::Group(CommandGroup {
            heading: Some(heading.into()),
            entries: g.group.entries,
        }));
    }

    /// Appends an unheaded group.
    pub fn group_bare(&mut self, build: impl FnOnce(&mut CommandGroupBuilder<'_>)) {
        let mut g = CommandGroupBuilder::new();
        build(&mut g);
        self.parts.push(CommandPart::Loose(CommandGroup {
            heading: None,
            entries: g.group.entries,
        }));
    }

    /// Appends a top-level item (an implicit bare group).
    pub fn item(&mut self, item: CommandItem) {
        self.group_bare(|g| g.item(item));
    }

    /// Appends a `CommandSeparator`.
    pub fn separator(&mut self) {
        self.group_bare(|g| g.separator());
    }

    /// Overrides the empty-state text (default `"No results found."`).
    pub fn empty_text(&mut self, text: impl Into<String>) {
        self.empty_text = text.into();
    }
}

/// Builder for the rows of one [`CommandBuilder::group`].
pub struct CommandGroupBuilder<'a> {
    group: CommandGroup,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl Default for CommandGroupBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandGroupBuilder<'_> {
    /// Creates an empty group builder.
    pub fn new() -> Self {
        Self {
            group: CommandGroup::default(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Appends an item.
    pub fn item(&mut self, item: CommandItem) {
        self.group.entries.push(CommandEntry::Item(item));
    }

    /// Appends a separator line.
    pub fn separator(&mut self) {
        self.group.entries.push(CommandEntry::Separator);
    }
}

// ---------------------------------------------------------------------------
// Pure filtering / selection helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Whether `text` matches the query: case-insensitive substring (an empty
/// query matches everything).
pub fn filter_matches(query: &str, text: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    text.to_lowercase().contains(&q)
}

/// Whether a row participates in the filter pass.
fn item_matches(item: &CommandItem, query: &str) -> bool {
    !item.should_filter || filter_matches(query, &item.label)
}

/// Whether a group has any matching row (separators don't count).
fn group_matches(group: &CommandGroup, query: &str) -> bool {
    group
        .entries
        .iter()
        .any(|e| matches!(e, CommandEntry::Item(it) if item_matches(it, query)))
}

/// One keyboard-navigable row of the flattened visible list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FlatItem {
    pub id: String,
    pub disabled: bool,
}

/// The visible, keyboard-navigable rows in render order.
pub(crate) fn collect_visible_items(parts: &[CommandPart], query: &str) -> Vec<FlatItem> {
    let mut flat = Vec::new();
    for part in parts {
        let group = match part {
            CommandPart::Group(g) | CommandPart::Loose(g) => g,
        };
        if !group_matches(group, query) {
            continue;
        }
        for entry in &group.entries {
            if let CommandEntry::Item(item) = entry {
                if item_matches(item, query) {
                    flat.push(FlatItem {
                        id: item.id.clone().unwrap_or_else(|| item.label.clone()),
                        disabled: item.disabled,
                    });
                }
            }
        }
    }
    flat
}

/// Moves the selection by `delta` through the visible rows, skipping
/// disabled ones and wrapping. `None` (or an id not in the list) selects the
/// first (or last) enabled row. Returns `None` when no row is selectable.
pub(crate) fn move_selection(
    flat: &[FlatItem],
    current: Option<&str>,
    delta: isize,
) -> Option<String> {
    if flat.is_empty() {
        return None;
    }
    let start = match current.and_then(|id| flat.iter().position(|it| it.id == id)) {
        Some(i) => i as isize,
        None => match delta < 0 {
            false => -1,
            true => flat.len() as isize,
        },
    };
    let mut idx = start;
    for _ in 0..flat.len() {
        idx = (idx + delta).rem_euclid(flat.len() as isize);
        if !flat[idx as usize].disabled {
            return Some(flat[idx as usize].id.clone());
        }
    }
    None
}

/// Normalizes the selection each frame: keep it when still visible and
/// enabled, otherwise fall back to the first enabled row (cmdk selects the
/// first item by default).
pub(crate) fn normalize_selection(flat: &[FlatItem], current: Option<&str>) -> Option<String> {
    if let Some(id) = current {
        if let Some(it) = flat.iter().find(|it| it.id == id) {
            if !it.disabled {
                return Some(id.to_owned());
            }
        }
    }
    flat.iter().find(|it| !it.disabled).map(|it| it.id.clone())
}

// ---------------------------------------------------------------------------
// Row colors (pure, unit-tested)
// ---------------------------------------------------------------------------

/// Resolved colors for one command row.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RowColors {
    pub fill: Color32,
    pub text: Color32,
    pub icon: Color32,
    pub shortcut: Color32,
}

/// Mirrors `data-[selected=true]:bg-accent data-[selected=true]:
/// text-accent-foreground`, `[&_svg:not([class*='text-'])]:text-muted-foreground`,
/// the shortcut recipe and `data-[disabled=true]:opacity-50`.
pub(crate) fn resolve_row_colors(p: &Palette, selected: bool, disabled: bool) -> RowColors {
    let mut c = RowColors {
        fill: Color32::TRANSPARENT,
        text: p.popover_foreground,
        icon: p.muted_foreground,
        shortcut: p.muted_foreground,
    };
    if selected {
        c.fill = p.accent;
        c.text = p.accent_foreground;
        c.icon = p.accent_foreground;
        c.shortcut = p.accent_foreground;
    }
    if disabled {
        c.fill = color_alpha(c.fill, 0.5);
        c.text = color_alpha(c.text, 0.5);
        c.icon = color_alpha(c.icon, 0.5);
        c.shortcut = color_alpha(c.shortcut, 0.5);
    }
    c
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Per-instance state: internal query, selection id.
#[derive(Clone, Debug, Default)]
struct CommandState {
    query: String,
    selected: Option<String>,
    /// Whether the input had keyboard focus (or just released it) last
    /// frame — drives standalone keyboard navigation.
    focused: bool,
}

fn read_state(ctx: &Context, id: Id) -> CommandState {
    ctx.data_mut(|d| d.get_temp::<CommandState>(id).unwrap_or_default())
}

fn write_state(ctx: &Context, id: Id, state: CommandState) {
    ctx.data_mut(|d| d.insert_temp(id, state));
}

/// One activated command item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandActivated {
    /// Activation id (explicit `id` or the label).
    pub id: String,
    /// The visible label.
    pub label: String,
}

/// What [`Command::show`] returns.
#[derive(Clone, Debug)]
pub struct CommandResponse {
    /// The item activated this frame (Enter or click), if any.
    pub activated: Option<CommandActivated>,
    /// The query after this frame.
    pub query: String,
    /// Whether any row matched the query.
    pub has_matches: bool,
    /// The input's response (for focus checks).
    pub input: Option<Response>,
}

// ---------------------------------------------------------------------------
// Command (root)
// ---------------------------------------------------------------------------

/// A cmdk-style command palette. See the [module docs](self) for an example.
#[derive(Debug, Default)]
pub struct Command {
    width: Option<f32>,
    /// Paint the standalone chrome (`rounded-lg border shadow-md`).
    chrome: bool,
    /// Focus the input on the first frame (dialogs set this).
    autofocus: bool,
    /// Clear the autofocus bookkeeping (dialogs pass `just_opened`).
    reset_autofocus: bool,
    /// Handle Arrow/Enter keys even when the input is not focused
    /// (dialogs set this so keys work anywhere over the modal).
    force_keyboard: bool,
    id_salt: Option<String>,
}

impl Command {
    /// Creates a standalone command palette with its own chrome.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an explicit width (default [`COMMAND_DEFAULT_WIDTH`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Whether to paint the standalone chrome (`rounded-lg border shadow-md
    /// bg-popover`); turn off when embedding into a dialog or popover that
    /// already provides it.
    pub fn chrome(mut self, chrome: bool) -> Self {
        self.chrome = chrome;
        self
    }

    /// Focuses the input on the first frame (used by [`CommandDialog`]).
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }

    /// Clears the autofocus bookkeeping so the next frame focuses the input
    /// again (used by [`CommandDialog`] when the dialog reopens).
    pub fn reset_autofocus(mut self, reset: bool) -> Self {
        self.reset_autofocus = reset;
        self
    }

    /// Handles navigation keys even when the input is not focused (used by
    /// [`CommandDialog`]).
    pub fn force_keyboard(mut self, force: bool) -> Self {
        self.force_keyboard = force;
        self
    }

    /// Sets a stable id salt so query/selection state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the command palette.
    pub fn show(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut CommandBuilder<'_>),
    ) -> CommandResponse {
        let mut builder = CommandBuilder::new();
        content(&mut builder);
        self.show_builder(ui, builder)
    }

    fn show_builder(self, ui: &mut Ui, mut builder: CommandBuilder<'_>) -> CommandResponse {
        let width = self.width.unwrap_or(COMMAND_DEFAULT_WIDTH);
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_command"),
        };
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, state_id);

        if self.reset_autofocus {
            ctx.data_mut(|d| {
                d.insert_temp(state_id.with("cmd_input").with("autofocus_done"), false);
            });
        }

        // Peek the raw keys before rendering: a singleline TextEdit
        // consumes Enter (surrendering focus) during its own pass, so a
        // later `key_pressed` would miss it.
        let (down, up, home, end, enter) = ctx.input(|i| {
            (
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::Home),
                i.key_pressed(Key::End),
                i.key_pressed(Key::Enter),
            )
        });

        // Take the input out of the builder so the query binding can be
        // borrowed mutably while the parts are read immutably.
        let mut input = builder.input.take();

        // Body (chrome optional). Phase 1: input row. Phase 2: list.
        let (input_response, hover, click, query) =
            render_chrome(ui, &self, width, &builder, &mut input, &mut state);
        builder.input = input.take();

        // Keyboard: arrows cycle, Home/End jump, Enter activates.
        let mut activated = None;
        let focus_now = input_response
            .as_ref()
            .is_some_and(|r| r.has_focus() || r.lost_focus());
        let keyboard_context = self.force_keyboard || focus_now || state.focused;
        if keyboard_context {
            let flat = collect_visible_items(&builder.parts, &query);
            if down {
                if let Some(next) = move_selection(&flat, state.selected.as_deref(), 1) {
                    state.selected = Some(next);
                }
            }
            if up {
                if let Some(next) = move_selection(&flat, state.selected.as_deref(), -1) {
                    state.selected = Some(next);
                }
            }
            if home || end {
                if let Some(it) = if end {
                    flat.iter().rev().find(|it| !it.disabled)
                } else {
                    flat.iter().find(|it| !it.disabled)
                } {
                    state.selected = Some(it.id.clone());
                }
            }
            if enter {
                if let Some(item) = find_item(&builder.parts, &query, state.selected.as_deref()) {
                    activated = Some(CommandActivated {
                        id: item.id.clone().unwrap_or_else(|| item.label.clone()),
                        label: item.label.clone(),
                    });
                }
            }
        }

        // Hover moves the selection; click activates.
        if let Some(id) = hover {
            state.selected = Some(id);
        }
        if let Some(id) = click {
            if let Some(item) = find_item(&builder.parts, &query, Some(&id)) {
                activated = Some(CommandActivated {
                    id: item.id.clone().unwrap_or_else(|| item.label.clone()),
                    label: item.label.clone(),
                });
            }
        }

        let has_matches = !collect_visible_items(&builder.parts, &query).is_empty();
        state.query = query.clone();
        state.focused = focus_now;
        write_state(&ctx, state_id, state);

        CommandResponse {
            activated,
            query,
            has_matches,
            input: input_response,
        }
    }
}

/// Paints the optional chrome, then the input row and the list. Returns the
/// input response, the hover/click row ids from the list, and the query
/// after this frame.
#[allow(clippy::type_complexity)]
fn render_chrome<'b>(
    ui: &mut Ui,
    cmd: &Command,
    width: f32,
    builder: &CommandBuilder<'b>,
    input: &mut Option<CommandInput<'b>>,
    state: &mut CommandState,
) -> (Option<Response>, Option<String>, Option<String>, String) {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let dark = theme.mode.is_dark();

    // Split the query binding off the input so it can be borrowed mutably
    // while the parts are read immutably; restored before returning.
    let placeholder = input.as_ref().and_then(|i| i.placeholder.clone());
    let mut binding = input.as_mut().and_then(|i| i.binding.take());
    let mut internal_query = state.query.clone();

    let body = |ui: &mut Ui,
                binding: &mut Option<&'b mut String>,
                internal_query: &mut String,
                state: &mut CommandState|
     -> (Option<Response>, Option<String>, Option<String>) {
        ui.set_min_width(width);
        ui.set_max_width(width);

        let mut out = (None, None, None);

        // Phase 1: the input row (`h-9 border-b px-3 gap-2`), which edits
        // the query in place.
        let query: &mut String = match binding {
            Some(b) => b,
            None => internal_query,
        };
        if let Some(ph) = placeholder.as_deref() {
            let input_id = ui.scope_id().with("cmd_input");
            out.0 = Some(render_input(
                ui,
                input_id,
                Some(ph),
                query,
                cmd.autofocus,
                cmd.reset_autofocus,
            ));
        }

        // Phase 2: the filtered, scrolling list (`max-h-[300px]`).
        let flat = collect_visible_items(&builder.parts, query);
        state.selected = normalize_selection(&flat, state.selected.as_deref());
        render_list(ui, builder, width, state, query, &mut out.1, &mut out.2);
        out
    };

    let (input_response, hover, click) = if cmd.chrome {
        let frame = egui::Frame::new()
            .fill(p.popover)
            .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
            .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
            .inner_margin(Margin::same(0))
            .shadow(crate::utils::shadow::shadow_md(dark));
        frame
            .show(ui, |ui| body(ui, &mut binding, &mut internal_query, state))
            .inner
    } else {
        body(ui, &mut binding, &mut internal_query, state)
    };

    let query = match binding.as_deref() {
        Some(b) => b.clone(),
        None => internal_query.clone(),
    };
    // Restore the binding for the caller.
    if let Some(inp) = input.as_mut() {
        inp.binding = binding.take();
    }
    (input_response, hover, click, query)
}

/// Renders the input row and borderless text field; edits `query` in place.
fn render_input(
    ui: &mut Ui,
    input_id: Id,
    placeholder: Option<&str>,
    query: &mut String,
    autofocus: bool,
    reset_autofocus: bool,
) -> Response {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), COMMAND_INPUT_HEIGHT));

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        // `border-b`
        painter.line_segment(
            [
                Pos2::new(rect.min.x, rect.max.y - 0.5),
                Pos2::new(rect.max.x, rect.max.y - 0.5),
            ],
            Stroke::new(1.0, p.border),
        );

        // Search icon: `size-4 shrink-0 opacity-50`.
        let icon_rect = Rect::from_center_size(
            Pos2::new(
                rect.min.x + crate::theme::tokens::spacing::MD + ICON_SIZE * 0.5,
                rect.center().y,
            ),
            Vec2::splat(ICON_SIZE),
        );
        Icon::Search.paint(
            painter,
            icon_rect,
            color_alpha(p.muted_foreground, 0.5),
            2.0 * ICON_SIZE / 24.0,
        );
    }

    // The text field itself (`py-3 text-sm outline-hidden
    // placeholder:text-muted-foreground`).
    let input_x = rect.min.x + crate::theme::tokens::spacing::MD + ICON_SIZE + ICON_GAP;
    let field_w = rect.max.x - crate::theme::tokens::spacing::MD - input_x;
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let query_empty = query.is_empty();
    let mut edit = egui::TextEdit::singleline(query)
        .id_salt(input_id)
        .font(font)
        .desired_width(field_w.max(1.0))
        .min_size(Vec2::new(field_w.max(1.0), COMMAND_INPUT_HEIGHT))
        .align(Align2::LEFT_CENTER)
        .frame(
            egui::Frame::new()
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
                .inner_margin(Margin::same(0)),
        )
        .text_color(p.popover_foreground);
    if let Some(placeholder) = placeholder {
        if query_empty {
            edit = edit.hint_text(placeholder.to_owned());
        }
    }
    let field = ui.put(
        Rect::from_min_size(
            Pos2::new(input_x, rect.min.y),
            Vec2::new(field_w.max(1.0), COMMAND_INPUT_HEIGHT),
        ),
        edit,
    );

    // Autofocus on the first frame (dialogs clear the flag on reopen).
    let done_key = input_id.with("autofocus_done");
    if reset_autofocus {
        ui.ctx().data_mut(|d| d.insert_temp(done_key, false));
    }
    if autofocus {
        let done: bool = ui
            .ctx()
            .data_mut(|d| d.get_temp::<bool>(done_key))
            .unwrap_or(false);
        if !done {
            ui.ctx().data_mut(|d| d.insert_temp(done_key, true));
            field.request_focus();
        }
    }
    field
}

/// Renders the visible groups / rows / empty state; reports hover + click.
fn render_list(
    ui: &mut Ui,
    builder: &CommandBuilder<'_>,
    width: f32,
    state: &CommandState,
    query: &str,
    hover: &mut Option<String>,
    click: &mut Option<String>,
) {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let flat = collect_visible_items(&builder.parts, query);

    let list_id = ui.scope_id().with("cmd_list");
    crate::components::scroll_area::ScrollArea::new()
        .height(COMMAND_LIST_MAX_HEIGHT)
        .id_salt(list_id.value().to_string())
        .show(ui, |ui| {
            ui.set_min_width(width);
            if flat.is_empty() {
                paint_empty(ui, &builder.empty_text, &p);
                return;
            }
            for (part_i, part) in builder.parts.iter().enumerate() {
                let group = match part {
                    CommandPart::Group(g) | CommandPart::Loose(g) => g,
                };
                if !group_matches(group, query) {
                    continue;
                }
                egui::Frame::new()
                    .inner_margin(Margin::same(GROUP_PADDING as i8))
                    .show(ui, |ui| {
                        if let Some(heading) = &group.heading {
                            paint_heading(ui, heading, &p);
                        }
                        for (i, entry) in group.entries.iter().enumerate() {
                            match entry {
                                CommandEntry::Separator => paint_separator(ui, &p),
                                CommandEntry::Item(item) => {
                                    if !item_matches(item, query) {
                                        continue;
                                    }
                                    let id = item.id.clone().unwrap_or_else(|| item.label.clone());
                                    let selected = state.selected.as_deref() == Some(id.as_str());
                                    let row_id = list_id.with(("cmd_row", part_i, i));
                                    paint_row(ui, row_id, item, selected, &p, hover, click);
                                }
                            }
                        }
                    });
            }
        });
}

/// Looks up an item by id (value or label) among the visible items.
fn find_item<'x>(
    parts: &'x [CommandPart],
    query: &str,
    id: Option<&str>,
) -> Option<&'x CommandItem> {
    let id = id?;
    for part in parts {
        let group = match part {
            CommandPart::Group(g) | CommandPart::Loose(g) => g,
        };
        if !group_matches(group, query) {
            continue;
        }
        for entry in &group.entries {
            if let CommandEntry::Item(item) = entry {
                if item_matches(item, query) && item.id.as_deref().unwrap_or(&item.label) == id {
                    return Some(item);
                }
            }
        }
    }
    None
}

fn paint_heading(ui: &mut Ui, text: &str, p: &Palette) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), HEADING_HEIGHT));
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
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), 1.0));
    if ui.is_rect_visible(rect) {
        // `-mx-1` — bleed into the group padding.
        let y = rect.center().y;
        ui.painter().line_segment(
            [
                Pos2::new(rect.min.x - GROUP_PADDING, y),
                Pos2::new(rect.max.x + GROUP_PADDING, y),
            ],
            Stroke::new(1.0, p.border),
        );
    }
}

fn paint_empty(ui: &mut Ui, text: &str, p: &Palette) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), EMPTY_HEIGHT));
    if ui.is_rect_visible(rect) {
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            font,
            p.popover_foreground,
        );
    }
}

/// Draws one command row; reports `(hover id, click id)`.
#[allow(clippy::too_many_arguments)]
fn paint_row(
    ui: &mut Ui,
    row_id: Id,
    item: &CommandItem,
    selected: bool,
    p: &Palette,
    hover: &mut Option<String>,
    click: &mut Option<String>,
) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), COMMAND_ITEM_HEIGHT));
    let sense = if item.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(rect, row_id, sense);

    let id = item.id.clone().unwrap_or_else(|| item.label.clone());
    let colors = resolve_row_colors(p, selected, item.disabled);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if colors.fill != Color32::TRANSPARENT {
            let radius = ShadcnTheme::of_ui(ui).radius.sm_u8();
            painter.rect_filled(rect, CornerRadius::same(radius), colors.fill);
        }

        // Leading icon / label: `px-2 gap-2 text-sm`.
        let mut text_x = rect.min.x + ITEM_PAD_X;
        if let Some(icon) = item.icon {
            let icon_rect = Rect::from_center_size(
                Pos2::new(rect.min.x + ITEM_PAD_X + ICON_SIZE * 0.5, rect.center().y),
                Vec2::splat(ICON_SIZE),
            );
            icon.paint(painter, icon_rect, colors.icon, 2.0 * ICON_SIZE / 24.0);
            text_x += ICON_SIZE + ICON_GAP;
        }
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        painter.text(
            Pos2::new(text_x, rect.center().y),
            Align2::LEFT_CENTER,
            item.label.clone(),
            font,
            colors.text,
        );

        // Trailing check indicator (combobox items).
        let mut right_pad = ITEM_PAD_X;
        if item.checked {
            let check_rect = Rect::from_center_size(
                Pos2::new(rect.max.x - ITEM_PAD_X - ICON_SIZE * 0.5, rect.center().y),
                Vec2::splat(ICON_SIZE),
            );
            Icon::Check.paint(painter, check_rect, colors.text, 2.0 * ICON_SIZE / 24.0);
            right_pad += ICON_SIZE;
        }

        // Shortcut: `ml-auto text-xs tracking-widest`.
        if let Some(shortcut) = &item.shortcut {
            let sfont = FontId::proportional(crate::theme::tokens::type_scale::XS);
            painter.text(
                Pos2::new(rect.max.x - right_pad, rect.center().y),
                Align2::RIGHT_CENTER,
                shortcut,
                sfont,
                colors.shortcut,
            );
        }
    }

    if !item.disabled {
        if response.hovered() {
            *hover = Some(id.clone());
        }
        if response.clicked() {
            *click = Some(id);
        }
    }
}

// ---------------------------------------------------------------------------
// CommandDialog
// ---------------------------------------------------------------------------

/// The `⌘K` modal palette (`CommandDialog`): the command rendered inside a
/// shadcn dialog (`overflow-hidden p-0`, no chrome of its own). Closes on
/// Escape, backdrop click and on activation. Focus the input via
/// `CommandDialog::controlled(&mut open)` and set `open = true` from a `⌘K`
/// key binding.
#[derive(Debug, Default)]
pub struct CommandDialog<'a> {
    controlled: Option<&'a mut bool>,
    default_open: bool,
    width: Option<f32>,
    id_salt: Option<String>,
}

impl<'a> CommandDialog<'a> {
    /// Creates an uncontrolled command dialog (starts closed).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a command dialog whose open state is owned by the caller
    /// (e.g. a `⌘K` key binding). Selecting a command writes `false`.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            ..Self::new()
        }
    }

    /// Initial open state (uncontrolled mode only).
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Overrides the modal width (default `sm:max-w-lg`, 512 px).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a stable id salt so open/query state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the dialog (and the command inside while open).
    pub fn show(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut CommandBuilder<'_>),
    ) -> CommandDialogResponse {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_command_dialog"),
        };
        let ctx = ui.ctx().clone();

        // Resolve the open state (caller bool wins; else per-instance temp).
        let mut open = match self.controlled.as_deref() {
            Some(o) => *o,
            None => ctx
                .data_mut(|d| d.get_temp::<bool>(state_id.with("open")))
                .unwrap_or(self.default_open),
        };
        let prev_open: bool = ctx
            .data_mut(|d| d.get_temp::<bool>(state_id.with("prev_open")))
            .unwrap_or(false);
        let just_opened = open && !prev_open;

        // The dialog owns Escape/backdrop/X; mirror its state through a
        // second bool the content closure cannot touch, and capture the
        // activation through the context store. Activation closes it.
        let mut open_mirror = open;
        let activated_store = state_id.with("activated");
        let dialog_id = state_id.value().to_string();
        let width = self.width.unwrap_or(512.0);
        crate::components::dialog::Dialog::controlled(&mut open_mirror)
            .id_salt(dialog_id)
            .show(ui, |ui| {
                let res = Command::new()
                    .chrome(false)
                    .width(width - 48.0)
                    .autofocus(true)
                    .reset_autofocus(just_opened)
                    .force_keyboard(true)
                    .id_salt(format!("{}_command", state_id.value()))
                    .show(ui, content);
                ui.ctx().data_mut(|d| {
                    d.insert_temp(activated_store, res.activated.clone());
                });
            });
        let activated: Option<CommandActivated> = ctx
            .data_mut(|d| d.remove_temp::<Option<CommandActivated>>(activated_store))
            .flatten();
        // Selecting a command closes the dialog (next frame, through the
        // open bool the Dialog re-seeds from).
        if activated.is_some() {
            open_mirror = false;
        }

        let closed = open && !open_mirror;
        open = open_mirror;
        if self.controlled.is_none() {
            ctx.data_mut(|d| d.insert_temp(state_id.with("open"), open));
        }
        ctx.data_mut(|d| d.insert_temp(state_id.with("prev_open"), open));
        if let Some(out) = self.controlled {
            *out = open;
        }

        CommandDialogResponse {
            open,
            activated,
            closed,
        }
    }
}

/// What [`CommandDialog::show`] returns.
#[derive(Clone, Debug)]
pub struct CommandDialogResponse {
    /// Whether the dialog is open after this frame.
    pub open: bool,
    /// The item activated this frame (closes the dialog).
    pub activated: Option<CommandActivated>,
    /// Whether the dialog was open at frame start and is closed now.
    pub closed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts() -> Vec<CommandPart> {
        let mut b = CommandBuilder::new();
        b.group("Suggestions", |g| {
            g.item(CommandItem::new("Calendar").id("cal"));
            g.item(CommandItem::new("Calculator").disabled(true));
            g.item(CommandItem::new("Search Emoji"));
        });
        b.group("Settings", |g| {
            g.item(CommandItem::new("Theme").should_filter(false));
        });
        b.parts
    }

    #[test]
    fn filter_matches_is_case_insensitive_substring() {
        assert!(filter_matches("", "anything"));
        assert!(filter_matches("CAL", "Calendar"));
        assert!(filter_matches("end", "Calendar"));
        assert!(!filter_matches("xyz", "Calendar"));
        assert!(filter_matches("  theme  ", "Theme"));
    }

    #[test]
    fn visibility_follows_query_and_should_filter() {
        let p = parts();
        // Empty query: everything visible (4 items).
        let flat = collect_visible_items(&p, "");
        assert_eq!(flat.len(), 4);

        // "calc" matches only Calculator; the pinned Theme row stays too.
        let flat = collect_visible_items(&p, "calc");
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].id, "Calculator");
        assert!(flat[0].disabled);
        assert_eq!(flat[1].id, "Theme", "should_filter(false) stays visible");

        // A query matching nothing leaves only pinned items.
        let flat = collect_visible_items(&p, "zzz");
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].id, "Theme");
    }

    #[test]
    fn move_selection_skips_disabled_and_wraps() {
        let p = parts();
        let flat = collect_visible_items(&p, "");
        // Order: Calendar, Calculator(disabled), Search Emoji, Theme.
        assert_eq!(
            move_selection(&flat, None, 1).as_deref(),
            Some("cal"),
            "starts at the first enabled row"
        );
        assert_eq!(
            move_selection(&flat, Some("cal"), 1).as_deref(),
            Some("Search Emoji"),
            "skips the disabled Calculator"
        );
        assert_eq!(
            move_selection(&flat, Some("Theme"), 1).as_deref(),
            Some("cal"),
            "wraps to the start"
        );
        assert_eq!(
            move_selection(&flat, Some("cal"), -1).as_deref(),
            Some("Theme"),
            "wraps backward"
        );
    }

    #[test]
    fn move_selection_handles_empty_and_all_disabled() {
        let empty: Vec<FlatItem> = Vec::new();
        assert_eq!(move_selection(&empty, None, 1), None);
        let all_disabled = vec![
            FlatItem {
                id: "a".into(),
                disabled: true,
            },
            FlatItem {
                id: "b".into(),
                disabled: true,
            },
        ];
        assert_eq!(move_selection(&all_disabled, None, 1), None);
    }

    #[test]
    fn normalize_selection_falls_back_to_first_enabled() {
        let p = parts();
        let flat = collect_visible_items(&p, "");
        // Unknown id -> first enabled.
        assert_eq!(
            normalize_selection(&flat, Some("nope")).as_deref(),
            Some("cal")
        );
        // Still-visible id is kept.
        assert_eq!(
            normalize_selection(&flat, Some("Search Emoji")).as_deref(),
            Some("Search Emoji")
        );
        // Disabled selection is dropped.
        assert_eq!(
            normalize_selection(&flat, Some("Calculator")).as_deref(),
            Some("cal")
        );
        assert_eq!(normalize_selection(&[], Some("x")), None);
    }

    #[test]
    fn row_colors_follow_state() {
        let p = crate::theme::Preset::default()
            .build(crate::theme::Mode::Light)
            .palette;
        let c = resolve_row_colors(&p, false, false);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.text, p.popover_foreground);
        assert_eq!(c.icon, p.muted_foreground);
        let s = resolve_row_colors(&p, true, false);
        assert_eq!(s.fill, p.accent);
        assert_eq!(s.text, p.accent_foreground);
        let d = resolve_row_colors(&p, false, true);
        assert_eq!(d.text, color_alpha(p.popover_foreground, 0.5));
    }

    #[test]
    fn geometry_constants_match_source() {
        assert_eq!(COMMAND_DEFAULT_WIDTH, 300.0, "md:w-[300px]");
        assert_eq!(COMMAND_LIST_MAX_HEIGHT, 300.0, "max-h-[300px]");
        assert_eq!(COMMAND_INPUT_HEIGHT, 36.0, "h-9");
        assert_eq!(COMMAND_ITEM_HEIGHT, 32.0, "py-1.5 + text-sm");
        assert_eq!(EMPTY_HEIGHT, 48.0, "py-6 + text-sm");
    }

    /// Advances time so animations make progress across frames.
    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            crate::theme::Preset::default().install(&ctx);
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
                    Id::unique("command_test_ui"),
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
                        Id::unique("command_test_ui"),
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
                    Id::unique("command_test_ui"),
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
                    Id::unique("command_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }
    }

    fn demo_command(ui: &mut Ui) -> CommandResponse {
        Command::new().id_salt("ui_cmd").show(ui, |c| {
            c.input(CommandInput::new().placeholder("Type a command or search..."));
            c.group("Actions", |g| {
                g.item(CommandItem::new("Log out").id("logout"));
                g.item(CommandItem::new("Delete").disabled(true));
                g.item(CommandItem::new("Refresh"));
            });
        })
    }

    #[test]
    fn typing_filters_and_enter_activates() {
        let mut h = Harness::new();
        // Lay out once so ids exist.
        h.run(|ui| {
            demo_command(ui);
        });

        // Focus the input by clicking it (top row of the palette).
        let mut input_rect = Rect::ZERO;
        h.run(|ui| {
            let res = demo_command(ui);
            input_rect = res.input.unwrap().rect;
        });
        h.click(input_rect.center(), |ui| {
            demo_command(ui);
        });

        // Type "ref": only Refresh remains visible; Enter activates it.
        h.text("ref", |ui| {
            demo_command(ui);
        });
        let ctx = h.ctx.clone();
        h.key(Key::Enter, |ui| {
            let res = demo_command(ui);
            if res.activated.is_some() {
                ctx.data_mut(|d| {
                    d.insert_temp(Id::unique("cmd_activated"), res.activated);
                });
            }
        });
        let activated: Option<CommandActivated> = h
            .ctx
            .data_mut(|d| d.get_temp::<Option<CommandActivated>>(Id::unique("cmd_activated")))
            .flatten();
        assert!(
            activated.as_ref().is_some_and(|a| a.id == "Refresh"),
            "Enter after typing activates the remaining item, got {activated:?}"
        );
    }

    #[test]
    fn clicking_a_row_activates_it() {
        let mut h = Harness::new();
        let mut input_rect = Rect::ZERO;
        h.run(|ui| {
            input_rect = demo_command(ui).input.unwrap().rect;
        });
        h.run(|ui| {
            demo_command(ui);
        });

        // First row under the input: 36 (input) + 8 (spacing) + 4 (group
        // pad) + 24 (heading) + 8 (spacing) + 16 (half row).
        let row_pos = Pos2::new(input_rect.center().x, input_rect.max.y + 60.0);
        let ctx = h.ctx.clone();
        h.click(row_pos, |ui| {
            let res = demo_command(ui);
            if res.activated.is_some() {
                let a = res.activated;
                ctx.data_mut(|d| d.insert_temp(Id::unique("cmd_click"), a));
            }
        });
        let activated: Option<CommandActivated> = h
            .ctx
            .data_mut(|d| d.get_temp::<Option<CommandActivated>>(Id::unique("cmd_click")))
            .flatten();
        assert!(
            activated.as_ref().is_some_and(|a| a.id == "logout"),
            "clicking the first row activates it, got {activated:?}"
        );
    }

    #[test]
    fn dialog_opens_selects_and_closes() {
        let mut h = Harness::new();
        let mut open = false;
        // Closed frame.
        h.run(|ui| {
            open = false;
            let _ = CommandDialog::controlled(&mut open).show(ui, |c| {
                c.input(CommandInput::new());
                c.group("A", |g| g.item(CommandItem::new("Log out").id("logout")));
            });
        });
        assert!(!open);

        // Caller opens it.
        open = true;
        h.run(|ui| {
            let res = CommandDialog::controlled(&mut open).show(ui, |c| {
                c.input(CommandInput::new());
                c.group("A", |g| g.item(CommandItem::new("Log out").id("logout")));
            });
            assert!(res.open);
        });
        assert!(open, "dialog stays open");

        // Settle one frame (modal warm-up), then Enter activates the first
        // item and closes the dialog.
        h.run(|ui| {
            CommandDialog::controlled(&mut open).show(ui, |c| {
                c.input(CommandInput::new());
                c.group("A", |g| g.item(CommandItem::new("Log out").id("logout")));
            });
        });
        let ctx = h.ctx.clone();
        h.key(Key::Enter, |ui| {
            let res = CommandDialog::controlled(&mut open).show(ui, |c| {
                c.input(CommandInput::new());
                c.group("A", |g| g.item(CommandItem::new("Log out").id("logout")));
            });
            if res.activated.is_some() {
                ctx.data_mut(|d| {
                    d.insert_temp(Id::unique("cmd_dlg_hit"), true);
                });
            }
        });
        let hit: bool = h
            .ctx
            .data_mut(|d| d.get_temp(Id::unique("cmd_dlg_hit")))
            .unwrap_or(false);
        assert!(hit, "Enter activates the selected item in the dialog");
        assert!(!open, "activation closes the dialog");
    }

    #[test]
    fn builders_keep_fields() {
        let mut b = CommandBuilder::new();
        b.input(CommandInput::new().placeholder("Search…"));
        b.group("G", |g| {
            g.item(CommandItem::new("A").shortcut("⌘A").icon(Icon::Copy));
            g.separator();
            g.item(CommandItem::new("B").disabled(true).should_filter(false));
        });
        b.item(CommandItem::new("C"));
        b.empty_text("Nothing here.");
        assert_eq!(b.empty_text, "Nothing here.");
        let input = b.input.as_ref().unwrap();
        assert_eq!(input.placeholder.as_deref(), Some("Search…"));
        assert!(input.binding.is_none());
        assert_eq!(b.parts.len(), 2, "one group + one loose-item group");
        match &b.parts[0] {
            CommandPart::Group(g) => {
                assert_eq!(g.heading.as_deref(), Some("G"));
                assert_eq!(g.entries.len(), 3);
            }
            other => panic!("expected group, got {other:?}"),
        }
    }
}
