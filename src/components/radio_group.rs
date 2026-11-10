//! shadcn `radio-group.tsx` — mutually exclusive option selector.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/radio-group.tsx`:
//! `RadioGroup` lays items out in a `grid gap-3`; each `RadioGroupItem` is a
//! `size-4 rounded-full border border-input shadow-xs` circle whose
//! indicator is a `size-2 fill-primary` dot (`dark:bg-input/30`), with the
//! shared focus/invalid/disabled styles.
//!
//! Nova recipe deltas: the group gaps at `gap-2`, and the checked item is a
//! solid primary disc (`data-checked:bg-primary data-checked:border-primary`)
//! whose indicator dot is `bg-primary-foreground`.
//!
//! Radix behavior is preserved: clicking an item selects it, Arrow keys move
//! the selection and focus (no wrap-around; egui spatial navigation), and
//! Space selects the focused item.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut plan = Some(0);
//! RadioGroup::new(&mut plan)
//!     .option("Default")
//!     .option("Comfortable")
//!     .option_disabled("Compact")
//!     .id_salt("plan")
//!     .show(ui);
//! ```

use crate::components::button::GroupOrientation;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// `size-4` — the radio item side length.
pub const RADIO_SIZE: f32 = 16.0;
/// `size-2` — the selected indicator dot diameter.
pub const RADIO_DOT_SIZE: f32 = 8.0;

/// The gap between items in the group: base `gap-3` (12), nova `gap-2` (8).
fn gap(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0,
        _ => crate::theme::tokens::spacing::MD,
    }
}

/// One selectable option inside a [`RadioGroup`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioOption {
    /// The visible label.
    pub label: String,
    /// `disabled` — the option cannot be selected.
    pub disabled: bool,
}

impl RadioOption {
    /// Creates an enabled option with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    /// Creates a disabled option with the given label.
    pub fn disabled(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: true,
        }
    }
}

/// Resolved colors for one radio item state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct RadioColors {
    fill: Color32,
    stroke: Stroke,
    /// Focus ring color (alpha already applied).
    ring: Color32,
    /// Indicator dot color (`Color32::TRANSPARENT` when unchecked).
    dot: Color32,
}

/// Mirrors the base recipe: `border-input dark:bg-input/30`, indicator
/// `fill-primary`, `aria-invalid:border-destructive
/// aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40`.
///
/// Style deltas (nova recipe): `data-checked:bg-primary
/// data-checked:border-primary` with a `bg-primary-foreground` dot (solid
/// disc), `dark:aria-invalid:border-destructive/50`, and
/// `aria-invalid:aria-checked:border-primary` (a checked item keeps its
/// primary border even when invalid), plus `focus-visible:border-ring`.
pub(crate) fn resolve_radio_colors(
    p: &Palette,
    dark: bool,
    style: Style,
    checked: bool,
    invalid: bool,
    focused: bool,
) -> RadioColors {
    let nova = style == Style::Nova;
    let fill = if nova && checked {
        p.primary
    } else if dark {
        color_alpha(p.input, 0.3)
    } else {
        Color32::TRANSPARENT
    };
    let stroke_color = if nova && checked {
        // `data-checked:border-primary aria-invalid:aria-checked:border-primary`
        p.primary
    } else if invalid {
        // `aria-invalid:border-destructive` (nova fades the dark border: /50)
        if nova && dark {
            color_alpha(p.destructive, 0.5)
        } else {
            p.destructive
        }
    } else if focused {
        // `focus-visible:border-ring` (invalid wins: handled above)
        p.ring
    } else {
        p.input
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    RadioColors {
        fill,
        stroke: Stroke::new(1.0, stroke_color),
        ring,
        dot: if checked {
            if nova {
                p.primary_foreground
            } else {
                p.primary
            }
        } else {
            Color32::TRANSPARENT
        },
    }
}

/// Paints one radio item on the given rect (shared by group and standalone).
pub(crate) fn paint_radio(
    ui: &Ui,
    rect: Rect,
    colors: &RadioColors,
    disabled: bool,
    focused: bool,
    radius: u8,
) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    let corner = CornerRadius::same(radius);

    // `shadow-xs`
    if !disabled {
        let dark = ShadcnTheme::of_ui(ui).mode.is_dark();
        let sh = crate::utils::shadow::shadow_xs(dark);
        painter.add(
            egui::epaint::RectShape::filled(
                rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                corner,
                sh.color,
            )
            .with_blur_width(2.0),
        );
    }

    let mut fill = colors.fill;
    let mut stroke = colors.stroke;
    let mut dot = colors.dot;
    if disabled {
        // `disabled:opacity-50`
        fill = color_alpha(fill, 0.5);
        stroke.color = color_alpha(stroke.color, 0.5);
        dot = color_alpha(dot, 0.5);
    }

    painter.rect_filled(rect, corner, fill);
    painter.circle_stroke(rect.center(), RADIO_SIZE / 2.0, stroke);

    // `size-2 fill-primary` indicator dot.
    if colors.dot != Color32::TRANSPARENT {
        painter.circle_filled(rect.center(), RADIO_DOT_SIZE / 2.0, dot);
    }

    // `focus-visible:border-ring focus-visible:ring-[3px]
    // focus-visible:ring-ring/50`
    if focused && !disabled {
        let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
        crate::utils::focus::paint_ring(painter, ring_rect, colors.ring, 3.0, radius as f32);
    }
}

