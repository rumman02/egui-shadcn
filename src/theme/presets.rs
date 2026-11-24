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
    /// the accent theme's sparse palette, overridden by mode defaults. In
    /// light mode, `surface-foreground` / `code-foreground` then resolve from
    /// the merged `foreground`, following the `globals.css` `var()` chain.
    /// In dark mode with a neutral base, `accent` resolves to the docs-site
    /// value `oklch(0.371 0 0)` (`globals.css:156`) instead of the registry
    /// data's `0.269` (see the override below). In light mode with a neutral
    /// base, `foreground` / `card-foreground` / `popover-foreground` /
    /// `primary` resolve to the docs-site value `oklch(0% 0 0)`
    /// (`globals.css:102-107`) instead of the registry data's `0.145` /
    /// `0.205` (see the override below; `primary` only when no accent theme
    /// overlays its own primary).
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
        // Light neutral foregrounds: the registry data (`themes.ts`) carries
        // `oklch(0.145 0 0)` for the neutral light foreground /
        // card-foreground / popover-foreground and `oklch(0.205 0 0)` for
        // primary, but the docs site's stylesheet renders `oklch(0% 0 0)`
        // for all four (snapshot `apps/v4/app/globals.css:102-107`). Per the
        // coordinator ruling of 2026-11-24 (FND-D1, Expert Review decision:
        // globals.css wins), the rendered reference governs these slots:
        // override at resolve time so the generated theme data and its
        // fidelity tests stay untouched. Neutral + light only — every other
        // base keeps its `themes.ts` values, and the `gray` fallback
        // resolves through neutral so it inherits the override. Placed
        // before the light var() chain so `surface-foreground` /
        // `code-foreground` follow the overridden foreground, matching the
        // globals.css chain (`--surface-foreground: var(--foreground)`).
        // Chromatic accent themes carry a `primary` key but no foreground
        // keys (verified by parsing `themes.ts`): the three foreground
        // slots always resolve from the base, while `primary` is overridden
        // only when no accent theme overlays its own primary.
        if matches!(mode, crate::theme::Mode::Light) && base.name == "neutral" {
            slots[crate::theme::colors::slot::FOREGROUND] = "oklch(0% 0 0)";
            slots[crate::theme::colors::slot::CARD_FOREGROUND] = "oklch(0% 0 0)";
            slots[crate::theme::colors::slot::POPOVER_FOREGROUND] = "oklch(0% 0 0)";
            let accent_carries_primary =
                accent.is_some_and(|a| !a.light[crate::theme::colors::slot::PRIMARY].is_empty());
            if !accent_carries_primary {
                slots[crate::theme::colors::slot::PRIMARY] = "oklch(0% 0 0)";
            }
        }
        // Light mode resolves the globals.css var() chain
        // --surface-foreground: var(--foreground) and
        // --code-foreground: var(--surface-foreground) from the fully merged
        // foreground, after base/default/accent/chart merging, so non-neutral
        // bases get their own foreground instead of the neutral value in
        // MODE_DEFAULTS_LIGHT. Dark mode keeps the verbatim globals.css
        // defaults: dark --surface-foreground is not a var() reference.
        if matches!(mode, crate::theme::Mode::Light) {
            slots[crate::theme::colors::slot::SURFACE_FOREGROUND] =
                slots[crate::theme::colors::slot::FOREGROUND];
            slots[crate::theme::colors::slot::CODE_FOREGROUND] =
                slots[crate::theme::colors::slot::SURFACE_FOREGROUND];
        }
        // Dark neutral accent: the registry data (`themes.ts`) carries
        // `oklch(0.269 0 0)` for the dark neutral accent, but the docs
        // site's stylesheet renders `oklch(0.371 0 0)` (snapshot
        // `apps/v4/app/globals.css:156`), which the frozen select spec
        // cites. Per the coordinator ruling of 2026-11-15 (SEL-VIS-03,
        // Position A), the rendered reference governs this one slot:
        // override at resolve time so the generated theme data and its
        // fidelity tests stay untouched. Neutral + dark only — every
        // other base/mode keeps its `themes.ts` accent, and the `gray`
        // fallback resolves through neutral so it inherits the override.
        if matches!(mode, crate::theme::Mode::Dark) && base.name == "neutral" {
            slots[crate::theme::colors::slot::ACCENT] = "oklch(0.371 0 0)";
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
    // --- Token slot-shift regression tests (fix/token-slot-shift) ---
    //
    // EXPECTED_THEME_SLOTS is transcribed from the source
    // apps/v4/registry/themes.ts by key NAME (not position): each entry packs
    // slots 0..=32 in SLOT_NAMES order, `|`-separated, empty = the registry
    // does not carry that key for that theme/mode. Slots 33..40 are
    // globals.css-only and are covered by the mode-default tests instead.
    // If the registry gains themes/keys, regenerate this table from themes.ts
    // with a keyed parser and update the count assertion below.
    const EXPECTED_THEME_SLOTS: &[(&str, &str, &str)] = &[
        ("neutral", "light", "oklch(1 0 0)|oklch(0.145 0 0)|oklch(1 0 0)|oklch(0.145 0 0)|oklch(1 0 0)|oklch(0.145 0 0)|oklch(0.205 0 0)|oklch(0.985 0 0)|oklch(0.97 0 0)|oklch(0.205 0 0)|oklch(0.97 0 0)|oklch(0.556 0 0)|oklch(0.97 0 0)|oklch(0.205 0 0)|oklch(0.577 0.245 27.325)||oklch(0.922 0 0)|oklch(0.922 0 0)|oklch(0.708 0 0)|oklch(0.87 0 0)|oklch(0.556 0 0)|oklch(0.439 0 0)|oklch(0.371 0 0)|oklch(0.269 0 0)|0.625rem|oklch(0.985 0 0)|oklch(0.145 0 0)|oklch(0.205 0 0)|oklch(0.985 0 0)|oklch(0.97 0 0)|oklch(0.205 0 0)|oklch(0.922 0 0)|oklch(0.708 0 0)"),
        ("neutral", "dark", "oklch(0.145 0 0)|oklch(0.985 0 0)|oklch(0.205 0 0)|oklch(0.985 0 0)|oklch(0.205 0 0)|oklch(0.985 0 0)|oklch(0.922 0 0)|oklch(0.205 0 0)|oklch(0.269 0 0)|oklch(0.985 0 0)|oklch(0.269 0 0)|oklch(0.708 0 0)|oklch(0.269 0 0)|oklch(0.985 0 0)|oklch(0.704 0.191 22.216)||oklch(1 0 0 / 10%)|oklch(1 0 0 / 15%)|oklch(0.556 0 0)|oklch(0.87 0 0)|oklch(0.556 0 0)|oklch(0.439 0 0)|oklch(0.371 0 0)|oklch(0.269 0 0)||oklch(0.205 0 0)|oklch(0.985 0 0)|oklch(0.488 0.243 264.376)|oklch(0.985 0 0)|oklch(0.269 0 0)|oklch(0.985 0 0)|oklch(1 0 0 / 10%)|oklch(0.556 0 0)"),
        ("stone", "light", "oklch(1 0 0)|oklch(0.147 0.004 49.25)|oklch(1 0 0)|oklch(0.147 0.004 49.25)|oklch(1 0 0)|oklch(0.147 0.004 49.25)|oklch(0.216 0.006 56.043)|oklch(0.985 0.001 106.423)|oklch(0.97 0.001 106.424)|oklch(0.216 0.006 56.043)|oklch(0.97 0.001 106.424)|oklch(0.553 0.013 58.071)|oklch(0.97 0.001 106.424)|oklch(0.216 0.006 56.043)|oklch(0.577 0.245 27.325)||oklch(0.923 0.003 48.717)|oklch(0.923 0.003 48.717)|oklch(0.709 0.01 56.259)|oklch(0.869 0.005 56.366)|oklch(0.553 0.013 58.071)|oklch(0.444 0.011 73.639)|oklch(0.374 0.01 67.558)|oklch(0.268 0.007 34.298)|0.625rem|oklch(0.985 0.001 106.423)|oklch(0.147 0.004 49.25)|oklch(0.216 0.006 56.043)|oklch(0.985 0.001 106.423)|oklch(0.97 0.001 106.424)|oklch(0.216 0.006 56.043)|oklch(0.923 0.003 48.717)|oklch(0.709 0.01 56.259)"),
        ("stone", "dark", "oklch(0.147 0.004 49.25)|oklch(0.985 0.001 106.423)|oklch(0.216 0.006 56.043)|oklch(0.985 0.001 106.423)|oklch(0.216 0.006 56.043)|oklch(0.985 0.001 106.423)|oklch(0.923 0.003 48.717)|oklch(0.216 0.006 56.043)|oklch(0.268 0.007 34.298)|oklch(0.985 0.001 106.423)|oklch(0.268 0.007 34.298)|oklch(0.709 0.01 56.259)|oklch(0.268 0.007 34.298)|oklch(0.985 0.001 106.423)|oklch(0.704 0.191 22.216)||oklch(1 0 0 / 10%)|oklch(1 0 0 / 15%)|oklch(0.553 0.013 58.071)|oklch(0.869 0.005 56.366)|oklch(0.553 0.013 58.071)|oklch(0.444 0.011 73.639)|oklch(0.374 0.01 67.558)|oklch(0.268 0.007 34.298)||oklch(0.216 0.006 56.043)|oklch(0.985 0.001 106.423)|oklch(0.488 0.243 264.376)|oklch(0.985 0.001 106.423)|oklch(0.268 0.007 34.298)|oklch(0.985 0.001 106.423)|oklch(1 0 0 / 10%)|oklch(0.553 0.013 58.071)"),
        ("zinc", "light", "oklch(1 0 0)|oklch(0.141 0.005 285.823)|oklch(1 0 0)|oklch(0.141 0.005 285.823)|oklch(1 0 0)|oklch(0.141 0.005 285.823)|oklch(0.21 0.006 285.885)|oklch(0.985 0 0)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)|oklch(0.967 0.001 286.375)|oklch(0.552 0.016 285.938)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)|oklch(0.577 0.245 27.325)||oklch(0.92 0.004 286.32)|oklch(0.92 0.004 286.32)|oklch(0.705 0.015 286.067)|oklch(0.871 0.006 286.286)|oklch(0.552 0.016 285.938)|oklch(0.442 0.017 285.786)|oklch(0.37 0.013 285.805)|oklch(0.274 0.006 286.033)|0.625rem|oklch(0.985 0 0)|oklch(0.141 0.005 285.823)|oklch(0.21 0.006 285.885)|oklch(0.985 0 0)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)|oklch(0.92 0.004 286.32)|oklch(0.705 0.015 286.067)"),
        ("zinc", "dark", "oklch(0.141 0.005 285.823)|oklch(0.985 0 0)|oklch(0.21 0.006 285.885)|oklch(0.985 0 0)|oklch(0.21 0.006 285.885)|oklch(0.985 0 0)|oklch(0.92 0.004 286.32)|oklch(0.21 0.006 285.885)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)|oklch(0.274 0.006 286.033)|oklch(0.705 0.015 286.067)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)|oklch(0.704 0.191 22.216)||oklch(1 0 0 / 10%)|oklch(1 0 0 / 15%)|oklch(0.552 0.016 285.938)|oklch(0.871 0.006 286.286)|oklch(0.552 0.016 285.938)|oklch(0.442 0.017 285.786)|oklch(0.37 0.013 285.805)|oklch(0.274 0.006 286.033)||oklch(0.21 0.006 285.885)|oklch(0.985 0 0)|oklch(0.488 0.243 264.376)|oklch(0.985 0 0)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)|oklch(1 0 0 / 10%)|oklch(0.552 0.016 285.938)"),
        ("mauve", "light", "oklch(1 0 0)|oklch(0.145 0.008 326)|oklch(1 0 0)|oklch(0.145 0.008 326)|oklch(1 0 0)|oklch(0.145 0.008 326)|oklch(0.212 0.019 322.12)|oklch(0.985 0 0)|oklch(0.96 0.003 325.6)|oklch(0.212 0.019 322.12)|oklch(0.96 0.003 325.6)|oklch(0.542 0.034 322.5)|oklch(0.96 0.003 325.6)|oklch(0.212 0.019 322.12)|oklch(0.577 0.245 27.325)||oklch(0.922 0.005 325.62)|oklch(0.922 0.005 325.62)|oklch(0.711 0.019 323.02)|oklch(0.865 0.012 325.68)|oklch(0.542 0.034 322.5)|oklch(0.435 0.029 321.78)|oklch(0.364 0.029 323.89)|oklch(0.263 0.024 320.12)|0.625rem|oklch(0.985 0 0)|oklch(0.145 0.008 326)|oklch(0.212 0.019 322.12)|oklch(0.985 0 0)|oklch(0.96 0.003 325.6)|oklch(0.212 0.019 322.12)|oklch(0.922 0.005 325.62)|oklch(0.711 0.019 323.02)"),
        ("mauve", "dark", "oklch(0.145 0.008 326)|oklch(0.985 0 0)|oklch(0.212 0.019 322.12)|oklch(0.985 0 0)|oklch(0.212 0.019 322.12)|oklch(0.985 0 0)|oklch(0.922 0.005 325.62)|oklch(0.212 0.019 322.12)|oklch(0.263 0.024 320.12)|oklch(0.985 0 0)|oklch(0.263 0.024 320.12)|oklch(0.711 0.019 323.02)|oklch(0.263 0.024 320.12)|oklch(0.985 0 0)|oklch(0.704 0.191 22.216)||oklch(1 0 0 / 10%)|oklch(1 0 0 / 15%)|oklch(0.542 0.034 322.5)|oklch(0.865 0.012 325.68)|oklch(0.542 0.034 322.5)|oklch(0.435 0.029 321.78)|oklch(0.364 0.029 323.89)|oklch(0.263 0.024 320.12)||oklch(0.212 0.019 322.12)|oklch(0.985 0 0)|oklch(0.488 0.243 264.376)|oklch(0.985 0 0)|oklch(0.263 0.024 320.12)|oklch(0.985 0 0)|oklch(1 0 0 / 10%)|oklch(0.542 0.034 322.5)"),
        ("olive", "light", "oklch(1 0 0)|oklch(0.153 0.006 107.1)|oklch(1 0 0)|oklch(0.153 0.006 107.1)|oklch(1 0 0)|oklch(0.153 0.006 107.1)|oklch(0.228 0.013 107.4)|oklch(0.988 0.003 106.5)|oklch(0.966 0.005 106.5)|oklch(0.228 0.013 107.4)|oklch(0.966 0.005 106.5)|oklch(0.58 0.031 107.3)|oklch(0.966 0.005 106.5)|oklch(0.228 0.013 107.4)|oklch(0.577 0.245 27.325)||oklch(0.93 0.007 106.5)|oklch(0.93 0.007 106.5)|oklch(0.737 0.021 106.9)|oklch(0.88 0.011 106.6)|oklch(0.58 0.031 107.3)|oklch(0.466 0.025 107.3)|oklch(0.394 0.023 107.4)|oklch(0.286 0.016 107.4)|0.625rem|oklch(0.988 0.003 106.5)|oklch(0.153 0.006 107.1)|oklch(0.228 0.013 107.4)|oklch(0.988 0.003 106.5)|oklch(0.966 0.005 106.5)|oklch(0.228 0.013 107.4)|oklch(0.93 0.007 106.5)|oklch(0.737 0.021 106.9)"),
        ("olive", "dark", "oklch(0.153 0.006 107.1)|oklch(0.988 0.003 106.5)|oklch(0.228 0.013 107.4)|oklch(0.988 0.003 106.5)|oklch(0.228 0.013 107.4)|oklch(0.988 0.003 106.5)|oklch(0.93 0.007 106.5)|oklch(0.228 0.013 107.4)|oklch(0.286 0.016 107.4)|oklch(0.988 0.003 106.5)|oklch(0.286 0.016 107.4)|oklch(0.737 0.021 106.9)|oklch(0.286 0.016 107.4)|oklch(0.988 0.003 106.5)|oklch(0.704 0.191 22.216)||oklch(1 0 0 / 10%)|oklch(1 0 0 / 15%)|oklch(0.58 0.031 107.3)|oklch(0.88 0.011 106.6)|oklch(0.58 0.031 107.3)|oklch(0.466 0.025 107.3)|oklch(0.394 0.023 107.4)|oklch(0.286 0.016 107.4)||oklch(0.228 0.013 107.4)|oklch(0.988 0.003 106.5)|oklch(0.488 0.243 264.376)|oklch(0.988 0.003 106.5)|oklch(0.286 0.016 107.4)|oklch(0.988 0.003 106.5)|oklch(1 0 0 / 10%)|oklch(0.58 0.031 107.3)"),
        ("mist", "light", "oklch(1 0 0)|oklch(0.148 0.004 228.8)|oklch(1 0 0)|oklch(0.148 0.004 228.8)|oklch(1 0 0)|oklch(0.148 0.004 228.8)|oklch(0.218 0.008 223.9)|oklch(0.987 0.002 197.1)|oklch(0.963 0.002 197.1)|oklch(0.218 0.008 223.9)|oklch(0.963 0.002 197.1)|oklch(0.56 0.021 213.5)|oklch(0.963 0.002 197.1)|oklch(0.218 0.008 223.9)|oklch(0.577 0.245 27.325)||oklch(0.925 0.005 214.3)|oklch(0.925 0.005 214.3)|oklch(0.723 0.014 214.4)|oklch(0.872 0.007 219.6)|oklch(0.56 0.021 213.5)|oklch(0.45 0.017 213.2)|oklch(0.378 0.015 216)|oklch(0.275 0.011 216.9)|0.625rem|oklch(0.987 0.002 197.1)|oklch(0.148 0.004 228.8)|oklch(0.218 0.008 223.9)|oklch(0.987 0.002 197.1)|oklch(0.963 0.002 197.1)|oklch(0.218 0.008 223.9)|oklch(0.925 0.005 214.3)|oklch(0.723 0.014 214.4)"),
        ("mist", "dark", "oklch(0.148 0.004 228.8)|oklch(0.987 0.002 197.1)|oklch(0.218 0.008 223.9)|oklch(0.987 0.002 197.1)|oklch(0.218 0.008 223.9)|oklch(0.987 0.002 197.1)|oklch(0.925 0.005 214.3)|oklch(0.218 0.008 223.9)|oklch(0.275 0.011 216.9)|oklch(0.987 0.002 197.1)|oklch(0.275 0.011 216.9)|oklch(0.723 0.014 214.4)|oklch(0.275 0.011 216.9)|oklch(0.987 0.002 197.1)|oklch(0.704 0.191 22.216)||oklch(1 0 0 / 10%)|oklch(1 0 0 / 15%)|oklch(0.56 0.021 213.5)|oklch(0.872 0.007 219.6)|oklch(0.56 0.021 213.5)|oklch(0.45 0.017 213.2)|oklch(0.378 0.015 216)|oklch(0.275 0.011 216.9)||oklch(0.218 0.008 223.9)|oklch(0.987 0.002 197.1)|oklch(0.488 0.243 264.376)|oklch(0.987 0.002 197.1)|oklch(0.275 0.011 216.9)|oklch(0.987 0.002 197.1)|oklch(1 0 0 / 10%)|oklch(0.56 0.021 213.5)"),
        ("taupe", "light", "oklch(1 0 0)|oklch(0.147 0.004 49.3)|oklch(1 0 0)|oklch(0.147 0.004 49.3)|oklch(1 0 0)|oklch(0.147 0.004 49.3)|oklch(0.214 0.009 43.1)|oklch(0.986 0.002 67.8)|oklch(0.96 0.002 17.2)|oklch(0.214 0.009 43.1)|oklch(0.96 0.002 17.2)|oklch(0.547 0.021 43.1)|oklch(0.96 0.002 17.2)|oklch(0.214 0.009 43.1)|oklch(0.577 0.245 27.325)||oklch(0.922 0.005 34.3)|oklch(0.922 0.005 34.3)|oklch(0.714 0.014 41.2)|oklch(0.868 0.007 39.5)|oklch(0.547 0.021 43.1)|oklch(0.438 0.017 39.3)|oklch(0.367 0.016 35.7)|oklch(0.268 0.011 36.5)|0.625rem|oklch(0.986 0.002 67.8)|oklch(0.147 0.004 49.3)|oklch(0.214 0.009 43.1)|oklch(0.986 0.002 67.8)|oklch(0.96 0.002 17.2)|oklch(0.214 0.009 43.1)|oklch(0.922 0.005 34.3)|oklch(0.714 0.014 41.2)"),
        ("taupe", "dark", "oklch(0.147 0.004 49.3)|oklch(0.986 0.002 67.8)|oklch(0.214 0.009 43.1)|oklch(0.986 0.002 67.8)|oklch(0.214 0.009 43.1)|oklch(0.986 0.002 67.8)|oklch(0.922 0.005 34.3)|oklch(0.214 0.009 43.1)|oklch(0.268 0.011 36.5)|oklch(0.986 0.002 67.8)|oklch(0.268 0.011 36.5)|oklch(0.714 0.014 41.2)|oklch(0.268 0.011 36.5)|oklch(0.986 0.002 67.8)|oklch(0.704 0.191 22.216)||oklch(1 0 0 / 10%)|oklch(1 0 0 / 15%)|oklch(0.547 0.021 43.1)|oklch(0.868 0.007 39.5)|oklch(0.547 0.021 43.1)|oklch(0.438 0.017 39.3)|oklch(0.367 0.016 35.7)|oklch(0.268 0.011 36.5)||oklch(0.214 0.009 43.1)|oklch(0.986 0.002 67.8)|oklch(0.488 0.243 264.376)|oklch(0.986 0.002 67.8)|oklch(0.268 0.011 36.5)|oklch(0.986 0.002 67.8)|oklch(1 0 0 / 10%)|oklch(0.547 0.021 43.1)"),
        ("amber", "light", "||||||oklch(0.555 0.163 48.998)|oklch(0.987 0.022 95.277)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.879 0.169 91.605)|oklch(0.769 0.188 70.08)|oklch(0.666 0.179 58.318)|oklch(0.555 0.163 48.998)|oklch(0.473 0.137 46.201)||||oklch(0.666 0.179 58.318)|oklch(0.987 0.022 95.277)||||"),
        ("amber", "dark", "||||||oklch(0.473 0.137 46.201)|oklch(0.987 0.022 95.277)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.879 0.169 91.605)|oklch(0.769 0.188 70.08)|oklch(0.666 0.179 58.318)|oklch(0.555 0.163 48.998)|oklch(0.473 0.137 46.201)||||oklch(0.769 0.188 70.08)|oklch(0.279 0.077 45.635)||||"),
        ("blue", "light", "||||||oklch(0.488 0.243 264.376)|oklch(0.97 0.014 254.604)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.809 0.105 251.813)|oklch(0.623 0.214 259.815)|oklch(0.546 0.245 262.881)|oklch(0.488 0.243 264.376)|oklch(0.424 0.199 265.638)||||oklch(0.546 0.245 262.881)|oklch(0.97 0.014 254.604)||||"),
        ("blue", "dark", "||||||oklch(0.424 0.199 265.638)|oklch(0.97 0.014 254.604)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.809 0.105 251.813)|oklch(0.623 0.214 259.815)|oklch(0.546 0.245 262.881)|oklch(0.488 0.243 264.376)|oklch(0.424 0.199 265.638)||||oklch(0.623 0.214 259.815)|oklch(0.97 0.014 254.604)||||"),
        ("cyan", "light", "||||||oklch(0.52 0.105 223.128)|oklch(0.984 0.019 200.873)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.865 0.127 207.078)|oklch(0.715 0.143 215.221)|oklch(0.609 0.126 221.723)|oklch(0.52 0.105 223.128)|oklch(0.45 0.085 224.283)||||oklch(0.609 0.126 221.723)|oklch(0.984 0.019 200.873)||||"),
        ("cyan", "dark", "||||||oklch(0.45 0.085 224.283)|oklch(0.984 0.019 200.873)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.865 0.127 207.078)|oklch(0.715 0.143 215.221)|oklch(0.609 0.126 221.723)|oklch(0.52 0.105 223.128)|oklch(0.45 0.085 224.283)||||oklch(0.715 0.143 215.221)|oklch(0.302 0.056 229.695)||||"),
        ("emerald", "light", "||||||oklch(0.508 0.118 165.612)|oklch(0.979 0.021 166.113)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.845 0.143 164.978)|oklch(0.696 0.17 162.48)|oklch(0.596 0.145 163.225)|oklch(0.508 0.118 165.612)|oklch(0.432 0.095 166.913)||||oklch(0.596 0.145 163.225)|oklch(0.979 0.021 166.113)||||"),
        ("emerald", "dark", "||||||oklch(0.432 0.095 166.913)|oklch(0.979 0.021 166.113)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.845 0.143 164.978)|oklch(0.696 0.17 162.48)|oklch(0.596 0.145 163.225)|oklch(0.508 0.118 165.612)|oklch(0.432 0.095 166.913)||||oklch(0.696 0.17 162.48)|oklch(0.262 0.051 172.552)||||"),
        ("fuchsia", "light", "||||||oklch(0.518 0.253 323.949)|oklch(0.977 0.017 320.058)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.833 0.145 321.434)|oklch(0.667 0.295 322.15)|oklch(0.591 0.293 322.896)|oklch(0.518 0.253 323.949)|oklch(0.452 0.211 324.591)||||oklch(0.591 0.293 322.896)|oklch(0.977 0.017 320.058)||||"),
        ("fuchsia", "dark", "||||||oklch(0.452 0.211 324.591)|oklch(0.977 0.017 320.058)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.833 0.145 321.434)|oklch(0.667 0.295 322.15)|oklch(0.591 0.293 322.896)|oklch(0.518 0.253 323.949)|oklch(0.452 0.211 324.591)||||oklch(0.667 0.295 322.15)|oklch(0.977 0.017 320.058)||||"),
        ("green", "light", "||||||oklch(0.527 0.154 150.069)|oklch(0.982 0.018 155.826)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.871 0.15 154.449)|oklch(0.723 0.219 149.579)|oklch(0.627 0.194 149.214)|oklch(0.527 0.154 150.069)|oklch(0.448 0.119 151.328)||||oklch(0.627 0.194 149.214)|oklch(0.982 0.018 155.826)||||"),
        ("green", "dark", "||||||oklch(0.448 0.119 151.328)|oklch(0.982 0.018 155.826)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.871 0.15 154.449)|oklch(0.723 0.219 149.579)|oklch(0.627 0.194 149.214)|oklch(0.527 0.154 150.069)|oklch(0.448 0.119 151.328)||||oklch(0.723 0.219 149.579)|oklch(0.982 0.018 155.826)||||"),
        ("indigo", "light", "||||||oklch(0.457 0.24 277.023)|oklch(0.962 0.018 272.314)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.785 0.115 274.713)|oklch(0.585 0.233 277.117)|oklch(0.511 0.262 276.966)|oklch(0.457 0.24 277.023)|oklch(0.398 0.195 277.366)||||oklch(0.511 0.262 276.966)|oklch(0.962 0.018 272.314)||||"),
        ("indigo", "dark", "||||||oklch(0.398 0.195 277.366)|oklch(0.962 0.018 272.314)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.785 0.115 274.713)|oklch(0.585 0.233 277.117)|oklch(0.511 0.262 276.966)|oklch(0.457 0.24 277.023)|oklch(0.398 0.195 277.366)||||oklch(0.585 0.233 277.117)|oklch(0.962 0.018 272.314)||||"),
        ("lime", "light", "||||||oklch(0.841 0.238 128.85)|oklch(0.405 0.101 131.063)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.897 0.196 126.665)|oklch(0.768 0.233 130.85)|oklch(0.648 0.2 131.684)|oklch(0.532 0.157 131.589)|oklch(0.453 0.124 130.933)||||oklch(0.648 0.2 131.684)|oklch(0.986 0.031 120.757)||||"),
        ("lime", "dark", "||||||oklch(0.768 0.233 130.85)|oklch(0.405 0.101 131.063)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.897 0.196 126.665)|oklch(0.768 0.233 130.85)|oklch(0.648 0.2 131.684)|oklch(0.532 0.157 131.589)|oklch(0.453 0.124 130.933)||||oklch(0.768 0.233 130.85)|oklch(0.274 0.072 132.109)||||"),
        ("orange", "light", "||||||oklch(0.553 0.195 38.402)|oklch(0.98 0.016 73.684)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.837 0.128 66.29)|oklch(0.705 0.213 47.604)|oklch(0.646 0.222 41.116)|oklch(0.553 0.195 38.402)|oklch(0.47 0.157 37.304)||||oklch(0.646 0.222 41.116)|oklch(0.98 0.016 73.684)||||"),
        ("orange", "dark", "||||||oklch(0.47 0.157 37.304)|oklch(0.98 0.016 73.684)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.837 0.128 66.29)|oklch(0.705 0.213 47.604)|oklch(0.646 0.222 41.116)|oklch(0.553 0.195 38.402)|oklch(0.47 0.157 37.304)||||oklch(0.705 0.213 47.604)|oklch(0.98 0.016 73.684)||||"),
        ("pink", "light", "||||||oklch(0.525 0.223 3.958)|oklch(0.971 0.014 343.198)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.823 0.12 346.018)|oklch(0.656 0.241 354.308)|oklch(0.592 0.249 0.584)|oklch(0.525 0.223 3.958)|oklch(0.459 0.187 3.815)||||oklch(0.592 0.249 0.584)|oklch(0.971 0.014 343.198)||||"),
        ("pink", "dark", "||||||oklch(0.459 0.187 3.815)|oklch(0.971 0.014 343.198)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.823 0.12 346.018)|oklch(0.656 0.241 354.308)|oklch(0.592 0.249 0.584)|oklch(0.525 0.223 3.958)|oklch(0.459 0.187 3.815)||||oklch(0.656 0.241 354.308)|oklch(0.971 0.014 343.198)||||"),
        ("purple", "light", "||||||oklch(0.496 0.265 301.924)|oklch(0.977 0.014 308.299)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.827 0.119 306.383)|oklch(0.627 0.265 303.9)|oklch(0.558 0.288 302.321)|oklch(0.496 0.265 301.924)|oklch(0.438 0.218 303.724)||||oklch(0.558 0.288 302.321)|oklch(0.977 0.014 308.299)||||"),
        ("purple", "dark", "||||||oklch(0.438 0.218 303.724)|oklch(0.977 0.014 308.299)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.827 0.119 306.383)|oklch(0.627 0.265 303.9)|oklch(0.558 0.288 302.321)|oklch(0.496 0.265 301.924)|oklch(0.438 0.218 303.724)||||oklch(0.627 0.265 303.9)|oklch(0.977 0.014 308.299)||||"),
        ("red", "light", "||||||oklch(0.505 0.213 27.518)|oklch(0.971 0.013 17.38)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.808 0.114 19.571)|oklch(0.637 0.237 25.331)|oklch(0.577 0.245 27.325)|oklch(0.505 0.213 27.518)|oklch(0.444 0.177 26.899)||||oklch(0.577 0.245 27.325)|oklch(0.971 0.013 17.38)||||"),
        ("red", "dark", "||||||oklch(0.444 0.177 26.899)|oklch(0.971 0.013 17.38)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.808 0.114 19.571)|oklch(0.637 0.237 25.331)|oklch(0.577 0.245 27.325)|oklch(0.505 0.213 27.518)|oklch(0.444 0.177 26.899)||||oklch(0.637 0.237 25.331)|oklch(0.971 0.013 17.38)||||"),
        ("rose", "light", "||||||oklch(0.514 0.222 16.935)|oklch(0.969 0.015 12.422)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.81 0.117 11.638)|oklch(0.645 0.246 16.439)|oklch(0.586 0.253 17.585)|oklch(0.514 0.222 16.935)|oklch(0.455 0.188 13.697)||||oklch(0.586 0.253 17.585)|oklch(0.969 0.015 12.422)||||"),
        ("rose", "dark", "||||||oklch(0.455 0.188 13.697)|oklch(0.969 0.015 12.422)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.81 0.117 11.638)|oklch(0.645 0.246 16.439)|oklch(0.586 0.253 17.585)|oklch(0.514 0.222 16.935)|oklch(0.455 0.188 13.697)||oklch(0.21 0.006 285.885)||oklch(0.645 0.246 16.439)|oklch(0.969 0.015 12.422)||||"),
        ("sky", "light", "||||||oklch(0.5 0.134 242.749)|oklch(0.977 0.013 236.62)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.828 0.111 230.318)|oklch(0.685 0.169 237.323)|oklch(0.588 0.158 241.966)|oklch(0.5 0.134 242.749)|oklch(0.443 0.11 240.79)||||oklch(0.588 0.158 241.966)|oklch(0.977 0.013 236.62)||||"),
        ("sky", "dark", "||||||oklch(0.443 0.11 240.79)|oklch(0.977 0.013 236.62)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.828 0.111 230.318)|oklch(0.685 0.169 237.323)|oklch(0.588 0.158 241.966)|oklch(0.5 0.134 242.749)|oklch(0.443 0.11 240.79)||||oklch(0.685 0.169 237.323)|oklch(0.293 0.066 243.157)||||"),
        ("teal", "light", "||||||oklch(0.511 0.096 186.391)|oklch(0.984 0.014 180.72)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.855 0.138 181.071)|oklch(0.704 0.14 182.503)|oklch(0.6 0.118 184.704)|oklch(0.511 0.096 186.391)|oklch(0.437 0.078 188.216)||||oklch(0.6 0.118 184.704)|oklch(0.984 0.014 180.72)||||"),
        ("teal", "dark", "||||||oklch(0.437 0.078 188.216)|oklch(0.984 0.014 180.72)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.855 0.138 181.071)|oklch(0.704 0.14 182.503)|oklch(0.6 0.118 184.704)|oklch(0.511 0.096 186.391)|oklch(0.437 0.078 188.216)||||oklch(0.704 0.14 182.503)|oklch(0.277 0.046 192.524)||||"),
        ("violet", "light", "||||||oklch(0.491 0.27 292.581)|oklch(0.969 0.016 293.756)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.811 0.111 293.571)|oklch(0.606 0.25 292.717)|oklch(0.541 0.281 293.009)|oklch(0.491 0.27 292.581)|oklch(0.432 0.232 292.759)||||oklch(0.541 0.281 293.009)|oklch(0.969 0.016 293.756)||||"),
        ("violet", "dark", "||||||oklch(0.432 0.232 292.759)|oklch(0.969 0.016 293.756)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.811 0.111 293.571)|oklch(0.606 0.25 292.717)|oklch(0.541 0.281 293.009)|oklch(0.491 0.27 292.581)|oklch(0.432 0.232 292.759)||||oklch(0.606 0.25 292.717)|oklch(0.969 0.016 293.756)||||"),
        ("yellow", "light", "||||||oklch(0.852 0.199 91.936)|oklch(0.421 0.095 57.708)|oklch(0.967 0.001 286.375)|oklch(0.21 0.006 285.885)||||||||||oklch(0.905 0.182 98.111)|oklch(0.795 0.184 86.047)|oklch(0.681 0.162 75.834)|oklch(0.554 0.135 66.442)|oklch(0.476 0.114 61.907)||||oklch(0.681 0.162 75.834)|oklch(0.987 0.026 102.212)||||"),
        ("yellow", "dark", "||||||oklch(0.795 0.184 86.047)|oklch(0.421 0.095 57.708)|oklch(0.274 0.006 286.033)|oklch(0.985 0 0)||||||||||oklch(0.905 0.182 98.111)|oklch(0.795 0.184 86.047)|oklch(0.681 0.162 75.834)|oklch(0.554 0.135 66.442)|oklch(0.476 0.114 61.907)||||oklch(0.795 0.184 86.047)|oklch(0.987 0.026 102.212)||||"),
    ];

    #[test]
    fn generated_theme_data_matches_registry_values() {
        use crate::theme::colors::SLOT_NAMES;
        assert_eq!(
            THEMES.len(),
            24,
            "registry theme count changed; regenerate EXPECTED_THEME_SLOTS"
        );
        for (name, mode, packed) in EXPECTED_THEME_SLOTS {
            let theme = THEMES
                .iter()
                .find(|t| t.name == *name)
                .unwrap_or_else(|| panic!("theme {name} missing from THEMES"));
            let slots = if *mode == "light" {
                theme.light
            } else {
                theme.dark
            };
            let expected: Vec<&str> = packed.split('|').collect();
            assert_eq!(
                expected.len(),
                33,
                "packed row for {name}/{mode} must have 33 fields"
            );
            for (i, want) in expected.iter().enumerate() {
                assert_eq!(
                    slots[i], *want,
                    "theme {name} mode {mode} slot {i} ({})",
                    SLOT_NAMES[i]
                );
            }
        }
        // Base/accent classification follows the registry: the 7 full palettes
        // are base colors, the 17 sparse palettes are accents.
        let base_names = [
            "neutral", "stone", "zinc", "mauve", "olive", "mist", "taupe",
        ];
        for theme in THEMES {
            assert_eq!(
                theme.is_base_color,
                base_names.contains(&theme.name),
                "theme {} is_base_color",
                theme.name
            );
        }
    }

    #[test]
    fn raw_theme_data_leaves_globals_css_slots_empty() {
        // destructive-foreground and slots 33..40 are not in themes.ts; the
        // generator must emit "" so Preset::slots fills MODE_DEFAULTS_*.
        // (On shifted data, slot 15 held the border value and the defaults
        // were dead code.)
        use crate::theme::colors::slot;
        for theme in THEMES {
            for (mode, slots) in [("light", theme.light), ("dark", theme.dark)] {
                assert_eq!(
                    slots[slot::DESTRUCTIVE_FOREGROUND],
                    "",
                    "{} {mode} destructive-foreground must be empty in raw data",
                    theme.name
                );
                for (i, value) in slots.iter().enumerate().skip(slot::SURFACE) {
                    assert_eq!(
                        *value, "",
                        "{} {mode} slot {i} must be empty in raw data",
                        theme.name
                    );
                }
            }
        }
    }

    #[test]
    fn raw_theme_slots_parse_as_their_kind() {
        // Structural guard independent of the value table: every non-empty
        // color slot parses as oklch, and the radius slot holds a rem value.
        use crate::theme::colors::{slot, Oklch};
        for theme in THEMES {
            for (mode, slots) in [("light", theme.light), ("dark", theme.dark)] {
                for (i, value) in slots.iter().enumerate().take(slot::SIDEBAR_RING + 1) {
                    let v = *value;
                    if v.is_empty() {
                        continue;
                    }
                    if i == slot::RADIUS {
                        assert!(
                            v.ends_with("rem"),
                            "{} {mode} radius slot holds non-radius value {v:?}",
                            theme.name
                        );
                    } else {
                        assert!(
                            Oklch::parse(v).is_some(),
                            "{} {mode} slot {i} holds unparseable value {v:?}",
                            theme.name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn merged_slots_apply_mode_defaults_for_globals_css_tokens() {
        // globals.css values, light and dark, via the default (neutral) preset.
        use crate::theme::colors::slot;
        let p = Preset::default_preset();
        let light = p.slots(crate::theme::Mode::Light);
        assert_eq!(light[slot::DESTRUCTIVE_FOREGROUND], "oklch(0.97 0.01 17)");
        assert_eq!(light[slot::CODE], "oklch(0.98 0 0)");
        assert_eq!(light[slot::CODE_FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::SURFACE], "oklch(0.98 0 0)");
        assert_eq!(light[slot::SELECTION], "oklch(0% 0 0)");
        let dark = p.slots(crate::theme::Mode::Dark);
        assert_eq!(dark[slot::DESTRUCTIVE_FOREGROUND], "oklch(0.58 0.22 27)");
        assert_eq!(dark[slot::CODE], "oklch(0.2 0 0)");
        assert_eq!(dark[slot::CODE_FOREGROUND], "oklch(0.708 0 0)");
        assert_eq!(dark[slot::SURFACE], "oklch(0.2 0 0)");
        assert_eq!(dark[slot::SELECTION], "oklch(0.922 0 0)");
        // Accent overrides must not clobber the mode defaults: blue carries no
        // destructive-foreground/code slots, so the defaults survive merging.
        let blue = Preset::default_preset().with_theme_color(ThemeColor::Blue);
        let blue_dark = blue.slots(crate::theme::Mode::Dark);
        assert_eq!(
            blue_dark[slot::DESTRUCTIVE_FOREGROUND],
            "oklch(0.58 0.22 27)"
        );
        assert_eq!(blue_dark[slot::CODE], "oklch(0.2 0 0)");
        // Resolved palette: the light code background is near-white, not the
        // black fallback the empty slots used to produce.
        let theme = p.build(crate::theme::Mode::Light);
        assert!(
            theme.palette.code.r() > 200,
            "light code bg {:?}",
            theme.palette.code
        );
        assert!(
            theme.palette.code_foreground.r() < 60,
            "light code fg {:?}",
            theme.palette.code_foreground
        );
    }

    #[test]
    fn merged_slots_carry_corrected_ring_chart_sidebar_values() {
        // Spot values that the one-position shift corrupted (neutral default).
        use crate::theme::colors::slot;
        let p = Preset::default_preset();
        let light = p.slots(crate::theme::Mode::Light);
        assert_eq!(light[slot::RING], "oklch(0.708 0 0)");
        assert_eq!(light[slot::CHART_1], "oklch(0.87 0 0)");
        assert_eq!(light[slot::CHART_5], "oklch(0.269 0 0)");
        assert_eq!(light[slot::SIDEBAR], "oklch(0.985 0 0)");
        assert_eq!(light[slot::SIDEBAR_RING], "oklch(0.708 0 0)");
        assert_eq!(light[slot::RADIUS], "0.625rem");
        let dark = p.slots(crate::theme::Mode::Dark);
        assert_eq!(dark[slot::BORDER], "oklch(1 0 0 / 10%)");
        assert_eq!(dark[slot::INPUT], "oklch(1 0 0 / 15%)");
        assert_eq!(dark[slot::SIDEBAR_FOREGROUND], "oklch(0.985 0 0)");
        assert_eq!(dark[slot::SIDEBAR_PRIMARY], "oklch(0.488 0.243 264.376)");
    }

    #[test]
    fn light_surface_foreground_follows_active_theme_foreground() {
        // globals.css light mode: --surface-foreground: var(--foreground)
        // and --code-foreground: var(--surface-foreground), so both resolve
        // through the ACTIVE theme's foreground. Expected values are the
        // themes.ts light foregrounds per base color — except neutral and
        // gray, which resolve to the docs-site value `oklch(0% 0 0)` per
        // the FND-D1 override (globals.css wins).
        use crate::theme::colors::slot;
        let cases = [
            (BaseColor::Neutral, "oklch(0% 0 0)"),
            (BaseColor::Stone, "oklch(0.147 0.004 49.25)"),
            (BaseColor::Zinc, "oklch(0.141 0.005 285.823)"),
            // "gray" is not a registry theme; it falls back to neutral.
            (BaseColor::Gray, "oklch(0% 0 0)"),
            (BaseColor::Mauve, "oklch(0.145 0.008 326)"),
            (BaseColor::Olive, "oklch(0.153 0.006 107.1)"),
            (BaseColor::Mist, "oklch(0.148 0.004 228.8)"),
            (BaseColor::Taupe, "oklch(0.147 0.004 49.3)"),
        ];
        for (base, expected) in cases {
            let preset = Preset::default_preset().with_base_color(base);
            let light = preset.slots(crate::theme::Mode::Light);
            assert_eq!(light[slot::FOREGROUND], expected, "{base:?} foreground");
            assert_eq!(
                light[slot::SURFACE_FOREGROUND],
                expected,
                "{base:?} surface-foreground"
            );
            assert_eq!(
                light[slot::CODE_FOREGROUND],
                light[slot::SURFACE_FOREGROUND],
                "{base:?} code-foreground"
            );
        }
    }

    #[test]
    fn dark_surface_foreground_unchanged_across_bases() {
        // globals.css dark --surface-foreground is a verbatim value, not a
        // var() reference, so every base keeps the same dark defaults.
        use crate::theme::colors::slot;
        for base in [
            BaseColor::Neutral,
            BaseColor::Stone,
            BaseColor::Zinc,
            BaseColor::Gray,
            BaseColor::Mauve,
            BaseColor::Olive,
            BaseColor::Mist,
            BaseColor::Taupe,
        ] {
            let preset = Preset::default_preset().with_base_color(base);
            let dark = preset.slots(crate::theme::Mode::Dark);
            assert_eq!(
                dark[slot::SURFACE_FOREGROUND],
                "oklch(0.708 0 0)",
                "{base:?} dark surface-foreground"
            );
            assert_eq!(
                dark[slot::CODE_FOREGROUND],
                "oklch(0.708 0 0)",
                "{base:?} dark code-foreground"
            );
        }
    }

    #[test]
    fn light_surface_foreground_follows_merged_foreground_with_accent() {
        // Accent themes carry no foreground key, so the merged foreground is
        // the base theme's; the light var() chain must follow it. Blue
        // accent over a stone base resolves to stone's foreground, while the
        // accent still overrides its own slots (primary).
        use crate::theme::colors::slot;
        let preset = Preset::default_preset()
            .with_base_color(BaseColor::Stone)
            .with_theme_color(ThemeColor::Blue);
        let light = preset.slots(crate::theme::Mode::Light);
        assert_eq!(light[slot::PRIMARY], "oklch(0.488 0.243 264.376)");
        assert_eq!(light[slot::FOREGROUND], "oklch(0.147 0.004 49.25)");
        assert_eq!(light[slot::SURFACE_FOREGROUND], "oklch(0.147 0.004 49.25)");
        assert_eq!(light[slot::CODE_FOREGROUND], "oklch(0.147 0.004 49.25)");
        let dark = preset.slots(crate::theme::Mode::Dark);
        assert_eq!(dark[slot::SURFACE_FOREGROUND], "oklch(0.708 0 0)");
        assert_eq!(dark[slot::CODE_FOREGROUND], "oklch(0.708 0 0)");
    }

    #[test]
    fn dark_neutral_accent_resolves_to_globals_css_value() {
        // SEL-VIS-03, coordinator ruling 2026-11-15 (Position A): the dark
        // neutral accent resolves to the docs-site value oklch(0.371 0 0)
        // (snapshot globals.css:156), not the registry data's 0.269. The
        // `gray` fallback resolves through neutral and inherits it; the
        // light neutral accent is untouched.
        use crate::theme::colors::slot;
        let p = Preset::default_preset();
        assert_eq!(
            p.slots(crate::theme::Mode::Dark)[slot::ACCENT],
            "oklch(0.371 0 0)"
        );
        assert_eq!(
            p.slots(crate::theme::Mode::Light)[slot::ACCENT],
            "oklch(0.97 0 0)"
        );
        let gray = Preset::default_preset().with_base_color(BaseColor::Gray);
        assert_eq!(
            gray.slots(crate::theme::Mode::Dark)[slot::ACCENT],
            "oklch(0.371 0 0)"
        );
    }

    #[test]
    fn dark_accent_unchanged_for_other_base_colors() {
        // The SEL-VIS-03 override is neutral-only: every other base keeps
        // its themes.ts dark accent verbatim.
        use crate::theme::colors::slot;
        let cases = [
            (BaseColor::Stone, "oklch(0.268 0.007 34.298)"),
            (BaseColor::Zinc, "oklch(0.274 0.006 286.033)"),
            (BaseColor::Mauve, "oklch(0.263 0.024 320.12)"),
            (BaseColor::Olive, "oklch(0.286 0.016 107.4)"),
            (BaseColor::Mist, "oklch(0.275 0.011 216.9)"),
            (BaseColor::Taupe, "oklch(0.268 0.011 36.5)"),
        ];
        for (base, expected) in cases {
            let preset = Preset::default_preset().with_base_color(base);
            assert_eq!(
                preset.slots(crate::theme::Mode::Dark)[slot::ACCENT],
                expected,
                "{base:?} dark accent"
            );
        }
    }

    #[test]
    fn dark_neutral_accent_override_survives_accent_theme_overlay() {
        // Sparse accent themes carry no accent slot, so the accent still
        // resolves from the base — and the neutral-dark override applies
        // after merging, while the overlay still overrides its own slots.
        use crate::theme::colors::slot;
        let preset = Preset::default_preset()
            .with_base_color(BaseColor::Neutral)
            .with_theme_color(ThemeColor::Blue);
        let dark = preset.slots(crate::theme::Mode::Dark);
        assert_eq!(dark[slot::ACCENT], "oklch(0.371 0 0)");
        assert_eq!(dark[slot::PRIMARY], "oklch(0.424 0.199 265.638)");
    }

    #[test]
    fn light_neutral_foregrounds_resolve_to_globals_css_values() {
        // FND-D1, coordinator ruling 2026-11-24 (Expert Review decision:
        // globals.css wins): the neutral light foreground, card-foreground,
        // popover-foreground and primary resolve to the docs-site value
        // `oklch(0% 0 0)` (snapshot globals.css:102-107), not the registry
        // data's 0.145 / 0.205. The var() chain carries the overridden
        // foreground into surface-foreground / code-foreground. Dark is
        // unchanged, and the `gray` fallback resolves through neutral so it
        // inherits the override.
        use crate::theme::colors::slot;
        let p = Preset::default_preset();
        let light = p.slots(crate::theme::Mode::Light);
        assert_eq!(light[slot::FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::CARD_FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::POPOVER_FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::PRIMARY], "oklch(0% 0 0)");
        assert_eq!(light[slot::SURFACE_FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::CODE_FOREGROUND], "oklch(0% 0 0)");
        let dark = p.slots(crate::theme::Mode::Dark);
        assert_eq!(dark[slot::FOREGROUND], "oklch(0.985 0 0)");
        assert_eq!(dark[slot::CARD_FOREGROUND], "oklch(0.985 0 0)");
        assert_eq!(dark[slot::POPOVER_FOREGROUND], "oklch(0.985 0 0)");
        assert_eq!(dark[slot::PRIMARY], "oklch(0.922 0 0)");
        let gray = Preset::default_preset().with_base_color(BaseColor::Gray);
        let gray_light = gray.slots(crate::theme::Mode::Light);
        assert_eq!(gray_light[slot::FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(gray_light[slot::PRIMARY], "oklch(0% 0 0)");
    }

    #[test]
    fn light_neutral_primary_override_yields_to_accent_theme() {
        // Chromatic accent themes carry a `primary` key but no foreground
        // keys, so the foreground override still applies under an accent
        // overlay while the accent's own primary wins.
        use crate::theme::colors::slot;
        let preset = Preset::default_preset()
            .with_base_color(BaseColor::Neutral)
            .with_theme_color(ThemeColor::Blue);
        let light = preset.slots(crate::theme::Mode::Light);
        assert_eq!(light[slot::FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::CARD_FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::POPOVER_FOREGROUND], "oklch(0% 0 0)");
        assert_eq!(light[slot::PRIMARY], "oklch(0.488 0.243 264.376)");
    }

    #[test]
    fn light_foregrounds_unchanged_for_other_base_colors() {
        // The FND-D1 override is neutral-only: every other base keeps its
        // themes.ts light foreground and primary verbatim.
        use crate::theme::colors::slot;
        let cases = [
            (
                BaseColor::Stone,
                "oklch(0.147 0.004 49.25)",
                "oklch(0.216 0.006 56.043)",
            ),
            (
                BaseColor::Zinc,
                "oklch(0.141 0.005 285.823)",
                "oklch(0.21 0.006 285.885)",
            ),
            (
                BaseColor::Mauve,
                "oklch(0.145 0.008 326)",
                "oklch(0.212 0.019 322.12)",
            ),
            (
                BaseColor::Olive,
                "oklch(0.153 0.006 107.1)",
                "oklch(0.228 0.013 107.4)",
            ),
            (
                BaseColor::Mist,
                "oklch(0.148 0.004 228.8)",
                "oklch(0.218 0.008 223.9)",
            ),
            (
                BaseColor::Taupe,
                "oklch(0.147 0.004 49.3)",
                "oklch(0.214 0.009 43.1)",
            ),
        ];
        for (base, foreground, primary) in cases {
            let preset = Preset::default_preset().with_base_color(base);
            let light = preset.slots(crate::theme::Mode::Light);
            assert_eq!(light[slot::FOREGROUND], foreground, "{base:?} foreground");
            assert_eq!(light[slot::PRIMARY], primary, "{base:?} primary");
        }
    }
}
