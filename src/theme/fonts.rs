//! Embedded fonts: Inter (Regular/Medium/SemiBold) + Geist Mono (Regular/Medium).
//!
//! The original shadcn/ui renders almost all control text in `font-medium`
//! (nova uses it in 40 rules) and headings/labels in `font-semibold`, but this
//! crate previously installed no font at all: egui's single-weight default
//! fonts rendered everything, and the preset's `"inter"` font name was a
//! string that was never loaded. This module closes that gap: the fonts are
//! embedded in the binary, installed by [`ShadcnTheme::install`] (via
//! [`Preset::install`]) with no extra setup, and weights are reachable through
//! named families because egui 0.37 [`FontId`] has no weight field.
//!
//! # Weight mechanism (the API components use)
//!
//! egui selects a weight by family name, so each weight is its own family:
//!
//! | Helper | Family | Weight | Use |
//! |---|---|---|---|
//! | [`regular`] / [`FontId::proportional`] | `Proportional` | Inter 400 | Body text |
//! | [`medium`] | `Inter Medium` | Inter 500 | Buttons, labels, menu items (`font-medium`) |
//! | [`semibold`] | `Inter SemiBold` | Inter 600 | Headings, titles (`font-semibold`) |
//! | [`mono`] / [`FontId::monospace`] | `Monospace` | Geist Mono 400 | Code, `kbd`, tabular values |
//! | [`mono_medium`] | `Geist Mono Medium` | Geist Mono 500 | `font-mono font-medium` (kbd, chart tooltips) |
//!
//! Component code in the Pilot/Waves selects weights with these helpers, e.g.
//! `fonts::medium(14.0)` for a button label. [`RichText::strong`] does NOT
//! select a weight in egui — it only changes the text color — so it must not
//! be used as a `font-medium` substitute. Each weight family falls back to
//! the lighter weights and then to the host's default fonts, so glyphs the
//! subsets do not cover (Arabic, CJK, emoji) render exactly as they did
//! before these fonts were embedded.
//!
//! # What is embedded, and where it came from
//!
//! All files are SIL Open Font License 1.1; the full licence texts live
//! beside the files in `src/theme/fonts/` (`LICENSE-Inter.txt`,
//! `LICENSE-GeistMono.txt`).
//!
//! * Inter 4.1, static instances instantiated from the official variable
//!   font in `Inter-4.1.zip`
//!   (<https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip>):
//!   `wght` 400/500/600, `opsz` pinned to 14 (the text optical size; the
//!   Display variant is for large sizes). Copyright 2016 The Inter Project
//!   Authors.
//! * Geist Mono, static instances instantiated from the official variable
//!   font in the `google/fonts` repository (`ofl/geistmono/GeistMono[wght].ttf`,
//!   variable `wght` 100–900): `wght` 400/500. Copyright 2024 The Geist
//!   Project Authors. Geist Mono is the original's mono default: the
//!   registry's `--font-mono` is an app-supplied variable and the shadcn/ui
//!   defaults pair Inter (sans/heading) with Geist Mono (mono); the snapshot
//!   `globals.css` pins no mono family itself.
//!
//! # Subsets and size
//!
//! Each instance is subset with `fonttools` (`pyftsubset`) to: Basic Latin,
//! Latin-1 Supplement, Latin Extended-A/B, Latin Extended Additional, Greek,
//! Cyrillic, General Punctuation, super/subscripts, Currency Symbols, Arrows,
//! Mathematical Operators, Miscellaneous Technical, Box Drawing, Geometric
//! Shapes, Miscellaneous Symbols, Dingbats, Miscellaneous Symbols and
//! Arrows, and U+FFFD. The subset drops nothing the crate or gallery uses:
//! every non-ASCII codepoint in `src/` and `examples/` that the full fonts
//! cover survives subsetting (verified against the full variable fonts with
//! `fontTools`). Total embedded size is [`EMBEDDED_BYTES`] bytes, versus
//! ~1.23 MB for the five full static instances.
//!
//! To regenerate (requires `fonttools`, run from the directory holding the
//! upstream downloads; the exact script used is kept with the Phase 1
//! evidence as `font-sources/build_fonts.sh`):
//!
//! ```sh
//! python3 -m fontTools.varLib.instancer InterVariable.ttf wght=400 opsz=14 \
//!   -o Inter-Regular.ttf --update-name-table
//! python3 -m fontTools.varLib.instancer "GeistMono[wght].ttf" wght=400 \
//!   -o GeistMono-Regular.ttf --update-name-table
//! python3 -m fontTools.subset Inter-Regular.ttf --output-file=Inter-Regular-sub.ttf \
//!   --unicodes="U+0020-007E,U+00A0-00FF,U+0100-017F,U+0180-024F,U+1E00-1EFF,U+0370-03FF,U+0400-04FF,U+2000-206F,U+2070-209F,U+20A0-20BF,U+2190-21FF,U+2200-22FF,U+2300-23FF,U+2500-257F,U+25A0-25FF,U+2600-26FF,U+2700-27BF,U+2B00-2BFF,U+FFFD" \
//!   --layout-features='' --no-hinting --desubroutinize
//! ```
//!
//! # Feature flag
//!
//! The `fonts` cargo feature (on by default) gates embedding: with
//! `default-features = false`, no font bytes are embedded, [`install`] still
//! registers the named weight families (aliased to the host's default
//! chains, so weight helpers degrade to regular-weight rendering instead of
//! referencing missing families), and text renders with whatever fonts the
//! host installed. `Preset::install` remains the single setup call either way.
//!
//! # Copy-paste users
//!
//! Component source copied shadcn-style into another project does not carry
//! these bytes: a copy-paste user must either depend on this crate with the
//! `fonts` feature and call [`Preset::install`], or copy `src/theme/fonts/`
//! (including the licence files) plus this module and call [`install`].
//! `Preset::font` names other fonts (`noto-sans`, `figtree`, …) in preset
//! codes, but only Inter is embedded today; every preset installs Inter
//! (previously no font was ever loaded, so this strictly improves
//! rendering). Per-font embedding is a follow-up, not part of this module.
//!
//! [`ShadcnTheme::install`]: super::ShadcnTheme::install
//! [`Preset::install`]: super::presets::Preset::install
//! [`FontId`]: egui::FontId
//! [`RichText::strong`]: egui::RichText::strong

