//! shadcn `alert.tsx` — a callout for user attention.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/alert.tsx`:
//! `rounded-lg border px-4 py-3 text-sm` with `default` (`bg-card
//! text-card-foreground`) and `destructive` (`text-destructive`,
//! description `text-destructive/90`) variants; the nova style tightens the
//! padding to `px-2.5 py-2` and narrows the icon gap.
//!
//! The optional icon slot sits left of the title/description column; the
//! optional [`AlertAction`] paints at the alert's top-right corner (nova
//! `absolute top-2 right-2`). `AlertAction` is a nova-era slot: the nova
//! recipe reserves `pr-18` for it — 72px TOTAL right padding, not 72px
//! additional. S8 decision §3.2 (D4, a documented port extension):
//! the 72px reservation applies in ALL styles, so base titles wrap
//! within the reserved column instead of running under the opaque
//! action button.
//!
//! Title: `line-clamp-1` — a single line, truncated with a trailing "…" when
//! it exceeds the text column (N4), painted at `font-medium` (500) with the
//! embedded Inter Medium face. Description: paragraph text at 22.75px
//! (`leading-relaxed`); paragraphs are separated by 4px in base and 20px in
//! nova (16px `mb-4` + 4px grid gap).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Alert::new()
//!     .icon(Icon::CircleCheck)
//!     .show(ui, |ui| {
//!         AlertTitle::new("Heads up!").show(ui);
//!         AlertDescription::new("You can add components to your app.").show(ui);
//!     });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, fonts, Palette, ShadcnTheme, Style};
use egui::{Color32, CornerRadius, FontId, InnerResponse, Margin, Pos2, Rect, Stroke, Ui};

/// Visual variants, mirroring the source `alertVariants.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    /// `bg-card text-card-foreground`.
    #[default]
    Default,
    /// `text-destructive` with `text-destructive/90` description.
    Destructive,
}

/// Horizontal padding (`px-4` base recipe, nova `px-2.5`).
pub(crate) fn padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0,
        _ => 16.0,
    }
}

/// Vertical padding (`py-3` base recipe, nova `py-2`).
pub(crate) fn padding_y(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0,
        _ => 12.0,
    }
}

/// Gap between the icon column and the text column
/// (`gap-x-3` base recipe, nova `gap-x-2`).
pub(crate) fn icon_gap(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0,
        _ => 12.0,
    }
}

/// Right padding when an action is present —
/// `has-data-[slot=alert-action]:pr-18` is 72px TOTAL, not 72px additional:
/// the `:has()` selector's higher specificity replaces the right side of
/// `px-2.5` (spec §3.4). S8 decision §3.2 (D4, a documented port
/// extension — "uniform action placement in all styles"): the 72px
/// reservation applies in every style, not just nova — in base the title
/// was cut mid-word under the opaque action button with no "…".
pub(crate) fn right_padding(style: Style, has_action: bool) -> f32 {
    if has_action {
        72.0
    } else {
        padding_x(style)
    }
}

/// Resolved colors for one alert state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct AlertColors {
    pub fill: Color32,
    pub stroke: Stroke,
    pub title: Color32,
    pub description: Color32,
    pub icon: Color32,
}

pub(crate) fn resolve_alert_colors(p: &Palette, variant: AlertVariant) -> AlertColors {
    match variant {
        AlertVariant::Default => AlertColors {
            fill: p.card,
            stroke: Stroke::new(1.0, p.border),
            title: p.card_foreground,
            description: p.muted_foreground,
            icon: p.card_foreground,
        },
        AlertVariant::Destructive => AlertColors {
            fill: p.card,
            stroke: Stroke::new(1.0, p.border),
            title: p.destructive,
            description: color_alpha(p.destructive, 0.9),
            icon: p.destructive,
        },
    }
}

/// Geometry of the alert currently being laid out, for [`AlertAction`].
#[derive(Clone, Copy, Debug)]
struct AlertCtx {
    id: egui::Id,
    /// Top-right anchor for the action (`top-2 right-2`).
    top: f32,
    right: f32,
    left: f32,
}

fn alert_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_alert_stack")
}

