//! shadcn `toggle.tsx` — a two-state pressed/unpressed button.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/toggle.tsx` (`toggleVariants`):
//! `inline-flex items-center justify-center gap-2 rounded-md text-sm
//! font-medium` with `hover:bg-muted hover:text-muted-foreground`,
//! `data-[state=on]:bg-accent data-[state=on]:text-accent-foreground`, and
//! variant/size presets:
//!
//! | variant | classes |
//! |---|---|
//! | default | `bg-transparent` |
//! | outline | `border border-input bg-transparent shadow-xs hover:bg-accent` |
//!
//! | size | classes |
//! |---|---|
//! | default | `h-9 min-w-9 px-2` |
//! | sm | `h-8 min-w-8 px-1.5` |
//! | lg | `h-10 min-w-10 px-2.5` |
//!
//! The nova style (`.cn-toggle*`) tightens the geometry (`h-8/h-7/h-9`,
//! `px-2.5`, `gap-1`, `rounded-lg`, sm `rounded-[min(var(--radius-md),12px)]`
//! with a 12.8pt label and 14pt icon), keeps the pressed fill on `bg-muted`
//! and the hover text on `text-foreground`, and drops the outline
//! `shadow-xs`. The `metrics` module holds those overrides; the
//! [`ToggleSize`] methods keep the base recipe.
//!
//! Inside a [`crate::components::toggle_group::ToggleGroup`] with `spacing 0`
//! (or a `ButtonGroup`), corners and borders fuse automatically via the
//! shared group protocol in `crate::components::button`; fused items also
//! drop the outline shadow (`data-[spacing=0]:shadow-none`).
//!
//! Keyboard: Space or Enter toggles (egui activates focused widgets).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut bold = false;
//! Toggle::new(&mut bold)
//!     .text("Bold")
//!     .icon(Icon::Sparkles)
//!     .show(ui);
//! ```

use crate::components::button::{fuse_radius, group, GroupOrientation, GroupPosition};
use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{
    Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget,
};

/// Visual style variants, mirroring the source `toggleVariants.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleVariant {
    /// `bg-transparent` — no border, muted hover.
    #[default]
    Default,
    /// `border border-input bg-transparent shadow-xs hover:bg-accent`.
    Outline,
}

/// Size presets, mirroring the source `toggleVariants.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleSize {
    /// `h-9 min-w-9 px-2` (36px tall).
    #[default]
    Default,
    /// `h-8 min-w-8 px-1.5` (32px tall).
    Sm,
    /// `h-10 min-w-10 px-2.5` (40px tall).
    Lg,
}

impl ToggleSize {
    /// Outer height in points.
    pub fn height(self) -> f32 {
        match self {
            ToggleSize::Default => crate::theme::tokens::spacing::CONTROL,
            ToggleSize::Sm => crate::theme::tokens::spacing::CONTROL_SM,
            ToggleSize::Lg => crate::theme::tokens::spacing::CONTROL_LG,
        }
    }

    /// Minimum width in points (`min-w-9` etc.).
    pub fn min_width(self) -> f32 {
        self.height()
    }

    /// Horizontal padding in points (`px-2` etc.).
    pub fn padding_x(self) -> f32 {
        match self {
            ToggleSize::Default => crate::theme::tokens::spacing::SM,
            ToggleSize::Sm => crate::theme::tokens::spacing::XS_HALF,
            ToggleSize::Lg => crate::theme::tokens::spacing::SM_HALF,
        }
    }

    /// Gap between icon and label (`gap-2`).
    pub fn gap(self) -> f32 {
        crate::theme::tokens::spacing::SM
    }

    /// Icon size in points (`size-4`).
    pub fn icon_size(self) -> f32 {
        crate::theme::tokens::spacing::LG
    }
}

/// Per-style size metrics: the [`ToggleSize`] methods hold the base recipe
/// (`toggleVariants`); these helpers branch on [`Style`] so the nova
/// overrides (`.cn-toggle`, `.cn-toggle-size-*`) apply wherever the active
/// theme is known.
pub(crate) mod metrics {
    use super::ToggleSize;
    use crate::theme::{RadiusTokens, Style};

