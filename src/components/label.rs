//! shadcn `label.tsx` — a form label, optionally wired to a control.
//!
//! Rebuilt to the frozen Batch A S0 spec (`~/workspace/batches-a-h/specs/
//! label-spec.md`, Coordinator-signed 2026-11-15). The original is one
//! `Label` function component wrapping Radix `LabelPrimitive.Root`
//! (`label.tsx`, 23 lines), with a single class string and no variants:
//!
//! ```text
//! flex items-center gap-2 text-sm leading-none font-medium select-none
//! group-data-[disabled=true]:pointer-events-none
//! group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed
//! peer-disabled:opacity-50
//! ```
//!
//! (`label.tsx:15`.) The nova rules (`.cn-label`, `.cn-label-aria`,
//! `style-nova.css:789-795`) restate a subset of the base string and are
//! silent on the rest, so under the settled LAYER ruling nova renders
//! identically to base in every state the snapshot establishes (spec
//! §2.3). There is no colour, background, border, radius, padding,
//! shadow, icon-sizing, hover, focus, or animation rule in either
//! recipe: the label paints text (and children) only, at the inherited
//! text colour, mapped to the `foreground` token (spec §5.1 — a mapping,
//! not a class-string fact).
//!
//! Click behaviour: a label bound with [`Label::for_id`] (the `htmlFor`
//! concept, spec §8) forwards clicks to the named control. Whether the
//! original's forwarding ACTIVATEs toggleable controls or only focuses
//! them is OPEN-1 (spec §7.2, the N6 question) — unestablished from the
//! snapshot, and this implementation's focus-only behaviour is recorded
//! without any parity claim. S6 settles OPEN-1 from Radix source.
//!
//! The disabled flag ([`Label::disabled`]) is proposed deviation D1
//! (spec §10, Expert Reviewer approval pending): egui has no CSS
//! sibling/ancestor selectors, so the group/peer disabled conditions
//! are expressed as one explicit flag applying the group-disabled
//! outcome (50% opacity + non-interactive).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Label::new("Email").for_id(email_id).show(ui);
//! Label::new("Profile").icon(Icon::User).show(ui);
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, fonts, Palette, ShadcnTheme};
use egui::{
    Align, Align2, Color32, FontId, Id, Layout, Pos2, Rect, Response, Sense, Ui, UiBuilder, Vec2,
    Widget,
};

/// `gap-2` (spec §3.2): 8px between every pair of flex children.
const GAP: f32 = crate::theme::tokens::spacing::SM;

/// Leading/trailing icon box. A builder API decision, NOT a spec
/// requirement: the label recipe contains no `[&_svg]` sizing rule
/// (spec §3.3), so icons keep their own size; the 16px slot is the size
/// this API renders its icon slots at.
const ICON_SIZE: f32 = crate::theme::tokens::spacing::LG;

/// `leading-none` text line box (spec §3.1): 1× the 14px `text-sm` size.
const LINE_H: f32 = crate::theme::tokens::type_scale::SM;

/// The label's font: `text-sm` (14px) at `font-medium` (500), spec §5.2.
/// The concrete family behind `--font-sans` is OPEN-4; the rendered face
/// is the Phase 1 embedded Inter Medium, not snapshot truth.
pub(crate) fn label_font() -> FontId {
    fonts::medium(crate::theme::tokens::type_scale::SM)
}

/// Label text color: the `foreground` slot (spec §5.1 mapping), 50% alpha
/// when disabled (`group-data-[disabled=true]:opacity-50` / D1). The same
/// color paints text and icon children uniformly (element-level opacity,
/// spec §4).
pub(crate) fn text_color(p: &Palette, disabled: bool) -> Color32 {
    if disabled {
        color_alpha(p.foreground, 0.5)
    } else {
        p.foreground
    }
}

