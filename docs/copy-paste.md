# Copy-paste route: use components without the crate

Like the original shadcn/ui, you can copy component source into your own
project instead of adding this crate as a dependency. You own the copied
code: no upstream updates, no version resolution, and you can edit it
freely. The trade-off is that you also own keeping it up to date.

## What "copy-paste" means for this crate

Components are not standalone files. Every component reads the shared
theme, and most use the shared icon, shadow, focus, and animation
helpers. So copying a component means copying:

1. The component file itself (plus any sibling component file it
   references — see "Finding the closure" below).
2. The three foundation modules, whole: `theme/`, `utils/`, `icons/`.
3. The licence files that ship with the embedded assets (see "Licences").
4. A minimal `lib.rs` / `prelude.rs` / `components/mod.rs` wiring, and a
   `Cargo.toml` with the same single runtime dependency (`egui` 0.37).

There is no per-component feature selection: the foundation modules come
as a unit. The Button example below was verified end to end by following
these exact steps in a scratch crate and running `cargo build` and
`cargo test` (see the distribution report in the project history for the
verification record).

## Finding the closure for any component

A component's closure is the set of files its code's `crate::` paths
reach, transitively. To find it for component `X`:

```sh
# Direct crate paths used by the component's code (outside its own tests).
# Comment lines are stripped first: doc comments carry intra-doc links
# such as [`crate::components::other::Thing`] that look like paths but
# are not dependencies, and counting them over-approximates the closure.
sed '/mod tests/,$d' src/components/X.rs | grep -v '^[[:space:]]*//' | grep -o 'crate::[a-z_:]*' | sort -u
```

Repeat for every `crate::components::<other>` hit (a sibling component
file the code uses), and include the whole `theme/`, `utils/`, and
`icons/` modules whenever any path reaches them — their files
reference each other, so partial copies do not compile. A hit that
appears only in a trailing comment after code is not a dependency:
confirm each hit is a code use with `grep -n` before adding a file.
The component's own `#[cfg(test)]` module comes along with the file;
its tests only ever reference the component itself and `crate::theme`,
so no extra files are needed for them.

Two files are *not* part of any runtime closure and are only needed if
you want to regenerate data:

- `src/theme/gen_theme_data.awk` — regenerates `theme_data.rs` from the
  shadcn registry's `themes.ts`.
- `src/icons/gen_icons.py` — regenerates `data.rs` from Lucide SVG files.

## Worked example: Button

Button's closure (verified by the grep above, plus the transitive walk)
is everything in the table below except `button_group.rs`, which is an
optional companion. The worked example copies both files, so the
`ButtonGroup` container is available too:

| Copy | Contents |
|---|---|
| `src/lib.rs` | Verbatim from the repo (5 lines: the `components`, `icons`, `prelude`, `theme`, `utils` modules). |
| `src/prelude.rs` | Verbatim from the repo. |
| `src/components/button.rs` | Verbatim. |
| `src/components/button_group.rs` | Optional companion, verbatim. ButtonGroup reads the group-fusing context (`GroupOrientation`, `GroupPosition`, `fuse_radius`) from `button.rs` — the dependency runs from ButtonGroup to Button, not the other way. Copying `button.rs` alone is sufficient for Button, including its group-fusing API; copy this file as well only if you want the `ButtonGroup` container. |
| `src/components/mod.rs` | Minimal wiring (below) — do **not** copy the repo's file, it declares all 64 components. |
| `src/theme/` | Whole directory: `mod.rs`, `colors.rs`, `fonts.rs`, `presets.rs`, `theme_data.rs`, `tokens.rs`, plus the `fonts/` subdirectory (5 `.ttf` files + 2 licence texts). |
| `src/utils/` | Whole directory: `mod.rs`, `animation.rs`, `focus.rs`, `id.rs`, `shadow.rs`. |
| `src/icons/` | `mod.rs`, `icons.rs`, `data.rs`, `LICENSE-Lucide.txt`. (`gen_icons.py` only if you regenerate.) |

To copy Button alone, omit `button_group.rs` and its two lines in the
minimal `mod.rs` below — `button.rs` compiles without it.

