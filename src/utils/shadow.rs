//! Layered shadows, cached as textures, with style layering.
//!
//! This module is the crate's one shadow system. It replaces the old
//! single-blurred-rect approach (one `RectShape::filled(...).with_blur_width(2.0)`
//! rebuilt every frame) with true multi-layer shadows rasterized once into an
//! egui texture atlas and reused across frames.
//!
//! # Value table
//!
//! [`ShadowSize::layers`] returns the Tailwind v4 default `--shadow-*` values
//! (tailwindcss.com/docs/box-shadow). The snapshot defines no `--shadow-*`
//! tokens anywhere (`globals.css` `@theme inline` and all 8 style sheets were
//! grepped 2026-11-12), so the Tailwind defaults are what the original's
//! `shadow-*` classes resolve to. Each layer is one CSS `box-shadow` layer:
//! x/y offset, blur radius, spread radius, and black at a given alpha.
//!
//! Dark mode: the original has no dark shadow variant, but a 5%-black shadow
//! is invisible on a near-black panel, so layer alphas are multiplied by 1.6
//! in dark mode. This is a deliberate crate convention inherited from the
//! legacy [`shadow_xs`]-family helpers below (same 1.6 factor), not an
//! original value; the stored table holds the true light values and the boost
//! is applied at raster time. It keeps a shadowed widget visually consistent
//! with its siblings, which all use the boosted legacy helpers today.
//!
//! # Style layering (the §5(b) LAYER ruling, settled 2026-11-12)
//!
//! A style's `.cn-*` rules LAYER over the base recipe: a property the style
//! rule does not mention keeps its base value. [`resolve_shadow`] encodes
//! exactly that: it returns the base shadow UNLESS the active style's `.cn-*`
//! rule for that exact slot explicitly declares a `shadow-*` / `shadow-none`
//! class. Silence keeps the base value; an explicit declaration overrides it.
//! There is no blanket "if nova, no shadow" anywhere in this module — such a
//! gate is a defect by construction (it was the `button.rs` bug this system
//! fixes: the outline button paints `shadow-xs` under ALL styles).
//!
//! The override table in [`style_override`] was extracted mechanically from
//! the frozen snapshot (`registry/styles/style-*.css`, 2026-11-12): every
//! `shadow-*` class in every `.cn-*` rule, one arm per (style, slot) pair,
//! each citing its source file and line. `-aria` duplicate rules were folded
//! into their main rule (verified identical in every style); descendant
//! selectors (`*:`) and conditional declarations (`group-data-*`, `data-*`,
//! `before:`, `md:`) are recorded on the slot they actually target, with the
//! condition in the citation comment and in the [`ShadowSlot`] docs. Callers
//! must only consult a conditional slot when its condition holds — see the
//! per-variant docs. The `.cn-card` declarations were deliberately excluded:
//! card is a Batch C component (NOT AUDITED); its style declarations remain a
//! Batch C wave concern, and card resolution always returns the base value
//! until that wave adds the slot.
//!
//! # Caching
//!
//! [`paint_shadow`] rasterizes the shadow for a (layers, corner radii, dark,
//! pixels-per-point) key into ONE texture atlas — four corner patches plus
//! four oriented edge strips — and draws it as nine quads (corners 1:1, edges
//! stretched). The shadow contributes nothing strictly inside the widget
//! rect: the rasterizers mask the interior after blurring, so the exterior
//! profile is exactly the unmasked shadow while transparent widgets show the
//! page background through them (SEL-VIS-01); the center region of the
//! nine-slice is interior-only and is never painted. The atlas is
//! size-independent: resizing a widget reuses the same texture. Entries live in `Context` temp data (never
//! serialized, dropped with the context), bounded to [`MAX_CACHE_ENTRIES`]
//! with LRU eviction; dropping an entry drops its [`TextureHandle`], which
//! frees the GPU texture. `shadow-none` paints nothing and touches no cache.
//!
//! One exception: when a widget is small relative to the blur, the nine-slice
//! seam would fall where the corner's two-dimensional falloff has not yet
//! converged to the straight-edge profile, and the edge strip would show a
//! visible step against the corner patch. In that case
//! ([`needs_full_rect`]: the seam sits less than 3σ inside the widget edge)
//! the whole expanded rect is rasterized as ONE texture instead — the true
//! two-dimensional blur, with no seam by construction. Full-rect entries are
//! keyed by widget size as well (they cannot be shared across sizes) and
//! share the same LRU cache and bound; the case is rare in practice (short
//! widgets with `shadow-2xl`), and the texture is capped at 512px per side.
//!
//! # Backward compatibility
//!
//! The legacy [`shadow_xs`]/[`shadow_sm`]/[`shadow_md`]/[`shadow_lg`]/
//! `shadow_xl` helpers are unchanged (same single-layer `egui::Shadow` values,
//! same dark boost) so all 30+ existing call sites keep rendering exactly as
//! before. Waves migrate call sites to [`paint_shadow`] slot by slot; the
//! only component changed by this system itself is the `button.rs` outline
//! gate fix. The legacy values are noticeably stronger than the Tailwind
//! table (e.g. legacy `shadow_xs` light alpha is 50/255 vs the true 0.05),
//! so migrated widgets get subtler, original-faithful shadows — that change
//! is expected and must be classified as such in blast-radius captures.

use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};

use egui::epaint::ColorImage;
use egui::{Color32, Context, CornerRadius, Painter, Rect, TextureHandle, TextureOptions, Vec2};

use crate::theme::Style;

// ---------------------------------------------------------------------------
// Legacy single-layer helpers (unchanged — see module docs).
// ---------------------------------------------------------------------------

/// `shadow-xs` — the subtle border shadow used by buttons and inputs.
pub fn shadow_xs(dark: bool) -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 1],
        blur: 2,
        spread: 0,
        color: color(dark, 50),
    }
}

/// `shadow-sm`
pub fn shadow_sm(dark: bool) -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 1],
        blur: 4,
        spread: 0,
        color: color(dark, 70),
    }
}

/// `shadow-md`
pub fn shadow_md(dark: bool) -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: color(dark, 80),
    }
}

/// `shadow-lg` — popovers, dropdowns, dialogs.
pub fn shadow_lg(dark: bool) -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: color(dark, 110),
    }
}

/// `shadow-xl` — the chart tooltip and other large floating panels (Tailwind
/// xl = y-offset 20, blur 25): a deeper `shadow-lg`.
pub fn shadow_xl(dark: bool) -> egui::epaint::Shadow {
    let mut s = shadow_lg(dark);
    s.offset[1] += 4;
    s.blur += 8;
    s.color = Color32::from_black_alpha((s.color.a() as f32 * 1.15) as u8);
    s
}

fn color(dark: bool, alpha: u8) -> Color32 {
    if dark {
        Color32::from_black_alpha((alpha as f32 * 1.6) as u8)
    } else {
        Color32::from_black_alpha(alpha)
    }
}

// ---------------------------------------------------------------------------
// Value table: one CSS `box-shadow` layer per entry.
// ---------------------------------------------------------------------------

/// One CSS `box-shadow` layer: offset, blur radius, spread radius, and black
/// at [`ShadowLayer::alpha`] (0.0–1.0). All Tailwind shadow layers are black;
/// color is not a parameter. `blur` is the CSS blur radius (the rasterizer
/// uses σ = blur / 2, the standard CSS Gaussian approximation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    /// x/y offset in points (`offset[0]` = x, `offset[1]` = y).
    pub offset: [f32; 2],
    /// CSS blur radius in points.
    pub blur: f32,
    /// Spread radius in points (negative shrinks the shadow shape).
    pub spread: f32,
    /// Black alpha, 0.0–1.0 (light value; see module docs for dark mode).
    pub alpha: f32,
}

impl ShadowLayer {
    const fn new(ox: f32, oy: f32, blur: f32, spread: f32, alpha: f32) -> Self {
        Self {
            offset: [ox, oy],
            blur,
            spread,
            alpha,
        }
    }

    /// Alpha actually rasterized: the stored light alpha, × [`DARK_BOOST`] in
    /// dark mode, clamped to 1.0.
    fn resolved_alpha(self, dark: bool) -> f32 {
        let a = if dark {
            self.alpha * DARK_BOOST
        } else {
            self.alpha
        };
        a.clamp(0.0, 1.0)
    }
}

/// Dark-mode alpha multiplier, inherited from the legacy helpers' `color()`.
const DARK_BOOST: f32 = 1.6;

/// A Tailwind shadow size: the `shadow-*` utility scale the original uses.
/// Values are the Tailwind v4 defaults (see module docs); the snapshot
/// overrides none of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShadowSize {
    /// `shadow-xs`: `0 1px 2px 0 rgb(0 0 0 / 0.05)` — buttons, inputs.
    Xs,
    /// `shadow-sm`: `0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)`.
    Sm,
    /// `shadow-md`: `0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)`.
    Md,
    /// `shadow-lg`: `0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)`.
    Lg,
    /// `shadow-xl`: `0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)`.
    Xl,
    /// `shadow-2xl`: `0 25px 50px -12px rgb(0 0 0 / 0.25)`.
    Xxl,
    /// `shadow-none`: paints nothing.
    None,
}

const LAYERS_XS: &[ShadowLayer] = &[ShadowLayer::new(0.0, 1.0, 2.0, 0.0, 0.05)];
const LAYERS_SM: &[ShadowLayer] = &[
    ShadowLayer::new(0.0, 1.0, 3.0, 0.0, 0.10),
    ShadowLayer::new(0.0, 1.0, 2.0, -1.0, 0.10),
];
const LAYERS_MD: &[ShadowLayer] = &[
    ShadowLayer::new(0.0, 4.0, 6.0, -1.0, 0.10),
    ShadowLayer::new(0.0, 2.0, 4.0, -2.0, 0.10),
];
const LAYERS_LG: &[ShadowLayer] = &[
    ShadowLayer::new(0.0, 10.0, 15.0, -3.0, 0.10),
    ShadowLayer::new(0.0, 4.0, 6.0, -4.0, 0.10),
];
const LAYERS_XL: &[ShadowLayer] = &[
    ShadowLayer::new(0.0, 20.0, 25.0, -5.0, 0.10),
    ShadowLayer::new(0.0, 8.0, 10.0, -6.0, 0.10),
];
const LAYERS_XXL: &[ShadowLayer] = &[ShadowLayer::new(0.0, 25.0, 50.0, -12.0, 0.25)];
const LAYERS_NONE: &[ShadowLayer] = &[];

impl ShadowSize {
    /// The CSS layers for this size, light values, in paint order (first
    /// layer is the outermost/softest, as in CSS).
    pub fn layers(self) -> &'static [ShadowLayer] {
        match self {
            ShadowSize::Xs => LAYERS_XS,
            ShadowSize::Sm => LAYERS_SM,
            ShadowSize::Md => LAYERS_MD,
            ShadowSize::Lg => LAYERS_LG,
            ShadowSize::Xl => LAYERS_XL,
            ShadowSize::Xxl => LAYERS_XXL,
            ShadowSize::None => LAYERS_NONE,
        }
    }

    /// Whether this size paints nothing.
    pub fn is_none(self) -> bool {
        matches!(self, ShadowSize::None)
    }
}

// ---------------------------------------------------------------------------
// Slots: the paint sites the override table speaks about.
// ---------------------------------------------------------------------------