/// Flex-row content size (spec §3.2): child widths sum plus `gap-2` (8px)
/// between every pair of PRESENT children; height is the tallest child.
/// The text child exists only when the text is non-empty, so an icon-only
/// label is exactly the icon box and an empty label is zero-size
/// (spec §9). No padding, border, or chrome is added (spec §3.4) — the
/// widget allocates exactly this size.
pub(crate) fn content_size(text_w: f32, has_text: bool, icon_start: bool, icon_end: bool) -> Vec2 {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut children = 0usize;
    if icon_start {
        width += ICON_SIZE;
        height = height.max(ICON_SIZE);
        children += 1;
    }
    if has_text {
        width += text_w;
        height = height.max(LINE_H);
        children += 1;
    }
    if icon_end {
        width += ICON_SIZE;
        height = height.max(ICON_SIZE);
        children += 1;
    }
    if children > 1 {
        width += (children - 1) as f32 * GAP;
    }
    Vec2::new(width, height)
}

/// The label's interaction sense (spec §7). The label is never focusable
/// and never takes a Tab stop (§7.3, OPEN-6): the clickable case uses
/// [`Sense::CLICK`] — click without `FOCUSABLE` — never [`Sense::click`],
/// which would register a focusable widget. Clicks are sensed only when
/// the label names a target and is enabled; otherwise the label is
/// hover-only (`select-none`, no pointer response of its own).
pub(crate) fn label_sense(has_target: bool, disabled: bool) -> Sense {
    if has_target && !disabled {
        Sense::CLICK
    } else {
        Sense::hover()
    }
}

/// A shadcn label. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Label {
    text: String,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    disabled: bool,
    for_id: Option<Id>,
    id_salt: Option<String>,
}

