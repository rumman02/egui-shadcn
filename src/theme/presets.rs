//! The preset system, mirroring shadcn's `/create` page: style, base color,
//! theme color, chart color, font, icon library, radius, menu color and menu
//! accent — plus the bit-packed base62 preset codes (`"b0"` = defaults).

use crate::theme::colors::{ColorSpec, Slots, MODE_DEFAULTS_DARK, MODE_DEFAULTS_LIGHT};
use crate::theme::theme_data::{ThemeSpec, THEMES};
use crate::theme::tokens::RadiusTokens;

/// Component style. Each style is a full decoration set in the source
/// (`registry/styles/style-<name>.css`); components read this via
/// [`crate::theme::ShadcnTheme::style`] to adapt borders, shadows and radii.
///
/// Order matches the source `PRESET_STYLES` array (never reorder).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Style {
    #[default]
    Nova,
    Vega,
    Maia,
    Lyra,
    Mira,
    Luma,
    Sera,
    Rhea,
}

impl Style {
    /// Source registry name, e.g. `"nova"`.
    pub fn name(&self) -> &'static str {
        match self {
            Style::Nova => "nova",
            Style::Vega => "vega",
            Style::Maia => "maia",
            Style::Lyra => "lyra",
            Style::Mira => "mira",
            Style::Luma => "luma",
            Style::Sera => "sera",
            Style::Rhea => "rhea",
        }
    }

    fn from_index(idx: usize) -> Option<Self> {
        const ALL: [Style; 8] = [
            Style::Nova,
            Style::Vega,
            Style::Maia,
            Style::Lyra,
            Style::Mira,
            Style::Luma,
            Style::Sera,
            Style::Rhea,
        ];
        ALL.get(idx).copied()
    }

    fn index(&self) -> usize {
        match self {
            Style::Nova => 0,
            Style::Vega => 1,
            Style::Maia => 2,
            Style::Lyra => 3,
            Style::Mira => 4,
            Style::Luma => 5,
            Style::Sera => 6,
            Style::Rhea => 7,
        }
    }
}

/// The 8 base colors with full neutral palettes (source `PRESET_BASE_COLORS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BaseColor {
    #[default]
    Neutral,
    Stone,
    Zinc,
    Gray,
    Mauve,
    Olive,
    Mist,
    Taupe,
}

impl BaseColor {
    /// Source registry name.
    pub fn name(&self) -> &'static str {
        match self {
            BaseColor::Neutral => "neutral",
            BaseColor::Stone => "stone",
            BaseColor::Zinc => "zinc",
            BaseColor::Gray => "gray",
            BaseColor::Mauve => "mauve",
            BaseColor::Olive => "olive",
            BaseColor::Mist => "mist",
            BaseColor::Taupe => "taupe",
        }
    }

    fn from_index(idx: usize) -> Option<Self> {
        const ALL: [BaseColor; 8] = [
            BaseColor::Neutral,
            BaseColor::Stone,
            BaseColor::Zinc,
            BaseColor::Gray,
            BaseColor::Mauve,
            BaseColor::Olive,
            BaseColor::Mist,
            BaseColor::Taupe,
        ];
        ALL.get(idx).copied()
    }

    fn index(&self) -> usize {
        match self {
            BaseColor::Neutral => 0,
            BaseColor::Stone => 1,
            BaseColor::Zinc => 2,
            BaseColor::Gray => 3,
            BaseColor::Mauve => 4,
            BaseColor::Olive => 5,
            BaseColor::Mist => 6,
            BaseColor::Taupe => 7,
        }
    }
}

/// Accent colors that override `primary`, `secondary`, `chart-1..5` and
/// `sidebar-primary` over the base color (source `PRESET_THEMES`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ThemeColor {
    #[default]
    Neutral,
    Stone,
    Zinc,
    Gray,
    Amber,
    Blue,
    Cyan,
    Emerald,
    Fuchsia,
    Green,
    Indigo,
    Lime,
    Orange,
    Pink,
    Purple,
    Red,
    Rose,
    Sky,
    Teal,
    Violet,
    Yellow,
    Mauve,
    Olive,
    Mist,
    Taupe,
}

