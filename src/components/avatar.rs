//! shadcn `avatar.tsx` — a circular user image with fallback.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/avatar.tsx`:
//! `size-8 rounded-full` (`size-6` sm, `size-10` lg), a `bg-muted
//! text-muted-foreground` fallback (initials or an icon), a `bg-primary`
//! status badge bottom-right, and an overlapping `AvatarGroup`
//! (`-space-x-2` with a `ring-2 ring-background` on each member).
//!
//! Images: pass an [`egui::ImageSource`] to [`Avatar::image`]. Texture
//! sources paint immediately (circularly clipped, aspect-cover); URI/bytes
//! sources load through the context's texture loaders when available and
//! degrade gracefully to the fallback until/unless they resolve.
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

    /// Shows an icon instead of text when there is no image.
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
            painter.text(
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
/// square (CSS `object-cover`).
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
/// [`AvatarGroupCount`] (members adapt their ring/text/icon sizing).
pub(crate) mod group {
    use egui::{Id, Ui};

    #[derive(Clone, Copy, Debug)]
    pub struct GroupState {
        pub size: super::AvatarSize,
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
            .and_then(|s| s.last().copied())
    }

    /// True when the widget is inside an [`super::AvatarGroup`].
    pub fn is_member(ui: &Ui) -> bool {
        current(ui).is_some()
    }
}

/// `AvatarGroup` — overlapping avatars (`-space-x-2`); members get a
/// `ring-2 ring-background` separation ring.
#[derive(Clone, Debug, Default)]
pub struct AvatarGroup {
    size: AvatarSize,
    id_salt: Option<String>,
}

impl AvatarGroup {
    /// Creates an avatar group.
    pub fn new() -> Self {
        Self {
            size: AvatarSize::default(),
            id_salt: None,
        }
    }

    /// Sets the size for members that don't override it (`group-data-[size]`).
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
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
        let group_size = self.size;
        group::push(ui, group::GroupState { size: group_size });
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

    /// Overrides the group size (matches the group when omitted).
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
        let size = self
            .size
            .or_else(|| group::current(ui).map(|g| g.size))
            .unwrap_or_default();
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
        // group icon size-3 / size-4 / size-5.
        assert_eq!(AvatarSize::Sm.group_icon_size(), 12.0);
        assert_eq!(AvatarSize::Default.group_icon_size(), 16.0);
        assert_eq!(AvatarSize::Lg.group_icon_size(), 20.0);
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
