//! shadcn `avatar.tsx` — a circular user image with fallback.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/avatar.tsx`:
//! `size-8 rounded-full` (`size-6` sm, `size-10` lg), a `bg-muted
//! text-muted-foreground` fallback (initials or an icon), a `bg-primary`
//! status badge bottom-right, and an overlapping `AvatarGroup`
//! (`-space-x-2` with a `ring-2 ring-background` on each member).
//!
//! Verified final state (S10, `reports/avatar-s10-report.md`): PASS — the
//! S7 PASS holds in full. Settled diff 0 on all 535 probe rows in all 4
//! configs, rest captures pixel-identical to S7, round-trips
//! byte-identical. No S9 changes on this branch (avatar had no S9 fix
//! rows, S8 §2). Nova == base for every slot (spec §2.2 — every nova rule
//! restates its base value; the `after:rounded-full` in `.cn-avatar` is a
//! no-op, declaring no `content`).
//!
//! Non-interactive display element (spec §7): no Tab stop, no click
//! behaviour, no keys, and no hover/focus/disabled recipe (§4). S6/S10
//! verified the Tab walk is chrome-only (full sidebar interior 71/71 in
//! nova/light), clicks take no focus (20/20), hover changes nothing
//! (20/20), and the page does not scroll (R10, 8/8). Activation counting
//! is N/A — the demo has no interactive widgets (S8 §3.9, S10 §5).
//!
//! Images: pass an [`egui::ImageSource`] to [`Avatar::image`]. Texture
//! sources paint immediately (circularly clipped); URI/bytes sources load
//! through the context's texture loaders when available and degrade
//! gracefully to the fallback until/unless they resolve. The image is
//! centre-cropped to the square (proposed deviation D2, spec §10 — the
//! source stretches the image to fill, `object-fit: fill` default;
//! identical for square images). S8 §3.3 ACCEPTED the centre-crop; D2
//! stands as a proposed deviation for the Expert Reviewer. S5/S6
//! re-confirmed the crop from pixels in all 4 configs (centre-row texture
//! span t 0.24–0.76, zero white pixels, edges mid-gradient).
//!
//! Loading states (D3, spec §10): Radix loading/delay states are degraded
//! to texture-or-fallback end-states only — no `delayMs` timing, no
//! loading-status transitions. S6 fetched the public Radix source (`main`
//! + `@radix-ui/react-avatar@1.1.3`) and SETTLED D3 as a documented
//!   equivalence for the end-states (image shown only when Loaded, fallback
//!   otherwise); the transition timing stays OPEN-1. Nothing animates
//!   (spec §6 — no transition class anywhere in the source).
//!
//! Group counts size off the member avatars (`group-has-data-[size]`,
//! spec §3.6, audit N5): all-default / no members / outside a group →
//! 32px, all-sm → 24px, all-lg → 40px. Mixed member sizes follow the
//! hint-wins rule — the group's explicitly-set `.size()` hint when present,
//! else the base 32px (S5 check 6 values re-confirmed at S6: 24 hint-Sm /
//! 32 no-hint; the count's centre aligns with lg, 944.3 vs 944.4 / 1048.3
//! vs 1048.4). OPEN-2 is CLOSED by S8 §3.4: hint-wins is the only
//! deterministic rule for mixed groups.
//!
//! Mixed-row vertical alignment (S8 §3.5, ACCEPTED): smaller avatars sit
//! 2.0–2.5px above the row centre in mixed rows — egui horizontal-row
//! behaviour (children centred in the initial 36px row height; the row
//! grows when a taller item arrives, already-placed children keep their
//! positions). No frozen spec rule pins it; no code change.
//!
//! Tokens (spec §5): `--muted` / `--muted-foreground` / `--primary` /
//! `--background` from `globals.css`, with no upstream disagreement
//! (`themes.ts` agrees for avatar tokens). Fallback text is default
//! weight 400 — no weight slot exists in avatar.
//!
//! Budgets (S10 §2): flicker 7×0, idle CPU 0.0%, animating CPU 66.6–81.8%
//! (< 150%), RSS −0.23%..0.0% with no monotonic growth. Render floor NOT
//! VERIFIED (1.2–1.8 fps, pipeline-capped, load-independent — the same
//! verdict class as S7, not a defect). Disc fill modes constant across
//! all video legs: 245 light / 38 dark.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Avatar::new().fallback("CN").show(ui);
//! Avatar::new().fallback_icon(Icon::User).size(AvatarSize::Lg).show(ui);
//!
//! AvatarGroup::new().show(ui, |ui| {
//!     Avatar::new().fallback("AB").show(ui);
//!     Avatar::new().fallback("CD").show(ui);
//!     AvatarGroupCount::new("+2").show(ui);
//! });
//! ```

