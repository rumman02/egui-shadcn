# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project has not yet had a release: `0.1.0` is under development
toward v1.0 (see the README for status), so everything below is
unreleased.

## [Unreleased]

### Added

- Embedded fonts: subsetted Inter (Regular, Medium, SemiBold, instantiated
  at optical size 14) and Geist Mono (Regular, Medium), 385,772 bytes
  total, installed automatically by `Preset::install`. New `fonts`
  cargo feature (on by default) controls the embedding, and new weight
  helpers (`medium`, `semibold`, `mono`, `mono_medium`) in
  `crate::theme::fonts` select weights through named font families.
  (`84f111e`)
- Color and opacity transition system in `crate::utils::animation`:
  `Easing` (including CSS cubic-bezier curves), `TransitionSpec`,
  `transition_color`, `transition_alpha`, `transition_progress`, and
  perceptual `mix_oklab` mixing, built on egui's value animator with
  repaint only while animating. (`1f9d09d`)
- Layered, texture-cached shadows in `crate::utils::shadow`:
  `paint_shadow_resolved` resolves the Tailwind v4 default shadow scale
  per slot and style under the layering rule (a style rule that is silent
  on shadows keeps the base recipe's shadow; an explicit shadow
  declaration overrides it). Legacy single-layer helpers are unchanged.
  (`db29161`)
- Real Lucide icon data (Lucide 1.72.0) for all 44 icons, rendered
  through a cached distance-field rasterizer instead of hand-drawn
  painter paths, with the SVG-to-geometry generator committed as
  `src/icons/gen_icons.py`. The existing `Icon` API is unchanged.
  (`5f9420f`)
- Copy-paste distribution route: `docs/copy-paste.md` documents the
  true file closure for copying a component (component file plus the
  `theme`/`utils`/`icons` foundation modules), with a tested Button
  worked example. (This change)
- Migration note for the `Arc<ShadcnTheme>` breaking change:
  `docs/migration-0.1.0.md`. (This change)
- crates.io package metadata: `repository`, `keywords`, and `categories`
  in `Cargo.toml`. (This change)

### Fixed

- Token slot shift: the theme generator now emits the full 41-slot
  `SLOT_NAMES` order (including `destructive-foreground`, which the
  registry never carries), so every slot from 15 onward is read from
  the correct position in all 24 registry themes, light and dark.
  Previously, ring, chart, sidebar, and radius-adjacent values were
  shifted by one slot. `code` / `code-foreground` mode defaults were
  added, fixing the inverted light-mode code colors. (`e17a27f`)
- Light-mode `surface-foreground` / `code-foreground` now resolve from
  the active theme's foreground (following the `globals.css` `var()`
  chain) instead of always resolving to neutral's foreground.
  (`a68e7e0`)
- The outline button now paints `shadow-xs` under every style, including
  nova. The previous `!nova` gate rested on reading nova's silence on
  shadows as removal; under the layering rule, silence keeps the base
  value. (`db29161`)
- Fresh clones now build: `examples/` and `docs/` are tracked again, so
  the gallery example declared in `Cargo.toml` exists on a fresh clone.
  (This change)
- The README no longer links to `PROGRESS.md`, which is not shipped and
  returned 404 on GitHub. Status is now stated in the README itself.
  (This change)

### Changed

- **Breaking:** `ShadcnTheme::of` and `ShadcnTheme::of_ui` now return
  `Arc<ShadcnTheme>` instead of `ShadcnTheme`. The installed theme is
  shared instead of copied on every widget access. Field reads, `&theme`
  passing, and method calls work unchanged; code with explicit
  `ShadcnTheme` annotations, struct fields, or by-value parameters must
  adapt — see `docs/migration-0.1.0.md`. (`d3366a6`)
- Label text is now laid out once per widget and the same galley is
  reused for painting (previously the label string was cloned and laid
  out twice). No visual or behavioral change: gallery captures before
  and after are byte-identical. (`d3366a6`)
- Minimum supported Rust version is now 1.95. It was previously stated
  as 1.85, which could never build the crate: the pinned `egui` 0.37
  dependency requires rustc 1.95. (This change)
