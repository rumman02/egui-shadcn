//! shadcn `field.tsx` — form field layout and typography primitives.
//!
//! Rebuilt to the frozen Batch B S0 spec (`~/workspace/batch-b/specs/
//! field-spec.md`): all ten parts, base + nova recipes under LAYER (nova
//! layers over base; silence keeps the base value). Nothing animates
//! (spec §6); field parts take no Tab stop and show no focus ring of
//! their own (spec §7.1/§7.4).
//!
//! - [`Field`] — a labeled control group (`vertical` / `horizontal` /
//!   `responsive` orientation, `gap-3` base / `gap-2` nova); `invalid`
//!   tints the label and title `text-destructive`, `disabled` dims them to
//!   50% (element-level opacity, group fade). Responsive orientation
//!   measures the nearest enclosing [`FieldGroup`]'s inline size against
//!   the `@md` breakpoint (448px, OPEN-1 framework default); with no
//!   enclosing group it always renders columnar.
//! - [`FieldLabel`] / [`FieldTitle`] — `text-sm font-medium` label styles.
//!   [`FieldLabel::show_with_card`] renders the source's choice-card
//!   composition: the label wraps a nested [`Field`], the card gains
//!   `rounded-md border p-4` (base) / `rounded-lg border p-2.5` (nova), a
//!   nova-only `bg-muted/50` hover fill, a `bg-primary/5` fill with a
//!   primary-tinted border while the wrapped control is checked, and —
//!   under nova only, when a descendant holds focus (N9) — a border
//!   recolored to `ring` plus a 3px `ring/50` halo. Clicking a bound
//!   label forwards activation + focus to the target control (the
//!   settled label rule, spec §7.2).
//! - [`FieldDescription`] — `text-sm text-muted-foreground` helper text
//!   (21px line box, `leading-normal`), with the three frozen top-margin
//!   rules (§2.8, OPEN-5 legend-follow-wins).
//! - [`FieldError`] — `text-sm text-destructive`; renders nothing when
//!   empty, one message as bare text, or a bulleted list after the
//!   frozen dedup algorithm (N13).
//! - [`FieldContent`] — stacks description/error under each other
//!   (`gap-1.5` base / `gap-0.5` nova) and takes the remaining row space
//!   (`flex-1`).
//! - [`FieldGroup`] — stacks fields (`gap-7` base / `gap-5` nova; 16px
//!   when directly nested in another group, 12px when carrying the
//!   checkbox-group slot — N14), [`FieldSet`] — a form section (`gap-6`
//!   base / `gap-4` nova; 12px with a direct checkbox/radio-group child
//!   — N14), [`FieldLegend`] — a section heading (`mb-3` base / `mb-1.5`
//!   nova), and [`FieldSeparator`] — a rule with optional centered text
//!   (`h-5`, `-my-2` overlap).
//!
//! **Field-label focus contract (§4.5, coordinator decision S1).** In a
//! nova card composition the CARD ring is the visible focus indicator,
//! not the nested control's own ring: the nova checkbox/radio/switch
//! rules suppress the control's ring inside a field-label group. The
//! composing code wires this by constructing each card-nested control
//! with `.suppress_focus_indication(true)` (switch D4, radio-group D6,
//! and checkbox once the checkbox line lands the mechanism decided in
//! the field S1 report). The flag is static per composition — the
//! control suppresses its own indication only while IT holds focus —
//! and a no-op under the base recipe, where the control ring is the
//! visible indicator and the card paints no chrome. The card itself
//! cannot set the flag (it does not construct the control); the
//! composition author owns the wiring, and S3 demos must wire every
//! card-nested toggleable control.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut notifications = false;
//! Field::new().show(ui, |ui| {
//!     FieldLabel::new("Notifications").for_id(egui::Id::unique("notif")).show(ui);
//!     FieldDescription::new("Get notified about account activity.").show(ui);
//!     Checkbox::new(&mut notifications).id_salt("notif").show(ui);
//!     FieldError::new("Please choose at least one channel.").show(ui);
//! });
//!
//! // Choice card: the label wraps a nested Field around a control.
//! // The nested control carries `.suppress_focus_indication(true)` so
//! // the card ring (not the control ring) is the focus indicator.
//! let mut plan = true;
//! FieldLabel::new("")
//!     .checked(Some(plan))
//!     .show_with_card(ui, |ui| {
//!         Field::new().orientation(FieldOrientation::Horizontal).show(ui, |ui| {
//!             Checkbox::new(&mut plan).id_salt("plan").show(ui);
//!         });
//!     });
//! ```

use crate::theme::{color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Id, Pos2, Rect, Response, Sense, Ui, Vec2};

/// Shared field state passed to children through the temp stack.
///
/// Each container pushes its own entry and pops it after its content,
/// so the top entry describes the nearest enclosing field context.
/// `invalid`/`disabled` follow ANY-ancestor inheritance (FLD-VIS-02):
/// a container OR-accumulates its own flags with the enclosing entry,
/// so a nested Field never shadows an outer Field's flags. Group
/// ancestry passes through unchanged, and
/// `horizontal_content_present` stays scoped to direct children (every
/// container except the horizontal [`Field`] itself resets it).
#[derive(Clone, Copy, Debug)]
struct FieldCtx {
    /// Whether ANY enclosing [`Field`] declares the invalid flag (D1
    /// explicit flag for `data-invalid="true"`; label/title text
    /// renders destructive). OR-accumulated down the stack.
    invalid: bool,
    /// Whether ANY enclosing [`Field`] declares the disabled flag (D1
    /// explicit flag for the field group's `data-disabled="true"`;
    /// label/title render at element-level 50% opacity, clicks still
    /// forward — the opacity-only condition of spec §4.2, distinct
    /// from the label's own pointer-events-none flag).
    /// OR-accumulated down the stack.
    disabled: bool,
    /// Number of enclosing [`FieldGroup`]s. A group with depth > 0 is
    /// directly nested in another group and renders the N14 16px gap.
    group_depth: usize,
    /// Inline size of the nearest enclosing [`FieldGroup`] (the named
    /// `@container/field-group`), or `None` when no group ancestor
    /// exists. Responsive [`Field`]s switch on this width (D6); with no
    /// container the `@md` rules never match and the field stays
    /// columnar (spec §2.4).
    group_container_width: Option<f32>,
    /// True only for DIRECT children of a horizontal [`Field`] that has
    /// a direct [`FieldContent`] child. Exposes the `mt-px` condition
    /// (spec §2.4, OPEN-6) to control lines: a direct-child checkbox or
    /// radio role shifts down 1px. Field cannot apply the shift itself
    /// (children are caller closures); control lines that consume this
    /// flag own the offset. Containers that are not the horizontal
    /// Field itself reset it, so the scope stays direct-child-only.
    /// Control lines read it via
    /// [`field_horizontal_content_present`].
    horizontal_content_present: bool,
}

impl FieldCtx {
    fn root() -> Self {
        Self {
            invalid: false,
            disabled: false,
            group_depth: 0,
            group_container_width: None,
            horizontal_content_present: false,
        }
    }
}

fn ctx_stack_id() -> Id {
    Id::unique("shadcn_field_ctx_stack")
}

fn push_ctx(ui: &Ui, ctx: FieldCtx) {
    let mut stack: Vec<FieldCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctx_stack_id()).unwrap_or_default());
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(ctx_stack_id(), stack));
}

fn pop_ctx(ui: &Ui) {
    let mut stack: Vec<FieldCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctx_stack_id()).unwrap_or_default());
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(ctx_stack_id(), stack));
}

fn current_ctx(ui: &Ui) -> FieldCtx {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<FieldCtx>>(ctx_stack_id()))
        .and_then(|s| s.last().copied())
        .unwrap_or_else(FieldCtx::root)
}

/// Whether the calling widget is a DIRECT child of a horizontal
/// [`Field`] with a direct [`FieldContent`] child — the `mt-px`
/// condition of spec §2.4 (OPEN-6). Control lines consume this: a
/// direct-child checkbox or radio role shifts down 1px when it reads
/// true. The flag is set only by the horizontal [`Field`] itself and
/// reset by every other container, so the scope stays
/// direct-child-only; [`Field`] cannot apply the shift itself
/// (children are caller closures).
///
/// The readers land on the checkbox and radio-group lines, which is
/// why this accessor carries `#[allow(dead_code)]` until they do:
/// the field side of the OPEN-6 contract is complete here, and the
/// consumers arrive with their S1s (recorded in the field S1 report
/// for S8 ratification).
#[allow(dead_code)]
pub(crate) fn field_horizontal_content_present(ui: &Ui) -> bool {
    current_ctx(ui).horizontal_content_present
}

/// The `@md` container-query breakpoint for the `field-group` container:
/// 28rem = 448px (Tailwind v4 framework default, OPEN-1 — not snapshot
/// evidence; S1 implements it as stated, S4/S6 own the value).
pub(crate) const FIELD_GROUP_MD_BREAKPOINT: f32 = 448.0;

/// Resolves a responsive [`Field`]'s effective orientation from the
/// nearest enclosing [`FieldGroup`]'s inline size (pure, testable): at or
/// above the breakpoint the field renders horizontally; below it, or
/// with no enclosing group (no named container exists), it renders
/// columnar (spec §2.4).
pub(crate) fn resolve_responsive_orientation(container_width: Option<f32>) -> FieldOrientation {
    match container_width {
        Some(w) if w >= FIELD_GROUP_MD_BREAKPOINT => FieldOrientation::Horizontal,
        _ => FieldOrientation::Vertical,
    }
}

/// Gap between the field's rows (`gap-3` base recipe, nova `gap-2`).
pub(crate) fn field_gap(style: Style) -> f32 {
    match style {
        Style::Nova => crate::theme::tokens::spacing::SM,
        _ => crate::theme::tokens::spacing::MD,
    }
}

/// Gap inside a [`FieldContent`] column (`gap-1.5` base, nova `gap-0.5`).
pub(crate) fn field_content_gap(style: Style) -> f32 {
    match style {
        Style::Nova => crate::theme::tokens::spacing::HALF,
        _ => crate::theme::tokens::spacing::XS_HALF,
    }
}

/// Gap between fields in a [`FieldGroup`] with the N14 conditional rules
/// (pure, testable): `gap-7` base / `gap-5` nova by default; 16px when
/// the group is directly nested in another [`FieldGroup`] (both styles);
/// 12px when the group itself carries the checkbox-group slot (both
/// styles). When both conditions hold, the self-conditional wins (S1
/// precedence choice, PROVISIONAL — the snapshot cannot establish the
/// cascade order; S8 ratifies).
pub(crate) fn field_group_gap_for(style: Style, nested: bool, checkbox_group: bool) -> f32 {
    if checkbox_group {
        crate::theme::tokens::spacing::MD // gap-3
    } else if nested {
        crate::theme::tokens::spacing::LG // gap-4
    } else {
        match style {
            Style::Nova => 20.0,
            _ => crate::theme::tokens::spacing::X2L - crate::theme::tokens::spacing::XS,
        }
    }
}

/// Gap between fields in a [`FieldSet`] with the N14 conditional rule
/// (pure, testable): `gap-6` base / `gap-4` nova by default; 12px
/// (`gap-3`, both styles) when a direct child carries the
/// checkbox-group or radio-group slot (spec §2.1).
pub(crate) fn field_set_gap_for(style: Style, has_group_child: bool) -> f32 {
    if has_group_child {
        crate::theme::tokens::spacing::MD // gap-3
    } else {
        match style {
            Style::Nova => crate::theme::tokens::spacing::LG,
            _ => crate::theme::tokens::spacing::XL,
        }
    }
}

/// Bottom spacing after a [`FieldLegend`] (`mb-3` base, nova `mb-1.5`).
pub(crate) fn field_legend_mb(style: Style) -> f32 {
    match style {
        Style::Nova => crate::theme::tokens::spacing::XS_HALF,
        _ => crate::theme::tokens::spacing::MD,
    }
}

/// Padding around each nested field inside the [`FieldLabel`] choice card
/// (`*:data-[slot=field]:p-4` base, nova `p-2.5`).
pub(crate) fn field_label_card_padding(style: Style) -> f32 {
    match style {
        Style::Nova => crate::theme::tokens::spacing::SM_HALF,
        _ => crate::theme::tokens::spacing::LG,
    }
}