use crate::icons::Icon;
use crate::theme::ShadcnTheme;
use egui::{
    Color32, FontId, ImageSource, Pos2, Rect, Response, Sense, SizeHint, Stroke, TextureOptions,
    Ui, Vec2, Widget,
};

/// Size presets, mirroring the source `size` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// `size-6` (24px).
    Sm,
    /// `size-8` (32px).
    #[default]
    Default,
    /// `size-10` (40px).
    Lg,
}

impl AvatarSize {
    /// The circle side length (`size-8` etc.).
    pub fn side(self) -> f32 {
        match self {
            AvatarSize::Sm => 24.0,
            AvatarSize::Default => 32.0,
            AvatarSize::Lg => 40.0,
        }
    }

    /// Fallback text size (`text-sm`, sm: `text-xs`).
    pub fn text_size(self) -> f32 {
        match self {
            AvatarSize::Sm => crate::theme::tokens::type_scale::XS,
            _ => crate::theme::tokens::type_scale::SM,
        }
    }

    /// Badge side length (`size-2`, default `size-2.5`, lg `size-3`).
    pub fn badge_size(self) -> f32 {
        match self {
            AvatarSize::Sm => 8.0,
            AvatarSize::Default => 10.0,
            AvatarSize::Lg => 12.0,
        }
    }

    /// Badge icon size (`size-2`; hidden on sm).
    pub fn badge_icon_size(self) -> f32 {
        match self {
            AvatarSize::Sm => 0.0,
            _ => 8.0,
        }
    }

    /// Group-count icon size (`size-4`, sm: 3, lg: 5).
    pub fn group_icon_size(self) -> f32 {
        match self {
            AvatarSize::Sm => 12.0,
            AvatarSize::Default => 16.0,
            AvatarSize::Lg => 20.0,
        }
    }
}

/// Derives initials from a name: first character of the first two words,
/// or the first two characters of a single word. Uppercased.
///
/// Rust extension (spec §8): the source passes pre-computed text as
/// fallback children and computes no initials; the helper is a documented
/// convenience, not source behaviour.
pub fn initials_from_name(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let initials: String = match words.as_slice() {
        [] => String::new(),
        [w] => w.chars().take(2).collect(),
        [first, second, ..] => {
            let a = first.chars().next().unwrap_or_default();
            let b = second.chars().next().unwrap_or_default();
            [a, b].into_iter().collect()
        }
    };
    initials.to_uppercase()
}

/// The `bg-primary` status dot shown at the avatar's bottom-right corner.
#[derive(Clone, Debug, Default)]
pub struct AvatarBadge {
    /// Optional lucide icon drawn inside (`size-2`; hidden on sm avatars).
    pub icon: Option<Icon>,
}

impl AvatarBadge {
    /// Creates an empty (plain dot) badge.
    pub fn new() -> Self {
        Self { icon: None }
    }

    /// Shows an icon inside the badge.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// The avatar's `bg-primary` status badge, positioned bottom-right.
/// Attach with [`Avatar::badge`].
///
/// Non-interactive display element (spec §7): the avatar takes no Tab
/// stop, clicks do nothing, and there are no keys — the badge included.
#[derive(Clone, Debug, Default)]
pub struct Avatar<'a> {
    size: AvatarSize,
    image: Option<ImageSource<'a>>,
    fallback: Option<String>,
    fallback_icon: Option<Icon>,
    badge: Option<AvatarBadge>,
    id_salt: Option<String>,
}

impl<'a> Avatar<'a> {
    /// Creates an avatar with no image and no fallback (a muted circle).
    pub fn new() -> Self {
        Self {
            size: AvatarSize::default(),
            image: None,
            fallback: None,
            fallback_icon: None,
            badge: None,
            id_salt: None,
        }
    }

