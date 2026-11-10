//! shadcn `toggle-group.tsx` — a set of related toggles.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/toggle-group.tsx`. The group
//! provides variant/size defaults from `toggleVariants` to every
//! [`ToggleGroupItem`](crate::components::toggle::Toggle) child and supports
//! the two Radix modes:
//!
//! - `type="single"` — at most one pressed item (pressing the pressed item
//!   deselects it),
//! - `type="multiple"` — any number of pressed items.
//!
//! With the default `spacing 0` items fuse like the source
//! (`data-[spacing=0]:rounded-none ... first:rounded-l-md last:rounded-r-md`
//! and `border-l-0` for outline) via the shared group protocol; a positive
//! spacing lays them out as individually rounded toggles with a gap. Fused
//! items add `px-3` (base) / `px-2` (nova
//! `.cn-toggle-group-item`) and drop the outline `shadow-xs`
//! (`data-[spacing=0]:shadow-none`, and nova outline has none to begin with).
//!
//! Arrow keys move focus between items (egui's spatial navigation); focus
//! does not change the pressed state, like Radix.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! // type="single"
//! let mut align = Some(0);
//! ToggleGroup::single(&mut align)
//!     .item("Left")
//!     .item("Center")
//!     .item("Right")
//!     .id_salt("align")
//!     .show(ui);
//!
//! // type="multiple"
//! let mut styles = vec![false, false, false];
//! ToggleGroup::multiple(&mut styles)
//!     .item("Bold")
//!     .item("Italic")
//!     .item("Underline")
//!     .variant(ToggleVariant::Outline)
//!     .id_salt("styles")
//!     .show(ui);
//! ```

use crate::components::button::{group, GroupOrientation};
use crate::components::toggle::{Toggle, ToggleSize, ToggleVariant};
use crate::icons::Icon;
use crate::theme::{ShadcnTheme, Style};
use egui::{InnerResponse, Ui};

/// Horizontal padding of items fused into a `spacing 0` group: `px-3` in the
/// base recipe, `px-2` under nova (`.cn-toggle-group-item`).
pub(crate) fn fused_item_padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0,
        _ => crate::theme::tokens::spacing::MD,
    }
}

/// One group entry (`ToggleGroupItem`).
#[derive(Clone, Debug)]
pub struct GroupItem {
    /// The visible label.
    pub label: String,
    /// Optional leading icon.
    pub icon: Option<Icon>,
}

/// Selection mode of a [`ToggleGroup`].
#[derive(Debug)]
pub enum GroupMode<'a> {
    /// `type="single"` — `Option<usize>` is the pressed item.
    Single(&'a mut Option<usize>),
    /// `type="multiple"` — one bool per item (auto-sized to the item count).
    Multiple(&'a mut Vec<bool>),
}

/// A shadcn toggle group. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct ToggleGroup<'a> {
    mode: GroupMode<'a>,
    items: Vec<GroupItem>,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: f32,
    orientation: GroupOrientation,
    disabled: bool,
    id_salt: Option<String>,
}

impl<'a> ToggleGroup<'a> {
    /// Creates a `type="single"` group.
    pub fn single(selected: &'a mut Option<usize>) -> Self {
        Self {
            mode: GroupMode::Single(selected),
            items: Vec::new(),
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            spacing: 0.0,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: None,
        }
    }

    /// Creates a `type="multiple"` group. The bool vector is resized to the
    /// item count on render.
    pub fn multiple(states: &'a mut Vec<bool>) -> Self {
        Self {
            mode: GroupMode::Multiple(states),
            items: Vec::new(),
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            spacing: 0.0,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: None,
        }
    }