/// Corner radius of the [`FieldLabel`] choice card (`rounded-md` base,
/// nova `rounded-lg`).
pub(crate) fn field_label_card_radius(radius: &RadiusTokens, style: Style) -> u8 {
    match style {
        Style::Nova => radius.lg_u8(),
        _ => radius.md_u8(),
    }
}

/// Top-margin adjustment for a [`FieldDescription`] (pure, testable):
/// the three frozen margin rules of spec §2.8, as an explicit spacing
/// adjustment (D5). `last:mt-0` needs no adjustment (it asserts the
/// default: the parent gap alone separates); `nth-last-2:-mt-1` pulls
/// the description 4px toward the previous sibling; the legend-follow
/// `-mt-1.5` pulls it 6px. When legend-follow co-matches with
/// second-from-last, legend-follow wins (OPEN-5, the more specific
/// contextual rule — S1 precedence, S5 judges single-condition cases
/// exactly and records the co-match as OPEN-5).
pub(crate) fn description_margin_top(after_legend: bool, second_from_last: bool) -> f32 {
    if after_legend {
        -6.0 // -mt-1.5
    } else if second_from_last {
        -4.0 // -mt-1
    } else {
        0.0
    }
}

/// Fill and border colors of the [`FieldLabel`] choice card (pure,
/// testable): `has-data-[state=checked]:bg-primary/5 border-primary` base;
/// nova softens the checked border to `border-primary/30` (dark `/20`),
/// fills `bg-primary/10` in dark mode and adds a `bg-muted/50` hover fill
/// while enabled. Under nova, a focused descendant (`focus_inside`, N9)
/// recolors the border to `ring` at full alpha; the focus border wins
/// over the checked border (PROVISIONAL, OPEN-3 — generated-CSS order is
/// unestablishable; S5 judges each state separately and records the
/// combined case without verdict). Checked wins over hover for the fill
/// (PROVISIONAL, OPEN-3, same basis).
///
/// Disabled compositing: the label/title rule is element-level
/// `opacity-50` (group fade, spec §4.2). The card's fill paints over the
/// backdrop alone, so halving its alpha is exactly group fade; the
/// border paints over the fill, so halving both is exact when the fill
/// is transparent (resting card) and within ~1 level when the fill is a
/// near-backdrop tint (checked/hover) — inside measurement tolerance,
/// with no backdrop assumption. The focus ring (outside the border, over
/// the backdrop alone) is halved exactly; see
/// [`field_label_card_ring_color`].
pub(crate) fn field_label_card_colors(
    p: &Palette,
    dark: bool,
    style: Style,
    checked: bool,
    hovered: bool,
    disabled: bool,
    focus_inside: bool,
) -> (Color32, Color32) {
    let mut fill = if checked {
        color_alpha(p.primary, if dark { 0.10 } else { 0.05 })
    } else if hovered && !disabled && style == Style::Nova {
        color_alpha(p.muted, 0.5) // hover:bg-muted/50
    } else {
        Color32::TRANSPARENT
    };
    // N9 (nova-only): focus-inside recolors the border to `ring`.
    // PROVISIONAL (OPEN-3): focus wins over checked for the border.
    let border = if focus_inside && style == Style::Nova {
        p.ring
    } else if checked {
        match style {
            // nova has-data-checked:border-primary/30 (dark /20)
            Style::Nova => color_alpha(p.primary, if dark { 0.2 } else { 0.3 }),
            // base has-data-[state=checked]:border-primary
            _ => p.primary,
        }
    } else {
        p.border
    };
    if disabled {
        // group-data-[disabled=true]/field:opacity-50
        fill = color_alpha(fill, 0.5);
    }
    (fill, color_alpha(border, if disabled { 0.5 } else { 1.0 }))
}

/// Color of the choice card's focus-inside ring (pure, testable): `ring`
/// at 50% alpha (N9, nova-only), halved again when the label is disabled
/// — the ring paints outside the border over the backdrop alone, so
/// halving is exactly the element-level group fade (spec §4.2). The
/// focus rule itself has no disabled guard (spec §9): the ring paints
/// whenever a descendant holds focus, disabled or not.
pub(crate) fn field_label_card_ring_color(p: &Palette, disabled: bool) -> Color32 {
    color_alpha(p.ring, if disabled { 0.25 } else { 0.5 })
}

/// Whether a pointer-state click on a label/card forwards to the bound
/// control (pure, testable — the F2 release-position mechanism). A click
/// forwards when the pointer was released inside the label/card rect
/// AND the click did not land on the target control itself (the Radix
/// `isClickingControl` exclusion: a direct click on the nested control
/// fires it once, and forwarding would fire it twice). The release
/// position (`latest_pos`) is the only position input: `press_origin` is
/// dead for click detection in egui 0.37 (cleared in `begin_pass`
/// before `any_click()` is ever true), and egui only reports a click
/// when the pointer stayed within `max_click_dist` of the press, so a
/// release inside the rect means the press began inside (or within a
/// few px of) it.
pub(crate) fn card_click_detected(
    clicked_on_target: bool,
    any_click: bool,
    latest_pos: Option<Pos2>,
    outer: Rect,
) -> bool {
    !clicked_on_target && any_click && latest_pos.is_some_and(|pos| outer.contains(pos))
}

/// Deduplicates [`FieldError`] entries by message (pure, testable — N13,
/// `field.tsx:203-205`): `new Map(errors.map(error => [error?.message,
/// error]))`. The key is the message string, or `None` for entries
/// without one; later entries with an already-seen key REPLACE the
/// stored entry but keep the FIRST occurrence's position (Map
/// insertion-order semantics). Entries that are themselves `None`, and
/// entries with a `None` message, share the `None` key and collapse to
/// one. Only the message renders, so the last-wins replacement is
/// unobservable beyond the key set and order — but the function keeps
/// the stored entry faithful to the Map semantics anyway.
pub(crate) fn dedupe_error_entries(entries: &[Option<String>]) -> Vec<Option<String>> {
    let mut unique: Vec<Option<String>> = Vec::new();
    for entry in entries {
        let key = entry.as_deref();
        if let Some(slot) = unique.iter_mut().find(|e| e.as_deref() == key) {
            *slot = entry.clone(); // last-wins, first position kept
        } else {
            unique.push(entry.clone());
        }
    }
    unique
}

/// The render plan for a [`FieldError`] errors list (pure, testable —
/// N13, `field.tsx:199-223`): whether the list form renders, and the
/// messages that render. The form decision uses the DEDUPED count:
/// exactly one unique entry renders as bare text (or nothing when its
/// message is falsy); two or more unique entries render the `<ul>`
/// list, with falsy-message entries rendering nothing inside it — so
/// `["", "a"]` renders the list form with a single bullet, not bare
/// text. An empty plan (no truthy messages) renders nothing at all.
pub(crate) fn error_render_plan(entries: &[Option<String>]) -> (bool, Vec<String>) {
    let unique = dedupe_error_entries(entries);
    let list_form = unique.len() >= 2;
    let messages: Vec<String> = unique
        .into_iter()
        .flatten()
        .filter(|m| !m.is_empty())
        .collect();
    (list_form, messages)
}

/// The orientation preset of a [`Field`] (`fieldVariants.orientation`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldOrientation {
    /// `flex-col gap-3` — label above the control.
    #[default]
    Vertical,
    /// `flex-row items-center gap-3` — label beside the control.
    Horizontal,
    /// `@md/field-group:` — horizontal when the container is wide enough
    /// (>= 448pt, the `@md` breakpoint), vertical otherwise.
    Responsive,
}

/// A shadcn field: a labeled control group. See the [module docs](self).
#[derive(Debug)]
pub struct Field {
    orientation: FieldOrientation,
    invalid: bool,
    disabled: bool,
    /// Caller-declared: a direct [`FieldContent`] child is present
    /// (spec §2.4). Drives the horizontal `items-start` switch and the
    /// `mt-px` context flag. A closure's children cannot be inspected,
    /// so the caller declares it (D1); the default (`false`) is the
    /// no-content layout.
    has_content_child: bool,
    id_salt: Option<String>,
}

impl Default for Field {
    fn default() -> Self {
        Self::new()
    }
}

impl Field {
    /// Creates a vertical field.
    pub fn new() -> Self {
        Self {
            orientation: FieldOrientation::Vertical,
            invalid: false,
            disabled: false,
            has_content_child: false,
            id_salt: None,
        }
    }

    /// Sets the orientation preset.
    pub fn orientation(mut self, orientation: FieldOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Marks the field `data-invalid` (label/title render destructive).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Marks the field disabled (label/title render at 50% opacity).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Declares that a direct [`FieldContent`] child is present. Under
    /// the horizontal (or responsive-at-breakpoint) orientation this
    /// switches cross-axis alignment from centered to top-aligned
    /// (`items-start`, spec §2.4) and exposes the `mt-px` condition to
    /// direct-child checkbox/radio roles via the field context. Without
    /// a content child the row stays centered and no `mt-px` applies.
    pub fn has_content_child(mut self, has_content_child: bool) -> Self {
        self.has_content_child = has_content_child;
        self
    }

    /// Sets a stable id salt (Field keeps no state, but nested containers
    /// derive ids from the scope).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the field around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        // The responsive switch measures the nearest enclosing
        // FieldGroup's inline size (the `@container/field-group`); with
        // no group ancestor there is no named container and the field
        // always renders columnar (spec §2.4, D6).
        let enclosing = current_ctx(ui);
        let effective = match self.orientation {
            FieldOrientation::Vertical => FieldOrientation::Vertical,
            FieldOrientation::Horizontal => FieldOrientation::Horizontal,
            FieldOrientation::Responsive => {
                resolve_responsive_orientation(enclosing.group_container_width)
            }
        };

        push_ctx(
            ui,
            FieldCtx {
                // ANY-ancestor inheritance (FLD-VIS-02, spec §4.2/§4.4):
                // the snapshot's named-group disabled rule is a
                // descendant selector matching ANY ancestor Field, and
                // the invalid text colour inherits through intermediate
                // elements. A nested Field that declares no flag must
                // NOT shadow an outer Field's flag, so the pushed
                // values OR-accumulate with the enclosing context. This
                // changes only what descendants OBSERVE for visuals;
                // the forwarding distinction (the label's own flag =
                // pointer-events-none, the field flag = opacity-only)
                // lives in the consumers' use of `self.disabled` vs the
                // ctx flag and is untouched.
                invalid: self.invalid || enclosing.invalid,
                disabled: self.disabled || enclosing.disabled,
                // Group ancestry passes through: a Field neither opens
                // nor closes a field-group container.
                group_depth: enclosing.group_depth,
                group_container_width: enclosing.group_container_width,
                // The `mt-px` condition holds only for direct children
                // of this horizontal field with a content child present.
                horizontal_content_present: effective == FieldOrientation::Horizontal
                    && self.has_content_child,
            },
        );
        let gap = field_gap(ShadcnTheme::of_ui(ui).style); // gap-3 / nova gap-2
        let inner = match effective {
            FieldOrientation::Horizontal => {
                if self.has_content_child {
                    // `has-[>[data-slot=field-content]]:items-start`
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        content(ui)
                    })
                } else {
                    // `flex-row items-center`
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        content(ui)
                    })
                }
            }
            _ => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = gap;
                content(ui)
            }),
        };
        pop_ctx(ui);
        inner
    }
}

/// Temp-data key for a bound-label click that still needs forwarding to
/// its target (see [`record_pending_click`]). The salt differs from the
/// `Label` component's key so a field label and a plain label bound to
/// the same control never consume each other's pending click.
fn pending_click_id(target: Id) -> Id {
    target.with("shadcn_field_pending_click")
}

/// A pending click older than this is dropped instead of forwarded. The
/// forwarding normally lands on the very next pass; the bound only stops
/// a stale click from firing if the label vanishes before the next pass
/// and reappears much later. Generous on purpose: passes under load can
/// lag seconds behind wall time.
const PENDING_CLICK_TTL: f64 = 30.0;

/// Records that a bound-label click must be forwarded to `target` on the
/// next pass, and requests the repaint that guarantees the next pass
/// happens.
///
/// Why deferred: in the click pass the label is usually shown BEFORE its
/// target, and egui 0.37's surrender-on-click (`Context::create_widget`:
/// a click that lands elsewhere surrenders the focused widget) undoes a
/// focus request in the same pass the target is created. On the next
/// pass no click is in flight, so the focus request sticks, and the
/// synthetic Space press (see [`consume_pending_click`]) reaches the
/// target's `interact` while it holds focus. (Label-precedent shape —
/// `label.rs` `record_pending_click`; duplicated locally because a
/// shared helper is a foundation decision.)
fn record_pending_click(ctx: &egui::Context, target: Id) {
    let now = ctx.input(|i| i.time);
    ctx.data_mut(|data| data.insert_temp(pending_click_id(target), now));
    ctx.request_repaint();
}

