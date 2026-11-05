//! The shadcn theme: light/dark OKLCH palettes resolved to `egui::Color32`,
//! radius tokens, the active style, and the installed [`Preset`].
//!
//! Install once at startup with [`Preset::install`], then read anywhere with
//! [`ShadcnTheme::of`].

pub mod colors;
pub mod presets;
pub mod theme_data;
pub mod tokens;

pub use colors::{Oklch, SLOT_NAMES};
pub use presets::{
    find_theme, BaseColor, MenuAccent, MenuColor, Preset, Radius, Style, ThemeColor,
};
pub use tokens::{spacing, type_scale, RadiusTokens};

use crate::theme::colors::{slot, ColorSpec};
use egui::Color32;

/// Light or dark palette mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Light,
    Dark,
}

impl Mode {
    /// The other mode.
    pub fn toggle(self) -> Self {
        match self {
            Mode::Light => Mode::Dark,
            Mode::Dark => Mode::Light,
        }
    }

    /// True if this is the dark palette.
    pub fn is_dark(self) -> bool {
        self == Mode::Dark
    }
}

/// All named tokens resolved to `egui::Color32`.
///
/// Fields mirror the CSS custom properties 1:1 (`background`, `foreground`,
/// `primary`, `border`, ...). Use [`color_alpha`] for the `/50`-style opacity
/// variants the source CSS uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub background: Color32,
    pub foreground: Color32,
    pub card: Color32,
    pub card_foreground: Color32,
    pub popover: Color32,
    pub popover_foreground: Color32,
    pub primary: Color32,
    pub primary_foreground: Color32,
    pub secondary: Color32,
    pub secondary_foreground: Color32,
    pub muted: Color32,
    pub muted_foreground: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub destructive: Color32,
    pub destructive_foreground: Color32,
    pub border: Color32,
    pub input: Color32,
    pub ring: Color32,
    pub chart_1: Color32,
    pub chart_2: Color32,
    pub chart_3: Color32,
    pub chart_4: Color32,
    pub chart_5: Color32,
    pub sidebar: Color32,
    pub sidebar_foreground: Color32,
    pub sidebar_primary: Color32,
    pub sidebar_primary_foreground: Color32,
    pub sidebar_accent: Color32,
    pub sidebar_accent_foreground: Color32,
    pub sidebar_border: Color32,
    pub sidebar_ring: Color32,
    pub surface: Color32,
    pub surface_foreground: Color32,
    pub code: Color32,
    pub code_foreground: Color32,
    pub code_highlight: Color32,
    pub code_number: Color32,
    pub selection: Color32,
    pub selection_foreground: Color32,
}

impl Palette {
    fn resolve(spec: &ColorSpec) -> Self {
        use slot::*;
        // Ordered fallback: start from black; every real slot overrides it.
        let f = Oklch::new(0.0, 0.0, 0.0);
        let white = Oklch::new(1.0, 0.0, 0.0);
        Self {
            background: spec.color(BACKGROUND, f),
            foreground: spec.color(FOREGROUND, white),
            card: spec.color(CARD, f),
            card_foreground: spec.color(CARD_FOREGROUND, white),
            popover: spec.color(POPOVER, f),
            popover_foreground: spec.color(POPOVER_FOREGROUND, white),
            primary: spec.color(PRIMARY, f),
            primary_foreground: spec.color(PRIMARY_FOREGROUND, white),
            secondary: spec.color(SECONDARY, f),
            secondary_foreground: spec.color(SECONDARY_FOREGROUND, white),
            muted: spec.color(MUTED, f),
            muted_foreground: spec.color(MUTED_FOREGROUND, white),
            accent: spec.color(ACCENT, f),
            accent_foreground: spec.color(ACCENT_FOREGROUND, white),
            destructive: spec.color(DESTRUCTIVE, Oklch::new(0.577, 0.245, 27.325)),
            destructive_foreground: spec
                .color(DESTRUCTIVE_FOREGROUND, Oklch::new(0.985, 0.01, 17.0)),
            border: spec.color(BORDER, f),
            input: spec.color(INPUT, f),
            ring: spec.color(RING, f),
            chart_1: spec.color(CHART_1, f),
            chart_2: spec.color(CHART_2, f),
            chart_3: spec.color(CHART_3, f),
            chart_4: spec.color(CHART_4, f),
            chart_5: spec.color(CHART_5, f),
            sidebar: spec.color(SIDEBAR, f),
            sidebar_foreground: spec.color(SIDEBAR_FOREGROUND, f),
            sidebar_primary: spec.color(SIDEBAR_PRIMARY, f),
            sidebar_primary_foreground: spec.color(SIDEBAR_PRIMARY_FOREGROUND, f),
            sidebar_accent: spec.color(SIDEBAR_ACCENT, f),
            sidebar_accent_foreground: spec.color(SIDEBAR_ACCENT_FOREGROUND, f),
            sidebar_border: spec.color(SIDEBAR_BORDER, f),
            sidebar_ring: spec.color(SIDEBAR_RING, f),
            surface: spec.color(SURFACE, f),
            surface_foreground: spec.color(SURFACE_FOREGROUND, f),
            code: spec.color(CODE, f),
            code_foreground: spec.color(CODE_FOREGROUND, f),
            code_highlight: spec.color(CODE_HIGHLIGHT, f),
            code_number: spec.color(CODE_NUMBER, f),
            selection: spec.color(SELECTION, f),
            selection_foreground: spec.color(SELECTION_FOREGROUND, f),
        }
    }
}

