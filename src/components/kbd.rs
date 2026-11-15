//! shadcn `kbd.tsx` — a small keyboard-key chip.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/kbd.tsx` (28 lines). The
//! base recipe (`kbd.tsx:8-10`) is a single fixed class string — no variants,
//! no size prop, no Radix primitive:
//!
//! ```text
//! pointer-events-none inline-flex h-5 w-fit min-w-5 items-center justify-center
//! gap-1 rounded-sm bg-muted px-1 font-sans text-xs font-medium
//! text-muted-foreground select-none
//! [&_svg:not([class*='size-'])]:size-3
//! [[data-slot=tooltip-content]_&]:bg-background/20
//! [[data-slot=tooltip-content]_&]:text-background
//! dark:[[data-slot=tooltip-content]_&]:bg-background/10
//! ```
//!
//! The nova rule (`.cn-kbd`, `style-nova.css:780-782`) restates every visual
//! property with values identical to the base recipe and is silent on the
//! rest, so under the settled LAYER ruling the effective nova chip is
//! identical to the base chip on every slot (frozen spec §2.3). This module
//! therefore has no style branching at all: one geometry, one token set,
//! both recipes.
//!
//! Geometry (spec §3): fixed 20px height (`h-5`), shrink-to-fit width
//! (`w-fit`) with a 20px floor (`min-w-5`), 4px side padding (`px-1`), 4px
//! gaps (`gap-1`), 6px radius (`rounded-sm`, token-scaled), 12px
//! `font-medium` text, 12px default icon size (`size-3`). Width follows
//! spec §4: every icon counts its full width and every gap between adjacent
//! items counts; see [`chip_width`].
//!
//! Tooltip context (spec §2.4, SETTLED): inside tooltip content the chip
//! uses a translucent wash of the `background` token instead of `muted` —
//! `background/20` + `background` text in light mode, `background/10` +
//! `background` text in dark mode. The original triggers this by DOM
//! ancestry, which egui cannot express, so the Rust API exposes it as the
//! explicit [`Kbd::on_dark`] flag — proposed deviation D1 (spec §11),
//! pending Expert Reviewer approval. No other context rule exists: a chip
//! inside a menu, popover, or dialog uses the default values.
//!
//! No interactive states exist (spec §5): no hover, focus, pressed, or
//! disabled styling. The chip renders with a hover-only sense and takes no
//! tab stop (limitation D2, spec §11). The `<kbd>` element's keyboard-input
//! semantics have no egui runtime meaning, and no ARIA role is set by the
//! component in any case (limitation D3, spec §11). Nothing animates
//! (spec §7).
//!
//! OPEN items (spec §12) are left open: O1 (flag API shape — D1 decision),
//! O2 (concrete `font-sans` family; Inter is used per Phase 1, D4), O3
//! (`text-xs` paired line-height), O4 (tooltip/input-group interface
//! questions for later waves), O5 (`cn()` merge precedence — no className
//! analogue exists in this API).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Kbd::new("K").show(ui);
//! Kbd::new("M").icon(Icon::ChevronLeft).show(ui);
//! Kbd::new("⌘S").on_dark(true).show(ui); // inside tooltip content (D1)
//! KbdGroup::new().show(ui, |ui| {
//!     Kbd::new("⌘").show(ui);
//!     Kbd::new("⇧").show(ui);
//!     Kbd::new("P").show(ui);
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{Mode, Palette, ShadcnTheme};
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Layout, Pos2, Rect, Response, Sense, Ui,
    UiBuilder, Vec2, Widget,
};

/// Fixed chip height: `h-5` = 20px (spec §3.1).
const CHIP_HEIGHT: f32 = 20.0;
/// Minimum chip width: `min-w-5` = 20px (spec §3.1).
const MIN_WIDTH: f32 = 20.0;
/// Horizontal padding: `px-1` = 4px per side (spec §3.1).
const PAD_X: f32 = 4.0;
/// Gap between adjacent row items: `gap-1` = 4px (spec §3.1).
const GAP: f32 = 4.0;
/// Default icon size: `size-3` = 12px (spec §3.2).
const DEFAULT_ICON_SIZE: f32 = 12.0;