impl ThemeColor {
    /// Source registry name.
    pub fn name(&self) -> &'static str {
        match self {
            ThemeColor::Neutral => "neutral",
            ThemeColor::Stone => "stone",
            ThemeColor::Zinc => "zinc",
            ThemeColor::Gray => "gray",
            ThemeColor::Amber => "amber",
            ThemeColor::Blue => "blue",
            ThemeColor::Cyan => "cyan",
            ThemeColor::Emerald => "emerald",
            ThemeColor::Fuchsia => "fuchsia",
            ThemeColor::Green => "green",
            ThemeColor::Indigo => "indigo",
            ThemeColor::Lime => "lime",
            ThemeColor::Orange => "orange",
            ThemeColor::Pink => "pink",
            ThemeColor::Purple => "purple",
            ThemeColor::Red => "red",
            ThemeColor::Rose => "rose",
            ThemeColor::Sky => "sky",
            ThemeColor::Teal => "teal",
            ThemeColor::Violet => "violet",
            ThemeColor::Yellow => "yellow",
            ThemeColor::Mauve => "mauve",
            ThemeColor::Olive => "olive",
            ThemeColor::Mist => "mist",
            ThemeColor::Taupe => "taupe",
        }
    }

    fn from_index(idx: usize) -> Option<Self> {
        const ALL: [ThemeColor; 25] = [
            ThemeColor::Neutral,
            ThemeColor::Stone,
            ThemeColor::Zinc,
            ThemeColor::Gray,
            ThemeColor::Amber,
            ThemeColor::Blue,
            ThemeColor::Cyan,
            ThemeColor::Emerald,
            ThemeColor::Fuchsia,
            ThemeColor::Green,
            ThemeColor::Indigo,
            ThemeColor::Lime,
            ThemeColor::Orange,
            ThemeColor::Pink,
            ThemeColor::Purple,
            ThemeColor::Red,
            ThemeColor::Rose,
            ThemeColor::Sky,
            ThemeColor::Teal,
            ThemeColor::Violet,
            ThemeColor::Yellow,
            ThemeColor::Mauve,
            ThemeColor::Olive,
            ThemeColor::Mist,
            ThemeColor::Taupe,
        ];
        ALL.get(idx).copied()
    }

    fn index(&self) -> usize {
        match self {
            ThemeColor::Neutral => 0,
            ThemeColor::Stone => 1,
            ThemeColor::Zinc => 2,
            ThemeColor::Gray => 3,
            ThemeColor::Amber => 4,
            ThemeColor::Blue => 5,
            ThemeColor::Cyan => 6,
            ThemeColor::Emerald => 7,
            ThemeColor::Fuchsia => 8,
            ThemeColor::Green => 9,
            ThemeColor::Indigo => 10,
            ThemeColor::Lime => 11,
            ThemeColor::Orange => 12,
            ThemeColor::Pink => 13,
            ThemeColor::Purple => 14,
            ThemeColor::Red => 15,
            ThemeColor::Rose => 16,
            ThemeColor::Sky => 17,
            ThemeColor::Teal => 18,
            ThemeColor::Violet => 19,
            ThemeColor::Yellow => 20,
            ThemeColor::Mauve => 21,
            ThemeColor::Olive => 22,
            ThemeColor::Mist => 23,
            ThemeColor::Taupe => 24,
        }
    }
}

/// Radius options from the source `RADII` table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Radius {
    /// Inherit the theme's `--radius` (0.625rem).
    #[default]
    Default,
    /// `0`
    None,
    /// `0.45rem`
    Small,
    /// `0.625rem`
    Medium,
    /// `0.875rem`
    Large,
}

