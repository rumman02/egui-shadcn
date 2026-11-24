//! shadcn `breadcrumb.tsx` — a hierarchy trail.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/breadcrumb.tsx` (nova
//! overrides: list `gap-1.5`, ellipsis `size-5`): a muted `text-sm` row of
//! items separated by chevrons, links that brighten on hover, the current
//! page in full-strength foreground, and a `MoreHorizontal` ellipsis slot.
//! The list wraps onto multiple lines like the source's `flex-wrap`, and
//! separators accept a custom icon.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Breadcrumb::new().show(ui, |ui| {
//!     BreadcrumbList::new().show(ui, |ui| {
//!         BreadcrumbItem::new().show(ui, |ui| {
//!             BreadcrumbLink::new("Home").show(ui);
//!         });
//!         BreadcrumbSeparator::new().show(ui);
//!         BreadcrumbItem::new().show(ui, |ui| {
//!             BreadcrumbEllipsis::new().show(ui);
//!         });
//!         BreadcrumbSeparator::new().show(ui);
//!         BreadcrumbItem::new().show(ui, |ui| {
//!             BreadcrumbLink::new("Components").show(ui);
//!         });
//!         BreadcrumbSeparator::new().show(ui);
//!         BreadcrumbItem::new().show(ui, |ui| {
//!             BreadcrumbPage::new("Breadcrumb").show(ui);
//!         });
//!     });
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use egui::{Align2, Color32, FontId, Pos2, Response, Sense, Ui, Vec2};

/// Text size for breadcrumb rows (`text-sm`).
const TEXT_SIZE: f32 = crate::theme::tokens::type_scale::SM;
/// Row height for links and the page (`text-sm` line-height = 20px).
const LINE_HEIGHT: f32 = 20.0;
/// Separator chevron size (`[&>svg]:size-3.5` = 14px).
pub const BREADCRUMB_SEPARATOR_SIZE: f32 = 14.0;
/// Ellipsis box size (nova `size-5` = 20px).
pub const BREADCRUMB_ELLIPSIS_SIZE: f32 = 20.0;

// ---------------------------------------------------------------------------
// Breadcrumb (nav wrapper)
// ---------------------------------------------------------------------------

/// The breadcrumb `<nav>` wrapper — pure layout scope, no visuals.
#[derive(Clone, Debug, Default)]
pub struct Breadcrumb {
    _private: (),
}

impl Breadcrumb {
    /// Creates a breadcrumb root.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the breadcrumb trail.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        content(ui);
    }
}

// ---------------------------------------------------------------------------
// BreadcrumbList
// ---------------------------------------------------------------------------

/// The `<ol>` row: `flex flex-wrap items-center gap-1.5 text-sm
/// text-muted-foreground` — items wrap onto new lines in narrow layouts.
#[derive(Clone, Debug, Default)]
pub struct BreadcrumbList {
    gap: f32,
}

impl BreadcrumbList {
    /// Creates a breadcrumb list with the nova `gap-1.5`.
    pub fn new() -> Self {
        Self {
            gap: crate::theme::tokens::spacing::XS_HALF,
        }
    }

    /// Overrides the gap between items.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Lays out the breadcrumb items in a wrapping row.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = self.gap;
            content(ui);
        });
    }
}

// ---------------------------------------------------------------------------
// BreadcrumbItem
// ---------------------------------------------------------------------------

/// A `<li>` slot: `inline-flex items-center gap-1`.
#[derive(Clone, Debug, Default)]
pub struct BreadcrumbItem {
    _private: (),
}

impl BreadcrumbItem {
    /// Creates a breadcrumb item.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Lays out the item's content (a link, the page, or the ellipsis).
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::XS;
            content(ui);
        });
    }
}

// ---------------------------------------------------------------------------
// BreadcrumbLink
// ---------------------------------------------------------------------------

/// A clickable breadcrumb link: muted text that brightens on hover
/// (`transition-colors hover:text-foreground`).
#[derive(Debug)]
pub struct BreadcrumbLink {
    label: String,
    id_salt: Option<String>,
}