/// The chip background/text colors.
///
/// Default context: (`bg-muted`, `text-muted-foreground`). With `on_dark`
/// (the tooltip-context rule, spec §2.4): the `background` token at 20%
/// alpha (light) or 10% alpha (dark) for the fill, and the `background`
/// token at full alpha for the text. The text rule carries no `dark:`
/// qualifier in either recipe, so only the token value differs per theme.
pub(crate) fn colors(p: &Palette, mode: Mode, on_dark: bool) -> (Color32, Color32) {
    if on_dark {
        let alpha = match mode {
            Mode::Light => 0.20,
            Mode::Dark => 0.10,
        };
        (with_alpha(p.background, alpha), p.background)
    } else {
        (p.muted, p.muted_foreground)
    }
}

/// The same color with its alpha replaced by `alpha` (0.0–1.0), keeping the
/// unmultiplied RGB values. Used for the `/20` and `/10` alpha modifiers
/// of the tooltip-context rule; the translucent fill composites over the
/// tooltip surface when painted.
fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (alpha.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// The label font: 12px `font-medium` (`text-xs` + `font-medium`, spec
/// §3.1), i.e. Inter Medium (500) from the Phase 1 embedded fonts. The
/// concrete `font-sans` family is OPEN (spec §12 O2); the paired
/// line-height is OPEN (O3) and is the font's natural row height.
pub(crate) fn label_font() -> FontId {
    crate::theme::fonts::medium(crate::theme::tokens::type_scale::XS)
}

/// Width of the flex row's items plus the gaps between them (spec §4):
/// every icon counts its full width, the text run counts its measured
/// width (only when the label is non-empty — an empty label contributes no
/// item and no gap), and every gap between adjacent items counts.
pub(crate) fn content_width(text_width: f32, has_text: bool, icon_sizes: &[f32]) -> f32 {
    let mut items = icon_sizes.len();
    let mut width: f32 = icon_sizes.iter().sum();
    if has_text {
        items += 1;
        width += text_width;
    }
    if items > 1 {
        width += (items - 1) as f32 * GAP;
    }
    width
}

/// Chip width for a row content width: `px-1` padding, `min-w-5` floor
/// (spec §4). An empty chip still renders at 20px wide.
pub(crate) fn chip_width(content_width: f32) -> f32 {
    (content_width + 2.0 * PAD_X).max(MIN_WIDTH)
}

/// A single keyboard key chip. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Kbd {
    label: String,
    /// Icons in row order, each with an optional explicit size. `None`
    /// means the chip-level [`Self::icon_size`] default applies — the
    /// analogue of the CSS `:not([class*='size-'])` guard (spec §3.2).
    icons: Vec<(Icon, Option<f32>)>,
    /// Default icon size for icons without an explicit size (12px).
    icon_size: f32,
    /// Tooltip-context flag: applies the spec §2.4 values. Proposed
    /// deviation D1 (spec §11), pending Expert Reviewer approval.
    on_dark: bool,
    id_salt: Option<String>,
}