impl Radius {
    /// Explicit rem value, or `None` for [`Radius::Default`].
    pub fn rem(&self) -> Option<f32> {
        match self {
            Radius::Default => None,
            Radius::None => Some(0.0),
            Radius::Small => Some(0.45),
            Radius::Medium => Some(0.625),
            Radius::Large => Some(0.875),
        }
    }

    fn from_index(idx: usize) -> Self {
        const ALL: [Radius; 5] = [
            Radius::Default,
            Radius::None,
            Radius::Small,
            Radius::Medium,
            Radius::Large,
        ];
        ALL.get(idx).copied().unwrap_or(Radius::Default)
    }

    fn index(&self) -> usize {
        match self {
            Radius::Default => 0,
            Radius::None => 1,
            Radius::Small => 2,
            Radius::Medium => 3,
            Radius::Large => 4,
        }
    }
}

/// Menu color option (source `PRESET_MENU_COLORS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum MenuColor {
    #[default]
    Default,
    Inverted,
    DefaultTranslucent,
    InvertedTranslucent,
}

impl MenuColor {
    fn from_index(idx: usize) -> Self {
        const ALL: [MenuColor; 4] = [
            MenuColor::Default,
            MenuColor::Inverted,
            MenuColor::DefaultTranslucent,
            MenuColor::InvertedTranslucent,
        ];
        ALL.get(idx).copied().unwrap_or_default()
    }

    fn index(&self) -> usize {
        match self {
            MenuColor::Default => 0,
            MenuColor::Inverted => 1,
            MenuColor::DefaultTranslucent => 2,
            MenuColor::InvertedTranslucent => 3,
        }
    }
}

/// Menu accent option (source `PRESET_MENU_ACCENTS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum MenuAccent {
    #[default]
    Subtle,
    Bold,
}

impl MenuAccent {
    fn from_index(idx: usize) -> Self {
        if idx == 1 {
            MenuAccent::Bold
        } else {
            MenuAccent::Subtle
        }
    }

    fn index(&self) -> usize {
        match self {
            MenuAccent::Subtle => 0,
            MenuAccent::Bold => 1,
        }
    }
}

/// A full design-system preset, equivalent to one `/create` configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    pub style: Style,
    pub base_color: BaseColor,
    pub theme_color: ThemeColor,
    /// Chart color; defaults to the theme color like the source does
    /// (`V1_CHART_COLOR_MAP` legacy handling aside).
    pub chart_color: ThemeColor,
    /// Icon library; only `lucide` ships with this crate, other values are
    /// kept for code compatibility.
    pub icon_library: &'static str,
    pub font: &'static str,
    pub font_heading: &'static str,
    pub radius: Radius,
    pub menu_color: MenuColor,
    pub menu_accent: MenuAccent,
}

impl Default for Preset {
    fn default() -> Self {
        Self::default_preset()
    }
}

impl Preset {
    /// The source `DEFAULT_PRESET_CONFIG` (nova / neutral / neutral).
    pub fn default_preset() -> Self {
        Self {
            style: Style::Nova,
            base_color: BaseColor::Neutral,
            theme_color: ThemeColor::Neutral,
            chart_color: ThemeColor::Neutral,
            icon_library: "lucide",
            font: "inter",
            font_heading: "inherit",
            radius: Radius::Default,
            menu_color: MenuColor::Default,
            menu_accent: MenuAccent::Subtle,
        }
    }

    /// Builder-style setter for the style.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder-style setter for the base color.
    pub fn with_base_color(mut self, color: BaseColor) -> Self {
        self.base_color = color;
        self
    }

    /// Builder-style setter for the theme (accent) color.
    pub fn with_theme_color(mut self, color: ThemeColor) -> Self {
        self.theme_color = color;
        self
    }