    /// Outer height (`h-9/h-8/h-10` base; nova `h-8/h-7/h-9`).
    pub(crate) fn height(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ToggleSize::Default => 32.0, // h-8
                ToggleSize::Sm => 28.0,      // h-7
                ToggleSize::Lg => 36.0,      // h-9
            },
            _ => size.height(),
        }
    }

    /// Horizontal padding (`px-2/px-1.5/px-2.5` base; nova `px-2.5` on all).
    pub(crate) fn padding_x(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => 10.0, // px-2.5
            _ => size.padding_x(),
        }
    }

    /// Gap between icon and label (`gap-2` base; nova `gap-1`).
    pub(crate) fn gap(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => 4.0, // gap-1
            _ => size.gap(),
        }
    }

    /// Icon size (`size-4`; nova sm `size-3.5`).
    pub(crate) fn icon_size(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ToggleSize::Sm => 14.0, // size-3.5
                _ => 16.0,              // size-4
            },
            _ => size.icon_size(),
        }
    }

    /// Label size in points (`text-sm`; nova sm `text-[0.8rem]`).
    pub(crate) fn text_size(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ToggleSize::Sm => 12.8, // text-[0.8rem]
                _ => 14.0,              // text-sm
            },
            _ => 14.0, // text-sm
        }
    }

    /// Corner radius: `rounded-md` base; nova `rounded-lg` with the sm size
    /// on `rounded-[min(var(--radius-md),12px)]`.
    pub(crate) fn radius(radius: &RadiusTokens, size: ToggleSize, style: Style) -> u8 {
        match style {
            Style::Nova => match size {
                ToggleSize::Sm => radius.md().min(12.0).round() as u8,
                _ => radius.lg_u8(),
            },
            _ => radius.md_u8(),
        }
    }
}

/// Resolved colors for one toggle state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ToggleColors {
    fill: Color32,
    fg: Color32,
    stroke: Stroke,
    /// Focus ring color (alpha already applied).
    ring: Color32,
}

/// Mirrors the base recipe: `hover:bg-muted hover:text-muted-foreground`,
/// `data-[state=on]:bg-accent data-[state=on]:text-accent-foreground`,
/// outline `border border-input shadow-xs`. Nova (`aria-pressed:bg-muted
/// data-[state=on]:bg-muted hover:text-foreground`) keeps the pressed text
/// `accent-foreground`, moves the pressed fill to `muted` and the hover
/// text to `foreground`.
pub(crate) fn resolve_toggle_colors(
    variant: ToggleVariant,
    p: &Palette,
    dark: bool,
    hovered: bool,
    pressed: bool,
    style: Style,
) -> ToggleColors {
    let (fill, fg) = if pressed {
        let fill = match style {
            Style::Nova => p.muted, // data-[state=on]:bg-muted
            _ => p.accent,          // data-[state=on]:bg-accent
        };
        (fill, p.accent_foreground)
    } else if hovered {
        let fg = match style {
            Style::Nova => p.foreground, // hover:text-foreground
            _ => p.muted_foreground,     // hover:text-muted-foreground
        };
        (p.muted, fg)
    } else {
        (Color32::TRANSPARENT, p.foreground)
    };
    let stroke = match variant {
        ToggleVariant::Default => Stroke::NONE,
        ToggleVariant::Outline => Stroke::new(1.0, if dark { p.input } else { p.border }),
    };
    ToggleColors {
        fill,
        fg,
        stroke,
        ring: color_alpha(p.ring, 0.5),
    }
}

/// Whether the outline variant paints its `shadow-xs`: the base recipe does
/// for standalone toggles, nova drops it entirely and items fused into a
/// group (`data-[spacing=0]`) render `shadow-none` in both styles.
pub(crate) fn outline_has_shadow(style: Style, position: GroupPosition) -> bool {
    style != Style::Nova && position == GroupPosition::Only
}