/// Forwards a click recorded by [`record_pending_click`] on a previous
/// pass: requests focus on `target`, then pushes a synthetic Space
/// key-down + key-up pair into the live input events.
///
/// The pair is what makes the target ACTIVATE: egui's `interact`
/// reports a fake primary click for the focused widget whenever it sees
/// `key_pressed(Space)` or `key_pressed(Enter)`, and `key_pressed`
/// scans the live event list, so events pushed here are visible to every
/// widget shown later in this pass — including the target, which the
/// label precedes in the usual label-then-control order. Both halves of
/// the pair matter: the key-down is what `interact` reads, and the
/// key-up is what the crate's button needs — its Space-fires-on-keyup
/// handling hides a lone key-down and only re-creates it when the
/// release arrives. A bare Space key press inserts no text (text
/// insertion reads `Event::Text`, never a lone `Key` event), so a
/// text-input target focuses with no side effect; a disabled target is
/// protected twice over — egui's `ignore_focus` clears focus for
/// non-focusable widgets before `interact` reads it, and the crate's
/// controls gate `clicked()` on `!disabled`.
///
/// The pending flag is consumed (removed) whether or not `enabled`:
/// a label disabled after the click must not fire a stale activation
/// if it is re-enabled later. Callers place this where the target runs
/// AFTER it in the same pass: at the top of [`FieldLabel::show`], and
/// before the frame content in [`FieldLabel::show_with_card`].
fn consume_pending_click(ui: &mut Ui, target: Id, enabled: bool) {
    let set_at = ui
        .ctx()
        .data_mut(|data| data.remove_temp::<f64>(pending_click_id(target)));
    let Some(set_at) = set_at else {
        return;
    };
    if !enabled {
        return;
    }
    let now = ui.input(|i| i.time);
    if now - set_at > PENDING_CLICK_TTL {
        return;
    }
    // Focus first: the fake click only applies to the focused widget.
    ui.memory_mut(|mem| mem.request_focus(target));
    ui.ctx().input_mut(|input| {
        for pressed in [true, false] {
            input.events.push(egui::Event::Key {
                key: egui::Key::Space,
                physical_key: Some(egui::Key::Space),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
    });
}

/// `FieldLabel` — a `text-sm font-medium` label bound to the field state.
///
/// Reads the enclosing [`Field`]: 50% opacity when the field is disabled,
/// destructive color when it is invalid. For the source's choice-card
/// composition (the label wrapping a nested [`Field`], e.g. around a
/// checkbox), use [`FieldLabel::show_with_card`].
#[derive(Debug)]
pub struct FieldLabel {
    text: String,
    for_id: Option<Id>,
    disabled: bool,
    /// Checked state of the wrapped control for [`FieldLabel::show_with_card`]
    /// (`has-data-[state=checked]`).
    checked: Option<bool>,
    id_salt: Option<String>,
}

impl FieldLabel {
    /// Creates a field label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            for_id: None,
            disabled: false,
            checked: None,
            id_salt: None,
        }
    }

    /// Binds the label to a control id: clicking the label forwards the
    /// click to that widget — activation for checkbox, radio, switch,
    /// and button targets, focus for text inputs — and focuses it, like
    /// Radix `htmlFor` (spec §7.2, the settled label rule). `htmlFor`
    /// has no default: a label without a target has no associated
    /// control.
    ///
    /// When the control is a crate widget shown after the label in the
    /// same `Ui` scope, its id is usually `ui.scope_id().with(id_salt)`
    /// for the same `id_salt` the control is constructed with. Some
    /// widgets compute their widget id inside a child `Ui` — `Input`
    /// shows its text edit in a `ui.scope`, so its widget id is NOT
    /// `ui.scope_id().with(id_salt)`. For those, capture the id from
    /// the control's returned [`Response::id`] on one pass and bind it
    /// from the next; binding a computed id that matches no widget
    /// focuses nothing (scoped-id rule, Batch A pack §8 rule 10).
    pub fn for_id(mut self, id: Id) -> Self {
        self.for_id = Some(id);
        self
    }

    /// Disables the label: 50% opacity, non-interactive. This flag
    /// expresses the unnamed-group / peer disabled conditions of spec
    /// §4.2 (Label's `group-data-[disabled=true]:pointer-events-none`
    /// rule): clicks pass through entirely, with no focus transfer and
    /// no activation. It is DISTINCT from the enclosing [`Field`]'s
    /// disabled flag, which expresses the named-field-group condition
    /// (opacity only — clicks still forward). Both conditions render
    /// the label at 50% opacity; only this one suppresses clicks (S1
    /// API decision, D1 — S8 ratifies).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the checked state of the wrapped control, driving the choice
    /// card's `has-data-[state=checked]:bg-primary/5 border-primary` fill
    /// (nova `border-primary/30`, dark `/10` fill and `/20` border).
    pub fn checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the label.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let ctx = current_ctx(ui);
        // Visual disabled = either condition (both render 50% opacity);
        // click suppression = the label's own flag only (the
        // pointer-events-none condition — the field-group condition
        // still forwards clicks, spec §4.2/§7.2).
        let visual_disabled = self.disabled || ctx.disabled;

        // A click recorded on a previous pass is forwarded here, before
        // the target is created later in this pass.
        if let Some(target) = self.for_id {
            consume_pending_click(ui, target, !self.disabled);
        }

        let color = field_label_color(&p, ctx.invalid, visual_disabled);
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        // Laid out once with the paint color: the same galley sizes the
        // widget and paints the text.
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), color);
        // `leading-snug` = 1.375 line height: the text line box is
        // 19.25px (spec §3.6, via the `cn()` merge — OPEN-2).
        let desired = Vec2::new(galley.size().x, font.size * 1.375);

        let interactive = self.for_id.is_some() && !self.disabled;
        // `Sense::CLICK` — click without `FOCUSABLE`: the label never
        // takes a Tab stop (spec §7.4), like `Label`.
        let sense = if interactive {
            Sense::CLICK
        } else {
            Sense::hover()
        };
        let (rect, response) = {
            // Reserve space without registering an auto-id focusable
            // response, so egui's spatial arrow-key navigation only sees the
            // explicit widget id (checkbox.rs pattern).
            let (_, rect) = ui.allocate_space(desired);
            let id = match self.id_salt.as_deref() {
                Some(salt) => ui.scope_id().with(salt),
                None => ui.auto_id_with("shadcn_field_label"),
            };
            (rect, ui.interact(rect, id, sense))
        };

        // Click forwarding (spec §7.2, the settled label rule): the
        // click is recorded for the next pass, when no click is in
        // flight to surrender the focus request. The immediate request
        // is kept too — it helps when the label runs AFTER its target
        // — but on its own it never survives the click pass.
        if interactive && response.clicked() {
            if let Some(target) = self.for_id {
                ui.memory_mut(|mem| mem.request_focus(target));
                record_pending_click(ui.ctx(), target);
            }
        }

        if ui.is_rect_visible(rect) {
            // Same paint math as `Painter::text`, reusing the galley.
            let text_rect = egui::Align2::CENTER_CENTER.anchor_size(rect.center(), galley.size());
            ui.painter().galley(text_rect.min, galley, color);
        }
        response
    }

    /// Renders the label as the source's choice-card composition: the label
    /// wraps a nested [`Field`] (`has-[>[data-slot=field]]`), so the card
    /// gets `rounded-md border` with `p-4` padding (base) or `rounded-lg
    /// border` with `p-2.5` (nova), a nova-only `bg-muted/50` hover fill
    /// while no descendant is disabled, and — when [`FieldLabel::checked`]
    /// is `Some(true)` — a `bg-primary/5` fill with a primary-tinted
    /// border. Under nova, focus anywhere inside the card adds the 3px
    /// `ring-ring/50` halo and recolors the border to `ring` (N9, spec
    /// §4.5); under base the card paints no focus chrome (the control
    /// paints its own ring). Clicking the card forwards activation +
    /// focus to [`FieldLabel::for_id`], like a `<label>` element; clicks
    /// directly on the nested control fire it exactly once and are not
    /// re-forwarded.
    ///
    /// A non-empty label text renders above the wrapped field.
    pub fn show_with_card<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();
        let style = theme.style;
        let field_ctx = current_ctx(ui);
        // Visual disabled = either condition (both render 50% opacity);
        // click suppression = the label's own flag only (the
        // pointer-events-none condition — the field-group condition
        // still forwards clicks, spec §4.2/§7.2).
        let visual_disabled = self.disabled || field_ctx.disabled;
        let label_color = field_label_color(&p, field_ctx.invalid, visual_disabled);

        // A click recorded on a previous pass is forwarded here, before
        // the frame content runs, so the nested target sees the focused
        // + Space state during its own `interact`.
        if let Some(target) = self.for_id {
            consume_pending_click(ui, target, !self.disabled);
        }

        // Card frame: `rounded-md` + `p-4` base; nova `rounded-lg` + `p-2.5`.
        let radius = field_label_card_radius(&theme.radius, style);
        let pad = field_label_card_padding(style);
        let corner = egui::CornerRadius::same(radius);

        // The fill and stroke must render BEHIND the content, but their
        // colors depend on the content's final geometry (hover, focus).
        // Reserve two shape slots now and swap in the real shapes after the
        // content has been laid out.
        let painter = ui.painter().clone();
        let fill_idx = painter.add(egui::Shape::Noop);
        let stroke_idx = painter.add(egui::Shape::Noop);

        let frame = egui::Frame::new()
            .inner_margin(egui::Margin::same(pad as i8))
            .corner_radius(corner)
            .show(ui, |ui| {
                if !self.text.is_empty() {
                    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
                    // Placeholder color: the paint-time fallback replaces
                    // it exactly (FLD-VIS-01 class — never lay out with a
                    // real color and rely on the fallback).
                    let galley =
                        painter.layout_no_wrap(self.text.clone(), font, Color32::PLACEHOLDER);
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), galley.size().y),
                        Sense::hover(),
                    );
                    painter.galley(rect.min, galley, label_color);
                    // `gap-2` between the label text and the wrapped field.
                    ui.add_space(crate::theme::tokens::spacing::SM);
                }
                content(ui)
            });
        let outer = frame.response.rect;

        // Keyboard focus inside the card (`has-[:focus-visible]`).
        let focus_inside = ui
            .ctx()
            .memory(|mem| mem.focused())
            .and_then(|id| ui.ctx().read_response(id))
            .is_some_and(|r| r.has_focus() && outer.contains(r.rect.center()));

        // Nova hover fill requires no disabled descendant (spec §4.3);
        // the combined disabled flag is the D1 proxy for it.
        let hovered = !visual_disabled && ui.rect_contains_pointer(outer);
        let (fill, border) = field_label_card_colors(
            &p,
            dark,
            style,
            self.checked.unwrap_or(false),
            hovered,
            visual_disabled,
            focus_inside,
        );

        if ui.is_rect_visible(outer) {
            // Swap the reserved slots so fill/stroke paint behind content.
            painter.set(
                fill_idx,
                egui::Shape::Rect(egui::epaint::RectShape::filled(outer, corner, fill)),
            );
            painter.set(
                stroke_idx,
                egui::Shape::Rect(egui::epaint::RectShape::stroke(
                    outer,
                    corner,
                    egui::Stroke::new(1.0, border),
                    egui::epaint::StrokeKind::Inside,
                )),
            );

            // Nova `has-[:focus-visible]:border-ring ring-3 ring-ring/50`
            // (N9). No disabled guard: the rule applies whenever a
            // descendant holds focus; the ring color carries the
            // disabled fade itself (group fade, exact over the
            // backdrop).
            if focus_inside && style == Style::Nova {
                crate::utils::focus::paint_ring(
                    &painter,
                    outer,
                    field_label_card_ring_color(&p, visual_disabled),
                    3.0,
                    radius as f32,
                );
            }
        }

        // `<label>` semantics, read from pointer state so no click-sense
        // interact steals hits from the nested widgets (the `Label::
        // show_with` pattern). The click is recorded for the next pass,
        // when no click is in flight to surrender the focus request; a
        // click that landed on the target itself is excluded — the
        // target already fired from the real click, and forwarding
        // would fire it twice (Radix's `isClickingControl` exclusion).
        // The click position is the RELEASE position (`latest_pos`):
        // `press_origin` is dead for click detection in egui 0.37 (F2).
        if let Some(target) = self.for_id {
            if !self.disabled {
                let clicked_on_target =
                    ui.ctx().interaction_snapshot(|s| s.clicked == Some(target));
                let clicked = ui.input(|i| {
                    card_click_detected(
                        clicked_on_target,
                        i.pointer.any_click(),
                        i.pointer.latest_pos(),
                        outer,
                    )
                });
                if clicked {
                    ui.memory_mut(|mem| mem.request_focus(target));
                    record_pending_click(ui.ctx(), target);
                }
            }
        }

        frame
    }
}