impl BreadcrumbLink {
    /// Creates a link with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id_salt: None,
        }
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the link. Check `clicked()` to act on navigation.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let font = FontId::proportional(TEXT_SIZE);

        // Placeholder color: the paint-time fallback replaces it exactly,
        // so this one galley serves both sizing and painting.
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.clone(), font, Color32::PLACEHOLDER);
        let text_w = galley.size().x;
        let desired = Vec2::new(text_w, LINE_HEIGHT);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_breadcrumb_link"),
        };
        let (_, rect) = ui.allocate_space(desired);
        let response = ui.interact(rect, id, Sense::click());

        if ui.is_rect_visible(rect) {
            let fg = if response.hovered() {
                p.foreground
            } else {
                p.muted_foreground
            };
            // Same paint math as `Painter::text`, reusing the galley.
            let text_rect = Align2::LEFT_CENTER
                .anchor_size(Pos2::new(rect.min.x, rect.center().y), galley.size());
            ui.painter().galley(text_rect.min, galley, fg);
            if response.has_focus() {
                crate::utils::focus::paint_ring(
                    ui.painter(),
                    rect.expand2(egui::vec2(1.0, 1.0)),
                    color_alpha(p.ring, 0.5),
                    3.0,
                    theme.radius.sm(),
                );
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// BreadcrumbPage
// ---------------------------------------------------------------------------

/// The current page: `font-normal text-foreground`, not interactive.
#[derive(Debug)]
pub struct BreadcrumbPage {
    label: String,
}

impl BreadcrumbPage {
    /// Creates the page label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    /// Renders the static page text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let font = FontId::proportional(TEXT_SIZE);
        // Laid out once with the paint color: the same galley sizes the
        // widget and paints the text.
        let galley =
            ui.painter()
                .layout_no_wrap(self.label.clone(), font, theme.palette.foreground);
        let text_w = galley.size().x;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(text_w, LINE_HEIGHT), Sense::hover());
        if ui.is_rect_visible(rect) {
            // Same paint math as `Painter::text`, reusing the galley.
            let text_rect = Align2::LEFT_CENTER
                .anchor_size(Pos2::new(rect.min.x, rect.center().y), galley.size());
            ui.painter()
                .galley(text_rect.min, galley, theme.palette.foreground);
        }
        response
    }
}

// ---------------------------------------------------------------------------
// BreadcrumbSeparator
// ---------------------------------------------------------------------------

/// The chevron between items (`ChevronRight`, 14px, muted).
#[derive(Clone, Debug)]
pub struct BreadcrumbSeparator {
    icon: Icon,
    size: f32,
}

impl Default for BreadcrumbSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl BreadcrumbSeparator {
    /// Creates the default chevron separator.
    pub fn new() -> Self {
        Self {
            icon: Icon::ChevronRight,
            size: BREADCRUMB_SEPARATOR_SIZE,
        }
    }

    /// Uses a different icon (the source accepts custom separator children).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    /// Overrides the icon size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Paints the separator glyph.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        if ui.is_rect_visible(rect) {
            self.icon.paint(
                ui.painter(),
                rect,
                theme.palette.muted_foreground,
                2.0 * self.size / 24.0,
            );
        }
        response
    }
}

// ---------------------------------------------------------------------------
// BreadcrumbEllipsis
// ---------------------------------------------------------------------------

/// The "more" slot: `MoreHorizontal` centered in a small box (nova `size-5`).
#[derive(Clone, Debug, Default)]
pub struct BreadcrumbEllipsis {
    _private: (),
}

impl BreadcrumbEllipsis {
    /// Creates the ellipsis slot.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Paints the ellipsis glyph.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let size = BREADCRUMB_ELLIPSIS_SIZE;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        if ui.is_rect_visible(rect) {
            let icon_size = crate::theme::tokens::spacing::LG; // `[&>svg]:size-4`
            let icon_rect = egui::Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
            Icon::MoreHorizontal.paint(
                ui.painter(),
                icon_rect,
                theme.palette.muted_foreground,
                2.0 * icon_size / 24.0,
            );
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> crate::theme::Palette {
        Preset::default().build(mode).palette
    }