impl Kbd {
    /// Creates a key chip for the given key name.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icons: Vec::new(),
            icon_size: DEFAULT_ICON_SIZE,
            on_dark: false,
            id_salt: None,
        }
    }

    /// Appends a 12px icon before the label
    /// (`[&_svg:not([class*='size-'])]:size-3` slot, separated by `gap-1`).
    /// May be called more than once: every icon counts its full width and
    /// every gap counts (spec §4). The icon takes the chip-level
    /// [`Self::icon_size`] default unless [`Self::icon_sized`] is used.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icons.push((icon, None));
        self
    }

    /// Appends an icon with an explicit size. The explicit size always
    /// wins over the 12px default (spec §3.2: an svg carrying its own
    /// `size-*` class keeps its own size); the width arithmetic uses the
    /// icon's actual size.
    pub fn icon_sized(mut self, icon: Icon, size: f32) -> Self {
        self.icons.push((icon, Some(size)));
        self
    }

    /// Sets the default icon size for icons added without an explicit
    /// size. The default is 12px (`size-3`, spec §3.2).
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    /// Requests the tooltip-context values (spec §2.4): a translucent
    /// `background` wash instead of the `muted` chip, for use inside
    /// tooltip content. Default `false` (default chip values).
    ///
    /// PROPOSED DEVIATION D1 (spec §11): the original triggers these
    /// values by DOM ancestry, which egui cannot express; this explicit
    /// flag is the proposed faithful mapping, pending Expert Reviewer
    /// approval.
    pub fn on_dark(mut self, on_dark: bool) -> Self {
        self.on_dark = on_dark;
        self
    }

    /// Sets a stable id salt (Kbd keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Resolved icon sizes in row order: an icon's explicit size wins,
    /// otherwise the chip-level default (spec §3.2).
    fn resolved_icon_sizes(&self) -> Vec<f32> {
        self.icons
            .iter()
            .map(|(_, explicit)| explicit.unwrap_or(self.icon_size))
            .collect()
    }

    /// Renders the key chip.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let (bg, fg) = colors(&p, theme.mode, self.on_dark);

        let font = label_font();
        // Laid out once with the paint color: the same galley sizes the
        // chip and paints the label. No wrapping, no truncation (spec §10:
        // no max-width, no ellipsis, no `whitespace-nowrap` rule constrains
        // the chip; the width follows §4).
        let galley = ui.painter().layout_no_wrap(self.label.clone(), font, fg);

        let icon_sizes = self.resolved_icon_sizes();
        let has_text = !self.label.is_empty();
        let content_w = content_width(galley.size().x, has_text, &icon_sizes);
        let width = chip_width(content_w);
        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(width, CHIP_HEIGHT), Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(Vec2::new(width, CHIP_HEIGHT), Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // `rounded-sm` — token-scaled, not a fixed px literal (spec §0).
            painter.rect_filled(rect, CornerRadius::same(theme.radius.sm_u8()), bg);
            // Content: icons and label in row order, each gap-1 apart,
            // centred on both axes (`items-center justify-center`).
            let center = rect.center();
            let mut cursor = center.x - content_w * 0.5;
            for (i, (icon, _)) in self.icons.iter().enumerate() {
                let size = icon_sizes[i];
                let r = Rect::from_center_size(
                    Pos2::new(cursor + size * 0.5, center.y),
                    Vec2::splat(size),
                );
                // Lucide stroke-width 2 at the 24px viewBox, scaled.
                icon.paint(painter, r, fg, 2.0 * size / 24.0);
                cursor += size;
                if i + 1 < self.icons.len() || has_text {
                    cursor += GAP;
                }
            }
            if has_text {
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = Align2::CENTER_CENTER.anchor_size(
                    Pos2::new(cursor + galley.size().x * 0.5, center.y),
                    galley.size(),
                );
                painter.galley(text_rect.min, galley, fg);
            }
        }

        response
    }
}

impl Widget for Kbd {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Lays out several [`Kbd`] chips with `gap-1` spacing, like the source
/// `KbdGroup` (`inline-flex items-center gap-1`, `kbd.tsx:22`). The group
/// declares no background, padding, radius, or border: it is a transparent
/// spacing container whose height is its tallest child (spec §3.3). The
/// original's div-props/`kbd`-element mismatch (spec §11 D5) is not
/// reproduced; it has no visual consequence.
///
/// The group measures its content before allocating (see
/// [`KbdGroup::show`]), so every child centres within the tallest-child
/// row (`items-center`), whatever order the children arrive in —
/// including mixed-height children.
#[derive(Clone, Copy, Debug, Default)]
pub struct KbdGroup;

impl KbdGroup {
    /// Creates an empty group.
    pub fn new() -> Self {
        Self
    }

    /// Renders the group around its children.
    ///
    /// The content closure runs TWICE: once invisibly to measure the
    /// tallest child, once for real. The measuring pass is invisible
    /// (hence disabled), so no callbacks can double-fire; `Kbd` children
    /// are stateless paints and are unaffected. Callers must not rely on
    /// the closure running exactly once.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        mut content: impl FnMut(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        // Measure-then-real (the `dialog_footer` pattern): the group must
        // PRESENT its true content-sized box to its parent, so a centring
        // parent centres the true box. Seeding the row with zero height
        // (the cycle-1 KBD-VIS-01 fix) is centred as a 0-height rect by
        // `justify_and_align` inside a centred parent, and egui's
        // expand-down rule then grows the row downward from the middle of
        // the parent frame — the group rendered 10px low inside a centred
        // `InputGroupAddon` (defect KBD-VIS-02). Measuring first gives the
        // real pass the tallest-child height upfront, so the parent
        // centres the true 20px box and every child centres within the
        // row. Do NOT use `ui.horizontal` here either: it seeds the row
        // height to `spacing().interact_size.y` (36px under the crate
        // theme), so the group would allocate 36px whatever its children
        // are (defect KBD-VIS-01). Direction handling mirrors
        // `Ui::horizontal`.
        let rtl = ui.layout().prefer_right_to_left();
        let flow_min = ui.cursor().min;
        let flow_w = ui.available_size_before_wrap().x;