/// `FieldTitle` — `text-sm font-medium` with an optional leading icon.
#[derive(Debug)]
pub struct FieldTitle {
    text: String,
    icon: Option<crate::icons::Icon>,
    disabled: bool,
    id_salt: Option<String>,
}

impl FieldTitle {
    /// Creates a field title.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Shows an icon before the title.
    pub fn icon(mut self, icon: crate::icons::Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Forces the disabled look regardless of the enclosing field.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let ctx = current_ctx(ui);
        let disabled = self.disabled || ctx.disabled;
        let color = field_label_color(&p, ctx.invalid, disabled);

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let icon_size = crate::theme::tokens::spacing::LG;
        // Laid out once with the paint color: the same galley sizes the
        // widget and paints the text.
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), color);
        let text_w = galley.size().x;
        let gap = self
            .icon
            .map(|_| crate::theme::tokens::spacing::SM)
            .unwrap_or(0.0);
        let width = icon_size * self.icon.is_some() as i32 as f32 + gap + text_w;
        // `leading-snug` = 1.375 line height.
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, font.size * 1.375), Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let mut cursor = rect.min.x;
            if let Some(icon) = self.icon {
                let icon_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(cursor, rect.center().y - icon_size / 2.0),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, icon_rect, color, 2.0 * icon_size / 24.0);
                cursor += icon_size + gap;
            }
            // Same paint math as `Painter::text`, reusing the galley.
            let text_rect = egui::Align2::LEFT_CENTER
                .anchor_size(egui::Pos2::new(cursor, rect.center().y), galley.size());
            painter.galley(text_rect.min, galley, color);
        }
        response
    }
}

/// `FieldDescription` — `text-sm text-muted-foreground` helper text that
/// wraps to the available width, with a 21px line box (`leading-normal`).
///
/// The three frozen top-margin rules of spec §2.8 are caller-declared
/// (D1 — a widget cannot see its siblings): [`FieldDescription::after_legend`]
/// and [`FieldDescription::second_from_last`]. `last:mt-0` needs no API:
/// it asserts the default (the parent gap alone separates). Link rules
/// (underline, hover `primary`) have no expression in the field API —
/// links render as plain muted-foreground text (proposed deviation D2).
/// `text-balance` in horizontal fields has no egui algorithm (D7
/// limitation): descriptions wrap by egui's default algorithm.
#[derive(Debug)]
pub struct FieldDescription {
    text: String,
    /// Immediately preceded by a `legend`-variant [`FieldLegend`]
    /// sibling: `-mt-1.5` (spec §2.8; wins over second-from-last,
    /// OPEN-5).
    after_legend: bool,
    /// Second-from-last child of its parent: `nth-last-2:-mt-1`
    /// (spec §2.8).
    second_from_last: bool,
    id_salt: Option<String>,
}

impl FieldDescription {
    /// Creates a field description.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            after_legend: false,
            second_from_last: false,
            id_salt: None,
        }
    }

    /// Declares the description immediately follows a `legend`-variant
    /// [`FieldLegend`] sibling, applying the `-mt-1.5` tightening
    /// (spec §2.8). A `label`-variant legend does NOT trigger it.
    pub fn after_legend(mut self, after_legend: bool) -> Self {
        self.after_legend = after_legend;
        self
    }

    /// Declares the description is the second-from-last child of its
    /// parent, applying the `nth-last-2:-mt-1` tightening (spec §2.8).
    pub fn second_from_last(mut self, second_from_last: bool) -> Self {
        self.second_from_last = second_from_last;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> Response {
        // Negative top margins (D5 explicit spacing adjustment): pull
        // the description toward the previous sibling. Vertical
        // layouts only — the rules are `mt-*` (top margins).
        let margin = description_margin_top(self.after_legend, self.second_from_last);
        if margin < 0.0 && ui.layout().is_vertical() {
            ui.add_space(margin);
        }

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // `text-muted-foreground` regardless of the invalid flag (it sets an
        // explicit color, unlike the label which inherits); the source has no
        // disabled rule on descriptions, so no fade.
        let color = p.muted_foreground;
        let max_width = ui.available_width();

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        // Placeholder color: the paint-time fallback replaces it exactly
        // (FLD-VIS-01 — laying out with WHITE made the fallback a dead
        // letter and the text painted white-on-white in light mode).
        // `leading-normal` = 1.5: the line box is 21px (spec §3.6).
        let mut job = LayoutJob::single_section(
            self.text.clone(),
            TextFormat {
                font_id: font,
                color: Color32::PLACEHOLDER,
                line_height: Some(crate::theme::tokens::type_scale::SM * 1.5),
                ..Default::default()
            },
        );
        job.wrap.max_width = max_width;
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(galley.size().x.max(1.0), galley.size().y),
            Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            ui.painter()
                .galley(egui::Pos2::new(rect.min.x, rect.min.y), galley, color);
        }
        response
    }
}

/// `FieldError` — `text-sm text-destructive`. Renders one message as
/// bare text, a bulleted list for several (after the frozen dedup
/// algorithm, N13), or nothing at all when there is no content.
///
/// The `errors` prop shape is `Array<{ message?: string } | undefined>`
/// (`field.tsx:186-191`): entries may be absent and messages may be
/// absent. [`FieldError::errors`] takes plain messages;
/// [`FieldError::entries`] takes the faithful shape, where `None` is an
/// entry without a message. [`FieldError::new`] is the `children`
/// override: a truthy message renders verbatim and the `errors` list is
/// ignored.
#[derive(Debug)]
pub struct FieldError {
    message: Option<String>,
    errors: Vec<Option<String>>,
    id_salt: Option<String>,
}

impl FieldError {
    /// Creates a field error with a single message (`children` — renders
    /// verbatim, ignoring any `errors` list).
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            errors: Vec::new(),
            id_salt: None,
        }
    }

    /// Creates a field error that renders one message or a bullet list
    /// after the N13 dedup (duplicate messages collapse to one bullet,
    /// in first-occurrence order).
    pub fn errors(errors: Vec<String>) -> Self {
        Self {
            message: None,
            errors: errors.into_iter().map(Some).collect(),
            id_salt: None,
        }
    }

    /// Creates a field error from the faithful `errors` prop shape:
    /// each entry is a message, or `None` for an entry without one
    /// (an `undefined` entry or an entry with an `undefined` message —
    /// both collapse under the `None` dedup key and render nothing).
    pub fn entries(entries: Vec<Option<String>>) -> Self {
        Self {
            message: None,
            errors: entries,
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the error (or nothing when there is no message).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let color = p.destructive;
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);

        // `field.tsx:194-223`: truthy `children` render verbatim and the
        // `errors` prop is ignored entirely; otherwise the N13 dedup
        // decides the form (bare text vs list) and the messages.
        let (list_form, messages) = match self.message {
            Some(m) if !m.is_empty() => (false, vec![m]),
            _ => error_render_plan(&self.errors),
        };
        if messages.is_empty() {
            // `if (!content) return null` — no space taken.
            return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
        }

        let max_width = ui.available_width();
        // Placeholder color: the paint-time fallback replaces it exactly
        // (FLD-VIS-01 class — never lay out with a real color and rely
        // on the fallback).
        let layout_text = |text: String, width: f32| {
            ui.painter()
                .layout(text, font.clone(), Color32::PLACEHOLDER, width)
        };

        let (total_height, rows): (f32, Vec<_>) = if !list_form {
            let galley = layout_text(messages[0].clone(), max_width);
            (galley.size().y, vec![(None, galley)])
        } else {
            // `ml-4 list-disc gap-1`: 16px left margin, disc bullets,
            // 4px gaps. The bullet paints separately from the text so
            // wrapped lines align with the item's text, not the bullet
            // (spec §3.5); marker geometry is a browser default (OPEN-9,
            // S5 observation only).
            let indent = crate::theme::tokens::spacing::LG;
            let bullet =
                ui.painter()
                    .layout_no_wrap("• ".to_owned(), font.clone(), Color32::PLACEHOLDER);
            let bullet_w = bullet.size().x;
            let mut height = 0.0;
            let mut rows = Vec::new();
            for (i, m) in messages.iter().enumerate() {
                let galley = layout_text(m.clone(), (max_width - indent - bullet_w).max(1.0));
                height += galley.size().y
                    + if i > 0 {
                        crate::theme::tokens::spacing::XS
                    } else {
                        0.0
                    };
                rows.push((Some(bullet.clone()), galley));
            }
            (height, rows)
        };

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(max_width, total_height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let indent = if list_form {
                crate::theme::tokens::spacing::LG
            } else {
                0.0
            };
            let mut y = rect.min.y;
            for (bullet, galley) in &rows {
                let mut x = rect.min.x + indent;
                if let Some(bullet) = bullet {
                    ui.painter()
                        .galley(egui::Pos2::new(x, y), bullet.clone(), color);
                    x += bullet.size().x;
                }
                ui.painter()
                    .galley(egui::Pos2::new(x, y), galley.clone(), color);
                y += galley.size().y + crate::theme::tokens::spacing::XS;
            }
        }
        response
    }
}

/// `FieldContent` — stacks the description/error column (`gap-1.5`
/// base / `gap-0.5` nova) and takes the remaining row space (`flex-1`).
/// In a row [`Field`] it fills the row with no trailing gap; the exact
/// split with a `flex-auto` label-slot sibling follows flex-grow
/// arithmetic and is not judged (spec §3.2). The `leading-snug`
/// inheritance has no egui expression: descendant text parts declare
/// their own leading.
#[derive(Debug, Default)]
pub struct FieldContent {
    id_salt: Option<String>,
}

impl FieldContent {
    /// Creates a field content column.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the column.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        // Children of a content column are not direct children of an
        // enclosing Field: the `mt-px` condition does not cross this
        // boundary. Everything else passes through.
        let enclosing = current_ctx(ui);
        push_ctx(
            ui,
            FieldCtx {
                horizontal_content_present: false,
                ..enclosing
            },
        );
        // `gap-1.5` base; nova `gap-0.5`.
        let gap = field_content_gap(ShadcnTheme::of_ui(ui).style);
        let inner = ui.vertical(|ui| {
            // `flex-1`: take the remaining width (in a row Field, the
            // rest of the row; in a column Field, the full width its
            // children already wrap to).
            let width = ui.available_width();
            ui.set_min_width(width);
            ui.spacing_mut().item_spacing.y = gap;
            content(ui)
        });
        pop_ctx(ui);
        inner
    }
}

/// `FieldGroup` — stacks fields with `gap-7` base / `gap-5` nova.
///
/// N14: a group directly nested in another group renders gap 16px, and
/// a group carrying the checkbox-group slot renders gap 12px (both
/// styles). The group establishes the named `@container/field-group`
/// for responsive [`Field`] descendants (D6): its inline size is the
/// width responsive fields switch on.
#[derive(Debug, Default)]
pub struct FieldGroup {
    /// This group carries `data-slot="checkbox-group"` (caller
    /// override of the default slot via the props spread, spec §2.3):
    /// the self-conditional `gap-3` applies.
    checkbox_group: bool,
    id_salt: Option<String>,
}

impl FieldGroup {
    /// Creates a field group.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares this group carries the `data-slot="checkbox-group"`
    /// slot, applying the self-conditional 12px gap (spec §2.3). The
    /// default slot is `field-group`; only a caller override fires
    /// this rule.
    pub fn checkbox_group(mut self, checkbox_group: bool) -> Self {
        self.checkbox_group = checkbox_group;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let enclosing = current_ctx(ui);
        // N14: nested (directly inside another group) or carrying the
        // checkbox-group slot.
        let gap = field_group_gap_for(
            ShadcnTheme::of_ui(ui).style,
            enclosing.group_depth > 0,
            self.checkbox_group,
        );
        // This group is the nearest `@container/field-group` for its
        // descendants: push its inline size (D6) and one more depth
        // level. The `mt-px` condition does not cross the group
        // boundary.
        let container_width = ui.available_width();
        push_ctx(
            ui,
            FieldCtx {
                invalid: enclosing.invalid,
                disabled: enclosing.disabled,
                group_depth: enclosing.group_depth + 1,
                group_container_width: Some(container_width),
                horizontal_content_present: false,
            },
        );
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = gap;
            content(ui)
        });
        pop_ctx(ui);
        inner
    }
}