    /// Sets the size preset (default: [`AvatarSize::Default`]).
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the image (`AvatarImage`). Texture sources paint immediately;
    /// URI/bytes sources degrade to the fallback until they resolve.
    pub fn image(mut self, image: impl Into<ImageSource<'a>>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Sets the fallback initials/name (`AvatarFallback`). Long names are
    /// reduced to initials with [`initials_from_name`].
    pub fn fallback(mut self, text: impl Into<String>) -> Self {
        self.fallback = Some(text.into());
        self
    }

    /// Shows an icon instead of text when there is no image. The icon
    /// renders at half the avatar size (16px default, 12px sm, 20px lg) —
    /// a Rust extension: the source specifies no icon size for fallback
    /// icons (spec §8, §0 correction 3).
    pub fn fallback_icon(mut self, icon: Icon) -> Self {
        self.fallback_icon = Some(icon);
        self
    }

    /// Attaches a status badge (`AvatarBadge`).
    pub fn badge(mut self, badge: AvatarBadge) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Sets a stable id salt (Avatar keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the avatar.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = self.size.side();
        // §3.6 (N5): record this member's size in the enclosing group so
        // the group count can key off member `data-size`. No-op outside a
        // group. Recorded before the visibility early-return so an
        // off-screen member still counts.
        group::record_member(ui, self.size);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let radius = theme.radius.full_u8();

        // `bg-muted` fallback circle, always painted underneath so image
        // loading (or its absence) degrades gracefully.
        painter.rect_filled(rect, radius, p.muted);

        // Fallback content: initials or an icon.
        let text = self.fallback.as_deref().map(initials_from_name);
        let text_color = p.muted_foreground;
        if let Some(initials) = text.filter(|s| !s.is_empty()) {
            let font = FontId::proportional(self.size.text_size());
            // §3.3 (derived): over-long fallback text is centre-clipped to
            // the circle, with no ellipsis — paint it through a clip rect.
            // (Unreachable through `.fallback()`, which reduces names to at
            // most 2 initials; kept as the spec rule, S5 judges it rendered.)
            painter.with_clip_rect(rect).text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                initials,
                font,
                text_color,
            );
        } else if let Some(icon) = self.fallback_icon {
            let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(size * 0.5));
            icon.paint(painter, icon_rect, text_color, 2.0 * size * 0.5 / 24.0);
        }

        // The image (`AvatarImage`) covers the fallback when available.
        if let Some(source) = self.image.as_ref() {
            if let Some(tex) = resolve_texture(ui, source, size) {
                let uv = cover_uv(tex.size, rect.size());
                painter.add(
                    egui::epaint::RectShape::filled(rect, radius, Color32::WHITE)
                        .with_texture(tex.id, uv),
                );
            }
        }

        // Group members get a `ring-2 ring-background` separation ring.
        if group::is_member(ui) {
            painter.circle_stroke(
                rect.center(),
                size * 0.5 + 1.0,
                Stroke::new(2.0, p.background),
            );
        }

        // `AvatarBadge`: `bg-primary` circle at the bottom-right corner,
        // with a 2px `ring-2 ring-background` ring; the icon is hidden on
        // sm avatars.
        if let Some(badge) = self.badge.as_ref() {
            let bs = self.size.badge_size();
            let center = Pos2::new(rect.right() - bs * 0.5, rect.bottom() - bs * 0.5);
            painter.circle_filled(center, bs * 0.5 + 2.0, p.background);
            painter.circle_filled(center, bs * 0.5, p.primary);
            if let Some(icon) = badge.icon {
                let icon_size = self.size.badge_icon_size();
                if icon_size > 0.0 {
                    let icon_rect = Rect::from_center_size(center, Vec2::splat(icon_size));
                    icon.paint(
                        painter,
                        icon_rect,
                        p.primary_foreground,
                        2.0 * icon_size / 24.0,
                    );
                }
            }
        }

        response
    }
}

impl Widget for Avatar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Resolves an [`ImageSource`] to a ready [`egui::SizedTexture`].
///
/// `ImageSource::Texture` resolves immediately; URI/bytes sources go through
/// the context texture loaders (available under eframe) and return `None`
/// until loaded or when no loader matches.
pub(crate) fn resolve_texture(
    ui: &Ui,
    source: &ImageSource<'_>,
    size: f32,
) -> Option<egui::load::SizedTexture> {
    match source.clone().load(
        ui.ctx(),
        TextureOptions::default(),
        SizeHint::Size {
            width: size.round().max(1.0) as u32,
            height: size.round().max(1.0) as u32,
            maintain_aspect_ratio: true,
        },
    ) {
        Ok(egui::load::TexturePoll::Ready { texture }) => Some(texture),
        _ => None,
    }
}