fn push_alert(ui: &Ui, ctx: AlertCtx) {
    let mut stack: Vec<AlertCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(alert_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx()
        .data_mut(|d| d.insert_temp(alert_stack_id(), stack));
}

fn pop_alert(ui: &Ui) {
    let mut stack: Vec<AlertCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(alert_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(alert_stack_id(), stack));
}

fn current_alert(ui: &Ui) -> Option<AlertCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<AlertCtx>>(alert_stack_id()))
        .and_then(|s| s.last().copied())
}

/// A shadcn alert. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Alert {
    variant: AlertVariant,
    icon: Option<Icon>,
    id_salt: Option<String>,
}

impl Default for Alert {
    fn default() -> Self {
        Self::new()
    }
}

impl Alert {
    /// Creates a default alert.
    pub fn new() -> Self {
        Self {
            variant: AlertVariant::default(),
            icon: None,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`AlertVariant::Default`]).
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows an icon in the left column (`has-[>svg]` grid column).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets a stable id salt; required for reliable [`AlertAction`]
    /// width reservation when multiple alerts render side by side.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the alert around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_alert_colors(&p, self.variant);
        let radius = theme.radius.lg_u8();

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_alert"),
        };
        // `has-data-[slot=alert-action]:pr-18` — 72px TOTAL right padding
        // for the action; read from the previous frame (self-correcting),
        // so on first paint with an action the reservation applies from
        // frame 2 (D7, one-frame settle). S8 decision §3.2 (D4): the
        // reservation applies in all styles, not just nova.
        let has_action: bool = ui
            .ctx()
            .data_mut(|d| d.get_temp(id.with("has_action")).unwrap_or(false));
        let right = right_padding(theme.style, has_action);

        let cursor = ui.cursor();
        push_variant(ui, self.variant);
        push_alert(
            ui,
            AlertCtx {
                id,
                top: cursor.top() + 8.0,
                right: cursor.right() - 8.0,
                left: cursor.left(),
            },
        );

        let px = padding_x(theme.style);
        let py = padding_y(theme.style);
        let gap = icon_gap(theme.style);
        let frame = egui::Frame::new()
            .fill(c.fill)
            .stroke(c.stroke)
            .corner_radius(CornerRadius::same(radius))
            .inner_margin(Margin {
                left: px as i8,
                right: right as i8,
                top: py as i8,
                bottom: py as i8,
            });

        let inner = frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width()); // `w-full`
            ui.style_mut().visuals.override_text_color = Some(c.title);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                if let Some(icon) = self.icon {
                    // `[&>svg]:size-4 [&>svg]:translate-y-0.5` — 16px icon,
                    // top-aligned with the title row, nudged 2px down.
                    // `horizontal_top` (not `horizontal`): the default
                    // horizontal row centres the icon in the 36px
                    // interact-size row, putting its top 12px below the
                    // title top (ALR-02); top alignment keeps it at the
                    // row top, so the +2 nudge below lands it at +2px
                    // from the title's top edge.
                    let (rect, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        let icon_rect = Rect::from_center_size(
                            rect.center() + Pos2::new(0.0, 2.0).to_vec2(),
                            egui::Vec2::splat(16.0),
                        );
                        icon.paint(ui.painter(), icon_rect, c.icon, 2.0 * 16.0 / 24.0);
                    }
                }
                ui.vertical(|ui| {
                    // `gap-y-0.5` / nova `gap-0.5`.
                    ui.spacing_mut().item_spacing.y = 2.0;
                    content(ui)
                })
                .inner
            })
            .inner
        });

        pop_alert(ui);
        pop_variant(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// The title's font — `font-medium` (500) at 14px (`text-sm`), spec §3.3.
/// Wired to the embedded Inter Medium face.
pub(crate) fn title_font() -> FontId {
    fonts::medium(crate::theme::tokens::type_scale::SM)
}

/// The title's line box — the `text-sm` default line-height (20px);
/// no `leading-*` class applies (spec §3.3). The 14px font's natural
/// line height is 17px, so this must be set explicitly (ALR-01).
pub(crate) fn title_line_height() -> f32 {
    20.0
}

/// Returns the longest prefix of `text` such that `prefix + "…"` fits within
/// `max_width`, measured by `measure` (the width of a candidate string).
/// `line-clamp-1` (N4): the title renders on exactly one line, truncated
/// with a trailing "…" when it exceeds the available width. Pure and
/// unit-testable with a stub measurer; the live call measures with the
/// title font via `layout_no_wrap`. (D5: the visual requirement is
/// preserved exactly; only the mechanism is a port implementation.)
pub(crate) fn elide_prefix(text: &str, max_width: f32, measure: impl Fn(&str) -> f32) -> String {
    if measure(text) <= max_width {
        return text.to_owned();
    }
    let ellipsis = "…";
    if measure(ellipsis) > max_width {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lo = 0usize; // longest prefix known to fit with the ellipsis
    let mut hi = chars.len(); // shortest prefix known not to fit
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        let candidate: String = chars[..mid].iter().collect::<String>() + ellipsis;
        if measure(&candidate) <= max_width {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    chars[..lo].iter().collect::<String>() + ellipsis
}

/// `AlertTitle` — `min-h-4 font-medium tracking-tight`, single line.
#[derive(Clone, Debug)]
pub struct AlertTitle {
    text: String,
}

impl AlertTitle {
    /// Creates an alert title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title — one line, truncated with a trailing "…" when it
    /// exceeds the text column (N4), at `font-medium` (500).
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_alert_colors(&p, current_variant(ui).unwrap_or_default());
        let font = title_font();
        let max_width = ui.available_width().max(0.0);
        let display = elide_prefix(&self.text, max_width, |s| {
            ui.painter()
                .layout_no_wrap(s.to_owned(), font.clone(), c.title)
                .size()
                .x
        });
        // Lay the title out through a job so the 20px line-height applies
        // (ALR-01): `layout_no_wrap` uses the font's natural 17px leading.
        // The job does not wrap (`max_width` is infinite by default), so
        // the title stays one line, exactly as before.
        let format = egui::TextFormat {
            font_id: font,
            color: c.title,
            line_height: Some(title_line_height()),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&display, 0.0, format);
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(
            galley.size().max(egui::Vec2::new(0.0, 16.0)),
            egui::Sense::hover(),
        );
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, c.title);
        }
        response
    }
}