/// A shadow paint site: one component slot whose style rule may declare a
/// shadow. The variant docs cite the snapshot `.cn-*` selector; slots with no
/// override in any style are documented as verified-silent so waves know the
/// silence was checked, not assumed. Conditional slots state their condition:
/// consult them only when it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShadowSlot {
    /// `.cn-alert-dialog-content` — alert-dialog panel.
    AlertDialogContent,
    /// `.cn-button-group-text` — button-group text slot.
    ButtonGroupText,
    /// `.cn-button-variant-outline` — outline button. The §5(b) ruling case:
    /// nova is silent, so the base `shadow-xs` survives under every style.
    ButtonOutline,
    /// `.cn-calendar-dropdown-root` — calendar caption dropdown-root button.
    CalendarDropdownRoot,
    /// `.cn-chart-tooltip` — chart tooltip panel.
    ChartTooltip,
    /// `.cn-checkbox` — checkbox box.
    Checkbox,
    /// `.cn-combobox-chips` — combobox chips container.
    ComboboxChips,
    /// `.cn-combobox-content` — combobox popover content.
    ComboboxContent,
    /// Combobox trigger button. Silent in all 8 style files (no
    /// `.cn-combobox-trigger` shadow declaration exists); resolution always
    /// returns the base value.
    ComboboxTrigger,
    /// `.cn-context-menu-content` — context-menu panel.
    ContextMenuContent,
    /// `.cn-context-menu-sub-content` — context-menu sub-panel (hyphenated
    /// selector; distinct from [`ShadowSlot::ContextMenuSubcontent`], which
    /// can declare a different value in the same style file).
    ContextMenuSubContent,
    /// `.cn-context-menu-subcontent` — context-menu sub-panel (unhyphenated
    /// selector; distinct from [`ShadowSlot::ContextMenuSubContent`]).
    ContextMenuSubcontent,
    /// `.cn-dialog-content` — dialog panel.
    DialogContent,
    /// `.cn-drawer-content` — drawer panel (Vaul).
    DrawerContent,
    /// `.cn-drawer-popup` — drawer panel (Base UI popup).
    DrawerPopup,
    /// `.cn-dropdown-menu-content` — dropdown-menu panel.
    DropdownMenuContent,
    /// `.cn-dropdown-menu-sub-content` — dropdown-menu sub-panel (hyphenated
    /// selector; distinct from [`ShadowSlot::DropdownMenuSubcontent`]).
    DropdownMenuSubContent,
    /// `.cn-dropdown-menu-subcontent` — dropdown-menu sub-panel
    /// (unhyphenated selector; distinct from
    /// [`ShadowSlot::DropdownMenuSubContent`]).
    DropdownMenuSubcontent,
    /// `.cn-hover-card-content` — hover-card panel.
    HoverCardContent,
    /// `.cn-input` — text input.
    Input,
    /// `.cn-input-group` — input-group container.
    InputGroup,
    /// Input-group nested inside combobox content: the
    /// `*:data-[slot=input-group]:shadow-none` descendant rule inside
    /// `.cn-combobox-content`. Consult only for a nested input-group.
    InputGroupInCombobox,
    /// `.cn-command-input-group` — input-group nested inside command content.
    /// Consult only for a nested input-group.
    InputGroupInCommand,
    /// `.cn-input-group-input` — the input inside an input-group.
    InputGroupInput,
    /// `.cn-input-group-textarea` — the textarea inside an input-group.
    InputGroupTextarea,
    /// `.cn-input-otp-slot` — input-OTP slot.
    InputOtpSlot,
    /// `.cn-menubar` — menubar bar.
    Menubar,
    /// `.cn-menubar-content` — menubar menu panel.
    MenubarContent,
    /// `.cn-menubar-sub-content` — menubar sub-panel.
    MenubarSubContent,
    /// `.cn-native-select` — native select trigger.
    NativeSelect,
    /// `.cn-navigation-menu-content`, `group-data-[viewport=false]` condition:
    /// the content panel when it floats WITHOUT a viewport. Consult only in
    /// the no-viewport case; with a viewport the viewport slot carries the
    /// shadow and the content keeps its base value.
    NavigationMenuContent,
    /// `.cn-navigation-menu-indicator-arrow` — navigation-menu indicator arrow.
    NavigationMenuIndicatorArrow,
    /// `.cn-navigation-menu-popup` — navigation-menu popup panel.
    NavigationMenuPopup,
    /// `.cn-navigation-menu-viewport` — navigation-menu viewport panel.
    NavigationMenuViewport,
    /// `.cn-popover-content` — popover panel.
    PopoverContent,
    /// `.cn-questionnaire-choice` — questionnaire choice row.
    QuestionnaireChoice,
    /// `.cn-questionnaire-input` — questionnaire input.
    QuestionnaireInput,
    /// `.cn-questionnaire-shortcut` — questionnaire shortcut chip.
    QuestionnaireShortcut,
    /// Radio-group item. Silent in all 8 style files (verified for nova in
    /// the §5(b) ruling, §3.4); resolution always returns the base value.
    RadioItem,
    /// `.cn-select-content` — select popover content.
    SelectContent,
    /// `.cn-select-trigger` — select trigger.
    SelectTrigger,
    /// `.cn-sheet-content` — sheet panel.
    SheetContent,
    /// `.cn-sidebar-inner`, `group-data-[variant=floating]` condition:
    /// consult only for the floating sidebar variant.
    SidebarInner,
    /// `.cn-sidebar-input` — sidebar search input.
    SidebarInput,
    /// `.cn-sidebar-inset`, `md:peer-data-[variant=inset]` condition:
    /// consult only for the inset sidebar variant (md breakpoint and up).
    SidebarInset,
    /// `.cn-slider-thumb` — slider thumb.
    SliderThumb,
    /// `.cn-switch` — switch track (the base recipe's `shadow-xs` slot).
    Switch,
    /// `.cn-switch-thumb` — switch thumb.
    SwitchThumb,
    /// `.cn-tabs-trigger`, `group-data-[variant=default]` + active condition:
    /// the ACTIVE trigger in the default tab-list variant. Consult only for
    /// an active trigger in the default variant.
    TabsTriggerDefault,
    /// `.cn-tabs-trigger`, `group-data-[variant=line]` + active condition:
    /// the ACTIVE trigger in the line tab-list variant. Consult only for an
    /// active trigger in the line variant.
    TabsTriggerLine,
    /// `.cn-textarea` — textarea.
    Textarea,
    /// `.cn-toggle-group` — toggle-group container. Vega's declaration is
    /// conditional (`data-[spacing=0]:data-[variant=outline]`); other
    /// styles are silent.
    ToggleGroup,
    /// `.cn-toggle-group-item`, `group-data-[spacing=0]` condition in the
    /// declaring styles: consult for spacing-0 items (the override is
    /// `shadow-none` wherever declared).
    ToggleGroupItem,
    /// `.cn-toggle-variant-outline` — outline toggle.
    ToggleOutline,
    /// Carousel arrow button (outline-button-like). Silent in all 8 style
    /// files (no `.cn-carousel-*` shadow declaration exists); resolution
    /// always returns the base value.
    CarouselArrow,
    /// Date-picker trigger button (outline-button-like). Silent in all 8
    /// style files; resolution always returns the base value.
    DatePickerTrigger,
}

// ---------------------------------------------------------------------------
// LAYER resolution: base value unless the style explicitly overrides.
// ---------------------------------------------------------------------------