/// Computes the normalized UV window that center-crops a texture to a
/// square.
///
/// Proposed deviation D2 (spec §10): the source declares `aspect-square
/// size-full` with no `object-fit` rule, so the CSS default is
/// stretch-to-fill; this port centre-crops instead. Identical for square
/// images; differs for non-square images. S8 §3.3 ACCEPTED the
/// centre-crop; D2 stands as a proposed deviation for the Expert
/// Reviewer.
pub(crate) fn cover_uv(texture_size: Vec2, target: Vec2) -> Rect {
    if texture_size.x <= 0.0 || texture_size.y <= 0.0 || target.x <= 0.0 || target.y <= 0.0 {
        return Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    }
    let target_aspect = target.x / target.y;
    let tex_aspect = texture_size.x / texture_size.y;
    let (uw, uh) = if tex_aspect > target_aspect {
        // Texture is wider: crop horizontally.
        (target_aspect / tex_aspect, 1.0)
    } else {
        (1.0, tex_aspect / target_aspect)
    };
    let u0 = (1.0 - uw) * 0.5;
    let v0 = (1.0 - uh) * 0.5;
    Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u0 + uw, v0 + uh))
}

/// Group plumbing shared by [`AvatarGroup`], [`Avatar`] and
/// [`AvatarGroupCount`].
///
/// Members record their sizes here (spec §3.6, audit N5) so the group count
/// can key off member `data-size` — the source's
/// `group-has-data-[size]/avatar-group` rules. The group has no size prop
/// in the source; `AvatarGroup::size()` is a Rust extension kept as the
/// hint-wins tiebreak for mixed member sizes (OPEN-2 CLOSED by S8 §3.4).
pub(crate) mod group {
    use egui::{Id, Ui};

    #[derive(Clone, Debug, Default)]
    pub struct GroupState {
        /// Explicitly-set group size hint (`AvatarGroup::size()`), if any.
        /// Used only as the hint-wins tiebreak for mixed member sizes
        /// (OPEN-2 CLOSED by S8 §3.4).
        pub size: Option<super::AvatarSize>,
        /// Sizes of the member avatars shown so far in this group, in show
        /// order. Fresh on every `AvatarGroup::show` (one frame).
        pub members: Vec<super::AvatarSize>,
    }

    fn stack_id() -> Id {
        Id::unique("shadcn_avatar_group_stack")
    }

    pub(crate) fn push(ui: &Ui, state: GroupState) {
        let mut stack: Vec<GroupState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(stack_id()))
            .unwrap_or_default();
        stack.push(state);
        ui.ctx().data_mut(|d| d.insert_temp(stack_id(), stack));
    }

    pub(crate) fn pop(ui: &Ui) {
        let mut stack: Vec<GroupState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(stack_id()))
            .unwrap_or_default();
        stack.pop();
        ui.ctx().data_mut(|d| d.insert_temp(stack_id(), stack));
    }

    pub(crate) fn current(ui: &Ui) -> Option<GroupState> {
        ui.ctx()
            .data_mut(|d| d.get_temp::<Vec<GroupState>>(stack_id()))
            .and_then(|s| s.last().cloned())
    }

    /// True when the widget is inside an [`super::AvatarGroup`].
    pub fn is_member(ui: &Ui) -> bool {
        current(ui).is_some()
    }

    /// Records a member avatar's size in the innermost enclosing group
    /// (no-op outside a group). Called from [`super::Avatar::show`] before
    /// any early return, so off-screen members still count.
    pub(crate) fn record_member(ui: &Ui, size: super::AvatarSize) {
        let mut stack: Vec<GroupState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(stack_id()))
            .unwrap_or_default();
        if let Some(top) = stack.last_mut() {
            top.members.push(size);
        }
        ui.ctx().data_mut(|d| d.insert_temp(stack_id(), stack));
    }

    /// Resolves the group-count size per spec §3.6 (audit N5).
    ///
    /// Order: an explicit size on the count itself wins; otherwise the
    /// members' size when they all agree (all-default, all-sm, or all-lg);
    /// mixed members follow the hint-wins rule (OPEN-2 CLOSED by S8 §3.4)
    /// — the group's explicitly-set `.size()` hint is used as the
    /// caller-declared tiebreak when present, else the base `size-8`
    /// (32px); no members recorded (or outside a group) → base.
    pub(crate) fn resolve_count_size(
        explicit: Option<super::AvatarSize>,
        members: &[super::AvatarSize],
        group_hint: Option<super::AvatarSize>,
    ) -> super::AvatarSize {
        if let Some(size) = explicit {
            return size;
        }
        if members.is_empty() {
            return super::AvatarSize::default();
        }
        if members.iter().all(|&m| m == members[0]) {
            return members[0];
        }
        // Mixed sizes — hint-wins (OPEN-2 CLOSED by S8 §3.4): never guess
        // a winner from the two `group-has-data-[size]` rules; use the
        // caller-declared group hint when present, else the count's base
        // `size-8`.
        group_hint.unwrap_or_default()
    }

    /// Reads the enclosing group's member sizes and resolves the count size
    /// (see [`resolve_count_size`]).
    pub(crate) fn resolve_count_size_for(
        ui: &Ui,
        explicit: Option<super::AvatarSize>,
    ) -> super::AvatarSize {
        let Some(state) = current(ui) else {
            return resolve_count_size(explicit, &[], None);
        };
        let hint = state.size;
        resolve_count_size(explicit, &state.members, hint)
    }
}