#[cfg(feature = "fonts")]
use std::sync::Arc;

#[cfg(feature = "fonts")]
use egui::FontData;
use egui::{FontDefinitions, FontFamily, FontId};

/// Font-data key and primary proportional face: Inter Regular (400).
pub const INTER_FAMILY: &str = "Inter";
/// Named family for Inter Medium (500), i.e. `font-medium`.
pub const INTER_MEDIUM_FAMILY: &str = "Inter Medium";
/// Named family for Inter SemiBold (600), i.e. `font-semibold`.
pub const INTER_SEMIBOLD_FAMILY: &str = "Inter SemiBold";
/// Font-data key and primary monospace face: Geist Mono Regular (400).
pub const MONO_FAMILY: &str = "Geist Mono";
/// Named family for Geist Mono Medium (500), i.e. `font-mono font-medium`.
pub const MONO_MEDIUM_FAMILY: &str = "Geist Mono Medium";

/// Embedded Inter Regular (400), subset; see the module docs for provenance.
#[cfg(feature = "fonts")]
pub const INTER_REGULAR: &[u8] = include_bytes!("fonts/Inter-Regular.ttf");
/// Embedded Inter Medium (500), subset; see the module docs for provenance.
#[cfg(feature = "fonts")]
pub const INTER_MEDIUM: &[u8] = include_bytes!("fonts/Inter-Medium.ttf");
/// Embedded Inter SemiBold (600), subset; see the module docs for provenance.
#[cfg(feature = "fonts")]
pub const INTER_SEMIBOLD: &[u8] = include_bytes!("fonts/Inter-SemiBold.ttf");
/// Embedded Geist Mono Regular (400), subset; see the module docs for provenance.
#[cfg(feature = "fonts")]
pub const MONO_REGULAR: &[u8] = include_bytes!("fonts/GeistMono-Regular.ttf");
/// Embedded Geist Mono Medium (500), subset; see the module docs for provenance.
#[cfg(feature = "fonts")]
pub const MONO_MEDIUM: &[u8] = include_bytes!("fonts/GeistMono-Medium.ttf");

/// Total embedded font bytes (0 when the `fonts` feature is off).
#[cfg(feature = "fonts")]
pub const EMBEDDED_BYTES: usize = INTER_REGULAR.len()
    + INTER_MEDIUM.len()
    + INTER_SEMIBOLD.len()
    + MONO_REGULAR.len()
    + MONO_MEDIUM.len();