/// Returns `color` composited at `alpha` (1.0 = unchanged), like Tailwind's
/// `bg-primary/50`. Handles already-translucent inputs by scaling.
pub fn color_alpha(color: Color32, alpha: f32) -> Color32 {
    let a = alpha.clamp(0.0, 1.0);
    // egui colors are premultiplied: scaling alpha by k scales the RGB
    // channels by the same k, whatever the old alpha was.
    Color32::from_rgba_premultiplied(
        ((color.r() as f32 * a) + 0.5) as u8,
        ((color.g() as f32 * a) + 0.5) as u8,
        ((color.b() as f32 * a) + 0.5) as u8,
        ((color.a() as f32 * a) + 0.5) as u8,
    )
}

/// Blends `overlay` (with its alpha) over `base` and returns the opaque
/// result. Useful for resolving Tailwind classes like `bg-destructive/60`
/// against a known backdrop.
pub fn blend(base: Color32, overlay: Color32) -> Color32 {
    let (br, bg, bb) = (base.r(), base.g(), base.b());
    // egui stores premultiplied alpha; un-premultiply the overlay first.
    let oa = overlay.a() as f32 / 255.0;
    let un_premul = |c: u8| -> f32 {
        if oa <= 0.0 {
            0.0
        } else {
            (c as f32 / 255.0 / oa).clamp(0.0, 1.0)
        }
    };
    let (or_, og, ob) = (
        un_premul(overlay.r()),
        un_premul(overlay.g()),
        un_premul(overlay.b()),
    );
    let mix = |b: u8, o: f32| -> u8 {
        (((b as f32 / 255.0) * (1.0 - oa) + o * oa).clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    };
    Color32::from_rgb(mix(br, or_), mix(bg, og), mix(bb, ob))
}

/// The fully resolved shadcn theme: palette + tokens + style + preset.
///
/// Stored in the [`egui::Context`] memory by [`Preset::install`]; read with
/// [`ShadcnTheme::of`].
#[derive(Clone, Debug)]
pub struct ShadcnTheme {
    /// Active palette mode.
    pub mode: Mode,
    /// Resolved colors.
    pub palette: Palette,
    /// Radius scale derived from the theme `--radius`.
    pub radius: RadiusTokens,
    /// Active component style.
    pub style: Style,
    /// The preset that produced this theme (used to rebuild on mode switch).
    pub preset: Preset,
}

impl ShadcnTheme {
    /// Builds a theme from a resolved [`ColorSpec`].
    pub fn new(
        mode: Mode,
        spec: &ColorSpec,
        radius: RadiusTokens,
        style: Style,
        preset: Preset,
    ) -> Self {
        Self {
            mode,
            palette: Palette::resolve(spec),
            radius,
            style,
            preset,
        }
    }

    /// The egui memory id this theme is stored under.
    pub(crate) fn id() -> egui::Id {
        egui::Id::unique("egui_shadcn_theme_v1")
    }

    /// The currently installed mode, if any.
    pub fn current_mode(ctx: &egui::Context) -> Option<Mode> {
        ctx.data_mut(|data| data.get_temp::<Self>(Self::id()))
            .map(|t| t.mode)
    }

    /// Reads the theme from the context, installing the default preset if
    /// none is present yet.
    pub fn of(ctx: &egui::Context) -> Self {
        let id = Self::id();
        ctx.data_mut(|data| {
            if let Some(theme) = data.get_temp::<Self>(id) {
                theme
            } else {
                let theme = Preset::default().build(Mode::Light);
                data.insert_temp(id, theme.clone());
                theme
            }
        })
    }

    /// Convenience accessor for [`egui::Ui`].
    pub fn of_ui(ui: &egui::Ui) -> Self {
        Self::of(ui.ctx())
    }

    /// Installs this theme into the context and applies it to egui visuals.
    pub fn install(&self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(Self::id(), self.clone()));
        self.apply_visuals(ctx);
    }

    /// Switches light/dark, rebuilding from the stored preset.
    pub fn set_mode(ctx: &egui::Context, mode: Mode) {
        let theme = Self::of(ctx);
        let rebuilt = theme.preset.build(mode);
        rebuilt.install(ctx);
    }

    /// Applies the palette to egui's global [`egui::Visuals`] so plain egui
    /// widgets (TextEdit, ScrollArea, Window, tooltips) match the theme.
    pub fn apply_visuals(&self, ctx: &egui::Context) {
        let p = &self.palette;
        let dark = self.mode.is_dark();
        let r = self.radius;

        let mut visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.dark_mode = dark;
        visuals.override_text_color = Some(p.foreground);
        visuals.weak_text_color = Some(p.muted_foreground); // placeholder/hint text
        visuals.panel_fill = p.background;
        visuals.window_fill = p.popover;
        visuals.extreme_bg_color = p.muted;
        visuals.faint_bg_color = p.muted;
        visuals.selection.stroke = egui::Stroke::new(1.0, p.ring);
        visuals.selection.bg_fill = color_alpha(p.primary, 0.35);

        visuals.widgets.noninteractive.bg_fill = p.background;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, p.foreground);
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, p.border);
        visuals.widgets.noninteractive.weak_bg_fill = p.background;

        visuals.widgets.inactive.bg_fill = p.secondary;
        visuals.widgets.inactive.weak_bg_fill = p.secondary;
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, p.foreground);
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, p.input);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(r.md_u8());

        visuals.widgets.hovered.bg_fill = p.accent;
        visuals.widgets.hovered.weak_bg_fill = p.accent;
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, p.accent_foreground);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, p.input);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(r.md_u8());

        visuals.widgets.active.bg_fill = p.accent;
        visuals.widgets.active.weak_bg_fill = p.accent;
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, p.accent_foreground);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(r.md_u8());

        visuals.widgets.open.bg_fill = p.accent;
        visuals.widgets.open.weak_bg_fill = p.accent;
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, p.accent_foreground);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(r.md_u8());

        visuals.window_corner_radius = egui::CornerRadius::same(r.xl_u8());
        visuals.window_stroke = egui::Stroke::new(1.0, p.border);
        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: shadow_color(dark),
        };
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: shadow_color(dark),
        };
        visuals.menu_corner_radius = egui::CornerRadius::same(r.md_u8());
        visuals.text_edit_bg_color = Some(p.input);
        visuals.code_bg_color = p.muted;
        visuals.text_cursor.stroke = egui::Stroke::new(2.0, p.foreground);
        visuals.hyperlink_color = p.primary;

        ctx.all_styles_mut(|style| {
            let sp = &mut style.spacing;
            sp.item_spacing = egui::vec2(8.0, 8.0);
            sp.button_padding = egui::vec2(16.0, 7.0); // px-4 + h-9
            sp.menu_margin = egui::Margin::symmetric(4, 4);
            sp.menu_spacing = 1.0;
            sp.icon_width_inner = 16.0;
            sp.icon_width = 16.0 + sp.button_padding.x;
            sp.interact_size = egui::vec2(40.0, spacing::CONTROL);
        });

        ctx.set_visuals(visuals);
    }
}