/// `AvatarGroup` — overlapping avatars (`-space-x-2`); members get a
/// `ring-2 ring-background` separation ring.
#[derive(Clone, Debug, Default)]
pub struct AvatarGroup {
    /// Explicitly-set group size hint, if any. Rust extension — the source
    /// has no group size prop (spec §3.5); used only as the hint-wins
    /// tiebreak for the group count when member sizes are mixed
    /// (OPEN-2 CLOSED by S8 §3.4).
    size: Option<AvatarSize>,
    id_salt: Option<String>,
}

impl AvatarGroup {
    /// Creates an avatar group.
    pub fn new() -> Self {
        Self {
            size: None,
            id_salt: None,
        }
    }

    /// Sets the group-size hint (Rust extension — the source has no group
    /// size prop, spec §3.5). Members do NOT inherit this size; each member
    /// carries its own. The hint is used only as the hint-wins tiebreak for
    /// the group count when member sizes are mixed (OPEN-2 CLOSED by
    /// S8 §3.4).
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        group::push(
            ui,
            group::GroupState {
                size: self.size,
                members: Vec::new(),
            },
        );
        // `-space-x-2`: members overlap by 8px.
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = -8.0;
            content(ui)
        });
        group::pop(ui);
        egui::InnerResponse::new(inner.inner, inner.response)
    }
}

/// `AvatarGroupCount` — the `+n` counter circle at the end of an
/// [`AvatarGroup`] (`bg-muted text-muted-foreground ring-2 ring-background`).
#[derive(Clone, Debug)]
pub struct AvatarGroupCount {
    text: String,
    icon: Option<Icon>,
    size: Option<AvatarSize>,
    id_salt: Option<String>,
}