/// Description paragraph line-height — `[&_p]:leading-relaxed`
/// (1.625 × 14px = 22.75px, spec §3.5).
pub(crate) fn description_line_height() -> f32 {
    crate::theme::tokens::type_scale::SM * 1.625
}

/// Gap between description paragraphs — the base 4px grid gap; nova adds
/// `mb-4` (16px) on non-last `<p>` for 20px total (spec §2.6).
pub(crate) fn paragraph_gap(style: Style) -> f32 {
    match style {
        Style::Nova => 20.0,
        _ => 4.0,
    }
}

/// Splits description text into paragraphs on blank lines (one or more).
/// A single paragraph (the common case) renders exactly as before.
pub(crate) fn split_paragraphs(text: &str) -> Vec<&str> {
    // Paragraphs are runs of non-blank lines, separated by blank lines.
    let mut paras = Vec::new();
    let mut start: Option<usize> = None;
    let mut cursor = 0usize;
    for line in text.split_inclusive('\n') {
        let line_start = cursor;
        cursor += line.len();
        if line.trim().is_empty() {
            if let Some(s) = start.take() {
                paras.push(text[s..line_start].trim_end_matches('\n'));
            }
        } else if start.is_none() {
            start = Some(line_start);
        }
    }
    if let Some(s) = start {
        paras.push(text[s..].trim_end_matches('\n'));
    }
    paras
}

/// `AlertDescription` — `text-sm text-muted-foreground` (destructive:
/// `text-destructive/90`) with `leading-relaxed`, wraps to the width.
#[derive(Clone, Debug)]
pub struct AlertDescription {
    text: String,
}

impl AlertDescription {
    /// Creates an alert description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description — wrapping text at 22.75px paragraph
    /// line-height, paragraphs separated by the style's paragraph gap.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_alert_colors(&p, current_variant(ui).unwrap_or_default());
        let size = crate::theme::tokens::type_scale::SM;
        let font = FontId::proportional(size);
        // `[&_p]:leading-relaxed` (1.625) is set through the TextFormat.
        let format = egui::TextFormat {
            font_id: font,
            color: c.description,
            line_height: Some(description_line_height()),
            ..egui::TextFormat::default()
        };
        let gap = paragraph_gap(theme.style);
        let mut paras = split_paragraphs(&self.text);
        if paras.is_empty() {
            // Empty / whitespace-only text: render one (possibly empty) job,
            // exactly as before — no paragraph gap applies.
            paras = vec![self.text.as_str()];
        }
        let mut galleys = Vec::with_capacity(paras.len());
        let mut total_height = 0.0f32;
        let mut max_width = 0.0f32;
        for (i, para) in paras.iter().enumerate() {
            if i > 0 {
                total_height += gap;
            }
            let mut job = egui::text::LayoutJob::default();
            job.append(para, 0.0, format.clone());
            job.wrap.max_width = ui.available_width().max(0.0);
            let galley = ui.painter().layout_job(job);
            max_width = max_width.max(galley.size().x);
            total_height += galley.size().y;
            galleys.push(galley);
        }
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(max_width, total_height),
            egui::Sense::hover(),
        );
        if ui.is_rect_visible(rect) {
            let mut y = rect.min.y;
            for galley in &galleys {
                ui.painter()
                    .galley(Pos2::new(rect.min.x, y), galley.clone(), c.description);
                y += galley.size().y + gap;
            }
        }
        response
    }
}