    /// Builder-style setter for the radius.
    pub fn with_radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    /// Encodes this preset into a base62 preset code (v2, `"b"` prefix),
    /// matching the source `encodePreset` bit layout exactly.
    pub fn to_code(&self) -> String {
        let mut bits: u64 = 0;
        let mut offset = 0u32;
        let fields: [(usize, u32); 10] = [
            (self.menu_color.index(), 3),
            (self.menu_accent.index(), 3),
            (self.radius.index(), 4),
            (font_index(self.font), 6),
            (icon_index(self.icon_library), 6),
            (self.theme_color.index(), 6),
            (self.base_color.index(), 6),
            (self.style.index(), 6),
            (self.chart_color.index(), 6),
            (font_heading_index(self.font_heading), 5),
        ];
        for (idx, width) in fields {
            bits += (idx as u64) << offset;
            offset += width;
        }
        format!("b{}", to_base62(bits))
    }

    /// Decodes a preset code (`"b0"`, `"a..."` v1 codes supported).
    /// Returns `None` for invalid codes.
    pub fn from_code(code: &str) -> Option<Self> {
        let mut chars = code.chars();
        let version = chars.next()?;
        if !matches!(version, 'a' | 'b') || code.len() < 2 || code.len() > 10 {
            return None;
        }
        let rest: String = chars.collect();
        let bits = from_base62(&rest)?;
        let mut offset = 0u32;
        let mut field = |width: u32| -> usize {
            let idx = ((bits >> offset) & ((1u64 << width) - 1)) as usize;
            offset += width;
            idx
        };
        let menu_color = MenuColor::from_index(field(3));
        let menu_accent = MenuAccent::from_index(field(3));
        let radius = Radius::from_index(field(4));
        let font = font_from_index(field(6));
        let icon_library = icon_from_index(field(6));
        let theme_color = ThemeColor::from_index(field(6));
        let base_color = BaseColor::from_index(field(6));
        let style = Style::from_index(field(6));
        let chart_color = if version == 'a' {
            theme_color.unwrap_or_default()
        } else {
            ThemeColor::from_index(field(6)).unwrap_or_default()
        };
        let font_heading = if version == 'a' {
            "inherit"
        } else {
            font_heading_from_index(field(5))
        };

        Some(Self {
            style: style.unwrap_or_default(),
            base_color: base_color.unwrap_or_default(),
            theme_color: theme_color.unwrap_or_default(),
            chart_color,
            icon_library,
            font,
            font_heading,
            radius,
            menu_color,
            menu_accent,
        })
    }

    /// Installs this preset into the context, preserving the currently
    /// installed mode if a theme is already present (defaults to light).
    pub fn install(&self, ctx: &egui::Context) {
        let mode =
            crate::theme::ShadcnTheme::current_mode(ctx).unwrap_or(crate::theme::Mode::Light);
        self.build(mode).install(ctx);
    }

    /// Resolves the merged slot table for `mode`: base palette, overridden by
    /// the accent theme's sparse palette, overridden by mode defaults.
    pub fn slots(&self, mode: crate::theme::Mode) -> Slots {
        let base = find_theme(self.base_color.name()).unwrap_or(&NEUTRAL_FALLBACK);
        // A theme that names a base color means "no accent" — the base color
        // already defines the full palette. (The source links theme and base
        // color on the create page; picking "neutral" theme over "stone" base
        // must not wipe stone's palette with neutral's.)
        let accent = if is_base_color_name(self.theme_color.name()) {
            None
        } else {
            find_theme(self.theme_color.name())
        };
        let source = match mode {
            crate::theme::Mode::Light => base.light,
            crate::theme::Mode::Dark => base.dark,
        };
        let mut slots: Slots = source;
        // Mode defaults fill slots the registry does not carry per theme
        // (destructive-foreground, surface, code, selection).
        let defaults = match mode {
            crate::theme::Mode::Light => &MODE_DEFAULTS_LIGHT,
            crate::theme::Mode::Dark => &MODE_DEFAULTS_DARK,
        };
        for (i, slot) in slots.iter_mut().enumerate() {
            if slot.is_empty() && !defaults[i].is_empty() {
                *slot = defaults[i];
            }
        }
        if let Some(accent) = accent {
            let accent_slots = match mode {
                crate::theme::Mode::Light => accent.light,
                crate::theme::Mode::Dark => accent.dark,
            };
            for (i, slot) in accent_slots.iter().enumerate() {
                if !slot.is_empty() {
                    slots[i] = slot;
                }
            }
        }
        // The chart color rides on top of everything, overriding chart slots.
        if self.chart_color.name() != self.theme_color.name() {
            if let Some(chart) = find_theme(self.chart_color.name()) {
                let chart_slots = match mode {
                    crate::theme::Mode::Light => chart.light,
                    crate::theme::Mode::Dark => chart.dark,
                };
                for i in crate::theme::colors::slot::CHART_1..=crate::theme::colors::slot::CHART_5 {
                    if !chart_slots[i].is_empty() {
                        slots[i] = chart_slots[i];
                    }
                }
            }
        }
        slots
    }