impl Label {
    /// Creates a label with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            icon_end: None,
            disabled: false,
            for_id: None,
            id_salt: None,
        }
    }

    /// Shows a 16px icon before the text (the flex row's first child,
    /// separated by `gap-2`). The 16px box is a builder API decision:
    /// the recipe defines no icon size (spec §3.3).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows a 16px icon after the text (the flex row's last child,
    /// separated by `gap-2`). Same builder-decision size as [`Label::icon`].
    /// Together the two slots cover the full §3.2 table (icon + text,
    /// text + icon, icon + text + icon).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Binds the label to a control id: clicking the label moves keyboard
    /// focus to that widget, like Radix `htmlFor` (spec §8). `htmlFor` has
    /// no default: a label without a target has no associated control.
    ///
    /// Whether the original also ACTIVATEs toggleable targets is OPEN-1
    /// (spec §7.2) and is not claimed here: this implementation forwards
    /// focus only.
    pub fn for_id(mut self, id: Id) -> Self {
        self.for_id = Some(id);
        self
    }

    /// Disables the label: 50% opacity, non-interactive (proposed
    /// deviation D1, spec §10 — Expert Reviewer approval pending). The
    /// flag is the caller's expression of the group/peer disabled
    /// conditions, which egui cannot express declaratively; the visual
    /// rule (50% opacity, instant, no fade) is frozen either way. The
    /// peer-only cursor distinction is not separately expressible (D5).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt (Label keeps no state). Only affects
    /// [`Label::show`]; [`Label::show_with`] registers no interact and
    /// ignores the salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the label: leading icon, text, trailing icon as flex
    /// children (spec §§2–3). Nothing animates (spec §6); no repaint is
    /// requested.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let font = label_font();
        // Placeholder color: the paint-time fallback replaces it exactly,
        // so this one galley serves both sizing and painting.
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font, Color32::PLACEHOLDER);

        let has_text = !self.text.is_empty();
        let text_w = galley.size().x;
        let desired = content_size(
            text_w,
            has_text,
            self.icon.is_some(),
            self.icon_end.is_some(),
        );

        let sense = label_sense(self.for_id.is_some(), self.disabled);
        let interactive = self.for_id.is_some() && !self.disabled;

        let (rect, response) = {
            // Reserve space without registering an auto-id focusable
            // response, so egui's spatial arrow-key navigation only sees the
            // explicit widget id (checkbox.rs pattern).
            let (_, rect) = ui.allocate_space(desired);
            let id = match self.id_salt.as_deref() {
                Some(salt) => ui.scope_id().with(salt),
                None => ui.auto_id_with("shadcn_label"),
            };
            (rect, ui.interact(rect, id, sense))
        };

        // Focus-only forwarding (OPEN-1 unestablished; no parity claim).
        if interactive && response.clicked() {
            if let Some(target) = self.for_id {
                ui.memory_mut(|mem| mem.request_focus(target));
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let color = text_color(&p, self.disabled);
            let center_y = rect.center().y;
            // Gaps exist only between present children (spec §3.2).
            let mut cursor = rect.left();
            let mut need_gap = false;
            if let Some(icon) = self.icon {
                if need_gap {
                    cursor += GAP;
                }
                let r = Rect::from_center_size(
                    Pos2::new(cursor + ICON_SIZE * 0.5, center_y),
                    Vec2::splat(ICON_SIZE),
                );
                icon.paint(painter, r, color, 2.0 * ICON_SIZE / 24.0);
                cursor += ICON_SIZE;
                need_gap = true;
            }
            if has_text {
                if need_gap {
                    cursor += GAP;
                }
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = Align2::CENTER_CENTER
                    .anchor_size(Pos2::new(cursor + text_w * 0.5, center_y), galley.size());
                painter.galley(text_rect.min, galley, color);
                cursor += text_w;
                need_gap = true;
            }
            if let Some(icon) = self.icon_end {
                if need_gap {
                    cursor += GAP;
                }
                let r = Rect::from_center_size(
                    Pos2::new(cursor + ICON_SIZE * 0.5, center_y),
                    Vec2::splat(ICON_SIZE),
                );
                icon.paint(painter, r, color, 2.0 * ICON_SIZE / 24.0);
            }
        }

        response
    }

    /// Renders the label with arbitrary flex children: the label's own
    /// leading icon, text, and trailing icon (when present), plus the
    /// widgets `content` adds, laid out as one flex row with `gap-2`
    /// (8px) between children and vertical centering (`items-center`,
    /// spec §2). This is the Rust analogue of the original's arbitrary
    /// JSX children, including nested controls (spec §8, OPEN-5) — the
    /// S1 API proposal, pending Coordinator confirmation.
    ///
    /// The closure runs TWICE (measure pass, then real pass — the
    /// `dialog_footer` pattern): the measure pass runs in an invisible,
    /// disabled child `Ui` so no callback can double-fire, and the real
    /// pass centers every child on the measured row height. Closure
    /// content must therefore be a pure function of its inputs, and must
    /// not depend on the available width for its height.
    ///
    /// Clicks anywhere in the row forward focus to the [`Label::for_id`]
    /// target (OPEN-1: focus-only, no parity claim), detected from
    /// pointer state rather than a click-sense interact, so no
    /// click-sense rect steals hits from nested widgets (the
    /// `FieldLabel::show_with_card` pattern). Nested interactive children
    /// keep their own click handling. When disabled (D1), the whole row
    /// renders at 50% opacity via row opacity and no click is forwarded;
    /// nested children keep their own enabled state (the caller disables
    /// the nested control itself, as on the web).
    pub fn show_with<R>(
        self,
        ui: &mut Ui,
        mut content: impl FnMut(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let Self {
            text,
            icon,
            icon_end,
            disabled,
            for_id,
            id_salt: _,
        } = self;
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // Full-strength color: the disabled dimming comes from the row
        // opacity in the real pass, so closure children dim uniformly too.
        let color = p.foreground;
        let has_text = !text.is_empty();

        // One row rendering, shared by the measure and real passes: the
        // label's own children plus the closure's, in DOM order.
        let mut render = |ui: &mut Ui| -> R {
            ui.spacing_mut().item_spacing.x = GAP;
            if disabled {
                ui.set_opacity(0.5);
            }
            if let Some(icon) = icon {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(ICON_SIZE), Sense::hover());
                if ui.is_rect_visible(rect) {
                    icon.paint(ui.painter(), rect, color, 2.0 * ICON_SIZE / 24.0);
                }
            }
            if has_text {
                let galley =
                    ui.painter()
                        .layout_no_wrap(text.clone(), label_font(), Color32::PLACEHOLDER);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(galley.size().x, LINE_H), Sense::hover());
                if ui.is_rect_visible(rect) {
                    let text_rect = Align2::CENTER_CENTER.anchor_size(rect.center(), galley.size());
                    ui.painter().galley(text_rect.min, galley, color);
                }
            }
            let ret = content(ui);
            if let Some(icon) = icon_end {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(ICON_SIZE), Sense::hover());
                if ui.is_rect_visible(rect) {
                    icon.paint(ui.painter(), rect, color, 2.0 * ICON_SIZE / 24.0);
                }
            }
            ret
        };

        // Measure pass (dialog_footer pattern): cross-align Min keeps the
        // measured frame widget-sized; Center would span the full
        // (infinite) available height.
        let flow_min = ui.cursor().min;
        let flow_w = ui.available_width();
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
        let mut measure = ui.new_child(UiBuilder::new().max_rect(measure_rect).invisible());
        measure.with_layout(Layout::left_to_right(Align::Min), |ui| render(ui));
        let measured = measure.min_rect().size();

        // Real pass: the row rect is the measured size, so Align::Center
        // centers every child on the row's center line (items-center).
        let row =
            ui.allocate_ui_with_layout(measured, Layout::left_to_right(Align::Center), |ui| {
                render(ui)
            });
        let row_rect = row.response.rect;

        // `<label>` semantics, read from pointer state so no click-sense
        // interact steals hits from nested widgets.
        if let Some(target) = for_id {
            if !disabled {
                let clicked = ui.input(|i| {
                    i.pointer.any_click()
                        && i.pointer
                            .press_origin()
                            .is_some_and(|o| row_rect.contains(o))
                });
                if clicked {
                    ui.memory_mut(|mem| mem.request_focus(target));
                }
            }
        }

        row
    }
}