/// The explicit style override for a (style, slot) pair, if the style's
/// `.cn-*` rule for that slot declares a shadow class. `None` means silence:
/// the caller keeps the base value. Extracted mechanically from the frozen
/// snapshot 2026-11-12; each arm cites its source rule. The arms share
/// bodies by design (it is a data table); merging them would destroy the
/// per-arm citations.
#[allow(clippy::match_same_arms)]
fn style_override(style: Style, slot: ShadowSlot) -> Option<ShadowSize> {
    match (style, slot) {
        // AlertDialogContent
        (Style::Luma, ShadowSlot::AlertDialogContent) => Some(ShadowSize::Xl), // style-luma.css:489
        (Style::Sera, ShadowSlot::AlertDialogContent) => Some(ShadowSize::Md), // style-sera.css:50
        (Style::Rhea, ShadowSlot::AlertDialogContent) => Some(ShadowSize::Xl), // style-rhea.css:489
        // ButtonGroupText
        (Style::Vega, ShadowSlot::ButtonGroupText) => Some(ShadowSize::Xs), // style-vega.css:219
        // ButtonOutline
        (Style::Vega, ShadowSlot::ButtonOutline) => Some(ShadowSize::Xs), // style-vega.css:154
        // CalendarDropdownRoot
        (Style::Vega, ShadowSlot::CalendarDropdownRoot) => Some(ShadowSize::Xs), // style-vega.css:232
        // ChartTooltip
        (Style::Nova, ShadowSlot::ChartTooltip) => Some(ShadowSize::Xl), // style-nova.css:279
        (Style::Vega, ShadowSlot::ChartTooltip) => Some(ShadowSize::Xl), // style-vega.css:279
        (Style::Maia, ShadowSlot::ChartTooltip) => Some(ShadowSize::Xl), // style-maia.css:279
        (Style::Lyra, ShadowSlot::ChartTooltip) => Some(ShadowSize::Xl), // style-lyra.css:258
        (Style::Mira, ShadowSlot::ChartTooltip) => Some(ShadowSize::Xl), // style-mira.css:279
        (Style::Luma, ShadowSlot::ChartTooltip) => Some(ShadowSize::Lg), // style-luma.css:1433
        (Style::Sera, ShadowSlot::ChartTooltip) => Some(ShadowSize::Sm), // style-sera.css:270
        (Style::Rhea, ShadowSlot::ChartTooltip) => Some(ShadowSize::Lg), // style-rhea.css:1433
        // Checkbox
        (Style::Vega, ShadowSlot::Checkbox) => Some(ShadowSize::Xs), // style-vega.css:284
        // ComboboxChips
        (Style::Vega, ShadowSlot::ComboboxChips) => Some(ShadowSize::Xs), // style-vega.css:345
        // ComboboxContent
        (Style::Nova, ShadowSlot::ComboboxContent) => Some(ShadowSize::Md), // style-nova.css:297
        (Style::Vega, ShadowSlot::ComboboxContent) => Some(ShadowSize::Md), // style-vega.css:297
        (Style::Maia, ShadowSlot::ComboboxContent) => Some(ShadowSize::Xxl), // style-maia.css:297
        (Style::Lyra, ShadowSlot::ComboboxContent) => Some(ShadowSize::Md), // style-lyra.css:276
        (Style::Mira, ShadowSlot::ComboboxContent) => Some(ShadowSize::Md), // style-mira.css:297
        (Style::Luma, ShadowSlot::ComboboxContent) => Some(ShadowSize::Lg), // style-luma.css:238
        (Style::Sera, ShadowSlot::ComboboxContent) => Some(ShadowSize::Md), // style-sera.css:288
        (Style::Rhea, ShadowSlot::ComboboxContent) => Some(ShadowSize::Lg), // style-rhea.css:238
        // ContextMenuContent
        (Style::Nova, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Md), // style-nova.css:415
        (Style::Vega, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Md), // style-vega.css:415
        (Style::Maia, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Xxl), // style-maia.css:415
        (Style::Lyra, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Md), // style-lyra.css:394
        (Style::Mira, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Md), // style-mira.css:415
        (Style::Luma, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Lg), // style-luma.css:667
        (Style::Sera, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Md), // style-sera.css:406
        (Style::Rhea, ShadowSlot::ContextMenuContent) => Some(ShadowSize::Lg), // style-rhea.css:667
        // ContextMenuSubContent
        (Style::Nova, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Lg), // style-nova.css:455
        (Style::Vega, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Lg), // style-vega.css:455
        (Style::Maia, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Lg), // style-maia.css:455
        (Style::Lyra, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Lg), // style-lyra.css:434
        (Style::Mira, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Md), // style-mira.css:455
        (Style::Luma, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Lg), // style-luma.css:707
        (Style::Sera, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Md), // style-sera.css:446
        (Style::Rhea, ShadowSlot::ContextMenuSubContent) => Some(ShadowSize::Lg), // style-rhea.css:707
        // ContextMenuSubcontent
        (Style::Nova, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Lg), // style-nova.css:459
        (Style::Vega, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Lg), // style-vega.css:459
        (Style::Maia, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Lg), // style-maia.css:459
        (Style::Lyra, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Lg), // style-lyra.css:438
        (Style::Mira, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Lg), // style-mira.css:459
        (Style::Luma, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Lg), // style-luma.css:711
        (Style::Sera, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Md), // style-sera.css:450
        (Style::Rhea, ShadowSlot::ContextMenuSubcontent) => Some(ShadowSize::Lg), // style-rhea.css:711
        // DialogContent
        (Style::Luma, ShadowSlot::DialogContent) => Some(ShadowSize::Xl), // style-luma.css:720
        (Style::Sera, ShadowSlot::DialogContent) => Some(ShadowSize::Md), // style-sera.css:459
        (Style::Rhea, ShadowSlot::DialogContent) => Some(ShadowSize::Xl), // style-rhea.css:720
        // DrawerContent
        (Style::Luma, ShadowSlot::DrawerContent) => Some(ShadowSize::Xl), // style-luma.css:749 (before: pseudo-element shadow)
        (Style::Rhea, ShadowSlot::DrawerContent) => Some(ShadowSize::Xl), // style-rhea.css:749 (before: pseudo-element shadow)
        // DrawerPopup
        (Style::Luma, ShadowSlot::DrawerPopup) => Some(ShadowSize::Xl), // style-luma.css:753
        (Style::Rhea, ShadowSlot::DrawerPopup) => Some(ShadowSize::Xl), // style-rhea.css:753
        // DropdownMenuContent
        (Style::Nova, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Md), // style-nova.css:538
        (Style::Vega, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Md), // style-vega.css:538
        (Style::Maia, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Xxl), // style-maia.css:539
        (Style::Lyra, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Md), // style-lyra.css:513
        (Style::Mira, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Md), // style-mira.css:538
        (Style::Luma, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Lg), // style-luma.css:790
        (Style::Sera, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Md), // style-sera.css:529
        (Style::Rhea, ShadowSlot::DropdownMenuContent) => Some(ShadowSize::Lg), // style-rhea.css:790
        // DropdownMenuSubContent
        (Style::Nova, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Lg), // style-nova.css:578
        (Style::Vega, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Lg), // style-vega.css:578
        (Style::Maia, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Xxl), // style-maia.css:579
        (Style::Lyra, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Lg), // style-lyra.css:553
        (Style::Mira, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Md), // style-mira.css:578
        (Style::Luma, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Lg), // style-luma.css:830
        (Style::Sera, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Md), // style-sera.css:569
        (Style::Rhea, ShadowSlot::DropdownMenuSubContent) => Some(ShadowSize::Lg), // style-rhea.css:830
        // DropdownMenuSubcontent
        (Style::Nova, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Lg), // style-nova.css:582
        (Style::Vega, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Lg), // style-vega.css:582
        (Style::Maia, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Lg), // style-maia.css:583
        (Style::Lyra, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Lg), // style-lyra.css:557
        (Style::Mira, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Lg), // style-mira.css:582
        (Style::Luma, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Lg), // style-luma.css:834
        (Style::Sera, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Md), // style-sera.css:573
        (Style::Rhea, ShadowSlot::DropdownMenuSubcontent) => Some(ShadowSize::Lg), // style-rhea.css:834
        // HoverCardContent
        (Style::Nova, ShadowSlot::HoverCardContent) => Some(ShadowSize::Md), // style-nova.css:669
        (Style::Vega, ShadowSlot::HoverCardContent) => Some(ShadowSize::Md), // style-vega.css:669
        (Style::Maia, ShadowSlot::HoverCardContent) => Some(ShadowSize::Xxl), // style-maia.css:670
        (Style::Lyra, ShadowSlot::HoverCardContent) => Some(ShadowSize::Md), // style-lyra.css:644
        (Style::Mira, ShadowSlot::HoverCardContent) => Some(ShadowSize::Md), // style-mira.css:669
        (Style::Luma, ShadowSlot::HoverCardContent) => Some(ShadowSize::Lg), // style-luma.css:1438
        (Style::Sera, ShadowSlot::HoverCardContent) => Some(ShadowSize::Md), // style-sera.css:660
        (Style::Rhea, ShadowSlot::HoverCardContent) => Some(ShadowSize::Lg), // style-rhea.css:1438
        // Input
        (Style::Vega, ShadowSlot::Input) => Some(ShadowSize::Xs), // style-vega.css:678
        // InputGroup
        (Style::Vega, ShadowSlot::InputGroup) => Some(ShadowSize::Xs), // style-vega.css:1398
        // InputGroupInCombobox
        (Style::Nova, ShadowSlot::InputGroupInCombobox) => Some(ShadowSize::None), // style-nova.css:297 (descendant *:data-[slot=input-group])
        (Style::Vega, ShadowSlot::InputGroupInCombobox) => Some(ShadowSize::None), // style-vega.css:297 (descendant *:data-[slot=input-group])
        (Style::Maia, ShadowSlot::InputGroupInCombobox) => Some(ShadowSize::None), // style-maia.css:297 (descendant *:data-[slot=input-group])
        (Style::Lyra, ShadowSlot::InputGroupInCombobox) => Some(ShadowSize::None), // style-lyra.css:276 (descendant *:data-[slot=input-group])
        (Style::Mira, ShadowSlot::InputGroupInCombobox) => Some(ShadowSize::None), // style-mira.css:297 (descendant *:data-[slot=input-group])
        (Style::Luma, ShadowSlot::InputGroupInCombobox) => Some(ShadowSize::None), // style-luma.css:238 (descendant *:data-[slot=input-group])
        (Style::Rhea, ShadowSlot::InputGroupInCombobox) => Some(ShadowSize::None), // style-rhea.css:238 (descendant *:data-[slot=input-group])
        // InputGroupInCommand
        (Style::Nova, ShadowSlot::InputGroupInCommand) => Some(ShadowSize::None), // style-nova.css:370
        (Style::Vega, ShadowSlot::InputGroupInCommand) => Some(ShadowSize::None), // style-vega.css:370
        (Style::Lyra, ShadowSlot::InputGroupInCommand) => Some(ShadowSize::None), // style-lyra.css:349
        // InputGroupInput
        (Style::Nova, ShadowSlot::InputGroupInput) => Some(ShadowSize::None), // style-nova.css:1442
        (Style::Vega, ShadowSlot::InputGroupInput) => Some(ShadowSize::None), // style-vega.css:1442
        (Style::Maia, ShadowSlot::InputGroupInput) => Some(ShadowSize::None), // style-maia.css:1439
        (Style::Lyra, ShadowSlot::InputGroupInput) => Some(ShadowSize::None), // style-lyra.css:1417
        (Style::Mira, ShadowSlot::InputGroupInput) => Some(ShadowSize::None), // style-mira.css:1444
        (Style::Luma, ShadowSlot::InputGroupInput) => Some(ShadowSize::None), // style-luma.css:343
        (Style::Rhea, ShadowSlot::InputGroupInput) => Some(ShadowSize::None), // style-rhea.css:343
        // InputGroupTextarea
        (Style::Nova, ShadowSlot::InputGroupTextarea) => Some(ShadowSize::None), // style-nova.css:1446
        (Style::Vega, ShadowSlot::InputGroupTextarea) => Some(ShadowSize::None), // style-vega.css:1446
        (Style::Maia, ShadowSlot::InputGroupTextarea) => Some(ShadowSize::None), // style-maia.css:1443
        (Style::Lyra, ShadowSlot::InputGroupTextarea) => Some(ShadowSize::None), // style-lyra.css:1421
        (Style::Mira, ShadowSlot::InputGroupTextarea) => Some(ShadowSize::None), // style-mira.css:1448
        (Style::Luma, ShadowSlot::InputGroupTextarea) => Some(ShadowSize::None), // style-luma.css:347
        (Style::Rhea, ShadowSlot::InputGroupTextarea) => Some(ShadowSize::None), // style-rhea.css:347
        // InputOtpSlot
        (Style::Vega, ShadowSlot::InputOtpSlot) => Some(ShadowSize::Xs), // style-vega.css:691
        // Menubar
        (Style::Vega, ShadowSlot::Menubar) => Some(ShadowSize::Xs), // style-vega.css:799
        // MenubarContent
        (Style::Nova, ShadowSlot::MenubarContent) => Some(ShadowSize::Md), // style-nova.css:807
        (Style::Vega, ShadowSlot::MenubarContent) => Some(ShadowSize::Md), // style-vega.css:807
        (Style::Maia, ShadowSlot::MenubarContent) => Some(ShadowSize::Xxl), // style-maia.css:808
        (Style::Lyra, ShadowSlot::MenubarContent) => Some(ShadowSize::Md), // style-lyra.css:782
        (Style::Mira, ShadowSlot::MenubarContent) => Some(ShadowSize::Md), // style-mira.css:807
        (Style::Luma, ShadowSlot::MenubarContent) => Some(ShadowSize::Lg), // style-luma.css:942
        (Style::Sera, ShadowSlot::MenubarContent) => Some(ShadowSize::Md), // style-sera.css:798
        (Style::Rhea, ShadowSlot::MenubarContent) => Some(ShadowSize::Lg), // style-rhea.css:942
        // MenubarSubContent
        (Style::Nova, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Lg), // style-nova.css:851
        (Style::Vega, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Lg), // style-vega.css:851
        (Style::Maia, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Xxl), // style-maia.css:852
        (Style::Lyra, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Lg), // style-lyra.css:826
        (Style::Mira, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Md), // style-mira.css:851
        (Style::Luma, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Lg), // style-luma.css:986
        (Style::Sera, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Md), // style-sera.css:842
        (Style::Rhea, ShadowSlot::MenubarSubContent) => Some(ShadowSize::Lg), // style-rhea.css:986
        // NativeSelect
        (Style::Vega, ShadowSlot::NativeSelect) => Some(ShadowSize::Xs), // style-vega.css:901
        // NavigationMenuContent
        (Style::Maia, ShadowSlot::NavigationMenuContent) => Some(ShadowSize::Xxl), // style-maia.css:877 (conditional: viewport=false (content floats on its own))
        (Style::Mira, ShadowSlot::NavigationMenuContent) => Some(ShadowSize::Md), // style-mira.css:876 (conditional: viewport=false (content floats on its own))
        (Style::Luma, ShadowSlot::NavigationMenuContent) => Some(ShadowSize::Lg), // style-luma.css:1011 (conditional: viewport=false (content floats on its own))
        (Style::Sera, ShadowSlot::NavigationMenuContent) => Some(ShadowSize::Md), // style-sera.css:867 (conditional: viewport=false (content floats on its own))
        (Style::Rhea, ShadowSlot::NavigationMenuContent) => Some(ShadowSize::Lg), // style-rhea.css:1011 (conditional: viewport=false (content floats on its own))
        // NavigationMenuIndicatorArrow
        (Style::Nova, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-nova.css:888
        (Style::Vega, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-vega.css:888
        (Style::Maia, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-maia.css:889
        (Style::Lyra, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-lyra.css:863
        (Style::Mira, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-mira.css:888
        (Style::Luma, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-luma.css:1023
        (Style::Sera, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-sera.css:879
        (Style::Rhea, ShadowSlot::NavigationMenuIndicatorArrow) => Some(ShadowSize::Md), // style-rhea.css:1023
        // NavigationMenuPopup
        (Style::Luma, ShadowSlot::NavigationMenuPopup) => Some(ShadowSize::Lg), // style-luma.css:1031
        (Style::Sera, ShadowSlot::NavigationMenuPopup) => Some(ShadowSize::Md), // style-sera.css:887
        (Style::Rhea, ShadowSlot::NavigationMenuPopup) => Some(ShadowSize::Lg), // style-rhea.css:1031
        // NavigationMenuViewport
        (Style::Maia, ShadowSlot::NavigationMenuViewport) => Some(ShadowSize::Xxl), // style-maia.css:881
        (Style::Mira, ShadowSlot::NavigationMenuViewport) => Some(ShadowSize::Md), // style-mira.css:880
        (Style::Luma, ShadowSlot::NavigationMenuViewport) => Some(ShadowSize::Lg), // style-luma.css:1015
        (Style::Sera, ShadowSlot::NavigationMenuViewport) => Some(ShadowSize::Md), // style-sera.css:871
        (Style::Rhea, ShadowSlot::NavigationMenuViewport) => Some(ShadowSize::Lg), // style-rhea.css:1015
        // PopoverContent
        (Style::Nova, ShadowSlot::PopoverContent) => Some(ShadowSize::Md), // style-nova.css:927
        (Style::Vega, ShadowSlot::PopoverContent) => Some(ShadowSize::Md), // style-vega.css:927
        (Style::Maia, ShadowSlot::PopoverContent) => Some(ShadowSize::Xxl), // style-maia.css:928
        (Style::Lyra, ShadowSlot::PopoverContent) => Some(ShadowSize::Md), // style-lyra.css:902
        (Style::Mira, ShadowSlot::PopoverContent) => Some(ShadowSize::Md), // style-mira.css:927
        (Style::Luma, ShadowSlot::PopoverContent) => Some(ShadowSize::Lg), // style-luma.css:1166
        (Style::Sera, ShadowSlot::PopoverContent) => Some(ShadowSize::Md), // style-sera.css:918
        (Style::Rhea, ShadowSlot::PopoverContent) => Some(ShadowSize::Lg), // style-rhea.css:1166
        // QuestionnaireChoice
        (Style::Vega, ShadowSlot::QuestionnaireChoice) => Some(ShadowSize::Xs), // style-vega.css:1623
        // QuestionnaireInput
        (Style::Vega, ShadowSlot::QuestionnaireInput) => Some(ShadowSize::Xs), // style-vega.css:1655
        // QuestionnaireShortcut
        (Style::Vega, ShadowSlot::QuestionnaireShortcut) => Some(ShadowSize::Xs), // style-vega.css:1647
        // SelectContent
        (Style::Nova, ShadowSlot::SelectContent) => Some(ShadowSize::Md), // style-nova.css:1020
        (Style::Vega, ShadowSlot::SelectContent) => Some(ShadowSize::Md), // style-vega.css:1020
        (Style::Maia, ShadowSlot::SelectContent) => Some(ShadowSize::Xxl), // style-maia.css:1021
        (Style::Lyra, ShadowSlot::SelectContent) => Some(ShadowSize::Md), // style-lyra.css:995
        (Style::Mira, ShadowSlot::SelectContent) => Some(ShadowSize::Md), // style-mira.css:1020
        (Style::Luma, ShadowSlot::SelectContent) => Some(ShadowSize::Lg), // style-luma.css:180
        (Style::Sera, ShadowSlot::SelectContent) => Some(ShadowSize::Md), // style-sera.css:1011
        (Style::Rhea, ShadowSlot::SelectContent) => Some(ShadowSize::Lg), // style-rhea.css:180
        // SelectTrigger
        (Style::Vega, ShadowSlot::SelectTrigger) => Some(ShadowSize::Xs), // style-vega.css:1004
        // SheetContent
        (Style::Nova, ShadowSlot::SheetContent) => Some(ShadowSize::Lg), // style-nova.css:1086
        (Style::Vega, ShadowSlot::SheetContent) => Some(ShadowSize::Lg), // style-vega.css:1086
        (Style::Maia, ShadowSlot::SheetContent) => Some(ShadowSize::Lg), // style-maia.css:1087
        (Style::Lyra, ShadowSlot::SheetContent) => Some(ShadowSize::Lg), // style-lyra.css:1061
        (Style::Mira, ShadowSlot::SheetContent) => Some(ShadowSize::Lg), // style-mira.css:1086
        (Style::Luma, ShadowSlot::SheetContent) => Some(ShadowSize::Xl), // style-luma.css:1239
        (Style::Sera, ShadowSlot::SheetContent) => Some(ShadowSize::Md), // style-sera.css:1077
        (Style::Rhea, ShadowSlot::SheetContent) => Some(ShadowSize::Xl), // style-rhea.css:1239
        // SidebarInner
        (Style::Nova, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-nova.css:1115 (conditional: floating variant)
        (Style::Vega, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-vega.css:1115 (conditional: floating variant)
        (Style::Maia, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-maia.css:1116 (conditional: floating variant)
        (Style::Lyra, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-lyra.css:1090 (conditional: floating variant)
        (Style::Mira, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-mira.css:1117 (conditional: floating variant)
        (Style::Luma, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-luma.css:1057 (conditional: floating variant)
        (Style::Sera, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-sera.css:1106 (conditional: floating variant)
        (Style::Rhea, ShadowSlot::SidebarInner) => Some(ShadowSize::Sm), // style-rhea.css:1057 (conditional: floating variant)
        // SidebarInput
        (Style::Nova, ShadowSlot::SidebarInput) => Some(ShadowSize::None), // style-nova.css:1127
        (Style::Vega, ShadowSlot::SidebarInput) => Some(ShadowSize::None), // style-vega.css:1127
        (Style::Maia, ShadowSlot::SidebarInput) => Some(ShadowSize::None), // style-maia.css:1128
        (Style::Lyra, ShadowSlot::SidebarInput) => Some(ShadowSize::None), // style-lyra.css:1102
        (Style::Luma, ShadowSlot::SidebarInput) => Some(ShadowSize::None), // style-luma.css:1069
        (Style::Sera, ShadowSlot::SidebarInput) => Some(ShadowSize::None), // style-sera.css:1118
        (Style::Rhea, ShadowSlot::SidebarInput) => Some(ShadowSize::None), // style-rhea.css:1069
        // SidebarInset
        (Style::Nova, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-nova.css:1123 (conditional: inset variant (md+))
        (Style::Vega, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-vega.css:1123 (conditional: inset variant (md+))
        (Style::Maia, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-maia.css:1124 (conditional: inset variant (md+))
        (Style::Lyra, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-lyra.css:1098 (conditional: inset variant (md+))
        (Style::Mira, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-mira.css:1125 (conditional: inset variant (md+))
        (Style::Luma, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-luma.css:1065 (conditional: inset variant (md+))
        (Style::Sera, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-sera.css:1114 (conditional: inset variant (md+))
        (Style::Rhea, ShadowSlot::SidebarInset) => Some(ShadowSize::Sm), // style-rhea.css:1065 (conditional: inset variant (md+))
        // SliderThumb
        (Style::Vega, ShadowSlot::SliderThumb) => Some(ShadowSize::Sm), // style-vega.css:1241
        (Style::Maia, ShadowSlot::SliderThumb) => Some(ShadowSize::Sm), // style-maia.css:1242
        (Style::Luma, ShadowSlot::SliderThumb) => Some(ShadowSize::Md), // style-luma.css:16
        (Style::Rhea, ShadowSlot::SliderThumb) => Some(ShadowSize::Md), // style-rhea.css:16
        // Switch
        (Style::Vega, ShadowSlot::Switch) => Some(ShadowSize::Xs), // style-vega.css:1251
        // SwitchThumb
        (Style::Luma, ShadowSlot::SwitchThumb) => Some(ShadowSize::Sm), // style-luma.css:29
        (Style::Rhea, ShadowSlot::SwitchThumb) => Some(ShadowSize::Sm), // style-rhea.css:29
        // TabsTriggerDefault
        (Style::Nova, ShadowSlot::TabsTriggerDefault) => Some(ShadowSize::Sm), // style-nova.css:1325 (conditional: default variant, active)
        (Style::Vega, ShadowSlot::TabsTriggerDefault) => Some(ShadowSize::Sm), // style-vega.css:1325 (conditional: default variant, active)
        // TabsTriggerLine
        (Style::Nova, ShadowSlot::TabsTriggerLine) => Some(ShadowSize::None), // style-nova.css:1325 (conditional: line variant, active)
        (Style::Vega, ShadowSlot::TabsTriggerLine) => Some(ShadowSize::None), // style-vega.css:1325 (conditional: line variant, active)
        // Textarea
        (Style::Vega, ShadowSlot::Textarea) => Some(ShadowSize::Xs), // style-vega.css:1338
        // ToggleGroup
        (Style::Vega, ShadowSlot::ToggleGroup) => Some(ShadowSize::Xs), // style-vega.css:1372 (conditional: spacing=0 + outline variant)
        // ToggleGroupItem
        (Style::Vega, ShadowSlot::ToggleGroupItem) => Some(ShadowSize::None), // style-vega.css:1376 (conditional: spacing=0)
        (Style::Maia, ShadowSlot::ToggleGroupItem) => Some(ShadowSize::None), // style-maia.css:1373 (conditional: spacing=0)
        (Style::Luma, ShadowSlot::ToggleGroupItem) => Some(ShadowSize::None), // style-luma.css:1356 (conditional: spacing=0)
        (Style::Sera, ShadowSlot::ToggleGroupItem) => Some(ShadowSize::None), // style-sera.css:1363 (conditional: spacing=0)
        (Style::Rhea, ShadowSlot::ToggleGroupItem) => Some(ShadowSize::None), // style-rhea.css:1356 (conditional: spacing=0)
        // ToggleOutline
        (Style::Vega, ShadowSlot::ToggleOutline) => Some(ShadowSize::Xs), // style-vega.css:1355
        _ => None,
    }
}

/// Resolve the shadow for one paint site: the base recipe's shadow, unless
/// the active style's rule for that exact slot explicitly declares a shadow
/// (see the module docs for the LAYER ruling). `base` is the shadow the base
/// recipe (`.tsx` class string) assigns the slot; callers pass it through
/// unchanged when the style is silent.
pub fn resolve_shadow(base: ShadowSize, style: Style, slot: ShadowSlot) -> ShadowSize {
    style_override(style, slot).unwrap_or(base)
}

// ---------------------------------------------------------------------------
// Rasterizer: shadow shape -> blurred alpha atlas.
// ---------------------------------------------------------------------------

/// Number of Gaussian sigma the rasterizer extends past a shadow shape
/// (3σ captures 99.7% of the blur energy; the remainder is sub-1/255 alpha).
const BLUR_SIGMA_EXTENT: f32 = 3.0;

/// Maximum corner-patch size in pixels. Huge blurs (shadow-2xl) render at a
/// reduced scale instead of growing the atlas without bound; a 50px blur
/// hides the downsampling completely.
const MAX_PATCH_PX: f32 = 256.0;

/// Maximum entries in the texture cache (LRU). Typical entries are a few KB;
/// the largest (shadow-2xl at high DPI) is ~2.5 MB, so the worst case stays
/// well under 100 MB and the realistic case under 1 MB.
const MAX_CACHE_ENTRIES: usize = 32;

/// How far past the widget rect a shadow's falloff extends, in points: the
/// worst layer's offset + spread + 3σ. At least 1pt so even hard (blur 0)
/// shadows have falloff room for the 1px edge antialiasing.
fn shadow_extent(layers: &[ShadowLayer]) -> f32 {
    layers
        .iter()
        .map(|l| {
            let sigma = (l.blur * 0.5).max(0.0);
            l.offset[0].abs().max(l.offset[1].abs()) + l.spread + BLUR_SIGMA_EXTENT * sigma
        })
        .fold(1.0f32, f32::max)
}

/// How far inward from the shape edge a corner's influence reaches, in
/// points: the worst layer's offset + spread + corner radius + 3σ. The
/// corner patches are this wide (plus the extent) so that where they end,
/// the edge profile is the pure straight-edge profile.
fn corner_reach(layers: &[ShadowLayer], radius_max: f32) -> f32 {
    layers
        .iter()
        .map(|l| {
            let sigma = (l.blur * 0.5).max(0.0);
            let r = (radius_max + l.spread).max(0.0);
            l.offset[0].abs().max(l.offset[1].abs()) - l.spread + r + BLUR_SIGMA_EXTENT * sigma
        })
        .fold(0.0f32, f32::max)
}

/// Separable Gaussian blur of a single-channel buffer, in place (via a temp
/// buffer). Edge texels are clamped (replicated), which is correct here: the
/// buffers extend past the shadow on every side. `sigma_px <= 0` is a no-op.
fn gaussian_blur(buf: &mut [f32], w: usize, h: usize, sigma_px: f32) {
    if sigma_px <= 0.0 || buf.is_empty() {
        return;
    }
    let radius = (BLUR_SIGMA_EXTENT * sigma_px).ceil() as usize;
    let mut kernel = Vec::with_capacity(2 * radius + 1);
    let mut sum = 0.0f32;
    for k in -(radius as i32)..=(radius as i32) {
        let x = k as f32 / sigma_px;
        let v = (-0.5 * x * x).exp();
        kernel.push(v);
        sum += v;
    }
    for v in &mut kernel {
        *v /= sum;
    }
    let mut tmp = vec![0.0f32; buf.len()];
    // Horizontal pass.
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sx = (x as i32 + ki as i32 - radius as i32).clamp(0, w as i32 - 1) as usize;
                acc += kv * buf[y * w + sx];
            }
            tmp[y * w + x] = acc;
        }
    }
    // Vertical pass.
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sy = (y as i32 + ki as i32 - radius as i32).clamp(0, h as i32 - 1) as usize;
                acc += kv * tmp[sy * w + x];
            }
            buf[y * w + x] = acc;
        }
    }
}

/// Signed distance to a rounded box, negative inside. `center`/`half` are the
/// box center and half-size; `r` the corner radius. Verified against the
/// center, edge-midpoint, and arc points (see tests).
fn sd_round_box(p: Vec2, center: Vec2, half: Vec2, r: f32) -> f32 {
    let q = Vec2::new(
        (p.x - center.x).abs() - (half.x - r),
        (p.y - center.y).abs() - (half.y - r),
    );
    let outside = Vec2::new(q.x.max(0.0), q.y.max(0.0)).length();
    let inside = q.x.max(q.y).min(0.0);
    outside + inside - r
}

/// Signed distance to a rounded rect with per-corner radii (NW, NE, SW, SE).
/// Each quadrant uses its own corner's radius; on the straight edges the
/// result is radius-independent, so the field is continuous across quadrant
/// boundaries. With uniform radii it equals [`sd_round_box`].
fn sd_round_rect(p: Vec2, center: Vec2, half: Vec2, radii: [f32; 4]) -> f32 {
    let r = if p.x < center.x {
        if p.y < center.y {
            radii[0]
        } else {
            radii[2]
        }
    } else if p.y < center.y {
        radii[1]
    } else {
        radii[3]
    };
    sd_round_box(p, center, half, r)
}

/// Render one corner patch: the shadow's alpha over a `patch`×`patch` point
/// region at the widget corner, at `scale` pixels per point. `flip_x` /
/// `flip_y` mirror the geometry for the NE/SW/SE corners (offsets mirror
/// with the corner). Returns row-major alpha in [0, 1].
#[allow(clippy::too_many_arguments)]
fn render_corner(
    patch_px: usize,
    patch: f32,
    scale: f32,
    extent: f32,
    radius: f32,
    layers: &[ShadowLayer],
    dark: bool,
    flip_x: bool,
    flip_y: bool,
) -> Vec<f32> {
    let mut acc = vec![0.0f32; patch_px * patch_px];
    // The shape is a huge box whose near corner is the widget corner; the
    // far edges are far outside the patch and never influence the SDF here.
    const BIG: f32 = 4096.0;
    for layer in layers {
        let ox = if flip_x {
            -layer.offset[0]
        } else {
            layer.offset[0]
        };
        let oy = if flip_y {
            -layer.offset[1]
        } else {
            layer.offset[1]
        };
        let ex = ox - layer.spread;
        let ey = oy - layer.spread;
        let r = (radius + layer.spread).max(0.0);
        let center = Vec2::new(ex + BIG, ey + BIG);
        let mut buf = vec![0.0f32; patch_px * patch_px];
        for j in 0..patch_px {
            for i in 0..patch_px {
                let mut px = (i as f32 + 0.5) / scale;
                let mut py = (j as f32 + 0.5) / scale;
                if flip_x {
                    px = patch - px;
                }
                if flip_y {
                    py = patch - py;
                }
                let sx = px - extent;
                let sy = py - extent;
                let sdf = sd_round_box(Vec2::new(sx, sy), center, Vec2::splat(BIG), r);
                // 1px linear antialiasing on the shape edge (matters when
                // blur is 0; the blur dominates otherwise).
                buf[j * patch_px + i] = (0.5 - sdf * scale).clamp(0.0, 1.0);
            }
        }
        gaussian_blur(
            &mut buf,
            patch_px,
            patch_px,
            (layer.blur * 0.5).max(0.0) * scale,
        );
        let a = layer.resolved_alpha(dark);
        for (dst, src) in acc.iter_mut().zip(buf.iter()) {
            *dst += a * src * (1.0 - *dst);
        }
    }
    // Interior mask (SEL-VIS-01): the shadow contributes nothing strictly
    // inside the widget's rounded rect. Applied AFTER the blur, so the
    // exterior profile is exactly the unmasked one; only texels whose center
    // lies strictly inside the unoffset widget rect (SDF < 0) are zeroed.
    // The widget's 1px border paints over the mask boundary.
    for j in 0..patch_px {
        for i in 0..patch_px {
            let mut px = (i as f32 + 0.5) / scale;
            let mut py = (j as f32 + 0.5) / scale;
            if flip_x {
                px = patch - px;
            }
            if flip_y {
                py = patch - py;
            }
            let sx = px - extent;
            let sy = py - extent;
            let sdf = sd_round_box(
                Vec2::new(sx, sy),
                Vec2::new(BIG, BIG),
                Vec2::splat(BIG),
                radius,
            );
            if sdf < 0.0 {
                acc[j * patch_px + i] = 0.0;
            }
        }
    }
    acc
}

/// The straight-edge alpha profile over the `patch`-point band, sampled on
/// the same grid as the corner patches: index `j` is distance
/// `(j + 0.5) / scale - extent` from the shape edge (negative = outside).
/// `sign` is +1 for the top/left edges (offset pushes the shape inward) and
/// -1 for the bottom/right edges (mirrored geometry).
fn edge_profile(
    patch_px: usize,
    scale: f32,
    extent: f32,
    layers: &[ShadowLayer],
    dark: bool,
    axis: usize,
    sign: f32,
) -> Vec<f32> {
    let mut acc = vec![0.0f32; patch_px];
    for layer in layers {
        let o = sign * layer.offset[axis];
        let edge = o - layer.spread;
        let mut buf = vec![0.0f32; patch_px];
        for (j, v) in buf.iter_mut().enumerate() {
            let t = (j as f32 + 0.5) / scale - extent;
            *v = (0.5 + (t - edge) * scale).clamp(0.0, 1.0);
        }
        gaussian_blur(&mut buf, patch_px, 1, (layer.blur * 0.5).max(0.0) * scale);
        let a = layer.resolved_alpha(dark);
        for (dst, src) in acc.iter_mut().zip(buf.iter()) {
            *dst += a * src * (1.0 - *dst);
        }
    }
    // Interior mask (SEL-VIS-01): zero the profile strictly inside the
    // widget edge (t > 0). Applied after the blur, so the exterior profile
    // (t <= 0) is exactly the unmasked one.
    for (j, v) in acc.iter_mut().enumerate() {
        let t = (j as f32 + 0.5) / scale - extent;
        if t > 0.0 {
            *v = 0.0;
        }
    }
    acc
}

/// Atlas geometry: the patch/extent sizes in points plus the UV rect of each
/// of the 8 regions (corners NW/NE/SW/SE, strips top/bottom/left/right).
/// There is no center fill: the center region of the nine-slice always lies
/// strictly inside the widget rect (see `draw_shadow_atlas`), and the shadow
/// contributes nothing there (SEL-VIS-01).
#[derive(Clone, Copy, Debug)]
struct AtlasGeom {
    extent: f32,
    patch: f32,
    uvs: [Rect; 8],
}

/// Build the atlas image + geometry for one (layers, radii, dark, ppp) key.
/// Layout is a single row: 4 corner patches, 2 vertical strips (4px wide),
/// 2 horizontal strips (4px tall). UVs are inset half a texel so bilinear
/// filtering never bleeds across region boundaries.
fn rasterize_atlas(
    radii: [f32; 4],
    layers: &[ShadowLayer],
    dark: bool,
    pixels_per_point: f32,
) -> (ColorImage, AtlasGeom) {
    let extent = shadow_extent(layers);
    let radius_max = radii.iter().cloned().fold(0.0f32, f32::max);
    let patch = extent + corner_reach(layers, radius_max);
    let scale = pixels_per_point.min(MAX_PATCH_PX / patch).max(0.05);
    let n = ((patch * scale).ceil() as usize).max(1);
    const STRIP: usize = 4;
    let w = 6 * n + 2 * STRIP;
    let h = n;
    let mut pixels = vec![Color32::TRANSPARENT; w * h];
    let put = |pixels: &mut [Color32], x0: usize, y0: usize, rw: usize, alpha: &[f32]| {
        for j in 0..alpha.len() / rw {
            for i in 0..rw {
                let a = alpha[j * rw + i];
                pixels[(y0 + j) * w + (x0 + i)] =
                    Color32::from_black_alpha((a * 255.0).round().clamp(0.0, 255.0) as u8);
            }
        }
    };
    // Corners: NW, NE, SW, SE.
    let flips = [(false, false), (true, false), (false, true), (true, true)];
    for (k, &(fx, fy)) in flips.iter().enumerate() {
        let buf = render_corner(n, patch, scale, extent, radii[k], layers, dark, fx, fy);
        put(&mut pixels, k * n, 0, n, &buf);
    }
    // Strips: top, bottom (vertical), left, right (horizontal). The bottom
    // and right profiles are the mirrored ones, sampled in reverse order so
    // row/column 0 is the outer edge in every strip.
    let top = edge_profile(n, scale, extent, layers, dark, 1, 1.0);
    let bottom = edge_profile(n, scale, extent, layers, dark, 1, -1.0);
    let left = edge_profile(n, scale, extent, layers, dark, 0, 1.0);
    let right = edge_profile(n, scale, extent, layers, dark, 0, -1.0);
    let strip_v = |pixels: &mut [Color32], x0: usize, profile: &[f32], reverse: bool| {
        for j in 0..n {
            let a = profile[if reverse { n - 1 - j } else { j }];
            let c = Color32::from_black_alpha((a * 255.0).round().clamp(0.0, 255.0) as u8);
            for i in 0..STRIP {
                pixels[j * w + (x0 + i)] = c;
            }
        }
    };
    let strip_h = |pixels: &mut [Color32], x0: usize, profile: &[f32], reverse: bool| {
        for i in 0..n {
            let a = profile[if reverse { n - 1 - i } else { i }];
            let c = Color32::from_black_alpha((a * 255.0).round().clamp(0.0, 255.0) as u8);
            for j in 0..STRIP {
                pixels[j * w + (x0 + i)] = c;
            }
        }
    };
    strip_v(&mut pixels, 4 * n, &top, false);
    strip_v(&mut pixels, 4 * n + STRIP, &bottom, true);
    strip_h(&mut pixels, 4 * n + 2 * STRIP, &left, false);
    strip_h(&mut pixels, 5 * n + 2 * STRIP, &right, true);
    let uv = |x: usize, y: usize, rw: usize, rh: usize| {
        Rect::from_min_max(
            Vec2::new((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32).to_pos2(),
            Vec2::new(
                (x as f32 + rw as f32 - 0.5) / w as f32,
                (y as f32 + rh as f32 - 0.5) / h as f32,
            )
            .to_pos2(),
        )
    };
    let uvs = [
        uv(0, 0, n, n),
        uv(n, 0, n, n),
        uv(2 * n, 0, n, n),
        uv(3 * n, 0, n, n),
        uv(4 * n, 0, STRIP, n),
        uv(4 * n + STRIP, 0, STRIP, n),
        uv(4 * n + 2 * STRIP, 0, n, STRIP),
        uv(5 * n + 2 * STRIP, 0, n, STRIP),
    ];
    (
        ColorImage::new([w, h], pixels),
        AtlasGeom { extent, patch, uvs },
    )
}

/// Maximum side of a full-rect texture in pixels. The fallback only triggers
/// for small widgets, so this cap rarely binds; when it does (high DPI), the
/// downsampled blur is visually identical — the content is already blurred.
const MAX_FULL_PX: f32 = 512.0;

/// Effective corner radii: each radius clamped to half the widget's smallest
/// side, so pill shapes key correctly per size.
fn effective_radii(width: f32, height: f32, corner: CornerRadius) -> [f32; 4] {
    let half_min = (width.min(height) * 0.5).max(0.0);
    [
        (corner.nw as f32).min(half_min),
        (corner.ne as f32).min(half_min),
        (corner.sw as f32).min(half_min),
        (corner.se as f32).min(half_min),
    ]
}

/// Whether the nine-slice draw would show a seam for this widget: the corner
/// patches are cropped to half the expanded rect, so the seam between a
/// corner patch and its edge strip sits `seam_inset` points inside the widget
/// edge. The corner content there is the two-dimensional blur, the strip
/// content the one-dimensional profile; the two agree only once the corner's
/// cross-edge falloff has settled (3σ). Below that, the strip reads as a
/// visible step — rasterize the whole rect instead (see module docs).
fn needs_full_rect(width: f32, height: f32, radii: [f32; 4], layers: &[ShadowLayer]) -> bool {
    let extent = shadow_extent(layers);
    let radius_max = radii.iter().cloned().fold(0.0f32, f32::max);
    let patch = extent + corner_reach(layers, radius_max);
    let p_draw = patch
        .min((width + 2.0 * extent) * 0.5)
        .min((height + 2.0 * extent) * 0.5)
        .max(0.0);
    let seam_inset = p_draw - extent;
    let sigma_max = layers
        .iter()
        .map(|l| (l.blur * 0.5).max(0.0))
        .fold(0.0f32, f32::max);
    seam_inset < BLUR_SIGMA_EXTENT * sigma_max
}

/// Raster scale and pixel dims for a full-rect texture over the expanded
/// rect. Shared by the cache key and the rasterizer so they always agree.
fn full_rect_dims(
    width: f32,
    height: f32,
    extent: f32,
    pixels_per_point: f32,
) -> (usize, usize, f32) {
    let ew = width + 2.0 * extent;
    let eh = height + 2.0 * extent;
    let scale = pixels_per_point.min(MAX_FULL_PX / ew.max(eh)).max(0.05);
    (
        ((ew * scale).ceil() as usize).max(1),
        ((eh * scale).ceil() as usize).max(1),
        scale,
    )
}

/// Rasterize the whole expanded rect as one texture: the true two-dimensional
/// blur of the layer stack over the widget's rounded rect. Used only when
/// [`needs_full_rect`] says the nine-slice seam would show. Returns the image
/// and the extent (the draw quad is the widget rect expanded by it).
fn rasterize_full_rect(
    width: f32,
    height: f32,
    radii: [f32; 4],
    layers: &[ShadowLayer],
    dark: bool,
    pixels_per_point: f32,
) -> (ColorImage, f32) {
    let extent = shadow_extent(layers);
    let (w, h, scale) = full_rect_dims(width, height, extent, pixels_per_point);
    let mut acc = vec![0.0f32; w * h];
    for layer in layers {
        let center = Vec2::new(
            width * 0.5 + layer.offset[0],
            height * 0.5 + layer.offset[1],
        );
        let half = Vec2::new(width * 0.5 + layer.spread, height * 0.5 + layer.spread);
        let r = [
            (radii[0] + layer.spread).max(0.0),
            (radii[1] + layer.spread).max(0.0),
            (radii[2] + layer.spread).max(0.0),
            (radii[3] + layer.spread).max(0.0),
        ];
        let mut buf = vec![0.0f32; w * h];
        for j in 0..h {
            for i in 0..w {
                let p = Vec2::new(
                    (i as f32 + 0.5) / scale - extent,
                    (j as f32 + 0.5) / scale - extent,
                );
                let sdf = sd_round_rect(p, center, half, r);
                buf[j * w + i] = (0.5 - sdf * scale).clamp(0.0, 1.0);
            }
        }
        gaussian_blur(&mut buf, w, h, (layer.blur * 0.5).max(0.0) * scale);
        let a = layer.resolved_alpha(dark);
        for (dst, src) in acc.iter_mut().zip(buf.iter()) {
            *dst += a * src * (1.0 - *dst);
        }
    }
    // Interior mask (SEL-VIS-01): zero accumulated alpha strictly inside the
    // unoffset widget rect. Applied after the blur, so the exterior profile
    // is exactly the unmasked one.
    let widget_center = Vec2::new(width * 0.5, height * 0.5);
    let widget_half = Vec2::new(width * 0.5, height * 0.5);
    for j in 0..h {
        for i in 0..w {
            let p = Vec2::new(
                (i as f32 + 0.5) / scale - extent,
                (j as f32 + 0.5) / scale - extent,
            );
            if sd_round_rect(p, widget_center, widget_half, radii) < 0.0 {
                acc[j * w + i] = 0.0;
            }
        }
    }
    let pixels = acc
        .iter()
        .map(|&a| Color32::from_black_alpha((a * 255.0).round().clamp(0.0, 255.0) as u8))
        .collect();
    (ColorImage::new([w, h], pixels), extent)
}

// ---------------------------------------------------------------------------
// Texture cache (LRU, bounded, stored in `Context` temp data).
// ---------------------------------------------------------------------------

/// Cache key: everything that affects the rasterized pixels. Radii are the
/// EFFECTIVE radii (clamped to half the widget's smallest side), so pill
/// shapes key correctly per size; ordinary radii are size-independent.
/// `full_size` is [0, 0] for atlas entries and the expanded-rect pixel dims
/// for full-rect entries (which cannot be shared across widget sizes).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ShadowCacheKey {
    layers_hash: u64,
    radii: [u32; 4],
    dark: bool,
    pixels_per_point: u32,
    full_size: [u32; 2],
}

fn layers_hash(layers: &[ShadowLayer]) -> u64 {
    let mut h = DefaultHasher::new();
    layers.len().hash(&mut h);
    for l in layers {
        l.offset[0].to_bits().hash(&mut h);
        l.offset[1].to_bits().hash(&mut h);
        l.blur.to_bits().hash(&mut h);
        l.spread.to_bits().hash(&mut h);
        l.alpha.to_bits().hash(&mut h);
    }
    h.finish()
}

/// How a cached texture is drawn: either the nine-slice atlas or a single
/// full-rect quad over the widget rect expanded by `extent`.
#[derive(Clone, Copy, Debug)]
enum ShadowGeom {
    Atlas(AtlasGeom),
    FullRect { extent: f32 },
}

/// One cached shadow texture: the handle (kept alive by the cache; dropping
/// it frees the GPU texture) plus its draw geometry.
#[derive(Clone)]
struct CacheEntry {
    handle: TextureHandle,
    geom: ShadowGeom,
}

/// LRU texture cache. Stored in `Context` temp data via
/// [`Context::data_mut`]; never serialized. `Clone` is required by the
/// `IdTypeMap` API (only `&mut` access is used on the hot path, so no clone
/// ever happens per frame).
#[derive(Clone)]
struct ShadowTextureCache {
    entries: HashMap<ShadowCacheKey, CacheEntry>,
    /// Least-recently-used first; every lookup/insert moves the key to the back.
    order: VecDeque<ShadowCacheKey>,
    max_entries: usize,
}

impl Default for ShadowTextureCache {
    fn default() -> Self {
        Self::new(MAX_CACHE_ENTRIES)
    }
}

impl ShadowTextureCache {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            max_entries: max_entries.max(1),
        }
    }

    fn lookup(&mut self, key: ShadowCacheKey) -> Option<CacheEntry> {
        let entry = self.entries.get(&key)?.clone();
        self.touch(key);
        Some(entry)
    }

    fn insert(&mut self, key: ShadowCacheKey, entry: CacheEntry) {
        if self.entries.insert(key, entry).is_some() {
            self.touch(key);
            return;
        }
        // Evict least-recently-used entries until the new one fits. The new
        // key is not in `order` yet, so it can never evict itself.
        while self.entries.len() > self.max_entries {
            if let Some(old) = self.order.pop_front() {
                // Dropping the handle frees the GPU texture.
                self.entries.remove(&old);
            } else {
                break;
            }
        }
        self.order.push_back(key);
    }

    fn touch(&mut self, key: ShadowCacheKey) {
        if let Some(pos) = self.order.iter().position(|k| *k == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }

    /// (atlas entries, full-rect entries) — lets tests assert which path a
    /// paint took.
    #[cfg(test)]
    fn counts(&self) -> (usize, usize) {
        let mut atlas = 0;
        let mut full = 0;
        for entry in self.entries.values() {
            match entry.geom {
                ShadowGeom::Atlas(_) => atlas += 1,
                ShadowGeom::FullRect { .. } => full += 1,
            }
        }
        (atlas, full)
    }
}

fn cache_id() -> egui::Id {
    egui::Id::unique("egui_shadcn_shadow_texture_cache")
}

fn cache_key(
    layers: &[ShadowLayer],
    radii: [f32; 4],
    dark: bool,
    pixels_per_point: f32,
    full_size: [u32; 2],
) -> ShadowCacheKey {
    ShadowCacheKey {
        layers_hash: layers_hash(layers),
        radii: [
            radii[0].to_bits(),
            radii[1].to_bits(),
            radii[2].to_bits(),
            radii[3].to_bits(),
        ],
        dark,
        pixels_per_point: pixels_per_point.to_bits(),
        full_size,
    }
}

/// Look up (or rasterize + upload) the texture for one key. Lock discipline:
/// the `Context` data lock is held only for the map lookup/insert;
/// rasterization and texture upload happen outside it (`load_texture` takes
/// its own locks and must never run inside `data_mut`).
fn cached_texture(
    ctx: &Context,
    key: ShadowCacheKey,
    make: impl FnOnce() -> (ColorImage, ShadowGeom),
) -> CacheEntry {
    if let Some(entry) = ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<ShadowTextureCache>(cache_id())
            .lookup(key)
    }) {
        return entry;
    }
    let (image, geom) = make();
    let handle = ctx.load_texture("egui-shadcn shadow", image, TextureOptions::LINEAR);
    let entry = CacheEntry { handle, geom };
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<ShadowTextureCache>(cache_id())
            .insert(key, entry.clone());
    });
    entry
}

/// Look up (or rasterize + upload) the atlas for one paint. Returns `None`
/// only for empty layer lists.
fn shadow_texture(
    ctx: &Context,
    width: f32,
    height: f32,
    corner: CornerRadius,
    layers: &[ShadowLayer],
    dark: bool,
) -> Option<CacheEntry> {
    if layers.is_empty() {
        return None;
    }
    let ppp = ctx.pixels_per_point();
    let radii = effective_radii(width, height, corner);
    let key = cache_key(layers, radii, dark, ppp, [0, 0]);
    Some(cached_texture(ctx, key, || {
        let (image, geom) = rasterize_atlas(radii, layers, dark, ppp);
        (image, ShadowGeom::Atlas(geom))
    }))
}

/// Look up (or rasterize + upload) the full-rect texture for one paint: the
/// fallback for small widgets with large blurs (see [`needs_full_rect`]).
/// Returns `None` only for empty layer lists.
fn shadow_texture_full(
    ctx: &Context,
    width: f32,
    height: f32,
    corner: CornerRadius,
    layers: &[ShadowLayer],
    dark: bool,
) -> Option<CacheEntry> {
    if layers.is_empty() {
        return None;
    }
    let ppp = ctx.pixels_per_point();
    let radii = effective_radii(width, height, corner);
    let extent = shadow_extent(layers);
    let (w, h, _) = full_rect_dims(width, height, extent, ppp);
    let key = cache_key(layers, radii, dark, ppp, [w as u32, h as u32]);
    Some(cached_texture(ctx, key, || {
        let (image, extent) = rasterize_full_rect(width, height, radii, layers, dark, ppp);
        (image, ShadowGeom::FullRect { extent })
    }))
}

// ---------------------------------------------------------------------------
// Painting.
// ---------------------------------------------------------------------------

/// Paint a shadow behind `rect` (call BEFORE painting the widget body).
/// `corner` is the widget's corner radius, `size` the Tailwind size,
/// `dark` the theme mode. `ShadowSize::None` paints nothing and touches no
/// cache. No allocation on cache hits; on a miss the atlas is rasterized
/// once and reused for every later paint with the same key.
pub fn paint_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    size: ShadowSize,
    dark: bool,
) {
    paint_shadow_layers(painter, rect, corner, size.layers(), dark);
}

/// Paint an arbitrary layer list (escape hatch for shadows outside the
/// Tailwind scale). Empty lists paint nothing. Small widgets with large
/// blurs take the full-rect fallback (see [`needs_full_rect`]); everything
/// else draws the size-independent atlas.
pub fn paint_shadow_layers(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    layers: &[ShadowLayer],
    dark: bool,
) {
    if layers.is_empty() || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let ctx = painter.ctx();
    let radii = effective_radii(rect.width(), rect.height(), corner);
    let entry = if needs_full_rect(rect.width(), rect.height(), radii, layers) {
        shadow_texture_full(ctx, rect.width(), rect.height(), corner, layers, dark)
    } else {
        shadow_texture(ctx, rect.width(), rect.height(), corner, layers, dark)
    };
    let Some(entry) = entry else {
        return;
    };
    draw_shadow(painter, rect, &entry);
}

/// Resolve the LAYER rule for one slot and paint the result. This is the
/// call waves use: pass the base recipe's shadow and let the active style
/// override it only where its rule explicitly declares a shadow.
pub fn paint_shadow_resolved(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    base: ShadowSize,
    style: Style,
    slot: ShadowSlot,
    dark: bool,
) {
    paint_shadow(
        painter,
        rect,
        corner,
        resolve_shadow(base, style, slot),
        dark,
    );
}

/// Draw one cached shadow texture. Atlas entries draw as nine quads;
/// full-rect entries draw as one quad over the expanded rect. Neither paints
/// anything strictly inside the widget rect (SEL-VIS-01).
fn draw_shadow(painter: &Painter, rect: Rect, entry: &CacheEntry) {
    match entry.geom {
        ShadowGeom::FullRect { extent } => {
            painter.image(
                entry.handle.id(),
                rect.expand(extent),
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        ShadowGeom::Atlas(geom) => draw_shadow_atlas(painter, rect, entry, geom),
    }
}

/// Draw the nine quads for one cached atlas. `patch_draw` is
/// the patch size clamped to half the expanded rect: for widgets narrower
/// than twice the corner influence, the corner patches are cropped to their
/// outer fraction (UV sub-rects) instead of overlapping. In the normal case
/// `patch_draw == patch` and the draw is exact.
///
/// No center fill is painted: the center region always lies strictly inside
/// the widget rect — `patch_draw >= extent` always holds (`patch >= extent`
/// and each `(side + 2*extent)/2 >= extent` for non-degenerate rects, and
/// degenerate rects never reach this function) — and the shadow contributes
/// nothing inside the widget rect, so the center region is never painted
/// (SEL-VIS-01).
fn draw_shadow_atlas(painter: &Painter, rect: Rect, entry: &CacheEntry, geom: AtlasGeom) {
    let e = geom.extent;
    let p = geom.patch;
    let p_draw = p
        .min((rect.width() + 2.0 * e) * 0.5)
        .min((rect.height() + 2.0 * e) * 0.5)
        .max(0.0);
    if p_draw <= 0.0 || p <= 0.0 {
        return;
    }
    let f = p_draw / p;
    let id = entry.handle.id();
    let x0 = rect.left() - e;
    let y0 = rect.top() - e;
    let x1 = rect.right() + e;
    let y1 = rect.bottom() + e;
    // Sub-rect of a region UV: the outer `f` fraction along each axis that is
    // `true` (start = outer on that axis), the full range where `false`.
    // Corners crop on both axes; edge strips crop only along their varying
    // axis (their content is constant along the other).
    let sub = |uv: Rect, u_start: bool, u_end: bool, v_start: bool, v_end: bool| {
        let (umin, umax) = match (u_start, u_end) {
            (true, false) => (uv.min.x, uv.min.x + f * uv.width()),
            (false, true) => (uv.max.x - f * uv.width(), uv.max.x),
            _ => (uv.min.x, uv.max.x),
        };
        let (vmin, vmax) = match (v_start, v_end) {
            (true, false) => (uv.min.y, uv.min.y + f * uv.height()),
            (false, true) => (uv.max.y - f * uv.height(), uv.max.y),
            _ => (uv.min.y, uv.max.y),
        };
        Rect::from_min_max(egui::pos2(umin, vmin), egui::pos2(umax, vmax))
    };
    let img = |r: Rect, uv: Rect| {
        if r.width() > 0.0 && r.height() > 0.0 {
            painter.image(id, r, uv, Color32::WHITE);
        }
    };
    // Corners.
    img(
        Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x0 + p_draw, y0 + p_draw)),
        sub(geom.uvs[0], true, false, true, false),
    );
    img(
        Rect::from_min_max(egui::pos2(x1 - p_draw, y0), egui::pos2(x1, y0 + p_draw)),
        sub(geom.uvs[1], false, true, true, false),
    );
    img(
        Rect::from_min_max(egui::pos2(x0, y1 - p_draw), egui::pos2(x0 + p_draw, y1)),
        sub(geom.uvs[2], true, false, false, true),
    );
    img(
        Rect::from_min_max(egui::pos2(x1 - p_draw, y1 - p_draw), egui::pos2(x1, y1)),
        sub(geom.uvs[3], false, true, false, true),
    );
    // Edges (skipped when the middle span is empty).
    img(
        Rect::from_min_max(
            egui::pos2(x0 + p_draw, y0),
            egui::pos2(x1 - p_draw, y0 + p_draw),
        ),
        sub(geom.uvs[4], false, false, true, false),
    );
    img(
        Rect::from_min_max(
            egui::pos2(x0 + p_draw, y1 - p_draw),
            egui::pos2(x1 - p_draw, y1),
        ),
        sub(geom.uvs[5], false, false, false, true),
    );
    img(
        Rect::from_min_max(
            egui::pos2(x0, y0 + p_draw),
            egui::pos2(x0 + p_draw, y1 - p_draw),
        ),
        sub(geom.uvs[6], true, false, false, false),
    );
    img(
        Rect::from_min_max(
            egui::pos2(x1 - p_draw, y0 + p_draw),
            egui::pos2(x1, y1 - p_draw),
        ),
        sub(geom.uvs[7], false, true, false, false),
    );
    // No center fill: the center region is interior-only (see the function
    // docs) and the shadow contributes nothing inside the widget rect.
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, LayerId};

    fn layer(offset: [f32; 2], blur: f32, spread: f32, alpha: f32) -> ShadowLayer {
        ShadowLayer {
            offset,
            blur,
            spread,
            alpha,
        }
    }

    // --- Value table ---

    #[test]
    fn value_table_matches_tailwind_v4_defaults() {
        assert_eq!(
            ShadowSize::Xs.layers(),
            &[layer([0.0, 1.0], 2.0, 0.0, 0.05)]
        );
        assert_eq!(
            ShadowSize::Sm.layers(),
            &[
                layer([0.0, 1.0], 3.0, 0.0, 0.10),
                layer([0.0, 1.0], 2.0, -1.0, 0.10),
            ]
        );
        assert_eq!(
            ShadowSize::Md.layers(),
            &[
                layer([0.0, 4.0], 6.0, -1.0, 0.10),
                layer([0.0, 2.0], 4.0, -2.0, 0.10),
            ]
        );
        assert_eq!(
            ShadowSize::Lg.layers(),
            &[
                layer([0.0, 10.0], 15.0, -3.0, 0.10),
                layer([0.0, 4.0], 6.0, -4.0, 0.10),
            ]
        );
        assert_eq!(
            ShadowSize::Xl.layers(),
            &[
                layer([0.0, 20.0], 25.0, -5.0, 0.10),
                layer([0.0, 8.0], 10.0, -6.0, 0.10),
            ]
        );
        assert_eq!(
            ShadowSize::Xxl.layers(),
            &[layer([0.0, 25.0], 50.0, -12.0, 0.25)]
        );
        assert!(ShadowSize::None.layers().is_empty());
        assert!(ShadowSize::None.is_none());
        assert!(!ShadowSize::Xs.is_none());
    }

    #[test]
    fn dark_boost_multiplies_alpha_and_clamps() {
        let l = layer([0.0, 1.0], 2.0, 0.0, 0.05);
        assert_eq!(l.resolved_alpha(false), 0.05);
        assert!((l.resolved_alpha(true) - 0.08).abs() < 1e-6);
        let strong = layer([0.0, 1.0], 2.0, 0.0, 0.9);
        assert_eq!(strong.resolved_alpha(true), 1.0);
    }

    #[test]
    fn legacy_helpers_unchanged() {
        let xs = shadow_xs(false);
        assert_eq!(xs.offset, [0, 1]);
        assert_eq!(xs.blur, 2);
        assert_eq!(xs.spread, 0);
        assert_eq!(xs.color, Color32::from_black_alpha(50));
        assert_eq!(shadow_xs(true).color, Color32::from_black_alpha(80));
        assert_eq!(shadow_sm(false).color, Color32::from_black_alpha(70));
        assert_eq!(shadow_md(false).color, Color32::from_black_alpha(80));
        assert_eq!(shadow_md(false).offset, [0, 3]);
        assert_eq!(shadow_lg(false).offset, [0, 8]);
        assert_eq!(shadow_lg(false).blur, 24);
        let xl = shadow_xl(false);
        assert_eq!(xl.offset, [0, 12]);
        assert_eq!(xl.blur, 32);
    }

    // --- LAYER resolution ---

    const ALL_STYLES: [Style; 8] = [
        Style::Nova,
        Style::Vega,
        Style::Maia,
        Style::Lyra,
        Style::Mira,
        Style::Luma,
        Style::Sera,
        Style::Rhea,
    ];

    #[test]
    fn nova_silence_keeps_base_shadow() {
        // The §5(b) ruling case: nova's outline rule never mentions shadow.
        assert_eq!(style_override(Style::Nova, ShadowSlot::ButtonOutline), None);
        assert_eq!(
            resolve_shadow(ShadowSize::Xs, Style::Nova, ShadowSlot::ButtonOutline),
            ShadowSize::Xs
        );
        assert_eq!(style_override(Style::Nova, ShadowSlot::Checkbox), None);
        assert_eq!(style_override(Style::Nova, ShadowSlot::Input), None);
        assert_eq!(style_override(Style::Nova, ShadowSlot::DialogContent), None);
        assert_eq!(
            resolve_shadow(ShadowSize::Md, Style::Nova, ShadowSlot::DialogContent),
            ShadowSize::Md
        );
    }

    #[test]
    fn explicit_override_wins_over_base() {
        assert_eq!(
            style_override(Style::Maia, ShadowSlot::PopoverContent),
            Some(ShadowSize::Xxl)
        );
        assert_eq!(
            resolve_shadow(ShadowSize::Sm, Style::Maia, ShadowSlot::PopoverContent),
            ShadowSize::Xxl
        );
        assert_eq!(
            style_override(Style::Luma, ShadowSlot::DialogContent),
            Some(ShadowSize::Xl)
        );
        assert_eq!(
            style_override(Style::Sera, ShadowSlot::SheetContent),
            Some(ShadowSize::Md)
        );
        assert_eq!(
            style_override(Style::Vega, ShadowSlot::ButtonOutline),
            Some(ShadowSize::Xs)
        );
    }

    #[test]
    fn shadow_none_overrides_remove_the_shadow() {
        assert_eq!(
            style_override(Style::Nova, ShadowSlot::InputGroupInCommand),
            Some(ShadowSize::None)
        );
        assert_eq!(
            resolve_shadow(ShadowSize::Xs, Style::Nova, ShadowSlot::InputGroupInCommand),
            ShadowSize::None
        );
        assert_eq!(
            style_override(Style::Lyra, ShadowSlot::InputGroupInCommand),
            Some(ShadowSize::None)
        );
        // Other styles are silent on the command input-group: base survives.
        assert_eq!(
            style_override(Style::Maia, ShadowSlot::InputGroupInCommand),
            None
        );
        assert_eq!(
            style_override(Style::Nova, ShadowSlot::InputGroupInCombobox),
            Some(ShadowSize::None)
        );
        assert_eq!(
            style_override(Style::Sera, ShadowSlot::InputGroupInCombobox),
            None
        );
    }

    #[test]
    fn tabs_trigger_conditionals() {
        assert_eq!(
            style_override(Style::Nova, ShadowSlot::TabsTriggerDefault),
            Some(ShadowSize::Sm)
        );
        assert_eq!(
            style_override(Style::Nova, ShadowSlot::TabsTriggerLine),
            Some(ShadowSize::None)
        );
        assert_eq!(
            style_override(Style::Vega, ShadowSlot::TabsTriggerLine),
            Some(ShadowSize::None)
        );
        assert_eq!(
            style_override(Style::Lyra, ShadowSlot::TabsTriggerDefault),
            None
        );
    }

    #[test]
    fn conditional_slots_record_their_conditions() {
        // Navigation-menu content: only the no-viewport case is declared.
        assert_eq!(
            style_override(Style::Luma, ShadowSlot::NavigationMenuContent),
            Some(ShadowSize::Lg)
        );
        assert_eq!(
            style_override(Style::Nova, ShadowSlot::NavigationMenuContent),
            None
        );
        // Sidebar inner/inset: floating / inset variants only, all styles.
        for style in ALL_STYLES {
            assert_eq!(
                style_override(style, ShadowSlot::SidebarInner),
                Some(ShadowSize::Sm),
                "{style:?} sidebar-inner"
            );
            assert_eq!(
                style_override(style, ShadowSlot::SidebarInset),
                Some(ShadowSize::Sm),
                "{style:?} sidebar-inset"
            );
        }
        // Single-style silences inside otherwise-declaring slots.
        assert_eq!(
            style_override(Style::Sera, ShadowSlot::InputGroupInput),
            None
        );
        assert_eq!(style_override(Style::Mira, ShadowSlot::SidebarInput), None);
    }

    #[test]
    fn subcontent_spellings_are_distinct_slots() {
        // maia/mira prove the hyphenated and unhyphenated selectors differ.
        assert_eq!(
            style_override(Style::Maia, ShadowSlot::DropdownMenuSubContent),
            Some(ShadowSize::Xxl)
        );
        assert_eq!(
            style_override(Style::Maia, ShadowSlot::DropdownMenuSubcontent),
            Some(ShadowSize::Lg)
        );
        assert_eq!(
            style_override(Style::Mira, ShadowSlot::ContextMenuSubContent),
            Some(ShadowSize::Md)
        );
        assert_eq!(
            style_override(Style::Mira, ShadowSlot::ContextMenuSubcontent),
            Some(ShadowSize::Lg)
        );
    }

    #[test]
    fn verified_silent_slots_always_return_base() {
        for style in ALL_STYLES {
            for slot in [
                ShadowSlot::RadioItem,
                ShadowSlot::CarouselArrow,
                ShadowSlot::DatePickerTrigger,
                ShadowSlot::ComboboxTrigger,
            ] {
                assert_eq!(style_override(style, slot), None, "{style:?} {slot:?}");
                assert_eq!(resolve_shadow(ShadowSize::Xs, style, slot), ShadowSize::Xs);
            }
        }
    }

    #[test]
    fn override_table_entry_count() {
        // Pins the mechanically extracted table: 223 (style, slot) pairs.
        // Any change to the snapshot extraction must update this count
        // deliberately, with the new citations in the table.
        let slots = [
            ShadowSlot::AlertDialogContent,
            ShadowSlot::ButtonGroupText,
            ShadowSlot::ButtonOutline,
            ShadowSlot::CalendarDropdownRoot,
            ShadowSlot::ChartTooltip,
            ShadowSlot::Checkbox,
            ShadowSlot::ComboboxChips,
            ShadowSlot::ComboboxContent,
            ShadowSlot::ComboboxTrigger,
            ShadowSlot::ContextMenuContent,
            ShadowSlot::ContextMenuSubContent,
            ShadowSlot::ContextMenuSubcontent,
            ShadowSlot::DialogContent,
            ShadowSlot::DrawerContent,
            ShadowSlot::DrawerPopup,
            ShadowSlot::DropdownMenuContent,
            ShadowSlot::DropdownMenuSubContent,
            ShadowSlot::DropdownMenuSubcontent,
            ShadowSlot::HoverCardContent,
            ShadowSlot::Input,
            ShadowSlot::InputGroup,
            ShadowSlot::InputGroupInCombobox,
            ShadowSlot::InputGroupInCommand,
            ShadowSlot::InputGroupInput,
            ShadowSlot::InputGroupTextarea,
            ShadowSlot::InputOtpSlot,
            ShadowSlot::Menubar,
            ShadowSlot::MenubarContent,
            ShadowSlot::MenubarSubContent,
            ShadowSlot::NativeSelect,
            ShadowSlot::NavigationMenuContent,
            ShadowSlot::NavigationMenuIndicatorArrow,
            ShadowSlot::NavigationMenuPopup,
            ShadowSlot::NavigationMenuViewport,
            ShadowSlot::PopoverContent,
            ShadowSlot::QuestionnaireChoice,
            ShadowSlot::QuestionnaireInput,
            ShadowSlot::QuestionnaireShortcut,
            ShadowSlot::RadioItem,
            ShadowSlot::SelectContent,
            ShadowSlot::SelectTrigger,
            ShadowSlot::SheetContent,
            ShadowSlot::SidebarInner,
            ShadowSlot::SidebarInput,
            ShadowSlot::SidebarInset,
            ShadowSlot::SliderThumb,
            ShadowSlot::Switch,
            ShadowSlot::SwitchThumb,
            ShadowSlot::TabsTriggerDefault,
            ShadowSlot::TabsTriggerLine,
            ShadowSlot::Textarea,
            ShadowSlot::ToggleGroup,
            ShadowSlot::ToggleGroupItem,
            ShadowSlot::ToggleOutline,
            ShadowSlot::CarouselArrow,
            ShadowSlot::DatePickerTrigger,
        ];
        assert_eq!(slots.len(), 56);
        let mut count = 0;
        for style in ALL_STYLES {
            for slot in slots {
                if style_override(style, slot).is_some() {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 223);
    }

    // --- Geometry / rasterizer ---

    #[test]
    fn sd_round_box_matches_known_points() {
        let center = Vec2::ZERO;
        let half = Vec2::splat(10.0);
        let r = 4.0;
        // Center: 10pt from the nearest edge.
        assert!((sd_round_box(Vec2::ZERO, center, half, r) - (-10.0)).abs() < 1e-4);
        // Edge midpoint: on the boundary.
        assert!((sd_round_box(Vec2::new(10.0, 0.0), center, half, r)).abs() < 1e-4);
        // 2pt outside the edge.
        assert!((sd_round_box(Vec2::new(12.0, 0.0), center, half, r) - 2.0).abs() < 1e-4);
        // On the arc (45° from the arc center at (6, 6)).
        let a = 6.0 + 4.0 / std::f32::consts::SQRT_2;
        assert!(sd_round_box(Vec2::new(a, a), center, half, r).abs() < 1e-3);
        // Deep outside the corner: distance to the arc.
        let d = sd_round_box(Vec2::new(14.0, 14.0), center, half, r);
        let expected = (8.0f32.powi(2) * 2.0).sqrt() - 4.0;
        assert!((d - expected).abs() < 1e-3);
    }

    #[test]
    fn extent_and_reach_values() {
        assert_eq!(shadow_extent(ShadowSize::Xs.layers()), 4.0);
        assert_eq!(shadow_extent(ShadowSize::Md.layers()), 12.0);
        assert_eq!(shadow_extent(ShadowSize::Xxl.layers()), 88.0);
        assert_eq!(corner_reach(ShadowSize::Xs.layers(), 6.0), 10.0);
        assert_eq!(corner_reach(ShadowSize::Xxl.layers(), 10.0), 112.0);
    }

    #[test]
    fn atlas_dimensions_follow_geometry() {
        let (image, geom) = rasterize_atlas([6.0; 4], ShadowSize::Xs.layers(), false, 1.0);
        assert_eq!(geom.extent, 4.0);
        assert_eq!(geom.patch, 14.0);
        // 6 patches of 14px + 2 strips of 4px wide.
        assert_eq!(image.size, [6 * 14 + 8, 14]);
        assert_eq!(image.pixels.len(), (6 * 14 + 8) * 14);
    }

    fn alpha_at(image: &ColorImage, x: usize, y: usize) -> u8 {
        image.pixels[y * image.size[0] + x].a()
    }

    #[test]
    fn corner_patch_falls_off_monotonically_outside_and_transparent_inside() {
        let (image, _) = rasterize_atlas([6.0; 4], ShadowSize::Xs.layers(), false, 1.0);
        // Far outside corner: fully transparent.
        assert_eq!(alpha_at(&image, 0, 0), 0);
        // Deep inside the widget rect: the shadow contributes nothing
        // (SEL-VIS-01 contract).
        assert_eq!(alpha_at(&image, 13, 13), 0);
        // Down the middle column of the NW patch: monotone non-decreasing
        // falloff from outside up to the widget edge, then transparent
        // inside. Column x=7 crosses the rounded corner (radius 6) between
        // rows 4 and 5 (SDF +0.04 at row 4, -0.85 at row 5).
        let mut prev = 0u8;
        for y in 0..5 {
            let a = alpha_at(&image, 7, y);
            assert!(a >= prev, "row {y}: {a} < {prev}");
            prev = a;
        }
        assert_eq!(prev, 2, "exterior edge texel keeps its pre-mask value");
        for y in 5..14 {
            assert_eq!(alpha_at(&image, 7, y), 0, "row {y} must be masked");
        }
    }

    #[test]
    fn multi_layer_interior_is_transparent_exterior_keeps_profile() {
        // shadow-sm used to composite both layers at the inner corner texel
        // (1 - (1-0.1)(1-0.1) = 0.19 -> 48). The SEL-VIS-01 contract masks
        // the interior: the shadow contributes nothing inside the widget
        // rect.
        let (image, _) = rasterize_atlas([6.0; 4], ShadowSize::Sm.layers(), false, 1.0);
        let n = image.size[1];
        assert_eq!(alpha_at(&image, n - 1, n - 1), 0);
        // Far outside corner: still transparent.
        assert_eq!(alpha_at(&image, 0, 0), 0);
        // The exterior falloff is pinned exactly by
        // `edge_profile_interior_masked_exterior_pinned`.
    }

    #[test]
    fn edge_profile_interior_masked_exterior_pinned() {
        // The exterior falloff (t <= 0) is pinned exactly from the pre-mask
        // rasterization; the interior (t > 0) is zero for every size. The
        // mask is applied after the blur, so exterior values are identical
        // to the unmasked computation (SEL-VIS-01).
        let cases = [
            (ShadowSize::Xs, vec![0u8, 0, 0, 1]),
            (ShadowSize::Sm, vec![0, 0, 0, 1, 3, 7]),
            (ShadowSize::Md, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
            (
                ShadowSize::Lg,
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 1, 1, 1,
                ],
            ),
        ];
        for (size, exterior) in cases {
            let layers = size.layers();
            let extent = shadow_extent(layers);
            let patch = extent + corner_reach(layers, 6.0);
            let n = patch.ceil() as usize;
            let profile = edge_profile(n, 1.0, extent, layers, false, 1, 1.0);
            let rounded: Vec<u8> = profile
                .iter()
                .map(|&a| (a * 255.0).round().clamp(0.0, 255.0) as u8)
                .collect();
            assert_eq!(
                &rounded[..exterior.len()],
                &exterior[..],
                "{size:?} exterior"
            );
            assert!(
                rounded[exterior.len()..].iter().all(|&a| a == 0),
                "{size:?} interior must be masked: {rounded:?}"
            );
        }
    }

    #[test]
    fn shadow_contributes_nothing_inside_widget_rect() {
        // Texture-level contract (SEL-VIS-01): every texel strictly inside
        // the widget rect is transparent, in all three rasterizers, for
        // every multi-layer size, in both modes. "Strictly inside" is the
        // same SDF the mask uses (independently tested by
        // `sd_round_box_matches_known_points`).
        for size in [
            ShadowSize::Xs,
            ShadowSize::Sm,
            ShadowSize::Md,
            ShadowSize::Lg,
        ] {
            let layers = size.layers();
            let extent = shadow_extent(layers);
            let patch = extent + corner_reach(layers, 6.0);
            let n = patch.ceil() as usize;
            for dark in [false, true] {
                // Corner patches, all four flips.
                for (fx, fy) in [(false, false), (true, false), (false, true), (true, true)] {
                    let buf = render_corner(n, patch, 1.0, extent, 6.0, layers, dark, fx, fy);
                    for j in 0..n {
                        for i in 0..n {
                            let mut px = i as f32 + 0.5;
                            let mut py = j as f32 + 0.5;
                            if fx {
                                px = patch - px;
                            }
                            if fy {
                                py = patch - py;
                            }
                            let sdf = sd_round_box(
                                Vec2::new(px - extent, py - extent),
                                Vec2::splat(4096.0),
                                Vec2::splat(4096.0),
                                6.0,
                            );
                            if sdf < 0.0 {
                                assert_eq!(
                                    buf[j * n + i],
                                    0.0,
                                    "{size:?} dark={dark} flip=({fx},{fy}) texel=({i},{j})"
                                );
                            }
                        }
                    }
                }
                // Edge profiles, all four orientations.
                for (axis, sign) in [(1usize, 1.0f32), (1, -1.0), (0, 1.0), (0, -1.0)] {
                    let profile = edge_profile(n, 1.0, extent, layers, dark, axis, sign);
                    for (j, &v) in profile.iter().enumerate() {
                        let t = (j as f32 + 0.5) - extent;
                        if t > 0.0 {
                            assert_eq!(
                                v, 0.0,
                                "{size:?} dark={dark} axis={axis} sign={sign} j={j}"
                            );
                        }
                    }
                }
            }
            // Full-rect rasterizer.
            let (image, _) = rasterize_full_rect(210.0, 72.0, [8.0; 4], layers, false, 1.0);
            let (w, h) = (image.size[0], image.size[1]);
            for j in 0..h {
                for i in 0..w {
                    let p = Vec2::new(i as f32 + 0.5 - extent, j as f32 + 0.5 - extent);
                    if sd_round_rect(p, Vec2::new(105.0, 36.0), Vec2::new(105.0, 36.0), [8.0; 4])
                        < 0.0
                    {
                        assert_eq!(
                            alpha_at(&image, i, j),
                            0,
                            "{size:?} full-rect texel=({i},{j})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn larger_blur_falls_off_further() {
        // The visible exterior tail — how far outside the widget edge the
        // shadow is still visible (rounded alpha >= 1) — grows with the
        // blur: shadow-xs fades within half a point of the edge, shadow-2xl
        // is still visible 21.5pt out. Measured on the exterior (t <= 0)
        // only: the interior is masked (SEL-VIS-01), so the old
        // half-of-saturated-max crossing no longer exists. Only the extremes
        // are ordered: intermediate tails depend on offset/spread as well as
        // blur (sm 2pt, md 0.5pt, lg 2pt), so they are bounded, not ordered.
        let tail = |size: ShadowSize| {
            let extent = shadow_extent(size.layers());
            let reach = corner_reach(size.layers(), 6.0);
            let patch = extent + reach;
            let n = patch.ceil() as usize;
            let profile = edge_profile(n, 1.0, extent, size.layers(), false, 1, 1.0);
            let first = profile
                .iter()
                .position(|&v| (v * 255.0).round() >= 1.0)
                .unwrap();
            -((first as f32 + 0.5) - extent)
        };
        let xs = tail(ShadowSize::Xs);
        let xxl = tail(ShadowSize::Xxl);
        assert_eq!(xs, 0.5);
        assert_eq!(xxl, 21.5);
        assert!(xs < xxl, "xs {xs} xxl {xxl}");
        for size in [ShadowSize::Sm, ShadowSize::Md, ShadowSize::Lg] {
            let t = tail(size);
            assert!(
                (xs..=xxl).contains(&t),
                "{size:?} tail {t} outside [{xs}, {xxl}]"
            );
        }
    }

    #[test]
    fn layers_hash_distinguishes_specs() {
        let a = layers_hash(ShadowSize::Xs.layers());
        let b = layers_hash(ShadowSize::Sm.layers());
        assert_ne!(a, b);
        assert_eq!(a, layers_hash(ShadowSize::Xs.layers()));
    }

    // --- Cache ---

    fn cache_len(ctx: &Context) -> usize {
        ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<ShadowTextureCache>(cache_id())
                .len()
        })
    }

    #[test]
    fn cache_same_key_same_texture() {
        let ctx = Context::default();
        let corner = CornerRadius::same(6);
        let a = shadow_texture(&ctx, 100.0, 36.0, corner, ShadowSize::Xs.layers(), false)
            .expect("atlas");
        let b = shadow_texture(&ctx, 120.0, 40.0, corner, ShadowSize::Xs.layers(), false)
            .expect("atlas");
        // Size-independent key: different widget sizes share one texture.
        assert_eq!(a.handle.id(), b.handle.id());
        assert_eq!(cache_len(&ctx), 1);
    }

    #[test]
    fn cache_different_keys_different_textures() {
        let ctx = Context::default();
        let a = shadow_texture(
            &ctx,
            100.0,
            36.0,
            CornerRadius::same(6),
            ShadowSize::Xs.layers(),
            false,
        )
        .expect("atlas");
        let b = shadow_texture(
            &ctx,
            100.0,
            36.0,
            CornerRadius::same(10),
            ShadowSize::Xs.layers(),
            false,
        )
        .expect("atlas");
        let c = shadow_texture(
            &ctx,
            100.0,
            36.0,
            CornerRadius::same(6),
            ShadowSize::Xs.layers(),
            true,
        )
        .expect("atlas");
        assert_ne!(a.handle.id(), b.handle.id());
        assert_ne!(a.handle.id(), c.handle.id());
        assert_eq!(cache_len(&ctx), 3);
    }

    #[test]
    fn cache_bound_is_enforced_with_lru_eviction() {
        let ctx = Context::default();
        // 33 distinct radii through the global cache (bound 32).
        let mut first_id = None;
        for r in 0..33u8 {
            let e = shadow_texture(
                &ctx,
                200.0,
                200.0,
                CornerRadius::same(r),
                ShadowSize::Xs.layers(),
                false,
            )
            .expect("atlas");
            if r == 0 {
                first_id = Some(e.handle.id());
            }
        }
        assert_eq!(cache_len(&ctx), MAX_CACHE_ENTRIES);
        // The first entry was evicted: requesting it again rasterizes anew
        // (a fresh texture id) and the cache stays bounded.
        let again = shadow_texture(
            &ctx,
            200.0,
            200.0,
            CornerRadius::same(0),
            ShadowSize::Xs.layers(),
            false,
        )
        .expect("atlas");
        assert_ne!(again.handle.id(), first_id.unwrap());
        assert_eq!(cache_len(&ctx), MAX_CACHE_ENTRIES);
    }

    #[test]
    fn shadow_none_allocates_nothing() {
        let ctx = Context::default();
        assert!(shadow_texture(
            &ctx,
            100.0,
            36.0,
            CornerRadius::same(6),
            ShadowSize::None.layers(),
            false
        )
        .is_none());
        assert_eq!(cache_len(&ctx), 0);
    }

    // --- Painting smoke tests ---

    fn run_pass(ctx: &Context, f: impl FnOnce()) {
        ctx.begin_pass(egui::RawInput {
            time: Some(0.0),
            ..Default::default()
        });
        f();
        let mut full = ctx.end_pass();
        full.textures_delta.clear();
    }

    #[test]
    fn paint_all_sizes_without_panicking() {
        let ctx = Context::default();
        run_pass(&ctx, || {
            let painter = ctx.layer_painter(LayerId::background());
            let rect = Rect::from_min_size(egui::pos2(10.0, 10.0), Vec2::new(200.0, 80.0));
            for size in [
                ShadowSize::Xs,
                ShadowSize::Sm,
                ShadowSize::Md,
                ShadowSize::Lg,
                ShadowSize::Xl,
                ShadowSize::Xxl,
                ShadowSize::None,
            ] {
                paint_shadow(&painter, rect, CornerRadius::same(8), size, false);
                paint_shadow(&painter, rect, CornerRadius::same(8), size, true);
            }
            // Degenerate inputs paint nothing and never panic.
            paint_shadow(
                &painter,
                Rect::NOTHING,
                CornerRadius::same(8),
                ShadowSize::Md,
                false,
            );
            paint_shadow_layers(&painter, rect, CornerRadius::same(8), &[], false);
            // Pill radius clamps to half the height; tiny widgets crop.
            paint_shadow(
                &painter,
                Rect::from_min_size(egui::pos2(10.0, 10.0), Vec2::new(24.0, 24.0)),
                CornerRadius::same(255),
                ShadowSize::Xxl,
                false,
            );
            paint_shadow_resolved(
                &painter,
                rect,
                CornerRadius::same(8),
                ShadowSize::Xs,
                Style::Nova,
                ShadowSlot::ButtonOutline,
                false,
            );
        });
        assert!(cache_len(&ctx) > 0);
    }

    // --- Full-rect fallback ---

    #[test]
    fn sd_round_rect_matches_sd_round_box_for_uniform_radii() {
        let center = Vec2::new(50.0, 40.0);
        let half = Vec2::new(30.0, 20.0);
        for p in [
            Vec2::new(50.0, 40.0),
            Vec2::new(80.0, 40.0),
            Vec2::new(50.0, 20.0),
            Vec2::new(85.0, 15.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(74.0, 56.0),
        ] {
            assert!(
                (sd_round_rect(p, center, half, [6.0; 4]) - sd_round_box(p, center, half, 6.0))
                    .abs()
                    < 1e-4
            );
        }
        // Per-corner radii: the NW arc uses the NW radius. Rect spans
        // (20, 20)–(80, 60); NW arc center is (30, 30); (20, 20) sits
        // sqrt(200) away, so the SDF is sqrt(200) - 10.
        let d = sd_round_rect(Vec2::new(20.0, 20.0), center, half, [10.0, 0.0, 0.0, 0.0]);
        assert!((d - ((200.0f32).sqrt() - 10.0)).abs() < 1e-3);
    }

    #[test]
    fn full_rect_trigger_cases() {
        let radii = [8.0; 4];
        // The demo card: 210x72 with shadow-2xl (sigma 25, seam 36pt in).
        assert!(needs_full_rect(
            210.0,
            72.0,
            radii,
            ShadowSize::Xxl.layers()
        ));
        // A dialog-sized panel: the seam sits past 3 sigma.
        assert!(!needs_full_rect(
            560.0,
            300.0,
            radii,
            ShadowSize::Xxl.layers()
        ));
        // Small blurs never trigger, even on tiny widgets.
        assert!(!needs_full_rect(24.0, 24.0, radii, ShadowSize::Xs.layers()));
        assert!(!needs_full_rect(36.0, 36.0, radii, ShadowSize::Sm.layers()));
        // shadow-lg on the demo card: seam 36pt in, 3 sigma = 22.5.
        assert!(!needs_full_rect(
            210.0,
            72.0,
            radii,
            ShadowSize::Lg.layers()
        ));
    }

    #[test]
    fn full_rect_rasterizer_is_seamless() {
        let (image, extent) =
            rasterize_full_rect(210.0, 72.0, [8.0; 4], ShadowSize::Xxl.layers(), false, 1.0);
        assert_eq!(extent, 88.0);
        assert_eq!(image.size, [386, 248]);
        // Deep inside the widget rect (shape center (105, 61) in widget
        // coords): the shadow contributes nothing (SEL-VIS-01 contract).
        // Pre-mask this texel read 42 (the blurred slab value); it is now
        // transparent.
        assert_eq!(alpha_at(&image, 193, 149), 0);
        // The brightest point sits outside the widget rect, below the card
        // (the shadow is offset downward): the exterior peak is 39, and
        // every texel attaining it lies outside the rounded rect.
        let max_alpha = image.pixels.iter().map(|c| c.a()).max().unwrap();
        assert_eq!(max_alpha, 39);
        for j in 0..image.size[1] {
            for i in 0..image.size[0] {
                if alpha_at(&image, i, j) == max_alpha {
                    let p = Vec2::new(i as f32 + 0.5 - extent, j as f32 + 0.5 - extent);
                    assert!(
                        sd_round_rect(p, Vec2::new(105.0, 36.0), Vec2::new(105.0, 36.0), [8.0; 4])
                            >= 0.0,
                        "max texel ({i},{j}) must be outside the widget rect"
                    );
                }
            }
        }
        // Far corner: transparent.
        assert_eq!(alpha_at(&image, 0, 0), 0);
        // No vertical step: the row 5pt below the card must vary smoothly
        // across the whole width. A nine-slice draw would step ~4 levels at
        // the corner/strip boundaries (image x = 124 and x = 262).
        let y = 165;
        let mut max_step = 0u8;
        for x in 40..346 {
            let step = alpha_at(&image, x, y).abs_diff(alpha_at(&image, x + 1, y));
            max_step = max_step.max(step);
        }
        assert!(max_step <= 2, "max horizontal step {max_step}");
        // The shadow is offset downward: the alpha centroid sits below the
        // image center row.
        let mut sum = 0.0f64;
        let mut wsum = 0.0f64;
        for j in 0..image.size[1] {
            for i in 0..image.size[0] {
                let a = alpha_at(&image, i, j) as f64;
                sum += a * j as f64;
                wsum += a;
            }
        }
        assert!(sum / wsum > 124.0, "centroid row {}", sum / wsum);
    }

    #[test]
    fn full_rect_cache_keys_by_size() {
        let ctx = Context::default();
        let corner = CornerRadius::same(8);
        let a = shadow_texture_full(&ctx, 210.0, 72.0, corner, ShadowSize::Xxl.layers(), false)
            .expect("full");
        let b = shadow_texture_full(&ctx, 210.0, 72.0, corner, ShadowSize::Xxl.layers(), false)
            .expect("full");
        assert_eq!(a.handle.id(), b.handle.id());
        let c = shadow_texture_full(&ctx, 220.0, 72.0, corner, ShadowSize::Xxl.layers(), false)
            .expect("full");
        assert_ne!(a.handle.id(), c.handle.id());
        assert_eq!(cache_len(&ctx), 2);
    }

    #[test]
    fn paint_routes_small_and_large_widgets_to_the_right_path() {
        let ctx = Context::default();
        run_pass(&ctx, || {
            let painter = ctx.layer_painter(LayerId::background());
            // Small card + 2xl -> full-rect.
            paint_shadow(
                &painter,
                Rect::from_min_size(egui::pos2(10.0, 10.0), Vec2::new(210.0, 72.0)),
                CornerRadius::same(8),
                ShadowSize::Xxl,
                false,
            );
            // Large panel + 2xl -> atlas.
            paint_shadow(
                &painter,
                Rect::from_min_size(egui::pos2(10.0, 100.0), Vec2::new(560.0, 300.0)),
                CornerRadius::same(8),
                ShadowSize::Xxl,
                false,
            );
            // Small widget + small shadow -> atlas.
            paint_shadow(
                &painter,
                Rect::from_min_size(egui::pos2(10.0, 420.0), Vec2::new(24.0, 24.0)),
                CornerRadius::same(6),
                ShadowSize::Xs,
                false,
            );
        });
        let (atlas, full) = ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<ShadowTextureCache>(cache_id())
                .counts()
        });
        assert_eq!(full, 1);
        assert_eq!(atlas, 2);
    }
}