    /// Builds the fully resolved theme for `mode`.
    pub fn build(&self, mode: crate::theme::Mode) -> crate::theme::ShadcnTheme {
        let slots = self.slots(mode);
        let spec = ColorSpec::from_slots(slots);
        let base_radius = spec.radius_px(10.0);
        let radius_px = self.radius.rem().map(|r| r * 16.0).unwrap_or(base_radius);
        crate::theme::ShadcnTheme::new(
            mode,
            &spec,
            RadiusTokens::new(radius_px),
            self.style,
            self.clone(),
        )
    }
}

/// `gray` is a preset value but not a registry theme; fall back to neutral.
const NEUTRAL_FALLBACK: ThemeSpec = crate::theme::theme_data::NEUTRAL;

/// The 7 base colors with full palettes; a theme naming one of these means
/// "no accent override".
fn is_base_color_name(name: &str) -> bool {
    matches!(
        name,
        "neutral" | "stone" | "zinc" | "mauve" | "olive" | "mist" | "taupe"
    )
}

/// Finds a theme spec by registry name.
pub fn find_theme(name: &str) -> Option<&'static ThemeSpec> {
    THEMES.iter().find(|t| t.name == name)
}

// ---------------------------------------------------------------------------
// Base62 + field tables (verbatim from the source preset module).
// ---------------------------------------------------------------------------

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Source `PRESET_FONTS` order (never reorder).
const PRESET_FONTS: &[&str] = &[
    "inter",
    "noto-sans",
    "nunito-sans",
    "figtree",
    "roboto",
    "raleway",
    "dm-sans",
    "public-sans",
    "outfit",
    "jetbrains-mono",
    "geist",
    "geist-mono",
    "lora",
    "merriweather",
    "playfair-display",
    "noto-serif",
    "roboto-slab",
    "oxanium",
    "manrope",
    "space-grotesk",
    "montserrat",
    "ibm-plex-sans",
    "source-sans-3",
    "instrument-sans",
    "eb-garamond",
    "instrument-serif",
];

/// Source `PRESET_ICON_LIBRARIES` order.
const PRESET_ICON_LIBRARIES: &[&str] = &["lucide", "hugeicons", "tabler", "phosphor", "remixicon"];

fn font_index(name: &str) -> usize {
    PRESET_FONTS.iter().position(|f| *f == name).unwrap_or(0)
}

fn font_from_index(idx: usize) -> &'static str {
    PRESET_FONTS.get(idx).copied().unwrap_or("inter")
}

fn font_heading_index(name: &str) -> usize {
    // fontHeading = "inherit" + fonts; index 0 = inherit.
    if name == "inherit" {
        0
    } else {
        font_index(name) + 1
    }
}

fn font_heading_from_index(idx: usize) -> &'static str {
    if idx == 0 {
        "inherit"
    } else {
        font_from_index(idx - 1)
    }
}

