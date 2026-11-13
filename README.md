# egui-shadcn-ui

A pure Rust port of [shadcn/ui](https://ui.shadcn.com) to [egui](https://github.com/emilk/egui).

The API reads like shadcn/ui but is idiomatic Rust: builder structs, variant/size enums,
compound components composed with closures, and controlled state via `&mut`.

```rust
use egui_shadcn_ui::prelude::*;

Button::new("Save")
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Default)
    .show(ui);
```

> **Status: pre-release.** Version 0.1.0 is under development toward v1.0.
> The shared foundation is done — correct OKLCH tokens, embedded Inter and
> Geist Mono fonts with weights, color transitions, layered cached shadows,
> real Lucide icons, and a theme system shared across widgets instead of
> copied per frame — and the 64 components are being verified one by one
> against the original for visual, behavior, and keyboard parity. Until
> that verification is complete, treat component behavior as ported, not
> parity-verified. The crate is not yet published to crates.io.

## Install

The crate is not on crates.io yet (publication is planned for v1.0). For
now, depend on the git repository:

```toml
[dependencies]
egui-shadcn-ui = { git = "https://github.com/rumman02/egui-shadcn" }
```

Requirements: Rust 1.95 or later (the minimum supported Rust version),
and `egui` 0.37 in your own dependencies — the crate's API takes
`egui::Ui` and `egui::Context`, so your app and the crate must share the
same egui version. The crate itself has exactly one runtime dependency
(`egui` 0.37, pinned for v1.0).

Prefer to own the source instead? See
[docs/copy-paste.md](docs/copy-paste.md): every component can be copied
into your project shadcn-style, with its dependencies stated.

## Quick start

Install a preset once, in your app setup. This installs the theme, the
embedded fonts, and the egui visuals — no other setup call is needed:

```rust
use egui_shadcn_ui::prelude::*;

eframe::run_native(
    "my app",
    eframe::NativeOptions::default(),
    Box::new(|cc| {
        Preset::default().install(&cc.egui_ctx);
        Ok(Box::new(MyApp::default()))
    }),
)
```

Then use components in any widget:

```rust
use egui_shadcn_ui::prelude::*;

if Button::new("Save").show(ui).clicked() {
    // ...
}
```

Read the installed theme inside a widget:

```rust
use egui_shadcn_ui::prelude::*;

let theme = ShadcnTheme::of_ui(ui);
let background = theme.palette.background;
```

Switch light/dark mode (the preset is preserved):

```rust
use egui_shadcn_ui::prelude::*;

ShadcnTheme::set_mode(ui.ctx(), Mode::Dark);
```

Pick a different style, base color, or accent color with a preset code
(`"b0"` is the default preset, encoded) or a struct literal:

```rust
use egui_shadcn_ui::prelude::*;

Preset::from_code("b0").expect("valid preset").install(ctx);

Preset {
    style: Style::Vega,
    base_color: BaseColor::Stone,
    ..Preset::default()
}
.install(ctx);
```

## Feature flags

| Flag | Default | Effect |
|---|---|---|
| `fonts` | on | Embeds subsetted Inter (Regular, Medium, SemiBold) and Geist Mono (Regular, Medium) — 385,772 bytes of font data — and installs them with the preset, so `font-medium` / `font-semibold` text renders at the right weights. |

Disable it to ship without the embedded font bytes:

```toml
[dependencies]
egui-shadcn-ui = { git = "https://github.com/rumman02/egui-shadcn", default-features = false }
```

With `fonts` off, the weight helpers degrade to regular-weight rendering
with the host's fonts; everything else works unchanged.

## Theming

A preset combines a style, a base color, an accent (theme) color, a chart
color, and a radius:

- **Styles (8):** `nova` (the default), `vega`, `maia`, `lyra`, `mira`,
  `luma`, `sera`, `rhea`. The style picks component recipes; all styles
  share the same token tables.
- **Base colors (7):** neutral, stone, zinc, mauve, olive, mist, taupe.
- **Accent colors (17):** amber, blue, cyan, emerald, fuchsia, green,
  indigo, lime, orange, pink, purple, red, rose, sky, teal, violet,
  yellow. Accents override the primary/secondary/chart slots of the base
  palette.
- **Preset codes** (`Preset::to_code` / `Preset::from_code`) encode a
  whole preset as a short base62 string with a `b` prefix, matching the
  original's preset-code format, so a theme can be pasted as one string.

Colors resolve from OKLCH tokens (41 slots per theme, 24 registry
themes, light and dark) through `ShadcnTheme::of` / `of_ui`, which
return a shared `Arc<ShadcnTheme>`. See
[docs/migration-0.1.0.md](docs/migration-0.1.0.md) if you wrote code
against the pre-`Arc` accessors.

## Gallery

The repository ships a gallery app with a demo page for every component,
plus motion, shadow, and icon foundation pages, with a light/dark toggle
and a preset picker:

```sh
cargo run --example gallery
```

## Architecture

- `theme/` — OKLCH design tokens (41 slots, 24 registry themes, light/dark
  palettes), radius/spacing/type scales, the preset system, and the
  embedded fonts.
- `icons/` — real Lucide 1.72.0 icon geometry, rasterized once and cached
  as textures, with the SVG-to-geometry generator committed alongside.
- `utils/` — the color/opacity transition system, layered cached shadows,
  focus-ring painting, and stable id helpers shared by all components.
- `components/` — one `.rs` file per shadcn component, reading the shared
  theme and painting through the shared helpers.
- `examples/gallery` — the gallery app described above.

## Credits

- Original design and tokens: [shadcn/ui](https://ui.shadcn.com) (MIT).
- Inter and Geist Mono fonts: SIL Open Font License 1.1, full texts in
  [src/theme/fonts/LICENSE-Inter.txt](src/theme/fonts/LICENSE-Inter.txt)
  and
  [src/theme/fonts/LICENSE-GeistMono.txt](src/theme/fonts/LICENSE-GeistMono.txt).
- Lucide icons: ISC licence, full text in
  [src/icons/LICENSE-Lucide.txt](src/icons/LICENSE-Lucide.txt).
- Ported independently to Rust/egui; no code copied from other ports.

## License

MIT. See [CHANGELOG.md](CHANGELOG.md) for what changed.