        // Measure pass: invisible (hence disabled) with a Min cross-align,
        // so the measured height is the tallest child. A Center
        // cross-align would make each child's frame span the full
        // (infinite) available height, so the measured height would be
        // infinite. The layout must be set on the `UiBuilder`: the child
        // inherits the parent's layout at creation, and a centred
        // horizontal parent would make the region seed computation centre
        // an infinite height (NaN).
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
        let measure_layout = if rtl {
            Layout::right_to_left(Align::Min)
        } else {
            Layout::left_to_right(Align::Min)
        };
        let mut measure = ui.new_child(
            UiBuilder::new()
                .max_rect(measure_rect)
                .layout(measure_layout)
                .invisible(),
        );
        // `gap-1`, as in the real pass.
        measure.spacing_mut().item_spacing.x = GAP;
        content(&mut measure);
        let content_h = measure.min_rect().height().max(0.0);

        // Real pass: the true height, so `justify_and_align` centres the
        // true box in a centring parent, and `Align::Center` centres every
        // child within the row.
        let layout = if rtl {
            Layout::right_to_left(Align::Center)
        } else {
            Layout::left_to_right(Align::Center)
        };
        let initial_size = Vec2::new(flow_w, content_h);
        ui.allocate_ui_with_layout(initial_size, layout, |ui| {
            // `gap-1`
            ui.spacing_mut().item_spacing.x = GAP;
            content(ui)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset, Style};

    #[test]
    fn chip_uses_muted_tokens() {
        let p = Preset::default().build(Mode::Light).palette;
        let (bg, fg) = colors(&p, Mode::Light, false);
        assert_eq!(bg, p.muted);
        assert_eq!(fg, p.muted_foreground);
    }

    #[test]
    fn chip_uses_muted_tokens_for_every_style_and_mode() {
        // No style branching: both recipes render identically (spec §2.3).
        for style in [
            Style::Nova,
            Style::Vega,
            Style::Maia,
            Style::Lyra,
            Style::Mira,
            Style::Luma,
            Style::Sera,
            Style::Rhea,
        ] {
            for mode in [Mode::Light, Mode::Dark] {
                let preset = Preset {
                    style,
                    ..Preset::default()
                };
                let p = preset.build(mode).palette;
                let (bg, fg) = colors(&p, mode, false);
                assert_eq!(bg, p.muted, "style {style:?} mode {mode:?}");
                assert_eq!(fg, p.muted_foreground, "style {style:?} mode {mode:?}");
            }
        }
    }

    #[test]
    fn tooltip_context_colors_match_spec() {
        // Spec §2.4: light `background/20` + `background` text, dark
        // `background/10` + `background` text. The wash keeps the token's
        // RGB and carries only the stated alpha (no pre-compositing).
        for mode in [Mode::Light, Mode::Dark] {
            let p = Preset::default().build(mode).palette;
            let (bg, fg) = colors(&p, mode, true);
            assert_eq!(fg, p.background, "mode {mode:?}");
            let [r, g, b, a] = bg.to_srgba_unmultiplied();
            let [er, eg, eb, _] = p.background.to_srgba_unmultiplied();
            assert_eq!((r, g, b), (er, eg, eb), "mode {mode:?}");
            let expected_alpha = match mode {
                Mode::Light => 51, // 20% of 255
                Mode::Dark => 26,  // 10% of 255
            };
            assert_eq!(a, expected_alpha, "mode {mode:?}");
        }
    }

    #[test]
    fn tooltip_context_colors_for_every_style_and_mode() {
        for style in [
            Style::Nova,
            Style::Vega,
            Style::Maia,
            Style::Lyra,
            Style::Mira,
            Style::Luma,
            Style::Sera,
            Style::Rhea,
        ] {
            for mode in [Mode::Light, Mode::Dark] {
                let preset = Preset {
                    style,
                    ..Preset::default()
                };
                let p = preset.build(mode).palette;
                let (bg, fg) = colors(&p, mode, true);
                assert_eq!(fg, p.background, "style {style:?} mode {mode:?}");
                let [_, _, _, a] = bg.to_srgba_unmultiplied();
                let expected = match mode {
                    Mode::Light => 51,
                    Mode::Dark => 26,
                };
                assert_eq!(a, expected, "style {style:?} mode {mode:?}");
            }
        }
    }

    #[test]
    fn label_font_is_12px_medium() {
        // Spec §3.1: `text-xs` (12px) + `font-medium` (500) = Inter Medium.
        let font = label_font();
        assert_eq!(font.size, 12.0);
        assert_eq!(font.family, crate::theme::fonts::medium_family());
    }

    #[test]
    fn radius_sm_is_6px_at_default_radius() {
        // Spec §0/§3.1: `rounded-sm` = 6px at the snapshot `--radius`.
        let theme = Preset::default().build(Mode::Light);
        assert_eq!(theme.radius.sm(), 6.0);
        assert_eq!(theme.radius.sm_u8(), 6);
    }

    #[test]
    fn content_width_matches_spec_table() {
        // Spec §4 worked arithmetic (icon 12px, gap 4px, padding 4px/side).
        // Text only: max(20, text + 8).
        assert_eq!(chip_width(content_width(10.0, true, &[])), 20.0);
        assert_eq!(chip_width(content_width(30.0, true, &[])), 38.0);
        // 1 icon, no text: 4 + 12 + 4 = 20 (exactly the floor).
        assert_eq!(chip_width(content_width(0.0, false, &[12.0])), 20.0);
        // 1 icon + text: max(20, text + 24).
        assert_eq!(chip_width(content_width(30.0, true, &[12.0])), 54.0);
        // 2 icons + text: max(20, text + 40).
        assert_eq!(chip_width(content_width(30.0, true, &[12.0, 12.0])), 70.0);
        // Empty (no children): 20 (the floor).
        assert_eq!(chip_width(content_width(0.0, false, &[])), 20.0);
        // Explicit icon sizes use their actual size (spec §3.2).
        assert_eq!(chip_width(content_width(0.0, false, &[16.0])), 24.0);
        assert_eq!(chip_width(content_width(30.0, true, &[12.0, 16.0])), 74.0);
    }

    #[test]
    fn chip_width_respects_min_w_5() {
        assert_eq!(chip_width(0.0), 20.0);
        assert_eq!(chip_width(10.0), 20.0);
        assert_eq!(chip_width(30.0), 38.0);
        assert_eq!(CHIP_HEIGHT, 20.0);
        assert_eq!(MIN_WIDTH, 20.0);
    }

    #[test]
    fn kbd_builder_fields() {
        let k = Kbd::new("⌘")
            .icon(Icon::ChevronRight)
            .id_salt("meta")
            .on_dark(true);
        assert_eq!(k.label, "⌘");
        assert_eq!(k.icons.len(), 1);
        assert_eq!(k.id_salt.as_deref(), Some("meta"));
        assert!(k.on_dark);
        // Defaults: no icons, 12px icon size, default context.
        let k = Kbd::new("K");
        assert!(k.icons.is_empty());
        assert_eq!(k.icon_size, 12.0);
        assert!(!k.on_dark);
    }

    #[test]
    fn multiple_icons_accumulate() {
        let k = Kbd::new("M")
            .icon(Icon::ChevronLeft)
            .icon(Icon::ChevronRight);
        assert_eq!(k.icons.len(), 2);
        assert_eq!(k.resolved_icon_sizes(), vec![12.0, 12.0]);
    }

    #[test]
    fn explicit_icon_size_wins_over_default() {
        // Spec §3.2: an explicit size keeps its own size; the chip-level
        // default applies only to icons without one.
        let k = Kbd::new("M")
            .icon(Icon::ChevronLeft)
            .icon_sized(Icon::ChevronRight, 20.0)
            .icon_size(16.0);
        assert_eq!(k.resolved_icon_sizes(), vec![16.0, 20.0]);
        let k = Kbd::new("M").icon_size(16.0).icon(Icon::ChevronLeft);
        assert_eq!(k.resolved_icon_sizes(), vec![16.0]);
    }

    #[test]
    fn group_is_a_unit_struct() {
        let _ = KbdGroup::new();
    }

    #[test]
    fn group_height_is_tallest_child_not_interact_size() {
        // Spec §3.3 + defect KBD-VIS-01: the group allocates its tallest
        // child's height (20px), not `spacing().interact_size.y`. The
        // installed preset is essential to the test: it applies the
        // gallery theme spacing (`interact_size.y` = 36px), so the old
        // `ui.horizontal` implementation fails this test with 36px while
        // the fix returns 20px. Default egui spacing would prove less
        // (the typography S1 lesson: a test passing only because the test
        // context uses default spacing proves nothing).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("kbd_group_height_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            );
            assert_eq!(
                ui.spacing().interact_size.y,
                crate::theme::tokens::spacing::CONTROL,
                "the gallery theme spacing must be in effect for this test to discriminate"
            );
            let group = KbdGroup::new().show(ui, |ui| {
                let first = Kbd::new("⌘").show(ui).rect;
                let second = Kbd::new("K").show(ui).rect;
                (first, second)
            });
            let (first, second) = group.inner;
            assert_eq!(first.height(), CHIP_HEIGHT);
            assert_eq!(second.height(), CHIP_HEIGHT);
            assert_eq!(group.response.rect.height(), CHIP_HEIGHT);
            assert!((second.min.x - first.max.x - GAP).abs() < 0.001);
            assert!(
                (group.response.rect.width() - (first.width() + GAP + second.width())).abs()
                    < 0.001
            );
            // Chips are vertically centred in the group: same top edge.
            assert_eq!(group.response.rect.min.y, first.min.y);
            assert_eq!(group.response.rect.min.y, second.min.y);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn group_centres_in_a_centred_parent_row() {
        // Defect KBD-VIS-02: inside a centred parent row (the
        // `InputGroupAddon` case — `ui.horizontal` with a 36px min
        // height), the group must present its true 20px box so the
        // parent centres it: group centre == row centre, i.e. 8px from
        // the row top. The zero-height seed rendered the group 10px low
        // (row_top + 18, grown downward) and stretched the row to 38px,
        // so this test fails on the unfixed code on both counts. The
        // installed preset applies the gallery theme spacing, as in the
        // KBD-VIS-01 test above.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("kbd_group_centred_parent_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            );
            let row = ui.horizontal(|ui| {
                ui.set_min_size(Vec2::new(0.0, crate::theme::tokens::spacing::CONTROL));
                let group = KbdGroup::new().show(ui, |ui| {
                    let first = Kbd::new("⌘").show(ui).rect;
                    let second = Kbd::new("K").show(ui).rect;
                    (first, second)
                });
                (group.response.rect, group.inner)
            });
            let (group_rect, (first, second)) = row.inner;
            let row_rect = row.response.rect;
            assert_eq!(
                row_rect.height(),
                crate::theme::tokens::spacing::CONTROL,
                "the row stays 36px tall; the group must not stretch it"
            );
            assert_eq!(group_rect.height(), CHIP_HEIGHT);
            assert!(
                (group_rect.center().y - row_rect.center().y).abs() < 0.001,
                "group centre {:?} != row centre {:?}",
                group_rect.center(),
                row_rect.center()
            );
            assert!((group_rect.min.y - row_rect.min.y - 8.0).abs() < 0.001);
            assert_eq!(group_rect.min.y, first.min.y);
            assert_eq!(group_rect.min.y, second.min.y);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn group_centres_mixed_height_children() {
        // The measure-then-real allocation centres every child in the
        // tallest-child row, whatever order children arrive in. The
        // zero-height seed left an earlier shorter child at the top
        // when a later child was taller (the S9 limitation, now
        // superseded): the short-first order below fails on the
        // unfixed code, where the 12px child sits at the row top
        // instead of centred 4px down.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("kbd_group_mixed_height_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            );
            let group = KbdGroup::new().show(ui, |ui| {
                let (short, _) = ui.allocate_exact_size(Vec2::new(10.0, 12.0), Sense::hover());
                let tall = Kbd::new("K").show(ui).rect;
                (short, tall)
            });
            let (short, tall) = group.inner;
            assert_eq!(group.response.rect.height(), CHIP_HEIGHT);
            assert!((short.min.y - group.response.rect.min.y - 4.0).abs() < 0.001);
            assert_eq!(tall.min.y, group.response.rect.min.y);
        });
        out.textures_delta.clear();
    }
}