/// A single radio circle bound to a boolean (`RadioGroupItem`).
///
/// Clicking or pressing Space selects it (radios never uncheck themselves).
/// For a group of mutually exclusive options prefer [`RadioGroup`].
#[derive(Debug)]
pub struct RadioGroupItem<'a> {
    checked: &'a mut bool,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> RadioGroupItem<'a> {
    /// Creates a standalone radio item bound to the given boolean.
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Disables the item (`disabled:opacity-50 disabled:pointer-events-none`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the item aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so focus state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the item.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_radio_item"),
        };
        // Reserve space without an auto-id widget so egui's built-in
        // spatial arrow navigation only sees the explicit item ids.
        let (_, rect) = ui.allocate_space(Vec2::splat(RADIO_SIZE));
        let mut response = ui.interact(rect, id, sense);

        if response.clicked() && !self.disabled {
            *self.checked = true;
            response.mark_changed();
        }

        let checked = *self.checked;
        let colors = resolve_radio_colors(
            &p,
            theme.mode.is_dark(),
            theme.style,
            checked,
            self.invalid,
            response.has_focus(),
        );
        paint_radio(
            ui,
            rect,
            &colors,
            self.disabled,
            response.has_focus(),
            theme.radius.md_u8(),
        );

        response
    }
}

impl Widget for RadioGroupItem<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A shadcn radio group. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct RadioGroup<'a> {
    selected: &'a mut Option<usize>,
    options: Vec<RadioOption>,
    orientation: GroupOrientation,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> RadioGroup<'a> {
    /// Creates a group bound to the selected index (`None` = no selection).
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self {
            selected,
            options: Vec::new(),
            orientation: GroupOrientation::Vertical,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Adds an enabled option (`RadioGroupItem` with a label).
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options.push(RadioOption::new(label));
        self
    }

    /// Adds a disabled option.
    pub fn option_disabled(mut self, label: impl Into<String>) -> Self {
        self.options.push(RadioOption::disabled(label));
        self
    }

    /// Adds several options at once.
    pub fn options(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.options
            .extend(labels.into_iter().map(RadioOption::new));
        self
    }

    /// Sets the layout orientation (`grid` is vertical by default; the group
    /// can also lay items out in a row).
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables the whole group.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks every item aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt (used to derive per-item focus ids for the
    /// roving-focus behavior).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group with its options.
    pub fn show(self, ui: &mut Ui) -> egui::InnerResponse<()> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();
        let radius = theme.radius.md_u8();

        let group_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_radio_group"),
        };

        let RadioGroup {
            selected,
            options,
            orientation,
            disabled,
            invalid,
            id_salt: _,
        } = self;

        let style = ItemStyle {
            p: &p,
            dark,
            style: theme.style,
            radius,
            disabled,
            invalid,
        };
        let mut run_items = |ui: &mut Ui| {
            render_items(ui, group_id, selected, &options, &style);
        };
        match orientation {
            GroupOrientation::Vertical => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = gap(theme.style);
                run_items(ui);
            }),
            GroupOrientation::Horizontal => ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap(theme.style);
                run_items(ui);
            }),
        }
    }
}