impl AvatarGroupCount {
    /// Creates a count chip with the given text (e.g. `+2`).
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            size: None,
            id_salt: None,
        }
    }

    /// Shows an icon instead of text.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Overrides the resolved size (spec §3.6). When omitted, the count
    /// follows the member avatars' sizes (all-default → 32px, all-sm →
    /// 24px, all-lg → 40px; mixed sizes follow the hint-wins rule,
    /// OPEN-2 CLOSED by S8 §3.4).
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the count chip.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // §3.6 (N5): the count keys off member sizes, not the group builder.
        let size = group::resolve_count_size_for(ui, self.size);
        let side = size.side();
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let radius = theme.radius.full_u8();
        painter.rect_filled(rect, radius, p.muted);
        painter.circle_stroke(
            rect.center(),
            side * 0.5 + 1.0,
            Stroke::new(2.0, p.background),
        );
        if let Some(icon) = self.icon {
            let isz = size.group_icon_size();
            let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(isz));
            icon.paint(painter, icon_rect, p.muted_foreground, 2.0 * isz / 24.0);
        } else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.text,
                FontId::proportional(crate::theme::tokens::type_scale::SM),
                p.muted_foreground,
            );
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    #[test]
    fn sizes_match_source() {
        // size-6 / size-8 / size-10.
        assert_eq!(AvatarSize::Sm.side(), 24.0);
        assert_eq!(AvatarSize::Default.side(), 32.0);
        assert_eq!(AvatarSize::Lg.side(), 40.0);
        // badge size-2 / size-2.5 / size-3.
        assert_eq!(AvatarSize::Sm.badge_size(), 8.0);
        assert_eq!(AvatarSize::Default.badge_size(), 10.0);
        assert_eq!(AvatarSize::Lg.badge_size(), 12.0);
        // badge icon hidden on sm.
        assert_eq!(AvatarSize::Sm.badge_icon_size(), 0.0);
        assert_eq!(AvatarSize::Default.badge_icon_size(), 8.0);
        // fallback text size: text-sm (14px), sm: text-xs (12px).
        assert_eq!(AvatarSize::Sm.text_size(), 12.0);
        assert_eq!(AvatarSize::Default.text_size(), 14.0);
        assert_eq!(AvatarSize::Lg.text_size(), 14.0);
        // group icon size-3 / size-4 / size-5.
        assert_eq!(AvatarSize::Sm.group_icon_size(), 12.0);
        assert_eq!(AvatarSize::Default.group_icon_size(), 16.0);
        assert_eq!(AvatarSize::Lg.group_icon_size(), 20.0);
    }

    #[test]
    fn group_count_size_resolution() {
        // Pure §3.6 resolver (audit N5): explicit wins, unanimous members
        // win, mixed falls back to the group hint or the base size-8.
        use AvatarSize::{Default as D, Lg, Sm};
        let r = group::resolve_count_size;
        assert_eq!(r(None, &[], None), D);
        assert_eq!(r(None, &[D, D], None), D);
        assert_eq!(r(None, &[Sm, Sm, Sm], None), Sm);
        assert_eq!(r(None, &[Lg], None), Lg);
        // Mixed sizes follow the hint-wins rule (OPEN-2 CLOSED by S8 §3.4):
        // group hint when set, else base.
        assert_eq!(r(None, &[Sm, Lg], Some(Sm)), Sm);
        assert_eq!(r(None, &[Sm, Lg], Some(Lg)), Lg);
        assert_eq!(r(None, &[Sm, Lg], None), D);
        assert_eq!(r(None, &[D, Sm, Lg], None), D);
        // Explicit count size wins over everything.
        assert_eq!(r(Some(Lg), &[Sm, Sm], Some(Sm)), Lg);
        assert_eq!(r(Some(Sm), &[Lg, Lg], None), Sm);
    }

    #[test]
    fn group_count_follows_member_sizes() {
        // End-to-end §3.6 (N5): the rendered count chip's rect follows the
        // member avatars' sizes, not the group's `.size()` builder.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut all_default = 0.0;
        let mut all_sm = 0.0;
        let mut all_lg = 0.0;
        let mut mixed_hint = 0.0;
        let mut mixed_no_hint = 0.0;
        let mut no_members = 0.0;
        let mut outside_group = 0.0;
        let mut explicit = 0.0;
        let mut overlap = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("avatar_count_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 800.0))),
            );
            // All-default members → 32px count.
            AvatarGroup::new().show(ui, |ui| {
                Avatar::new().fallback("AB").show(ui);
                Avatar::new().fallback("CD").show(ui);
                all_default = AvatarGroupCount::new("+2").show(ui).rect.width();
            });
            // All-sm members → 24px count.
            AvatarGroup::new().show(ui, |ui| {
                Avatar::new().size(AvatarSize::Sm).fallback("AB").show(ui);
                Avatar::new().size(AvatarSize::Sm).fallback("CD").show(ui);
                all_sm = AvatarGroupCount::new("+2").show(ui).rect.width();
            });
            // All-lg members → 40px count.
            AvatarGroup::new().show(ui, |ui| {
                Avatar::new().size(AvatarSize::Lg).fallback("AB").show(ui);
                Avatar::new().size(AvatarSize::Lg).fallback("CD").show(ui);
                all_lg = AvatarGroupCount::new("+2").show(ui).rect.width();
            });
            // Mixed members + explicit group hint → hint size (hint-wins,
            // OPEN-2 CLOSED by S8 §3.4).
            AvatarGroup::new().size(AvatarSize::Sm).show(ui, |ui| {
                Avatar::new().size(AvatarSize::Sm).fallback("AB").show(ui);
                Avatar::new().size(AvatarSize::Lg).fallback("CD").show(ui);
                mixed_hint = AvatarGroupCount::new("+2").show(ui).rect.width();
            });
            // Mixed members, no hint → base 32px (hint-wins, OPEN-2 CLOSED
            // by S8 §3.4).
            AvatarGroup::new().show(ui, |ui| {
                Avatar::new().size(AvatarSize::Sm).fallback("AB").show(ui);
                Avatar::new().size(AvatarSize::Lg).fallback("CD").show(ui);
                mixed_no_hint = AvatarGroupCount::new("+2").show(ui).rect.width();
            });
            // No members → base 32px, even with a group hint set.
            AvatarGroup::new().size(AvatarSize::Sm).show(ui, |ui| {
                no_members = AvatarGroupCount::new("+2").show(ui).rect.width();
            });
            // Count outside a group → base 32px.
            outside_group = AvatarGroupCount::new("+2").show(ui).rect.width();
            // Explicit count size wins over member sizes.
            AvatarGroup::new().show(ui, |ui| {
                Avatar::new().size(AvatarSize::Sm).fallback("AB").show(ui);
                explicit = AvatarGroupCount::new("+2")
                    .size(AvatarSize::Lg)
                    .show(ui)
                    .rect
                    .width();
            });
            // Overlap: two default members share 8px (`-space-x-2`).
            let mut first_right = 0.0;
            let mut second_left = 0.0;
            AvatarGroup::new().show(ui, |ui| {
                first_right = Avatar::new().fallback("AB").show(ui).rect.right();
                second_left = Avatar::new().fallback("CD").show(ui).rect.left();
            });
            overlap = first_right - second_left;
        });
        out.textures_delta.clear();
        assert_eq!(all_default, 32.0);
        assert_eq!(all_sm, 24.0);
        assert_eq!(all_lg, 40.0);
        assert_eq!(mixed_hint, 24.0);
        assert_eq!(mixed_no_hint, 32.0);
        assert_eq!(no_members, 32.0);
        assert_eq!(outside_group, 32.0);
        assert_eq!(explicit, 40.0);
        assert_eq!(overlap, 8.0);
    }

    #[test]
    fn initials_derivation() {
        assert_eq!(initials_from_name("Colm Tuite"), "CT");
        assert_eq!(initials_from_name("sage"), "SA");
        assert_eq!(initials_from_name("ada lovelace king"), "AL");
        assert_eq!(initials_from_name(""), "");
        assert_eq!(initials_from_name("   "), "");
    }

    #[test]
    fn cover_uv_center_crops() {
        // Square texture over square target: full uv.
        let uv = cover_uv(Vec2::splat(100.0), Vec2::splat(32.0));
        assert_eq!(uv.min, Pos2::ZERO);
        assert_eq!(uv.max, Pos2::new(1.0, 1.0));
        // Wider texture: horizontal crop keeps full height.
        let uv = cover_uv(Vec2::new(200.0, 100.0), Vec2::splat(32.0));
        assert_eq!(uv.min.y, 0.0);
        assert_eq!(uv.max.y, 1.0);
        assert!((uv.min.x - 0.25).abs() < 1e-4);
        assert!((uv.max.x - 0.75).abs() < 1e-4);
        // Taller texture: vertical crop.
        let uv = cover_uv(Vec2::new(100.0, 300.0), Vec2::splat(32.0));
        assert_eq!(uv.min.x, 0.0);
        assert_eq!(uv.max.x, 1.0);
        assert!((uv.min.y - 1.0 / 3.0).abs() < 1e-4);
        assert!((uv.max.y - 2.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn unknown_texture_size_uses_full_uv() {
        let uv = cover_uv(Vec2::ZERO, Vec2::splat(32.0));
        assert_eq!(uv.min, Pos2::ZERO);
        assert_eq!(uv.max, Pos2::new(1.0, 1.0));
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("avatar_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                Avatar::new()
                    .fallback("CN")
                    .badge(AvatarBadge::new())
                    .show(ui);
                Avatar::new()
                    .size(AvatarSize::Lg)
                    .fallback_icon(Icon::User)
                    .show(ui);
                AvatarGroup::new().show(ui, |ui| {
                    Avatar::new().fallback("AB").show(ui);
                    Avatar::new().fallback("CD").show(ui);
                    AvatarGroupCount::new("+2").show(ui);
                });
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn group_membership_applies_to_members() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut member = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("avatar_group_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            assert!(!group::is_member(ui));
            AvatarGroup::new().show(ui, |ui| {
                member = group::is_member(ui);
            });
            assert!(!group::is_member(ui));
        });
        out.textures_delta.clear();
        assert!(member);
    }
}