/// Total embedded font bytes (0 when the `fonts` feature is off).
#[cfg(not(feature = "fonts"))]
pub const EMBEDDED_BYTES: usize = 0;

/// Inter Regular at `size` — the default body face.
pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Inter Medium (500) at `size` — `font-medium`: buttons, labels, menu items.
pub fn medium(size: f32) -> FontId {
    FontId::new(size, medium_family())
}

/// Inter SemiBold (600) at `size` — `font-semibold`: headings, titles.
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, semibold_family())
}

/// Geist Mono Regular (400) at `size` — code, `kbd`, tabular values.
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// Geist Mono Medium (500) at `size` — `font-mono font-medium`.
pub fn mono_medium(size: f32) -> FontId {
    FontId::new(size, mono_medium_family())
}

/// The `Inter Medium` named family.
pub fn medium_family() -> FontFamily {
    FontFamily::Name(INTER_MEDIUM_FAMILY.into())
}

/// The `Inter SemiBold` named family.
pub fn semibold_family() -> FontFamily {
    FontFamily::Name(INTER_SEMIBOLD_FAMILY.into())
}

/// The `Geist Mono Medium` named family.
pub fn mono_medium_family() -> FontFamily {
    FontFamily::Name(MONO_MEDIUM_FAMILY.into())
}

/// Builds the [`FontDefinitions`] this crate installs: egui's defaults plus
/// the embedded faces prepended to their chains and the named weight
/// families. Pure (no [`egui::Context`] needed) so tests can inspect it.
///
/// With the `fonts` feature off, no font data is added, but the named
/// weight families are still registered, aliased to the default chains, so
/// weight helpers degrade to regular-weight rendering.
pub fn font_definitions() -> FontDefinitions {
    let mut defs = FontDefinitions::default();
    let proportional_fallback = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let monospace_fallback = defs
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();

    #[cfg(feature = "fonts")]
    {
        defs.font_data.insert(
            INTER_FAMILY.to_owned(),
            Arc::new(FontData::from_static(INTER_REGULAR)),
        );
        defs.font_data.insert(
            INTER_MEDIUM_FAMILY.to_owned(),
            Arc::new(FontData::from_static(INTER_MEDIUM)),
        );
        defs.font_data.insert(
            INTER_SEMIBOLD_FAMILY.to_owned(),
            Arc::new(FontData::from_static(INTER_SEMIBOLD)),
        );
        defs.font_data.insert(
            MONO_FAMILY.to_owned(),
            Arc::new(FontData::from_static(MONO_REGULAR)),
        );
        defs.font_data.insert(
            MONO_MEDIUM_FAMILY.to_owned(),
            Arc::new(FontData::from_static(MONO_MEDIUM)),
        );

        defs.families.insert(
            FontFamily::Proportional,
            chain(&[INTER_FAMILY], &proportional_fallback),
        );
        defs.families.insert(
            FontFamily::Monospace,
            chain(&[MONO_FAMILY], &monospace_fallback),
        );
        defs.families.insert(
            medium_family(),
            chain(&[INTER_MEDIUM_FAMILY, INTER_FAMILY], &proportional_fallback),
        );
        defs.families.insert(
            semibold_family(),
            chain(
                &[INTER_SEMIBOLD_FAMILY, INTER_FAMILY],
                &proportional_fallback,
            ),
        );
        defs.families.insert(
            mono_medium_family(),
            chain(&[MONO_MEDIUM_FAMILY, MONO_FAMILY], &monospace_fallback),
        );
    }

    #[cfg(not(feature = "fonts"))]
    {
        defs.families
            .insert(medium_family(), proportional_fallback.clone());
        defs.families
            .insert(semibold_family(), proportional_fallback);
        defs.families
            .insert(mono_medium_family(), monospace_fallback);
    }

    defs
}

/// Installs [`font_definitions`] into the context, replacing its fonts.
/// Called by [`ShadcnTheme::install`], so [`Preset::install`] is the only
/// setup call a user needs. Install before adding other custom fonts: like
/// egui's own documented pattern, this starts from [`FontDefinitions::default`]
/// and therefore replaces any fonts installed earlier on this context.
///
/// [`ShadcnTheme::install`]: super::ShadcnTheme::install
/// [`Preset::install`]: super::presets::Preset::install
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}

