//! Stable id helpers so component state persists across frames.

use egui::Id;

/// Builds a stable component id from a dotted path, e.g.
/// `id_path(ui, &["tabs", "profile"])`.
///
/// Prefer passing explicit `id_salt`s from callers; this helper is for
/// internal compound widgets that derive their id from a parent response.
pub fn id_path(parent: Id, parts: &[&str]) -> Id {
    Id::unique((parent, parts))
}

/// `Id::unique` with a debug-friendly source string.
pub fn unique_id(name: &str) -> Id {
    Id::unique(name)
}
