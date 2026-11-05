//! OKLCH color values and conversion to `egui::Color32`.
//!
//! shadcn/ui v4 defines every design token in OKLCH. We keep the original
//! `oklch(...)` strings as data (see [`crate::theme::theme_data`]) so the port
//! stays 1:1 traceable to the source, and convert to sRGB once when a theme is
//! installed into the [`egui::Context`].

use egui::Color32;

/// An OKLCH color: lightness, chroma, hue and alpha.
///
/// `parse`/`format` round-trip the CSS strings used by shadcn/ui, e.g.
/// `oklch(0.145 0 0)` or `oklch(1 0 0 / 10%)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklch {
    /// Lightness, 0.0 (black) .. 1.0 (white).
    pub l: f32,
    /// Chroma, 0.0 (gray) .. ~0.4 (vivid).
    pub c: f32,
    /// Hue in degrees, 0.0 .. 360.0.
    pub h: f32,
    /// Alpha, 0.0 .. 1.0.
    pub a: f32,
}

impl Oklch {
    /// A fully opaque OKLCH color.
    pub const fn new(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h, a: 1.0 }
    }

    /// A fully opaque OKLCH color, const form used in data tables.
    pub const fn opaque(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h, a: 1.0 }
    }

    /// Parses a CSS `oklch(L C H)` or `oklch(L C H / A)` string.
    ///
    /// Lightness and alpha may be percentages (`0%`..`100%`); hue may carry a
    /// `deg` suffix. Returns `None` on malformed input.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let inner = s.strip_prefix("oklch(")?.strip_suffix(')')?;
        let mut parts = inner.split('/');
        let lch = parts.next()?.split_whitespace().collect::<Vec<_>>();
        if lch.len() != 3 {
            return None;
        }
        let l = parse_number(lch[0])?; // 0..1 or percentage
        let c = parse_number(lch[1])?;
        let h = parse_number(lch[2].trim_end_matches("deg"))?;
        let a = match parts.next() {
            Some(alpha) => {
                let alpha = alpha.trim();
                if let Some(pct) = alpha.strip_suffix('%') {
                    parse_number(pct)? / 100.0
                } else {
                    parse_number(alpha)?
                }
            }
            None => 1.0,
        };
        Some(Self {
            l: l.clamp(0.0, 1.0),
            c: c.max(0.0),
            h: h.rem_euclid(360.0),
            a: a.clamp(0.0, 1.0),
        })
    }

    /// Converts to an sRGB [`Color32`].
    ///
    /// egui expects premultiplied alpha, so the RGB channels are scaled by
    /// alpha. Fully transparent colors keep the original hue channels (not
    /// black), which matters for translucent borders on dark backgrounds.
    pub fn to_color32(&self) -> Color32 {
        let (r, g, b) = oklch_to_srgb_linear(self.l, self.c, self.h);
        let encode = |x: f32| -> u8 {
            let x = x.clamp(0.0, 1.0);
            let v = if x <= 0.0031308 {
                12.92 * x
            } else {
                1.055 * x.powf(1.0 / 2.4) - 0.055
            };
            (v * 255.0 + 0.5) as u8
        };
        let (r, g, b) = (encode(r), encode(g), encode(b));
        if self.a >= 1.0 {
            Color32::from_rgb(r, g, b)
        } else {
            let a = (self.a * 255.0 + 0.5) as u8;
            Color32::from_rgba_premultiplied(
                ((r as u32 * a as u32) / 255) as u8,
                ((g as u32 * a as u32) / 255) as u8,
                ((b as u32 * a as u32) / 255) as u8,
                a,
            )
        }
    }

    /// Returns the same color with alpha set to `alpha`.
    pub fn with_alpha(&self, alpha: f32) -> Self {
        let mut out = *self;
        out.a = alpha.clamp(0.0, 1.0);
        out
    }
}

/// Parses `0.145`, `55%`, or `0` into an f32. Percentages are divided by 100.
fn parse_number(s: &str) -> Option<f32> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        pct.trim().parse::<f32>().ok().map(|v| v / 100.0)
    } else {
        s.parse::<f32>().ok()
    }
}

/// OKLCH -> linear sRGB via OKLab (Björn Ottosson's reference matrices).
fn oklch_to_srgb_linear(l: f32, c: f32, h: f32) -> (f32, f32, f32) {
    let (sin_h, cos_h) = (h.to_radians()).sin_cos();
    let a = c * cos_h;
    let b = c * sin_h;

    // OKLab -> LMS^T (cube root domain)
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_35 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // LMS -> linear sRGB
    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_93 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3;
    let b = -0.004_196_09 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;
    (r, g, b)
}

