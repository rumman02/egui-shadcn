# Migration note: `ShadcnTheme::of` / `of_ui` now return `Arc<ShadcnTheme>`

**Applies to:** code written against the crate before the per-frame
refactor (commit `d3366a6`, unreleased — see `CHANGELOG.md`). If you
started from the current tree, this note is background only: nothing to
migrate.

## What changed

`ShadcnTheme::of` and `ShadcnTheme::of_ui` now return
`Arc<ShadcnTheme>` instead of `ShadcnTheme`:

```rust
// Before
pub fn of(ctx: &egui::Context) -> Self { /* ... */ }
pub fn of_ui(ui: &egui::Ui) -> Self { Self::of(ui.ctx()) }

// After
pub fn of(ctx: &egui::Context) -> Arc<Self> { /* ... */ }
pub fn of_ui(ui: &egui::Ui) -> Arc<Self> { Self::of(ui.ctx()) }
```

Every call used to copy the whole 224-byte theme struct out of the
context; now every call shares one allocation and costs a reference-count
bump. There is no deprecated shim returning the old owned value, by
design — a shim would reintroduce exactly the per-frame copy being
eliminated.

Nothing else changed. `ShadcnTheme::new`, `Preset::build`,
`Preset::install`, `ShadcnTheme::install`, `ShadcnTheme::set_mode`, and
`ShadcnTheme::current_mode` keep their signatures and behavior, and no
other public API changed.

## What keeps working unchanged

Field reads (`theme.palette`, `theme.mode`, `theme.style`,
`theme.radius`), passing `&theme` to any function taking
`&ShadcnTheme`, and method calls on the theme all work exactly as before
via `Arc` deref coercion and auto-ref. Most call sites need no edit at
all — inside this crate, all 178 call sites compiled and behaved
identically with zero textual change.

## What must adapt

Downstream code that does any of the following must change:

| Old code | New code |
|---|---|
| `let theme: ShadcnTheme = ShadcnTheme::of(ctx);` | `let theme: Arc<ShadcnTheme> = ShadcnTheme::of(ctx);` (or drop the annotation) |
| Storing the returned theme in a struct field typed `ShadcnTheme` | Store `Arc<ShadcnTheme>` |
| Passing the theme by value to `fn f(theme: ShadcnTheme)` | Change the parameter to `Arc<ShadcnTheme>` (or `&ShadcnTheme` and pass `&theme`) |
| Matching on / moving fields out of the returned theme | Borrow through the `Arc`, or take an owned copy explicitly: `(*theme).clone()` |
| `let owned = ShadcnTheme::of(ctx);` then mutating `owned` | `let mut owned = (*ShadcnTheme::of(ctx)).clone();` — one explicit copy, at the caller's choice |

The new-code patterns, as one compilable example:

```rust
use egui_shadcn_ui::prelude::*;
use std::sync::Arc;

fn read_theme(ctx: &egui::Context) {
    // Field reads and method calls work unchanged.
    let theme = ShadcnTheme::of(ctx);
    let _bg = theme.palette.background;
    let _mode = theme.mode;
}

fn store_theme(ctx: &egui::Context) -> Arc<ShadcnTheme> {
    // Explicit annotation, or a struct field: store the Arc.
    ShadcnTheme::of(ctx)
}

fn takes_theme(theme: &ShadcnTheme) {
    let _primary = theme.palette.primary;
}

fn pass_by_ref(ctx: &egui::Context) {
    let theme = ShadcnTheme::of(ctx);
    takes_theme(&theme); // &Arc<ShadcnTheme> coerces to &ShadcnTheme
}

fn owned_copy(ctx: &egui::Context) {
    // Mutating a private copy: one explicit clone, at your choice.
    let mut owned = (*ShadcnTheme::of(ctx)).clone();
    owned.mode = Mode::Dark;
    assert!(owned.mode.is_dark());
}
```

To find code that needs attention, search for the old patterns:

```sh
grep -rn "ShadcnTheme::of" --include='*.rs' .
grep -rn ": ShadcnTheme" --include='*.rs' .   # explicit annotations / fields
```

## Semantics note

The returned `Arc` is a snapshot of the installed theme at read time. If
the application installs a new preset or switches mode, previously
obtained handles keep the old values; call `of` / `of_ui` again to see
the new theme. This matches the old behavior: each old call returned an
independent copy frozen at read time.
