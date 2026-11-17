//! shadcn `toggle-group.tsx` — a set of related toggles.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/toggle-group.tsx`. The group
//! provides variant/size defaults from `toggleVariants` to every
//! [`ToggleGroupItem`](crate::components::toggle::Toggle) child and supports
//! the two Radix modes (`type` flows through `...props` to Radix; the value
//! model itself is OPEN-1 in the frozen spec — see below):
//!
//! - `type="single"` — at most one pressed item,
//! - `type="multiple"` — any number of pressed items.
//!
//! ## Spacing units (spec §0.3)
//!
//! The original `spacing` prop is a unitless number of Tailwind spacing
//! units: the group sets `style={{ "--gap": spacing }}` and the gap class is
//! `gap-[--spacing(var(--gap))]`, so the gap is **4 × `spacing` px**.
//! [`ToggleGroup::spacing`] takes the same units. `spacing` 0 (the default)
//! fuses items like the source (`data-[spacing=0]:rounded-none ...
//! first:rounded-l-md last:rounded-r-md` and `border-l-0` for outline) via
//! the shared group protocol; any nonzero spacing disables ALL fusing rules.
//! Fused items get `px-3` (base) / `px-2` (nova `.cn-toggle-group-item`) and
//! drop the outline `shadow-xs` (`data-[spacing=0]:shadow-none`, both
//! styles). Spaced outline items keep `shadow-xs` in both styles (Decision
//! S-1: nova silence keeps the base shadow; the fused `shadow-none` is a
//! separate group rule).
//!
//! ## Context resolution (spec §8.1, settled)
//!
//! The item resolves `context.variant || variant` and `context.size ||
//! size`: the group's value wins whenever it is defined; the item's own
//! prop applies ONLY when the group leaves that prop undefined. The Rust
//! API mirrors this with `Option` group values: a group built without
//! `.variant()`/`.size()` leaves that prop undefined, and each item's own
//! [`GroupItem::variant`]/[`GroupItem::size`] (or the `toggleVariants`
//! default) applies.
//!
//! ## Invalid (N17, decided option (a))
//!
//! Invalid is per-item only (coordinator decision 2026-11-16): the original
//! exposes no group-level invalid prop and no context propagation, so the
//! group API exposes invalid on [`GroupItem`] only. Invalid items render
//! the `toggleVariants` invalid styling (full `destructive` border, both
//! styles/modes; focus-gated invalid ring colour — no rest ring, spec §4.7).
//!
//! ## Hypotheses (OPEN-1/OPEN-2 — NOT Radix facts)
//!
//! The snapshot contains no Radix source, so the following are the crate's
//! established behaviours kept as labelled working assumptions, pending the
//! S6 Work B Radix-source evidence:
//!
//! - **W-TG-2 (OPEN-1):** single-mode re-press deselects. The value model,
//!   deselect semantics, roving focus, arrow/Home/End/Tab behaviour, and
//!   activation keys are all OPEN-1. Arrow keys currently move focus via
//!   egui's spatial navigation; focus does not change the pressed state.
//! - **Group disabled propagation (OPEN-2):** [`ToggleGroup::disabled`]
//!   disables every item. Whether Radix Root `disabled` propagates to items
//!   is unsettled; the propagation is kept as a port convention (API
//!   stability) and re-opens when OPEN-2 settles. Per-item
//!   [`GroupItem::disabled`] mirrors the original's per-item prop surface.
//! - **W-TG-1 (OPEN-4):** the item string's `px-3`/`min-w-0` win their
//!   `cn()` merges (later class wins, the same reading the toggle spec
//!   §2.3 applies to the variant merge), so items render `px-3` at all
//!   spacings in base. The nova fused `px-2` is specificity-settled
//!   regardless. The `min-w-0` half of this assumption is NOT implemented:
//!   `Toggle` enforces its size minimum width and exposes no min-width
//!   override (cross-line gap, see the S1 report).
//! - **Orientation (OPEN-2/OPEN-3):** the original declares no orientation
//!   prop; [`ToggleGroup::orientation`] is a port API addition. Vertical
//!   lays out as a flex column (port reading); the vertical fusing intent
//!   (including the 2px horizontal seam in vertical fused outline groups,
//!   spec §9 case 5) is OPEN-3.
//!
//! ## Deviations (PROPOSED — Expert Reviewer rules at the wave pack)
//!
//! D1 (no ARIA tree / group semantics), D2 (`:focus-visible` modality — the
//! ring shows on any focus), D3 (`data-*` attributes dropped; their styling
//! effects are implemented directly). See spec §10.
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
use crate::components::toggle::{metrics, Toggle, ToggleSize, ToggleVariant};
use crate::icons::Icon;
use crate::theme::{ShadcnTheme, Style};
use egui::{vec2, Align, InnerResponse, Layout, Ui};

/// One Tailwind spacing unit in points: `--spacing = 0.25rem` at the
/// default 16px root (framework default, spec §0).
pub(crate) const SPACING_UNIT_PX: f32 = crate::theme::tokens::spacing::XS;

/// Effective spacing in Tailwind units. Negative values clamp to 0 — the
/// port's choice for the invalid-CSS case (spec §9 case 6: a negative gap
/// length is invalid CSS and browser behaviour is unestablished); no
/// original fidelity is claimed for the clamp.
pub(crate) fn effective_spacing_units(spacing: f32) -> f32 {
    spacing.max(0.0)
}