/// `FieldSet` — a form section stacking fields with `gap-6` base /
/// `gap-4` nova.
///
/// N14: the gap collapses to 12px when a direct child carries the
/// checkbox-group or radio-group slot (caller-declared via
/// [`FieldSet::has_group_child`] — a closure's children cannot be
/// inspected, D1).
///
/// Native fieldset semantics (OPEN-8, S1 decision): a native disabled
/// `<fieldset>` disables its descendant controls, but egui has no
/// fieldset element and no propagation mechanism (D3 limitation) — the
/// Rust API models no disabled propagation, and callers disable
/// individual controls. Legend positioning is normal flow: a
/// [`FieldLegend`] renders as the set's first child with its bottom
/// margin. No `disabled` builder is offered: a no-op flag would promise
/// behaviour the port cannot deliver.
#[derive(Debug, Default)]
pub struct FieldSet {
    /// A direct child carries the checkbox-group or radio-group slot
    /// (spec §2.1): the gap collapses to 12px.
    has_group_child: bool,
    id_salt: Option<String>,
}

impl FieldSet {
    /// Creates a field set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares a direct checkbox-group or radio-group child is
    /// present, collapsing the gap to 12px (spec §2.1, N14). The
    /// radio-group case fires for a direct `RadioGroup` child (which
    /// renders `data-slot="radio-group"`); the checkbox-group case has
    /// no default producer and fires only for a caller-declared child.
    pub fn has_group_child(mut self, has_group_child: bool) -> Self {
        self.has_group_child = has_group_child;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the set.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        // Children of a set are not direct children of an enclosing
        // Field: the `mt-px` condition does not cross this boundary.
        // Everything else passes through.
        let enclosing = current_ctx(ui);
        push_ctx(
            ui,
            FieldCtx {
                horizontal_content_present: false,
                ..enclosing
            },
        );
        // `gap-6` base; nova `gap-4` (12px with a group child, N14).
        let gap = field_set_gap_for(ShadcnTheme::of_ui(ui).style, self.has_group_child);
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = gap;
            content(ui)
        });
        pop_ctx(ui);
        inner
    }
}

/// `FieldLegend` variant (`data-variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldLegendVariant {
    /// `text-base font-medium` section heading.
    #[default]
    Legend,
    /// `text-sm font-medium` sub-heading.
    Label,
}

/// `FieldLegend` — a section heading with `mb-3` bottom spacing.
#[derive(Debug)]
pub struct FieldLegend {
    text: String,
    variant: FieldLegendVariant,
    id_salt: Option<String>,
}

impl FieldLegend {
    /// Creates a legend.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: FieldLegendVariant::Legend,
            id_salt: None,
        }
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: FieldLegendVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the legend.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = match self.variant {
            FieldLegendVariant::Legend => crate::theme::tokens::type_scale::BASE,
            FieldLegendVariant::Label => crate::theme::tokens::type_scale::SM,
        };
        let font = FontId::proportional(size);
        // Placeholder color: the paint-time fallback replaces it exactly
        // (FLD-VIS-01 class — never lay out with a real color and rely
        // on the fallback).
        let galley =
            ui.painter()
                .layout_no_wrap(self.text.clone(), font.clone(), Color32::PLACEHOLDER);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(galley.size().x, galley.size().y), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.foreground);
        }
        // `mb-3` base; nova `mb-1.5`.
        ui.add_space(field_legend_mb(theme.style));
        response
    }
}

/// `FieldSeparator` — a horizontal rule with optional centered text
/// (`relative -my-2 h-5` with the content chip on top).
#[derive(Debug, Default)]
pub struct FieldSeparator {
    content: Option<String>,
    id_salt: Option<String>,
}

impl FieldSeparator {
    /// Creates a plain rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a rule with centered text.
    pub fn content(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the separator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // `-my-2` (D5 explicit spacing adjustment): the 20px row
        // overlaps 8px into each neighbour's space. Vertical layouts
        // only — the rule is a vertical margin.
        let vertical = ui.layout().is_vertical();
        if vertical {
            ui.add_space(-8.0);
        }
        let height = 20.0; // `h-5` visual row
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // The 1px line's TOP edge sits at 50% of the 20px row
            // (y = 10px within the row; no centering transform, spec
            // §2.9): it occupies y ∈ [10px, 11px). A filled rect
            // rasterizes that band exactly (the `Separator` component's
            // own paint shape).
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(rect.min.x, rect.min.y + 10.0),
                    Vec2::new(rect.width(), 1.0),
                ),
                0,
                p.border,
            );
            if let Some(text) = self.content {
                // The chip paints `bg-background` over the rule, then the text.
                let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
                // Laid out once with the paint color: the same galley sizes
                // the chip and paints the text.
                let galley = painter.layout_no_wrap(text, font, p.muted_foreground);
                let pad = crate::theme::tokens::spacing::SM; // px-2
                let chip_w = galley.size().x + pad * 2.0;
                // Chip height is content-determined (the text line box),
                // centered in the 20px row (spec §3.4).
                let chip =
                    egui::Rect::from_center_size(rect.center(), Vec2::new(chip_w, galley.size().y));
                painter.rect_filled(chip, 0, p.background);
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = egui::Align2::CENTER_CENTER.anchor_size(
                    egui::Pos2::new(chip.center().x, chip.center().y),
                    galley.size(),
                );
                painter.galley(text_rect.min, galley, p.muted_foreground);
            }
        }
        if vertical {
            ui.add_space(-8.0);
        }
        response
    }
}