/// Maps egui's [`TextStyle`]s onto the shadcn type scale and weights, so
/// plain egui widgets (labels, buttons, text edits) match the theme. Sizes
/// come from [`type_scale`]; only the families/weights differ from egui's
/// defaults for `Button` (Inter Medium, like the source's `font-medium`
/// buttons) and `Heading` (Inter SemiBold). Components that paint text
/// manually use [`type_scale`] sizes with the weight helpers directly.
///
/// [`type_scale`]: super::tokens::type_scale
pub fn apply_text_styles(style: &mut egui::Style) {
    use super::tokens::type_scale;
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(type_scale::XS));
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(type_scale::SM));
    style
        .text_styles
        .insert(egui::TextStyle::Button, medium(type_scale::SM));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, semibold(type_scale::XL));
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        FontId::monospace(type_scale::SM),
    );
}

/// `primary` faces followed by the `fallback` chain, without duplicates.
#[cfg(feature = "fonts")]
fn chain(primary: &[&str], fallback: &[String]) -> Vec<String> {
    let mut out: Vec<String> = primary.iter().map(|name| (*name).to_owned()).collect();
    for name in fallback {
        if !out.contains(name) {
            out.push(name.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs one pass and drains the texture delta, which must be cleared
    /// before the output is dropped.
    fn run(ctx: &egui::Context, f: impl FnMut(&mut egui::Ui)) {
        let mut out = ctx.run_ui(egui::RawInput::default(), f);
        out.textures_delta.clear();
    }

    #[test]
    #[cfg(feature = "fonts")]
    fn embedded_data_is_nonempty_truetype() {
        // TrueType scaler tag: 0x00010000.
        for data in [
            INTER_REGULAR,
            INTER_MEDIUM,
            INTER_SEMIBOLD,
            MONO_REGULAR,
            MONO_MEDIUM,
        ] {
            assert!(!data.is_empty());
            assert_eq!(&data[..4], &[0x00, 0x01, 0x00, 0x00]);
        }
        assert_eq!(
            EMBEDDED_BYTES,
            INTER_REGULAR.len()
                + INTER_MEDIUM.len()
                + INTER_SEMIBOLD.len()
                + MONO_REGULAR.len()
                + MONO_MEDIUM.len()
        );
        const _: () = assert!(EMBEDDED_BYTES > 0);
    }

    #[test]
    #[cfg(not(feature = "fonts"))]
    fn embedded_bytes_is_zero_without_feature() {
        assert_eq!(EMBEDDED_BYTES, 0);
    }

    #[test]
    #[cfg(feature = "fonts")]
    fn definitions_contain_embedded_families() {
        let defs = font_definitions();
        for key in [
            INTER_FAMILY,
            INTER_MEDIUM_FAMILY,
            INTER_SEMIBOLD_FAMILY,
            MONO_FAMILY,
            MONO_MEDIUM_FAMILY,
        ] {
            assert!(defs.font_data.contains_key(key), "missing font data {key}");
        }
        assert_eq!(
            defs.families[&FontFamily::Proportional].first().unwrap(),
            INTER_FAMILY
        );
        assert_eq!(
            defs.families[&FontFamily::Monospace].first().unwrap(),
            MONO_FAMILY
        );
        assert_eq!(
            defs.families[&medium_family()].first().unwrap(),
            INTER_MEDIUM_FAMILY
        );
        assert_eq!(
            defs.families[&semibold_family()].first().unwrap(),
            INTER_SEMIBOLD_FAMILY
        );
        assert_eq!(
            defs.families[&mono_medium_family()].first().unwrap(),
            MONO_MEDIUM_FAMILY
        );
        // Weight families fall back through the lighter weight, then the
        // host defaults (egui-icons last, as in FontDefinitions::default).
        assert!(defs.families[&medium_family()].contains(&INTER_FAMILY.to_owned()));
        assert!(
            defs.families[&medium_family()]
                .last()
                .unwrap()
                .contains("egui-icons")
                || defs.families[&medium_family()].contains(&"Ubuntu-Light".to_owned())
        );
    }

    #[test]
    #[cfg(not(feature = "fonts"))]
    fn definitions_degrade_without_feature() {
        let defs = font_definitions();
        for key in [
            INTER_FAMILY,
            INTER_MEDIUM_FAMILY,
            INTER_SEMIBOLD_FAMILY,
            MONO_FAMILY,
            MONO_MEDIUM_FAMILY,
        ] {
            assert!(
                !defs.font_data.contains_key(key),
                "unexpected font data {key}"
            );
        }
        // Named families still exist so weight helpers never reference a
        // missing family; they alias the default chains.
        assert!(defs.families.contains_key(&medium_family()));
        assert!(defs.families.contains_key(&semibold_family()));
        assert!(defs.families.contains_key(&mono_medium_family()));
    }

    #[test]
    fn weight_helpers_select_named_families() {
        assert_eq!(medium(14.0).family, medium_family());
        assert_eq!(semibold(14.0).family, semibold_family());
        assert_eq!(mono_medium(14.0).family, mono_medium_family());
        assert_eq!(regular(14.0).family, FontFamily::Proportional);
        assert_eq!(mono(14.0).family, FontFamily::Monospace);
        assert_eq!(medium(14.0).size, 14.0);
    }

    #[test]
    fn text_styles_use_inter_weights_and_type_scale() {
        let mut style = egui::Style::default();
        apply_text_styles(&mut style);
        let styles = &style.text_styles;
        assert_eq!(
            styles[&egui::TextStyle::Body],
            FontId::proportional(crate::theme::tokens::type_scale::SM)
        );
        assert_eq!(
            styles[&egui::TextStyle::Button],
            medium(crate::theme::tokens::type_scale::SM)
        );
        assert_eq!(
            styles[&egui::TextStyle::Heading],
            semibold(crate::theme::tokens::type_scale::XL)
        );
        assert_eq!(
            styles[&egui::TextStyle::Small],
            FontId::proportional(crate::theme::tokens::type_scale::XS)
        );
        assert_eq!(
            styles[&egui::TextStyle::Monospace],
            FontId::monospace(crate::theme::tokens::type_scale::SM)
        );
    }

    #[test]
    #[cfg(feature = "fonts")]
    fn install_wires_fonts_and_glyphs_into_context() {
        let ctx = egui::Context::default();
        // Negative control: before install, the default proportional chain
        // (Ubuntu-Light) has no arrow glyph.
        run(&ctx, |ui| {
            ui.fonts_mut(|fonts| {
                assert!(!fonts.has_glyph(&FontId::proportional(14.0), '→'));
            });
        });

        install(&ctx);

        run(&ctx, |ui| {
            ui.fonts_mut(|fonts| {
                // Inter covers the symbols the crate uses in proportional text.
                for (id, label) in [
                    (FontId::proportional(14.0), "regular"),
                    (medium(14.0), "medium"),
                    (semibold(14.0), "semibold"),
                ] {
                    for c in ['A', 'é', '→', '⌘', 'π', '•', '×', '—', '…'] {
                        assert!(fonts.has_glyph(&id, c), "{label} lacks {c:?}");
                    }
                }
                for id in [FontId::monospace(14.0), mono_medium(14.0)] {
                    assert!(fonts.has_glyphs(&id, "const x = 42; // →"));
                }
                // Distinct weights rasterize at distinct widths: Medium and
                // SemiBold advances are wider than Regular at the same size.
                let w_regular = fonts.glyph_width(&FontId::proportional(14.0), 'A');
                let w_medium = fonts.glyph_width(&medium(14.0), 'A');
                let w_semibold = fonts.glyph_width(&semibold(14.0), 'A');
                assert!(w_medium > w_regular, "{w_medium} <= {w_regular}");
                assert!(w_semibold > w_medium, "{w_semibold} <= {w_medium}");
            });
        });
    }

    #[test]
    fn preset_install_wires_fonts() {
        // The documented setup path installs fonts: Preset::install ->
        // ShadcnTheme::install -> apply_visuals -> fonts::install.
        let ctx = egui::Context::default();
        crate::theme::presets::Preset::default().install(&ctx);
        run(&ctx, |ui| {
            ui.fonts_mut(|fonts| {
                assert!(fonts.has_glyph(&medium(14.0), 'A'));
                assert!(fonts.has_glyph(&semibold(14.0), 'A'));
                assert!(fonts.has_glyph(&mono_medium(14.0), 'A'));
            });
        });
        // Text-style wiring by the same install path is covered by
        // `text_styles_use_inter_weights_and_type_scale`.
    }
}