/// Gap between items in points: `spacing × --spacing` = 4 × spacing px
/// (spec §0.3, §2.4).
pub(crate) fn gap_px(spacing_units: f32) -> f32 {
    effective_spacing_units(spacing_units) * SPACING_UNIT_PX
}

/// Whether items fuse. Fusing keys off `data-spacing` being exactly `0`
/// (spec §9 case 6): any nonzero spacing — including fractional values —
/// disables ALL fusing rules.
pub(crate) fn is_fused(spacing: f32) -> bool {
    effective_spacing_units(spacing) == 0.0
}

/// Horizontal padding of group items (spec §2.2, §2.5).
///
/// Under working assumption W-TG-1 (OPEN-4, merge-wins), the item string's
/// `px-3` (12px) overrides the size `px-*` at ALL spacings in base. Under
/// nova the fused `px-2` (8px) is specificity-settled regardless of OPEN-4
/// (spec §2.3), and the spaced value follows W-TG-1 (`px-3`, 12px).
/// Single change point if OPEN-4 settles the other way.
pub(crate) fn item_padding_x(style: Style, fused: bool) -> f32 {
    match (style, fused) {
        (Style::Nova, true) => crate::theme::tokens::spacing::SM, // px-2
        _ => crate::theme::tokens::spacing::MD,                   // px-3
    }
}

/// Context resolution (spec §8.1, settled): the group's value wins whenever
/// it is defined (truthy); the item's own prop applies only when the group
/// leaves that prop undefined; otherwise the `toggleVariants` default.
pub(crate) fn resolve_context<T: Copy>(group: Option<T>, item: Option<T>, default: T) -> T {
    group.or(item).unwrap_or(default)
}

/// Effective disabled state of one item: the group flag OR the item flag.
/// Group-level propagation is the OPEN-2-pending port convention (see the
/// module docs); the per-item flag mirrors the original's per-item prop.
pub(crate) fn effective_disabled(group_disabled: bool, item_disabled: bool) -> bool {
    group_disabled || item_disabled
}

/// Single-mode click transition (working assumption W-TG-2, OPEN-1):
/// pressing the pressed item deselects it. Pure, so the assumption is
/// pinned in one place and S9 can change it in one place when the Radix
/// evidence settles OPEN-1. This is the crate's established behaviour kept
/// as a labelled hypothesis, NOT a Radix claim (spec §9 case 4: S1 must not
/// implement deselect — or its absence — as established fact).
pub(crate) fn single_click_next(selected: Option<usize>, index: usize) -> Option<usize> {
    if selected == Some(index) {
        None
    } else {
        Some(index)
    }
}

/// One group entry (`ToggleGroupItem`).
#[derive(Clone, Debug)]
pub struct GroupItem {
    /// The visible label.
    pub label: String,
    /// Optional leading icon.
    pub icon: Option<Icon>,
    /// Per-item `aria-invalid` (N17 option (a); spec §4.7/§8.2). The group
    /// exposes no group-level invalid API.
    pub invalid: bool,
    /// Per-item `disabled` (item prop surface, spec §8.1). The effective
    /// state is group-disabled OR item-disabled.
    pub disabled: bool,
    /// The item's own variant; applies only when the group leaves `variant`
    /// undefined (context resolution, spec §8.1).
    pub variant: Option<ToggleVariant>,
    /// The item's own size; applies only when the group leaves `size`
    /// undefined (context resolution, spec §8.1).
    pub size: Option<ToggleSize>,
}

impl GroupItem {
    /// Creates an item with default per-item state (valid, enabled, no
    /// own variant/size — the group's values or the `toggleVariants`
    /// defaults apply).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            invalid: false,
            disabled: false,
            variant: None,
            size: None,
        }
    }

    /// Shows an icon before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Marks this item aria-invalid (per-item only, N17 option (a)).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disables this item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the item's own variant; ignored when the group defines a
    /// variant (context resolution, spec §8.1).
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Sets the item's own size; ignored when the group defines a size
    /// (context resolution, spec §8.1).
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = Some(size);
        self
    }
}

/// Selection mode of a [`ToggleGroup`].
#[derive(Debug)]
pub enum GroupMode<'a> {
    /// `type="single"` — `Option<usize>` is the pressed item. Index
    /// identity and the deselect-on-repress transition are working
    /// assumptions pending OPEN-1 (Radix identifies items by `value`
    /// string; see the module docs).
    Single(&'a mut Option<usize>),
    /// `type="multiple"` — one bool per item (auto-sized to the item count).
    Multiple(&'a mut Vec<bool>),
}

/// A shadcn toggle group. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct ToggleGroup<'a> {
    mode: GroupMode<'a>,
    items: Vec<GroupItem>,
    /// Group `variant` prop; `None` = undefined, so each item's own variant
    /// (or the default) applies (context resolution, spec §8.1).
    variant: Option<ToggleVariant>,
    /// Group `size` prop; `None` = undefined, so each item's own size (or
    /// the default) applies (context resolution, spec §8.1).
    size: Option<ToggleSize>,
    /// Gap in Tailwind spacing units (spec §0.3); the rendered gap is
    /// 4 × this value in points. `0` (default) fuses items.
    spacing: f32,
    orientation: GroupOrientation,
    /// Group-level disabled flag, propagated to every item (OPEN-2-pending
    /// port convention; see the module docs).
    disabled: bool,
    id_salt: Option<String>,
}