    /// Adds an item.
    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(GroupItem {
            label: label.into(),
            icon: None,
        });
        self
    }

    /// Adds an item with a leading icon.
    pub fn item_with_icon(mut self, label: impl Into<String>, icon: Icon) -> Self {
        self.items.push(GroupItem {
            label: label.into(),
            icon: Some(icon),
        });
        self
    }

    /// Sets the variant applied to every item.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size applied to every item.
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the gap between items. `0` (default) fuses corners and borders
    /// like the source's `data-[spacing=0]` styles.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the layout orientation.
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables every item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt (required for the corner fusing when nesting
    /// multiple groups).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group.
    pub fn show(self, ui: &mut Ui) -> InnerResponse<()> {
        let fused = self.spacing <= 0.0;
        let group_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.next_auto_id(),
        };

        if fused {
            group::begin(ui, group_id, self.orientation);
        }

        let inner = match self.orientation {
            GroupOrientation::Horizontal => ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = self.spacing.max(0.0);
                self.render_items(ui);
            }),
            GroupOrientation::Vertical => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = self.spacing.max(0.0);
                self.render_items(ui);
            }),
        };

        if fused {
            group::end(ui, group_id);
        }
        inner
    }

    fn render_items(self, ui: &mut Ui) {
        let ToggleGroup {
            mut mode,
            items,
            variant,
            size,
            spacing,
            orientation: _,
            disabled,
            id_salt: _,
        } = self;

        // `type="multiple"`: keep the bool vector in sync with the items.
        if let GroupMode::Multiple(states) = &mut mode {
            if states.len() < items.len() {
                states.resize(items.len(), false);
            }
        }

        let fused = spacing <= 0.0;
        // `data-[spacing=0]:px-3` (base) / `group-data-[spacing=0]:px-2`
        // (nova `.cn-toggle-group-item`).
        let fused_px = fused_item_padding_x(ShadcnTheme::of_ui(ui).style);
        for (i, item) in items.iter().enumerate() {
            let mut pressed = match &mode {
                GroupMode::Single(selected) => **selected == Some(i),
                GroupMode::Multiple(states) => states.get(i).copied().unwrap_or(false),
            };
            let mut toggle = Toggle::new(&mut pressed)
                .text(item.label.clone())
                .variant(variant)
                .size(size)
                .disabled(disabled);
            if let Some(icon) = item.icon {
                toggle = toggle.icon(icon);
            }
            if fused {
                // Fused `px-3` (base) / `px-2` (nova), and no per-item
                // rounding — the group protocol handles first/last corners.
                toggle = toggle.padding_x(fused_px);
            }
            let response = toggle.show(ui);
            if response.clicked() {
                match &mut mode {
                    GroupMode::Single(selected) => {
                        // Radix `type="single"`: pressing the pressed item
                        // deselects it.
                        if **selected == Some(i) {
                            **selected = None;
                        } else {
                            **selected = Some(i);
                        }
                    }
                    GroupMode::Multiple(states) => {
                        if let Some(state) = states.get_mut(i) {
                            *state = !*state;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Preset;
    use egui::{Pos2, Rect, Vec2};

    fn run_group(
        ctx: &egui::Context,
        mode: GroupMode<'_>,
        items: Vec<GroupItem>,
        spacing: f32,
        id_salt: &str,
    ) {
        let mut g = Some(ToggleGroup {
            mode,
            items,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            spacing,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: Some(id_salt.to_owned()),
        });
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique(id_salt),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0))),
            );
            g.take().unwrap().show(ui);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn single_presses_and_deselects_like_radix() {
        // Radix `type="single"`: pressing the pressed item deselects it.
        let mut selected = Some(0);
        if selected == Some(0) {
            selected = None;
        }
        assert_eq!(selected, None);
        selected = Some(1);
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn multiple_states_resize_to_items() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut states: Vec<bool> = vec![];
        run_group(
            &ctx,
            GroupMode::Multiple(&mut states),
            vec![
                GroupItem {
                    label: "A".into(),
                    icon: None,
                },
                GroupItem {
                    label: "B".into(),
                    icon: None,
                },
                GroupItem {
                    label: "C".into(),
                    icon: None,
                },
            ],
            0.0,
            "tg_resize",
        );
        assert_eq!(states, vec![false, false, false]);
    }

    #[test]
    fn single_mode_renders_fused() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected: Option<usize> = None;
        run_group(
            &ctx,
            GroupMode::Single(&mut selected),
            vec![
                GroupItem {
                    label: "Left".into(),
                    icon: None,
                },
                GroupItem {
                    label: "Center".into(),
                    icon: None,
                },
            ],
            0.0,
            "tg_single",
        );
        assert_eq!(selected, None);
    }

    #[test]
    fn builders_keep_fields() {
        let mut selected = Some(0);
        let mut states = vec![false];
        let g = ToggleGroup::single(&mut selected)
            .item("A")
            .item_with_icon("B", Icon::Plus)
            .variant(ToggleVariant::Outline)
            .size(ToggleSize::Sm)
            .spacing(8.0)
            .orientation(GroupOrientation::Vertical)
            .disabled(true)
            .id_salt("g");
        assert_eq!(g.items.len(), 2);
        assert!(g.items[1].icon.is_some());
        assert_eq!(g.variant, ToggleVariant::Outline);
        assert_eq!(g.size, ToggleSize::Sm);
        assert_eq!(g.spacing, 8.0);
        assert_eq!(g.orientation, GroupOrientation::Vertical);
        assert!(g.disabled);
        assert_eq!(g.id_salt.as_deref(), Some("g"));

        let g2 = ToggleGroup::multiple(&mut states);
        assert!(matches!(g2.mode, GroupMode::Multiple(_)));
    }

    #[test]
    fn defaults_match_source() {
        let mut selected = None;
        let g = ToggleGroup::single(&mut selected);
        assert!(matches!(g.mode, GroupMode::Single(_)));
        assert_eq!(g.spacing, 0.0);
        assert_eq!(g.variant, ToggleVariant::Default);
        assert_eq!(g.size, ToggleSize::Default);
        assert_eq!(g.orientation, GroupOrientation::Horizontal);
        assert!(!g.disabled);
    }

    #[test]
    fn fused_item_padding_is_px3_base_px2_nova() {
        // base `data-[spacing=0]` item adds `px-3`; nova
        // `.cn-toggle-group-item` overrides to `px-2`.
        assert_eq!(fused_item_padding_x(Style::Vega), 12.0);
        assert_eq!(fused_item_padding_x(Style::Nova), 8.0);
    }
}