/// All named tokens for one side (light or dark) of a theme, as raw
/// `oklch(...)` strings. Field order matches [`SLOT_NAMES`]; slots that a
/// theme preset does not override are empty strings and fall back to the
/// base color (for accent presets) or the mode defaults.
pub type Slots = [&'static str; 41];

/// Canonical field order for [`Slots`], documented for maintainers.
///
/// `background, foreground, card, card-foreground, popover, popover-foreground,
/// primary, primary-foreground, secondary, secondary-foreground, muted,
/// muted-foreground, accent, accent-foreground, destructive,
/// destructive-foreground, border, input, ring, chart-1..chart-5, radius,
/// sidebar, sidebar-foreground, sidebar-primary, sidebar-primary-foreground,
/// sidebar-accent, sidebar-accent-foreground, sidebar-border, sidebar-ring,
/// surface, surface-foreground, code, code-foreground, code-highlight,
/// code-number, selection, selection-foreground`
pub const SLOT_NAMES: [&str; 41] = [
    "background",
    "foreground",
    "card",
    "card-foreground",
    "popover",
    "popover-foreground",
    "primary",
    "primary-foreground",
    "secondary",
    "secondary-foreground",
    "muted",
    "muted-foreground",
    "accent",
    "accent-foreground",
    "destructive",
    "destructive-foreground",
    "border",
    "input",
    "ring",
    "chart-1",
    "chart-2",
    "chart-3",
    "chart-4",
    "chart-5",
    "radius",
    "sidebar",
    "sidebar-foreground",
    "sidebar-primary",
    "sidebar-primary-foreground",
    "sidebar-accent",
    "sidebar-accent-foreground",
    "sidebar-border",
    "sidebar-ring",
    "surface",
    "surface-foreground",
    "code",
    "code-foreground",
    "code-highlight",
    "code-number",
    "selection",
    "selection-foreground",
];

/// Slot indexes into [`Slots`] / [`SLOT_NAMES`].
pub mod slot {
    pub const BACKGROUND: usize = 0;
    pub const FOREGROUND: usize = 1;
    pub const CARD: usize = 2;
    pub const CARD_FOREGROUND: usize = 3;
    pub const POPOVER: usize = 4;
    pub const POPOVER_FOREGROUND: usize = 5;
    pub const PRIMARY: usize = 6;
    pub const PRIMARY_FOREGROUND: usize = 7;
    pub const SECONDARY: usize = 8;
    pub const SECONDARY_FOREGROUND: usize = 9;
    pub const MUTED: usize = 10;
    pub const MUTED_FOREGROUND: usize = 11;
    pub const ACCENT: usize = 12;
    pub const ACCENT_FOREGROUND: usize = 13;
    pub const DESTRUCTIVE: usize = 14;
    pub const DESTRUCTIVE_FOREGROUND: usize = 15;
    pub const BORDER: usize = 16;
    pub const INPUT: usize = 17;
    pub const RING: usize = 18;
    pub const CHART_1: usize = 19;
    pub const CHART_2: usize = 20;
    pub const CHART_3: usize = 21;
    pub const CHART_4: usize = 22;
    pub const CHART_5: usize = 23;
    pub const RADIUS: usize = 24;
    pub const SIDEBAR: usize = 25;
    pub const SIDEBAR_FOREGROUND: usize = 26;
    pub const SIDEBAR_PRIMARY: usize = 27;
    pub const SIDEBAR_PRIMARY_FOREGROUND: usize = 28;
    pub const SIDEBAR_ACCENT: usize = 29;
    pub const SIDEBAR_ACCENT_FOREGROUND: usize = 30;
    pub const SIDEBAR_BORDER: usize = 31;
    pub const SIDEBAR_RING: usize = 32;
    pub const SURFACE: usize = 33;
    pub const SURFACE_FOREGROUND: usize = 34;
    pub const CODE: usize = 35;
    pub const CODE_FOREGROUND: usize = 36;
    pub const CODE_HIGHLIGHT: usize = 37;
    pub const CODE_NUMBER: usize = 38;
    pub const SELECTION: usize = 39;
    pub const SELECTION_FOREGROUND: usize = 40;
}

/// Mode-level defaults for slots that `themes.ts` does not carry per-theme;
/// values come verbatim from the source `globals.css`.
pub const MODE_DEFAULTS_LIGHT: [&str; 41] = {
    let mut slots = [""; 41];
    slots[slot::DESTRUCTIVE_FOREGROUND] = "oklch(0.97 0.01 17)";
    slots[slot::SURFACE] = "oklch(0.98 0 0)";
    slots[slot::SURFACE_FOREGROUND] = "oklch(0.145 0 0)";
    slots[slot::CODE_HIGHLIGHT] = "oklch(0.96 0 0)";
    slots[slot::CODE_NUMBER] = "oklch(0.56 0 0)";
    slots[slot::SELECTION] = "oklch(0% 0 0)";
    slots[slot::SELECTION_FOREGROUND] = "oklch(1 0 0)";
    slots
};

/// Dark-mode counterpart of [`MODE_DEFAULTS_LIGHT`].
pub const MODE_DEFAULTS_DARK: [&str; 41] = {
    let mut slots = [""; 41];
    slots[slot::DESTRUCTIVE_FOREGROUND] = "oklch(0.58 0.22 27)";
    slots[slot::SURFACE] = "oklch(0.2 0 0)";
    slots[slot::SURFACE_FOREGROUND] = "oklch(0.708 0 0)";
    slots[slot::CODE_HIGHLIGHT] = "oklch(0.27 0 0)";
    slots[slot::CODE_NUMBER] = "oklch(0.72 0 0)";
    slots[slot::SELECTION] = "oklch(0.922 0 0)";
    slots[slot::SELECTION_FOREGROUND] = "oklch(0.205 0 0)";
    slots
};

/// Raw `oklch(...)` values for one side of a theme after preset merging.
#[derive(Clone, Copy, Debug)]
pub struct ColorSpec {
    slots: Slots,
}

impl ColorSpec {
    /// Builds from merged slot strings.
    pub const fn from_slots(slots: Slots) -> Self {
        Self { slots }
    }

    /// Returns the raw string for one slot (may be empty).
    pub fn slot(&self, index: usize) -> &'static str {
        self.slots[index]
    }

    /// Parses slot `index` into an [`Oklch`]; empty/invalid slots fall back to
    /// `fallback`.
    pub fn oklch(&self, index: usize, fallback: Oklch) -> Oklch {
        Oklch::parse(self.slots[index]).unwrap_or(fallback)
    }

    /// Resolves slot `index` to a [`Color32`].
    pub fn color(&self, index: usize, fallback: Oklch) -> Color32 {
        self.oklch(index, fallback).to_color32()
    }

    /// The `--radius` token in CSS pixels (`"0.625rem"` -> 10.0).
    pub fn radius_px(&self, fallback: f32) -> f32 {
        let s = self.slots[slot::RADIUS].trim();
        if let Some(rem) = s.strip_suffix("rem") {
            rem.parse::<f32>().map(|v| v * 16.0).unwrap_or(fallback)
        } else if let Some(px) = s.strip_suffix("px") {
            px.parse::<f32>().unwrap_or(fallback)
        } else {
            fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_oklch() {
        let c = Oklch::parse("oklch(0.145 0 0)").unwrap();
        assert!((c.l - 0.145).abs() < 1e-6);
        assert_eq!(c.c, 0.0);
        assert_eq!(c.h, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn parses_percent_lightness_and_alpha() {
        let c = Oklch::parse("oklch(55% 0.2 30)").unwrap();
        assert!((c.l - 0.55).abs() < 1e-6);
        let c = Oklch::parse("oklch(1 0 0 / 10%)").unwrap();
        assert!((c.a - 0.10).abs() < 1e-6);
    }

    #[test]
    fn rejects_malformed() {
        assert!(Oklch::parse("oklch(0.145 0)").is_none());
        assert!(Oklch::parse("rgb(0 0 0)").is_none());
        assert!(Oklch::parse("oklch(a b c)").is_none());
    }

    #[test]
    fn neutral_white_is_white() {
        let c = Oklch::parse("oklch(1 0 0)").unwrap().to_color32();
        assert_eq!(c, Color32::WHITE);
    }

    #[test]
    fn neutral_black_is_black() {
        let c = Oklch::parse("oklch(0 0 0)").unwrap().to_color32();
        assert_eq!(c, Color32::BLACK);
    }

    #[test]
    fn dark_background_matches_reference() {
        // oklch(0.145 0 0) is shadcn's dark background, equivalent to Tailwind
        // neutral-950 = #0a0a0a.
        let c = Oklch::parse("oklch(0.145 0 0)").unwrap().to_color32();
        assert!(c.r() >= 8 && c.r() <= 12, "got {}", c.r());
        assert_eq!(c.r(), c.g());
        assert_eq!(c.g(), c.b());
    }

    #[test]
    fn red_destructive_is_reddish() {
        // oklch(0.577 0.245 27.325) — shadcn light destructive.
        let c = Oklch::parse("oklch(0.577 0.245 27.325)")
            .unwrap()
            .to_color32();
        assert!(c.r() > 150, "red channel {}", c.r());
        assert!(c.g() < 80, "green channel {}", c.g());
        assert!(c.b() < 80, "blue channel {}", c.b());
    }

    #[test]
    fn alpha_is_premultiplied() {
        let c = Oklch::parse("oklch(1 0 0 / 10%)").unwrap().to_color32();
        assert_eq!(c.a(), 26); // 0.1 * 255 = 25.5, rounded
        assert!(c.r() <= c.a());
    }

    #[test]
    fn radius_px_from_rem() {
        let mut slots = [""; 41];
        slots[slot::RADIUS] = "0.625rem";
        let spec = ColorSpec::from_slots(slots);
        assert!((spec.radius_px(8.0) - 10.0).abs() < 1e-4);
    }
}