impl<'a> ToggleGroup<'a> {
    /// Creates a `type="single"` group.
    pub fn single(selected: &'a mut Option<usize>) -> Self {
        Self {
            mode: GroupMode::Single(selected),
            items: Vec::new(),
            variant: None,
            size: None,
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
            variant: None,
            size: None,
            spacing: 0.0,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: None,
        }
    }

    /// Adds an item.
    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(GroupItem::new(label));
        self
    }

    /// Adds an item with a leading icon.
    pub fn item_with_icon(mut self, label: impl Into<String>, icon: Icon) -> Self {
        self.items.push(GroupItem::new(label).icon(icon));
        self
    }

    /// Adds a fully-configured item (per-item invalid/disabled/variant/
    /// size — see [`GroupItem`]).
    pub fn add_item(mut self, item: GroupItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the variant applied to every item. When set, it wins over every
    /// item's own variant (context resolution, spec §8.1). When never
    /// called, the prop is undefined and each item's own variant (or the
    /// `toggleVariants` default) applies.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Sets the size applied to every item. When set, it wins over every
    /// item's own size (context resolution, spec §8.1). When never called,
    /// the prop is undefined and each item's own size (or the
    /// `toggleVariants` default) applies.
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the gap between items in Tailwind spacing units (spec §0.3):
    /// the rendered gap is 4 × `spacing` points. `0` (default) fuses
    /// corners and borders like the source's `data-[spacing=0]` styles;
    /// any nonzero value disables all fusing rules. Negative values clamp
    /// to 0 (port choice, spec §9 case 6).
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the layout orientation. Port API addition: the original
    /// declares no orientation prop (Radix prop via `...props`, OPEN-2/3);
    /// vertical lays out as a flex column (port reading).
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables every item (OPEN-2-pending port convention; see the module
    /// docs). Per-item disabling is available via [`GroupItem::disabled`].
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

    /// Renders the group. The group container paints nothing in either
    /// recipe (spec §2.4 — "no container frame" is faithful, not a gap)
    /// and does not animate (spec §6 — group level: none, settled). Item
    /// visuals, states, and transitions are [`Toggle`]'s paint, composed
    /// here per the spec's composition rules.
    pub fn show(self, ui: &mut Ui) -> InnerResponse<()> {
        let fused = is_fused(self.spacing);
        let gap = gap_px(self.spacing);
        let group_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.next_auto_id(),
        };

        if fused {
            group::begin(ui, group_id, self.orientation);
        }

        let inner = match self.orientation {
            GroupOrientation::Horizontal => {
                // TGG-VIS-01: `Ui::horizontal` seeds the row at
                // `interact_size.y` (36pt) and centers each widget in the
                // frame height seen so far; a taller item placed later
                // grows the row downward, leaving earlier shorter items
                // 2px above the final row center. Seed the row at the
                // tallest resolved item height (floored at
                // `interact_size.y`, preserving the 36px row-rect floor)
                // so every item centers on the final row height. This
                // replicates `Ui::horizontal` exactly apart from the seed
                // height (same layout, same RTL branch, no wrapping).
                let style = ShadcnTheme::of_ui(ui).style;
                let tallest = self
                    .items
                    .iter()
                    .map(|item| {
                        metrics::height(
                            resolve_context(self.size, item.size, ToggleSize::Default),
                            style,
                        )
                    })
                    .fold(0.0f32, f32::max);
                let row_h = ui.spacing().interact_size.y.max(tallest);
                let layout = if ui.layout().prefer_right_to_left() {
                    Layout::right_to_left(Align::Center)
                } else {
                    Layout::left_to_right(Align::Center)
                }
                .with_main_wrap(false);
                ui.allocate_ui_with_layout(
                    vec2(ui.available_size_before_wrap().x, row_h),
                    layout,
                    |ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        self.render_items(ui);
                    },
                )
            }
            GroupOrientation::Vertical => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = gap;
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

        let fused = is_fused(spacing);
        // Item-string padding (spec §2.2/§2.5, W-TG-1): `px-3` at all
        // spacings in base, nova fused `px-2` (specificity-settled).
        let px = item_padding_x(ShadcnTheme::of_ui(ui).style, fused);
        for (i, item) in items.iter().enumerate() {
            let mut pressed = match &mode {
                GroupMode::Single(selected) => **selected == Some(i),
                GroupMode::Multiple(states) => states.get(i).copied().unwrap_or(false),
            };
            let mut toggle = Toggle::new(&mut pressed)
                .text(item.label.clone())
                .variant(resolve_context(
                    variant,
                    item.variant,
                    ToggleVariant::Default,
                ))
                .size(resolve_context(size, item.size, ToggleSize::Default))
                .disabled(effective_disabled(disabled, item.disabled))
                .invalid(item.invalid)
                .padding_x(px);
            if let Some(icon) = item.icon {
                toggle = toggle.icon(icon);
            }
            let response = toggle.show(ui);
            if response.clicked() {
                match &mut mode {
                    GroupMode::Single(selected) => {
                        **selected = single_click_next(**selected, i);
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
    use crate::theme::{Mode, Preset};
    use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, Ui, Vec2};

    /// Renders `show` for `frames` frames under the given style/mode and
    /// returns the LAST frame's output (button-group `run_frames_output`
    /// pattern). Several frames are required: the group protocol learns
    /// the child count on the previous frame, so First/Middle/Last
    /// positions are only correct from frame 2 on.
    fn run_frames_output<T, F>(
        style: Style,
        mode: Mode,
        frames: usize,
        mut show: F,
    ) -> egui::FullOutput
    where
        F: FnMut(&mut Ui) -> T,
    {
        let ctx = egui::Context::default();
        Preset {
            style,
            ..Preset::default()
        }
        .build(mode)
        .install(&ctx);
        let mut last_out = None;
        for _ in 0..frames {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("toggle_group_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0))),
                );
                show(ui);
            });
            out.textures_delta.clear();
            last_out = Some(out);
        }
        last_out.expect("at least one frame")
    }

    /// Border-stroke shapes (`Shape::Rect` with a 1px stroke and a
    /// transparent fill) in paint order, with stroke colours and corner
    /// radii.
    fn border_strokes(out: &egui::FullOutput) -> Vec<(Rect, Color32, CornerRadius)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.stroke.width == 1.0 && rs.fill == Color32::TRANSPARENT =>
                {
                    Some((rs.rect, rs.stroke.color, rs.corner_radius))
                }
                _ => None,
            })
            .collect()
    }

    /// One border stroke per item: collapses the clip-based fused stroke.
    /// `paint_toggle_fused_stroke` paints Middle/Last items as THREE
    /// `rect_stroke` shapes with the identical (rect, colour, radius) —
    /// one main clip + two corner cells — differing only in clip rect, so
    /// the raw `border_strokes` list holds 1 + 3 + 3 = 7 entries for a
    /// 3-item fused group. Deduping exact triples (paint order preserved)
    /// recovers one stroke per item on the rebuilt paint; on the old paint
    /// (one shape per item) the dedupe is a no-op. Two distinct items can
    /// never share a triple (distinct rects).
    fn border_strokes_by_item(out: &egui::FullOutput) -> Vec<(Rect, Color32, CornerRadius)> {
        let mut seen: Vec<(Rect, Color32, CornerRadius)> = Vec::new();
        for stroke in border_strokes(out) {
            if !seen.contains(&stroke) {
                seen.push(stroke);
            }
        }
        seen
    }

    /// Fill shapes (non-transparent fill, no stroke, no blur) in paint
    /// order, with corner radii.
    fn fill_shapes(out: &egui::FullOutput) -> Vec<(Rect, CornerRadius)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.fill != Color32::TRANSPARENT
                        && rs.stroke == Stroke::NONE
                        && rs.blur_width == 0.0 =>
                {
                    Some((rs.rect, rs.corner_radius))
                }
                _ => None,
            })
            .collect()
    }

    /// Shadow quads: top-level `Shape::Mesh` shapes drawn from a non-font
    /// texture, with their bounds in paint order. The shadow painter
    /// (`paint_shadow_layers`) draws the cached shadow texture via
    /// `painter.image`, which emits one `Mesh` per atlas quad (four corners
    /// and four edges, no center) carrying the shadow texture id. Text is
    /// `Shape::Text` — glyph meshes live inside it and use the font
    /// texture (`TextureId::default()`) — so a non-default texture id on
    /// a top-level mesh identifies shadow quads exactly.
    fn shadow_shapes(out: &egui::FullOutput) -> Vec<Rect> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Mesh(mesh) if mesh.texture_id != egui::TextureId::default() => {
                    Some(mesh.calc_bounds())
                }
                _ => None,
            })
            .collect()
    }

    /// Ring shapes (`Shape::Rect` with a 3px stroke — `paint_ring`).
    fn ring_shapes(out: &egui::FullOutput) -> Vec<Rect> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.stroke.width == 3.0 => Some(rs.rect),
                _ => None,
            })
            .collect()
    }

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
            variant: None,
            size: None,
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
    fn spacing_units_convert_at_4px() {
        // Spec §0.3: gap = spacing × --spacing = 4 × spacing px.
        assert_eq!(SPACING_UNIT_PX, 4.0);
        assert_eq!(gap_px(0.0), 0.0);
        assert_eq!(gap_px(1.0), 4.0);
        assert_eq!(gap_px(2.0), 8.0);
        assert_eq!(gap_px(0.5), 2.0);
        // Fractional values are real gaps (spec §9 case 6).
        assert_eq!(gap_px(2.5), 10.0);
    }

    #[test]
    fn negative_spacing_clamps_to_fused() {
        // Spec §9 case 6: negative spacing is the port's clamp-to-0
        // choice (no original fidelity claimed).
        assert_eq!(effective_spacing_units(-3.0), 0.0);
        assert_eq!(gap_px(-3.0), 0.0);
        assert!(is_fused(-3.0));
    }

    #[test]
    fn fusing_keys_off_exact_zero() {
        // Spec §9 case 6: ANY nonzero spacing disables all fusing rules.
        assert!(is_fused(0.0));
        assert!(!is_fused(0.5));
        assert!(!is_fused(0.01));
        assert!(!is_fused(2.0));
    }

    #[test]
    fn item_padding_follows_section_2_5() {
        // Base: px-3 (12px) at all spacings (W-TG-1). Nova fused: px-2
        // (8px, specificity-settled). Nova spaced: px-3 (W-TG-1).
        assert_eq!(item_padding_x(Style::Vega, true), 12.0);
        assert_eq!(item_padding_x(Style::Vega, false), 12.0);
        assert_eq!(item_padding_x(Style::Nova, true), 8.0);
        assert_eq!(item_padding_x(Style::Nova, false), 12.0);
    }

    #[test]
    fn context_resolution_group_wins_then_item_then_default() {
        // Spec §8.1 (settled): group value wins when defined; the item's
        // own prop applies only when the group leaves it undefined.
        assert_eq!(
            resolve_context(
                Some(ToggleVariant::Outline),
                Some(ToggleVariant::Default),
                ToggleVariant::Default
            ),
            ToggleVariant::Outline
        );
        assert_eq!(
            resolve_context(None, Some(ToggleVariant::Outline), ToggleVariant::Default),
            ToggleVariant::Outline
        );
        assert_eq!(
            resolve_context(None, None, ToggleVariant::Default),
            ToggleVariant::Default
        );
        assert_eq!(
            resolve_context(
                Some(ToggleSize::Sm),
                Some(ToggleSize::Lg),
                ToggleSize::Default
            ),
            ToggleSize::Sm
        );
        assert_eq!(
            resolve_context(None, Some(ToggleSize::Lg), ToggleSize::Default),
            ToggleSize::Lg
        );
        assert_eq!(
            resolve_context(None, None, ToggleSize::Default),
            ToggleSize::Default
        );
    }

    #[test]
    fn effective_disabled_is_group_or_item() {
        assert!(effective_disabled(true, false));
        assert!(effective_disabled(false, true));
        assert!(effective_disabled(true, true));
        assert!(!effective_disabled(false, false));
    }

    #[test]
    fn single_click_next_deselects_on_repress() {
        // W-TG-2 (OPEN-1 hypothesis, labelled — not a Radix claim).
        assert_eq!(single_click_next(Some(0), 0), None);
        assert_eq!(single_click_next(Some(1), 0), Some(0));
        assert_eq!(single_click_next(None, 2), Some(2));
    }

    #[test]
    fn single_mode_renders_fused() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected: Option<usize> = None;
        run_group(
            &ctx,
            GroupMode::Single(&mut selected),
            vec![GroupItem::new("Left"), GroupItem::new("Center")],
            0.0,
            "tg_single",
        );
        assert_eq!(selected, None);
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
                GroupItem::new("A"),
                GroupItem::new("B"),
                GroupItem::new("C"),
            ],
            0.0,
            "tg_resize",
        );
        assert_eq!(states, vec![false, false, false]);
    }

    #[test]
    fn empty_group_renders_nothing() {
        // Spec §9 case 7: an empty group renders an empty w-fit box.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected: Option<usize> = None;
        run_group(
            &ctx,
            GroupMode::Single(&mut selected),
            vec![],
            0.0,
            "tg_empty",
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
            .add_item(GroupItem::new("C").invalid(true).disabled(true))
            .variant(ToggleVariant::Outline)
            .size(ToggleSize::Sm)
            .spacing(2.0)
            .orientation(GroupOrientation::Vertical)
            .disabled(true)
            .id_salt("g");
        assert_eq!(g.items.len(), 3);
        assert!(g.items[1].icon.is_some());
        assert!(g.items[2].invalid);
        assert!(g.items[2].disabled);
        assert_eq!(g.variant, Some(ToggleVariant::Outline));
        assert_eq!(g.size, Some(ToggleSize::Sm));
        assert_eq!(g.spacing, 2.0);
        assert_eq!(g.orientation, GroupOrientation::Vertical);
        assert!(g.disabled);
        assert_eq!(g.id_salt.as_deref(), Some("g"));

        let g2 = ToggleGroup::multiple(&mut states);
        assert!(matches!(g2.mode, GroupMode::Multiple(_)));

        let item = GroupItem::new("X")
            .icon(Icon::Plus)
            .invalid(true)
            .disabled(true)
            .variant(ToggleVariant::Outline)
            .size(ToggleSize::Lg);
        assert_eq!(item.label, "X");
        assert!(item.icon.is_some());
        assert!(item.invalid);
        assert!(item.disabled);
        assert_eq!(item.variant, Some(ToggleVariant::Outline));
        assert_eq!(item.size, Some(ToggleSize::Lg));
    }

    #[test]
    fn defaults_match_source() {
        let mut selected = None;
        let g = ToggleGroup::single(&mut selected);
        assert!(matches!(g.mode, GroupMode::Single(_)));
        assert_eq!(g.spacing, 0.0);
        // Group variant/size props are undefined until set (context
        // resolution, spec §8.1); items fall back to their own props or
        // the toggleVariants defaults.
        assert_eq!(g.variant, None);
        assert_eq!(g.size, None);
        assert_eq!(g.orientation, GroupOrientation::Horizontal);
        assert!(!g.disabled);
    }

    #[test]
    fn spaced_gap_is_4x_units_between_borders() {
        // Spec §0.3/§3.2: gap = 4 × spacing px, measured between the
        // allocated item boxes (spaced items are position Only, so the
        // border rect IS the allocated rect — no fusing shift).
        let mut states = vec![false, false, false];
        let out = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_gap")
                .spacing(2.0)
                .variant(ToggleVariant::Outline)
                .item("Filter A")
                .item("Filter B")
                .item("Filter C")
                .show(ui);
        });
        let borders = border_strokes(&out);
        assert_eq!(borders.len(), 3);
        for w in borders.windows(2) {
            let gap = w[1].0.min.x - w[0].0.max.x;
            assert!(
                (gap - 8.0).abs() < 0.01,
                "expected 8px gap for spacing 2.0, got {gap}"
            );
        }
    }

    #[test]
    fn vertical_spaced_gap_is_4x_units() {
        let mut states = vec![false, false];
        let out = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_vgap")
                .spacing(1.0)
                .orientation(GroupOrientation::Vertical)
                .variant(ToggleVariant::Outline)
                .item("Top")
                .item("Bottom")
                .show(ui);
        });
        let borders = border_strokes(&out);
        assert_eq!(borders.len(), 2);
        let gap = borders[1].0.min.y - borders[0].0.max.y;
        assert!(
            (gap - 4.0).abs() < 0.01,
            "expected 4px vertical gap for spacing 1.0, got {gap}"
        );
    }

    #[test]
    fn fused_corners_follow_first_middle_last() {
        // Spec §2.5 (base, fused, horizontal): first keeps left corners,
        // last keeps right corners, middle is square. All items pressed
        // so fill shapes exist to read corners from.
        let mut states = vec![true, true, true];
        let out = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_corners")
                .variant(ToggleVariant::Default)
                .item("Bold")
                .item("Italic")
                .item("Underline")
                .show(ui);
        });
        let fills = fill_shapes(&out);
        assert_eq!(fills.len(), 3);
        let r = 8u8; // rounded-md at the default radius
        assert_eq!((fills[0].1.nw, fills[0].1.sw), (r, r));
        assert_eq!((fills[0].1.ne, fills[0].1.se), (0, 0));
        assert_eq!(
            (fills[1].1.nw, fills[1].1.ne, fills[1].1.sw, fills[1].1.se),
            (0, 0, 0, 0)
        );
        assert_eq!((fills[2].1.nw, fills[2].1.sw), (0, 0));
        assert_eq!((fills[2].1.ne, fills[2].1.se), (r, r));
    }

    #[test]
    fn single_fused_item_is_fully_rounded() {
        // Spec §9 case 1: one item is both :first and :last — fully
        // rounded (position Only once the protocol learns the count).
        let mut states = vec![true];
        let out = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_single_item")
                .variant(ToggleVariant::Default)
                .item("Only")
                .show(ui);
        });
        let fills = fill_shapes(&out);
        assert_eq!(fills.len(), 1);
        assert_eq!(
            (fills[0].1.nw, fills[0].1.ne, fills[0].1.sw, fills[0].1.se),
            (8, 8, 8, 8)
        );
    }

    #[test]
    fn fused_outline_has_no_shadow_spaced_outline_has_shadow() {
        // Spec §4.5 + S-1: fused items render shadow-none in both styles;
        // spaced outline items keep shadow-xs (Vega here — the base
        // recipe; nova spaced shadow is the toggle line's S-1 fix).
        // Shadow reality (rebuilt toggle paint): `paint_shadow_layers`
        // draws the cached shadow texture as 8 atlas quads per shadowed
        // widget (4 corners + 4 edges, no center) — see `shadow_shapes`.
        let mut states = vec![false, false];
        let fused = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_fused_shadow")
                .variant(ToggleVariant::Outline)
                .item("A")
                .item("B")
                .show(ui);
        });
        assert_eq!(shadow_shapes(&fused).len(), 0);

        let mut states = vec![false, false];
        let spaced = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_spaced_shadow")
                .spacing(2.0)
                .variant(ToggleVariant::Outline)
                .item("A")
                .item("B")
                .show(ui);
        });
        // Each spaced item paints exactly its own 8-quad shadow: quads
        // are attributed to items by intersection with the item's border
        // rect (quads straddle the rect edge and never reach a neighbour
        // — the 8px gap exceeds the shadow extent).
        let quads = shadow_shapes(&spaced);
        let borders = border_strokes_by_item(&spaced);
        assert_eq!(borders.len(), 2);
        for (rect, _, _) in &borders {
            let item_quads: Vec<Rect> = quads
                .iter()
                .copied()
                .filter(|q| q.intersects(*rect))
                .collect();
            assert_eq!(item_quads.len(), 8, "expected 8 atlas quads for {rect:?}");
            // Every quad extends beyond the item rect: the shadow is a
            // halo, not an interior paint (SEL-VIS-01).
            for q in &item_quads {
                assert!(
                    q.min.x < rect.min.x
                        || q.min.y < rect.min.y
                        || q.max.x > rect.max.x
                        || q.max.y > rect.max.y,
                    "shadow quad {q:?} does not extend beyond {rect:?}"
                );
            }
        }
        // No quad is unattributed: the total is exactly the two items'
        // quads (no stray shadow paint, no double-counting).
        assert_eq!(quads.len(), 16);
    }

    #[test]
    fn per_item_invalid_forwards_destructive_border_no_rest_ring() {
        // Spec §4.7 + N17 option (a): invalid is per-item; the invalid
        // border is full destructive; the invalid ring is focus-gated
        // (no rest ring) in both styles.
        let theme = Preset {
            style: Style::Vega,
            ..Preset::default()
        }
        .build(Mode::Light);
        let destructive = theme.palette.destructive;
        let mut states = vec![false, false, false];
        let out = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_invalid")
                .variant(ToggleVariant::Outline)
                .add_item(GroupItem::new("A"))
                .add_item(GroupItem::new("B").invalid(true))
                .add_item(GroupItem::new("C"))
                .show(ui);
        });
        let raw = border_strokes(&out);
        // Clip-based fused stroke: First paints 1 ring shape, Middle/Last
        // paint 3 each (1 main clip + 2 corner cells) — see
        // `border_strokes_by_item`.
        assert_eq!(raw.len(), 7);
        let borders = border_strokes_by_item(&out);
        assert_eq!(borders.len(), 3);
        assert_eq!(borders[1].1, destructive);
        assert_ne!(borders[0].1, destructive);
        assert_ne!(borders[2].1, destructive);
        // No rest ring: no 3px ring shape on any unfocused item.
        assert_eq!(ring_shapes(&out).len(), 0);
    }

    #[test]
    fn mixed_sizes_via_resolution_quirk() {
        // Spec §8.1/§9 case 8: a group with no size prop lets each item
        // use its own size; heights follow the size table (Vega: sm 32,
        // default 36, lg 40).
        let mut states = vec![false, false, false];
        let out = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_mixed")
                .variant(ToggleVariant::Outline)
                .add_item(GroupItem::new("A").size(ToggleSize::Sm))
                .add_item(GroupItem::new("B"))
                .add_item(GroupItem::new("C").size(ToggleSize::Lg))
                .show(ui);
        });
        let raw = border_strokes(&out);
        // Clip-based fused stroke: First paints 1 ring shape, Middle/Last
        // paint 3 each (1 main clip + 2 corner cells) — see
        // `border_strokes_by_item`.
        assert_eq!(raw.len(), 7);
        let borders = border_strokes_by_item(&out);
        assert_eq!(borders.len(), 3);
        assert!((borders[0].0.height() - 32.0).abs() < 0.01);
        assert!((borders[1].0.height() - 36.0).abs() < 0.01);
        assert!((borders[2].0.height() - 40.0).abs() < 0.01);
    }

    /// Spec §3.2 (`items-center`) + §8.1: mixed-height items are
    /// vertically centered — every item's vertical center equals the
    /// group's vertical center (the tallest item's center). TGG-VIS-01:
    /// with the tallest item LAST, shorter items placed earlier kept the
    /// center of the 36pt seed row instead of the final row center
    /// (2px high under base styles). Spaced groups: each item paints
    /// exactly one border stroke in both the old and the rebuilt toggle
    /// paint (position `Only`, no clip cells), so these pins hold on the
    /// merged tree too.
    fn centered_border_rects(style: Style, sizes: [ToggleSize; 3]) -> Vec<Rect> {
        let mut states = vec![false, false, false];
        let out = run_frames_output(style, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_centering")
                .variant(ToggleVariant::Outline)
                .spacing(2.0)
                .add_item(GroupItem::new("A").size(sizes[0]))
                .add_item(GroupItem::new("B").size(sizes[1]))
                .add_item(GroupItem::new("C").size(sizes[2]))
                .show(ui);
        });
        let mut rects: Vec<Rect> = border_strokes(&out)
            .into_iter()
            .map(|(r, _, _)| r)
            .collect();
        assert_eq!(rects.len(), 3, "expected one border stroke per item");
        rects.sort_by(|a, b| a.min.x.partial_cmp(&b.min.x).unwrap());
        rects
    }

    #[test]
    fn mixed_heights_center_on_final_row_height_tallest_last() {
        // TGG-VIS-01: tallest item last (the §6a demo order). Pre-fix the
        // Sm/Default centers sat 2px above the Lg center under Vega.
        let rects = centered_border_rects(
            Style::Vega,
            [ToggleSize::Sm, ToggleSize::Default, ToggleSize::Lg],
        );
        assert!((rects[0].height() - 32.0).abs() < 0.01);
        assert!((rects[1].height() - 36.0).abs() < 0.01);
        assert!((rects[2].height() - 40.0).abs() < 0.01);
        let center = rects[2].center().y;
        for (i, rect) in rects.iter().enumerate() {
            assert!(
                (rect.center().y - center).abs() < 0.01,
                "item {i} center {} != tallest center {center}",
                rect.center().y
            );
        }
    }

    #[test]
    fn mixed_heights_center_tallest_first() {
        // Tallest item first: centered pre-fix too; the fix must not move it.
        let rects = centered_border_rects(
            Style::Vega,
            [ToggleSize::Lg, ToggleSize::Default, ToggleSize::Sm],
        );
        let center = rects[0].center().y;
        for (i, rect) in rects.iter().enumerate() {
            assert!(
                (rect.center().y - center).abs() < 0.01,
                "item {i} center {} != tallest center {center}",
                rect.center().y
            );
        }
    }

    #[test]
    fn mixed_heights_center_tallest_middle() {
        let rects = centered_border_rects(
            Style::Vega,
            [ToggleSize::Sm, ToggleSize::Lg, ToggleSize::Default],
        );
        let center = rects[1].center().y;
        for (i, rect) in rects.iter().enumerate() {
            assert!(
                (rect.center().y - center).abs() < 0.01,
                "item {i} center {} != tallest center {center}",
                rect.center().y
            );
        }
    }

    #[test]
    fn mixed_heights_center_nova() {
        // Nova heights (28/32/36) never exceed the 36pt seed row, so
        // centering held pre-fix; the fix must keep it exact.
        let rects = centered_border_rects(
            Style::Nova,
            [ToggleSize::Sm, ToggleSize::Default, ToggleSize::Lg],
        );
        assert!((rects[0].height() - 28.0).abs() < 0.01);
        assert!((rects[1].height() - 32.0).abs() < 0.01);
        assert!((rects[2].height() - 36.0).abs() < 0.01);
        let center = rects[2].center().y;
        for (i, rect) in rects.iter().enumerate() {
            assert!(
                (rect.center().y - center).abs() < 0.01,
                "item {i} center {} != tallest center {center}",
                rect.center().y
            );
        }
    }

    #[test]
    fn uniform_heights_centered() {
        let rects = centered_border_rects(
            Style::Vega,
            [
                ToggleSize::Default,
                ToggleSize::Default,
                ToggleSize::Default,
            ],
        );
        let center = rects[0].center().y;
        for (i, rect) in rects.iter().enumerate() {
            assert!((rect.height() - 36.0).abs() < 0.01);
            assert!(
                (rect.center().y - center).abs() < 0.01,
                "item {i} center {} != row center {center}",
                rect.center().y
            );
        }
    }

    #[test]
    fn group_size_wins_over_item_sizes() {
        // Spec §8.1: the group's value wins whenever it is defined.
        let mut states = vec![false, false];
        let out = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_group_wins")
                .variant(ToggleVariant::Outline)
                .size(ToggleSize::Sm)
                .add_item(GroupItem::new("A").size(ToggleSize::Lg))
                .add_item(GroupItem::new("B").size(ToggleSize::Lg))
                .show(ui);
        });
        let raw = border_strokes(&out);
        // Clip-based fused stroke: First paints 1 ring shape, Last paints
        // 3 (1 main clip + 2 corner cells) — see `border_strokes_by_item`.
        assert_eq!(raw.len(), 4);
        let borders = border_strokes_by_item(&out);
        assert_eq!(borders.len(), 2);
        for (rect, _, _) in &borders {
            assert!((rect.height() - 32.0).abs() < 0.01);
        }
    }

    #[test]
    fn disabled_group_renders_faded_items() {
        // The whole-element fade itself is Toggle's paint (toggle spec
        // §4.6, group fade per the coordinator ruling — toggle S1's
        // scope); the group contract pinned here is that the disabled
        // flag reaches every item and changes its render.
        let mut states = vec![true, true];
        let enabled = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_enabled")
                .variant(ToggleVariant::Default)
                .item("A")
                .item("B")
                .show(ui);
        });
        let mut states = vec![true, true];
        let disabled = run_frames_output(Style::Vega, Mode::Light, 3, |ui| {
            ToggleGroup::multiple(&mut states)
                .id_salt("tg_disabled")
                .disabled(true)
                .variant(ToggleVariant::Default)
                .item("A")
                .item("B")
                .show(ui);
        });
        // Compare fill colours via the raw shapes (fill_shapes drops the
        // colour); both groups paint two pressed fills.
        let enabled_fills: Vec<Color32> = enabled
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Rect(rs)
                    if rs.fill != Color32::TRANSPARENT && rs.stroke == Stroke::NONE =>
                {
                    Some(rs.fill)
                }
                _ => None,
            })
            .collect();
        let disabled_fills: Vec<Color32> = disabled
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Rect(rs)
                    if rs.fill != Color32::TRANSPARENT && rs.stroke == Stroke::NONE =>
                {
                    Some(rs.fill)
                }
                _ => None,
            })
            .collect();
        assert_eq!(enabled_fills.len(), 2);
        assert_eq!(disabled_fills.len(), 2);
        // The disabled render differs from the enabled render for every
        // item. The exact composite values are Toggle's paint (toggle
        // spec §4.6) and S5's judgement — this test pins only that the
        // group flag reaches the items.
        for (e, d) in enabled_fills.iter().zip(disabled_fills.iter()) {
            assert_ne!(e, d, "disabled fill must differ from enabled fill");
        }
    }

    #[test]
    fn settled_frames_are_deterministic() {
        // The group adds no animation state of its own (spec §6: group
        // level none, settled). Two settled frames with identical input
        // paint identical shapes.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut states = vec![true, false, true];
        let mut frames = Vec::new();
        for _ in 0..3 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("tg_determinism"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0))),
                );
                ToggleGroup::multiple(&mut states)
                    .id_salt("tg_determinism")
                    .variant(ToggleVariant::Outline)
                    .item("A")
                    .item("B")
                    .item("C")
                    .show(ui);
            });
            out.textures_delta.clear();
            frames.push((fill_shapes(&out), border_strokes(&out)));
        }
        // Frames 2 and 3 are both settled (the protocol learns the child
        // count on frame 1); their extracted geometry must be identical.
        assert_eq!(frames[1], frames[2]);
    }
}