/// Resolves the label/title color for the field state (pure, testable).
/// The disabled rule is element-level `opacity-50` (group fade, spec
/// §4.2): the text is the label/title's only painted slot over the
/// backdrop, so halving its alpha IS the group fade exactly — no
/// precomposite is needed (contrast the card chrome, where the border
/// paints over the fill; see [`field_label_card_colors`]).
pub(crate) fn field_label_color(p: &Palette, invalid: bool, disabled: bool) -> Color32 {
    let base = if invalid { p.destructive } else { p.foreground };
    if disabled {
        color_alpha(base, 0.5)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Checkbox;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn nova_gaps_match_the_nova_recipe() {
        // Field gap-3 -> gap-2; content gap-1.5 -> gap-0.5; group gap-7 -> gap-5;
        // set gap-6 -> gap-4; legend mb-3 -> mb-1.5; card p-4 -> p-2.5.
        assert_eq!(field_gap(Style::Vega), 12.0);
        assert_eq!(field_gap(Style::Nova), 8.0);
        assert_eq!(field_content_gap(Style::Vega), 6.0);
        assert_eq!(field_content_gap(Style::Nova), 2.0);
        assert_eq!(field_group_gap_for(Style::Vega, false, false), 28.0);
        assert_eq!(field_group_gap_for(Style::Nova, false, false), 20.0);
        assert_eq!(field_set_gap_for(Style::Vega, false), 24.0);
        assert_eq!(field_set_gap_for(Style::Nova, false), 16.0);
        assert_eq!(field_legend_mb(Style::Vega), 12.0);
        assert_eq!(field_legend_mb(Style::Nova), 6.0);
        assert_eq!(field_label_card_padding(Style::Vega), 16.0);
        assert_eq!(field_label_card_padding(Style::Nova), 10.0);
        let radius = crate::theme::RadiusTokens::new(10.0);
        assert_eq!(field_label_card_radius(&radius, Style::Vega), 8); // rounded-md
        assert_eq!(field_label_card_radius(&radius, Style::Nova), 10); // rounded-lg
    }

    #[test]
    fn conditional_gaps_follow_n14() {
        // FieldSet with a direct checkbox/radio-group child: 12px both styles.
        assert_eq!(field_set_gap_for(Style::Vega, true), 12.0);
        assert_eq!(field_set_gap_for(Style::Nova, true), 12.0);
        assert_eq!(field_set_gap_for(Style::Vega, false), 24.0);
        assert_eq!(field_set_gap_for(Style::Nova, false), 16.0);
        // Nested FieldGroup (direct child of a FieldGroup): 16px both styles.
        assert_eq!(field_group_gap_for(Style::Vega, true, false), 16.0);
        assert_eq!(field_group_gap_for(Style::Nova, true, false), 16.0);
        // Self-conditional checkbox-group slot: 12px both styles, winning
        // over the nested rule when both hold (S1 precedence, provisional).
        assert_eq!(field_group_gap_for(Style::Vega, false, true), 12.0);
        assert_eq!(field_group_gap_for(Style::Nova, false, true), 12.0);
        assert_eq!(field_group_gap_for(Style::Vega, true, true), 12.0);
        assert_eq!(field_group_gap_for(Style::Nova, true, true), 12.0);
        // Outermost groups keep the default gaps.
        assert_eq!(field_group_gap_for(Style::Vega, false, false), 28.0);
        assert_eq!(field_group_gap_for(Style::Nova, false, false), 20.0);
    }

    #[test]
    fn description_margin_precedence_is_legend_follow_wins() {
        // OPEN-5: legend-follow (-6) wins over second-from-last (-4);
        // `last:mt-0` needs no adjustment (the default).
        assert_eq!(description_margin_top(false, false), 0.0);
        assert_eq!(description_margin_top(false, true), -4.0);
        assert_eq!(description_margin_top(true, false), -6.0);
        assert_eq!(description_margin_top(true, true), -6.0);
    }

    #[test]
    fn card_colors_follow_checked_and_style() {
        let p = palette(Mode::Light);
        // Base resting: transparent fill, 1px border.
        let (fill, border) =
            field_label_card_colors(&p, false, Style::Vega, false, false, false, false);
        assert_eq!(fill, Color32::TRANSPARENT);
        assert_eq!(border, p.border);
        // Base checked: bg-primary/5 with a full `border-primary`.
        let (fill, border) =
            field_label_card_colors(&p, false, Style::Vega, true, false, false, false);
        assert_eq!(fill, color_alpha(p.primary, 0.05));
        assert_eq!(border, p.primary);
        // Nova checked: bg-primary/5 with `border-primary/30`.
        let (fill, border) =
            field_label_card_colors(&p, false, Style::Nova, true, false, false, false);
        assert_eq!(fill, color_alpha(p.primary, 0.05));
        assert_eq!(border, color_alpha(p.primary, 0.3));
        // Nova dark checked: bg-primary/10 with `border-primary/20`.
        let dark_p = palette(Mode::Dark);
        let (fill, border) =
            field_label_card_colors(&dark_p, true, Style::Nova, true, false, false, false);
        assert_eq!(fill, color_alpha(dark_p.primary, 0.10));
        assert_eq!(border, color_alpha(dark_p.primary, 0.2));
        // Nova hover (enabled, unchecked): bg-muted/50; base has no hover fill.
        let (fill, _) = field_label_card_colors(&p, false, Style::Nova, false, true, false, false);
        assert_eq!(fill, color_alpha(p.muted, 0.5));
        let (fill, _) = field_label_card_colors(&p, false, Style::Vega, false, true, false, false);
        assert_eq!(fill, Color32::TRANSPARENT);
        // Hover does not apply while disabled; disabled fades fill+border.
        let (fill, border) =
            field_label_card_colors(&p, false, Style::Nova, false, true, true, false);
        assert_eq!(fill, Color32::TRANSPARENT);
        assert_eq!(border, color_alpha(p.border, 0.5));
        // Disabled checked card fades the checked fill too.
        let (fill, _) = field_label_card_colors(&p, false, Style::Nova, true, false, true, false);
        assert_eq!(fill, color_alpha(color_alpha(p.primary, 0.05), 0.5));
    }

    #[test]
    fn card_focus_inside_recolors_border_nova_only() {
        // N9 (spec §4.5): nova card + focused descendant → border `ring`
        // at full alpha. Base has no focus-inside rule: the border is
        // unaffected by focus. Focus wins over checked for the border
        // (PROVISIONAL, OPEN-3); the checked fill is unaffected.
        let p = palette(Mode::Light);
        let (_, border) =
            field_label_card_colors(&p, false, Style::Nova, false, false, false, true);
        assert_eq!(border, p.ring);
        let (fill, border) =
            field_label_card_colors(&p, false, Style::Nova, true, false, false, true);
        assert_eq!(fill, color_alpha(p.primary, 0.05));
        assert_eq!(border, p.ring, "focus wins over checked (provisional)");
        let (_, border) =
            field_label_card_colors(&p, false, Style::Vega, false, false, false, true);
        assert_eq!(border, p.border, "base paints no focus chrome");
        let (_, border) = field_label_card_colors(&p, false, Style::Vega, true, false, false, true);
        assert_eq!(border, p.primary, "base checked border unaffected by focus");
        // Disabled focus-inside: the ring border halves with the rest of
        // the element (group fade); the rule itself has no disabled guard.
        let (_, border) = field_label_card_colors(&p, false, Style::Nova, false, false, true, true);
        assert_eq!(border, color_alpha(p.ring, 0.5));
        // Ring color: `ring/50`, halved again when disabled (exact group
        // fade over the backdrop — the ring paints outside the border).
        assert_eq!(
            field_label_card_ring_color(&p, false),
            color_alpha(p.ring, 0.5)
        );
        assert_eq!(
            field_label_card_ring_color(&p, true),
            color_alpha(p.ring, 0.25)
        );
    }

    #[test]
    fn label_color_follows_field_state() {
        let p = palette(Mode::Light);
        assert_eq!(field_label_color(&p, false, false), p.foreground);
        assert_eq!(field_label_color(&p, true, false), p.destructive);
        assert_eq!(
            field_label_color(&p, false, true),
            color_alpha(p.foreground, 0.5)
        );
        assert_eq!(
            field_label_color(&p, true, true),
            color_alpha(p.destructive, 0.5)
        );
    }

    #[test]
    fn responsive_orientation_matches_container_width() {
        // REPLACES the pre-S1 test of the same name, which pinned the
        // superseded behaviour (a responsive Field switching on its own
        // available width). The frozen spec §2.4 measures the NEAREST
        // enclosing FieldGroup's inline size, and a responsive Field
        // with no FieldGroup ancestor always renders columnar (no named
        // container exists). The breakpoint is 448px (OPEN-1).
        assert_eq!(FIELD_GROUP_MD_BREAKPOINT, 448.0);
        assert_eq!(
            resolve_responsive_orientation(None),
            FieldOrientation::Vertical,
            "no container: always columnar"
        );
        assert_eq!(
            resolve_responsive_orientation(Some(300.0)),
            FieldOrientation::Vertical
        );
        assert_eq!(
            resolve_responsive_orientation(Some(447.9)),
            FieldOrientation::Vertical
        );
        assert_eq!(
            resolve_responsive_orientation(Some(448.0)),
            FieldOrientation::Horizontal
        );
        assert_eq!(
            resolve_responsive_orientation(Some(600.0)),
            FieldOrientation::Horizontal
        );

        // End-to-end through the real widgets: the container width is
        // the enclosing group's, not the field's own available width,
        // and the nearest group governs when nested.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut no_group_wide = None;
        let mut group_wide = None;
        let mut group_narrow = None;
        let mut nested_nearest = None;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            // No group ancestor, 600px available: columnar.
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_resp_no_group"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(600.0, 400.0),
                )),
            );
            Field::new()
                .orientation(FieldOrientation::Responsive)
                .show(ui, |ui| {
                    no_group_wide = Some(ui.layout().is_vertical());
                });
            // Wide group: horizontal.
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_resp_wide"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(600.0, 400.0),
                )),
            );
            FieldGroup::new().show(ui, |ui| {
                Field::new()
                    .orientation(FieldOrientation::Responsive)
                    .show(ui, |ui| {
                        group_wide = Some(ui.layout().is_vertical());
                    });
            });
            // Narrow group: columnar.
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_resp_narrow"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(300.0, 400.0),
                )),
            );
            FieldGroup::new().show(ui, |ui| {
                Field::new()
                    .orientation(FieldOrientation::Responsive)
                    .show(ui, |ui| {
                        group_narrow = Some(ui.layout().is_vertical());
                    });
            });
            // Nested groups: the NEAREST group's width governs. The
            // outer group is 600px wide; the inner group is constrained
            // to 300px, so the field inside it stays columnar.
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_resp_nested"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(600.0, 400.0),
                )),
            );
            FieldGroup::new().show(ui, |ui| {
                ui.allocate_ui(Vec2::new(300.0, 400.0), |ui| {
                    FieldGroup::new().show(ui, |ui| {
                        Field::new()
                            .orientation(FieldOrientation::Responsive)
                            .show(ui, |ui| {
                                nested_nearest = Some(ui.layout().is_vertical());
                            });
                    });
                });
            });
        });
        out.textures_delta.clear();
        assert_eq!(no_group_wide, Some(true));
        assert_eq!(group_wide, Some(false));
        assert_eq!(group_narrow, Some(true));
        assert_eq!(nested_nearest, Some(true));
    }

    #[test]
    fn field_error_renders_nothing_when_empty() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut rect = egui::Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_error_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(400.0, 400.0),
                )),
            );
            rect = FieldError::errors(vec![]).show(ui).rect;
        });
        out.textures_delta.clear();
        assert!(rect.size() == Vec2::ZERO);
    }

    #[test]
    fn field_label_card_renders_around_content() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let checked = true;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_card_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(400.0, 400.0),
                )),
            );
            FieldLabel::new("")
                .checked(Some(checked))
                .show_with_card(ui, |ui| {
                    Field::new().show(ui, |ui| {
                        ui.label("wrapped");
                    });
                });
        });
        out.textures_delta.clear();
    }

    #[test]
    fn builders_keep_fields() {
        let f = Field::new()
            .orientation(FieldOrientation::Horizontal)
            .invalid(true)
            .disabled(true)
            .has_content_child(true)
            .id_salt("f");
        assert_eq!(f.orientation, FieldOrientation::Horizontal);
        assert!(f.invalid);
        assert!(f.disabled);
        assert!(f.has_content_child);
        assert_eq!(f.id_salt.as_deref(), Some("f"));

        let l = FieldLabel::new("Email")
            .for_id(Id::unique("email"))
            .disabled(true)
            .checked(Some(true))
            .id_salt("l");
        assert_eq!(l.for_id, Some(Id::unique("email")));
        assert!(l.disabled);
        assert_eq!(l.checked, Some(true));

        let t = FieldTitle::new("Name")
            .icon(crate::icons::Icon::User)
            .id_salt("t");
        assert!(t.icon.is_some());

        let d = FieldDescription::new("Help")
            .after_legend(true)
            .second_from_last(true)
            .id_salt("d");
        assert!(d.after_legend);
        assert!(d.second_from_last);
        assert_eq!(d.id_salt.as_deref(), Some("d"));

        let e = FieldError::new("bad").id_salt("e");
        assert_eq!(e.id_salt.as_deref(), Some("e"));
        let el = FieldError::entries(vec![Some("a".to_owned()), None]);
        assert_eq!(el.errors.len(), 2);
        assert_eq!(el.errors[1], None);

        let legend = FieldLegend::new("Billing")
            .variant(FieldLegendVariant::Label)
            .id_salt("legend");
        assert_eq!(legend.variant, FieldLegendVariant::Label);

        let sep = FieldSeparator::content("or").id_salt("sep");
        assert_eq!(sep.content.as_deref(), Some("or"));

        let g = FieldGroup::new().checkbox_group(true);
        assert!(g.checkbox_group);
        let s = FieldSet::new().has_group_child(true);
        assert!(s.has_group_child);
    }

    #[test]
    fn error_dedup_follows_map_semantics() {
        // N13 (`field.tsx:203-205`): dedup key is the message (or None);
        // later entries replace the stored entry but keep the FIRST
        // occurrence's position; None entries collapse to one.
        let some = |s: &str| Some(s.to_owned());
        assert!(dedupe_error_entries(&[]).is_empty());
        assert_eq!(
            dedupe_error_entries(&[some("a"), some("a"), some("b")]),
            vec![some("a"), some("b")]
        );
        assert_eq!(
            dedupe_error_entries(&[some("b"), some("a"), some("b")]),
            vec![some("b"), some("a")],
            "first-occurrence order kept"
        );
        assert_eq!(
            dedupe_error_entries(&[None, None, some("a")]),
            vec![None, some("a")],
            "absent messages collapse under one key"
        );
        assert_eq!(
            dedupe_error_entries(&[some(""), some(""), some("a")]),
            vec![some(""), some("a")],
            "empty messages are one key"
        );
        assert_eq!(dedupe_error_entries(&[some("a")]), vec![some("a")]);
    }

    #[test]
    fn error_render_plan_forms_follow_the_algorithm() {
        // N13 (`field.tsx:199-223`): the form decision uses the DEDUPED
        // count; falsy messages render nothing in either form.
        let some = |s: &str| Some(s.to_owned());
        assert_eq!(error_render_plan(&[]), (false, vec![]));
        assert_eq!(
            error_render_plan(&[some("a")]),
            (false, vec!["a".to_owned()]),
            "single unique entry: bare text"
        );
        assert_eq!(
            error_render_plan(&[some("a"), some("a")]),
            (false, vec!["a".to_owned()]),
            "duplicates collapse to bare text"
        );
        assert_eq!(
            error_render_plan(&[some("a"), some("b")]),
            (true, vec!["a".to_owned(), "b".to_owned()]),
            "two unique entries: list form"
        );
        assert_eq!(
            error_render_plan(&[some(""), some("a")]),
            (true, vec!["a".to_owned()]),
            "falsy + truthy: list form with one visible bullet"
        );
        assert_eq!(
            error_render_plan(&[some(""), some("")]),
            (false, vec![]),
            "sole falsy entry renders nothing"
        );
        assert_eq!(
            error_render_plan(&[None, None]),
            (false, vec![]),
            "sole absent entry renders nothing"
        );
        assert_eq!(
            error_render_plan(&[None, some("a"), some("b")]),
            (true, vec!["a".to_owned(), "b".to_owned()]),
            "absent entry counts toward the list form but renders nothing"
        );
    }

    #[test]
    fn card_click_detection_uses_release_position() {
        // F2: the predicate's only position input is the release
        // position — `press_origin` plays no part (it is dead in
        // egui 0.37). A click on the target itself never forwards.
        let outer = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(100.0, 50.0));
        assert!(card_click_detected(
            false,
            true,
            Some(Pos2::new(50.0, 30.0)),
            outer
        ));
        assert!(
            !card_click_detected(false, true, Some(Pos2::new(200.0, 30.0)), outer),
            "release outside: no forward"
        );
        assert!(
            !card_click_detected(false, true, None, outer),
            "no release position: no forward"
        );
        assert!(
            !card_click_detected(false, false, Some(Pos2::new(50.0, 30.0)), outer),
            "no click event: no forward"
        );
        assert!(
            !card_click_detected(true, true, Some(Pos2::new(50.0, 30.0)), outer),
            "click on the target itself: no re-forward (exactly once)"
        );
    }

    // --- Harness tests: click forwarding and geometry through real
    // `run_ui` passes (the label.rs S9 pattern: a fixed Ui id and rect
    // so widget ids are stable across passes). These pin the wiring —
    // the deferred click, the ctx propagation, the negative margins;
    // runtime behaviour verdicts belong to S6/S10.

    fn test_ctx() -> egui::Context {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        ctx
    }

    fn test_ctx_base() -> egui::Context {
        let ctx = egui::Context::default();
        Preset::default().with_style(Style::Vega).install(&ctx);
        ctx
    }

    fn run_ui_capture(
        ctx: &egui::Context,
        input: egui::RawInput,
        mut f: impl FnMut(&mut Ui),
    ) -> egui::FullOutput {
        let mut output = ctx.run_ui(input, |ctx| {
            let mut ui = Ui::new(
                ctx.clone(),
                Id::unique("field_s1_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 800.0))),
            );
            f(&mut ui);
        });
        // No renderer consumes the texture uploads in a headless test;
        // dropping an unapplied delta panics, so clear it explicitly.
        output.textures_delta.clear();
        output
    }

    fn run_ui(ctx: &egui::Context, input: egui::RawInput, f: impl FnMut(&mut Ui)) {
        let _ = run_ui_capture(ctx, input, f);
    }

    fn click_event(pos: Pos2, pressed: bool) -> egui::RawInput {
        egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn plain_label_click_focuses_and_toggles_checkbox() {
        use std::cell::{Cell, RefCell};
        let ctx = test_ctx();
        let checked = RefCell::new(false);
        let label_rect = Cell::new(Rect::NOTHING);
        let target = Cell::new(Id::unique("field_click_placeholder"));
        // The checkbox's real id is captured from its response (the
        // scoped-id rule) on the first pass and used as the label's
        // `for_id` from then on.
        let render = |ui: &mut Ui| {
            Field::new().show(ui, |ui| {
                label_rect.set(FieldLabel::new("Accept").for_id(target.get()).show(ui).rect);
                target.set(
                    Checkbox::new(&mut checked.borrow_mut())
                        .id_salt("field_click_target")
                        .show(ui)
                        .id,
                );
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert!(label_rect.get().width() > 0.0);
        let at = label_rect.get().center();
        run_ui(&ctx, click_event(at, true), &render);
        run_ui(&ctx, click_event(at, false), &render);
        assert!(
            !*checked.borrow(),
            "the click pass only records the forward; nothing fires yet"
        );
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert!(
            *checked.borrow(),
            "the deferred forward toggles the target on the next pass"
        );
        assert_eq!(
            ctx.memory(|mem| mem.focused()),
            Some(target.get()),
            "the forward focuses the target, not the label"
        );
    }

    #[test]
    fn card_click_forwards_and_direct_click_fires_once() {
        use std::cell::{Cell, RefCell};
        let ctx = test_ctx();
        let checked = RefCell::new(false);
        let card_rect = Cell::new(Rect::NOTHING);
        let box_rect = Cell::new(Rect::NOTHING);
        let target = Cell::new(Id::unique("field_card_placeholder"));
        let render = |ui: &mut Ui| {
            Field::new().show(ui, |ui| {
                let frame = FieldLabel::new("Accept")
                    .for_id(target.get())
                    .show_with_card(ui, |ui| {
                        Checkbox::new(&mut checked.borrow_mut())
                            .id_salt("field_card_target")
                            .show(ui)
                    });
                card_rect.set(frame.response.rect);
                box_rect.set(frame.inner.rect);
                target.set(frame.inner.id);
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &render);
        let card = card_rect.get();
        let click_at = card.center();
        assert!(
            card.contains(click_at) && !box_rect.get().contains(click_at),
            "the card-body click point must miss the nested control"
        );
        // Click the card body (not the control): forwards exactly once.
        run_ui(&ctx, click_event(click_at, true), &render);
        run_ui(&ctx, click_event(click_at, false), &render);
        assert!(!*checked.borrow());
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert!(
            *checked.borrow(),
            "a card-body click forwards to the nested control"
        );
        assert_eq!(ctx.memory(|mem| mem.focused()), Some(target.get()));
        // Click directly on the control: fires exactly once, and the
        // card does not re-forward (the `interaction_snapshot`
        // exclusion — Radix's `isClickingControl`).
        let on_box = box_rect.get().center();
        run_ui(&ctx, click_event(on_box, true), &render);
        run_ui(&ctx, click_event(on_box, false), &render);
        assert!(
            !*checked.borrow(),
            "the direct click toggles the control once"
        );
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert!(
            !*checked.borrow(),
            "no re-forward follows a direct click on the control"
        );
    }

    #[test]
    fn disabled_distinction_plain_label() {
        use std::cell::{Cell, RefCell};
        // Spec §4.2: the label's own `.disabled()` is the
        // pointer-events-none condition (clicks pass through); the
        // enclosing Field's disabled flag only fades (clicks still
        // forward). The checkbox itself stays enabled in all cases —
        // the fieldset does not propagate disabled (OPEN-8).
        for (own_disabled, group_disabled, expect_forward) in [
            (false, false, true),
            (true, false, false),
            (false, true, true),
            (true, true, false),
        ] {
            let ctx = test_ctx();
            let checked = RefCell::new(false);
            let label_rect = Cell::new(Rect::NOTHING);
            let target = Cell::new(Id::unique("field_dis_placeholder"));
            let render = |ui: &mut Ui| {
                Field::new().disabled(group_disabled).show(ui, |ui| {
                    label_rect.set(
                        FieldLabel::new("Accept")
                            .for_id(target.get())
                            .disabled(own_disabled)
                            .show(ui)
                            .rect,
                    );
                    target.set(
                        Checkbox::new(&mut checked.borrow_mut())
                            .id_salt("field_dis_target")
                            .show(ui)
                            .id,
                    );
                });
            };
            run_ui(&ctx, egui::RawInput::default(), &render);
            let at = label_rect.get().center();
            run_ui(&ctx, click_event(at, true), &render);
            run_ui(&ctx, click_event(at, false), &render);
            run_ui(&ctx, egui::RawInput::default(), &render);
            assert_eq!(
                *checked.borrow(),
                expect_forward,
                "own_disabled={own_disabled} group_disabled={group_disabled}"
            );
        }
    }

    #[test]
    fn disabled_distinction_card_label() {
        use std::cell::{Cell, RefCell};
        // The same two-condition distinction, for the card composition.
        for (own_disabled, group_disabled, expect_forward) in [
            (false, false, true),
            (true, false, false),
            (false, true, true),
            (true, true, false),
        ] {
            let ctx = test_ctx();
            let checked = RefCell::new(false);
            let card_rect = Cell::new(Rect::NOTHING);
            let box_rect = Cell::new(Rect::NOTHING);
            let target = Cell::new(Id::unique("field_card_dis_placeholder"));
            let render = |ui: &mut Ui| {
                Field::new().disabled(group_disabled).show(ui, |ui| {
                    let frame = FieldLabel::new("Accept")
                        .for_id(target.get())
                        .disabled(own_disabled)
                        .show_with_card(ui, |ui| {
                            Checkbox::new(&mut checked.borrow_mut())
                                .id_salt("field_card_dis_target")
                                .show(ui)
                        });
                    card_rect.set(frame.response.rect);
                    box_rect.set(frame.inner.rect);
                    target.set(frame.inner.id);
                });
            };
            run_ui(&ctx, egui::RawInput::default(), &render);
            let click_at = card_rect.get().center();
            assert!(!box_rect.get().contains(click_at));
            run_ui(&ctx, click_event(click_at, true), &render);
            run_ui(&ctx, click_event(click_at, false), &render);
            run_ui(&ctx, egui::RawInput::default(), &render);
            assert_eq!(
                *checked.borrow(),
                expect_forward,
                "card: own_disabled={own_disabled} group_disabled={group_disabled}"
            );
        }
    }

    #[test]
    fn focused_descendant_paints_nova_card_ring_and_border() {
        use std::cell::Cell;
        // N9 (spec §4.5): a focused descendant inside a nova card paints
        // the 3px `ring-ring/50` halo on the card rect and recolors the
        // card border to `ring`. Judged from the emitted shapes: the
        // ring and border strokes sit exactly on the card's outer rect,
        // which distinguishes them from the nested control's own ring.
        let ctx = test_ctx();
        let mut checked = false;
        let card_rect = Cell::new(Rect::NOTHING);
        let target = Cell::new(Id::unique("field_ring_placeholder"));
        let mut render = |ui: &mut Ui| {
            Field::new().show(ui, |ui| {
                let frame = FieldLabel::new("Accept")
                    .for_id(target.get())
                    .show_with_card(ui, |ui| {
                        Checkbox::new(&mut checked)
                            .id_salt("field_ring_target")
                            .show(ui)
                    });
                card_rect.set(frame.response.rect);
                target.set(frame.inner.id);
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        ctx.memory_mut(|mem| mem.request_focus(target.get()));
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        let output = run_ui_capture(&ctx, egui::RawInput::default(), &mut render);
        let outer = card_rect.get();
        let p = palette(Mode::Light);
        let ring_color = color_alpha(p.ring, 0.5);
        let mut saw_ring = false;
        let mut saw_border = false;
        for clipped in &output.shapes {
            if let egui::Shape::Rect(rect_shape) = &clipped.shape {
                if rect_shape.rect == outer {
                    if rect_shape.stroke.width == 3.0 && rect_shape.stroke.color == ring_color {
                        saw_ring = true;
                    }
                    if rect_shape.stroke.width == 1.0 && rect_shape.stroke.color == p.ring {
                        saw_border = true;
                    }
                }
            }
        }
        assert!(
            saw_ring,
            "nova card ring paints for a focused descendant (outer={outer:?})"
        );
        assert!(
            saw_border,
            "nova card border recolors to ring for a focused descendant"
        );
    }

    #[test]
    fn field_content_fills_the_row() {
        use std::cell::Cell;
        // `flex-1`: in a horizontal Field, the content column takes the
        // remaining row width — its right edge meets the row's right
        // edge (spec §3.2; the split with the label slot is not judged).
        let ctx = test_ctx();
        let content_rect = Cell::new(Rect::NOTHING);
        let row_width = Cell::new(0.0f32);
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            row_width.set(ui.available_width());
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .has_content_child(true)
                .show(ui, |ui| {
                    FieldLabel::new("Name").show(ui);
                    let inner = FieldContent::new().show(ui, |ui| {
                        ui.allocate_space(Vec2::new(10.0, 10.0));
                    });
                    content_rect.set(inner.response.rect);
                });
        });
        assert!(
            (content_rect.get().max.x - row_width.get()).abs() < 1.0,
            "content fills to the row edge: {:?} vs width {}",
            content_rect.get(),
            row_width.get()
        );
    }

    #[test]
    fn nested_and_conditional_gaps_measure_16_and_12() {
        use std::cell::Cell;
        // N14 through the real widgets: a nested group measures 16px, a
        // checkbox-group group and a set with a group child measure
        // 12px, and a default nova set measures 16px (the control).
        let ctx = test_ctx();
        let first = Cell::new(Rect::NOTHING);
        let second = Cell::new(Rect::NOTHING);
        let render_pair = |ui: &mut Ui| {
            first.set(
                Field::new()
                    .show(ui, |ui| {
                        FieldLabel::new("A").show(ui);
                    })
                    .response
                    .rect,
            );
            second.set(
                Field::new()
                    .show(ui, |ui| {
                        FieldLabel::new("B").show(ui);
                    })
                    .response
                    .rect,
            );
        };
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            FieldGroup::new().show(ui, |ui| {
                FieldGroup::new().show(ui, |ui| render_pair(ui));
            });
        });
        let gap = second.get().min.y - first.get().max.y;
        assert!((gap - 16.0).abs() < 0.5, "nested group gap: {gap}");
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            FieldGroup::new()
                .checkbox_group(true)
                .show(ui, |ui| render_pair(ui));
        });
        let gap = second.get().min.y - first.get().max.y;
        assert!((gap - 12.0).abs() < 0.5, "checkbox-group gap: {gap}");
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            FieldSet::new()
                .has_group_child(true)
                .show(ui, |ui| render_pair(ui));
        });
        let gap = second.get().min.y - first.get().max.y;
        assert!((gap - 12.0).abs() < 0.5, "set-with-group-child gap: {gap}");
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            FieldSet::new().show(ui, |ui| render_pair(ui));
        });
        let gap = second.get().min.y - first.get().max.y;
        assert!((gap - 16.0).abs() < 0.5, "default nova set gap: {gap}");
    }

    #[test]
    fn description_margins_pull_up_by_6_and_4() {
        use std::cell::Cell;
        // Spec §2.8, measured differentially against the default gap so
        // the legend's own bottom margin and the group gap cancel out:
        // `after_legend` pulls the description up 6px, `second_from_last`
        // pulls it up 4px.
        let ctx = test_ctx();
        let legend_rect = Cell::new(Rect::NOTHING);
        let desc_rect = Cell::new(Rect::NOTHING);
        let measure = |after_legend: bool, second_from_last: bool| -> f32 {
            run_ui(&ctx, egui::RawInput::default(), |ui| {
                FieldGroup::new().show(ui, |ui| {
                    legend_rect.set(FieldLegend::new("Billing").show(ui).rect);
                    desc_rect.set(
                        FieldDescription::new("Help text")
                            .after_legend(after_legend)
                            .second_from_last(second_from_last)
                            .show(ui)
                            .rect,
                    );
                });
            });
            desc_rect.get().min.y - legend_rect.get().max.y
        };
        let base = measure(false, false);
        let after = measure(true, false);
        let second = measure(false, true);
        assert!(
            (base - after - 6.0).abs() < 0.5,
            "after_legend pulls up 6px: base={base} after={after}"
        );
        assert!(
            (base - second - 4.0).abs() < 0.5,
            "second_from_last pulls up 4px: base={base} second={second}"
        );
    }

    #[test]
    fn separator_negative_margins_overlap_8px_each_side() {
        use std::cell::Cell;
        // `-my-2` (spec §2.9): the 20px row overlaps 8px into each
        // neighbour's space, leaving 12px of the 20px nova group gap on
        // each side.
        let ctx = test_ctx();
        let first = Cell::new(Rect::NOTHING);
        let sep = Cell::new(Rect::NOTHING);
        let second = Cell::new(Rect::NOTHING);
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            FieldGroup::new().show(ui, |ui| {
                first.set(FieldLabel::new("A").show(ui).rect);
                sep.set(FieldSeparator::new().show(ui).rect);
                second.set(FieldLabel::new("B").show(ui).rect);
            });
        });
        let above = sep.get().min.y - first.get().max.y;
        let below = second.get().min.y - sep.get().max.y;
        assert!((above - 12.0).abs() < 0.5, "gap above separator: {above}");
        assert!((below - 12.0).abs() < 0.5, "gap below separator: {below}");
        assert!(
            (sep.get().height() - 20.0).abs() < 0.5,
            "separator row height: {}",
            sep.get().height()
        );
    }

    #[test]
    fn horizontal_content_flag_scopes_to_direct_children() {
        use std::cell::Cell;
        // OPEN-6: the `mt-px` condition reads true only for direct
        // children of a horizontal Field with a content child, and is
        // reset by every other container (FieldContent, FieldGroup,
        // FieldSet) and by vertical/no-content fields.
        let ctx = test_ctx();
        let direct = Cell::new(false);
        let in_content = Cell::new(true);
        let in_group = Cell::new(true);
        let in_set = Cell::new(true);
        let no_content = Cell::new(true);
        let vertical = Cell::new(true);
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .has_content_child(true)
                .show(ui, |ui| {
                    direct.set(field_horizontal_content_present(ui));
                    FieldContent::new().show(ui, |ui| {
                        in_content.set(field_horizontal_content_present(ui));
                    });
                    FieldGroup::new().show(ui, |ui| {
                        in_group.set(field_horizontal_content_present(ui));
                    });
                    FieldSet::new().show(ui, |ui| {
                        in_set.set(field_horizontal_content_present(ui));
                    });
                });
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    no_content.set(field_horizontal_content_present(ui));
                });
            Field::new().show(ui, |ui| {
                vertical.set(field_horizontal_content_present(ui));
            });
        });
        assert!(direct.get(), "direct child of a horizontal content field");
        assert!(!in_content.get(), "flag resets inside FieldContent");
        assert!(!in_group.get(), "flag resets inside FieldGroup");
        assert!(!in_set.get(), "flag resets inside FieldSet");
        assert!(!no_content.get(), "no content child: no flag");
        assert!(!vertical.get(), "vertical field: no flag");
    }

    #[test]
    fn description_paints_muted_foreground_never_white() {
        // FLD-VIS-01 closing evidence at S1 level: the description's
        // text shape carries `muted-foreground` as the fallback color
        // and every layout section uses PLACEHOLDER, so the fallback —
        // not a baked-in WHITE — decides the rendered color (egui's
        // `Painter::galley` contract: only non-placeholder colors in
        // the galley take precedence over the fallback). Runtime pixel
        // confirmation belongs to S5/S10.
        let ctx = test_ctx();
        let output = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            FieldDescription::new("Helper text").show(ui);
        });
        let p = palette(Mode::Light);
        let mut saw_description = false;
        for clipped in &output.shapes {
            if let egui::Shape::Text(text_shape) = &clipped.shape {
                if text_shape.galley.job.text == "Helper text" {
                    saw_description = true;
                    assert_eq!(
                        text_shape.fallback_color, p.muted_foreground,
                        "description paints muted-foreground in light mode"
                    );
                    assert!(
                        !text_shape.galley.job.sections.is_empty()
                            && text_shape
                                .galley
                                .job
                                .sections
                                .iter()
                                .all(|s| { s.format.color == Color32::PLACEHOLDER }),
                        "no real color is baked into the galley (FLD-VIS-01)"
                    );
                }
            }
        }
        assert!(saw_description, "the description text shape was emitted");
    }

    #[test]
    fn base_card_paints_no_focus_ring_or_recolor() {
        use std::cell::Cell;
        // N9 is nova-only: under the base recipe a focused descendant
        // paints no card ring and leaves the border at `border`
        // (unchecked) — the control paints its own ring.
        let ctx = test_ctx_base();
        let mut checked = false;
        let card_rect = Cell::new(Rect::NOTHING);
        let target = Cell::new(Id::unique("field_base_ring_placeholder"));
        let mut render = |ui: &mut Ui| {
            Field::new().show(ui, |ui| {
                let frame = FieldLabel::new("Accept")
                    .for_id(target.get())
                    .show_with_card(ui, |ui| {
                        Checkbox::new(&mut checked)
                            .id_salt("field_base_ring_target")
                            .show(ui)
                    });
                card_rect.set(frame.response.rect);
                target.set(frame.inner.id);
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        ctx.memory_mut(|mem| mem.request_focus(target.get()));
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        let output = run_ui_capture(&ctx, egui::RawInput::default(), &mut render);
        let outer = card_rect.get();
        let p = Preset::default()
            .with_style(Style::Vega)
            .build(Mode::Light)
            .palette;
        for clipped in &output.shapes {
            if let egui::Shape::Rect(rect_shape) = &clipped.shape {
                if rect_shape.rect == outer {
                    assert_ne!(
                        rect_shape.stroke.width, 3.0,
                        "base card paints no 3px focus ring"
                    );
                    if rect_shape.stroke.width == 1.0 {
                        assert_eq!(
                            rect_shape.stroke.color, p.border,
                            "base card border is not recolored by focus"
                        );
                    }
                }
            }
        }
        // The focus request itself landed (else the test is vacuous).
        assert_eq!(ctx.memory(|mem| mem.focused()), Some(target.get()));
    }

    // --- FLD-VIS-02 pins: nested Field inherits outer disabled/invalid
    // flags (ANY-ancestor semantics, spec §4.2/§4.4). The title/label
    // text shape carries the paint color in its galley section (the
    // widgets lay out with the paint color directly), so the emitted
    // shape proves which flags the nested widget observed.

    fn title_text_color(output: &egui::FullOutput, text: &str) -> Option<Color32> {
        for clipped in &output.shapes {
            if let egui::Shape::Text(text_shape) = &clipped.shape {
                if text_shape.galley.job.text == text {
                    return text_shape
                        .galley
                        .job
                        .sections
                        .first()
                        .map(|s| s.format.color);
                }
            }
        }
        None
    }

    #[test]
    fn nested_field_inherits_outer_disabled_flag() {
        // FLD-VIS-02 (disabled half): outer disabled Field -> inner
        // plain Field -> FieldTitle renders at 50% opacity. Before the
        // fix the inner Field shadowed the outer flag and the title
        // painted full foreground.
        let ctx = test_ctx();
        let output = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            Field::new().disabled(true).show(ui, |ui| {
                Field::new().show(ui, |ui| {
                    FieldTitle::new("Inner title").show(ui);
                });
            });
        });
        let p = palette(Mode::Light);
        assert_eq!(
            title_text_color(&output, "Inner title"),
            Some(color_alpha(p.foreground, 0.5)),
            "nested title fades under a disabled outer Field"
        );
    }

    #[test]
    fn nested_field_inherits_outer_invalid_flag() {
        // FLD-VIS-02 (invalid half): outer invalid Field -> inner plain
        // Field -> FieldTitle renders destructive. Before the fix the
        // inner Field shadowed the outer flag and the title painted
        // foreground.
        let ctx = test_ctx();
        let output = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            Field::new().invalid(true).show(ui, |ui| {
                Field::new().show(ui, |ui| {
                    FieldTitle::new("Inner title").show(ui);
                });
            });
        });
        let p = palette(Mode::Light);
        assert_eq!(
            title_text_color(&output, "Inner title"),
            Some(p.destructive),
            "nested title is destructive under an invalid outer Field"
        );
    }

    #[test]
    fn nested_field_inherits_combined_flags_for_label() {
        // The same inheritance through FieldLabel, both flags at once:
        // outer disabled + invalid -> inner plain Field -> label paints
        // destructive at 50%. Also pins that the label path observes the
        // inherited flags, not just the title path.
        let ctx = test_ctx();
        let output = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            Field::new().disabled(true).invalid(true).show(ui, |ui| {
                Field::new().show(ui, |ui| {
                    FieldLabel::new("Inner label").show(ui);
                });
            });
        });
        let p = palette(Mode::Light);
        assert_eq!(
            title_text_color(&output, "Inner label"),
            Some(color_alpha(p.destructive, 0.5)),
            "nested label is faded destructive under a disabled+invalid outer Field"
        );
    }

    #[test]
    fn nested_field_inherits_through_group_and_content() {
        // The §8 card shape: outer disabled Field -> FieldGroup ->
        // inner plain Field -> FieldContent -> FieldTitle. Groups and
        // content already pass flags through; the inner Field was the
        // only shadowing site.
        let ctx = test_ctx();
        let output = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            Field::new().disabled(true).show(ui, |ui| {
                FieldGroup::new().show(ui, |ui| {
                    Field::new().show(ui, |ui| {
                        FieldContent::new().show(ui, |ui| {
                            FieldTitle::new("Inner title").show(ui);
                        });
                    });
                });
            });
        });
        let p = palette(Mode::Light);
        assert_eq!(
            title_text_color(&output, "Inner title"),
            Some(color_alpha(p.foreground, 0.5)),
            "nested title fades through group + content boundaries"
        );
    }

    #[test]
    fn nested_field_own_flags_and_plain_cases_unchanged() {
        // Guards: the fix OR-accumulates, so (a) an inner Field's own
        // flags still apply with a plain outer Field, (b) outer +
        // inner both declaring a flag fades exactly once (no double
        // fade — the flag is a bool, not an alpha), (c) plain outer +
        // plain inner stays full opacity.
        let p = palette(Mode::Light);
        for (outer_disabled, inner_disabled, expected) in [
            (false, true, color_alpha(p.foreground, 0.5)),
            (true, true, color_alpha(p.foreground, 0.5)),
            (false, false, p.foreground),
        ] {
            let ctx = test_ctx();
            let output = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
                Field::new().disabled(outer_disabled).show(ui, |ui| {
                    Field::new().disabled(inner_disabled).show(ui, |ui| {
                        FieldTitle::new("Inner title").show(ui);
                    });
                });
            });
            assert_eq!(
                title_text_color(&output, "Inner title"),
                Some(expected),
                "outer_disabled={outer_disabled} inner_disabled={inner_disabled}"
            );
        }
    }

    #[test]
    fn nested_inherited_disabled_still_forwards_clicks() {
        // Guard for the G1-item-3 distinction: an INHERITED field flag
        // is the opacity-only condition — clicks still forward. Only
        // the label's OWN `.disabled()` suppresses forwarding. This
        // must hold before and after the fix (forwarding never read
        // the ctx flag); it pins that the inheritance change cannot
        // leak into the pointer-events-none condition.
        use std::cell::{Cell, RefCell};
        for (own_disabled, expect_forward) in [(false, true), (true, false)] {
            let ctx = test_ctx();
            let checked = RefCell::new(false);
            let label_rect = Cell::new(Rect::NOTHING);
            let target = Cell::new(Id::unique("field_nested_dis_placeholder"));
            let render = |ui: &mut Ui| {
                Field::new().disabled(true).show(ui, |ui| {
                    Field::new().show(ui, |ui| {
                        label_rect.set(
                            FieldLabel::new("Accept")
                                .for_id(target.get())
                                .disabled(own_disabled)
                                .show(ui)
                                .rect,
                        );
                        target.set(
                            Checkbox::new(&mut checked.borrow_mut())
                                .id_salt("field_nested_dis_target")
                                .show(ui)
                                .id,
                        );
                    });
                });
            };
            run_ui(&ctx, egui::RawInput::default(), &render);
            let at = label_rect.get().center();
            run_ui(&ctx, click_event(at, true), &render);
            run_ui(&ctx, click_event(at, false), &render);
            run_ui(&ctx, egui::RawInput::default(), &render);
            assert_eq!(
                *checked.borrow(),
                expect_forward,
                "own_disabled={own_disabled} under an inherited disabled flag"
            );
        }
    }
}