/// A shadcn toggle. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Toggle<'a> {
    pressed: &'a mut bool,
    text: Option<String>,
    icon: Option<Icon>,
    variant: ToggleVariant,
    size: ToggleSize,
    padding_x_override: Option<f32>,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> Toggle<'a> {
    /// Creates a toggle bound to the given pressed state.
    pub fn new(pressed: &'a mut bool) -> Self {
        Self {
            pressed,
            text: None,
            icon: None,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            padding_x_override: None,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Sets the label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Shows an icon before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the visual variant (default: [`ToggleVariant::Default`]).
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size (default: [`ToggleSize::Default`]).
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// Overrides the horizontal padding. `ToggleGroupItem` uses `px-3`.
    pub fn padding_x(mut self, padding_x: f32) -> Self {
        self.padding_x_override = Some(padding_x);
        self
    }

    /// Disables the toggle (`disabled:pointer-events-none disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the toggle aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so focus state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Content width for the current text/icon, before `min_width`.
    fn content_width(&self, ui: &Ui, style: Style) -> f32 {
        let text_w = match self.text.as_deref() {
            Some(t) if !t.is_empty() => {
                ui.painter()
                    .layout_no_wrap(
                        t.to_owned(),
                        FontId::proportional(metrics::text_size(self.size, style)),
                        Color32::WHITE,
                    )
                    .size()
                    .x
            }
            _ => 0.0,
        };
        let icon_w = self
            .icon
            .map(|_| metrics::icon_size(self.size, style))
            .unwrap_or(0.0);
        let children = self.text.is_some() as i32 + self.icon.is_some() as i32;
        let gaps = (children - 1).max(0) as f32 * metrics::gap(self.size, style);
        let px = self
            .padding_x_override
            .unwrap_or_else(|| metrics::padding_x(self.size, style));
        px * 2.0 + icon_w + gaps + text_w
    }

    /// Renders the toggle.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let style = theme.style;
        let size = self.size;

        // Auto-detected position inside a group (ButtonGroup / ToggleGroup
        // with spacing 0): corners fuse like `rounded-l-none`.
        let grp = group::next_child_position(ui);
        let position = grp.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = grp.map(|(o, _)| o).unwrap_or(GroupOrientation::Horizontal);

        let height = metrics::height(size, style);
        let desired = Vec2::new(self.content_width(ui, style).max(height), height);
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_toggle"),
        };
        let (_, rect) = ui.allocate_space(desired);
        let response = ui.interact(rect, id, sense);

        if response.clicked() && !self.disabled {
            *self.pressed = !*self.pressed;
        }

        if ui.is_rect_visible(rect) {
            let pressed = *self.pressed;
            let hovered = response.hovered() && !self.disabled;
            let mut c = resolve_toggle_colors(self.variant, &p, dark, hovered, pressed, style);

            if self.invalid {
                c.ring = color_alpha(p.destructive, if dark { 0.4 } else { 0.2 });
                c.stroke = Stroke::new(1.0, p.destructive);
            }
            // `focus-visible:border-ring` (invalid keeps its border).
            if response.has_focus() && !self.disabled && !self.invalid && c.stroke.width > 0.0 {
                c.stroke.color = p.ring;
            }
            if self.disabled {
                c.fill = color_alpha(c.fill, 0.5);
                c.fg = color_alpha(c.fg, 0.5);
                c.stroke.color = color_alpha(c.stroke.color, 0.5);
            }

            let radius = metrics::radius(&theme.radius, size, style);
            let corner = fuse_radius(CornerRadius::same(radius), orientation, position);
            let painter = ui.painter();

            // `shadow-xs` on the outline variant; nova drops it entirely and
            // fused group items render `data-[spacing=0]:shadow-none`.
            if self.variant == ToggleVariant::Outline
                && !self.disabled
                && outline_has_shadow(style, position)
            {
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

            // Non-first children of a group overlap the previous border so
            // the shared edge shows one 1px divider (`border-l-0`).
            let draw_rect = match orientation {
                GroupOrientation::Horizontal
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(rect.min + egui::vec2(1.0, 0.0), rect.max)
                }
                GroupOrientation::Vertical
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(rect.min + egui::vec2(0.0, 1.0), rect.max)
                }
                _ => rect,
            };

            if c.fill != Color32::TRANSPARENT {
                painter.rect_filled(draw_rect, corner, c.fill);
            }
            if c.stroke.width > 0.0 {
                painter.rect_stroke(
                    draw_rect,
                    corner,
                    c.stroke,
                    egui::epaint::StrokeKind::Inside,
                );
            }

            // Content: [icon] gap [text], centered.
            let icon_size = metrics::icon_size(size, style);
            let text = self.text.clone().unwrap_or_default();
            let font = FontId::proportional(metrics::text_size(size, style));
            let text_w = if text.is_empty() {
                0.0
            } else {
                painter
                    .layout_no_wrap(text.clone(), font.clone(), c.fg)
                    .size()
                    .x
            };
            let has_icon = self.icon.is_some();
            let children = has_icon as i32 + (!text.is_empty()) as i32;
            let gaps = (children - 1).max(0) as f32 * metrics::gap(size, style);
            let total_w = icon_size * has_icon as i32 as f32 + gaps + text_w;
            let center = draw_rect.center();
            let mut cursor = center.x - total_w * 0.5;

            if let Some(icon) = self.icon {
                let icon_rect = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, icon_rect, c.fg, 2.0 * icon_size / 24.0);
                cursor += icon_size + size.gap();
            }
            if !text.is_empty() {
                painter.text(
                    Pos2::new(cursor + text_w * 0.5, center.y),
                    Align2::CENTER_CENTER,
                    text,
                    font,
                    c.fg,
                );
            }

            // `focus-visible:border-ring focus-visible:ring-[3px]
            // focus-visible:ring-ring/50`
            if response.has_focus() && !self.disabled {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, c.ring, 3.0, radius as f32);
            }
        }

        response
    }
}