impl Widget for Label {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{fonts, Mode, Preset};

    #[test]
    fn label_font_is_medium_14() {
        let font = label_font();
        assert_eq!(font.size, 14.0);
        assert_eq!(font.family, fonts::medium_family());
        assert_eq!(font, fonts::medium(14.0));
    }

    #[test]
    fn content_size_matches_gap_arithmetic() {
        // Spec §3.2 table, text advance W = 50, icon box I = 16, gap = 8.
        let w = 50.0;
        assert_eq!(content_size(w, true, false, false), Vec2::new(50.0, 14.0));
        assert_eq!(content_size(w, true, true, false), Vec2::new(74.0, 16.0));
        assert_eq!(content_size(w, true, false, true), Vec2::new(74.0, 16.0));
        assert_eq!(content_size(w, true, true, true), Vec2::new(98.0, 16.0));
    }

    #[test]
    fn content_size_icon_only_and_empty() {
        // Icon-only: exactly the icon box, no text line box (spec §9).
        assert_eq!(content_size(0.0, false, true, false), Vec2::new(16.0, 16.0));
        assert_eq!(content_size(0.0, false, false, true), Vec2::new(16.0, 16.0));
        // Empty label: zero-size (spec §9).
        assert_eq!(content_size(0.0, false, false, false), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn content_size_height_is_tallest_child() {
        // Text-only height is the 14px line box; a 16px icon beside 14px
        // text makes the row 16px tall (spec §3.2).
        assert_eq!(content_size(30.0, true, false, false).y, 14.0);
        assert_eq!(content_size(30.0, true, true, false).y, 16.0);
    }

    #[test]
    fn sense_is_never_focusable() {
        // Spec §7.3: the label adds no Tab stop in any state (OPEN-6 is
        // verified at runtime by S6; the sense itself must never carry
        // FOCUSABLE).
        for (has_target, disabled) in [(false, false), (true, false), (false, true), (true, true)] {
            let sense = label_sense(has_target, disabled);
            assert!(!sense.is_focusable(), "{has_target} {disabled}");
            assert_eq!(sense.senses_click(), has_target && !disabled);
        }
    }

    #[test]
    fn disabled_label_is_half_alpha() {
        for mode in [Mode::Light, Mode::Dark] {
            let p = Preset::default().build(mode).palette;
            assert_eq!(text_color(&p, false), p.foreground);
            assert_eq!(text_color(&p, true), color_alpha(p.foreground, 0.5));
        }
    }

    #[test]
    fn builder_fields() {
        let label = Label::new("Email")
            .icon(Icon::User)
            .icon_end(Icon::Check)
            .for_id(Id::unique("email"))
            .disabled(true)
            .id_salt("email_label");
        assert_eq!(label.text, "Email");
        assert!(label.icon.is_some());
        assert!(label.icon_end.is_some());
        assert!(label.disabled);
        assert_eq!(label.for_id, Some(Id::unique("email")));
        assert_eq!(label.id_salt.as_deref(), Some("email_label"));
    }
}