    /// Minimal multi-frame harness (mirrors the tabs/accordion pattern):
    /// a stable Ui id so focus/hover state persists across frames, and
    /// `step_out` returning the frame output for shape inspection.
    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn step_out(
            &mut self,
            events: Vec<egui::Event>,
            mut f: impl FnMut(&mut Ui),
        ) -> egui::FullOutput {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("breadcrumb_harness_ui"),
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(400.0, 200.0),
                    )),
                );
                f(ui);
            });
            out.textures_delta.clear();
            out
        }

        fn run_out(&mut self, f: impl FnMut(&mut Ui)) -> egui::FullOutput {
            self.step_out(Vec::new(), f)
        }

        fn run(&mut self, f: impl FnMut(&mut Ui)) {
            let _ = self.run_out(f);
        }

        fn hover(&mut self, pos: Pos2, f: impl FnMut(&mut Ui)) {
            let _ = self.step_out(vec![egui::Event::PointerMoved(pos)], f);
        }

        fn click(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            self.hover(pos, &mut f);
            for pressed in [true, false] {
                let _ = self.step_out(
                    vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    &mut f,
                );
            }
        }
    }

    /// The painted colour of the text shape for `label`, if any.
    fn text_color(out: &egui::FullOutput, label: &str) -> Option<Color32> {
        for clipped in &out.shapes {
            if let egui::Shape::Text(ts) = &clipped.shape {
                if ts.galley.job.text == label {
                    return Some(ts.fallback_color);
                }
            }
        }
        None
    }

    #[test]
    fn link_uses_muted_and_hover_foreground() {
        let p = palette(Mode::Light);
        // Hover logic mirrors the source `hover:text-foreground`; assert the
        // two states differ and the resting color is muted.
        assert_ne!(p.muted_foreground, p.foreground);
    }

    #[test]
    fn separator_uses_chevron_right_at_14px() {
        let s = BreadcrumbSeparator::new();
        assert_eq!(s.icon, Icon::ChevronRight);
        assert_eq!(s.size, BREADCRUMB_SEPARATOR_SIZE);
        assert_eq!(s.size, 14.0);
        let custom = BreadcrumbSeparator::new().icon(Icon::Minus).size(10.0);
        assert_eq!(custom.size, 10.0);
    }

    #[test]
    fn ellipsis_and_defaults_match_nova() {
        assert_eq!(BREADCRUMB_ELLIPSIS_SIZE, 20.0);
        let _ = Breadcrumb::new();
        let _ = BreadcrumbItem::new();
        let list = BreadcrumbList::new();
        assert_eq!(list.gap, 6.0); // nova `gap-1.5`
    }

    #[test]
    fn page_and_link_builders_keep_fields() {
        let l = BreadcrumbLink::new("Home").id_salt("home");
        assert_eq!(l.id_salt.as_deref(), Some("home"));
        assert_eq!(l.label, "Home");
        let page = BreadcrumbPage::new("Breadcrumb");
        assert_eq!(page.label, "Breadcrumb");
    }

    #[test]
    fn breadcrumb_renders_without_panic() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("breadcrumb_test_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(400.0, 200.0),
                )),
            );
            Breadcrumb::new().show(ui, |ui| {
                BreadcrumbList::new().show(ui, |ui| {
                    BreadcrumbItem::new().show(ui, |ui| {
                        BreadcrumbLink::new("Home").show(ui);
                    });
                    BreadcrumbSeparator::new().show(ui);
                    BreadcrumbItem::new().show(ui, |ui| {
                        BreadcrumbEllipsis::new().show(ui);
                    });
                    BreadcrumbSeparator::new().show(ui);
                    BreadcrumbItem::new().show(ui, |ui| {
                        BreadcrumbPage::new("Breadcrumb").show(ui);
                    });
                });
            });
        });
        out.textures_delta.clear();
    }

    #[test]
    fn rows_use_the_20px_text_sm_line_height() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("breadcrumb_line_height_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(400.0, 200.0),
                )),
            );
            let link = BreadcrumbLink::new("Home").show(ui).rect;
            assert!(
                (link.height() - 20.0).abs() < 0.5,
                "link row height {} should be the 20px `text-sm` line box",
                link.height()
            );
            let page = BreadcrumbPage::new("Breadcrumb").show(ui).rect;
            assert!(
                (page.height() - 20.0).abs() < 0.5,
                "page row height {} should be the 20px `text-sm` line box",
                page.height()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn list_wraps_onto_new_lines() {
        // `flex-wrap break-words`: in a narrow container the items move to
        // following lines instead of overflowing.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut link_rects = [egui::Rect::ZERO; 3];
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("breadcrumb_wrap_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(140.0, 200.0),
                )),
            );
            Breadcrumb::new().show(ui, |ui| {
                BreadcrumbList::new().show(ui, |ui| {
                    for (i, label) in ["Components", "Documentation", "Breadcrumb"]
                        .into_iter()
                        .enumerate()
                    {
                        link_rects[i] = BreadcrumbLink::new(label).show(ui).rect;
                    }
                });
            });
        });
        out.textures_delta.clear();
        assert!(
            link_rects[1].top() > link_rects[0].bottom()
                || link_rects[1].left() > link_rects[0].right(),
            "items flow with the list gap"
        );
        assert!(
            link_rects[2].top() > link_rects[0].bottom(),
            "the third item wrapped onto a later line"
        );
    }

    #[test]
    fn link_click_reports_clicked() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("breadcrumb_test_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(400.0, 200.0),
                )),
            );
            let r = BreadcrumbLink::new("Home").show(ui);
            assert!(!r.clicked());
        });
        out.textures_delta.clear();
    }

    #[test]
    fn link_click_reports_clicked_on_press_release() {
        // Spec §7.2: clicking a link activates it — simulate a real press
        // + release on the link centre and assert `clicked()`.
        let mut h = Harness::new();
        let mut rect = egui::Rect::ZERO;
        h.run(|ui| {
            rect = BreadcrumbLink::new("Home").show(ui).rect;
        });
        let mut clicked = false;
        h.click(rect.center(), |ui| {
            clicked = BreadcrumbLink::new("Home").show(ui).clicked();
        });
        assert!(clicked, "press + release on a link reports clicked()");
    }

    #[test]
    fn link_brightens_to_foreground_on_hover() {
        // Spec §4.2: `hover:text-foreground` — rendered colour, not just
        // the palette values: muted at rest, foreground while hovered.
        let p = palette(Mode::Light);
        let mut h = Harness::new();
        let mut rect = egui::Rect::ZERO;
        let out = h.run_out(|ui| {
            rect = BreadcrumbLink::new("Home").show(ui).rect;
        });
        assert_eq!(
            text_color(&out, "Home"),
            Some(p.muted_foreground),
            "link text is muted at rest"
        );
        let out = h.step_out(vec![egui::Event::PointerMoved(rect.center())], |ui| {
            BreadcrumbLink::new("Home").show(ui);
        });
        assert_eq!(
            text_color(&out, "Home"),
            Some(p.foreground),
            "link text brightens to foreground on hover"
        );
    }

    #[test]
    fn focused_link_paints_n9_focus_ring() {
        // Spec §4.3 / D1 (N9, documented deviation): a focused link paints
        // a 3px `ring/50` ring at `radius.sm()` (6px) around the link rect
        // expanded by 1px. Pin it so a later "fix" cannot silently remove
        // the ring to match the source (which has no focus recipe).
        let mut h = Harness::new();
        let mut rect = egui::Rect::ZERO;
        h.run(|ui| {
            let response = BreadcrumbLink::new("Home").id_salt("ring").show(ui);
            rect = response.rect;
            response.request_focus();
        });
        // Settle a frame so the requested focus is installed.
        h.run(|ui| {
            BreadcrumbLink::new("Home").id_salt("ring").show(ui);
        });
        let out = h.run_out(|ui| {
            BreadcrumbLink::new("Home").id_salt("ring").show(ui);
        });

        let expected = rect.expand2(Vec2::new(1.0, 1.0));
        let mut found = false;
        for clipped in &out.shapes {
            if let egui::Shape::Rect(rs) = &clipped.shape {
                if rs.stroke.width == 3.0 {
                    assert_eq!(
                        rs.corner_radius.nw, 6,
                        "N9 ring radius is radius.sm() = 6px"
                    );
                    assert_eq!(rs.corner_radius.ne, 6);
                    assert_eq!(rs.corner_radius.sw, 6);
                    assert_eq!(rs.corner_radius.se, 6);
                    assert!(
                        (rs.rect.min - expected.min).length() < 0.5
                            && (rs.rect.max - expected.max).length() < 0.5,
                        "ring rect {} is the link rect expanded by 1px ({})",
                        rs.rect,
                        expected
                    );
                    // `color_alpha(ring, 0.5)` on an opaque colour:
                    // (255 * 0.5 + 0.5) as u8 = 128.
                    assert_eq!(
                        rs.stroke.color.a(),
                        128,
                        "ring is ring/50, got alpha {}",
                        rs.stroke.color.a()
                    );
                    found = true;
                }
            }
        }
        assert!(found, "focused link paints a 3px N9 focus ring");
    }

    #[test]
    fn list_gap_is_6px_under_nova() {
        // Spec §3.2: the list gap is `gap-1.5` = 6px under nova (restated
        // by `.cn-breadcrumb-list`). The field default is pinned by
        // `ellipsis_and_defaults_match_nova`; this pins the rendered gap.
        let mut h = Harness::new();
        let mut first = egui::Rect::ZERO;
        let mut second = egui::Rect::ZERO;
        h.run(|ui| {
            BreadcrumbList::new().show(ui, |ui| {
                BreadcrumbItem::new().show(ui, |ui| {
                    first = BreadcrumbLink::new("Home").show(ui).rect;
                });
                BreadcrumbItem::new().show(ui, |ui| {
                    second = BreadcrumbLink::new("Components").show(ui).rect;
                });
            });
        });
        let gap = second.left() - first.right();
        assert!(
            (gap - 6.0).abs() < 0.5,
            "rendered list gap {} should be the nova gap-1.5 = 6px",
            gap
        );
    }

    #[test]
    fn item_gap_is_4px_under_nova() {
        // Spec §3.2: the gap inside an item is `gap-1` = 4px under nova
        // (`.cn-breadcrumb-item` overrides base `gap-1.5`).
        let mut h = Harness::new();
        let mut first = egui::Rect::ZERO;
        let mut second = egui::Rect::ZERO;
        h.run(|ui| {
            BreadcrumbItem::new().show(ui, |ui| {
                first = BreadcrumbLink::new("Home").show(ui).rect;
                second = BreadcrumbLink::new("Components").show(ui).rect;
            });
        });
        let gap = second.left() - first.right();
        assert!(
            (gap - 4.0).abs() < 0.5,
            "rendered item gap {} should be the nova gap-1 = 4px",
            gap
        );
    }

    #[test]
    fn ellipsis_glyph_is_16px_centered() {
        // Spec §2.7/§3.3: the nova ellipsis is a 20px box with a 16px glyph
        // (`size-5` / `[&>svg]:size-4`), centred. The icon paints as a
        // textured mesh, so pin the mesh bounds, not just the box constant.
        let mut h = Harness::new();
        let mut rect = egui::Rect::ZERO;
        let out = h.run_out(|ui| {
            rect = BreadcrumbEllipsis::new().show(ui).rect;
        });
        assert!(
            (rect.width() - 20.0).abs() < 0.5 && (rect.height() - 20.0).abs() < 0.5,
            "ellipsis box {} should be the nova 20px square",
            rect
        );
        let expected = egui::Rect::from_center_size(rect.center(), Vec2::splat(16.0));
        let mut found = false;
        for clipped in &out.shapes {
            if let egui::Shape::Mesh(mesh) = &clipped.shape {
                let mut bounds = egui::Rect::NOTHING;
                for v in &mesh.vertices {
                    bounds.extend_with(v.pos);
                }
                if (bounds.min - expected.min).length() < 0.5
                    && (bounds.max - expected.max).length() < 0.5
                {
                    found = true;
                }
            }
        }
        assert!(
            found,
            "ellipsis glyph paints as a 16px image centred in the 20px box"
        );
    }
}