fn icon_index(name: &str) -> usize {
    PRESET_ICON_LIBRARIES
        .iter()
        .position(|f| *f == name)
        .unwrap_or(0)
}

fn icon_from_index(idx: usize) -> &'static str {
    PRESET_ICON_LIBRARIES.get(idx).copied().unwrap_or("lucide")
}

fn to_base62(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while n > 0 {
        out.push(BASE62[(n % 62) as usize] as char);
        n /= 62;
    }
    out.iter().rev().collect()
}

fn from_base62(s: &str) -> Option<u64> {
    let mut n: u64 = 0;
    for c in s.chars() {
        let idx = BASE62.iter().position(|b| *b as char == c)?;
        n = n.checked_mul(62)?.checked_add(idx as u64)?;
    }
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_code_is_b0() {
        assert_eq!(Preset::default_preset().to_code(), "b0");
    }

    #[test]
    fn round_trip_default() {
        let code = Preset::default_preset().to_code();
        let decoded = Preset::from_code(&code).unwrap();
        assert_eq!(decoded, Preset::default_preset());
    }

    #[test]
    fn round_trip_all_styles() {
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
            let p = Preset::default_preset().with_style(style);
            assert_eq!(
                Preset::from_code(&p.to_code()).unwrap(),
                p,
                "style {}",
                style.name()
            );
        }
    }

    #[test]
    fn round_trip_base_colors_and_radii() {
        for color in [
            BaseColor::Neutral,
            BaseColor::Stone,
            BaseColor::Zinc,
            BaseColor::Gray,
            BaseColor::Mauve,
            BaseColor::Olive,
            BaseColor::Mist,
            BaseColor::Taupe,
        ] {
            for radius in [
                Radius::Default,
                Radius::None,
                Radius::Small,
                Radius::Medium,
                Radius::Large,
            ] {
                let p = Preset::default_preset()
                    .with_base_color(color)
                    .with_radius(radius);
                let radius_label = format!("{radius:?}");
                assert_eq!(
                    Preset::from_code(&p.to_code()).unwrap(),
                    p,
                    "{} {}",
                    color.name(),
                    radius_label
                );
            }
        }
    }

    #[test]
    fn rejects_bad_codes() {
        assert!(Preset::from_code("").is_none());
        assert!(Preset::from_code("b").is_none());
        assert!(Preset::from_code("z0").is_none()); // z not a version
        assert!(Preset::from_code("b!").is_none());
        assert!(Preset::from_code("b01234567890").is_none()); // too long
    }

    #[test]
    fn v1_codes_decode_without_chart_color() {
        // Hand-built v1 code: same bits minus chartColor/fontHeading fields.
        let p = Preset::from_code("a0").unwrap();
        assert_eq!(p.style, Style::Nova);
        assert_eq!(p.chart_color, ThemeColor::Neutral); // inherits theme
        assert_eq!(p.font_heading, "inherit");
    }

    #[test]
    fn merged_slots_fall_back_to_base() {
        let p = Preset::default_preset().with_theme_color(ThemeColor::Blue);
        let light = p.slots(crate::theme::Mode::Light);
        assert_eq!(
            light[crate::theme::colors::slot::BACKGROUND],
            "oklch(1 0 0)"
        );
        assert_eq!(
            light[crate::theme::colors::slot::PRIMARY],
            "oklch(0.488 0.243 264.376)"
        );
        assert_eq!(
            light[crate::theme::colors::slot::DESTRUCTIVE],
            "oklch(0.577 0.245 27.325)"
        );
    }

    #[test]
    fn base_color_switch_changes_background() {
        let p = Preset::default_preset().with_base_color(BaseColor::Stone);
        let light = p.slots(crate::theme::Mode::Light);
        assert_eq!(
            light[crate::theme::colors::slot::FOREGROUND],
            "oklch(0.147 0.004 49.25)"
        );
    }
}