/// Shared per-group style/state for item rendering.
struct ItemStyle<'a> {
    p: &'a Palette,
    dark: bool,
    style: Style,
    radius: u8,
    disabled: bool,
    invalid: bool,
}

fn render_items(
    ui: &mut Ui,
    group_id: egui::Id,
    selected: &mut Option<usize>,
    options: &[RadioOption],
    style: &ItemStyle<'_>,
) {
    let ItemStyle {
        p,
        dark,
        style,
        radius,
        disabled,
        invalid,
    } = *style;
    let mut responses: Vec<Response> = Vec::with_capacity(options.len());
    for (i, option) in options.iter().enumerate() {
        // Disabled items are not focusable (`Sense::hover`), so egui's
        // built-in spatial Arrow-key navigation skips them automatically.
        let sense = if disabled || option.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (_, rect) = ui.allocate_space(Vec2::splat(RADIO_SIZE));
        let response = ui.interact(rect, group_id.with(i), sense);
        let is_selected = *selected == Some(i);
        let colors =
            resolve_radio_colors(p, dark, style, is_selected, invalid, response.has_focus());
        paint_radio(
            ui,
            rect,
            &colors,
            disabled || option.disabled,
            response.has_focus(),
            radius,
        );

        // Clicking selects (radios never uncheck).
        if response.clicked() && !disabled && !option.disabled && *selected != Some(i) {
            *selected = Some(i);
        }
        responses.push(response);
    }

    // Roving focus, mirroring Radix's `RovingFocusGroup`: focus movement is
    // left to egui's spatial Arrow-key navigation (Up/Down/Left/Right, no
    // wrap-around); whichever item holds focus becomes the selection.
    let focused_now = responses.iter().position(|r| r.has_focus());
    let last_focused: Option<usize> = ui
        .ctx()
        .data_mut(|d| d.get_temp(group_id.with("last_focused")));
    if let Some(focused) = focused_now {
        if last_focused != focused_now
            && !disabled
            && !options[focused].disabled
            && *selected != Some(focused)
        {
            *selected = Some(focused);
        }
    }
    ui.ctx()
        .data_mut(|d| d.insert_temp(group_id.with("last_focused"), focused_now));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Key, Pos2};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    fn key_input(key: Key) -> egui::RawInput {
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        });
        input
    }

    #[test]
    fn checked_item_shows_primary_dot() {
        let p = palette(Mode::Light);
        // Base: transparent fill, `fill-primary` dot, `border-input`.
        let c = resolve_radio_colors(&p, false, Style::Vega, true, false, false);
        assert_eq!(c.dot, p.primary);
        assert_eq!(c.stroke.color, p.input);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        let u = resolve_radio_colors(&p, false, Style::Vega, false, false, false);
        assert_eq!(u.dot, Color32::TRANSPARENT);
        let d = resolve_radio_colors(&palette(Mode::Dark), true, Style::Vega, false, false, false);
        assert_eq!(d.fill, color_alpha(palette(Mode::Dark).input, 0.3));
        // Nova: solid primary disc with a `primary-foreground` dot.
        let n = resolve_radio_colors(&p, false, Style::Nova, true, false, false);
        assert_eq!(n.fill, p.primary);
        assert_eq!(n.stroke.color, p.primary);
        assert_eq!(n.dot, p.primary_foreground);
        let dp = palette(Mode::Dark);
        let nd = resolve_radio_colors(&dp, true, Style::Nova, true, false, false);
        assert_eq!(nd.fill, dp.primary);
        assert_eq!(nd.dot, dp.primary_foreground);
    }

    #[test]
    fn focused_border_uses_ring() {
        let p = palette(Mode::Light);
        let f = resolve_radio_colors(&p, false, Style::Vega, false, false, true);
        assert_eq!(f.stroke.color, p.ring);
        let nf = resolve_radio_colors(&p, false, Style::Nova, false, false, true);
        assert_eq!(nf.stroke.color, p.ring);
        // Invalid wins over focus: `aria-invalid:border-destructive`.
        let fi = resolve_radio_colors(&p, false, Style::Nova, false, true, true);
        assert_eq!(fi.stroke.color, p.destructive);
    }

    #[test]
    fn nova_invalid_checked_keeps_primary() {
        let p = palette(Mode::Light);
        // Nova `aria-invalid:aria-checked:border-primary`.
        let ic = resolve_radio_colors(&p, false, Style::Nova, true, true, false);
        assert_eq!(ic.stroke.color, p.primary);
        assert_eq!(ic.fill, p.primary);
        assert_eq!(ic.dot, p.primary_foreground);
        // Base fades nothing: a checked invalid item still gets the
        // destructive border (`aria-invalid:border-destructive`).
        let bc = resolve_radio_colors(&p, false, Style::Vega, true, true, false);
        assert_eq!(bc.stroke.color, p.destructive);
        let dark = palette(Mode::Dark);
        let dn = resolve_radio_colors(&dark, true, Style::Nova, false, true, false);
        // `dark:aria-invalid:border-destructive/50`
        assert_eq!(dn.stroke.color, color_alpha(dark.destructive, 0.5));
    }

    #[test]
    fn invalid_item_uses_destructive() {
        let p = palette(Mode::Light);
        let c = resolve_radio_colors(&p, false, Style::Vega, false, true, false);
        assert_eq!(c.stroke.color, p.destructive);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        let c = resolve_radio_colors(&dark, true, Style::Vega, false, true, false);
        assert_eq!(c.ring, color_alpha(dark.destructive, 0.4));
    }

    #[test]
    fn gap_matches_style() {
        // Base `gap-3` (12), nova `gap-2` (8).
        assert_eq!(gap(Style::Vega), 12.0);
        assert_eq!(gap(Style::Nova), 8.0);
    }

    #[test]
    fn arrows_move_selection_and_focus() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_arrows_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_arrows")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        // Focus the selected item (roving focus starts on the selection).
        let base = egui::Id::unique("radio_arrows_test").with("radio_arrows");
        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        let arrow_input = key_input(Key::ArrowDown);

        // ArrowDown: egui's spatial navigation moves focus down one item; on
        // the next frame the item that gained focus becomes the selection.
        run(&ctx, &mut selected, arrow_input.clone());
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(1));

        run(&ctx, &mut selected, arrow_input);
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(2));

        // ArrowUp moves back up.
        let up_input = key_input(Key::ArrowUp);
        run(&ctx, &mut selected, up_input);
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn arrows_skip_disabled_options() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);
        let base = egui::Id::unique("radio_skip_test").with("radio_skip");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_skip_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_skip")
                    .option("A")
                    .option_disabled("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        let arrow_input = key_input(Key::ArrowDown);

        // The disabled item B is not focusable, so navigation lands on C.
        run(&ctx, &mut selected, arrow_input);
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(2));
    }

    #[test]
    fn builders_keep_fields() {
        let mut selected = None;
        let group = RadioGroup::new(&mut selected)
            .option("A")
            .option_disabled("B")
            .options(["C", "D"])
            .orientation(GroupOrientation::Horizontal)
            .disabled(true)
            .invalid(true)
            .id_salt("g");
        assert_eq!(group.options.len(), 4);
        assert!(group.options[1].disabled);
        assert!(!group.options[2].disabled);
        assert_eq!(group.orientation, GroupOrientation::Horizontal);
        assert!(group.disabled);
        assert!(group.invalid);
        assert_eq!(group.id_salt.as_deref(), Some("g"));

        let mut checked = false;
        let item = RadioGroupItem::new(&mut checked)
            .disabled(true)
            .invalid(true)
            .id_salt("item");
        assert!(item.disabled);
        assert!(item.invalid);
        assert_eq!(item.id_salt.as_deref(), Some("item"));
    }

    #[test]
    fn item_is_sixteen_pt() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("radio_size_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
            );
            rect = RadioGroupItem::new(&mut checked).show(ui).rect;
        });
        out.textures_delta.clear();
        assert_eq!(rect.size(), Vec2::splat(RADIO_SIZE));
    }
}
