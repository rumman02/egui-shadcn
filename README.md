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

## Architecture

- `theme/` — OKLCH design tokens straight from shadcn v4, `light`/`dark` palettes,
  radius/spacing/type scales, and the preset system (8 styles × 7 base colors × 17 accents).
- `icons/` — lucide-style icons drawn with `egui::Painter` paths (no JS, no assets).
- `utils/` — animation, focus ring, shadow, id helpers shared by all components.
- `components/` — one `.rs` file per shadcn component.
- `examples/gallery` — an eframe app showing every component, variant, size and state,
  with a light/dark toggle and a preset picker.

## Usage

```rust
use egui_shadcn_ui::prelude::*;

// In your app setup:
Preset::from_code("b0").expect("valid preset").apply(ctx);

// In a widget:
ShadcnTheme::of(ui).colors() // read the resolved palette
```

## Status

See [PROGRESS.md](PROGRESS.md) for the live per-component checklist.

## Credits

- Original design and tokens: [shadcn/ui](https://ui.shadcn.com) (MIT).
- Ported independently to Rust/egui; no code copied from other ports.

## License

MIT