/// The variant of the enclosing [`Alert`], from the ui temp stack.
fn variant_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_alert_variant_stack")
}

fn current_variant(ui: &Ui) -> Option<AlertVariant> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<AlertVariant>>(variant_stack_id()))
        .and_then(|s| s.last().copied())
}

fn push_variant(ui: &Ui, v: AlertVariant) {
    let mut stack: Vec<AlertVariant> = ui
        .ctx()
        .data_mut(|d| d.get_temp(variant_stack_id()))
        .unwrap_or_default();
    stack.push(v);
    ui.ctx()
        .data_mut(|d| d.insert_temp(variant_stack_id(), stack));
}

fn pop_variant(ui: &Ui) {
    let mut stack: Vec<AlertVariant> = ui
        .ctx()
        .data_mut(|d| d.get_temp(variant_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(variant_stack_id(), stack));
}

/// `AlertAction` — nova `absolute top-2 right-2` action slot. Shown inside
/// an [`Alert`], it paints at the alert's top-right corner without
/// consuming flow height (the alert reserves `pr-18` for it).
#[derive(Clone, Debug, Default)]
pub struct AlertAction {
    id_salt: Option<String>,
}

impl AlertAction {
    /// Creates an alert action slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the action contents at the alert's top-right.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        match current_alert(ui) {
            Some(ctx) => {
                // Mark the alert so the next frame reserves `pr-18`.
                ui.ctx()
                    .data_mut(|d| d.insert_temp(ctx.id.with("has_action"), true));
                let region = Rect::from_min_max(
                    Pos2::new(ctx.left, ctx.top),
                    Pos2::new(ctx.right, ctx.top + 100.0),
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(region)
                        .layout(egui::Layout::right_to_left(egui::Align::Min)),
                );
                let ret = content(&mut child);
                InnerResponse::new(ret, child.response())
            }
            None => {
                let inner = ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), content);
                InnerResponse::new(inner.inner, inner.response)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Vec2;

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn paddings_match_style_recipes() {
        // base `px-4 py-3`, nova `px-2.5 py-2`.
        assert_eq!(padding_x(Style::Vega), 16.0);
        assert_eq!(padding_y(Style::Vega), 12.0);
        assert_eq!(padding_x(Style::Nova), 10.0);
        assert_eq!(padding_y(Style::Nova), 8.0);
        assert_eq!(icon_gap(Style::Nova), 8.0);
        assert_eq!(icon_gap(Style::Vega), 12.0);
    }

    #[test]
    fn right_padding_is_72_total_with_action() {
        // `pr-18` REPLACES the right padding (72px total) — it does not add
        // to it. The audit's 82px reading was wrong (spec §3.4). S8
        // decision §3.2 (D4): the reservation applies in all styles.
        assert_eq!(right_padding(Style::Nova, true), 72.0);
        assert_eq!(right_padding(Style::Nova, false), 10.0);
        assert_eq!(right_padding(Style::Vega, true), 72.0);
        assert_eq!(right_padding(Style::Vega, false), 16.0);
    }

    #[test]
    fn title_uses_medium_face() {
        // `font-medium` (500) at 14px — wired, not regular (spec §3.3).
        assert_eq!(title_font().size, crate::theme::tokens::type_scale::SM);
        assert_eq!(title_font().family, fonts::medium_family());
    }

    #[test]
    fn title_line_height_is_text_sm_default() {
        // `text-sm` default line-height is 20px; no `leading-*` class
        // (spec §3.3, ALR-01). The font's natural leading is 17px.
        assert_eq!(title_line_height(), 20.0);
    }

    #[test]
    fn description_line_height_is_relaxed() {
        // `[&_p]:leading-relaxed` = 1.625 × 14px = 22.75px (spec §3.5).
        assert_eq!(description_line_height(), 22.75);
    }

    #[test]
    fn paragraph_gaps_match_recipes() {
        // Base 4px grid gap; nova 16px `mb-4` + 4px gap = 20px (spec §2.6).
        assert_eq!(paragraph_gap(Style::Nova), 20.0);
        assert_eq!(paragraph_gap(Style::Vega), 4.0);
    }

    #[test]
    fn paragraphs_split_on_blank_lines() {
        assert_eq!(split_paragraphs("one"), vec!["one"]);
        assert_eq!(split_paragraphs("one\n\ntwo"), vec!["one", "two"]);
        assert_eq!(split_paragraphs("one\n\n\ntwo"), vec!["one", "two"]);
        assert_eq!(
            split_paragraphs("one\ntwo\n\nthree"),
            vec!["one\ntwo", "three"]
        );
        assert!(split_paragraphs("").is_empty());
        assert!(split_paragraphs("\n\n").is_empty());
    }

    #[test]
    fn elide_prefix_truncates_with_ellipsis() {
        // Stub measurer: 10px per char, "…" is 10px.
        let measure = |s: &str| s.chars().count() as f32 * 10.0;
        assert_eq!(elide_prefix("short", 100.0, measure), "short");
        assert_eq!(elide_prefix("hello world", 100.0, measure), "hello wor…");
        assert_eq!(elide_prefix("hello world", 10.0, measure), "…");
        assert_eq!(elide_prefix("hello world", 5.0, measure), "");
        assert_eq!(elide_prefix("", 100.0, measure), "");
    }

    fn render_alert<R>(style: Style, render: impl Fn(&mut Ui) -> R) -> R {
        let ctx = egui::Context::default();
        Preset::default()
            .with_style(style)
            .build(Mode::Light)
            .install(&ctx);
        let mut out = None;
        let mut full = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("alert_s1_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            out = Some(Alert::new().show(ui, |ui| render(ui)).inner);
        });
        full.textures_delta.clear();
        out.expect("alert rendered")
    }

    #[test]
    fn long_title_truncates_to_one_line() {
        // N4: a title longer than the text column shows "…" on one line
        // and never overflows the alert (base padding 16px → 368px column).
        let rect = render_alert(Style::Vega, |ui| {
            AlertTitle::new("This is a very long alert title that must truncate with an ellipsis")
                .show(ui)
                .rect
        });
        assert!(rect.width() <= 368.0, "title width: {}", rect.width());
        assert!(rect.height() < 36.0, "title height: {}", rect.height());
    }

    #[test]
    fn title_row_is_20px_tall_in_both_styles() {
        // Spec §3.3 / ALR-01: the title line box is 20px, in both styles
        // (the title geometry is style-independent per spec §2.5).
        for style in [Style::Nova, Style::Vega] {
            let rect = render_alert(style, |ui| AlertTitle::new("Heads up!").show(ui).rect);
            assert!(
                (rect.height() - 20.0).abs() < 0.01,
                "title height ({style:?}): {}",
                rect.height()
            );
        }
    }

    /// Like [`render_alert`], but with an icon set, returning the
    /// closure's output together with the frame's output shapes.
    fn render_alert_with_icon<R>(
        style: Style,
        render: impl Fn(&mut Ui) -> R,
    ) -> (R, Vec<egui::epaint::ClippedShape>) {
        let ctx = egui::Context::default();
        Preset::default()
            .with_style(style)
            .build(Mode::Light)
            .install(&ctx);
        let mut out = None;
        let mut full = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("alert_icon_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            out = Some(
                Alert::new()
                    .icon(Icon::CircleCheck)
                    .show(ui, |ui| render(ui))
                    .inner,
            );
        });
        full.textures_delta.clear();
        (out.expect("alert rendered"), full.shapes)
    }

    #[test]
    fn icon_is_top_aligned_with_title_row() {
        // Spec §3.3 / ALR-02: the 16px icon is top-aligned with the
        // title, its top edge exactly 2px below the title's top edge
        // (`translate-y-0.5`), in both styles — measured against the
        // corrected 20px title row (ALR-01). The icon is painted as a
        // 16×16 textured mesh, so its rect is read back from the
        // output shapes; the title rect comes from the content closure.
        for style in [Style::Nova, Style::Vega] {
            let (title_rect, shapes) = render_alert_with_icon(style, |ui| {
                let t = AlertTitle::new("Heads up!").show(ui).rect;
                AlertDescription::new("Description line.").show(ui);
                t
            });
            assert!(
                (title_rect.height() - 20.0).abs() < 0.01,
                "title height ({style:?}): {}",
                title_rect.height()
            );
            let icon_rect = shapes
                .iter()
                .map(|s| s.shape.visual_bounding_rect())
                .find(|r| {
                    (r.width() - 16.0).abs() < 0.01
                        && (r.height() - 16.0).abs() < 0.01
                        && r.min.x < 33.0
                })
                .expect("icon shape found");
            assert!(
                (icon_rect.top() - title_rect.top() - 2.0).abs() < 0.01,
                "icon top ({style:?}): {}, title top: {}",
                icon_rect.top(),
                title_rect.top()
            );
        }
    }

    #[test]
    fn base_title_wraps_within_reserved_action_column() {
        // S8 decision §3.2 (D4): the 72px action reservation applies in
        // all styles, so a long base title elides within the reserved
        // column instead of running under the opaque action button.
        // The reservation is read from the previous frame (D7 one-frame
        // settle), so the second pass is the settled one.
        let ctx = egui::Context::default();
        Preset::default()
            .with_style(Style::Vega)
            .build(Mode::Light)
            .install(&ctx);
        let title = "This is a very long alert title that must truncate with an ellipsis";
        let mut widths: Vec<f32> = Vec::new();
        for _ in 0..2 {
            let title_rect = std::cell::Cell::new(Rect::NOTHING);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("alert_d4_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                Alert::new().show(ui, |ui| {
                    title_rect.set(AlertTitle::new(title).show(ui).rect);
                    AlertAction::new().show(ui, |_ui| {});
                });
            });
            out.textures_delta.clear();
            widths.push(title_rect.get().width());
        }
        // Frame 1 has no reservation yet (full width); frame 2 reserves
        // 72px: 400 − 16 (left) − 72 (right) = 312px.
        assert!(widths[0] > 312.0, "frame 1 width: {}", widths[0]);
        assert!(
            widths[1] <= 312.0,
            "frame 2 (settled) base title width with action: {}",
            widths[1]
        );
    }

    #[test]
    fn multi_paragraph_gap_is_20px_in_nova_and_4px_in_base() {
        // §2.6: two one-line paragraphs — the nova/base height difference
        // is exactly the 16px `mb-4` (differential pin, robust to the
        // font's exact row height).
        let nova = render_alert(Style::Nova, |ui| {
            AlertDescription::new("Para one.\n\nPara two.")
                .show(ui)
                .rect
                .height()
        });
        let base = render_alert(Style::Vega, |ui| {
            AlertDescription::new("Para one.\n\nPara two.")
                .show(ui)
                .rect
                .height()
        });
        assert!(nova > 45.5, "nova height: {nova}");
        assert!(base > 45.5, "base height: {base}");
        assert!(
            (nova - base - 16.0).abs() < 0.01,
            "nova: {nova}, base: {base}"
        );
    }

    #[test]
    fn destructive_uses_destructive_text() {
        let p = palette(Mode::Light);
        let d = resolve_alert_colors(&p, AlertVariant::Destructive);
        assert_eq!(d.fill, p.card);
        assert_eq!(d.title, p.destructive);
        assert_eq!(d.description, color_alpha(p.destructive, 0.9));
        assert_eq!(d.icon, p.destructive);
        assert_eq!(d.stroke.color, p.border);

        let def = resolve_alert_colors(&p, AlertVariant::Default);
        assert_eq!(def.title, p.card_foreground);
        assert_eq!(def.description, p.muted_foreground);
        assert_eq!(def.icon, p.card_foreground);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("alert_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                Alert::new()
                    .id_salt("render_alert")
                    .icon(Icon::CircleCheck)
                    .show(ui, |ui| {
                        AlertTitle::new("Heads up!").show(ui);
                        AlertDescription::new("Description line.").show(ui);
                        AlertAction::new().show(ui, |ui| {
                            ui.label("Retry");
                        });
                    });
            });
            out.textures_delta.clear();
        }
    }
}