impl Widget for Toggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset, Style};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn size_geometry_matches_source() {
        // default h-9 min-w-9 px-2, sm h-8 min-w-8 px-1.5, lg h-10 min-w-10 px-2.5
        assert_eq!(ToggleSize::Default.height(), 36.0);
        assert_eq!(ToggleSize::Default.min_width(), 36.0);
        assert_eq!(ToggleSize::Default.padding_x(), 8.0);
        assert_eq!(ToggleSize::Sm.height(), 32.0);
        assert_eq!(ToggleSize::Sm.min_width(), 32.0);
        assert_eq!(ToggleSize::Sm.padding_x(), 6.0);
        assert_eq!(ToggleSize::Lg.height(), 40.0);
        assert_eq!(ToggleSize::Lg.min_width(), 40.0);
        assert_eq!(ToggleSize::Lg.padding_x(), 10.0);
        // gap-2, icon size-4
        assert_eq!(ToggleSize::Default.gap(), 8.0);
        assert_eq!(ToggleSize::Default.icon_size(), 16.0);
    }

    #[test]
    fn colors_follow_state() {
        let p = palette(Mode::Light);
        // Off, resting: transparent, foreground.
        let c = resolve_toggle_colors(ToggleVariant::Default, &p, false, false, false, Style::Vega);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.fg, p.foreground);
        // Off, hovered: muted / muted-foreground.
        let c = resolve_toggle_colors(ToggleVariant::Default, &p, false, true, false, Style::Vega);
        assert_eq!(c.fill, p.muted);
        assert_eq!(c.fg, p.muted_foreground);
        // Pressed: accent, regardless of hover.
        let c = resolve_toggle_colors(ToggleVariant::Default, &p, false, true, true, Style::Vega);
        assert_eq!(c.fill, p.accent);
        assert_eq!(c.fg, p.accent_foreground);
        // Outline: input border, transparent fill.
        let c = resolve_toggle_colors(ToggleVariant::Outline, &p, false, false, false, Style::Vega);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.stroke.color, p.border);
        let dark = palette(Mode::Dark);
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &dark,
            true,
            false,
            false,
            Style::Vega,
        );
        assert_eq!(c.stroke.color, dark.input);
        // Pressed outline keeps the border.
        let c = resolve_toggle_colors(ToggleVariant::Outline, &p, false, false, true, Style::Vega);
        assert_eq!(c.fill, p.accent);
        assert_eq!(c.stroke.color, p.border);
    }

    #[test]
    fn nova_pressed_and_hover_follow_the_nova_recipe() {
        let p = palette(Mode::Light);
        // Pressed: `data-[state=on]:bg-muted`, text stays accent-foreground.
        let c = resolve_toggle_colors(ToggleVariant::Default, &p, false, true, true, Style::Nova);
        assert_eq!(c.fill, p.muted);
        assert_eq!(c.fg, p.accent_foreground);
        // Hover: bg stays muted, text moves to `hover:text-foreground`.
        let c = resolve_toggle_colors(ToggleVariant::Default, &p, false, true, false, Style::Nova);
        assert_eq!(c.fill, p.muted);
        assert_eq!(c.fg, p.foreground);
        // Base keeps muted / muted-foreground on hover.
        let c = resolve_toggle_colors(ToggleVariant::Default, &p, false, true, false, Style::Vega);
        assert_eq!(c.fg, p.muted_foreground);
    }

    #[test]
    fn nova_size_table_matches_recipe() {
        // nova default h-8 min-w-8 px-2.5, sm h-7 min-w-7 px-2.5, lg h-9 min-w-9 px-2.5
        assert_eq!(metrics::height(ToggleSize::Default, Style::Nova), 32.0);
        assert_eq!(metrics::height(ToggleSize::Sm, Style::Nova), 28.0);
        assert_eq!(metrics::height(ToggleSize::Lg, Style::Nova), 36.0);
        assert_eq!(metrics::padding_x(ToggleSize::Default, Style::Nova), 10.0);
        assert_eq!(metrics::padding_x(ToggleSize::Sm, Style::Nova), 10.0);
        assert_eq!(metrics::padding_x(ToggleSize::Lg, Style::Nova), 10.0);
        // nova gap-1, sm icon size-3.5, sm label text-[0.8rem]
        assert_eq!(metrics::gap(ToggleSize::Default, Style::Nova), 4.0);
        assert_eq!(metrics::icon_size(ToggleSize::Sm, Style::Nova), 14.0);
        assert_eq!(metrics::icon_size(ToggleSize::Default, Style::Nova), 16.0);
        assert_eq!(metrics::text_size(ToggleSize::Sm, Style::Nova), 12.8);
        assert_eq!(metrics::text_size(ToggleSize::Default, Style::Nova), 14.0);
        // base keeps the toggleVariants table
        assert_eq!(metrics::height(ToggleSize::Default, Style::Vega), 36.0);
        assert_eq!(metrics::height(ToggleSize::Sm, Style::Vega), 32.0);
        assert_eq!(metrics::height(ToggleSize::Lg, Style::Vega), 40.0);
        assert_eq!(metrics::padding_x(ToggleSize::Sm, Style::Vega), 6.0);
        assert_eq!(metrics::gap(ToggleSize::Default, Style::Vega), 8.0);
        assert_eq!(metrics::icon_size(ToggleSize::Sm, Style::Vega), 16.0);
        assert_eq!(metrics::text_size(ToggleSize::Sm, Style::Vega), 14.0);
    }

    #[test]
    fn nova_radius_is_lg_with_sm_cap() {
        let radius = crate::theme::RadiusTokens::new(10.0);
        assert_eq!(
            metrics::radius(&radius, ToggleSize::Default, Style::Nova),
            10
        ); // rounded-lg
           // sm: rounded-[min(var(--radius-md),12px)] = min(8, 12) = 8
        assert_eq!(metrics::radius(&radius, ToggleSize::Sm, Style::Nova), 8);
        assert_eq!(
            metrics::radius(&radius, ToggleSize::Default, Style::Vega),
            8
        );
        assert_eq!(metrics::radius(&radius, ToggleSize::Sm, Style::Vega), 8);
    }

    #[test]
    fn outline_shadow_drops_for_nova_and_fused_items() {
        use crate::components::button::GroupPosition;
        // Base standalone: shadow-xs.
        assert!(outline_has_shadow(Style::Vega, GroupPosition::Only));
        // Base inside a fused group: data-[spacing=0]:shadow-none.
        assert!(!outline_has_shadow(Style::Vega, GroupPosition::First));
        assert!(!outline_has_shadow(Style::Vega, GroupPosition::Middle));
        assert!(!outline_has_shadow(Style::Vega, GroupPosition::Last));
        // Nova never paints the outline shadow.
        assert!(!outline_has_shadow(Style::Nova, GroupPosition::Only));
    }

    #[test]
    fn invalid_uses_destructive_ring() {
        let p = palette(Mode::Light);
        let dark_p = palette(Mode::Dark);
        let mut c =
            resolve_toggle_colors(ToggleVariant::Default, &p, false, false, false, Style::Vega);
        assert_eq!(c.ring, color_alpha(p.ring, 0.5));
        c.ring = color_alpha(dark_p.destructive, 0.4);
        assert_eq!(c.ring, color_alpha(dark_p.destructive, 0.4));
    }

    #[test]
    fn toggles_on_click() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut pressed = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("toggle_click_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(200.0, 200.0),
                )),
            );
            let r = Toggle::new(&mut pressed).text("Bold").show(ui);
            assert!(!r.clicked()); // no pointer input
        });
        out.textures_delta.clear();
        assert!(!pressed);
    }

    #[test]
    fn builders_keep_fields() {
        let mut pressed = false;
        let t = Toggle::new(&mut pressed)
            .text("Bold")
            .icon(Icon::Sparkles)
            .variant(ToggleVariant::Outline)
            .size(ToggleSize::Sm)
            .padding_x(12.0)
            .disabled(true)
            .invalid(true)
            .id_salt("t");
        assert_eq!(t.text.as_deref(), Some("Bold"));
        assert!(t.icon.is_some());
        assert_eq!(t.variant, ToggleVariant::Outline);
        assert_eq!(t.size, ToggleSize::Sm);
        assert_eq!(t.padding_x_override, Some(12.0));
        assert!(t.disabled);
        assert!(t.invalid);
        assert_eq!(t.id_salt.as_deref(), Some("t"));
    }
}