### 1. Crate manifest

```toml
[package]
name = "copied_button"
version = "0.1.0"
edition = "2021"

[features]
default = ["fonts"]
# Embed Inter + Geist Mono (see "Fonts" below). Omit the default to
# ship without the embedded font bytes; weight helpers then degrade
# to regular-weight rendering.
fonts = []

[dependencies]
egui = { version = "0.37", default-features = false, features = ["default_fonts"] }
# eframe is only needed for the example app below, not for the copied code.
eframe = { version = "0.37", default-features = true }
```

The copied code pins to `egui` 0.37, the same version the crate itself
uses. Other egui versions are untested with this code.

### 2. Minimal `src/components/mod.rs`

```rust
pub mod button;
pub mod button_group;

pub use button::{Button, ButtonSize, ButtonVariant, GroupOrientation, GroupPosition};
pub use button_group::{ButtonGroup, ButtonGroupSeparator, ButtonGroupText};
```

### 3. Minimal app (`src/main.rs`)

```rust
use copied_button::prelude::*;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native(
        "copied button",
        options,
        Box::new(|cc| {
            // One setup call: installs the default preset (nova style,
            // neutral base), the fonts, and the egui visuals.
            Preset::default().install(&cc.egui_ctx);
            Ok(Box::new(App))
        }),
    )
}

#[derive(Default)]
struct App;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Copied Button");
            if Button::new("Save").show(ui).clicked() {
                // ...
            }
            Button::new("Delete")
                .variant(ButtonVariant::Destructive)
                .show(ui);
        });
    }
}
```

### 4. Build and test

```sh
cargo build
cargo test
```

`cargo test` runs the unit tests embedded in the copied files (theme,
button, button-group, shadow, icon, animation, and id tests) — they all
pass in the copied home, which is how you know the closure is complete.
If a file is missing, the failure is a compile error naming the missing
module, not a silent behavior change.

## Setup every copied component needs

Copied components assume a theme is installed on the `egui::Context`
before the first frame. One call, in your app setup, covers every
component:

```rust
Preset::default().install(&ctx);
```

This installs the default preset (nova style, neutral base color, light
mode), the embedded fonts (when the `fonts` feature is on), and the
`egui::Style` visuals. Variants:

```rust
// A preset code ("b0" is the default preset, encoded).
Preset::from_code("b0").expect("valid preset").install(&ctx);
// A custom combination: any style, base color, and accent color.
Preset { style: Style::Vega, base_color: BaseColor::Stone, ..Preset::default() }.install(&ctx);
// Switch light/dark later; the preset is preserved.
ShadcnTheme::set_mode(&ctx, Mode::Dark);
```

If no theme is installed, `ShadcnTheme::of` installs the default preset
on first read, so components still render — but install explicitly so
the mode and preset are the ones you chose.

## Fonts

The `fonts` feature (on by default) embeds subsetted Inter (Regular,
Medium, SemiBold) and Geist Mono (Regular, Medium) — 385,772 bytes of
font data — and installs them when the preset is installed. With the
feature off, no font bytes are embedded and the weight helpers
(`medium`, `semibold`, `mono_medium`) render at regular weight using
the host's fonts. Copy the `src/theme/fonts/` directory (`.ttf` files
plus licence texts) whenever the feature is on; the `include_bytes!`
paths in `fonts.rs` are relative to that directory.

## Licences

Copied code carries the crate's MIT licence. The embedded assets carry
their own licences, and their full texts ship beside the assets — copy
them with the code:

- Inter and Geist Mono: SIL Open Font License 1.1
  (`src/theme/fonts/LICENSE-Inter.txt`,
  `src/theme/fonts/LICENSE-GeistMono.txt`).
- Lucide icon data: ISC licence (`src/icons/LICENSE-Lucide.txt`).

No attribution notice in the binary is required beyond shipping these
licence texts.

## When to prefer the library route

Copy-paste is the right choice when you want to fork or heavily
customize a component, or when you only need one or two components and
want no dependency. Prefer the Cargo library route (see the README)
when you want upstream fixes, new components, and theme updates without
re-copying: the library is one dependency, and `cargo update` carries
the maintenance.