fn shadow_color(dark: bool) -> Color32 {
    if dark {
        Color32::from_black_alpha(96)
    } else {
        Color32::from_black_alpha(40)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_of_opaque_scales_channels() {
        let c = Color32::from_rgb(200, 100, 50);
        let a = color_alpha(c, 0.5);
        assert_eq!(a.a(), 128);
        assert_eq!(a.r(), (200.0 * 0.5 + 0.5) as u8);
    }

    #[test]
    fn alpha_of_translucent_scales_again() {
        let c = Color32::from_rgba_premultiplied(26, 26, 26, 26); // white 10%
        let a = color_alpha(c, 0.5);
        assert_eq!(a.a(), 13); // 5%
        assert_eq!(a.r(), 13);
    }

    #[test]
    fn blend_white_over_black() {
        let overlay = Color32::from_rgba_premultiplied(127, 127, 127, 127); // 50% white
        let out = blend(Color32::BLACK, overlay);
        assert_eq!(out, Color32::from_rgb(127, 127, 127));
    }

    #[test]
    fn default_theme_light_background_is_white() {
        let theme = Preset::default().build(Mode::Light);
        assert_eq!(theme.palette.background, Color32::WHITE);
        assert_eq!(
            theme.palette.foreground,
            Oklch::parse("oklch(0.145 0 0)").unwrap().to_color32()
        );
    }

    #[test]
    fn dark_background_is_dark_gray() {
        let theme = Preset::default().build(Mode::Dark);
        assert!(theme.palette.background.r() < 40);
    }

    #[test]
    fn preset_rebuild_carries_preset_for_mode_switch() {
        let ctx = egui::Context::default();
        let preset = Preset::default().with_theme_color(ThemeColor::Blue);
        preset.install(&ctx);
        let theme = ShadcnTheme::of(&ctx);
        assert_eq!(theme.preset, preset);
        ShadcnTheme::set_mode(&ctx, Mode::Dark);
        let theme = ShadcnTheme::of(&ctx);
        assert_eq!(theme.mode, Mode::Dark);
        assert_eq!(theme.preset, preset);
        // Blue dark primary survives the mode switch.
        let blue_dark = Oklch::parse("oklch(0.424 0.199 265.638)")
            .unwrap()
            .to_color32();
        assert_eq!(theme.palette.primary, blue_dark);
    }

    #[test]
    fn theme_of_installs_default_when_missing() {
        let ctx = egui::Context::default();
        let theme = ShadcnTheme::of(&ctx);
        assert_eq!(theme.mode, Mode::Light);
        assert_eq!(theme.style, Style::Nova);
    }
}
