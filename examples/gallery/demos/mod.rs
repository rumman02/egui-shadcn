//! One demo module per ported component.

pub mod accordion_demo;
pub mod alert_dialog_demo;
pub mod chart_demo;
pub mod data_table_demo;
pub mod dialog_demo;
pub mod drawer_demo;
pub mod form_demo;
pub mod hover_card_demo;
pub mod icon_demo;
pub mod motion_demo;
pub mod popover_demo;
pub mod select_demo;
pub mod shadow_demo;
pub mod sheet_demo;
pub mod sidebar_demo;
pub mod sonner_demo;
pub mod tooltip_demo;

use egui_shadcn_ui::prelude::*;

/// Demo-local state for the button activation probe and live toggles.
/// Held in egui temp data (the gallery's established demo-state pattern),
/// so the stateless `button_demo` function can keep counters across frames.
#[derive(Clone)]
struct ButtonProbeState {
    /// Firings classified as pointer clicks.
    clicks: u32,
    /// Firings on a frame where Enter was pressed (keydown fire).
    enter_activations: u32,
    /// Firings on a frame where Space was pressed or released.
    space_activations: u32,
    /// Human-readable path of the most recent firing.
    last_activation: String,
    /// Live state of the disabled-toggle target.
    target_disabled: bool,
    /// Live state of the expanded-toggle target.
    target_expanded: bool,
}

impl Default for ButtonProbeState {
    fn default() -> Self {
        Self {
            clicks: 0,
            enter_activations: 0,
            space_activations: 0,
            last_activation: "none yet".to_owned(),
            target_disabled: false,
            target_expanded: false,
        }
    }
}

pub fn button_demo(ui: &mut egui::Ui) {
    ui.label("Variants");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Default").show(ui);
        Button::new("Secondary")
            .variant(ButtonVariant::Secondary)
            .show(ui);
        Button::new("Destructive")
            .variant(ButtonVariant::Destructive)
            .show(ui);
        Button::new("Outline")
            .variant(ButtonVariant::Outline)
            .show(ui);
        Button::new("Ghost").variant(ButtonVariant::Ghost).show(ui);
        Button::new("Link").variant(ButtonVariant::Link).show(ui);
    });

    ui.add_space(12.0);
    ui.label("Sizes");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Extra small").size(ButtonSize::Xs).show(ui);
        Button::new("Small").size(ButtonSize::Sm).show(ui);
        Button::new("Default").show(ui);
        Button::new("Large").size(ButtonSize::Lg).show(ui);
    });

    ui.add_space(12.0);
    ui.label("All variants × all sizes");
    ui.add_space(4.0);
    egui::Grid::new("button_demo_matrix")
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            ui.label("");
            ui.small("Xs");
            ui.small("Sm");
            ui.small("Default");
            ui.small("Lg");
            ui.small("Icon-xs");
            ui.small("Icon-sm");
            ui.small("Icon");
            ui.small("Icon-lg");
            ui.end_row();
            for (variant, name) in [
                (ButtonVariant::Default, "Default"),
                (ButtonVariant::Destructive, "Destructive"),
                (ButtonVariant::Outline, "Outline"),
                (ButtonVariant::Secondary, "Secondary"),
                (ButtonVariant::Ghost, "Ghost"),
                (ButtonVariant::Link, "Link"),
            ] {
                ui.label(name);
                for size in [
                    ButtonSize::Xs,
                    ButtonSize::Sm,
                    ButtonSize::Default,
                    ButtonSize::Lg,
                ] {
                    Button::new("Button").variant(variant).size(size).show(ui);
                }
                for size in [
                    ButtonSize::IconXs,
                    ButtonSize::IconSm,
                    ButtonSize::Icon,
                    ButtonSize::IconLg,
                ] {
                    Button::new("")
                        .variant(variant)
                        .size(size)
                        .icon_start(Icon::Plus)
                        .show(ui);
                }
                ui.end_row();
            }
        });

    ui.add_space(12.0);
    ui.label("With icons");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("New item").icon_end(Icon::Plus).show(ui);
        Button::new("Continue").icon_end(Icon::ArrowRight).show(ui);
        Button::new("Back").icon_start(Icon::ArrowLeft).show(ui);
        Button::new("Loading")
            .icon_start(Icon::LoaderCircle)
            .disabled(true)
            .show(ui);
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        Button::new("Both sides")
            .icon_start(Icon::Plus)
            .icon_end(Icon::ArrowRight)
            .show(ui);
        Button::new("")
            .icon_start(Icon::ChevronLeft)
            .icon_end(Icon::ChevronRight)
            .show(ui);
        Button::new("Send")
            .icon_start(Icon::Send)
            .icon_end(Icon::Check)
            .show(ui);
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        Button::new("Two icons")
            .size(ButtonSize::Xs)
            .icon_start(Icon::Plus)
            .icon_end(Icon::ArrowRight)
            .show(ui);
        Button::new("Two icons")
            .size(ButtonSize::Sm)
            .icon_start(Icon::Plus)
            .icon_end(Icon::ArrowRight)
            .show(ui);
        Button::new("Two icons")
            .size(ButtonSize::Lg)
            .icon_start(Icon::Plus)
            .icon_end(Icon::ArrowRight)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Explicit icon sizes (explicit size wins over the size default)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Large icon")
            .icon_start_sized(Icon::Search, 20.0)
            .show(ui);
        Button::new("Small icon")
            .icon_end_sized(Icon::ArrowRight, 12.0)
            .show(ui);
        Button::new("Both sized")
            .icon_start_sized(Icon::Plus, 20.0)
            .icon_end_sized(Icon::Check, 12.0)
            .show(ui);
        Button::new("")
            .size(ButtonSize::Icon)
            .icon_start_sized(Icon::Search, 24.0)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Icon-only sizes");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("")
            .size(ButtonSize::IconXs)
            .icon_start(Icon::Plus)
            .show(ui);
        Button::new("")
            .size(ButtonSize::IconSm)
            .icon_start(Icon::Plus)
            .show(ui);
        Button::new("")
            .size(ButtonSize::Icon)
            .icon_start(Icon::Plus)
            .show(ui);
        Button::new("")
            .size(ButtonSize::IconLg)
            .icon_start(Icon::Plus)
            .show(ui);
        Button::new("")
            .size(ButtonSize::Icon)
            .variant(ButtonVariant::Outline)
            .icon_start(Icon::Search)
            .show(ui);
        Button::new("")
            .size(ButtonSize::Icon)
            .variant(ButtonVariant::Ghost)
            .icon_start(Icon::MoreHorizontal)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Icon-only with text sizes (icon + padding, no label)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("")
            .size(ButtonSize::Xs)
            .icon_start(Icon::Plus)
            .show(ui);
        Button::new("")
            .size(ButtonSize::Sm)
            .icon_start(Icon::Plus)
            .show(ui);
        Button::new("").icon_start(Icon::Plus).show(ui);
        Button::new("")
            .size(ButtonSize::Lg)
            .icon_start(Icon::Plus)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Disabled (all variants, 50% opacity, no hover / focus / press)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Disabled").disabled(true).show(ui);
        Button::new("Disabled outline")
            .variant(ButtonVariant::Outline)
            .disabled(true)
            .show(ui);
        Button::new("Disabled secondary")
            .variant(ButtonVariant::Secondary)
            .disabled(true)
            .show(ui);
        Button::new("Disabled destructive")
            .variant(ButtonVariant::Destructive)
            .disabled(true)
            .show(ui);
        Button::new("Disabled ghost")
            .variant(ButtonVariant::Ghost)
            .disabled(true)
            .show(ui);
        Button::new("Disabled link")
            .variant(ButtonVariant::Link)
            .disabled(true)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Invalid, and disabled + invalid (both apply)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Invalid")
            .variant(ButtonVariant::Outline)
            .invalid(true)
            .show(ui);
        Button::new("Invalid default").invalid(true).show(ui);
        Button::new("Invalid ghost")
            .variant(ButtonVariant::Ghost)
            .invalid(true)
            .show(ui);
        Button::new("Disabled + invalid")
            .variant(ButtonVariant::Outline)
            .disabled(true)
            .invalid(true)
            .show(ui);
        Button::new("Disabled + invalid")
            .disabled(true)
            .invalid(true)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Expanded (nova aria-expanded fills: outline / secondary / ghost)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Expanded outline")
            .variant(ButtonVariant::Outline)
            .expanded(true)
            .show(ui);
        Button::new("Expanded secondary")
            .variant(ButtonVariant::Secondary)
            .expanded(true)
            .show(ui);
        Button::new("Expanded ghost")
            .variant(ButtonVariant::Ghost)
            .expanded(true)
            .show(ui);
        Button::new("Expanded default").expanded(true).show(ui);
        Button::new("Expanded destructive")
            .variant(ButtonVariant::Destructive)
            .expanded(true)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Popup triggers (has_popup: pressing shows no 1px shift)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Popup trigger").has_popup(true).show(ui);
        Button::new("Menu trigger")
            .variant(ButtonVariant::Outline)
            .has_popup(true)
            .show(ui);
        Button::new("Plain trigger (shifts on press)").show(ui);
    });

    ui.add_space(12.0);
    ui.label("Long text (never wraps, grows to fit) and link long text");
    ui.add_space(4.0);
    Button::new("This is a very long button label that must not wrap or clip").show(ui);
    ui.add_space(8.0);
    Button::new("A very long link button label, underline spans the full width on hover")
        .variant(ButtonVariant::Link)
        .show(ui);

    ui.add_space(12.0);
    ui.label("Empty label (padding only, no text, no icon)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("").show(ui);
        ui.label("← empty-label button (20px wide under nova)");
    });

    ui.add_space(12.0);
    ui.label("Inside a button group (fused corners; Tab to a member for the ring)");
    ui.add_space(4.0);
    ButtonGroup::new()
        .id_salt("button_demo_group")
        .show(ui, |ui| {
            Button::new("Day").variant(ButtonVariant::Outline).show(ui);
            Button::new("Month")
                .variant(ButtonVariant::Outline)
                .show(ui);
            Button::new("Year").variant(ButtonVariant::Outline).show(ui);
        });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ButtonGroup::new()
            .id_salt("button_demo_group_sm")
            .show(ui, |ui| {
                Button::new("Sm left").size(ButtonSize::Sm).show(ui);
                Button::new("Sm right").size(ButtonSize::Sm).show(ui);
            });
        ButtonGroup::new()
            .id_salt("button_demo_group_xs")
            .show(ui, |ui| {
                Button::new("Xs left").size(ButtonSize::Xs).show(ui);
                Button::new("Xs right").size(ButtonSize::Xs).show(ui);
            });
    });

    ui.add_space(12.0);
    ui.label("Activation probe (counters show which path fired the probe button; watch them while holding a key to see firing timing)");
    ui.add_space(4.0);
    let probe_store = egui::Id::unique("button_demo_probe");
    let mut probe: ButtonProbeState = ui
        .ctx()
        .data_mut(|d| d.get_temp(probe_store).unwrap_or_default());
    // Snapshot this frame's activation events BEFORE showing the button, so
    // classification sees the raw input for the frame the click fires on.
    let (enter_pressed, space_pressed, space_released) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::Space),
            i.key_released(egui::Key::Space),
        )
    });
    ui.horizontal(|ui| {
        let response = Button::new("Probe button").show(ui);
        if response.clicked() {
            // Priority matches the only ways egui reports a click: a key
            // press/release on the focused button, else a pointer release.
            // Space is classified by WHICH event coincided with the firing
            // frame, so a keydown fire and a keyup fire read differently.
            if enter_pressed {
                probe.enter_activations += 1;
                probe.last_activation = "Enter (keydown)".to_owned();
            } else if space_pressed {
                probe.space_activations += 1;
                probe.last_activation = "Space (keydown)".to_owned();
            } else if space_released {
                probe.space_activations += 1;
                probe.last_activation = "Space (keyup)".to_owned();
            } else {
                probe.clicks += 1;
                probe.last_activation = "Pointer click".to_owned();
            }
        }
        ui.label(format!(
            "clicks: {}  Enter: {}  Space: {}",
            probe.clicks, probe.enter_activations, probe.space_activations
        ));
    });
    ui.label(format!("Last activation: {}", probe.last_activation));

    ui.add_space(12.0);
    ui.label("Live toggles (flip a toggle while recording to drive the disabled / expanded transition; the expanded fill shows under nova)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut probe.target_disabled, "Disabled");
        Button::new("Disabled target")
            .disabled(probe.target_disabled)
            .show(ui);
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut probe.target_expanded, "Expanded");
        Button::new("Expanded target")
            .variant(ButtonVariant::Outline)
            .expanded(probe.target_expanded)
            .show(ui);
    });
    ui.ctx().data_mut(|d| d.insert_temp(probe_store, probe));

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a button, press Enter or Space to activate. Disabled buttons are skipped. Space held shows the pressed shift (except has_popup triggers). Hover a link button to see its underline.");
}

pub fn badge_demo(ui: &mut egui::Ui) {
    // Header: recipe position, stated honestly (spec §§0, 3.1, 7).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Badge — 6 variants, one size. Active recipe: {recipe} (style: {}, mode: {mode}).",
        theme.style.name()
    ));
    ui.label("Heights: nova h-5 = 20px fixed; base content-derived 22px with text (O5 derivation: 16px line box + 4px padding + 2px border). Transitions 150ms + cubic-bezier(0.4, 0, 0.2, 1) are O1 interim values, not snapshot-verified. No variant paints a shadow in either recipe (§2.3).");

    // 1. All 6 variants, label-only (spec §2). The row rect is demo chrome:
    // `ui.horizontal` sizes the row to `interact_size.y` (36px); each badge
    // allocates its own exact rect, printed below.
    ui.add_space(12.0);
    ui.label("1. All 6 variants, label-only (spec §2; height 20px nova / 22px base)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        [
            ("Default", Badge::new("Default").show(ui).rect),
            (
                "Secondary",
                Badge::new("Secondary")
                    .variant(BadgeVariant::Secondary)
                    .show(ui)
                    .rect,
            ),
            (
                "Destructive",
                Badge::new("Destructive")
                    .variant(BadgeVariant::Destructive)
                    .show(ui)
                    .rect,
            ),
            (
                "Outline",
                Badge::new("Outline")
                    .variant(BadgeVariant::Outline)
                    .show(ui)
                    .rect,
            ),
            (
                "Ghost",
                Badge::new("Ghost")
                    .variant(BadgeVariant::Ghost)
                    .show(ui)
                    .rect,
            ),
            (
                "Link",
                Badge::new("Link").variant(BadgeVariant::Link).show(ui).rect,
            ),
        ]
    });
    ui.label(format!(
        "row rect: {:.1} x {:.1} px (36px-tall horizontal row: demo layout, not the component)",
        row.response.rect.width(),
        row.response.rect.height()
    ));
    let mut summary = String::from("allocated:");
    for (name, rect) in row.inner {
        summary.push_str(&format!(
            " {name} {:.1}x{:.1};",
            rect.width(),
            rect.height()
        ));
    }
    ui.label(summary);

    // 2. Icons (spec §4). One label ("New") throughout, so the text width T
    // is identical in every case: T = label-only width − 16 (8px padding per
    // side). Expected widths follow the §4 tables for the active recipe.
    ui.add_space(12.0);
    ui.label("2. Icons — 12px icons, 4px gap (spec §4; 6px icon-side padding under nova only)");
    ui.add_space(4.0);
    let label_only = Badge::new("New").show(ui).rect;
    let t = label_only.width() - 16.0;
    ui.label(format!(
        "label-only \"New\": {:.1} x {:.1} px (T + 16, so T = {:.1}px)",
        label_only.width(),
        label_only.height(),
        t
    ));
    let (one_icon, two_icons, icon_only) = if nova {
        (t + 30.0, t + 44.0, 26.0)
    } else {
        (t + 32.0, t + 48.0, 28.0)
    };
    ui.add_space(4.0);
    let r = Badge::new("New").icon_start(Icon::Plus).show(ui).rect;
    ui.label(format!(
        "leading icon: {:.1} x {:.1} px (expected {:.1} under {recipe})",
        r.width(),
        r.height(),
        one_icon
    ));
    ui.add_space(4.0);
    let r = Badge::new("New").icon_end(Icon::X).show(ui).rect;
    ui.label(format!(
        "trailing icon: {:.1} x {:.1} px (expected {:.1} under {recipe})",
        r.width(),
        r.height(),
        one_icon
    ));
    ui.add_space(4.0);
    let r = Badge::new("New")
        .icon_start(Icon::Plus)
        .icon_end(Icon::ArrowRight)
        .show(ui)
        .rect;
    ui.label(format!(
        "two icons: {:.1} x {:.1} px (expected {:.1} under {recipe})",
        r.width(),
        r.height(),
        two_icons
    ));
    ui.add_space(4.0);
    let r = Badge::new("").icon_start(Icon::CircleAlert).show(ui).rect;
    ui.label(format!(
        "icon-only: {:.1} x {:.1} px (expected {:.1} under {recipe}; nova 26px assumes one flagged side — O2)",
        r.width(),
        r.height(),
        icon_only
    ));

    // 3. Long text (spec §10).
    ui.add_space(12.0);
    ui.label("3. Long text — grows to fit, never wraps, no truncation (spec §10)");
    ui.add_space(4.0);
    let r = Badge::new("This is a very long badge label that must not wrap or truncate")
        .show(ui)
        .rect;
    ui.label(format!(
        "allocated: {:.1} x {:.1} px (width = T + 16, single row)",
        r.width(),
        r.height()
    ));

    // 4. Empty badge (spec §10 vs §4).
    ui.add_space(12.0);
    ui.label("4. Empty badge — no label, no icon (spec §10)");
    ui.add_space(4.0);
    let r = Badge::new("").show(ui).rect;
    ui.label(format!(
        "allocated: {:.1} x {:.1} px (§10 states 18px wide, padding + border; the §4 rule extrapolates to 16px — S1 implements §10; S5 confirms)",
        r.width(),
        r.height()
    ));

    // 5. Invalid (spec §5.6).
    ui.add_space(12.0);
    ui.label("5. Invalid — .invalid(true): destructive border, fill/text unchanged, no ring invented (spec §5.6, O6)");
    ui.add_space(4.0);
    let r = Badge::new("Invalid").invalid(true).show(ui).rect;
    ui.label(format!(
        "default + invalid: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Badge::new("Invalid outline")
        .variant(BadgeVariant::Outline)
        .invalid(true)
        .show(ui)
        .rect;
    ui.label(format!(
        "outline + invalid: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 6. Disabled extension (spec §5.5, D4).
    ui.add_space(12.0);
    ui.label("6. Disabled extension — .disabled(true) is a D4 Rust extension; the original defines no disabled state (§5.5), so S5 must not judge it as parity");
    ui.add_space(4.0);
    let r = Badge::new("Disabled").disabled(true).show(ui).rect;
    ui.label(format!(
        "allocated: {:.1} x {:.1} px (fades to 50% opacity, hover suppressed)",
        r.width(),
        r.height()
    ));

    // 7. Hover targets (spec §5.2): the only span badges whose hover changes
    // anything under nova. Generous space around each so S5/S6 hover captures
    // isolate one badge; explicit id salts keep the transition animators
    // anchored. Rect origins are printed for hover targeting.
    ui.add_space(12.0);
    ui.label("7. Hover targets — nova span-hover cases only: ghost (muted fill + muted-foreground text) and link (underline) (spec §5.2)");
    ui.add_space(8.0);
    let r = Badge::new("Ghost hover")
        .variant(BadgeVariant::Ghost)
        .id_salt("demo_hover_ghost")
        .show(ui)
        .rect;
    ui.label(format!(
        "ghost: {:.1} x {:.1} px at ({:.1}, {:.1})",
        r.width(),
        r.height(),
        r.min.x,
        r.min.y
    ));
    ui.add_space(16.0);
    let r = Badge::new("Link hover")
        .variant(BadgeVariant::Link)
        .id_salt("demo_hover_link")
        .show(ui)
        .rect;
    ui.label(format!(
        "link: {:.1} x {:.1} px at ({:.1}, {:.1})",
        r.width(),
        r.height(),
        r.min.x,
        r.min.y
    ));
}

pub fn label_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the label actually allocated
    // (Response.rect), so S5/S6 can verify geometry from screenshots.
    // Rows built with `ui.horizontal` are demo chrome: their 36px row
    // rect is the egui `interact_size` artifact, never the label's
    // rect — the printed rects are always the label's own Response
    // rect. Target controls in section 8 and the nested checkbox in
    // section 9 are demo chrome; only the labels are under test.
    ui.label("Label — one class string, content-sized flex row, gap-2 (spec §2).");
    ui.label("Nova renders identically to base in every established state (LAYER, spec §2.3).");
    ui.add_space(12.0);

    // 1. Text-only (spec §3.1): 14px line box, width = text advance.
    ui.label("1. Text-only — 14px line box (spec §3.1)");
    ui.add_space(4.0);
    let r = Label::new("Email address").show(ui).rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: h=14, w=text advance at 14px/500)",
        r.width(),
        r.height()
    ));

    // 2. Icon + text (spec §3.2): the same text is shown bare first, so
    // the 24px icon+gap delta is measurable from the page itself.
    ui.add_space(12.0);
    ui.label("2. Icon + text — leading icon, gap-2 (spec §3.2)");
    ui.add_space(4.0);
    let bare = Label::new("Profile").show(ui).rect;
    let with = Label::new("Profile").icon(Icon::User).show(ui).rect;
    ui.label(format!(
        "text-only: {:.1} x {:.1} px; icon+text: {:.1} x {:.1} px; delta: {:.1} px (expected: h=16 tallest child, delta=24 = 16 icon + 8 gap)",
        bare.width(),
        bare.height(),
        with.width(),
        with.height(),
        with.width() - bare.width()
    ));

    // 3. Text + icon (spec §3.2): trailing slot via `.icon_end()`.
    ui.add_space(12.0);
    ui.label("3. Text + icon — trailing icon, gap-2 (spec §3.2)");
    ui.add_space(4.0);
    let bare = Label::new("Profile").show(ui).rect;
    let with = Label::new("Profile")
        .icon_end(Icon::ChevronDown)
        .show(ui)
        .rect;
    ui.label(format!(
        "text-only: {:.1} x {:.1} px; text+icon: {:.1} x {:.1} px; delta: {:.1} px (expected: h=16 tallest child, delta=24 = 8 gap + 16 icon)",
        bare.width(),
        bare.height(),
        with.width(),
        with.height(),
        with.width() - bare.width()
    ));

    // 4. Icon + text + icon (spec §3.2): both slots; full §3.2 table
    // covered across sections 1–4.
    ui.add_space(12.0);
    ui.label("4. Icon + text + icon — both slots, gap-2 (spec §3.2)");
    ui.add_space(4.0);
    let bare = Label::new("Profile").show(ui).rect;
    let with = Label::new("Profile")
        .icon(Icon::User)
        .icon_end(Icon::ChevronDown)
        .show(ui)
        .rect;
    ui.label(format!(
        "text-only: {:.1} x {:.1} px; icon+text+icon: {:.1} x {:.1} px; delta: {:.1} px (expected: h=16 tallest child, delta=48 = 16 + 8 + 8 + 16)",
        bare.width(),
        bare.height(),
        with.width(),
        with.height(),
        with.width() - bare.width()
    ));

    // 5. Icon-only (spec §9): exactly the 16px icon box, no phantom gap.
    ui.add_space(12.0);
    ui.label("5. Icon-only — exactly the icon box (spec §9)");
    ui.add_space(4.0);
    let r = Label::new("").icon(Icon::User).show(ui).rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: 16 x 16, no phantom gap; icon size is a builder API decision, spec §3.3)",
        r.width(),
        r.height()
    ));

    // 6. Empty label (spec §9): zero-size, not an error.
    ui.add_space(12.0);
    ui.label("6. Empty label — zero-size (spec §9)");
    ui.add_space(4.0);
    let r = Label::new("").show(ui).rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: 0 x 0, no panic)",
        r.width(),
        r.height()
    ));

    // 7. Disabled (D1 PROPOSED deviation, spec §10): 50% element
    // opacity, instant; geometry identical to the enabled twin.
    ui.add_space(12.0);
    ui.label(
        "7. Disabled — D1 flag (PROPOSED deviation, pending Expert Reviewer approval; spec §10)",
    );
    ui.add_space(4.0);
    let enabled = Label::new("Disabled label").icon(Icon::User).show(ui).rect;
    let disabled = Label::new("Disabled label")
        .icon(Icon::User)
        .disabled(true)
        .show(ui)
        .rect;
    ui.label(format!(
        "enabled: {:.1} x {:.1} px; disabled: {:.1} x {:.1} px; geometry identical: {} (expected: same rect, 50% opacity, instant)",
        enabled.width(),
        enabled.height(),
        disabled.width(),
        disabled.height(),
        enabled.size() == disabled.size()
    ));

    // 8. `for_id` bound labels (spec §7). Each label is bound to its
    // target via `ui.scope_id().with(salt)`, which matches the target
    // control's `.id_salt(salt)` (child Uis inherit the parent scope) —
    // except the input pair (§8a), which binds the id captured from the
    // `Input` response: `Input` shows its text edit inside a child
    // scope, so its widget id is not `ui.scope_id().with(salt)` (S9
    // finding F1; see `Label::for_id`). OPEN-1 is SETTLED (S6 Work B,
    // Radix source): a label click fires a click on the labeled control
    // (activation for checkbox/radio/switch/button; focus for text
    // inputs) AND focuses it. The labels take NO Tab stop (S1 sense
    // fix; S6 verifies runtime, OPEN-6). Control state lives in egui
    // temp data (gallery pattern).
    ui.add_space(12.0);
    ui.label("8. Bound labels — for_id targets (spec §7; OPEN-1 settled: click fires + focuses; §8a uses captured-id binding)");
    ui.add_space(4.0);
    let input_store = egui::Id::unique("label_demo_input_text");
    let mut input_text: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(input_store))
        .unwrap_or_else(|| "m@example.com".to_owned());
    let cb_store = egui::Id::unique("label_demo_checkbox");
    let mut cb_checked: bool = ui.ctx().data_mut(|d| d.get_temp(cb_store).unwrap_or(false));
    let radio_store = egui::Id::unique("label_demo_radio");
    let mut radio_checked: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(radio_store).unwrap_or(false));
    let sw_store = egui::Id::unique("label_demo_switch");
    let mut sw_on: bool = ui.ctx().data_mut(|d| d.get_temp(sw_store).unwrap_or(false));
    let dis_store = egui::Id::unique("label_demo_disabled_cb");
    let mut dis_checked: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(dis_store).unwrap_or(false));

    // 8a. Text input target. The binding id is captured from the
    // `Input` response: its widget id is computed inside a child scope
    // (see `Label::for_id`), so the label binds the id captured on the
    // previous pass (`Id::NULL` on the first pass targets no widget).
    let input_id_store = egui::Id::unique("label_demo_input_id");
    let (lr, focused) = ui
        .horizontal(|ui| {
            let target = ui
                .ctx()
                .data_mut(|d| d.get_temp(input_id_store))
                .unwrap_or(egui::Id::NULL);
            let lr = Label::new("Email").for_id(target).show(ui).rect;
            let resp = Input::new(&mut input_text)
                .id_salt("label_demo_input")
                .show(ui);
            ui.ctx()
                .data_mut(|d| d.insert_temp(input_id_store, resp.id));
            (lr, resp.has_focus())
        })
        .inner;
    ui.label(format!(
        "input target: label {:.1} x {:.1} px; target focused: {}",
        lr.width(),
        lr.height(),
        focused
    ));
    ui.add_space(4.0);
    // 8b. Checkbox target.
    let (lr, focused) = ui
        .horizontal(|ui| {
            let target = ui.scope_id().with("label_demo_cb");
            let lr = Label::new("Accept terms").for_id(target).show(ui).rect;
            let resp = Checkbox::new(&mut cb_checked)
                .id_salt("label_demo_cb")
                .show(ui);
            (lr, resp.has_focus())
        })
        .inner;
    ui.label(format!(
        "checkbox target: label {:.1} x {:.1} px; checked: {}; target focused: {}",
        lr.width(),
        lr.height(),
        cb_checked,
        focused
    ));
    ui.add_space(4.0);
    // 8c. Radio target.
    let (lr, focused) = ui
        .horizontal(|ui| {
            let target = ui.scope_id().with("label_demo_radio");
            let lr = Label::new("Option A").for_id(target).show(ui).rect;
            let resp = RadioGroupItem::new(&mut radio_checked)
                .id_salt("label_demo_radio")
                .show(ui);
            (lr, resp.has_focus())
        })
        .inner;
    ui.label(format!(
        "radio target: label {:.1} x {:.1} px; selected: {}; target focused: {}",
        lr.width(),
        lr.height(),
        radio_checked,
        focused
    ));
    ui.add_space(4.0);
    // 8d. Switch target.
    let (lr, focused) = ui
        .horizontal(|ui| {
            let target = ui.scope_id().with("label_demo_sw");
            let lr = Label::new("Notifications").for_id(target).show(ui).rect;
            let resp = Switch::new(&mut sw_on).id_salt("label_demo_sw").show(ui);
            (lr, resp.has_focus())
        })
        .inner;
    ui.label(format!(
        "switch target: label {:.1} x {:.1} px; on: {}; target focused: {}",
        lr.width(),
        lr.height(),
        sw_on,
        focused
    ));
    ui.add_space(4.0);
    // 8e. Button target.
    let (lr, focused) = ui
        .horizontal(|ui| {
            let target = ui.scope_id().with("label_demo_btn");
            let lr = Label::new("Submit").for_id(target).show(ui).rect;
            let resp = Button::new("Target button")
                .id_salt("label_demo_btn")
                .show(ui);
            (lr, resp.has_focus())
        })
        .inner;
    ui.label(format!(
        "button target: label {:.1} x {:.1} px; target focused: {}",
        lr.width(),
        lr.height(),
        focused
    ));
    ui.add_space(4.0);
    // 8f. Disabled checkbox target.
    let (lr, focused) = ui
        .horizontal(|ui| {
            let target = ui.scope_id().with("label_demo_dis_cb");
            let lr = Label::new("Disabled option").for_id(target).show(ui).rect;
            let resp = Checkbox::new(&mut dis_checked)
                .id_salt("label_demo_dis_cb")
                .disabled(true)
                .show(ui);
            (lr, resp.has_focus())
        })
        .inner;
    ui.label(format!(
        "disabled checkbox target: label {:.1} x {:.1} px; checked: {}; target focused: {}",
        lr.width(),
        lr.height(),
        dis_checked,
        focused
    ));
    ui.ctx().data_mut(|d| {
        d.insert_temp(input_store, input_text);
        d.insert_temp(cb_store, cb_checked);
        d.insert_temp(radio_store, radio_checked);
        d.insert_temp(sw_store, sw_on);
        d.insert_temp(dis_store, dis_checked);
    });

    // 9. `show_with` generic composition (OPEN-5 DECIDED — coordinator
    // note 2026-11-16): closure widgets render as flex children in DOM
    // order, one content-sized row, 8px gaps, vertical centering. The
    // closure runs TWICE (measure pass + real pass); closure content
    // must be a pure function of its inputs.
    ui.add_space(12.0);
    ui.label(
        "9. show_with composition — closure children as flex children (spec §8, OPEN-5 decided)",
    );
    ui.add_space(4.0);
    let row = Label::new("Status").show_with(ui, |ui| {
        ui.label("active");
    });
    let r = row.response.rect;
    ui.label(format!(
        "text + closure label: row {:.1} x {:.1} px (expected: content-sized, 8px gap, children vertically centred)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    // 9b. Nested control: the nested checkbox keeps its own hits; the
    // label adds no click-sense rect over it (pointer-state forwarding).
    // No `for_id` here — the nested-is-target case needs an id-salted
    // control and is S6's construction, not this demo's.
    let nested_store = egui::Id::unique("label_demo_nested_cb");
    let mut nested: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(nested_store).unwrap_or(false));
    let row = Label::new("Accept").show_with(ui, |ui| {
        ui.checkbox(&mut nested, "");
    });
    let r = row.response.rect;
    ui.ctx().data_mut(|d| d.insert_temp(nested_store, nested));
    ui.label(format!(
        "nested checkbox: row {:.1} x {:.1} px; checked: {}; nested control keeps its own hits (expected: content-sized, 8px gap)",
        r.width(),
        r.height(),
        nested
    ));

    // 10. Long text (spec §9): no `whitespace-nowrap` class exists, so
    // the original wraps by layout into 14px line boxes. The measured
    // rect below is the OBSERVED behaviour in a 320px container; S5
    // judges it against §9. No verdict is stated here.
    ui.add_space(12.0);
    ui.label("10. Long text — 320px container (spec §9; observed behaviour, S5 judges)");
    ui.add_space(4.0);
    let r = ui
        .allocate_ui(egui::Vec2::new(320.0, 60.0), |ui| {
            Label::new(
                "This is a very long label that exceeds the width of its container by a wide margin",
            )
            .show(ui)
            .rect
        })
        .inner;
    ui.label(format!(
        "measured: {:.1} x {:.1} px in a 320px container (spec §9: wraps into 14px line boxes)",
        r.width(),
        r.height()
    ));
}

pub fn separator_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the separator actually allocated, so
    // S5/S6 can verify geometry from screenshots. Outlines and fills drawn
    // by this demo are labelled demo chrome, not component rendering.

    ui.label("1. Horizontal — full width, 1px (spec §2.3)");
    ui.add_space(4.0);
    ui.label("Text above the separator.");
    let rect = ui
        .allocate_ui(egui::Vec2::new(320.0, 1.0), |ui| {
            Separator::new().show(ui).rect
        })
        .inner;
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (container 320px wide)",
        rect.width(),
        rect.height()
    ));
    ui.label("Text below the separator.");

    ui.add_space(12.0);
    ui.label("2. Vertical — 1px, full height of a 48px row (spec §2.3)");
    ui.add_space(4.0);
    // NOTE: a plain `ui.horizontal` sizes its child Ui to
    // `interact_size.y` (36px), so the row height must be set explicitly
    // via `allocate_ui_with_layout` for the separator to claim 48px.
    let row = ui.allocate_ui_with_layout(
        egui::Vec2::new(360.0, 48.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label("Left");
            let sep = Separator::new()
                .orientation(SeparatorOrientation::Vertical)
                .show(ui)
                .rect;
            ui.label("Right");
            sep
        },
    );
    ui.painter().rect_stroke(
        row.response.rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(136, 136, 136)),
        egui::epaint::StrokeKind::Inside,
    );
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (row 360 x 48 px; grey outline is demo chrome)",
        row.inner.width(),
        row.inner.height()
    ));

    ui.add_space(12.0);
    ui.label("3. Decorative true/false — identical rendering (spec §3)");
    ui.add_space(4.0);
    ui.label("Two identical 200px containers; only `decorative` differs.");
    let mut rects = [egui::Rect::NOTHING; 2];
    ui.horizontal(|ui| {
        for (i, decorative) in [true, false].into_iter().enumerate() {
            rects[i] = ui
                .allocate_ui(egui::Vec2::new(200.0, 1.0), |ui| {
                    Separator::new().decorative(decorative).show(ui).rect
                })
                .inner;
            if i == 0 {
                ui.add_space(16.0);
            }
        }
    });
    ui.label(format!(
        "decorative=true: {:.0} x {:.0} px; decorative=false: {:.0} x {:.0} px; sizes identical: {}",
        rects[0].width(),
        rects[0].height(),
        rects[1].width(),
        rects[1].height(),
        rects[0].size() == rects[1].size()
    ));

    ui.add_space(12.0);
    ui.label("4. Flex row — separator is never compressed (spec §9.1, shrink-0)");
    ui.add_space(4.0);
    let spacing = ui.spacing().item_spacing.x;
    let sep_rect = ui
        .allocate_ui(egui::Vec2::new(400.0, 20.0), |ui| {
            ui.horizontal(|ui| {
                // Fixed 80px sibling (blue fill is demo chrome).
                let (sib_rect, _) =
                    ui.allocate_exact_size(egui::Vec2::new(80.0, 20.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(sib_rect, 0.0, egui::Color32::from_rgb(59, 130, 246));
                Separator::new().show(ui).rect
            })
            .inner
        })
        .inner;
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (container 400px - 80px sibling - {:.0}px spacing = {:.0}px)",
        sep_rect.width(),
        sep_rect.height(),
        spacing,
        400.0 - 80.0 - spacing
    ));

    ui.add_space(12.0);
    ui.label("5. Flex column — vertical separator claims the full parent height (spec §9.2)");
    ui.add_space(4.0);
    let column = ui.allocate_ui(egui::Vec2::new(120.0, 64.0), |ui| {
        Separator::new().vertical(true).show(ui).rect
    });
    ui.painter().rect_stroke(
        column.response.rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(136, 136, 136)),
        egui::epaint::StrokeKind::Inside,
    );
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (parent 120 x 64 px; grey outline is demo chrome)",
        column.inner.width(),
        column.inner.height()
    ));

    ui.add_space(12.0);
    ui.label("6. Length override — Rust-side extension, not spec behaviour (spec §8.2)");
    ui.add_space(4.0);
    let rect = ui
        .allocate_ui(egui::Vec2::new(320.0, 1.0), |ui| {
            Separator::new().length(160.0).show(ui).rect
        })
        .inner;
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (override 160px in a 320px container)",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("7. Color override — Rust-side extension, caller-className analogue (spec §9.4)");
    ui.add_space(4.0);
    ui.label("Default fill (border token):");
    let default_rect = ui
        .allocate_ui(egui::Vec2::new(320.0, 1.0), |ui| {
            Separator::new().show(ui).rect
        })
        .inner;
    ui.add_space(8.0);
    ui.label("Override fill (red):");
    let override_rect = ui
        .allocate_ui(egui::Vec2::new(320.0, 1.0), |ui| {
            Separator::new()
                .color(egui::Color32::from_rgb(239, 68, 68))
                .show(ui)
                .rect
        })
        .inner;
    ui.label(format!(
        "default {:.0} x {:.0} px; override {:.0} x {:.0} px — same geometry, only the fill differs",
        default_rect.width(),
        default_rect.height(),
        override_rect.width(),
        override_rect.height()
    ));

    ui.add_space(12.0);
    ui.label("8. Adjacent separators — each renders independently (spec §9.7)");
    ui.add_space(4.0);
    let (first, second) = ui
        .allocate_ui(egui::Vec2::new(320.0, 8.0), |ui| {
            let a = Separator::new().show(ui).rect;
            let b = Separator::new().show(ui).rect;
            (a, b)
        })
        .inner;
    ui.label(format!(
        "first y={:.0} h={:.0}; second y={:.0} h={:.0}; gap {:.0}px is parent-layout spacing, not the component",
        first.min.y,
        first.height(),
        second.min.y,
        second.height(),
        second.min.y - first.max.y
    ));
}

pub fn kbd_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the chip actually allocated
    // (`Response.rect`), so S5/S6 can verify geometry from screenshots.
    // The tooltip-content backdrop in section 9 is demo chrome
    // simulating `bg-foreground`, not component rendering.

    ui.label("1. Single key — 20px height, width per §4 (spec §3.1/§4)");
    ui.add_space(4.0);
    let rect = Kbd::new("A").show(ui).rect;
    ui.label(format!(
        "measured: {:.0} x {:.0} px (expected: h=20, w=max(20, text+8))",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("2. Text key — width grows with the label (spec §4)");
    ui.add_space(4.0);
    let rect = Kbd::new("Enter").show(ui).rect;
    ui.label(format!(
        "measured: {:.0} x {:.0} px (expected: h=20, w=max(20, text+8))",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("3. Group — 4px gaps, height = tallest child (spec §3.3/§4)");
    ui.add_space(4.0);
    let group = KbdGroup::new().show(ui, |ui| {
        let first = Kbd::new("⌘").show(ui).rect;
        let second = Kbd::new("K").show(ui).rect;
        (first, second)
    });
    // NOTE: the group row rect is the tallest child (20px for standard
    // chips, spec §3.3). It used to read 36px: plain `ui.horizontal`
    // seeds its row to `interact_size.y` (defect KBD-VIS-01, fixed by
    // the S9 content-sized allocation in `KbdGroup::show`). The readout
    // prints both the row rect and the chip union so the §3.3 claim can
    // be judged against either.
    let (first, second) = group.inner;
    let content = first.union(second);
    ui.label(format!(
        "group row rect: {:.0} x {:.0} px; chip union: {:.0} x {:.0} px; chips: {:.0} x {:.0} + {:.0} x {:.0} px; measured gap: {:.0} px (expected: gap 4, group w = chips + 4, content h = tallest child = 20)",
        group.response.rect.width(),
        group.response.rect.height(),
        content.width(),
        content.height(),
        first.width(),
        first.height(),
        second.width(),
        second.height(),
        second.min.x - first.max.x
    ));

    ui.add_space(12.0);
    ui.label("4. Icon + text — every icon and gap counts (spec §4)");
    ui.add_space(4.0);
    let rect = Kbd::new("M").icon(Icon::ChevronLeft).show(ui).rect;
    ui.label(format!(
        "measured: {:.0} x {:.0} px (expected: h=20, w=max(20, text+24) = 4 pad + 12 icon + 4 gap + text + 4 pad)",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("5. Multiple icons + text — every icon and gap counts (spec §4)");
    ui.add_space(4.0);
    let rect = Kbd::new("M")
        .icon(Icon::ChevronLeft)
        .icon(Icon::ChevronRight)
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: {:.0} x {:.0} px (expected: h=20, w=max(20, text+40))",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("6. Explicit icon sizes — an explicit size always wins (spec §3.2)");
    ui.add_space(4.0);
    ui.label("Icon-only chips, so the width arithmetic is exact (4 + icon + 4, 20px floor).");
    let mut rects = [egui::Rect::NOTHING; 3];
    ui.horizontal(|ui| {
        rects[0] = Kbd::new("").icon(Icon::ChevronDown).show(ui).rect;
        rects[1] = Kbd::new("")
            .icon(Icon::ChevronDown)
            .icon_size(16.0)
            .show(ui)
            .rect;
        rects[2] = Kbd::new("")
            .icon_sized(Icon::ChevronDown, 20.0)
            .icon_size(16.0)
            .show(ui)
            .rect;
    });
    ui.label(format!(
        "default 12px icon: {:.0} x {:.0} px (expected 20 x 20); .icon_size(16): {:.0} x {:.0} px (expected 24 x 20); .icon_sized(20) + .icon_size(16): {:.0} x {:.0} px (expected 28 x 20 — explicit size wins)",
        rects[0].width(),
        rects[0].height(),
        rects[1].width(),
        rects[1].height(),
        rects[2].width(),
        rects[2].height()
    ));

    ui.add_space(12.0);
    ui.label("7. Empty chip — the 20px floor still renders (spec §4/§10)");
    ui.add_space(4.0);
    let rect = Kbd::new("").show(ui).rect;
    ui.label(format!(
        "measured: {:.0} x {:.0} px (expected: 20 x 20, the min-w-5/h-5 floor)",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("8. Icon-only chip — exactly the floor (spec §4/§10)");
    ui.add_space(4.0);
    let rect = Kbd::new("").icon(Icon::ChevronDown).show(ui).rect;
    ui.label(format!(
        "measured: {:.0} x {:.0} px (expected: 20 x 20 = 4 + 12 + 4)",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("9. Tooltip context — D1 flag (PROPOSED deviation, pending Expert Reviewer approval; spec §2.4)");
    ui.add_space(4.0);
    let theme = ShadcnTheme::of_ui(ui);
    let (wash, mode_name) = match theme.mode {
        Mode::Light => ("background/20", "light"),
        Mode::Dark => ("background/10", "dark"),
    };
    ui.label(format!(
        "Backdrop is demo chrome simulating tooltip content (bg-foreground: dark in light mode, light in dark mode). Chips with .on_dark(true) in {} mode: wash {} + background text; all other slots unchanged.",
        mode_name, wash
    ));
    ui.add_space(4.0);
    let backdrop = egui::Frame::new()
        .fill(theme.palette.foreground)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(12));
    let on_dark_rect = backdrop
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let first = Kbd::new("⌘").on_dark(true).show(ui).rect;
                Kbd::new("K").on_dark(true).show(ui);
                first
            })
            .inner
        })
        .inner;
    ui.add_space(8.0);
    ui.label("Same chips with the flag off (default muted values, for contrast):");
    ui.add_space(4.0);
    let off_rect = egui::Frame::new()
        .fill(theme.palette.foreground)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let first = Kbd::new("⌘").show(ui).rect;
                Kbd::new("K").show(ui);
                first
            })
            .inner
        })
        .inner;
    ui.label(format!(
        "on_dark(true): {:.0} x {:.0} px; flag off: {:.0} x {:.0} px (geometry unchanged by the flag; only fill/text differ)",
        on_dark_rect.width(),
        on_dark_rect.height(),
        off_rect.width(),
        off_rect.height()
    ));

    ui.add_space(12.0);
    ui.label("10. Long key names — full label, no wrap, no truncation (spec §10)");
    ui.add_space(4.0);
    let mut rects = [egui::Rect::NOTHING; 2];
    ui.horizontal(|ui| {
        rects[0] = Kbd::new("Caps Lock").show(ui).rect;
        rects[1] = Kbd::new("Backspace").show(ui).rect;
    });
    ui.label(format!(
        "Caps Lock: {:.0} x {:.0} px; Backspace: {:.0} x {:.0} px (expected: h=20, w=max(20, text+8), full label visible)",
        rects[0].width(),
        rects[0].height(),
        rects[1].width(),
        rects[1].height()
    ));
}

pub fn spinner_demo(ui: &mut egui::Ui) {
    // Intro (brief §7): nova = base, colour inherited, gallery chrome owns
    // recipe/theme switching. No demo-side theme switching.
    ui.label("Spinner — one 16px icon, rotating 1 rev/s (size-4 animate-spin).");
    ui.label("No .cn-spinner* rule exists, so nova renders identically to base (LAYER).");
    ui.label("Colour is inherited currentColor in every config — switch style/theme with the gallery chrome.");
    ui.add_space(12.0);

    // 1. Default spinner (spec §§3.1, 6).
    ui.label("1. Default — size-4 (16px), animate-spin: 1s linear infinite clockwise");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let resp = Spinner::new().show(ui);
        ui.label(format!(
            "allocated: {:.0} x {:.0} px; stroke 2s/24 = 1.333px at 16px (spec §3.2)",
            resp.rect.width(),
            resp.rect.height()
        ));
    });

    // 2. Custom sizes (spec §§3.2–3.3; OPEN-3). Period is S7's job, judged
    // from frame sequences; the demo only exposes the sizes.
    ui.add_space(12.0);
    ui.label("2. Custom sizes — .size() override replaces the 16px default; glyph + stroke scale, period stays 1s");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for size in [24.0, 32.0] {
            let resp = Spinner::new().size(size).show(ui);
            ui.label(format!(
                "{:.0}px: allocated {:.0} x {:.0}, stroke {:.3}px",
                size,
                resp.rect.width(),
                resp.rect.height(),
                2.0 * size / 24.0
            ));
            ui.add_space(16.0);
        }
    });

    // 3. Relational colour (spec §5) — the key S5 check. No explicit
    // spinner colour is set anywhere in this section: the spinner must
    // match the adjacent label, which shares its style context.
    ui.add_space(12.0);
    ui.label("3. Inherited colour — spinner stroke must equal the adjacent text colour in the same context (spec §5)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Spinner::new().show(ui);
        ui.label("Default context: spinner inherits the surrounding text colour.");
    });
    ui.add_space(4.0);
    ui.scope(|ui| {
        let theme = ShadcnTheme::of_ui(ui);
        ui.style_mut().visuals.override_text_color = Some(theme.palette.muted_foreground);
        ui.horizontal(|ui| {
            Spinner::new().show(ui);
            ui.label("Muted-foreground context: spinner inherits the muted text colour.");
        });
    });
    ui.add_space(4.0);
    ui.scope(|ui| {
        let theme = ShadcnTheme::of_ui(ui);
        ui.style_mut().visuals.override_text_color = Some(theme.palette.primary);
        ui.horizontal(|ui| {
            Spinner::new().show(ui);
            ui.label("Primary context: spinner inherits the primary text colour.");
        });
    });

    // 4. Custom colour (spec §§5, 8): .color() is the Rust equivalent of
    // a caller text-* class. The label is painted the same explicit
    // colour so S5 can compare spinner stroke vs text directly.
    ui.add_space(12.0);
    ui.label("4. Custom colour — .color() override, the equivalent of a caller text-* class");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let theme = ShadcnTheme::of_ui(ui);
        Spinner::new().color(theme.palette.destructive).show(ui);
        ui.label(
            egui::RichText::new("Destructive text — spinner uses the same explicit colour.")
                .color(theme.palette.destructive),
        );
    });

    // 5. Button context (spec §9.1). The Button API takes no spinner
    // child, so the spinner's inheritance in the button's label context
    // is shown in a demo-chrome frame replicating that context, next to
    // a real default button for direct pixel comparison.
    ui.add_space(12.0);
    ui.label("5. Button context — spinner inherits the button's label colour and aligns with button text");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Button::new("Loading").show(ui);
        ui.label("Real default button, for reference.");
    });
    ui.add_space(4.0);
    let theme = ShadcnTheme::of_ui(ui);
    ui.scope(|ui| {
        ui.style_mut().visuals.override_text_color = Some(theme.palette.primary_foreground);
        egui::Frame::new()
            .fill(theme.palette.primary)
            .inner_margin(egui::Margin::symmetric(12, 6))
            .corner_radius(egui::CornerRadius::same(6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    Spinner::new().show(ui);
                    ui.label("Loading");
                });
            });
    });
    ui.label("Demo chrome: the filled frame replicates the default button's label context (fill primary, text primary_foreground). The spinner inside inherits that colour — compare its stroke with the real button's label above.");

    // 6. Multiple spinners (spec §9.7). Phase alignment between spinners
    // is unspecified in the original and must not be judged; all three
    // rotate from the shared absolute input-time clock (S1), which S7
    // observes from sequences.
    ui.add_space(12.0);
    ui.label("6. Multiple spinners — three defaults in a row (spec §9.7: phase alignment unspecified, observed not judged)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Spinner::new().show(ui);
        Spinner::new().show(ui);
        Spinner::new().show(ui);
        ui.label("All three spin from the same absolute clock.");
    });
}

pub fn skeleton_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the skeleton actually allocated, so
    // S5/S6/S7 can verify geometry from screenshots. The pulse values are
    // the OPEN-1 interim position (Tailwind v4 defaults), not
    // snapshot-verified (spec §6).
    ui.label(
        "Pulse (OPEN-1 interim): 2s cubic-bezier(0.4, 0, 0.6, 1), opacity 1 → 0.5 → 1, infinite.",
    );

    ui.add_space(12.0);
    ui.label("1. Default — 120 x 20 px (Rust-side convenience default; the original declares no size, spec §3.1)");
    ui.add_space(4.0);
    let rect = Skeleton::new().show(ui).rect;
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (expected: 120 x 20)",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("2. Explicit sizes — caller-sized, no minimum/maximum/aspect rule (spec §3.1, §9)");
    for (w, h) in [(200.0, 20.0), (80.0, 80.0), (320.0, 12.0)] {
        ui.add_space(4.0);
        let rect = Skeleton::new().size(w, h).show(ui).rect;
        ui.label(format!(
            "requested: {:.0} x {:.0} px; allocated: {:.0} x {:.0} px",
            w,
            h,
            rect.width(),
            rect.height()
        ));
    }

    ui.add_space(12.0);
    ui.label("3. Static — full opacity, no pulse (D3 Rust extension, spec §10; use this section for clean fill/radius pixel measurement)");
    ui.add_space(4.0);
    let rect = Skeleton::new()
        .size(200.0, 20.0)
        .animate(false)
        .show(ui)
        .rect;
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (expected: 200 x 20); fill is the `muted` token at full opacity",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("4. Circle / avatar placeholder — caller radius override wins over the recipe radius (spec §3.3; merge mechanism OPEN-2)");
    ui.add_space(4.0);
    let override_radius = theme_radius_full(ui);
    let rect = Skeleton::new()
        .size(48.0, 48.0)
        .corner_radius(override_radius)
        .show(ui)
        .rect;
    ui.label(format!(
        "allocated: {:.0} x {:.0} px; corner_radius override: {} px (painter clamps it to a full circle)",
        rect.width(),
        rect.height(),
        override_radius
    ));

    ui.add_space(12.0);
    ui.label("5. Multiple skeletons — three identical placeholders in a row (phase observed, not judged; the spec states no sync requirement, spec §9)");
    ui.add_space(4.0);
    let mut rects = [egui::Rect::NOTHING; 3];
    ui.horizontal(|ui| {
        for rect in &mut rects {
            *rect = Skeleton::new().size(120.0, 20.0).show(ui).rect;
        }
    });
    let identical = rects
        .iter()
        .all(|r| r.width() == 120.0 && r.height() == 20.0);
    ui.label(format!(
        "allocated: {:.0} x {:.0}, {:.0} x {:.0}, {:.0} x {:.0} px (identical sizes: {})",
        rects[0].width(),
        rects[0].height(),
        rects[1].width(),
        rects[1].height(),
        rects[2].width(),
        rects[2].height(),
        identical
    ));

    ui.add_space(12.0);
    ui.label("6. Zero-size — allocates without panic (spec §9; nothing visible is expected)");
    ui.add_space(4.0);
    let rect = Skeleton::new().size(0.0, 0.0).show(ui).rect;
    ui.label(format!(
        "allocated: {:.0} x {:.0} px (expected: 0 x 0, no panic)",
        rect.width(),
        rect.height()
    ));

    ui.add_space(12.0);
    ui.label("7. Text-line composition — demo composition (OPEN-4), not spec behaviour");
    for w in [320.0, 280.0, 300.0] {
        ui.add_space(4.0);
        let rect = Skeleton::new().size(w, 16.0).show(ui).rect;
        ui.label(format!(
            "requested: {:.0} x 16 px; allocated: {:.0} x {:.0} px",
            w,
            rect.width(),
            rect.height()
        ));
    }
}

fn theme_radius_full(ui: &egui::Ui) -> u8 {
    ShadcnTheme::of_ui(ui).radius.full_u8()
}

pub fn typography_demo(ui: &mut egui::Ui) {
    // Evidence position (spec §0, binding): NO typography recipe is
    // snapshot-verified — shadcn/ui ships no typography component and the
    // docs example files are absent from the snapshot. Every recipe below
    // is QUOTED (gaps-A + the module table, which agree); sizes/weights
    // are TAILWIND-DEFAULT pairings; CodeBlock is an INVENTED element
    // (spec §2.4, OPEN-5 metrics confirmed by the Coordinator). Nova is
    // silent on every typography slot (VERIFIED, spec §2.3), so one
    // recipe per element renders under both recipes and both themes.
    // Gaps between blocks follow the §4.2 caller-applied margins
    // (QUOTED, OPEN-2): H2 `mt-10` 40px, H3/H4 `mt-8` 32px, P/Blockquote/
    // lists `mt-6`/`my-6` 24px, TableCaption `mt-4` 16px. Section headers
    // and readout lines are demo chrome; every other visible rect is the
    // component. Readouts print the rect the element actually allocated
    // (`Response.rect`), so S5 can verify geometry from screenshots.
    ui.label(
        "Recipes are QUOTED (no snapshot-verified original exists); nova is \
         silent, so one recipe renders under both recipes and both themes \
         (spec §§0, 2.3). Gaps follow the §4.2 caller-applied margins \
         (QUOTED, OPEN-2).",
    );
    ui.add_space(12.0);

    // Section 1 — Headings (spec §2.2, QUOTED; D1 tracking renders 0;
    // D2 H1 renders at 600, requirement 800 frozen).
    ui.label("1. Headings H1–H4 (spec §2.2, QUOTED; D1/D2 as stated)");
    ui.add_space(4.0);
    let h1 = H1::new("Taxing Laughter: The Joke Tax Chronicles").show(ui);
    let content_left = h1.rect.left();
    ui.label(format!(
        "H1: allocated {:.1} × {:.1} px (expected 36px/40px leading, one row)",
        h1.rect.width(),
        h1.rect.height()
    ));
    ui.add_space(40.0); // H2 `mt-10`
    let h2 = H2::new("The King's Plan").show(ui);
    ui.label(format!(
        "H2: allocated {:.1} × {:.1} px (expected 30px/36px + 8px pb-2 + 1px rule = 45px; rule spans the full width)",
        h2.rect.width(),
        h2.rect.height()
    ));
    ui.add_space(32.0); // H3 `mt-8`
    let h3 = H3::new("The Joke Tax Chronicles").show(ui);
    ui.label(format!(
        "H3: allocated {:.1} × {:.1} px (expected 24px/32px leading, one row)",
        h3.rect.width(),
        h3.rect.height()
    ));
    ui.add_space(32.0); // H4 `mt-8`
    let h4 = H4::new("Exemptions").show(ui);
    ui.label(format!(
        "H4: allocated {:.1} × {:.1} px (expected 20px/28px leading, one row)",
        h4.rect.width(),
        h4.rect.height()
    ));

    // Section 2 — P, Lead, Large (spec §2.2, QUOTED).
    ui.add_space(24.0); // P `mt-6`
    ui.label("2. Paragraph, Lead, Large (spec §2.2, QUOTED)");
    ui.add_space(4.0);
    let p = P::new("The joke tax is a levy imposed on the punchline economy.").show(ui);
    ui.label(format!(
        "P: allocated {:.1} × {:.1} px (expected 16px, leading-7 = 28px, one row)",
        p.rect.width(),
        p.rect.height()
    ));
    ui.add_space(12.0);
    let lead = Lead::new(
        "A modal dialog that interrupts the user with important content and expects a response.",
    )
    .show(ui);
    ui.label(format!(
        "Lead: allocated {:.1} × {:.1} px (expected 20px/28px, muted-foreground)",
        lead.rect.width(),
        lead.rect.height()
    ));
    ui.add_space(12.0);
    let large = Large::new("Display a prominent message in a large size.").show(ui);
    ui.label(format!(
        "Large: allocated {:.1} × {:.1} px (expected 18px/28px, semibold 600)",
        large.rect.width(),
        large.rect.height()
    ));

    // Section 3 — Small, Muted (spec §2.2, QUOTED).
    ui.add_space(12.0);
    ui.label("3. Small, Muted (spec §2.2, QUOTED)");
    ui.add_space(4.0);
    let small = Small::new("Email address").show(ui);
    ui.label(format!(
        "Small: allocated {:.1} × {:.1} px (expected 14px, medium 500, leading-none = 14px tall)",
        small.rect.width(),
        small.rect.height()
    ));
    ui.add_space(12.0);
    let muted = Muted::new("Enter your email to subscribe to the joke tax digest.").show(ui);
    ui.label(format!(
        "Muted: allocated {:.1} × {:.1} px (expected 14px/20px, muted-foreground)",
        muted.rect.width(),
        muted.rect.height()
    ));

    // Section 4 — Blockquote (spec §5.2, QUOTED + DERIVED).
    ui.add_space(24.0); // Blockquote `mt-6`
    ui.label("4. Blockquote (spec §5.2, QUOTED; bar + offset DERIVED)");
    ui.add_space(4.0);
    let bq = Blockquote::new("After the tax was repealed, the jester's salary doubled.").show(ui);
    ui.label(format!(
        "Blockquote: allocated {:.1} × {:.1} px; 2px bar at the left edge (x = {:.1}), text starts 26px in (2px bar + 24px pl-6), italic, 16px/24px",
        bq.rect.width(),
        bq.rect.height(),
        bq.rect.left()
    ));

    // Section 5 — Lists (spec §5.3, QUOTED + DERIVED). Nested lists are
    // OPEN-4 and are NOT shown.
    ui.add_space(24.0); // Ul `my-6`
    ui.label("5. Lists Ul / Ol (spec §5.3, QUOTED; no nesting — OPEN-4)");
    ui.add_space(4.0);
    let ul = Ul::new(&[
        "Puns under one second",
        "Knock-knock jokes",
        "Sighs and groans",
    ])
    .show(ui);
    ui.label(format!(
        "Ul: allocated {:.1} × {:.1} px (spec §5.3 breakdown 8 + 3×24 + 2×8 = 96px; rect left offset {:.1} px = the 24px ml-6 indent, applied as an explicit offset; no indent guide line — defects TYP-VIS-01/02/03 fixed in S9, verified by S10)",
        ul.rect.width(),
        ul.rect.height(),
        ul.rect.left() - content_left
    ));
    ui.add_space(24.0); // Ol `my-6`
    let ol = Ol::new(&[
        "File form LOL-1",
        "Attach proof of laughter",
        "Pay within 30 days",
    ])
    .show(ui);
    ui.label(format!(
        "Ol: allocated {:.1} × {:.1} px (same 96px total as Ul; spec breakdown 8 + 3×24 + 2×8; decimal markers 1. 2. 3.)",
        ol.rect.width(),
        ol.rect.height()
    ));

    // Section 6 — InlineCode (spec §5.4, QUOTED; D7 proposed: renders at
    // Geist Mono Medium 500, requirement 600 frozen — no 600 mono face
    // is embedded).
    ui.add_space(12.0);
    ui.label("6. InlineCode (spec §5.4, QUOTED; D7: renders at mono 500)");
    ui.add_space(4.0);
    let chip = InlineCode::new("Cmd + K").show(ui);
    ui.label(format!(
        "Standalone chip: allocated {:.1} × {:.1} px (expected 14px mono, 20px leading + 6.4px padding = 26.4px tall; 4px radius; muted fill)",
        chip.rect.width(),
        chip.rect.height()
    ));
    ui.add_space(12.0);
    // E4 composition: the chip inside a line of text. NOTE: a plain
    // `ui.horizontal` sizes its row to `interact_size.y` (36px) — the
    // same egui layout artifact the separator/kbd demos documented.
    // The readout prints BOTH the row rect and the chip rect so S5 can
    // judge the chip's own allocation (26.4px) against the row.
    let row = ui.horizontal(|ui| {
        ui.label("Press");
        let chip = InlineCode::new("GDP_giggles").show(ui);
        ui.label("to open the palette.");
        chip.rect
    });
    ui.label(format!(
        "Inline in a text line: row rect {:.1} × {:.1} px (ui.horizontal 36px artifact); chip rect {:.1} × {:.1} px (expected 26.4px tall)",
        row.response.rect.width(),
        row.response.rect.height(),
        row.inner.width(),
        row.inner.height()
    ));

    // Section 7 — CodeBlock (spec §2.4, INVENTED; OPEN-5 metrics
    // CONFIRMED by the Coordinator: 16px mono, 24px leading, 400,
    // `foreground`; fill `muted`, radius 8px, 16px padding).
    ui.add_space(12.0);
    ui.label("7. CodeBlock (spec §2.4, INVENTED; OPEN-5 metrics confirmed)");
    ui.add_space(4.0);
    let code = CodeBlock::new(
        "fn levy(punchline: &Punchline) -> Tax {\n    Tax::new(punchline.words()).rate(0.2)\n}",
    )
    .show(ui);
    ui.label(format!(
        "CodeBlock: allocated {:.1} × {:.1} px (expected 3 rows × 24px + 32px padding = 104px; 16px mono, muted fill, 8px radius)",
        code.rect.width(),
        code.rect.height()
    ));

    // Section 8 — TableCaption in a table context (spec §9; recipe §2.2,
    // QUOTED). The grid rows are demo chrome (plain labels); only the
    // caption is the component. Placement is the table's concern.
    ui.add_space(12.0);
    ui.label("8. TableCaption in a table context (spec §9; recipe QUOTED)");
    ui.add_space(4.0);
    egui::Grid::new("typography_table_caption")
        .striped(false)
        .show(ui, |ui| {
            ui.label("Quarter");
            ui.label("Joke revenue");
            ui.end_row();
            ui.label("Q1");
            ui.label("12,400 giggles");
            ui.end_row();
            ui.label("Q2");
            ui.label("18,900 giggles");
            ui.end_row();
        });
    ui.add_space(16.0); // TableCaption `mt-4`
    let caption = TableCaption::new("Table 1 — joke revenue by quarter.").show(ui);
    ui.label(format!(
        "TableCaption: allocated {:.1} × {:.1} px (expected 14px/20px, muted-foreground; grid above is demo chrome)",
        caption.rect.width(),
        caption.rect.height()
    ));

    // Section 9 — Long-heading wrap (spec §10 E1; D5: no `text-balance`,
    // normal greedy wrapping). Width constrained to 420px via
    // `allocate_ui` (the separator demo's pattern) so the heading wraps
    // at any window size.
    ui.add_space(40.0); // H2 `mt-10`
    ui.label("9. Long H2 wrapping at 420px (spec §10 E1; D5, no text-balance)");
    ui.add_space(4.0);
    let wrapped = ui
        .allocate_ui(egui::Vec2::new(420.0, 1.0), |ui| {
            H2::new(
                "The Royal Decree on the Taxation of Punchlines, Jokes, and Other Matters of Mirth",
            )
            .show(ui)
        })
        .inner;
    let wrap_rows = (wrapped.rect.height() - 9.0) / 36.0;
    ui.label(format!(
        "Wrapped H2: allocated {:.1} × {:.1} px at 420px wide ({:.1} rows × 36px + 8px pb-2 + 1px rule; no truncation, no ellipsis)",
        wrapped.rect.width(),
        wrapped.rect.height(),
        wrap_rows
    ));

    // Section 10 — Empty elements (spec §10 E2): no source rule is
    // quoted. Observed behaviour only — the spec defers the ruling and
    // S5 records what allocates. No panic, no stray chrome expected.
    ui.add_space(24.0); // P `mt-6`
    ui.label("10. Empty elements (spec §10 E2 — observed behaviour, spec defers the ruling)");
    ui.add_space(4.0);
    let empty_p = P::new("").show(ui);
    ui.label(format!(
        "Empty P: allocated {:.1} × {:.1} px",
        empty_p.rect.width(),
        empty_p.rect.height()
    ));
    ui.add_space(12.0);
    let empty_h3 = H3::new("").show(ui);
    ui.label(format!(
        "Empty H3: allocated {:.1} × {:.1} px",
        empty_h3.rect.width(),
        empty_h3.rect.height()
    ));

    ui.add_space(12.0);
    ui.label("No animation, no keyboard behaviour, no interactive states (spec §§7–8).");
}

/// Demo chrome for a ratio box: fills the whole box the child `Ui` was
/// given and centers a label in it. The component paints nothing itself;
/// this fill + label exist only so S5/S6 can see the allocated box.
/// Returns the box rect (the child `Ui`'s max rect). Paints without
/// allocating, so nested boxes still start at the top of the outer box.
fn paint_ratio_box(
    ui: &mut egui::Ui,
    fill: egui::Color32,
    text_color: egui::Color32,
    label: &str,
) -> egui::Rect {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 8, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        text_color,
    );
    rect
}

/// Readout line comparing the rect the component actually allocated
/// against the expected width-driven geometry.
fn ratio_readout(
    ui: &mut egui::Ui,
    what: &str,
    rect: egui::Rect,
    expected_w: f32,
    expected_h: f32,
) {
    ui.label(format!(
        "{what}: measured {w:.1} × {h:.1} px, expected {ew:.1} × {eh:.1} px",
        w = rect.width(),
        h = rect.height(),
        ew = expected_w,
        eh = expected_h,
    ));
}

pub fn aspect_ratio_demo(ui: &mut egui::Ui) {
    let theme = ShadcnTheme::of_ui(ui);
    ui.label("AspectRatio paints nothing itself: every fill, outline, and label inside a box below is demo chrome. The readouts report the rect the component allocated. Renders identically under both recipes and both themes (spec §5) — switch with the gallery chrome.");
    ui.add_space(12.0);

    ui.label("Ratios — fixed width 320px (spec §4.1: height = width / ratio)");
    ui.add_space(4.0);
    for (ratio, name, fill, fg) in [
        (
            16.0 / 9.0,
            "16:9",
            theme.palette.muted,
            theme.palette.muted_foreground,
        ),
        (
            1.0,
            "1:1",
            theme.palette.accent,
            theme.palette.accent_foreground,
        ),
        (
            9.0 / 16.0,
            "9:16",
            theme.palette.secondary,
            theme.palette.secondary_foreground,
        ),
        (
            4.0 / 3.0,
            "4:3",
            theme.palette.primary,
            theme.palette.primary_foreground,
        ),
    ] {
        ui.label(format!("ratio {name}"));
        let resp = AspectRatio::new(ratio).max_width(320.0).show(ui, |ui| {
            paint_ratio_box(ui, fill, fg, name);
        });
        ratio_readout(ui, "box", resp.response.rect, 320.0, 320.0 / ratio);
        ui.add_space(12.0);
    }

    ui.label("Default ratio — AspectRatio::default() renders square (ratio 1; spec §3.2, coordinator decision D1-a)");
    ui.add_space(4.0);
    let resp = AspectRatio::default().max_width(240.0).show(ui, |ui| {
        paint_ratio_box(
            ui,
            theme.palette.accent,
            theme.palette.accent_foreground,
            "default",
        );
    });
    ratio_readout(ui, "default box", resp.response.rect, 240.0, 240.0);
    ui.add_space(12.0);

    ui.label("Nested ratios — each level sizes independently (spec §8)");
    ui.add_space(4.0);
    let outer = AspectRatio::new(16.0 / 9.0)
        .max_width(320.0)
        .show(ui, |ui| {
            let outer_rect =
                paint_ratio_box(ui, theme.palette.muted, theme.palette.muted_foreground, "");
            let inner = AspectRatio::new(1.0).max_width(180.0).show(ui, |ui| {
                paint_ratio_box(
                    ui,
                    theme.palette.primary,
                    theme.palette.primary_foreground,
                    "inner 1:1",
                );
            });
            // Label the outer box in the strip the inner box does not cover.
            ui.painter().text(
                egui::pos2(outer_rect.max.x - 8.0, outer_rect.min.y + 8.0),
                egui::Align2::RIGHT_TOP,
                "outer 16:9",
                egui::FontId::proportional(14.0),
                theme.palette.muted_foreground,
            );
            inner.response.rect
        });
    ui.label(format!(
        "outer: measured {ow:.1} × {oh:.1} px (expected 320.0 × 180.0); inner: measured {iw:.1} × {ih:.1} px (expected 180.0 × 180.0)",
        ow = outer.response.rect.width(),
        oh = outer.response.rect.height(),
        iw = outer.inner.width(),
        ih = outer.inner.height(),
    ));
    ui.add_space(12.0);

    ui.label("max_width() — Rust-only convenience, no original counterpart (spec §9 D4)");
    ui.add_space(4.0);
    let available = ui.available_width();
    let resp = AspectRatio::new(16.0 / 9.0)
        .max_width(200.0)
        .show(ui, |ui| {
            paint_ratio_box(
                ui,
                theme.palette.secondary,
                theme.palette.secondary_foreground,
                "capped at 200px",
            );
        });
    ui.label(format!(
        "available width {available:.1} px; measured {w:.1} × {h:.1} px — the 200px cap applies, height follows 200 / (16/9) = 112.5",
        w = resp.response.rect.width(),
        h = resp.response.rect.height(),
    ));
    ui.add_space(12.0);

    ui.label("Overflow — child content taller than the ratio box (O2 CLOSED; spec §4.3)");
    ui.label("The Radix primitive sets no overflow control on either element, so overflowing content paints outside the box under the CSS `visible` default — lines below the box edge are the original behaviour.");
    ui.add_space(4.0);
    let overflow = AspectRatio::new(16.0 / 9.0)
        .max_width(320.0)
        .show(ui, |ui| {
            paint_ratio_box(ui, theme.palette.muted, theme.palette.muted_foreground, "");
            for i in 1..=14 {
                ui.label(format!("overflow line {i}"));
            }
            ui.min_rect().height()
        });
    // Room for the overflowing lines to stay visible below the box:
    // content is ~342px tall in a 180px box, so ~162px overflows.
    ui.add_space(180.0);
    ui.label(format!(
        "box: measured {w:.1} × {h:.1} px (expected 320.0 × 180.0); child content height {ch:.1} px — content is taller than the box",
        w = overflow.response.rect.width(),
        h = overflow.response.rect.height(),
        ch = overflow.inner,
    ));
    ui.add_space(12.0);

    ui.label("Degenerate ratios — 0-height box, matching the derived original behaviour (O4 CLOSED as derived; S8 gate E1; D3 SUPERSEDED)");
    ui.label("Radix enforces the ratio via paddingBottom: `${100 / ratio}%`; for 0, negative, NaN, or infinite ratios the declaration is invalid CSS and is dropped, leaving padding 0 — the box collapses and content overflows visibly. The rect must stay finite and non-negative.");
    ui.add_space(4.0);
    for (ratio, name) in [(0.0f32, "ratio 0"), (-1.0f32, "ratio -1")] {
        ui.label(name);
        let resp = AspectRatio::new(ratio).max_width(2.0).show(ui, |ui| {
            paint_ratio_box(
                ui,
                theme.palette.destructive,
                theme.palette.destructive_foreground,
                "",
            );
        });
        let rect = resp.response.rect;
        ui.label(format!(
            "{name}: measured {w:.1} × {h:.1} px, finite: {finite}, non-negative: {non_neg} (expected 2.0 × 0.0 — collapsed box)",
            w = rect.width(),
            h = rect.height(),
            finite = rect.width().is_finite() && rect.height().is_finite(),
            non_neg = rect.width() >= 0.0 && rect.height() >= 0.0,
        ));
        ui.add_space(12.0);
    }

    ui.label("Empty child — the sized box is still allocated (spec §8)");
    ui.add_space(4.0);
    let resp = AspectRatio::new(16.0 / 9.0)
        .max_width(320.0)
        .show(ui, |_ui| {});
    let rect = resp.response.rect;
    // Demo-only chrome: outline the invisible box so S5 can see it.
    ui.painter().rect_stroke(
        rect,
        8,
        egui::Stroke::new(1.0, theme.palette.border),
        egui::epaint::StrokeKind::Inside,
    );
    ui.label("The outline above is demo-only chrome around the empty box.");
    ratio_readout(ui, "empty box", rect, 320.0, 180.0);
}

pub fn marker_demo(ui: &mut egui::Ui) {
    // Intro: nova effective values equal base (spec §2.3) — one geometry
    // and one colour rule serve both recipes; no style branching. Settled
    // frames: Marker stores last frame's measured content size (S1
    // report); the first frame uses the 16px floor, so S5/S6/S7 must
    // capture settled frames.
    ui.label("Marker — nova effective values equal base (§2.3): one geometry, one colour rule, both recipes.");
    ui.label("Row height = max(16px, tallest item) (§4.1, N5); gap 8px set explicitly (§4.2, N12). Captures must use settled frames (one-frame stored-measurement transient).");
    ui.add_space(12.0);

    // 1. Default variant (§§3, 4.1, 4.2).
    ui.label("1. Default — text, icon+text, text+icon, icon-only, empty (spec §§3, 4.1, 4.2)");
    ui.add_space(4.0);
    let row = Marker::new()
        .id_salt("s1-text")
        .show(ui, |ui| MarkerContent::new("Single-line text").show(ui));
    ui.label(format!(
        "text-only: row {:.1} x {:.1} px; content {:.1} x {:.1} px (expect 20px tall: 14px text in a 20px line box, O1)",
        row.response.rect.width(),
        row.response.rect.height(),
        row.inner.rect.width(),
        row.inner.rect.height()
    ));
    ui.add_space(4.0);
    let row = Marker::new().id_salt("s1-icon-text").show(ui, |ui| {
        let icon = MarkerIcon::new(Icon::Info).show(ui);
        let text = MarkerContent::new("Icon + text").show(ui);
        (icon.rect, text.rect)
    });
    let (icon_rect, text_rect) = row.inner;
    ui.label(format!(
        "icon+text: row {:.1} x {:.1} px; icon {:.1} x {:.1} px; measured gap {:.1} px (expect 8px, §4.2)",
        row.response.rect.width(),
        row.response.rect.height(),
        icon_rect.width(),
        icon_rect.height(),
        text_rect.left() - icon_rect.right()
    ));
    ui.add_space(4.0);
    let row = Marker::new().id_salt("s1-text-icon").show(ui, |ui| {
        let text = MarkerContent::new("Text + icon").show(ui);
        let icon = MarkerIcon::new(Icon::ArrowRight).show(ui);
        (text.rect, icon.rect)
    });
    let (text_rect, icon_rect) = row.inner;
    ui.label(format!(
        "text+icon: row {:.1} x {:.1} px; measured gap {:.1} px (expect 8px, §4.2)",
        row.response.rect.width(),
        row.response.rect.height(),
        icon_rect.left() - text_rect.right()
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .id_salt("s1-icon-only")
        .show(ui, |ui| MarkerIcon::new(Icon::Check).show(ui));
    ui.label(format!(
        "icon-only: row {:.1} x {:.1} px (expect 16px tall, §4.1)",
        row.response.rect.width(),
        row.response.rect.height()
    ));
    ui.add_space(4.0);
    let row = Marker::new().id_salt("s1-empty").show(ui, |_ui| {});
    ui.label(format!(
        "empty: row {:.1} x {:.1} px (expect 16px tall, the min-h-4 floor, §4.1; full width)",
        row.response.rect.width(),
        row.response.rect.height()
    ));

    // 2. Separator variant (§5). Rule spans below are DERIVED from the
    // measured content rect (12px each side); S5 verifies painted pixels.
    ui.add_space(12.0);
    ui.label("2. Separator — rules share free space equally, 12px from content each side, vertically centred (spec §5)");
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Separator)
        .id_salt("s2-short")
        .show(ui, |ui| {
            MarkerContent::new("or continue with").show(ui).rect
        });
    let content = row.inner;
    let row_rect = row.response.rect;
    ui.label(format!(
        "short: row {:.1} x {:.1} px; content {:.1} x {:.1} px; content centre offset {:.1} px from row centre (expect 0); expected rule spans x {:.1}..{:.1} and {:.1}..{:.1} (derived; S5 verifies pixels)",
        row_rect.width(),
        row_rect.height(),
        content.width(),
        content.height(),
        content.center().x - row_rect.center().x,
        row_rect.left(),
        content.left() - 12.0,
        content.right() + 12.0,
        row_rect.right()
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Separator)
        .id_salt("s2-long")
        .show(ui, |ui| {
            MarkerContent::new("a much longer separator label")
                .show(ui)
                .rect
        });
    let content = row.inner;
    let row_rect = row.response.rect;
    ui.label(format!(
        "long: row {:.1} x {:.1} px; content {:.1} x {:.1} px; content centre offset {:.1} px (expect 0); expected rule spans x {:.1}..{:.1} and {:.1}..{:.1} (derived; S5 verifies pixels)",
        row_rect.width(),
        row_rect.height(),
        content.width(),
        content.height(),
        content.center().x - row_rect.center().x,
        row_rect.left(),
        content.left() - 12.0,
        content.right() + 12.0,
        row_rect.right()
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Separator)
        .id_salt("s2-empty")
        .show(ui, |_ui| {});
    let row_rect = row.response.rect;
    ui.label(format!(
        "empty: row {:.1} x {:.1} px; expected two rules with a centred 16px gap: x {:.1}..{:.1} and {:.1}..{:.1} (derived; S5 verifies pixels)",
        row_rect.width(),
        row_rect.height(),
        row_rect.left(),
        row_rect.center().x - 8.0,
        row_rect.center().x + 8.0,
        row_rect.right()
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Separator)
        .id_salt("s2-tall")
        .show(ui, |ui| {
            MarkerContent::new("Two-line content:\nthe rules stay centred")
                .show(ui)
                .rect
        });
    let content = row.inner;
    let row_rect = row.response.rect;
    ui.label(format!(
        "tall: row {:.1} x {:.1} px; content {:.1} x {:.1} px (expect 40px tall; rules vertically centred on the grown row, §5 — S5 verifies pixels)",
        row_rect.width(),
        row_rect.height(),
        content.width(),
        content.height()
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Separator)
        .id_salt("s2-icon-text")
        .show(ui, |ui| {
            let icon = MarkerIcon::new(Icon::Plus).show(ui);
            let text = MarkerContent::new("add item").show(ui);
            (icon.rect, text.rect)
        });
    let (icon_rect, text_rect) = row.inner;
    let row_rect = row.response.rect;
    let group_left = icon_rect.left();
    let group_right = text_rect.right();
    ui.label(format!(
        "icon+text: row {:.1} x {:.1} px; group {:.1} px wide, centre offset {:.1} px (expect 0); gap {:.1} px (expect 8px)",
        row_rect.width(),
        row_rect.height(),
        group_right - group_left,
        (group_left + group_right) / 2.0 - row_rect.center().x,
        text_rect.left() - icon_rect.right()
    ));

    // 3. Border variant (§6). Rule centre is DERIVED (row bottom - 0.5px);
    // S5 verifies painted pixels.
    ui.add_space(12.0);
    ui.label("3. Border — content + 8px padding + 1px full-width rule (spec §6)");
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Border)
        .id_salt("s3-text")
        .show(ui, |ui| {
            let icon = MarkerIcon::new(Icon::CircleAlert).show(ui);
            let text = MarkerContent::new("Deployment pending review").show(ui);
            (icon.rect, text.rect)
        });
    let (icon_rect, text_rect) = row.inner;
    let row_rect = row.response.rect;
    ui.label(format!(
        "text: row {:.1} x {:.1} px (expect 29 = 20 + 9); gap {:.1} px; derived rule centre y = {:.1} (row bottom - 0.5px; S5 verifies pixels)",
        row_rect.width(),
        row_rect.height(),
        text_rect.left() - icon_rect.right(),
        row_rect.bottom() - 0.5
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Border)
        .id_salt("s3-tall")
        .show(ui, |ui| {
            MarkerContent::new("Two-line content:\nthe row and its rule grow with it")
                .show(ui)
                .rect
        });
    let row_rect = row.response.rect;
    ui.label(format!(
        "tall: row {:.1} x {:.1} px (expect 49 = 40 + 9); derived rule centre y = {:.1} (S5 verifies pixels)",
        row_rect.width(),
        row_rect.height(),
        row_rect.bottom() - 0.5
    ));
    ui.add_space(4.0);
    // Ruled 16px total (S5 ruling, MRK-VIS-01 fixed by S9 7ff3f66):
    // base `min-h-4` + border `border-b pb-2` under border-box
    // (Tailwind v4 Preflight) => max(16, content + 9); the 8px padding
    // + 1px rule fit inside the 16px minimum for empty content.
    ui.label("EMPTY border row — ruled 16px total: border-box floor max(16, content + 9) (S5 ruling; MRK-VIS-01 fixed by S9 7ff3f66).");
    let row = Marker::new()
        .variant(MarkerVariant::Border)
        .id_salt("s3-empty")
        .show(ui, |_ui| {});
    let row_rect = row.response.rect;
    ui.label(format!(
        "empty border: row {:.1} x {:.1} px (ruled 16px total, border-box floor)",
        row_rect.width(),
        row_rect.height()
    ));

    // 4. Links (§7.2, D2 PROPOSED). MarkerLink is reachable only via the
    // full path until the coordinator adds the mod.rs re-export at merge
    // time (S1 report, API decision 3) — no re-export is added here.
    ui.add_space(12.0);
    ui.label("4. Links — underlined, 3px offset, hover -> foreground, instant (spec §7.2). D2 PROPOSED: clicks surface via Response, navigation caller-owned.");
    ui.add_space(4.0);
    // Click counter (demo chrome): proves click surfacing for S6.
    let store = ui.scope_id().with("marker_demo_link_clicks");
    let mut clicks: u32 = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or(0));
    let row = Marker::new().id_salt("s4-link").show(ui, |ui| {
        MarkerIcon::new(Icon::Info).show(ui);
        egui_shadcn_ui::components::marker::MarkerLink::new("View details").show(ui)
    });
    let link_rect = row.inner.rect;
    if row.inner.clicked() {
        clicks += 1;
    }
    ui.ctx().data_mut(|d| d.insert_temp(store, clicks));
    ui.label(format!(
        "default link: row {:.1} x {:.1} px; link origin ({:.1}, {:.1}), {:.1} x {:.1} px (hover target, id_salt \"s4-link\"); clicks so far: {} (demo-chrome counter)",
        row.response.rect.width(),
        row.response.rect.height(),
        link_rect.left(),
        link_rect.top(),
        link_rect.width(),
        link_rect.height(),
        clicks
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Separator)
        .id_salt("s4-sep-link")
        .show(ui, |ui| {
            egui_shadcn_ui::components::marker::MarkerLink::new("See all activity")
                .show(ui)
                .rect
        });
    let link_rect = row.inner;
    let row_rect = row.response.rect;
    ui.label(format!(
        "separator link: row {:.1} x {:.1} px; link origin ({:.1}, {:.1}), {:.1} x {:.1} px (hover target, id_salt \"s4-sep-link\"); centre offset {:.1} px (expect 0)",
        row_rect.width(),
        row_rect.height(),
        link_rect.left(),
        link_rect.top(),
        link_rect.width(),
        link_rect.height(),
        link_rect.center().x - row_rect.center().x
    ));
    ui.add_space(4.0);
    let row = Marker::new().id_salt("s4-mixed").show(ui, |ui| {
        let text = MarkerContent::new("Shared by").show(ui);
        let link = egui_shadcn_ui::components::marker::MarkerLink::new("ada@example.com").show(ui);
        (text.rect, link.rect)
    });
    let (text_rect, link_rect) = row.inner;
    ui.label(format!(
        "text + link: row {:.1} x {:.1} px; gap {:.1} px (expect 8px); link origin ({:.1}, {:.1}) (hover target, id_salt \"s4-mixed\")",
        row.response.rect.width(),
        row.response.rect.height(),
        link_rect.left() - text_rect.right(),
        link_rect.left(),
        link_rect.top()
    ));
    ui.label("Underline: 1px, text colour, baseline + 3px (D4 closest equivalent; O2 open — S5 judges). No :visited/:active/:focus-visible styling (§7.2).");

    // 5. Generic slot (§11.2, M-P2-2 composition half).
    ui.add_space(12.0);
    ui.label("5. Generic slot — MarkerContent::show_custom accepts arbitrary widgets (spec §11.2, M-P2-2)");
    ui.add_space(4.0);
    let row = Marker::new().id_salt("s5-button").show(ui, |ui| {
        let text = MarkerContent::new("Status:").show(ui);
        let slot = MarkerContent::show_custom(ui, |ui| ui.button("Retry"));
        (text.rect, slot.inner.rect)
    });
    let (text_rect, button_rect) = row.inner;
    ui.label(format!(
        "text + button: row {:.1} x {:.1} px (follows tallest child); button {:.1} x {:.1} px; gap {:.1} px (expect 8px)",
        row.response.rect.width(),
        row.response.rect.height(),
        button_rect.width(),
        button_rect.height(),
        button_rect.left() - text_rect.right()
    ));
    ui.add_space(4.0);
    let row = Marker::new()
        .variant(MarkerVariant::Separator)
        .id_salt("s5-sep-button")
        .show(ui, |ui| {
            MarkerContent::show_custom(ui, |ui| ui.button("Load more"))
                .inner
                .rect
        });
    let button_rect = row.inner;
    let row_rect = row.response.rect;
    ui.label(format!(
        "separator + button slot: row {:.1} x {:.1} px; button {:.1} x {:.1} px; centre offset {:.1} px (expect 0)",
        row_rect.width(),
        row_rect.height(),
        button_rect.width(),
        button_rect.height(),
        button_rect.center().x - row_rect.center().x
    ));

    // 6. Long text (§§3.3, 12) in a 320px container. The grey outline is
    // demo chrome marking the container edge for S5.
    ui.add_space(12.0);
    ui.label("6. Long text — wraps at the container edge, row grows vertically (spec §§3.3, 12)");
    ui.add_space(4.0);
    let container = ui.allocate_ui(egui::Vec2::new(320.0, 80.0), |ui| {
        Marker::new().id_salt("s6-long").show(ui, |ui| {
            MarkerContent::new(
                "This is a long marker sentence that should wrap onto multiple lines within the 320px container.",
            )
            .show(ui)
            .rect
        })
    });
    let marker_rect = container.inner.response.rect;
    ui.painter().rect_stroke(
        marker_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(136, 136, 136)),
        egui::epaint::StrokeKind::Inside,
    );
    ui.label(format!(
        "long sentence: marker {:.1} x {:.1} px in a 320px container (expect width 320, height a multiple of 20px; grey outline is demo chrome)",
        marker_rect.width(),
        marker_rect.height()
    ));
    ui.add_space(4.0);
    let container = ui.allocate_ui(egui::Vec2::new(320.0, 60.0), |ui| {
        Marker::new().id_salt("s6-word").show(ui, |ui| {
            MarkerContent::new("antidisestablishmentarianismantidisestablishmentarianism")
                .show(ui)
                .rect
        })
    });
    let marker_rect = container.inner.response.rect;
    ui.painter().rect_stroke(
        marker_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(136, 136, 136)),
        egui::epaint::StrokeKind::Inside,
    );
    ui.label(format!(
        "unbroken word (56 chars): marker {:.1} x {:.1} px in a 320px container (expect wrap-break-word: breaks at the edge, no horizontal overflow past x = {:.1}; S5 verifies pixels)",
        marker_rect.width(),
        marker_rect.height(),
        marker_rect.left() + 320.0
    ));

    // 7. Nesting (§12; rendered counterpart of the S1 nesting test).
    ui.add_space(12.0);
    ui.label("7. Nesting — each marker renders independently (spec §12; rendered counterpart of the S1 nesting test)");
    ui.add_space(4.0);
    let row = Marker::new().id_salt("s7-outer").show(ui, |ui| {
        let icon = MarkerIcon::new(Icon::Info).show(ui);
        let slot = MarkerContent::show_custom(ui, |ui| {
            Marker::new()
                .variant(MarkerVariant::Border)
                .id_salt("s7-inner")
                .show(ui, |ui| {
                    MarkerContent::new("Nested border marker").show(ui).rect
                })
        });
        (icon.rect, slot.inner.response.rect)
    });
    let (icon_rect, inner_rect) = row.inner;
    ui.label(format!(
        "outer default + inner border: outer row {:.1} x {:.1} px; inner row {:.1} x {:.1} px (expect 29 = 20 + 9); gap {:.1} px (expect 8px)",
        row.response.rect.width(),
        row.response.rect.height(),
        inner_rect.width(),
        inner_rect.height(),
        inner_rect.left() - icon_rect.right()
    ));

    // 8. Multi-part row (§§3, 4.2, 7.2).
    ui.add_space(12.0);
    ui.label(
        "8. Multi-part row — icon + text + icon + link, 8px gaps throughout (spec §§3, 4.2, 7.2)",
    );
    ui.add_space(4.0);
    let row = Marker::new().id_salt("s8-multi").show(ui, |ui| {
        let icon1 = MarkerIcon::new(Icon::Sparkles).show(ui);
        let text = MarkerContent::new("Shared with").show(ui);
        let icon2 = MarkerIcon::new(Icon::ArrowRight).show(ui);
        let link = egui_shadcn_ui::components::marker::MarkerLink::new("3 people").show(ui);
        (icon1.rect, text.rect, icon2.rect, link.rect)
    });
    let (icon1_rect, text_rect, icon2_rect, link_rect) = row.inner;
    ui.label(format!(
        "multi-part: row {:.1} x {:.1} px; gaps {:.1} / {:.1} / {:.1} px (expect 8px each); link origin ({:.1}, {:.1})",
        row.response.rect.width(),
        row.response.rect.height(),
        text_rect.left() - icon1_rect.right(),
        icon2_rect.left() - text_rect.right(),
        link_rect.left() - icon2_rect.right(),
        link_rect.left(),
        link_rect.top()
    ));

    ui.add_space(12.0);
    ui.label("No background/radius/shadow/ring on any marker slot (§4.3); no row hover/focus/disabled styling (§7.1); nothing animates (§9); row/parts not Tab-focusable, links are (§10, D2).");
}

pub fn button_group_demo(ui: &mut egui::Ui) {
    // Every section prints the rects the group and its members actually
    // allocated (`Response.rect`), so S5/S6 can verify geometry from
    // screenshots. Rects are returned by the group closure (only the real
    // pass's return value is kept); the closure carries no side effects —
    // it runs 2-3x per frame (measure + real passes). All groups carry
    // explicit id_salts so ids are stable regardless of layout order.
    // Spec references are to the frozen button-group spec. The D8
    // mismatch cases (section 8) and the empty Text (section 6) are
    // OBSERVED-ONLY. OPEN-5 (input child in a constrained parent) and
    // O6/D5 (select triggers, Batch F) are NOT exercised by this demo.

    ui.label("1. Horizontal fused trio — shared 1px borders, zero gap (spec §§3.1, 8)");
    ui.add_space(4.0);
    let trio = ButtonGroup::new().id_salt("s1_trio").show(ui, |ui| {
        let a = Button::new("Day").show(ui).rect;
        let b = Button::new("Month").show(ui).rect;
        let c = Button::new("Year").show(ui).rect;
        (a, b, c)
    });
    let (a, b, c) = trio.inner;
    let g = trio.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; buttons: {:.1} x {:.1} + {:.1} x {:.1} + {:.1} x {:.1} px; shared edges: {:.1} / {:.1} px (expected: 0 / 0)",
        g.min.x, g.min.y, g.width(), g.height(),
        a.width(), a.height(), b.width(), b.height(), c.width(), c.height(),
        b.min.x - a.max.x, c.min.x - b.max.x
    ));

    ui.add_space(12.0);
    ui.label("2. Vertical group — text buttons stretch to the widest member, icon button keeps its fixed size (spec §5, FG-1)");
    ui.add_space(4.0);
    let vert = ButtonGroup::new()
        .orientation(GroupOrientation::Vertical)
        .id_salt("s2_vertical")
        .show(ui, |ui| {
            let a = Button::new("Cut").show(ui).rect;
            let b = Button::new("Copy a longer label").show(ui).rect;
            let c = Button::new("Paste").show(ui).rect;
            let icon = Button::new("")
                .size(ButtonSize::Icon)
                .icon_start(Icon::MoreHorizontal)
                .show(ui)
                .rect;
            (a, b, c, icon)
        });
    let (a, b, c, icon) = vert.inner;
    let g = vert.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; text buttons: {:.1} / {:.1} / {:.1} px wide; icon button: {:.1} x {:.1} px (fixed square, cross-start aligned); shared edges: {:.1} / {:.1} / {:.1} px (expected: 0 / 0 / 0)",
        g.min.x, g.min.y, g.width(), g.height(),
        a.width(), b.width(), c.width(), icon.width(), icon.height(),
        b.min.y - a.max.y, c.min.y - b.max.y, icon.min.y - c.max.y
    ));

    ui.add_space(12.0);
    ui.label("3. Trailing non-slot child — the button is the last slot-carrying child though Text follows it (spec §3.2, FG-2)");
    ui.add_space(4.0);
    let fg2 = ButtonGroup::new().id_salt("s3_fg2").show(ui, |ui| {
        let btn = Button::new("Action").show(ui).rect;
        let text = ButtonGroupText::new("https://example.com").show(ui).rect;
        (btn, text)
    });
    let (btn, text) = fg2.inner;
    let g = fg2.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; button: {:.1} x {:.1} px; Text: {:.1} x {:.1} px (stretched to the button height); S5 judges the button trailing corners + Text shadow-xs pixels",
        g.min.x, g.min.y, g.width(), g.height(),
        btn.width(), btn.height(), text.width(), text.height()
    ));
    ui.add_space(8.0);
    ui.label("3b. Common case — Text first, button last (spec §3.2)");
    ui.add_space(4.0);
    let fg2b = ButtonGroup::new().id_salt("s3_fg2b").show(ui, |ui| {
        let text = ButtonGroupText::new("https://example.com").show(ui).rect;
        let btn = Button::new("Action").show(ui).rect;
        (text, btn)
    });
    let (text, btn) = fg2b.inner;
    let g = fg2b.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; Text: {:.1} x {:.1} px; button: {:.1} x {:.1} px",
        g.min.x,
        g.min.y,
        g.width(),
        g.height(),
        text.width(),
        text.height(),
        btn.width(),
        btn.height()
    ));

    ui.add_space(12.0);
    ui.label("4. Nested group — automatic 8px gap, no .gap() call (spec §8, D4 automatic)");
    ui.add_space(4.0);
    let nested = ButtonGroup::new().id_salt("s4_nested").show(ui, |ui| {
        let inner = ButtonGroup::new()
            .id_salt("s4_nested_inner")
            .show(ui, |ui| {
                let l = Button::new("Left").show(ui).rect;
                let r = Button::new("Right").show(ui).rect;
                (l, r)
            });
        let submit = Button::new("Submit").show(ui).rect;
        (inner.response.rect, inner.inner, submit)
    });
    let (inner_rect, (l, r), submit) = nested.inner;
    let g = nested.response.rect;
    ui.label(format!(
        "outer group: x={:.1} y={:.1} {:.1} x {:.1} px; outer gap (nested group to Submit): {:.1} px (expected: 8); inner gap: {:.1} px (expected: 0); inner group: {:.1} x {:.1} px; Submit: {:.1} x {:.1} px",
        g.min.x, g.min.y, g.width(), g.height(),
        submit.min.x - inner_rect.max.x, r.min.x - l.max.x,
        inner_rect.width(), inner_rect.height(), submit.width(), submit.height()
    ));
    ui.add_space(8.0);
    ui.label(
        "4b. Explicit .gap(16.0) override — labelled override, the automatic rule is 8px (spec §8)",
    );
    ui.add_space(4.0);
    let over = ButtonGroup::new()
        .gap(16.0)
        .id_salt("s4_override")
        .show(ui, |ui| {
            let inner = ButtonGroup::new()
                .id_salt("s4_override_inner")
                .show(ui, |ui| {
                    let l = Button::new("Left").show(ui).rect;
                    let r = Button::new("Right").show(ui).rect;
                    (l, r)
                });
            let submit = Button::new("Submit").show(ui).rect;
            (inner.response.rect, inner.inner, submit)
        });
    let (inner_rect, (l, r), submit) = over.inner;
    let g = over.response.rect;
    ui.label(format!(
        "outer group: {:.1} x {:.1} px; outer gap: {:.1} px (expected: 16, the explicit override); inner gap: {:.1} px (expected: 0)",
        g.width(), g.height(),
        submit.min.x - inner_rect.max.x, r.min.x - l.max.x
    ));

    ui.add_space(12.0);
    ui.label("5. Separators — vertical rules in a horizontal group, stretched to the group height (spec §6)");
    ui.add_space(4.0);
    let seps = ButtonGroup::new().id_salt("s5a_seps").show(ui, |ui| {
        let a = Button::new("")
            .size(ButtonSize::Icon)
            .icon_start(Icon::ChevronLeft)
            .show(ui)
            .rect;
        let s1 = ButtonGroupSeparator::new().show(ui).rect;
        let b = Button::new("")
            .size(ButtonSize::Icon)
            .icon_start(Icon::MoreHorizontal)
            .show(ui)
            .rect;
        let s2 = ButtonGroupSeparator::new().show(ui).rect;
        let c = Button::new("")
            .size(ButtonSize::Icon)
            .icon_start(Icon::ChevronRight)
            .show(ui)
            .rect;
        (a, s1, b, s2, c)
    });
    let (a, s1, _b, s2, _c) = seps.inner;
    let g = seps.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; icon buttons: {:.1} x {:.1} px; separators: {:.1} x {:.1} px at x={:.1} and {:.1} x {:.1} px at x={:.1} (expected: 1px wide, full group height)",
        g.min.x, g.min.y, g.width(), g.height(),
        a.width(), a.height(),
        s1.width(), s1.height(), s1.min.x, s2.width(), s2.height(), s2.min.x
    ));
    ui.add_space(8.0);
    ui.label("5b. Separators — horizontal rules in a vertical group, full group width (spec §6)");
    ui.add_space(4.0);
    let vseps = ButtonGroup::new()
        .orientation(GroupOrientation::Vertical)
        .id_salt("s5b_vseps")
        .show(ui, |ui| {
            let a = Button::new("Top")
                .variant(ButtonVariant::Outline)
                .show(ui)
                .rect;
            let s1 = ButtonGroupSeparator::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui)
                .rect;
            let b = Button::new("Middle")
                .variant(ButtonVariant::Outline)
                .show(ui)
                .rect;
            let s2 = ButtonGroupSeparator::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui)
                .rect;
            let c = Button::new("Bottom")
                .variant(ButtonVariant::Outline)
                .show(ui)
                .rect;
            (a, s1, b, s2, c)
        });
    let (a, s1, b, s2, c) = vseps.inner;
    let g = vseps.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; separators: {:.1} x {:.1} px at y={:.1} and {:.1} x {:.1} px at y={:.1} (expected: full group width, 1px tall); buttons: {:.1} / {:.1} / {:.1} px wide",
        g.min.x, g.min.y, g.width(), g.height(),
        s1.width(), s1.height(), s1.min.y, s2.width(), s2.height(), s2.min.y,
        a.width(), b.width(), c.width()
    ));

    ui.add_space(12.0);
    ui.label("6. Text variants — Text with icon, stretched to the group height (spec §§2.3, 5, 7)");
    ui.add_space(4.0);
    let t1 = ButtonGroup::new().id_salt("s6a_text").show(ui, |ui| {
        let text = ButtonGroupText::new("https://example.com")
            .icon_start(Icon::Paperclip)
            .show(ui)
            .rect;
        let btn = Button::new("")
            .size(ButtonSize::Icon)
            .icon_start(Icon::Copy)
            .show(ui)
            .rect;
        (text, btn)
    });
    let (text, btn) = t1.inner;
    let g = t1.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; Text: {:.1} x {:.1} px; icon button: {:.1} x {:.1} px",
        g.min.x, g.min.y, g.width(), g.height(),
        text.width(), text.height(), btn.width(), btn.height()
    ));
    ui.add_space(8.0);
    ui.label("6b. Text with leading + trailing icons — 8px gaps, content centred vertically (spec §§2.3, 14)");
    ui.add_space(4.0);
    let t2 = ButtonGroup::new().id_salt("s6b_text_icons").show(ui, |ui| {
        let text = ButtonGroupText::new("Label")
            .icon_start(Icon::Search)
            .icon_end(Icon::ChevronDown)
            .show(ui)
            .rect;
        let btn = Button::new("Go").show(ui).rect;
        (text, btn)
    });
    let (text, btn) = t2.inner;
    let g = t2.response.rect;
    ui.label(format!(
        "group: {:.1} x {:.1} px; Text: {:.1} x {:.1} px; button: {:.1} x {:.1} px",
        g.width(),
        g.height(),
        text.width(),
        text.height(),
        btn.width(),
        btn.height()
    ));
    ui.add_space(8.0);
    ui.label("6c. Empty Text — OBSERVED ONLY, spec silent on empty Text (must-not-judge 24)");
    ui.add_space(4.0);
    let t3 = ButtonGroup::new().id_salt("s6c_empty_text").show(ui, |ui| {
        let text = ButtonGroupText::new("").show(ui).rect;
        let btn = Button::new("Go").show(ui).rect;
        (text, btn)
    });
    let (text, btn) = t3.inner;
    let g = t3.response.rect;
    ui.label(format!(
        "group: {:.1} x {:.1} px; empty Text: {:.1} x {:.1} px; button: {:.1} x {:.1} px",
        g.width(),
        g.height(),
        text.width(),
        text.height(),
        btn.width(),
        btn.height()
    ));
    ui.add_space(8.0);
    ui.label("6d. Standalone Text — outside any group, content height (spec §7.1: 20px text-only line box)");
    ui.add_space(4.0);
    let standalone = ButtonGroupText::new("Standalone text").show(ui).rect;
    ui.label(format!(
        "standalone Text: x={:.1} y={:.1} {:.1} x {:.1} px",
        standalone.min.x,
        standalone.min.y,
        standalone.width(),
        standalone.height()
    ));

    ui.add_space(12.0);
    ui.label("7. Focus — middle member reachable by Tab; S5/S6 judge the shared-edge ring pixels (spec §4, FG-3)");
    ui.add_space(4.0);
    let focus = ButtonGroup::new().id_salt("s7_focus").show(ui, |ui| {
        let a = Button::new("One").show(ui).rect;
        let mid = Button::new("Two").show(ui).rect;
        let c = Button::new("Three").show(ui).rect;
        (a, mid, c)
    });
    let (a, mid, c) = focus.inner;
    let g = focus.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; middle button: x={:.1} y={:.1} {:.1} x {:.1} px; neighbours: {:.1} / {:.1} px wide",
        g.min.x, g.min.y, g.width(), g.height(),
        mid.min.x, mid.min.y, mid.width(), mid.height(), a.width(), c.width()
    ));

    ui.add_space(12.0);
    ui.label("8. OBSERVED ONLY — D8 PROPOSED: horizontal rule in a horizontal group (§14 mismatch; Expert Reviewer decides)");
    ui.add_space(4.0);
    let m1 = ButtonGroup::new().id_salt("s8a_mismatch_h").show(ui, |ui| {
        let a = Button::new("Left").show(ui).rect;
        let sep = ButtonGroupSeparator::new()
            .orientation(GroupOrientation::Vertical)
            .show(ui)
            .rect;
        let b = Button::new("Right").show(ui).rect;
        (a, sep, b)
    });
    let (a, sep, b) = m1.inner;
    let g = m1.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; separator measured: {:.1} x {:.1} px at x={:.1}; buttons: {:.1} / {:.1} px wide; gap between buttons: {:.1} px",
        g.min.x, g.min.y, g.width(), g.height(),
        sep.width(), sep.height(), sep.min.x, a.width(), b.width(),
        b.min.x - a.max.x
    ));
    ui.add_space(8.0);
    ui.label("8b. OBSERVED ONLY — D8 PROPOSED: vertical rule in a vertical group (§14 mismatch; Expert Reviewer decides)");
    ui.add_space(4.0);
    let m2 = ButtonGroup::new()
        .orientation(GroupOrientation::Vertical)
        .id_salt("s8b_mismatch_v")
        .show(ui, |ui| {
            let a = Button::new("Top").show(ui).rect;
            let sep = ButtonGroupSeparator::new().show(ui).rect;
            let b = Button::new("Bottom").show(ui).rect;
            (a, sep, b)
        });
    let (a, sep, b) = m2.inner;
    let g = m2.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; separator measured: {:.1} x {:.1} px at y={:.1}; buttons: {:.1} / {:.1} px wide; gap between buttons: {:.1} px",
        g.min.x, g.min.y, g.width(), g.height(),
        sep.width(), sep.height(), sep.min.y, a.width(), b.width(),
        b.min.y - a.max.y
    ));

    ui.add_space(12.0);
    ui.label("9. Heights — mixed sizes: Text stretches to the tallest member, buttons keep fixed heights (spec §§5, 7.1)");
    ui.add_space(4.0);
    let heights = ButtonGroup::new().id_salt("s9_heights").show(ui, |ui| {
        let xs = Button::new("Xs").size(ButtonSize::Xs).show(ui).rect;
        let def = Button::new("Default").show(ui).rect;
        let lg = Button::new("Lg").size(ButtonSize::Lg).show(ui).rect;
        let text = ButtonGroupText::new("Text").show(ui).rect;
        (xs, def, lg, text)
    });
    let (xs, def, lg, text) = heights.inner;
    let g = heights.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; heights — xs: {:.1}, default: {:.1}, lg: {:.1}, Text: {:.1} px (Text stretched to the tallest member)",
        g.min.x, g.min.y, g.width(), g.height(),
        xs.height(), def.height(), lg.height(), text.height()
    ));
    ui.add_space(8.0);
    ui.label("9b. Heights — xs-only group: Text follows the 24px members (spec §7.1)");
    ui.add_space(4.0);
    let xs_only = ButtonGroup::new().id_salt("s9_xs").show(ui, |ui| {
        let a = Button::new("Xs").size(ButtonSize::Xs).show(ui).rect;
        let text = ButtonGroupText::new("Text").show(ui).rect;
        (a, text)
    });
    let (a, text) = xs_only.inner;
    let g = xs_only.response.rect;
    ui.label(format!(
        "group: {:.1} x {:.1} px; xs button: {:.1} px tall; Text: {:.1} px tall",
        g.width(),
        g.height(),
        a.height(),
        text.height()
    ));

    ui.add_space(12.0);
    ui.label(
        "10. States — disabled child renders per its own spec; the group adds nothing (spec §9)",
    );
    ui.add_space(4.0);
    let states = ButtonGroup::new().id_salt("s10_states").show(ui, |ui| {
        let dis = Button::new("Disabled").disabled(true).show(ui).rect;
        let act = Button::new("Active").show(ui).rect;
        (dis, act)
    });
    let (dis, act) = states.inner;
    let g = states.response.rect;
    ui.label(format!(
        "group: x={:.1} y={:.1} {:.1} x {:.1} px; disabled: {:.1} x {:.1} px; active: {:.1} x {:.1} px; shared edge: {:.1} px (expected: 0)",
        g.min.x, g.min.y, g.width(), g.height(),
        dis.width(), dis.height(), act.width(), act.height(), act.min.x - dis.max.x
    ));
    ui.add_space(8.0);
    ui.label("10b. Outline pair — fusing applies to outline buttons the same way (spec §3.1)");
    ui.add_space(4.0);
    let outline = ButtonGroup::new().id_salt("s10_outline").show(ui, |ui| {
        let a = Button::new("Outline left")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .rect;
        let b = Button::new("Outline right")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .rect;
        (a, b)
    });
    let (a, b) = outline.inner;
    let g = outline.response.rect;
    ui.label(format!(
        "group: {:.1} x {:.1} px; buttons: {:.1} + {:.1} px wide; shared edge: {:.1} px (expected: 0)",
        g.width(), g.height(), a.width(), b.width(), b.min.x - a.max.x
    ));

    ui.add_space(12.0);
    ui.label("11. Edge cases — single member keeps all radii/borders; empty group is zero-size (spec §14)");
    ui.add_space(4.0);
    let single = ButtonGroup::new()
        .id_salt("s11_single")
        .show(ui, |ui| Button::new("Only").show(ui).rect);
    let btn = single.inner;
    let g = single.response.rect;
    ui.label(format!(
        "single-member group: {:.1} x {:.1} px; button: {:.1} x {:.1} px",
        g.width(),
        g.height(),
        btn.width(),
        btn.height()
    ));
    ui.add_space(4.0);
    let empty = ButtonGroup::new().id_salt("s11_empty").show(ui, |_| {});
    let g = empty.response.rect;
    ui.label(format!(
        "empty group: {:.1} x {:.1} px (expected: 0 x 0)",
        g.width(),
        g.height()
    ));

    ui.add_space(12.0);
    ui.label("Notes: OPEN-5 (input child in a constrained parent) and O6/D5 (select triggers, Batch F) are not exercised on this page. D8 mismatch cases in section 8 are observed-only pending the Expert Reviewer.");
}

pub fn input_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the input actually allocated
    // (`Response.rect`), so S5/S6 can verify geometry from screenshots.
    // Header: recipe position, stated honestly (spec §§0, 3.4, 6, 10, 11).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Input — single-line text field. Active recipe: {recipe} (style: {}, mode: {mode}). Height 32px nova / 36px base; radius 10px nova / 8px base.",
        theme.style.name()
    ));
    ui.label("OPEN-1: focused+invalid precedence is PROVISIONAL (invalid wins). OPEN-2: transition timing 150ms + cubic-bezier(0.4, 0, 0.2, 1) is INTERIM (framework defaults, not snapshot truth). D3 (proposed): 14px text at all widths. D8 (proposed): text + password types only. No hover rule (§4.2) and no active rule (§4.3): hover/press change nothing.");

    let store = egui::Id::unique("input_demo_values");
    let mut values: Vec<String> = ui.ctx().data_mut(|d| {
        d.get_temp::<Vec<String>>(store).unwrap_or_else(|| {
            let mut v = vec![String::new(); 18];
            v[1] = "Hello, shadcn!".to_owned();
            v[5] = "not-an-email".to_owned();
            v[6] = "bad value".to_owned();
            v[8] = "Disabled value".to_owned();
            v[10] = "s3cr3t-value".to_owned();
            v[16] = "Select me".to_owned();
            v
        })
    });

    // 1. Resting: empty + filled + placeholder (spec §4.1/§4.7).
    ui.add_space(12.0);
    ui.label("1. Resting — empty, filled, placeholder (spec §4.1/§4.7; fill transparent light / input/30 dark)");
    ui.add_space(4.0);
    let r_empty = Input::new(&mut values[0])
        .id_salt("s3-resting-empty")
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_filled = Input::new(&mut values[1])
        .id_salt("s3-resting-filled")
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_ph = Input::new(&mut values[2])
        .id_salt("s3-resting-placeholder")
        .placeholder("m@example.com")
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: empty {:.1} x {:.1} px; filled {:.1} x {:.1} px; placeholder {:.1} x {:.1} px (expected: h=32 nova / 36 base, w = available)",
        r_empty.width(),
        r_empty.height(),
        r_filled.width(),
        r_filled.height(),
        r_ph.width(),
        r_ph.height()
    ));

    // 2. Focus target (spec §4.4).
    ui.add_space(12.0);
    ui.label("2. Focus target — Tab here or click; ring 3px + border ring on focus (spec §4.4; D1: ring on any focus)");
    ui.add_space(4.0);
    let resp = Input::new(&mut values[3])
        .id_salt("s3-focus")
        .placeholder("Focus me")
        .show(ui);
    // Label F1 rule: the widget id comes from the control's returned
    // `Response::id`, never a guessed scope id.
    let focus_id = resp.id;
    let focused = ui.ctx().memory(|m| m.has_focus(focus_id));
    let r_focus = resp.rect;
    ui.label(format!(
        "focus target: {:.1} x {:.1} px; focused: {focused}",
        r_focus.width(),
        r_focus.height()
    ));

    // 3. Invalid (spec §4.5).
    ui.add_space(12.0);
    ui.label("3. Invalid — destructive border both styles; 3px rest ring under NOVA only, none at rest under base (spec §4.5; nova dark border destructive/50, N5)");
    ui.add_space(4.0);
    let r_inv_empty = Input::new(&mut values[4])
        .id_salt("s3-invalid-empty")
        .invalid(true)
        .placeholder("Invalid email")
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_inv_filled = Input::new(&mut values[5])
        .id_salt("s3-invalid-filled")
        .invalid(true)
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_inv_disabled = Input::new(&mut values[6])
        .id_salt("s3-invalid-disabled")
        .invalid(true)
        .disabled(true)
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: invalid empty {:.1} x {:.1} px; invalid filled {:.1} x {:.1} px; invalid+disabled {:.1} x {:.1} px",
        r_inv_empty.width(),
        r_inv_empty.height(),
        r_inv_filled.width(),
        r_inv_filled.height(),
        r_inv_disabled.width(),
        r_inv_disabled.height()
    ));

    // 4. Disabled (spec §4.6).
    ui.add_space(12.0);
    ui.label("4. Disabled — whole element at 50% (§4.6); the disabled compositing model is an S5 question, not asserted here. Nova adds disabled fills (input/50 light, input/80 dark) before compositing.");
    ui.add_space(4.0);
    let r_dis_empty = Input::new(&mut values[7])
        .id_salt("s3-disabled-empty")
        .disabled(true)
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_dis_filled = Input::new(&mut values[8])
        .id_salt("s3-disabled-filled")
        .disabled(true)
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_dis_ph = Input::new(&mut values[9])
        .id_salt("s3-disabled-placeholder")
        .disabled(true)
        .placeholder("Disabled field")
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: disabled empty {:.1} x {:.1} px; disabled filled {:.1} x {:.1} px; disabled placeholder {:.1} x {:.1} px (invalid+disabled is section 3, row 3)",
        r_dis_empty.width(),
        r_dis_empty.height(),
        r_dis_filled.width(),
        r_dis_filled.height(),
        r_dis_ph.width(),
        r_dis_ph.height()
    ));

    // 5. Password (spec §9; D8).
    ui.add_space(12.0);
    ui.label("5. Password — typed value masked; placeholder renders UNMASKED (spec §9; D8: text + password only)");
    ui.add_space(4.0);
    let r_pw_filled = Input::new(&mut values[10])
        .id_salt("s3-password-filled")
        .password(true)
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_pw_ph = Input::new(&mut values[11])
        .id_salt("s3-password-placeholder")
        .password(true)
        .placeholder("Enter password")
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: password filled {:.1} x {:.1} px; password placeholder {:.1} x {:.1} px",
        r_pw_filled.width(),
        r_pw_filled.height(),
        r_pw_ph.width(),
        r_pw_ph.height()
    ));

    // 6. Widths (spec §3.3, OPEN-5).
    ui.add_space(12.0);
    ui.label("6. Widths — default w-full, explicit .width(240.0), horizontal-layout fallback (spec §3.3, OPEN-5)");
    ui.add_space(4.0);
    let r_w_full = Input::new(&mut values[12])
        .id_salt("s3-width-full")
        .placeholder("w-full: fills the available width")
        .show(ui)
        .rect;
    ui.add_space(4.0);
    let r_w_240 = Input::new(&mut values[13])
        .id_salt("s3-width-240")
        .width(240.0)
        .show(ui)
        .rect;
    ui.add_space(4.0);
    // Fallback case: the input sits in a horizontal row, where `w-full`
    // has no containing block and the 160pt fallback applies (OPEN-5,
    // `INPUT_FALLBACK_WIDTH`). The row rect is demo chrome:
    // `ui.horizontal` sizes the row to `interact_size.y` (36px); the
    // input allocates its own exact rect, printed below.
    let row = ui.horizontal(|ui| {
        Input::new(&mut values[14])
            .id_salt("s3-width-fallback")
            .show(ui)
            .rect
    });
    let r_w_fb = row.inner;
    let row_rect = row.response.rect;
    ui.label(format!(
        "measured: w-full {:.1} x {:.1} px; width(240) {:.1} x {:.1} px; fallback row {:.1} x {:.1} px, input {:.1} x {:.1} px (expected: fallback w=160)",
        r_w_full.width(),
        r_w_full.height(),
        r_w_240.width(),
        r_w_240.height(),
        row_rect.width(),
        row_rect.height(),
        r_w_fb.width(),
        r_w_fb.height()
    ));

    // 7. Hover target (spec §4.2: hover changes nothing).
    ui.add_space(12.0);
    ui.label(
        "7. Hover target — no hover rule: hovering must equal resting, pixel-identical (spec §4.2)",
    );
    ui.add_space(4.0);
    let resp = Input::new(&mut values[15])
        .id_salt("s3-hover")
        .placeholder("Hover me")
        .show(ui);
    let hovered = resp.hovered();
    let r_hover = resp.rect;
    ui.label(format!(
        "hover target: {:.1} x {:.1} px; hovered: {hovered}",
        r_hover.width(),
        r_hover.height()
    ));

    // 8. Selection target (spec §5, OPEN-6).
    ui.add_space(12.0);
    ui.label("8. Selection target — select-all here (Ctrl/Cmd+A); selection bg primary, text primary-foreground (spec §5, OPEN-6 per-widget override)");
    ui.add_space(4.0);
    let r_sel = Input::new(&mut values[16])
        .id_salt("s3-selection")
        .show(ui)
        .rect;
    ui.label(format!(
        "selection target: {:.1} x {:.1} px; value: \"{}\"",
        r_sel.width(),
        r_sel.height(),
        values[16]
    ));

    // 9. Char limit (spec §8.2; S1 documents the unit as Unicode scalars).
    ui.add_space(12.0);
    ui.label("9. Char limit — .char_limit(8); counting unit is Unicode scalar values (S1), not HTML maxLength UTF-16 units");
    ui.add_space(4.0);
    let r_limit = Input::new(&mut values[17])
        .id_salt("s3-char-limit")
        .char_limit(8)
        .placeholder("Max 8 chars")
        .show(ui)
        .rect;
    let n_chars = values[17].chars().count();
    ui.label(format!(
        "char-limit target: {:.1} x {:.1} px; length: {n_chars} / 8 chars",
        r_limit.width(),
        r_limit.height()
    ));

    ui.add_space(12.0);
    ui.label("Keyboard: Tab / Shift+Tab move focus; editing keys follow the native matrix (spec §7.1). Undo/redo, word jumps, paste newline-stripping, IME, and Escape are OPEN-10 — S6 observes the TextEdit mapping and reports. No fused/group row on this page: group fusing is the button-group line's page and input-group composition the input-group line's; the Input API exposes no group-position builder (position is auto-detected in show()).");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn textarea_demo(ui: &mut egui::Ui) {
    // Header: recipe position, stated honestly (spec §§2, 6, 10, 11 +
    // the transition narrow-reading coordinator ruling).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Textarea — single element, no variants. Active recipe: {recipe} (style: {}, mode: {mode}).",
        theme.style.name()
    ));
    ui.label("Base: field-sizing-content, min-h 64px, radius 8px, padding-x 12px, transition-[color,box-shadow]. Nova: radius 10px, padding-x 10px, transition-colors, invalid rest ring, /50 dark invalid border, disabled fills. OPEN-1: 150ms + cubic-bezier(0.4, 0, 0.2, 1) are interim framework defaults. OPEN-2: invalid-wins precedence is interim. OPEN-3: field-sizing edges (trailing newline, overflow) open. D5 single-size rendering (14px/20px at all widths) is PROPOSED. TXT-ANM-01 pending S9: base border/fill changes currently animate; the narrow-reading ruling requires snaps (ring fades under base).");

    // Demo state: one backing string per textarea, so editing one never
    // moves another, and every textarea carries a distinct id_salt so S6
    // probes have stable ids. Initial values are the §3.2 ladder cases.
    let store = egui::Id::unique("textarea_demo_values");
    let mut values: Vec<String> = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Vec<String>>(store))
        .filter(|v| v.len() == 17)
        .unwrap_or_else(|| {
            vec![
                String::new(), // 0: §1 empty
                "One line of text.".to_owned(), // 1: §1 one line
                "Line one\nLine two\nLine three".to_owned(), // 2: §1 three lines
                "Line one\nLine two\nLine three\nLine four".to_owned(), // 3: §1 four lines
                "Line one\nLine two\nLine three\nLine four\nLine five".to_owned(), // 4: §1 five lines
                "Growth target\nline two\nline three\nline four".to_owned(), // 5: §2 growth
                String::new(), // 6: §3 focus target
                String::new(), // 7: §4 invalid, empty
                "Invalid with content\nsecond line".to_owned(), // 8: §4 invalid, content
                String::new(), // 9: §5 disabled, empty
                "Disabled with content\nsecond line".to_owned(), // 10: §5 disabled, content
                String::new(), // 11: §6 placeholder
                String::new(), // 12: §6 placeholder, focusable
                String::new(), // 13: §7 rows(5), empty
                "Row one\nRow two\nRow three\nRow four".to_owned(), // 14: §7 rows(1), content
                "Wrapping grows this box on the §3.2 ladder: one long line soft-wraps into several visual rows at 300px wide, and the height follows max(64, 18 + 20 x N).".to_owned(), // 15: §7 narrow
                String::new(), // 16: §8 hover target
            ]
        });

    // Newline-separated line count (0 when empty). Soft wraps add visual
    // rows beyond this count; S5 derives visual rows from the height.
    let line_count = |s: &str| {
        if s.is_empty() {
            0
        } else {
            s.matches('\n').count() + 1
        }
    };

    // 1. Empty + content heights (spec §3.2 ladder).
    ui.add_space(12.0);
    ui.label("1. Empty + content heights (spec §3.2: height = max(64, 18 + 20 x N) px; 0-2 rows 64px, 3 rows 78px, 4 rows 98px, 5 rows 118px)");
    ui.add_space(4.0);
    let r = Textarea::new(&mut values[0]).id_salt("s3-empty").show(ui);
    ui.label(format!(
        "empty: {:.1} x {:.1} px; lines: {}; expect the 64px floor",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[0])
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[1]).id_salt("s3-one").show(ui);
    ui.label(format!(
        "one line: {:.1} x {:.1} px; lines: {}; expect 64px",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[1])
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[2]).id_salt("s3-three").show(ui);
    ui.label(format!(
        "three lines: {:.1} x {:.1} px; lines: {}; expect 78px",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[2])
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[3]).id_salt("s3-four").show(ui);
    ui.label(format!(
        "four lines: {:.1} x {:.1} px; lines: {}; expect 98px",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[3])
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[4]).id_salt("s3-five").show(ui);
    ui.label(format!(
        "five lines: {:.1} x {:.1} px; lines: {}; expect 118px",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[4])
    ));

    // 2. Growth (spec §3.4; D3 VOID — true content-sized growth).
    ui.add_space(12.0);
    ui.label("2. Growth (spec §3.4, D3 VOID — true content-sized growth)");
    ui.label("S6 drives growth/shrink by editing: typing new lines grows the box on the §3.2 ladder; deleting shrinks it back to the 64px floor. No internal scrollbar for fitted content.");
    ui.add_space(4.0);
    let r = Textarea::new(&mut values[5]).id_salt("s3-growth").show(ui);
    ui.label(format!(
        "growth target: {:.1} x {:.1} px; lines: {}; edit me",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[5])
    ));

    // 3. Focus target (spec §4), with clear space above and below.
    ui.add_space(12.0);
    ui.label("3. Focus target (spec §4: border ring + 3px ring/50 ring outside the border box)");
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[6]).id_salt("s3-focus").show(ui);
    ui.add_space(8.0);
    ui.label(format!(
        "focus target: {:.1} x {:.1} px; focused: {}",
        r.rect.width(),
        r.rect.height(),
        r.has_focus()
    ));

    // 4. Invalid (spec §4, N5 SETTLED).
    ui.add_space(12.0);
    ui.label("4. Invalid (spec §4, N5 SETTLED: nova shows the 3px rest ring unfocused + /50 dark border; base shows the destructive border with NO unfocused ring)");
    ui.add_space(4.0);
    let r = Textarea::new(&mut values[7])
        .invalid(true)
        .id_salt("s3-invalid-empty")
        .show(ui);
    ui.label(format!(
        "invalid, empty: {:.1} x {:.1} px",
        r.rect.width(),
        r.rect.height()
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[8])
        .invalid(true)
        .id_salt("s3-invalid-content")
        .show(ui);
    ui.label(format!(
        "invalid, with content: {:.1} x {:.1} px; lines: {}",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[8])
    ));

    // 5. Disabled (spec §4).
    ui.add_space(12.0);
    ui.label("5. Disabled (spec §4: whole element at 50% — nova fills composite at effective 25% light / 40% dark, base dark 15%; faded shadow; not-allowed cursor)");
    ui.add_space(4.0);
    let r = Textarea::new(&mut values[9])
        .disabled(true)
        .id_salt("s3-disabled-empty")
        .show(ui);
    ui.label(format!(
        "disabled, empty: {:.1} x {:.1} px",
        r.rect.width(),
        r.rect.height()
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[10])
        .disabled(true)
        .id_salt("s3-disabled-content")
        .show(ui);
    ui.label(format!(
        "disabled, with content: {:.1} x {:.1} px; lines: {}",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[10])
    ));

    // 6. Placeholder (spec §4).
    ui.add_space(12.0);
    ui.label("6. Placeholder (spec §4: muted-foreground, same size/line-height as body text; focus does not hide it — no focus:placeholder-* rule)");
    ui.add_space(4.0);
    let r = Textarea::new(&mut values[11])
        .placeholder("Type your message here.")
        .id_salt("s3-placeholder")
        .show(ui);
    ui.label(format!(
        "placeholder, empty: {:.1} x {:.1} px; value length: {} chars",
        r.rect.width(),
        r.rect.height(),
        values[11].chars().count()
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[12])
        .placeholder("Click here, then check the placeholder stays visible while focused.")
        .id_salt("s3-placeholder-focus")
        .show(ui);
    ui.label(format!(
        "placeholder, focusable: {:.1} x {:.1} px; value length: {} chars; focused: {}",
        r.rect.width(),
        r.rect.height(),
        values[12].chars().count(),
        r.has_focus()
    ));

    // 7. Rows + width (spec §3.2/§3.4/§8; D5 PROPOSED).
    ui.add_space(12.0);
    ui.label("7. Rows + width (spec §3.2/§3.4/§8; D5 PROPOSED: 14px text / 20px rows at all widths, including the 300px box below the 768px md breakpoint)");
    ui.add_space(4.0);
    let r = Textarea::new(&mut values[13])
        .rows(5)
        .id_salt("s3-rows5")
        .show(ui);
    ui.label(format!(
        "rows(5), empty: {:.1} x {:.1} px; rows is a minimum — expect the 118px floor",
        r.rect.width(),
        r.rect.height()
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[14])
        .rows(1)
        .id_salt("s3-rows1")
        .show(ui);
    ui.label(format!(
        "rows(1), four lines: {:.1} x {:.1} px; lines: {}; growth beyond the 1-row minimum — expect 98px",
        r.rect.width(),
        r.rect.height(),
        line_count(&values[14])
    ));
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[15])
        .width(300.0)
        .id_salt("s3-narrow")
        .show(ui);
    ui.label(format!(
        "narrow, 300px wide: {:.1} x {:.1} px; {} chars on one line; wrapping grows the box on the §3.2 ladder",
        r.rect.width(),
        r.rect.height(),
        values[15].chars().count()
    ));

    // 8. Hover target (spec §4: no hover rule), with clear space above
    // and below.
    ui.add_space(12.0);
    ui.label(
        "8. Hover target (spec §4: NO hover rule — hovering must render identically to resting)",
    );
    ui.add_space(8.0);
    let r = Textarea::new(&mut values[16]).id_salt("s3-hover").show(ui);
    ui.add_space(8.0);
    ui.label(format!(
        "hover target: {:.1} x {:.1} px; hovered: {}",
        r.rect.width(),
        r.rect.height(),
        r.hovered()
    ));

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn checkbox_demo(ui: &mut egui::Ui) {
    // Header: recipe position, stated honestly (spec §§0, 6, 11).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Checkbox — one size, no variants. Active recipe: {recipe} (style: {}, mode: {mode}).",
        theme.style.name()
    ));
    ui.label("Box 16px, radius 4px, 1px border, 14px icon box, 3px ring (§3). Transition 150ms + cubic-bezier(0.4, 0, 0.2, 1) is the OPEN-8 interim framework default, not snapshot-verified. OPEN-1 Radix key/click semantics settled by S6 Work B (all (a)-(h) closed); OPEN-2 settled — indeterminate renders the Check icon; OPEN-4 combined-state precedence is PROVISIONAL.");

    let store = egui::Id::unique("checkbox_demo_values");
    let mut values: [bool; 13] = ui.ctx().data_mut(|d| {
        d.get_temp(store).unwrap_or([
            false, true, false, false, true, false, true, false, false, true, false, false, false,
        ])
    });

    // 1. Unchecked + checked, resting (spec §4.1). `ui.horizontal` sizes
    // the row to `interact_size.y` (36px) — the row rect is demo chrome;
    // each checkbox allocates its own exact 16px rect, printed below
    // (separator/kbd S3 precedent: print both the row rect and the
    // content rects so S5 judges geometry against the content rect).
    ui.add_space(12.0);
    ui.label("1. Unchecked + checked, resting (spec §4.1; box 16px, radius 4px, 1px border)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let r_off = Checkbox::new(&mut values[0])
            .id_salt("cb_s3_rest_off")
            .show(ui);
        let r_on = Checkbox::new(&mut values[1])
            .id_salt("cb_s3_rest_on")
            .show(ui);
        (r_off.rect, r_on.rect)
    });
    let (off_rect, on_rect) = row.inner;
    ui.label(format!(
        "row rect: {:.1} x {:.1} px (36px row height is the ui.horizontal artifact); unchecked: {:.1} x {:.1} px; checked: {:.1} x {:.1} px; checked state: {}",
        row.response.rect.width(),
        row.response.rect.height(),
        off_rect.width(),
        off_rect.height(),
        on_rect.width(),
        on_rect.height(),
        values[1]
    ));

    // 2. Focus target (spec §4.2): one checkbox with clear space around
    // it, so S5/S6 focus captures are unambiguous.
    ui.add_space(12.0);
    ui.label("2. Focus target (spec §4.2: border `ring` + 3px `ring/50` ring; ring on any egui focus, D1 proposed)");
    ui.add_space(8.0);
    let resp = Checkbox::new(&mut values[2])
        .id_salt("cb_s3_focus")
        .show(ui);
    let focus_rect = resp.rect;
    let focused = resp.has_focus();
    ui.add_space(8.0);
    ui.label(format!(
        "focus target: {:.1} x {:.1} px; checked: {}; focused: {}",
        focus_rect.width(),
        focus_rect.height(),
        values[2],
        focused
    ));

    // 3. Invalid (spec §4.5): the base/nova split is the judgement —
    // nova shows the 3px rest ring unfocused, base shows the destructive
    // border with NO unfocused ring.
    ui.add_space(12.0);
    ui.label("3. Invalid (spec §4.5: nova shows the 3px rest ring unfocused + /50 dark border; base shows the destructive border with NO unfocused ring)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let r_off = Checkbox::new(&mut values[3])
            .invalid(true)
            .id_salt("cb_s3_inv_off")
            .show(ui);
        let r_on = Checkbox::new(&mut values[4])
            .invalid(true)
            .id_salt("cb_s3_inv_on")
            .show(ui);
        (r_off.rect, r_on.rect)
    });
    let (off_rect, on_rect) = row.inner;
    ui.label(format!(
        "row rect: {:.1} x {:.1} px; invalid unchecked: {:.1} x {:.1} px; invalid checked: {:.1} x {:.1} px",
        row.response.rect.width(),
        row.response.rect.height(),
        off_rect.width(),
        off_rect.height(),
        on_rect.width(),
        on_rect.height()
    ));

    // 4. Disabled (spec §4.3): whole element at 50% incl. the faded
    // shadow, no focus chrome. The third box is E1 (indeterminate +
    // disabled): unchecked recipe at 50%, no icon (provisional).
    ui.add_space(12.0);
    ui.label("4. Disabled (spec §4.3: whole element at 50% incl. faded shadow, no focus chrome; third box is E1 indeterminate+disabled)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let r_off = Checkbox::new(&mut values[5])
            .disabled(true)
            .id_salt("cb_s3_dis_off")
            .show(ui);
        let r_on = Checkbox::new(&mut values[6])
            .disabled(true)
            .id_salt("cb_s3_dis_on")
            .show(ui);
        let r_ind = Checkbox::new(&mut values[7])
            .disabled(true)
            .indeterminate(true)
            .id_salt("cb_s3_dis_ind")
            .show(ui);
        (r_off.rect, r_on.rect, r_ind.rect)
    });
    let (off_rect, on_rect, ind_rect) = row.inner;
    ui.label(format!(
        "row rect: {:.1} x {:.1} px; disabled unchecked: {:.1} x {:.1} px; disabled checked: {:.1} x {:.1} px; disabled indeterminate (E1): {:.1} x {:.1} px",
        row.response.rect.width(),
        row.response.rect.height(),
        off_rect.width(),
        off_rect.height(),
        on_rect.width(),
        on_rect.height(),
        ind_rect.width(),
        ind_rect.height()
    ));

    // 5. Indeterminate (spec §4.4 / OPEN-2, settled by S6 Work B):
    // unchecked-recipe box + the Check icon (the Radix Indicator
    // renders in this state with the consumer's CheckIcon; the minus
    // glyph was a port invention, D3 unexercised). S10 verified the
    // render by the coverage model (glyph masks pixel-identical to
    // the checked icon).
    ui.add_space(12.0);
    ui.label("5. Indeterminate (spec §4.4 / OPEN-2 settled: unchecked-recipe box + Check icon, S10-verified by the coverage model)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let r_ind = Checkbox::new(&mut values[8])
            .indeterminate(true)
            .id_salt("cb_s3_ind")
            .show(ui);
        let r_e5 = Checkbox::new(&mut values[9])
            .indeterminate(true)
            .id_salt("cb_s3_e5")
            .show(ui);
        (r_ind.rect, r_e5.rect)
    });
    let (ind_rect, e5_rect) = row.inner;
    ui.label(format!(
        "row rect: {:.1} x {:.1} px; indeterminate unchecked: {:.1} x {:.1} px; E5 indeterminate + checked (renders as checked, OPEN-3 ratified at S8): {:.1} x {:.1} px",
        row.response.rect.width(),
        row.response.rect.height(),
        ind_rect.width(),
        ind_rect.height(),
        e5_rect.width(),
        e5_rect.height()
    ));

    // 6. Bound label + checkbox pair (spec §7.3, settled label rule).
    // Captured-id binding (scoped-widget id rule): the label binds the
    // id captured from the checkbox's returned `Response::id` on this
    // same pass, so the label's forwarding reaches the checkbox's
    // actual widget id. The old demo bound a fresh `Id::unique`, which
    // matched no widget. The second row is the disabled pair: a label
    // click must do nothing (no toggle, no focus).
    ui.add_space(12.0);
    ui.label("6. Bound label + checkbox pair (spec §7.3, settled label rule: label click toggles AND focuses; direct click fires exactly once; disabled pair does nothing)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let resp = Checkbox::new(&mut values[10])
            .id_salt("cb_s3_label")
            .show(ui);
        let lr = Label::new("Enable notifications")
            .for_id(resp.id)
            .show(ui)
            .rect;
        (resp.rect, lr, resp.has_focus())
    });
    let (cb_rect, label_rect, focused) = row.inner;
    ui.label(format!(
        "row rect: {:.1} x {:.1} px; checkbox: {:.1} x {:.1} px; label: {:.1} x {:.1} px; checked: {}; checkbox focused: {}",
        row.response.rect.width(),
        row.response.rect.height(),
        cb_rect.width(),
        cb_rect.height(),
        label_rect.width(),
        label_rect.height(),
        values[10],
        focused
    ));
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let resp = Checkbox::new(&mut values[11])
            .disabled(true)
            .id_salt("cb_s3_label_disabled")
            .show(ui);
        let lr = Label::new("Disabled option").for_id(resp.id).show(ui).rect;
        (resp.rect, lr, resp.has_focus())
    });
    let (cb_rect, label_rect, focused) = row.inner;
    ui.label(format!(
        "disabled pair — row rect: {:.1} x {:.1} px; checkbox: {:.1} x {:.1} px; label: {:.1} x {:.1} px; checked: {}; checkbox focused: {} (label click must do nothing)",
        row.response.rect.width(),
        row.response.rect.height(),
        cb_rect.width(),
        cb_rect.height(),
        label_rect.width(),
        label_rect.height(),
        values[11],
        focused
    ));

    // 7. Hover target (spec §4.9): one checkbox with clear space around
    // it, so S5/S6 hover captures are unambiguous.
    ui.add_space(12.0);
    ui.label("7. Hover target (spec §4.9: NO hover rule — hover must equal resting)");
    ui.add_space(8.0);
    let resp = Checkbox::new(&mut values[12])
        .id_salt("cb_s3_hover")
        .show(ui);
    let hover_rect = resp.rect;
    let hovered = resp.hovered();
    ui.add_space(8.0);
    ui.label(format!(
        "hover target: {:.1} x {:.1} px; checked: {}; hovered: {}",
        hover_rect.width(),
        hover_rect.height(),
        values[12],
        hovered
    ));

    ui.add_space(12.0);
    ui.label("Keyboard (OPEN-1 settled by S6 Work B, S10-verified): Tab moves focus, one stop per checkbox; Space toggles on keyup, once per press; Enter never toggles; arrows/Home/End/Escape do nothing; a direct click focuses AND toggles.");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn radio_group_demo(ui: &mut egui::Ui) {
    // Header: recipe position, stated honestly (spec §§0, 6, 7, 10).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Radio group — single-select circles + labels (D5). Active recipe: {recipe} (style: {}, mode: {mode}).",
        theme.style.name()
    ));
    ui.label("Keyboard model = S6-settled (OPEN-1–OPEN-4 closed from Radix source) + S10-verified (554/554 rows, all 4 configs): roving tabindex, one Tab stop per group; all four arrows navigate with wrap + disabled-skip; Home/End/PageUp/PageDown are focus-only jump keys; Space fires on keyup, Enter never activates, Escape is a no-op. Labels render via Label composition (D5 PROPOSED: 8px circle–label gap, 16px rows). Focus+invalid precedence is DECIDED (gate G3: invalid-wins both halves). Transition timing 150ms + cubic-bezier(0.4, 0, 0.2, 1) is INTERIM (OPEN-6). Ring shows on any focus (D1 proposed). No hover rule: hovering changes nothing.");

    // Group focus readout helper. `RadioGroup::show` returns
    // `InnerResponse<()>` — no per-item `Response` exists to capture ids
    // from — so the readout checks `Memory::has_focus` against the item
    // ids, which follow the S1-documented scheme: `group_id =
    // ui.scope_id().with(salt)` computed from the `Ui` passed to `show`,
    // `item_id = group_id.with(i)`. This is NOT the Input child-scope
    // case (Label F1): no child scope intervenes between this `Ui` and
    // the group id computation, and the label demo §8c relies on the
    // same scheme for the standalone item. Verified empirically in the
    // S3 smoke run (click → `focused: true`).
    let group_focused = |ui: &egui::Ui, salt: &str, n: usize| -> bool {
        let group_id = ui.scope_id().with(salt);
        ui.ctx()
            .memory(|m| (0..n).any(|i| m.has_focus(group_id.with(i))))
    };

    let store = egui::Id::unique("radio_s3_values");
    let mut values: [Option<usize>; 13] = ui.ctx().data_mut(|d| {
        d.get_temp(store).unwrap_or([
            Some(1),
            Some(0),
            None,
            Some(0),
            None,
            Some(0),
            Some(1),
            Some(0),
            Some(0),
            Some(1),
            Some(0),
            Some(0),
            Some(0),
        ])
    });
    let standalone_store = egui::Id::unique("radio_s3_standalone");
    let mut standalone: [bool; 2] = ui
        .ctx()
        .data_mut(|d| d.get_temp(standalone_store).unwrap_or([true, false]));

    // 1. Labelled groups (N2 Option C, D5 proposed). Rows are 16px tall;
    // the group rect below lets S5 derive the row height (3 rows:
    // 64.0px nova / 72.0px base = 3×16 + 2×gap). The 8px circle–label
    // gap is S5's pixel job — no per-row rect is exposed.
    ui.add_space(12.0);
    ui.label("1. Labelled groups (N2 Option C; D5 proposed: 8px gap, 16px rows; label per frozen label spec)");
    ui.add_space(4.0);
    let g1 = RadioGroup::new(&mut values[0])
        .id_salt("s3-labelled")
        .option("Default")
        .option("Comfortable")
        .option("Compact")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "labelled group: {:.1} x {:.1} px; selected: {:?} (expected: 3 rows, 64.0 nova / 72.0 base tall)",
        g1.width(),
        g1.height(),
        values[0]
    ));
    ui.add_space(8.0);
    let g1b = RadioGroup::new(&mut values[1])
        .id_salt("s3-empty-label")
        .option("First")
        .option("")
        .option("Third")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "empty-label group: {:.1} x {:.1} px; selected: {:?} (middle option renders circle only, no gap)",
        g1b.width(),
        g1b.height(),
        values[1]
    ));

    // 2. Checked looks: nova solid primary disc + primary-foreground dot
    // vs base primary dot on the resting fill; standalone items shown
    // separately (unaffected by N2); a no-selection group shows no dot.
    ui.add_space(12.0);
    ui.label("2. Checked looks (spec §2.7; standalone items unaffected by N2; no-selection group shows no dot)");
    ui.add_space(4.0);
    let r_checked = RadioGroupItem::new(&mut standalone[0])
        .id_salt("s3-standalone-checked")
        .show(ui)
        .rect;
    ui.label(format!(
        "standalone checked: {:.1} x {:.1} px; checked: {}",
        r_checked.width(),
        r_checked.height(),
        standalone[0]
    ));
    ui.add_space(4.0);
    let r_unchecked = RadioGroupItem::new(&mut standalone[1])
        .id_salt("s3-standalone-unchecked")
        .show(ui)
        .rect;
    ui.label(format!(
        "standalone unchecked: {:.1} x {:.1} px; checked: {}",
        r_unchecked.width(),
        r_unchecked.height(),
        standalone[1]
    ));
    ui.add_space(8.0);
    let g2 = RadioGroup::new(&mut values[2])
        .id_salt("s3-no-selection")
        .option("Alpha")
        .option("Beta")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "no-selection group: {:.1} x {:.1} px; selected: {:?} (no dot anywhere)",
        g2.width(),
        g2.height(),
        values[2]
    ));

    // 3. Focus target, with clear space above and below. The live
    // `focused:` readout prints so S6 captures can confirm focus.
    ui.add_space(12.0);
    ui.label(
        "3. Focus target (spec §4: border ring + 3px ring/50 ring on focus; D1: ring on any focus)",
    );
    ui.add_space(4.0);
    let g3 = RadioGroup::new(&mut values[3])
        .id_salt("s3-focus")
        .option("Red")
        .option("Green")
        .option("Blue")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "focus target: {:.1} x {:.1} px; selected: {:?}; focused: {}",
        g3.width(),
        g3.height(),
        values[3],
        group_focused(ui, "s3-focus", 3)
    ));

    // 4. Invalid: nova shows the 3px destructive rest ring; base shows
    // the destructive border only. Invalid+checked keeps the primary
    // border under nova (checked wins). Focus+invalid precedence is
    // DECIDED (gate G3: invalid-wins both halves; S10 verified Tab 82
    // in all 4 configs).
    ui.add_space(12.0);
    ui.label("4. Invalid (spec §4; nova: 3px rest ring; base: border only; invalid+checked keeps primary border under nova; focus+invalid precedence DECIDED — gate G3, invalid-wins)");
    ui.add_space(4.0);
    let g4a = RadioGroup::new(&mut values[4])
        .id_salt("s3-invalid")
        .invalid(true)
        .option("One")
        .option("Two")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "invalid unchecked: {:.1} x {:.1} px; selected: {:?}",
        g4a.width(),
        g4a.height(),
        values[4]
    ));
    ui.add_space(8.0);
    let g4b = RadioGroup::new(&mut values[5])
        .id_salt("s3-invalid-checked")
        .invalid(true)
        .option("One")
        .option("Two")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "invalid + checked: {:.1} x {:.1} px; selected: {:?} (checked wins for border under nova)",
        g4b.width(),
        g4b.height(),
        values[5]
    ));
    ui.add_space(8.0);
    let g4c = RadioGroup::new(&mut values[6])
        .id_salt("s3-invalid-disabled")
        .invalid(true)
        .disabled(true)
        .option("One")
        .option("Two")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "invalid + disabled: {:.1} x {:.1} px; selected: {:?}",
        g4c.width(),
        g4c.height(),
        values[6]
    ));

    // 5. Disabled: whole-element 50% fade; a group with one disabled
    // option (middle) exposes the per-option case (spec §9).
    ui.add_space(12.0);
    ui.label("5. Disabled (spec §4: whole element at 50% opacity; disabled items take no focus)");
    ui.add_space(4.0);
    let g5a = RadioGroup::new(&mut values[7])
        .id_salt("s3-disabled")
        .disabled(true)
        .option("Enabled look, disabled group")
        .option("Second option")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "disabled group: {:.1} x {:.1} px; selected: {:?} (checked + unchecked options, both faded)",
        g5a.width(),
        g5a.height(),
        values[7]
    ));
    ui.add_space(8.0);
    let g5b = RadioGroup::new(&mut values[8])
        .id_salt("s3-option-disabled")
        .option("Push notifications")
        .option_disabled("Email digest (unavailable)")
        .option("Nothing")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "one disabled option: {:.1} x {:.1} px; selected: {:?} (middle option faded, skipped by keys per settled OPEN-1 — S6 Work B, S10-verified)",
        g5b.width(),
        g5b.height(),
        values[8]
    ));

    // 6. Horizontal (D4 Rust extension). The group itself must measure
    // exactly 16px tall — the S1 fix replaced `ui.horizontal` (which
    // seeds a 36px row) with a scope_builder + left-to-right layout.
    // This demo stacks vertically and never uses `ui.horizontal`.
    ui.add_space(12.0);
    ui.label("6. Horizontal (D4 extension; gap 12px base / 8px nova between options; group height must be exactly 16px)");
    ui.add_space(4.0);
    let g6 = RadioGroup::new(&mut values[9])
        .id_salt("s3-horizontal")
        .orientation(GroupOrientation::Horizontal)
        .option("Day")
        .option("Month")
        .option("Year")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "horizontal group: {:.1} x {:.1} px; selected: {:?} (expected height: exactly 16.0)",
        g6.width(),
        g6.height(),
        values[9]
    ));

    // 7. Hover target, with clear space above and below. The live
    // `hovered:` readout prints; hovering must equal resting (no hover
    // rule in either recipe).
    ui.add_space(12.0);
    ui.label("7. Hover target (no hover rule in either recipe: hovering must equal resting, pixel-identical)");
    ui.add_space(4.0);
    let r7 = RadioGroup::new(&mut values[10])
        .id_salt("s3-hover")
        .option("Hover one")
        .option("Hover two")
        .show(ui)
        .response;
    // Hover readout via rect containment, NOT `Response::hovered()`: the
    // group response is the scope Ui's response, and egui's hover set
    // holds the top interactive widget under the pointer (the item
    // circles/labels), so the scope response reports `hovered() == false`
    // even with the pointer parked on the group (verified empirically in
    // the S3 smoke run). Rect containment is the signal S6 needs: is the
    // pointer over the target while the render must equal resting.
    let hovered = ui.rect_contains_pointer(r7.rect);
    ui.label(format!(
        "hover target: {:.1} x {:.1} px; selected: {:?}; hovered: {}",
        r7.rect.width(),
        r7.rect.height(),
        values[10],
        hovered
    ));

    // 8. Suppression (D6 mechanism): one group with
    // `.suppress_focus_indication(true)` + one without. Nova-only; the
    // field line owns the wiring contract — this demo only exposes the
    // mechanism for S6 to drive focus against. Under base the builder
    // is a no-op and both groups must render identically.
    ui.add_space(12.0);
    ui.label("8. Suppression, D6 mechanism (nova-only; field line owns the wiring contract; under base both groups render identically)");
    ui.add_space(4.0);
    let g8a = RadioGroup::new(&mut values[11])
        .id_salt("s3-suppressed")
        .suppress_focus_indication(true)
        .option("Suppressed A")
        .option("Suppressed B")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "suppressed group: {:.1} x {:.1} px; selected: {:?}; focused: {} (no ring when focused under nova)",
        g8a.width(),
        g8a.height(),
        values[11],
        group_focused(ui, "s3-suppressed", 2)
    ));
    ui.add_space(8.0);
    let g8b = RadioGroup::new(&mut values[12])
        .id_salt("s3-unsuppressed")
        .option("Plain A")
        .option("Plain B")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "unsuppressed group: {:.1} x {:.1} px; selected: {:?}; focused: {} (ring renders on focus)",
        g8b.width(),
        g8b.height(),
        values[12],
        group_focused(ui, "s3-unsuppressed", 2)
    ));

    ui.add_space(12.0);
    ui.label("Keyboard: Tab/Shift+Tab move focus (one stop per group — roving-tabindex entry rules settled OPEN-4, S10-verified: entry lands on the checked item, else the stored stop, and never checks). All four arrows navigate with wrap + disabled-skip and selection following (settled OPEN-1); Home/End/PageUp/PageDown are focus-only jump keys (settled OPEN-2); Space fires on keyup, Enter never activates, Escape is a no-op (settled OPEN-3, S10-verified). A label click fires a click AND focuses (settled, spec §7.3); a direct click selects AND focuses (RAD-INT-06, S10-verified) and never unchecks.");

    ui.ctx().data_mut(|d| {
        d.insert_temp(store, values);
        d.insert_temp(standalone_store, standalone);
    });
}

pub fn switch_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("switch_demo_values");
    // One bool per switch, so S5/S6 can toggle each section independently.
    // Order: [sizes default off/on, sizes sm off/on, invalid default off/on,
    // invalid sm off/on, disabled off/on, disabled+invalid, focus target,
    // label pair, card suppressed, card contrast, hover target].
    let mut values: [bool; 16] = ui.ctx().data_mut(|d| {
        d.get_temp(store).unwrap_or([
            false, true, false, true, false, true, false, true, false, true, false, false, false,
            false, false, false,
        ])
    });

    // Header: recipe position, stated honestly (spec §§2.5, 6, 7). This
    // header is also the brief's section 2 (theme x recipe): the demo runs
    // under the gallery's active theme/style, printed here so every capture
    // is self-describing.
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Switch — track + thumb, two sizes. Active recipe: {recipe} (style: {}, mode: {mode}).",
        theme.style.name()
    ));
    ui.label("Nova geometry is identical to base in every dimension (spec §2.5); the only nova deltas are the invalid recipe (§2.6) and the field-label focus suppression (D4, PROPOSED). Transition timing 150ms + cubic-bezier(0.4, 0, 0.2, 1) is the OPEN-1 interim framework default, not snapshot-verified. Keyboard rules are OPEN-2 (Radix, S6 Work B); focus-on-click is OPEN-3.");

    // NOTE on row rects (separator/kbd S3 precedent): `ui.horizontal` sizes
    // the row to `interact_size.y` (36px). Each switch allocates its own
    // exact track rect, printed per switch below; the row rect is demo
    // chrome and is printed alongside only where it aids S5.

    // 1. Sizes (spec §3): default + sm, unchecked + checked.
    ui.add_space(12.0);
    ui.label("1. Sizes (spec §3; default 32 x 18.4, sm 24 x 14; travel 14 / 10)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let r0 = Switch::new(&mut values[0])
            .id_salt("sw_demo_def_off")
            .show(ui);
        let r1 = Switch::new(&mut values[1])
            .id_salt("sw_demo_def_on")
            .show(ui);
        let r2 = Switch::new(&mut values[2])
            .size(SwitchSize::Sm)
            .id_salt("sw_demo_sm_off")
            .show(ui);
        let r3 = Switch::new(&mut values[3])
            .size(SwitchSize::Sm)
            .id_salt("sw_demo_sm_on")
            .show(ui);
        [r0, r1, r2, r3]
    });
    let rects = row.inner;
    ui.label(format!(
        "row {:.1} x {:.1} px (the 36px row height is the ui.horizontal artifact); default off {:.1} x {:.1}, on {:.1} x {:.1} (expected 32.0 x 18.4, travel {:.1}); sm off {:.1} x {:.1}, on {:.1} x {:.1} (expected 24.0 x 14.0, travel {:.1})",
        row.response.rect.width(),
        row.response.rect.height(),
        rects[0].rect.width(),
        rects[0].rect.height(),
        rects[1].rect.width(),
        rects[1].rect.height(),
        SwitchSize::Default.checked_offset(),
        rects[2].rect.width(),
        rects[2].rect.height(),
        rects[3].rect.width(),
        rects[3].rect.height(),
        SwitchSize::Sm.checked_offset(),
    ));

    // 3. Invalid (spec §2.6 / OPEN-10): unchecked + checked, both sizes.
    ui.add_space(12.0);
    ui.label("3. Invalid (spec §2.6 / OPEN-10: nova shows a 3px rest ring + destructive border, dark border /50; under base, invalid paints nothing)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let r0 = Switch::new(&mut values[4])
            .invalid(true)
            .id_salt("sw_demo_inv_def_off")
            .show(ui);
        let r1 = Switch::new(&mut values[5])
            .invalid(true)
            .id_salt("sw_demo_inv_def_on")
            .show(ui);
        let r2 = Switch::new(&mut values[6])
            .size(SwitchSize::Sm)
            .invalid(true)
            .id_salt("sw_demo_inv_sm_off")
            .show(ui);
        let r3 = Switch::new(&mut values[7])
            .size(SwitchSize::Sm)
            .invalid(true)
            .id_salt("sw_demo_inv_sm_on")
            .show(ui);
        [r0, r1, r2, r3]
    });
    let rects = row.inner;
    ui.label(format!(
        "invalid default off {:.1} x {:.1}, on {:.1} x {:.1}; invalid sm off {:.1} x {:.1}, on {:.1} x {:.1} (geometry identical to section 1; the ring paints outside the border box and changes no layout)",
        rects[0].rect.width(),
        rects[0].rect.height(),
        rects[1].rect.width(),
        rects[1].rect.height(),
        rects[2].rect.width(),
        rects[2].rect.height(),
        rects[3].rect.width(),
        rects[3].rect.height(),
    ));

    // 4. Disabled (spec §4): unchecked + checked + invalid+disabled.
    ui.add_space(12.0);
    ui.label("4. Disabled (spec §4: the whole element composites at 50%, no focus chrome, not-allowed cursor; invalid+disabled renders the invalid chrome at 50% under nova)");
    ui.add_space(4.0);
    let row = ui.horizontal(|ui| {
        let r0 = Switch::new(&mut values[8])
            .disabled(true)
            .id_salt("sw_demo_dis_off")
            .show(ui);
        let r1 = Switch::new(&mut values[9])
            .disabled(true)
            .id_salt("sw_demo_dis_on")
            .show(ui);
        let r2 = Switch::new(&mut values[10])
            .disabled(true)
            .invalid(true)
            .id_salt("sw_demo_dis_inv")
            .show(ui);
        [r0, r1, r2]
    });
    let rects = row.inner;
    ui.label(format!(
        "disabled off {:.1} x {:.1}, on {:.1} x {:.1}, invalid+disabled {:.1} x {:.1}",
        rects[0].rect.width(),
        rects[0].rect.height(),
        rects[1].rect.width(),
        rects[1].rect.height(),
        rects[2].rect.width(),
        rects[2].rect.height(),
    ));

    // 5. Focus target (spec §4): one switch with clear space around it.
    ui.add_space(12.0);
    ui.label("5. Focus target (spec §4: focus shows border `ring` + 3px `ring/50` ring; Tab here or click to focus)");
    ui.add_space(8.0);
    let resp = Switch::new(&mut values[11])
        .id_salt("sw_demo_focus")
        .show(ui);
    ui.add_space(8.0);
    ui.label(format!(
        "track {:.1} x {:.1} px; on: {}; focused: {}",
        resp.rect.width(),
        resp.rect.height(),
        values[11],
        resp.has_focus()
    ));

    // 6. Bound label + switch pair (spec §7.3/§7.5; the label demo's
    // binding pattern: the label's `for_id` is the switch's scoped id).
    ui.add_space(12.0);
    ui.label("6. Bound label + switch (spec §7.3/§7.5: a label click forwards activation AND focuses the switch; a direct click on the switch fires exactly once)");
    ui.add_space(4.0);
    let (label_rect, resp) = ui
        .horizontal(|ui| {
            let target = ui.scope_id().with("sw_demo_label");
            let lr = Label::new("Airplane mode").for_id(target).show(ui).rect;
            let resp = Switch::new(&mut values[12])
                .id_salt("sw_demo_label")
                .show(ui);
            (lr, resp)
        })
        .inner;
    ui.label(format!(
        "label {:.1} x {:.1} px; switch {:.1} x {:.1} px; on: {}; switch focused: {}",
        label_rect.width(),
        label_rect.height(),
        resp.rect.width(),
        resp.rect.height(),
        values[12],
        resp.has_focus()
    ));

    // 7. Card composition, focus suppression ON (spec §2.3 / D4 PROPOSED,
    // nova-only). The flag is pinned ON in this section so S6 can verify
    // the mechanism end-to-end: focus the inner switch (Tab or a direct
    // click) and the card carries the focus indication while the switch
    // shows no ring and a transparent border. The focus-inside-driven
    // wiring is the field line's composition contract, not this demo's;
    // no `for_id` is bound here because card-click focus is field-line
    // behaviour (Label F2). The second card below is the suppression-OFF
    // contrast: the same composition with the flag off.
    ui.add_space(12.0);
    ui.label("7a. Card composition, suppression ON (D4 PROPOSED, nova-only: inner switch shows no ring; the card carries focus)");
    ui.add_space(4.0);
    let card_on = FieldLabel::new("")
        .checked(Some(values[13]))
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    let resp = Switch::new(&mut values[13])
                        .id_salt("sw_demo_card_on")
                        .suppress_focus_indication(true)
                        .show(ui);
                    FieldContent::new().show(ui, |ui| {
                        FieldTitle::new("Notifications").show(ui);
                        FieldDescription::new(
                            "Suppression ON: this switch never shows its own focus ring.",
                        )
                        .show(ui);
                    });
                    resp
                })
                .inner
        });
    ui.label(format!(
        "card {:.1} x {:.1} px; switch {:.1} x {:.1} px; on: {}; switch focused: {}",
        card_on.response.rect.width(),
        card_on.response.rect.height(),
        card_on.inner.rect.width(),
        card_on.inner.rect.height(),
        values[13],
        card_on.inner.has_focus()
    ));
    ui.add_space(8.0);
    ui.label("7b. Card composition, suppression OFF (contrast: the inner switch shows its own focus ring)");
    ui.add_space(4.0);
    let card_off = FieldLabel::new("")
        .checked(Some(values[14]))
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    let resp = Switch::new(&mut values[14])
                        .id_salt("sw_demo_card_off")
                        .show(ui);
                    FieldContent::new().show(ui, |ui| {
                        FieldTitle::new("Marketing emails").show(ui);
                        FieldDescription::new(
                            "Suppression OFF: this switch shows its own focus ring.",
                        )
                        .show(ui);
                    });
                    resp
                })
                .inner
        });
    ui.label(format!(
        "card {:.1} x {:.1} px; switch {:.1} x {:.1} px; on: {}; switch focused: {}",
        card_off.response.rect.width(),
        card_off.response.rect.height(),
        card_off.inner.rect.width(),
        card_off.inner.rect.height(),
        values[14],
        card_off.inner.has_focus()
    ));

    // 8. Hover target (spec §4): one switch with clear space around it.
    ui.add_space(12.0);
    ui.label("8. Hover target (spec §4: NO hover rule in either recipe — hover must render identically to resting)");
    ui.add_space(8.0);
    let resp = Switch::new(&mut values[15])
        .id_salt("sw_demo_hover")
        .show(ui);
    ui.add_space(8.0);
    ui.label(format!(
        "track {:.1} x {:.1} px; on: {}; hovered: {}",
        resp.rect.width(),
        resp.rect.height(),
        values[15],
        resp.hovered()
    ));

    ui.add_space(12.0);
    ui.label("Keyboard: Tab moves focus between switches. Which keys activate a focused switch is OPEN-2 (Radix source, S6 Work B) — the current click-sense activation is implementation, not spec truth. Whether a pointer click focuses the switch is OPEN-3 (incl. OBS-LBL-S6-2).");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn slider_demo(ui: &mut egui::Ui) {
    // Demo-local state, held in egui temp data (the gallery's established
    // demo-state pattern). All stored values are ABSOLUTE values (spec
    // §7.1); no fraction is ever stored or printed.
    #[derive(Clone)]
    struct SliderDemoState {
        single: f32,
        range: (f32, f32),
        stepped: f32,
        f2: f32,
        f3: f32,
        f4: f32,
        disabled_single: f32,
        disabled_range: (f32, f32),
        vertical: f32,
        vertical_range: (f32, f32),
        keyboard: f32,
        pointer: (f32, f32),
    }

    impl Default for SliderDemoState {
        fn default() -> Self {
            Self {
                single: 30.0,
                range: (0.0, 100.0),
                stepped: 30.0,
                f2: 25.0,
                f3: 5.0,
                f4: 0.0,
                disabled_single: 50.0,
                disabled_range: (30.0, 70.0),
                vertical: 60.0,
                vertical_range: (25.0, 75.0),
                keyboard: 50.0,
                pointer: (30.0, 70.0),
            }
        }
    }

    let store = egui::Id::unique("slider_demo_values");
    let mut state: SliderDemoState = ui.ctx().data_mut(|d| d.get_temp(store)).unwrap_or_default();

    // Header: recipe position, stated honestly (spec §§2.3, 6, 8).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Slider — Root + Track + Range + thumbs, one widget. Active recipe: {recipe} (style: {}, mode: {mode}).",
        theme.style.name()
    ));
    ui.label("Geometry (§2.3): base track 6px / thumb 16px / ring 4px / hit = thumb box; nova track 4px / thumb 12px / ring 3px / hit 28px. Vertical min-height 176px base / 160px nova (§3.2). Thumb fill is white in both modes (§4.5). No invalid state exists (§4.6).");
    ui.label("OPEN-11: transition timing 150ms + cubic-bezier(0.4, 0, 0.2, 1) is the interim Tailwind framework default, not snapshot-verified (§6). Keyboard/pointer behaviour: OPEN-1…OPEN-10 CLOSED (gate G11) and S10-verified. Readouts print ABSOLUTE values only (§7.1).");

    // Thumb-centre readout helpers: the thumb CENTRE travels the full Track
    // length (spec §3.3), and the Track spans the full allocated rect, so
    // each centre follows from the allocated rect + the absolute value.
    // The fraction is an internal display computation only; it is never
    // stored or printed.
    let h_centre = |rect: egui::Rect, value: f32, min: f32, max: f32| -> egui::Pos2 {
        let fraction = (value - min) / (max - min);
        egui::Pos2::new(rect.left() + fraction * rect.width(), rect.center().y)
    };
    let v_centre = |rect: egui::Rect, value: f32, min: f32, max: f32| -> egui::Pos2 {
        let fraction = (value - min) / (max - min);
        egui::Pos2::new(rect.center().x, rect.bottom() - fraction * rect.height())
    };

    // 1. Single + range, resting (spec §§2–3).
    ui.add_space(12.0);
    ui.label("1. Single + range, resting (spec §§2–3; at the extremes the thumb overhangs the Track end by half its diameter, §3.3)");
    ui.add_space(4.0);
    let single_rect = Slider::new(&mut state.single)
        .min(0.0)
        .max(100.0)
        .id_salt("s3-single")
        .show(ui)
        .rect;
    let centre = h_centre(single_rect, state.single, 0.0, 100.0);
    ui.small(format!(
        "single: allocated {:.1} x {:.1} px; thumb centre ({:.1}, {:.1}); value: {}",
        single_rect.width(),
        single_rect.height(),
        centre.x,
        centre.y,
        state.single
    ));
    ui.add_space(4.0);
    let range_rect = Slider::range(&mut state.range)
        .min(0.0)
        .max(100.0)
        .id_salt("s3-range")
        .show(ui)
        .rect;
    let lo = h_centre(range_rect, state.range.0, 0.0, 100.0);
    let hi = h_centre(range_rect, state.range.1, 0.0, 100.0);
    ui.small(format!(
        "range at [min, max]: allocated {:.1} x {:.1} px; thumb centres ({:.1}, {:.1}) and ({:.1}, {:.1}); values: {} – {}",
        range_rect.width(),
        range_rect.height(),
        lo.x,
        lo.y,
        hi.x,
        hi.y,
        state.range.0,
        state.range.1
    ));

    // 2. Stepped + §7.6 discriminating fixtures (S6 drives the exact
    // keyboard table against these sliders).
    ui.add_space(12.0);
    ui.label("2. Stepped + §7.6 discriminating fixtures (S6 drives the exact keyboard table against these)");
    ui.add_space(4.0);
    ui.label("Stepped, step 10 (0–100):");
    let stepped_rect = Slider::new(&mut state.stepped)
        .min(0.0)
        .max(100.0)
        .step(10.0)
        .id_salt("s3-stepped")
        .show(ui)
        .rect;
    ui.small(format!(
        "stepped: allocated {:.1} x {:.1} px; value: {}",
        stepped_rect.width(),
        stepped_rect.height(),
        state.stepped
    ));
    ui.add_space(4.0);
    ui.label("F2 (min 20, max 40, step 5, v=25): ArrowRight→30, ArrowLeft→20, Home→20, End→40:");
    let f2_rect = Slider::new(&mut state.f2)
        .min(20.0)
        .max(40.0)
        .step(5.0)
        .id_salt("s3-f2")
        .show(ui)
        .rect;
    ui.small(format!(
        "F2: allocated {:.1} x {:.1} px; value: {}",
        f2_rect.width(),
        f2_rect.height(),
        state.f2
    ));
    ui.add_space(4.0);
    ui.label("F3 (min 0, max 10, step 0.5, v=5): ArrowRight→5.5:");
    let f3_rect = Slider::new(&mut state.f3)
        .min(0.0)
        .max(10.0)
        .step(0.5)
        .id_salt("s3-f3")
        .show(ui)
        .rect;
    ui.small(format!(
        "F3: allocated {:.1} x {:.1} px; value: {}",
        f3_rect.width(),
        f3_rect.height(),
        state.f3
    ));
    ui.add_space(4.0);
    ui.label("F4 (min −50, max 50, step 10, v=0): ArrowLeft→−10:");
    let f4_rect = Slider::new(&mut state.f4)
        .min(-50.0)
        .max(50.0)
        .step(10.0)
        .id_salt("s3-f4")
        .show(ui)
        .rect;
    ui.small(format!(
        "F4: allocated {:.1} x {:.1} px; value: {}",
        f4_rect.width(),
        f4_rect.height(),
        state.f4
    ));

    // 3. Disabled (spec §4.4 N8).
    ui.add_space(12.0);
    ui.label("3. Disabled (spec §4.4 N8: uniform single 50% fade — every part at the same effective 50%, no part at 25%; no ring; no Tab stop, W-10)");
    ui.add_space(4.0);
    let dis_rect = Slider::new(&mut state.disabled_single)
        .min(0.0)
        .max(100.0)
        .disabled(true)
        .id_salt("s3-disabled-single")
        .show(ui)
        .rect;
    ui.small(format!(
        "disabled single: allocated {:.1} x {:.1} px; value: {}",
        dis_rect.width(),
        dis_rect.height(),
        state.disabled_single
    ));
    ui.add_space(4.0);
    let dis_range_rect = Slider::range(&mut state.disabled_range)
        .min(0.0)
        .max(100.0)
        .disabled(true)
        .id_salt("s3-disabled-range")
        .show(ui)
        .rect;
    ui.small(format!(
        "disabled range: allocated {:.1} x {:.1} px; values: {} – {}",
        dis_range_rect.width(),
        dis_range_rect.height(),
        state.disabled_range.0,
        state.disabled_range.1
    ));

    // 4. Vertical (spec §3.2). NOTE: a vertical slider stretches to the
    // OFFERED height (`h-full`), and inside the gallery ScrollArea the
    // offered height is the viewport height — so the row height is set
    // explicitly via `allocate_ui_with_layout` (separator S3 precedent):
    // 200px, above both floors (176px base / 160px nova), so the offered
    // height — not the floor — determines the rendered height in both
    // styles. The row's response rect is the content union (the requested
    // 120px width shrink-wraps to the two 24px sliders + spacing); each
    // slider's allocated rect and thumb centres are printed below. Do NOT
    // change the component to fit the demo.
    ui.add_space(12.0);
    ui.label("4. Vertical (spec §3.2: h-full with the 176px base / 160px nova floor; values increase bottom-up)");
    ui.add_space(4.0);
    let mut vertical = state.vertical;
    let mut vertical_pair = state.vertical_range;
    let row = ui.allocate_ui_with_layout(
        egui::Vec2::new(120.0, 200.0),
        egui::Layout::left_to_right(egui::Align::Min),
        |ui| {
            let r1 = Slider::new(&mut vertical)
                .min(0.0)
                .max(100.0)
                .orientation(GroupOrientation::Vertical)
                .id_salt("s3-vertical")
                .show(ui)
                .rect;
            let r2 = Slider::range(&mut vertical_pair)
                .min(0.0)
                .max(100.0)
                .orientation(GroupOrientation::Vertical)
                .id_salt("s3-vertical-range")
                .show(ui)
                .rect;
            (r1, r2)
        },
    );
    state.vertical = vertical;
    state.vertical_range = vertical_pair;
    let row_rect = row.response.rect;
    let (v_single_rect, v_range_rect) = row.inner;
    let v_single_centre = v_centre(v_single_rect, state.vertical, 0.0, 100.0);
    let v_lo = v_centre(v_range_rect, state.vertical_range.0, 0.0, 100.0);
    let v_hi = v_centre(v_range_rect, state.vertical_range.1, 0.0, 100.0);
    ui.small(format!(
        "vertical row: {:.1} x {:.1} px (row rect = content union; 200px height offered explicitly); single {:.1} x {:.1} px, thumb centre ({:.1}, {:.1}), value: {}; range {:.1} x {:.1} px, thumb centres ({:.1}, {:.1}) and ({:.1}, {:.1}), values: {} – {}",
        row_rect.width(),
        row_rect.height(),
        v_single_rect.width(),
        v_single_rect.height(),
        v_single_centre.x,
        v_single_centre.y,
        state.vertical,
        v_range_rect.width(),
        v_range_rect.height(),
        v_lo.x,
        v_lo.y,
        v_hi.x,
        v_hi.y,
        state.vertical_range.0,
        state.vertical_range.1
    ));

    // 5. Keyboard target (spec §8.1; S6 drives it).
    ui.add_space(16.0);
    ui.label("5. Keyboard target (spec §8.1 — S6 drives it; F1 fixture: 0–100, step 1, v=50)");
    ui.label(
        "Arrows ±1, Home→0, End→100, PageUp/PageDown ±10 (W-2), clamp at the bounds, no wrap (F6).",
    );
    ui.add_space(4.0);
    let kb = Slider::new(&mut state.keyboard)
        .min(0.0)
        .max(100.0)
        .step(1.0)
        .id_salt("s3-keyboard")
        .show(ui);
    ui.small(format!(
        "keyboard target: allocated {:.1} x {:.1} px; value: {}; focused: {}",
        kb.rect.width(),
        kb.rect.height(),
        state.keyboard,
        kb.has_focus()
    ));
    ui.add_space(16.0);

    // 6. Pointer target (spec §8.2; S6 drives it).
    ui.label(
        "6. Pointer target (spec §8.2 — S6 drives it; F5a values: range 0–100, step 10, [30, 70])",
    );
    ui.label("Press on the Track moves the NEAREST thumb (W-6a) and begins a drag (W-6b); right/middle-click and wheel have no effect (W-6d/f).");
    ui.add_space(4.0);
    let ptr = Slider::range(&mut state.pointer)
        .min(0.0)
        .max(100.0)
        .step(10.0)
        .id_salt("s3-pointer")
        .show(ui);
    ui.small(format!(
        "pointer target: allocated {:.1} x {:.1} px; values: {} – {}; hovered: {}; dragged: {}",
        ptr.rect.width(),
        ptr.rect.height(),
        state.pointer.0,
        state.pointer.1,
        ptr.hovered(),
        ptr.dragged()
    ));
    ui.add_space(16.0);

    // 7. Hit-area note (spec §3.5).
    ui.label("7. Hit areas (spec §3.5): nova 28×28px — thumb box + 8px on all sides (after:-inset-2); base 16×16px thumb box. Hover/focus/drag sensing uses the hit rect; paint stays the thumb box. S6 verifies with pointer probes. Overlapping thumbs: nearest centre wins (W-6c); exact ties go to the lowest index (tie-first, OPEN-6 CLOSED, G11).");

    ui.add_space(12.0);
    ui.label("Keyboard: Tab moves between thumbs (W-9, INT-05: every thumb a Tab stop, no roving); arrows step 1×, Shift+Arrow 10× (OPEN-3 CLOSED, INT-01), Home/End target first/last index (OPEN-4 CLOSED, G7), Escape is a no-op on a focused thumb (INT-07). The ring shows on any focus (deviation D2, PROPOSED).");

    ui.ctx().data_mut(|d| d.insert_temp(store, state));
}

pub fn toggle_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the toggle actually allocated
    // (`Response.rect`), so S5/S6 can verify geometry from screenshots.
    // Header: recipe position, stated honestly (spec §§0, 2.4, 3, 4, 6, 10, 11).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    let gap = if nova { 4 } else { 8 };
    ui.label(format!(
        "Toggle — pressed-state button. Active recipe: {recipe} (style: {}, mode: {mode}). Heights 32/28/36px nova, 36/32/40px base (default/sm/lg).",
        theme.style.name()
    ));
    ui.label("S-1: outline shadow-xs paints under ALL styles incl. nova. N16: the .icon_inline_start builder (§3) is the RATIFIED trigger mapping (gate G3) — no trigger is inferred; OPEN-6 stays OPEN. OPEN-3 (pressed+hover) precedence stays PROVISIONAL; OPEN-4 (invalid+focus) is DECIDED invalid-wins (gate G4). Transition timing 150ms + cubic-bezier(0.4, 0, 0.2, 1) is INTERIM (OPEN-5); property sets per ruling 6 (base: text+ring animate, fill/border snap; nova: transition-all). Hover fill/text per §4.2 only. Keyboard: Space fires on keyup, Enter on keydown (OPEN-1/OPEN-2 CLOSED by S6 Work B); click focuses + toggles; Escape is a no-op.");

    // Index layout: 0-7 §1 variants, 8-11 §2 gap, 12-17 §3 icon-inline,
    // 18 §4 focus, 19-22 §5 invalid, 23-27 §6 disabled, 28 §7 hover,
    // 29-30 §8 pressed+hover. Pressed rows start pressed.
    let store = egui::Id::unique("toggle_demo_values");
    let mut values: [bool; 31] = ui.ctx().data_mut(|d| {
        d.get_temp(store).unwrap_or([
            false, true, false, true, false, true, false, true, false, false, false, false, false,
            false, false, false, false, false, false, false, true, false, true, false, true, false,
            true, true, false, true, true,
        ])
    });

    // 1. Variants: default + outline, unpressed/pressed, default + sm
    // sizes (spec §2/§3.1). Outline rows expose S-1 (shadow under both
    // styles). One toggle per line: no `ui.horizontal` rows, so no
    // 36px interact-size row artifact pollutes the rects.
    ui.add_space(12.0);
    ui.label("1. Variants — default + outline, unpressed/pressed, default + sm sizes (spec §2/§3.1; outline shadow-xs under BOTH styles, S-1)");
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[0])
        .id_salt("s3-v1-default")
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "default unpressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[1])
        .id_salt("s3-v1-default-pressed")
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "default pressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[2])
        .id_salt("s3-v1-default-sm")
        .size(ToggleSize::Sm)
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "default sm unpressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[3])
        .id_salt("s3-v1-default-sm-pressed")
        .size(ToggleSize::Sm)
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "default sm pressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[4])
        .id_salt("s3-v1-outline")
        .variant(ToggleVariant::Outline)
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "outline unpressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[5])
        .id_salt("s3-v1-outline-pressed")
        .variant(ToggleVariant::Outline)
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "outline pressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[6])
        .id_salt("s3-v1-outline-sm")
        .variant(ToggleVariant::Outline)
        .size(ToggleSize::Sm)
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "outline sm unpressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[7])
        .id_salt("s3-v1-outline-sm-pressed")
        .variant(ToggleVariant::Outline)
        .size(ToggleSize::Sm)
        .text("Bold")
        .show(ui)
        .rect;
    ui.label(format!(
        "outline sm pressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 2. Icon + text gap (N6, spec §3.2): one gap value (8px base /
    // 4px nova) in width math AND paint. The rects below let S5 check
    // the width arithmetic; the painted gap itself is S5's pixel job.
    // Icon-only + empty-content boxes exercise §3.4 (width = min-width
    // floor, fixed height, still focusable/toggleable).
    ui.add_space(12.0);
    ui.label(format!(
        "2. Icon + text gap (N6) — gap {gap}px in this recipe (8px base / 4px nova); icon-only + empty-content boxes (spec §3.2/§3.4)"
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[8])
        .id_salt("s3-gap-default")
        .icon(Icon::Sparkles)
        .text("Starred")
        .show(ui)
        .rect;
    ui.label(format!(
        "icon+text default: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[9])
        .id_salt("s3-gap-sm")
        .size(ToggleSize::Sm)
        .icon(Icon::Sparkles)
        .text("Starred")
        .show(ui)
        .rect;
    ui.label(format!(
        "icon+text sm: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[10])
        .id_salt("s3-gap-icon-only")
        .icon(Icon::Search)
        .show(ui)
        .rect;
    ui.label(format!(
        "icon-only: {:.1} x {:.1} px (expected: 36 wide both styles — base: 32 content < 36 floor; nova: 10+16+10 = 36)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[11])
        .id_salt("s3-gap-empty")
        .show(ui)
        .rect;
    ui.label(format!(
        "empty content: {:.1} x {:.1} px (expected: min-width x height, still focusable/toggleable)",
        r.width(),
        r.height()
    ));

    // 3. Icon-inline padding (N16, spec §3.3). The padding VALUES are
    // frozen; the `.icon_inline_start` trigger mapping is RATIFIED
    // (gate G3; OPEN-6 stays OPEN, no trigger inferred). Each pair shows the
    // same icon+text toggle with the builder on vs off: under nova the
    // builder-on toggle is narrower (left padding 8 vs 10 default/lg,
    // 6 vs 10 sm); under base both are identical.
    ui.add_space(12.0);
    ui.label("3. Icon-inline padding (N16) — .icon_inline_start(true) is the RATIFIED trigger mapping (gate G3); padding values frozen (nova 8px default/lg, 6px sm on the icon side; base: no adjustment)");
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[12])
        .id_salt("s3-n16-default-on")
        .icon(Icon::ChevronLeft)
        .text("Previous")
        .icon_inline_start(true)
        .show(ui)
        .rect;
    ui.label(format!(
        "inline-start default, builder ON: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[13])
        .id_salt("s3-n16-default-off")
        .icon(Icon::ChevronLeft)
        .text("Previous")
        .show(ui)
        .rect;
    ui.label(format!(
        "inline-start default, builder OFF: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[14])
        .id_salt("s3-n16-sm-on")
        .size(ToggleSize::Sm)
        .icon(Icon::ChevronLeft)
        .text("Previous")
        .icon_inline_start(true)
        .show(ui)
        .rect;
    ui.label(format!(
        "inline-start sm, builder ON: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[15])
        .id_salt("s3-n16-sm-off")
        .size(ToggleSize::Sm)
        .icon(Icon::ChevronLeft)
        .text("Previous")
        .show(ui)
        .rect;
    ui.label(format!(
        "inline-start sm, builder OFF: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[16])
        .id_salt("s3-n16-lg-on")
        .size(ToggleSize::Lg)
        .icon(Icon::ChevronLeft)
        .text("Previous")
        .icon_inline_start(true)
        .show(ui)
        .rect;
    ui.label(format!(
        "inline-start lg, builder ON: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[17])
        .id_salt("s3-n16-lg-off")
        .size(ToggleSize::Lg)
        .icon(Icon::ChevronLeft)
        .text("Previous")
        .show(ui)
        .rect;
    ui.label(format!(
        "inline-start lg, builder OFF: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 4. Focus target (spec §4.4). Label F1 rule: the widget id comes
    // from the control's returned `Response::id`, never a guessed
    // scope id.
    ui.add_space(12.0);
    ui.label("4. Focus target — Tab here or click; ring 3px + border ring on focus (spec §4.4; D1: ring on any focus)");
    ui.add_space(4.0);
    let resp = Toggle::new(&mut values[18])
        .id_salt("s3-focus")
        .variant(ToggleVariant::Outline)
        .text("Focus me")
        .show(ui);
    let focus_id = resp.id;
    let focused = ui.ctx().memory(|m| m.has_focus(focus_id));
    let r_focus = resp.rect;
    ui.label(format!(
        "focus target: {:.1} x {:.1} px; focused: {focused}",
        r_focus.width(),
        r_focus.height()
    ));

    // 5. Invalid (spec §4.5): full-alpha `destructive` border on the
    // outline variant ONLY (the default variant has no border width,
    // so no visible invalid border), NO rest ring in either style
    // (N5 does NOT apply to toggle). OPEN-4 precedence is DECIDED
    // (gate G4: invalid wins over focus).
    ui.add_space(12.0);
    ui.label("5. Invalid — full destructive border, outline only, NO rest ring in either style (spec §4.5; N5 does NOT apply); OPEN-4 precedence DECIDED invalid-wins (gate G4)");
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[19])
        .id_salt("s3-invalid-default")
        .invalid(true)
        .text("Invalid")
        .show(ui)
        .rect;
    ui.label(format!(
        "invalid default unpressed: {:.1} x {:.1} px (no visible border — borderless variant)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[20])
        .id_salt("s3-invalid-default-pressed")
        .invalid(true)
        .text("Invalid")
        .show(ui)
        .rect;
    ui.label(format!(
        "invalid default pressed: {:.1} x {:.1} px (pressed fill + no border)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[21])
        .id_salt("s3-invalid-outline")
        .variant(ToggleVariant::Outline)
        .invalid(true)
        .text("Invalid")
        .show(ui)
        .rect;
    ui.label(format!(
        "invalid outline unpressed: {:.1} x {:.1} px (destructive border, no ring at rest)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[22])
        .id_salt("s3-invalid-outline-pressed")
        .variant(ToggleVariant::Outline)
        .invalid(true)
        .text("Invalid")
        .show(ui)
        .rect;
    ui.label(format!(
        "invalid outline pressed: {:.1} x {:.1} px (pressed fill + destructive border, no ring at rest)",
        r.width(),
        r.height()
    ));

    // 6. Disabled (spec §4.6): whole-element 50% group fade. The
    // compositing model is an S5 question — judge the rendered
    // composite per ruling 7, not per-slot alphas.
    ui.add_space(12.0);
    ui.label("6. Disabled — whole-element 50% group fade (spec §4.6); compositing model is an S5 question (judge the rendered composite, ruling 7)");
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[23])
        .id_salt("s3-disabled-default")
        .disabled(true)
        .text("Disabled")
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled default unpressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[24])
        .id_salt("s3-disabled-default-pressed")
        .disabled(true)
        .text("Disabled")
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled default pressed: {:.1} x {:.1} px (pressed fill at 50%)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[25])
        .id_salt("s3-disabled-outline")
        .variant(ToggleVariant::Outline)
        .disabled(true)
        .text("Disabled")
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled outline unpressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[26])
        .id_salt("s3-disabled-outline-pressed")
        .variant(ToggleVariant::Outline)
        .disabled(true)
        .text("Disabled")
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled outline pressed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[27])
        .id_salt("s3-disabled-invalid-outline")
        .variant(ToggleVariant::Outline)
        .disabled(true)
        .invalid(true)
        .text("Disabled")
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled + invalid outline pressed: {:.1} x {:.1} px (invalid chrome at 50%)",
        r.width(),
        r.height()
    ));

    // 7. Hover target (spec §4.2). S5/S6: park the pointer and settle
    // before capturing — egui paints a cursor preview while the
    // pointer moves, so hover captures need a SETTLED pointer.
    ui.add_space(12.0);
    ui.label("7. Hover target — hover fill/text per §4.2; park the pointer and settle before capturing (settled-pointer captures)");
    ui.add_space(4.0);
    let resp = Toggle::new(&mut values[28])
        .id_salt("s3-hover")
        .variant(ToggleVariant::Outline)
        .text("Hover me")
        .show(ui);
    let hovered = resp.hovered();
    let r_hover = resp.rect;
    ui.label(format!(
        "hover target: {:.1} x {:.1} px; hovered: {hovered}",
        r_hover.width(),
        r_hover.height()
    ));

    // 8. Pressed + hover precedence (OPEN-3, PROVISIONAL: pressed wins,
    // as built). Pressed toggles for the S6 hover-while-pressed
    // checks; S5/S6 judge the rendered result against the OPEN-3
    // decision, not against this demo text.
    ui.add_space(12.0);
    ui.label("8. Pressed + hover — pressed toggles for hover-while-pressed checks; OPEN-3 precedence PROVISIONAL (pressed wins)");
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[29])
        .id_salt("s3-pressed-hover-default")
        .text("Pressed")
        .show(ui)
        .rect;
    ui.label(format!(
        "pressed default: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = Toggle::new(&mut values[30])
        .id_salt("s3-pressed-hover-outline")
        .variant(ToggleVariant::Outline)
        .text("Pressed")
        .show(ui)
        .rect;
    ui.label(format!(
        "pressed outline: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    ui.add_space(12.0);
    ui.label("Provisional: OPEN-3 pressed-wins. Decided: OPEN-4 invalid-wins (gate G4). Ratified: N16 trigger mapping (gate G3). Interim: OPEN-5 timing. Deviations D1 (focus-visible modality) and D2 (no ARIA tree) PROPOSED. No field compositions and no toggle-group rows on this page — fused/group behaviour belongs to the toggle-group page (FG-3 ring queueing is verified by toggle-group S5).");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn toggle_group_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the group actually allocated
    // (`Response.rect`), so S5/S6 can verify geometry from screenshots.
    // Header: verified final state (spec §§0, 2, 4, 6, 10, 11 + coordinator
    // rulings 5/6/7 + S8 gate dispositions + S10 verification).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "ToggleGroup — pressed-state button group. Active recipe: {recipe} (style: {}, mode: {mode}). Item heights 32/28/36px nova, 36/32/40px base (default/sm/lg). Gap = 4 x spacing units.",
        theme.style.name()
    ));
    ui.label("S10-VERIFIED (scratch merge, integrated code): spacing is Tailwind units (gap = 4 x spacing px; spacing 0 = fused; RATIFIED gate G1). Single-mode re-press deselects is CONFIRMED original behaviour (W-TG-2 SETTLED, gate G1). Spaced item padding px-3 is W-TG-1 (OPEN-4 merge-wins, working assumption, stays OPEN); nova fused px-2 is specificity-settled. Group .disabled() propagation is CONFIRMED original behaviour (gate G1). Keyboard = deviation D4 RATIFIED (per-item Tab stops, egui spatial arrows, no wrap/Home/End; gate G2). FG-3 ring-above-siblings CLOSED. Vertical fused outline follows TOG-C1 (2px seams, style-correct corners; S10-verified).");

    // Demo-local selection state, held in egui temp data (the gallery's
    // established demo-state pattern), so the stateless demo function
    // keeps selections across frames.
    #[derive(Clone)]
    struct TgDemoState {
        s1a: Option<usize>,
        s1b: Option<usize>,
        s1c: Vec<bool>,
        s1d: Option<usize>,
        s1e: Option<usize>,
        s2a: Option<usize>,
        s2b: Vec<bool>,
        s2c: Option<usize>,
        s2d: Option<usize>,
        s3: Option<usize>,
        s4a: Option<usize>,
        s4b: Option<usize>,
        s4c: Option<usize>,
        s5a: Option<usize>,
        s5b: Vec<bool>,
        s5c: Option<usize>,
        s6a: Vec<bool>,
        s6b: Option<usize>,
        s7: Option<usize>,
    }

    impl Default for TgDemoState {
        fn default() -> Self {
            Self {
                s1a: Some(0),
                s1b: Some(1),
                s1c: vec![true, false, false],
                s1d: Some(0),
                s1e: Some(0),
                s2a: Some(0),
                s2b: vec![false, true, false],
                s2c: Some(1),
                s2d: Some(0),
                s3: Some(0),
                s4a: Some(0),
                s4b: Some(1),
                s4c: Some(0),
                s5a: Some(0),
                s5b: vec![true, false, false],
                s5c: Some(0),
                s6a: vec![true, false, true],
                s6b: Some(1),
                s7: Some(0),
            }
        }
    }

    let store = egui::Id::unique("toggle_group_demo_state");
    let mut st: TgDemoState = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or_default());

    // §1. Spaced groups. §1a/§1b share identical labels, so the width
    // delta is exactly (n-1) x the 4px gap step; §1d/§1e do the same
    // vertically with the height delta.
    ui.add_space(12.0);
    ui.label("1. Spaced groups — outline + default, spacing 1 (4px gap) and 2 (8px gap), horizontal + vertical (spec §0.3/§2.4). NOTE: the horizontal row is seeded at the tallest resolved item height, floored at 36px (TGG-VIS-01 fix); judge item heights from pixels or from the vertical groups below, whose rects show true heights.");
    ui.add_space(4.0);
    let r1a = ToggleGroup::single(&mut st.s1a)
        .id_salt("s3-s1a")
        .variant(ToggleVariant::Outline)
        .spacing(1.0)
        .item("Bold")
        .item("Italic")
        .item("Underline")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "outline spacing 1 (gap 4px): {:.1} x {:.1} px; selected: {:?}",
        r1a.width(),
        r1a.height(),
        st.s1a
    ));
    ui.add_space(4.0);
    let r1b = ToggleGroup::single(&mut st.s1b)
        .id_salt("s3-s1b")
        .variant(ToggleVariant::Outline)
        .spacing(2.0)
        .item("Bold")
        .item("Italic")
        .item("Underline")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "outline spacing 2 (gap 8px): {:.1} x {:.1} px; selected: {:?}; width delta vs §1a: {:.1} px (expected 8.0 = 2 gaps x 4px step)",
        r1b.width(),
        r1b.height(),
        st.s1b,
        r1b.width() - r1a.width()
    ));
    ui.add_space(4.0);
    let r1c = ToggleGroup::multiple(&mut st.s1c)
        .id_salt("s3-s1c")
        .spacing(1.0)
        .item_with_icon("Left", Icon::ChevronLeft)
        .item("Center")
        .item_with_icon("Right", Icon::ChevronRight)
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "default spacing 1, with icons (gap 4px): {:.1} x {:.1} px; pressed: {:?}; icon-inline padding is symmetric here (per-side 6px is a RECORDED limitation, gate G6; toggle OPEN-6 stays OPEN)",
        r1c.width(),
        r1c.height(),
        st.s1c
    ));
    ui.add_space(4.0);
    let r1d = ToggleGroup::single(&mut st.s1d)
        .id_salt("s3-s1d")
        .orientation(GroupOrientation::Vertical)
        .spacing(1.0)
        .item("Top")
        .item("Middle")
        .item("Bottom")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "default vertical spacing 1 (gap 4px): {:.1} x {:.1} px; selected: {:?}",
        r1d.width(),
        r1d.height(),
        st.s1d
    ));
    ui.add_space(4.0);
    let r1e = ToggleGroup::single(&mut st.s1e)
        .id_salt("s3-s1e")
        .orientation(GroupOrientation::Vertical)
        .spacing(2.0)
        .item("Top")
        .item("Middle")
        .item("Bottom")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "default vertical spacing 2 (gap 8px): {:.1} x {:.1} px; selected: {:?}; height delta vs §1d: {:.1} px (expected 8.0 = 2 gaps x 4px step)",
        r1e.width(),
        r1e.height(),
        st.s1e,
        r1e.height() - r1d.height()
    ));

    // §2. Fused groups (spacing 0): outer corners rounded, inner square;
    // fused outline shares one 1px border between neighbours; shadow-none
    // in both styles. §2c states the §9 case 5 vertical reading.
    ui.add_space(12.0);
    ui.label("2. Fused groups (spacing 0) — outer corners rounded, inner square; fused outline shares one 1px border; shadow-none in both styles (spec §2.2/§2.5)");
    ui.add_space(4.0);
    let r2a = ToggleGroup::single(&mut st.s2a)
        .id_salt("s3-s2a")
        .variant(ToggleVariant::Outline)
        .item("Bold")
        .item("Italic")
        .item("Underline")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "outline fused horizontal: {:.1} x {:.1} px; selected: {:?}",
        r2a.width(),
        r2a.height(),
        st.s2a
    ));
    ui.add_space(4.0);
    let r2b = ToggleGroup::multiple(&mut st.s2b)
        .id_salt("s3-s2b")
        .item("Bold")
        .item("Italic")
        .item("Underline")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "default fused horizontal: {:.1} x {:.1} px; pressed: {:?}",
        r2b.width(),
        r2b.height(),
        st.s2b
    ));
    ui.add_space(4.0);
    let r2c = ToggleGroup::single(&mut st.s2c)
        .id_salt("s3-s2c")
        .variant(ToggleVariant::Outline)
        .orientation(GroupOrientation::Vertical)
        .item("Top")
        .item("Middle")
        .item("Bottom")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "outline fused vertical: {:.1} x {:.1} px; selected: {:?}; §9 case 5 as built (TOG-C1): base horizontal corner treatment + 2px horizontal seams (no vertical border rule exists); S10-verified (seams 2,2; corners style-correct)",
        r2c.width(),
        r2c.height(),
        st.s2c
    ));
    ui.add_space(4.0);
    let r2d = ToggleGroup::single(&mut st.s2d)
        .id_salt("s3-s2d")
        .variant(ToggleVariant::Outline)
        .item("Only")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "single-item fused group (§9 case 1): {:.1} x {:.1} px; selected: {:?}; fully rounded from frame 2 (protocol learns the count on the previous frame)",
        r2d.width(),
        r2d.height(),
        st.s2d
    ));

    // §3. Focus target. The group API returns no per-item Response, so
    // the readout below reflects the group SCOPE response id (Label F1:
    // captured from the returned Response, never derived). Item focus
    // itself is judged by S6 from ring pixels (slider O-4 precedent).
    ui.add_space(12.0);
    ui.label("3. Focus target — Tab here; ring 3px + border ring on focus (spec §3.4); FG-3 CLOSED: the focused item's ring paints above its siblings (S5 + S10 verified)");
    ui.add_space(4.0);
    let resp3 = ToggleGroup::single(&mut st.s3)
        .id_salt("s3-focus")
        .variant(ToggleVariant::Outline)
        .item("Focus A")
        .item("Focus B")
        .item("Focus C")
        .show(ui);
    let r3 = resp3.response.rect;
    let scope_focused = ui.ctx().memory(|m| m.has_focus(resp3.response.id));
    ui.label(format!(
        "focus target: {:.1} x {:.1} px; selected: {:?}; scope focused: {scope_focused}",
        r3.width(),
        r3.height(),
        st.s3
    ));
    ui.label("NOTE: 'scope focused' reflects the group scope response, NOT per-item focus (items expose no Response). S6 judges item focus from ring pixels. FG-3 ring queueing landed in toggle S1; S5 + S10 verified the ring lands above siblings on integrated code.");

    // §4. Invalid, per-item only (ruling 5 / N17 option (a)): destructive
    // border on exactly the invalid item, NO rest ring in either style.
    ui.add_space(12.0);
    ui.label("4. Invalid (per-item only, ruling 5) — full destructive border on exactly the invalid item, NO rest ring in either style (spec §4.7); invalid+focus rendering PROVISIONAL (gate G3; S5 findings-only cell)");
    ui.add_space(4.0);
    let r4a = ToggleGroup::single(&mut st.s4a)
        .id_salt("s3-s4a")
        .variant(ToggleVariant::Outline)
        .spacing(2.0)
        .item("Valid A")
        .add_item(GroupItem::new("Invalid B").invalid(true))
        .item("Valid C")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "spaced outline, middle item invalid: {:.1} x {:.1} px; selected: {:?}",
        r4a.width(),
        r4a.height(),
        st.s4a
    ));
    ui.add_space(4.0);
    let r4b = ToggleGroup::single(&mut st.s4b)
        .id_salt("s3-s4b")
        .variant(ToggleVariant::Outline)
        .add_item(GroupItem::new("Invalid A").invalid(true))
        .item("Valid B")
        .item("Valid C")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "fused outline, first item invalid: {:.1} x {:.1} px; selected: {:?}",
        r4b.width(),
        r4b.height(),
        st.s4b
    ));
    ui.add_space(4.0);
    let r4c = ToggleGroup::single(&mut st.s4c)
        .id_salt("s3-s4c")
        .item("Valid A")
        .add_item(GroupItem::new("Invalid B").invalid(true))
        .item("Valid C")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "fused default, middle item invalid: {:.1} x {:.1} px; selected: {:?}; destructive border on exactly the invalid item, no rest ring (S5 + S10 verified)",
        r4c.width(),
        r4c.height(),
        st.s4c
    ));

    // §5. Disabled: group propagation (CONFIRMED, gate G1 item 3),
    // per-item disabled, and invalid+disabled. Group fade per ruling 7
    // (S5 verified on integrated code).
    ui.add_space(12.0);
    ui.label("5. Disabled — group propagation + per-item disabled + invalid+disabled; whole-item 50% group fade per ruling 7 (S5 verified, spec §4.6)");
    ui.add_space(4.0);
    let r5a = ToggleGroup::single(&mut st.s5a)
        .id_salt("s3-s5a")
        .variant(ToggleVariant::Outline)
        .disabled(true)
        .item("Cut")
        .item("Copy")
        .item("Paste")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "group disabled (propagation CONFIRMED, gate G1): {:.1} x {:.1} px; selected: {:?}",
        r5a.width(),
        r5a.height(),
        st.s5a
    ));
    ui.add_space(4.0);
    let r5b = ToggleGroup::multiple(&mut st.s5b)
        .id_salt("s3-s5b")
        .spacing(2.0)
        .item("Enabled A")
        .add_item(GroupItem::new("Disabled B").disabled(true))
        .item("Enabled C")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "spaced default, middle item disabled: {:.1} x {:.1} px; pressed: {:?}",
        r5b.width(),
        r5b.height(),
        st.s5b
    ));
    ui.add_space(4.0);
    let r5c = ToggleGroup::single(&mut st.s5c)
        .id_salt("s3-s5c")
        .variant(ToggleVariant::Outline)
        .add_item(
            GroupItem::new("Invalid+disabled")
                .invalid(true)
                .disabled(true),
        )
        .item("Valid B")
        .item("Valid C")
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "fused outline, first item invalid+disabled: {:.1} x {:.1} px; selected: {:?}",
        r5c.width(),
        r5c.height(),
        st.s5c
    ));

    // §6. Mixed sizes/variants via the §8.1 context resolution: the
    // group's value wins when defined; the item's own prop applies only
    // when the group leaves it undefined.
    ui.add_space(12.0);
    ui.label("6. Mixed sizes/variants — context resolution (spec §8.1): group-defined wins; item's own prop applies only when the group leaves it undefined");
    ui.add_space(4.0);
    let r6a = ToggleGroup::multiple(&mut st.s6a)
        .id_salt("s3-s6a")
        .spacing(2.0)
        .add_item(
            GroupItem::new("Sm outline")
                .size(ToggleSize::Sm)
                .variant(ToggleVariant::Outline),
        )
        .add_item(GroupItem::new("Default"))
        .add_item(
            GroupItem::new("Lg outline")
                .size(ToggleSize::Lg)
                .variant(ToggleVariant::Outline),
        )
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "group undefined + per-item sizes/variants (mixed heights, vertically centered): {:.1} x {:.1} px; pressed: {:?}",
        r6a.width(),
        r6a.height(),
        st.s6a
    ));
    ui.add_space(4.0);
    let r6b = ToggleGroup::single(&mut st.s6b)
        .id_salt("s3-s6b")
        .variant(ToggleVariant::Outline)
        .size(ToggleSize::Sm)
        .spacing(2.0)
        .add_item(
            GroupItem::new("Own Lg ignored")
                .size(ToggleSize::Lg)
                .variant(ToggleVariant::Default),
        )
        .add_item(GroupItem::new("Plain"))
        .add_item(GroupItem::new("Plain 2").size(ToggleSize::Lg))
        .show(ui)
        .response
        .rect;
    ui.label(format!(
        "group outline+Sm forces all items (own props ignored): {:.1} x {:.1} px; selected: {:?}",
        r6b.width(),
        r6b.height(),
        st.s6b
    ));

    // §7. Hover target. The group scope response's `hovered()` reads
    // false with the pointer parked (radio S3 precedent), so hover is
    // read from the pointer position over the captured group rect.
    ui.add_space(12.0);
    ui.label("7. Hover target — hover fill/text per §4.3; park the pointer and settle before capturing (settled-pointer captures)");
    ui.add_space(4.0);
    let r7 = ToggleGroup::single(&mut st.s7)
        .id_salt("s3-hover")
        .variant(ToggleVariant::Outline)
        .spacing(2.0)
        .item("Hover A")
        .item("Hover B")
        .item("Hover C")
        .show(ui)
        .response
        .rect;
    let hovered = ui.rect_contains_pointer(r7);
    ui.label(format!(
        "hover target: {:.1} x {:.1} px; selected: {:?}; hovered: {hovered}",
        r7.width(),
        r7.height(),
        st.s7
    ));

    ui.add_space(12.0);
    ui.label("Dispositions (S8 gate + S10): W-TG-1 (OPEN-4 merge-wins px-3) stays a WORKING ASSUMPTION, OPEN-4 OPEN/PROVISIONAL; W-TG-2 (deselect-on-repress) CONFIRMED/SETTLED; OPEN-2 group-disabled propagation CONFIRMED; OPEN-3/OPEN-5/OPEN-6 stay OPEN; transition timing INTERIM. Deviations D1 (no ARIA tree), D2 (focus-visible modality), D3 (data-* dropped) PROPOSED; D4 (roving deviation) RATIFIED. No field compositions on this page.");

    ui.ctx().data_mut(|d| d.insert_temp(store, st));
}

pub fn input_otp_demo(ui: &mut egui::Ui) {
    // Header: recipe position, stated honestly (spec §§0, 2.3, 3, 4, 6, 11).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Input OTP — single-hidden-input model, one Tab stop per control (S1, library input-otp 1.4.2). Active recipe: {recipe} (style: {}, mode: {mode}). Slots 36px base / 32px nova; end corners 8px base / 10px nova; fused 1px dividers (N7); active ring 3px ring/50; invalid full destructive (N5 does NOT apply); disabled = whole-control 50% fade; caret 1x16px, 1.25s keyframe blink.",
        theme.style.name()
    ));
    ui.label("Frozen: caret blink 1.25s + caret-blink keyframes (OPEN-6 CLOSED, S4 §9; D4 VOID); base separator 24px (F-05 corrected; D5 VOID); ring eases 150ms (ANM-01, S10-verified); blink advances with no interaction via a repaint driver (gate G4); light foreground follows themes.ts (OPEN-9 settled). Deviations D1 (no ARIA tree), D2 (focus/active trigger modality), D3 (focus mechanism) PROPOSED. Click a group to focus it — the caret lands in the first empty slot; typing the last digit fires onComplete (§4). Row readouts are scope rects: widths are exact under nova (192 = 6x32, 224 = 96+8+16+8+96) and base §2 is 256 (108+8+24+8+108), heights read 36.0 because the scope adds 4px layout spacing — slots are 32px tall under nova, 36px under base.");

    let simple_store = egui::Id::unique("otp_demo_simple");
    let custom_store = egui::Id::unique("otp_demo_custom");
    let disabled_store = egui::Id::unique("otp_demo_disabled");
    let complete_store = egui::Id::unique("otp_demo_complete");
    let complete_seen_store = egui::Id::unique("otp_demo_complete_seen");
    // Pre-filled values are deterministic: §1 shows filled slots + the
    // caret landing (slot 3) on focus; §3 shows faded/invalid chars;
    // §4 sits one digit short of full so S5/S6 can fire onComplete.
    let mut code: String = ui.ctx().data_mut(|d| {
        d.get_temp(simple_store)
            .unwrap_or_else(|| "123".to_string())
    });
    let mut custom: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(custom_store).unwrap_or_default());
    let mut frozen: String = ui.ctx().data_mut(|d| {
        d.get_temp(disabled_store)
            .unwrap_or_else(|| "12".to_string())
    });
    let mut complete_code: String = ui.ctx().data_mut(|d| {
        d.get_temp(complete_store)
            .unwrap_or_else(|| "12345".to_string())
    });
    let mut complete_seen: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(complete_seen_store).unwrap_or(false));

    // §1. Single 6-slot group — the primary composition.
    ui.add_space(12.0);
    ui.label("1. Single 6-slot group — fused 1px dividers (N7), active ring + caret on the focused slot (spec §3.1, §4.2); click to focus, caret lands in the first empty slot (slot 3)");
    ui.add_space(4.0);
    let resp1 = ui.vertical(|ui| {
        let out = InputOTP::new(&mut code)
            .length(6)
            .numeric(true)
            .id_salt("otp_demo_simple")
            .show(ui);
        if out.changed {
            ui.ctx().request_repaint();
        }
        out
    });
    let r1 = resp1.response.rect;
    let out1 = resp1.inner;
    ui.label(format!(
        "§1: {:.1} x {:.1} px; code: {:?}; focused: {}",
        r1.width(),
        r1.height(),
        code,
        out1.focused
    ));

    // §2. 3+3 with separator.
    ui.add_space(12.0);
    ui.label(
        "2. Pattern with separator (3+3) — group boundaries + 24px base / 16px nova separator (spec §2.5; D5 VOID, F-05 corrected)",
    );
    ui.add_space(4.0);
    let resp2 = ui.vertical(|ui| {
        let out = InputOTP::new(&mut custom)
            .length(6)
            .numeric(true)
            .id_salt("otp_demo_custom")
            .show_with(ui, |ui| {
                InputOTPGroup::new().show(ui, |ui| {
                    InputOTPSlot::new(0).show(ui);
                    InputOTPSlot::new(1).show(ui);
                    InputOTPSlot::new(2).show(ui);
                });
                InputOTPSeparator::new().show(ui);
                InputOTPGroup::new().show(ui, |ui| {
                    InputOTPSlot::new(3).show(ui);
                    InputOTPSlot::new(4).show(ui);
                    InputOTPSlot::new(5).show(ui);
                });
            });
        if out.changed {
            ui.ctx().request_repaint();
        }
        out
    });
    let r2 = resp2.response.rect;
    let out2 = resp2.inner;
    ui.label(format!(
        "§2: {:.1} x {:.1} px; code: {:?}; focused: {}",
        r2.width(),
        r2.height(),
        custom,
        out2.focused
    ));

    // §3. Disabled + invalid row (both bound to the same `frozen` string
    // with different salts, per the S1 report — the two instances sync).
    ui.add_space(12.0);
    ui.label("3. Disabled and invalid — disabled fades the whole control 50% incl. chars, separator and shadow (spec §4.4); invalid slots carry full destructive borders (spec §4.3); click the invalid group to focus it for the active+invalid slot (border stays destructive, ring destructive 20%/40%)");
    ui.add_space(4.0);
    let resp3 = ui.horizontal(|ui| {
        InputOTP::new(&mut frozen)
            .length(4)
            .numeric(true)
            .disabled(true)
            .id_salt("otp_demo_disabled")
            .show(ui);
        let out_inv = InputOTP::new(&mut frozen)
            .length(4)
            .numeric(true)
            .invalid(true)
            .id_salt("otp_demo_invalid")
            .show(ui);
        if out_inv.changed {
            ui.ctx().request_repaint();
        }
        out_inv
    });
    let r3 = resp3.response.rect;
    let out3 = resp3.inner;
    ui.label(format!(
        "§3: {:.1} x {:.1} px; shared value: {:?}; invalid group focused: {}",
        r3.width(),
        r3.height(),
        frozen,
        out3.focused
    ));

    // §4. Completed row — one digit short of full.
    ui.add_space(12.0);
    ui.label("4. Completed — type the 6th digit to fire onComplete; the latch below stays set until the value drops below full (OtpOutput.completed is true only on the transition frame)");
    ui.add_space(4.0);
    let resp4 = ui.vertical(|ui| {
        let out = InputOTP::new(&mut complete_code)
            .length(6)
            .numeric(true)
            .id_salt("otp_demo_complete")
            .show(ui);
        if out.changed {
            ui.ctx().request_repaint();
        }
        out
    });
    let r4 = resp4.response.rect;
    let out4 = resp4.inner;
    if out4.completed {
        complete_seen = true;
    }
    if complete_code.chars().count() < 6 {
        complete_seen = false;
    }
    ui.label(format!(
        "§4: {:.1} x {:.1} px; code: {:?}; completed (latched): {}; completed this frame: {}; focused: {}",
        r4.width(),
        r4.height(),
        complete_code,
        complete_seen,
        out4.completed,
        out4.focused
    ));

    ui.add_space(12.0);
    ui.label(
        "Keyboard: digits fill the focused slot, Backspace clears, Delete forward-deletes, Arrow keys move, paste is atomic under .numeric() (one bad char rejects the whole paste).",
    );

    ui.ctx().data_mut(|d| {
        d.insert_temp(simple_store, code);
        d.insert_temp(custom_store, custom);
        d.insert_temp(disabled_store, frozen);
        d.insert_temp(complete_store, complete_code);
        d.insert_temp(complete_seen_store, complete_seen);
    });
}

pub fn native_select_demo(ui: &mut egui::Ui) {
    // `SelectOptGroup` is re-exported in `components/mod.rs` (S1 flag F1,
    // landed as the S12 merge-time one-liner). The full-path import below
    // is kept; it resolves to the same type.
    use egui_shadcn_ui::components::native_select::SelectOptGroup;

    // Every section prints the rect the trigger actually allocated
    // (`Response.rect`), so S5/S6 can verify geometry from screenshots.
    // Header: recipe position, stated honestly (spec §§0, 2.3, 3, 4, 6, 11).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Native select — native <select> port. Active recipe: {recipe} (style: {}, mode: {mode}). Heights 36/32px base, 32/28px nova (default/sm); radii 8px base, 10/8px nova.",
        theme.style.name()
    ));
    ui.label("Frozen: N5 nova-dark invalid border destructive/50; invalid rest ring 3px under nova ONLY (base: focus-gated); disabled = whole-wrapper group fade (ruling 7); shadow-xs under ALL styles incl. nova (LAYER); no-selection = D4 placeholder convention (ruling 4, port invention); single-select only (OPEN-8). DECIDED: OPEN-2 invalid+focus precedence (invalid-wins, gate G1 item 7, S10-verified). Popup = port design D1 per the 12 decided values (ruling 9): popover fill, 36px rows, accent hover/selected, shadow-md, max 8 visible rows + scroll, trigger width, radius md + 4px margins, 1px border stroke. Transition timing 150ms + cubic-bezier(0.4, 0, 0.2, 1) is INTERIM (OPEN-3); property sets per ruling 6 (base: text+ring animate, border/fill snap; nova: colours animate, ring snaps).");

    // Index layout: 0-10 §1 trigger states, 11-14 §2 optgroups, 15-16 §3
    // long lists, 17 §4 empty state, 18-19 §5 disabled rows, 20 §6 keyboard
    // target, 21 §7 hover target, 22 §8 long label (clip). One select per
    // line: no `ui.horizontal` rows, so no 36px interact-size row artifact
    // pollutes the rects.
    let store = egui::Id::unique("native_select_demo_values");
    let mut values: [Option<usize>; 23] = ui.ctx().data_mut(|d| {
        d.get_temp(store).unwrap_or([
            Some(0),
            Some(1),
            None,
            None,
            None,
            Some(0),
            None,
            Some(0),
            Some(0),
            Some(1),
            Some(0),
            Some(0),
            Some(1),
            Some(0),
            None,
            Some(0),
            Some(0),
            None,
            Some(0),
            Some(1),
            Some(0),
            Some(0),
            Some(0),
        ])
    });

    // 1. Trigger states (spec §2/§3/§4): default + sm, resting, selected,
    // placeholder, empty, disabled, invalid.
    ui.add_space(12.0);
    ui.label("1. Trigger states — default + sm sizes, resting/selected/placeholder/empty/disabled/invalid (spec §2/§3/§4)");
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[0])
        .id_salt("s3-default")
        .options(["Light", "Dark", "System"])
        .show(ui)
        .rect;
    ui.label(format!(
        "default selected: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[1])
        .id_salt("s3-sm")
        .size(NativeSelectSize::Sm)
        .options(["Light", "Dark", "System"])
        .show(ui)
        .rect;
    ui.label(format!(
        "sm selected: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[2])
        .id_salt("s3-placeholder")
        .options(["Light", "Dark", "System"])
        .placeholder("Pick a theme")
        .show(ui)
        .rect;
    ui.label(format!(
        "default no selection + placeholder (D4, muted-foreground): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[3])
        .id_salt("s3-placeholder-sm")
        .size(NativeSelectSize::Sm)
        .options(["Light", "Dark", "System"])
        .placeholder("Pick a theme")
        .show(ui)
        .rect;
    ui.label(format!(
        "sm no selection + placeholder (D4): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[4])
        .id_salt("s3-empty-trigger")
        .options(["Light", "Dark", "System"])
        .show(ui)
        .rect;
    ui.label(format!(
        "no selection, no placeholder — renders EMPTY, geometry unchanged (ruling 4): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[5])
        .id_salt("s3-disabled")
        .disabled(true)
        .options(["Light", "Dark", "System"])
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled + selected — text foreground @ 50% composite, N11 (group fade, ruling 7): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[6])
        .id_salt("s3-disabled-placeholder")
        .disabled(true)
        .options(["Light", "Dark", "System"])
        .placeholder("Pick a theme")
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled + no selection — D4 colour composited at 50%: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[7])
        .id_salt("s3-invalid")
        .invalid(true)
        .options(["Light", "Dark", "System"])
        .show(ui)
        .rect;
    ui.label(format!(
        "invalid — destructive border (nova dark: /50, N5); 3px rest ring under NOVA only: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[8])
        .id_salt("s3-invalid-sm")
        .size(NativeSelectSize::Sm)
        .invalid(true)
        .options(["Light", "Dark", "System"])
        .show(ui)
        .rect;
    ui.label(format!(
        "invalid sm: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[9])
        .id_salt("s3-invalid-disabled")
        .invalid(true)
        .disabled(true)
        .options(["Light", "Dark", "System"])
        .show(ui)
        .rect;
    ui.label(format!(
        "invalid + disabled — invalid colours composited at 50%: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let resp = NativeSelect::new(&mut values[10])
        .id_salt("s3-invalid-focus")
        .invalid(true)
        .options(["Light", "Dark", "System"])
        .show(ui);
    let focus_id = resp.id;
    let focused = ui.ctx().memory(|m| m.has_focus(focus_id));
    let r = resp.rect;
    ui.label(format!(
        "invalid + focus probe — DECIDED (OPEN-2, gate G1 item 7): invalid-wins, S10-verified; focused: {focused}; {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 2. Optgroups (N10, spec §2.5/§8/§10): grouped, mixed flat+group,
    // empty group, only-empty-group.
    ui.add_space(12.0);
    ui.label("2. Optgroups — grouped select, mixed flat+group, empty group (N10; headers 36px, muted-foreground, non-interactive; ruling 9 item 7)");
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[11])
        .id_salt("s3-optgroups")
        .optgroups([
            SelectOptGroup::with_options(
                "Fruits",
                [SelectOption::new("Apple"), SelectOption::new("Banana")],
            ),
            SelectOptGroup::with_options(
                "Vegetables",
                [SelectOption::new("Carrot"), SelectOption::new("Daikon")],
            )
            .disabled(true),
        ])
        .show(ui)
        .rect;
    ui.label(format!(
        "grouped: 2 groups, second group DISABLED (disables all its options): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[12])
        .id_salt("s3-mixed")
        .option("Ungrouped first")
        .optgroup(SelectOptGroup::with_options(
            "Grouped",
            [SelectOption::new("Alpha"), SelectOption::new("Beta")],
        ))
        .option("Ungrouped last")
        .show(ui)
        .rect;
    ui.label(format!(
        "mixed flat + group in insertion order (flat indices count options only): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[13])
        .id_salt("s3-empty-group")
        .optgroups([
            SelectOptGroup::new("Empty group"),
            SelectOptGroup::with_options(
                "Colors",
                [SelectOption::new("Red"), SelectOption::new("Green")],
            ),
        ])
        .show(ui)
        .rect;
    ui.label(format!(
        "empty group renders its header with no rows, beside a populated group: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[14])
        .id_salt("s3-only-empty-group")
        .optgroup(SelectOptGroup::new("Nothing here"))
        .placeholder("Only an empty group")
        .show(ui)
        .rect;
    ui.label(format!(
        "ONLY an empty group: popup shows the header only, no \"No options\" row (RATIFIED, gate G1 item 3): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 3. Long lists (ruling 9 item 6): max 8 visible rows + internal scroll.
    ui.add_space(12.0);
    ui.label("3. Long lists — popup max-height + internal scroll (ruling 9 item 6: max 8 visible rows of 36px + 4px margins; longer lists scroll)");
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[15])
        .id_salt("s3-long-list")
        .options([
            "Option 1",
            "Option 2",
            "Option 3",
            "Option 4",
            "Option 5",
            "Option 6",
            "Option 7",
            "Option 8",
            "Option 9",
            "Option 10",
            "Option 11",
            "Option 12",
        ])
        .show(ui)
        .rect;
    ui.label(format!(
        "12 options — popup scrolls internally (also the PageUp/PageDown target): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[16])
        .id_salt("s3-eight-options")
        .options([
            "Option 1", "Option 2", "Option 3", "Option 4", "Option 5", "Option 6", "Option 7",
            "Option 8",
        ])
        .show(ui)
        .rect;
    ui.label(format!(
        "exactly 8 options — the max-visible boundary, no scroll needed: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 4. Empty state (ruling 9 item 9): zero options.
    ui.add_space(12.0);
    ui.label("4. Empty state — zero options (ruling 9 item 9: popup opens with ONE non-interactive \"No options\" row in muted-foreground)");
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[17])
        .id_salt("s3-zero-options")
        .placeholder("Choose an option")
        .show(ui)
        .rect;
    ui.label(format!(
        "zero-option select (160px width floor; placeholder + padding may exceed it): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 5. Disabled rows (ruling 9 item 8): per-slot halving INSIDE popup rows
    // — NOT the trigger group fade. Plus the §10 disabled-selected case.
    ui.add_space(12.0);
    ui.label("5. Disabled rows — per-slot halving INSIDE popup rows (ruling 9 item 8; NOT ruling 7 group fade, which governs the trigger only)");
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[18])
        .id_salt("s3-disabled-rows")
        .option("Push notifications")
        .option_disabled("Email digest (unavailable)")
        .option("Nothing")
        .show(ui)
        .rect;
    ui.label(format!(
        "one disabled option among enabled ones: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[19])
        .id_salt("s3-disabled-selected")
        .option("Available")
        .option_disabled("Unavailable (selected)")
        .option("Other")
        .show(ui)
        .rect;
    ui.label(format!(
        "disabled option HOLDING the value (spec §10: trigger renders its label per the normal foreground rule; arrows/type-ahead skip it): {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    // 6. Keyboard target (spec §7.1). Label F1 rule: the widget id comes
    // from the control's returned `Response::id`, never a guessed scope id.
    ui.add_space(12.0);
    ui.label("6. Keyboard target — Tab here; arrows change the selection when closed, move the highlight when open; Enter confirms, Escape closes (spec §7.1). F4: settle 2+ frames after focusing before sending keys.");
    ui.add_space(4.0);
    let resp = NativeSelect::new(&mut values[20])
        .id_salt("s3-keyboard")
        .options(["Alpha", "Beta", "Gamma", "Delta"])
        .placeholder("Keyboard target")
        .show(ui);
    let focus_id = resp.id;
    let focused = ui.ctx().memory(|m| m.has_focus(focus_id));
    let r = resp.rect;
    ui.label(format!(
        "keyboard target: {:.1} x {:.1} px; focused: {focused}; selected: {:?}",
        r.width(),
        r.height(),
        values[20]
    ));

    // 7. Hover target (spec §4.2): dark hover fill `input/50`; light mode
    // has NO hover rule. Hover readout via `rect_contains_pointer` (radio
    // S3 precedent) — the trigger response's `hovered()` is not trusted
    // with the pointer parked.
    ui.add_space(12.0);
    ui.label("7. Hover target — dark: fill becomes input/50 on hover; light: NO hover change (spec §4.2)");
    ui.add_space(4.0);
    let resp = NativeSelect::new(&mut values[21])
        .id_salt("s3-hover")
        .options(["Light", "Dark", "System"])
        .show(ui);
    let hovered = ui.rect_contains_pointer(resp.rect);
    let r = resp.rect;
    ui.label(format!(
        "hover target: {:.1} x {:.1} px; hovered: {hovered}",
        r.width(),
        r.height()
    ));

    // 8. Long label (spec §10): trigger truncation is RATIFIED —
    // CLIP, no ellipsis (D-NS2, gate G1 item 1).
    ui.add_space(12.0);
    ui.label("8. Long label — explicit narrow width; trigger text CLIPS, no ellipsis (D-NS2 RATIFIED, gate G1 item 1)");
    ui.add_space(4.0);
    let r = NativeSelect::new(&mut values[22])
        .id_salt("s3-long-label")
        .width(160.0)
        .option("A very long option label that exceeds the trigger width")
        .option("Short")
        .show(ui)
        .rect;
    ui.label(format!(
        "long label, 160px width: {:.1} x {:.1} px",
        r.width(),
        r.height()
    ));

    ui.add_space(12.0);
    ui.label("S5/S6/S7 handoff: F4 — settle 2+ frames after focusing the trigger (click or Tab) before sending Escape/arrows, or keys misbehave for focus-lock reasons (egui 0.37). Popup targets: 36px rows, max 8 visible + internal scroll (§3), trigger width, radius md + 4px margins, shadow-md, accent hover/selected/highlight, muted-foreground headers + \"No options\" row. Tab calibration is NOT measured here — S6 establishes the stop inventory via the §6 focused: readout. OPEN-1 keyboard details CLOSED as documented port behaviour (gate G2); the combined invalid+focus cell (§1) is DECIDED invalid-wins (OPEN-2, gate G1 item 7). Disabled triggers never open a popup.");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

/// Demo-local state for the field demo, held in egui temp data (the
/// gallery's established demo-state pattern). Live control values,
/// activation counters for the F2 click targets, and control ids
/// captured from the previous pass for `for_id` bindings (scoped-id
/// rule: capture from the returned `Response::id`, never
/// `scope_id().with(salt)`; `Id::NULL` on the first pass targets no
/// widget).
#[derive(Clone)]
struct FieldDemoState {
    // §1 composition targets.
    s1_notif: bool,
    s1_email: String,
    s1_name: String,
    // §2e N14 radio-group child.
    s2e_sel: Option<usize>,
    // §5 responsive switches.
    s5_wide: bool,
    s5_narrow: bool,
    s5_nested: bool,
    s5_nogroup: bool,
    // §6 horizontal checkboxes.
    s6_content_cb: bool,
    s6_plain_cb: bool,
    // §7 card controls.
    s7a_sw: bool,
    s7b_cb: bool,
    s7c_radio: bool,
    s7d_text: String,
    s7e_sw: bool,
    s7f_cb: bool,
    // §8 disabled-distinction checkboxes (stay enabled in all four
    // flag combinations; only the label/field flags vary).
    s8_plain: [bool; 4],
    s8_card: [bool; 4],
    // §9 plain-label targets.
    s9a_text: String,
    s9b_cb: bool,
    s9c_sw: bool,
    s9d_radio: bool,
    s9e_cb: bool,
    // §10 field-state inputs.
    s10a_text: String,
    s10b_text: String,
    // Activation counters (F2 exactly-once probes).
    s7a_act: u32,
    s7b_act: u32,
    s7c_act: u32,
    s7e_act: u32,
    s7f_act: u32,
    s8_plain_act: [u32; 4],
    s8_card_act: [u32; 4],
    s9b_act: u32,
    s9c_act: u32,
    s9d_act: u32,
    // Ids captured from the previous pass's control responses.
    s7a_id: egui::Id,
    s7b_id: egui::Id,
    s7c_id: egui::Id,
    s7d_id: egui::Id,
    s7e_id: egui::Id,
    s7f_id: egui::Id,
    s8_plain_id: [egui::Id; 4],
    s8_card_id: [egui::Id; 4],
    s9a_id: egui::Id,
    s9b_id: egui::Id,
    s9c_id: egui::Id,
    s9d_id: egui::Id,
}

impl FieldDemoState {
    fn new() -> Self {
        Self {
            s1_notif: false,
            s1_email: String::new(),
            s1_name: String::new(),
            s2e_sel: Some(0),
            s5_wide: false,
            s5_narrow: false,
            s5_nested: false,
            s5_nogroup: false,
            s6_content_cb: false,
            s6_plain_cb: false,
            s7a_sw: false,
            s7b_cb: true,
            s7c_radio: false,
            s7d_text: String::new(),
            s7e_sw: false,
            s7f_cb: true,
            s8_plain: [false; 4],
            s8_card: [false; 4],
            s9a_text: String::new(),
            s9b_cb: false,
            s9c_sw: false,
            s9d_radio: false,
            s9e_cb: false,
            s10a_text: String::new(),
            s10b_text: String::new(),
            s7a_act: 0,
            s7b_act: 0,
            s7c_act: 0,
            s7e_act: 0,
            s7f_act: 0,
            s8_plain_act: [0; 4],
            s8_card_act: [0; 4],
            s9b_act: 0,
            s9c_act: 0,
            s9d_act: 0,
            s7a_id: egui::Id::NULL,
            s7b_id: egui::Id::NULL,
            s7c_id: egui::Id::NULL,
            s7d_id: egui::Id::NULL,
            s7e_id: egui::Id::NULL,
            s7f_id: egui::Id::NULL,
            s8_plain_id: [egui::Id::NULL; 4],
            s8_card_id: [egui::Id::NULL; 4],
            s9a_id: egui::Id::NULL,
            s9b_id: egui::Id::NULL,
            s9c_id: egui::Id::NULL,
            s9d_id: egui::Id::NULL,
        }
    }
}

/// One §8 plain pair: a `FieldLabel` bound to a `Checkbox` inside a
/// `Field`. The checkbox stays enabled; only the label's own flag and
/// the Field's flag vary. Returns (label rect, checkbox focused).
#[allow(clippy::too_many_arguments)]
fn field_s3_plain_pair(
    ui: &mut egui::Ui,
    salt: &str,
    label: &str,
    checked: &mut bool,
    activations: &mut u32,
    bound_id: &mut egui::Id,
    label_disabled: bool,
    field_disabled: bool,
) -> (egui::Rect, bool) {
    Field::new()
        .disabled(field_disabled)
        .show(ui, |ui| {
            let target = *bound_id;
            let lr = FieldLabel::new(label)
                .for_id(target)
                .disabled(label_disabled)
                .id_salt(format!("{salt}-label"))
                .show(ui)
                .rect;
            let resp = Checkbox::new(checked)
                .id_salt(format!("{salt}-cb"))
                .show(ui);
            if resp.clicked() {
                *activations += 1;
            }
            let focused = resp.has_focus();
            *bound_id = resp.id;
            (lr, focused)
        })
        .inner
}

/// One §8 card: a choice card wrapping a horizontal `Field` with a
/// `Checkbox` + `FieldContent`. The enclosing `Field` carries the
/// field-group disabled flag (the card label reads the ENCLOSING
/// context, spec §4.2). Returns (card rect, checkbox focused, card
/// hovered).
#[allow(clippy::too_many_arguments)]
fn field_s3_card_pair(
    ui: &mut egui::Ui,
    salt: &str,
    title: &str,
    desc: &str,
    checked: &mut bool,
    activations: &mut u32,
    bound_id: &mut egui::Id,
    label_disabled: bool,
    field_disabled: bool,
) -> (egui::Rect, bool, bool) {
    let target = *bound_id;
    let card = Field::new().disabled(field_disabled).show(ui, |ui| {
        FieldLabel::new("")
            .for_id(target)
            .checked(Some(*checked))
            .disabled(label_disabled)
            .id_salt(format!("{salt}-label"))
            .show_with_card(ui, |ui| {
                Field::new()
                    .orientation(FieldOrientation::Horizontal)
                    .show(ui, |ui| {
                        let resp = Checkbox::new(checked)
                            .id_salt(format!("{salt}-cb"))
                            .show(ui);
                        if resp.clicked() {
                            *activations += 1;
                        }
                        let focused = resp.has_focus();
                        *bound_id = resp.id;
                        FieldContent::new()
                            .id_salt(format!("{salt}-content"))
                            .show(ui, |ui| {
                                FieldTitle::new(title)
                                    .id_salt(format!("{salt}-title"))
                                    .show(ui);
                                FieldDescription::new(desc)
                                    .id_salt(format!("{salt}-desc"))
                                    .show(ui);
                            });
                        focused
                    })
                    .inner
            })
    });
    let card = card.inner;
    let outer = card.response.rect;
    let hovered = ui.rect_contains_pointer(outer);
    (outer, card.inner, hovered)
}

/// Detects whether a `Field` laid out horizontally (label and control
/// share a row) or vertically from their rects.
fn field_s3_orientation(label: egui::Rect, control: egui::Rect) -> &'static str {
    if (label.center().y - control.center().y).abs() < 5.0 {
        "horizontal"
    } else {
        "vertical"
    }
}

pub fn field_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("field_s3_demo_state");
    let mut state: FieldDemoState = ui
        .ctx()
        .data_mut(|d| d.get_temp(store))
        .unwrap_or_else(FieldDemoState::new);

    // Header: what field is + the honest positions (S5/S6 judge labels
    // as part of honesty).
    ui.label("Field — form layout + typography composition: 10 parts, no state of its own (spec §1). Nothing animates (spec §6).");
    ui.label("Honest positions: the 448px @md breakpoint is a Tailwind v4 framework default, NOT snapshot evidence (OPEN-1). OPEN-3 combined-state precedences are PROVISIONAL readings RATIFIED by gate G1. N14 self-wins is PROVISIONAL (RATIFIED, gate G1). Description co-match: legend-follow wins (OPEN-5, RATIFIED as PROVISIONAL, gate G1).");
    ui.label("Suppression: card sections below do NOT wire .suppress_focus_indication — field S3 cannot wire it (ruling 12 §3). Nested control rings in card captures are expected; rendered §4.8 verification happened at field S10 via scratch merge.");
    ui.label("All for_id bindings use ids captured from the control's returned Response::id on the previous pass (scoped-id rule); Id::NULL on the first pass targets no widget.");
    ui.add_space(12.0);

    // §1. All 10 parts in one base composition (spec §2). Each part is
    // exercised in detail in its own section below.
    ui.label("1. All 10 parts — FieldSet + FieldLegend + FieldGroup + Field + FieldContent + FieldLabel + FieldTitle + FieldDescription + FieldSeparator + FieldError (spec §2)");
    ui.add_space(4.0);
    FieldSet::new().id_salt("field-s3-s1-set").show(ui, |ui| {
        FieldLegend::new("Notification settings")
            .id_salt("field-s3-s1-legend")
            .show(ui);
        FieldDescription::new("Choose how you want to be contacted. These apply to all projects.")
            .id_salt("field-s3-s1-desc")
            .show(ui);
        FieldGroup::new()
            .id_salt("field-s3-s1-group")
            .show(ui, |ui| {
                Field::new().show(ui, |ui| {
                    FieldLabel::new("Push notifications")
                        .id_salt("field-s3-s1-label")
                        .show(ui);
                    FieldDescription::new("Send notifications to device.")
                        .id_salt("field-s3-s1-desc2")
                        .show(ui);
                    Checkbox::new(&mut state.s1_notif)
                        .id_salt("field-s3-s1-cb")
                        .show(ui);
                });
                Field::new().show(ui, |ui| {
                    FieldLabel::new("Email")
                        .id_salt("field-s3-s1-label2")
                        .show(ui);
                    FieldDescription::new("We use your email for the digest only.")
                        .id_salt("field-s3-s1-desc3")
                        .show(ui);
                    Input::new(&mut state.s1_email)
                        .placeholder("m@example.com")
                        .id_salt("field-s3-s1-input")
                        .show(ui);
                });
                Field::new().invalid(true).show(ui, |ui| {
                    FieldLabel::new("Name")
                        .id_salt("field-s3-s1-label3")
                        .show(ui);
                    Input::new(&mut state.s1_name)
                        .invalid(true)
                        .id_salt("field-s3-s1-input2")
                        .show(ui);
                    FieldError::new("Your display name cannot be empty.")
                        .id_salt("field-s3-s1-err")
                        .show(ui);
                });
                Field::new().show(ui, |ui| {
                    FieldLabel::new("Billing plan")
                        .id_salt("field-s3-s1-label4")
                        .show(ui);
                    FieldContent::new()
                        .id_salt("field-s3-s1-content")
                        .show(ui, |ui| {
                            FieldTitle::new("Pro plan")
                                .icon(Icon::User)
                                .id_salt("field-s3-s1-title")
                                .show(ui);
                            FieldDescription::new("Updates your plan to Pro.")
                                .id_salt("field-s3-s1-desc4")
                                .show(ui);
                        });
                });
            });
        FieldSeparator::content("or continue with")
            .id_salt("field-s3-s1-sep")
            .show(ui);
        FieldSeparator::new().id_salt("field-s3-s1-sep2").show(ui);
    });

    // §2. N14 conditional gaps (spec §2.1/§2.3/§3.1). Each case prints
    // its measured gap; S5 judges the pixels.
    ui.add_space(12.0);
    ui.label("2. N14 conditional gaps — measured gap printed per case (spec §3.1)");
    ui.add_space(4.0);

    // 2a. FieldGroup default: 28px base / 20px nova.
    ui.label("2a. FieldGroup default gap (expected: 28px base / 20px nova)");
    ui.add_space(4.0);
    let (r1, r2) = FieldGroup::new()
        .id_salt("field-s3-s2a-group")
        .show(ui, |ui| {
            let r1 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("First field")
                        .id_salt("field-s3-s2a-l1")
                        .show(ui);
                })
                .response
                .rect;
            let r2 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("Second field")
                        .id_salt("field-s3-s2a-l2")
                        .show(ui);
                })
                .response
                .rect;
            (r1, r2)
        })
        .inner;
    ui.label(format!(
        "measured gap: {:.1} px (expected: 28 base / 20 nova)",
        r2.min.y - r1.max.y
    ));
    ui.add_space(4.0);

    // 2b. Nested group: 16px in both styles.
    ui.label("2b. Nested FieldGroup gap (expected: 16px both styles)");
    ui.add_space(4.0);
    let (r1, r2) = FieldGroup::new()
        .id_salt("field-s3-s2b-outer")
        .show(ui, |ui| {
            FieldGroup::new()
                .id_salt("field-s3-s2b-inner")
                .show(ui, |ui| {
                    let r1 = Field::new()
                        .show(ui, |ui| {
                            FieldLabel::new("Nested first")
                                .id_salt("field-s3-s2b-l1")
                                .show(ui);
                        })
                        .response
                        .rect;
                    let r2 = Field::new()
                        .show(ui, |ui| {
                            FieldLabel::new("Nested second")
                                .id_salt("field-s3-s2b-l2")
                                .show(ui);
                        })
                        .response
                        .rect;
                    (r1, r2)
                })
                .inner
        })
        .inner;
    ui.label(format!(
        "measured gap: {:.1} px (expected: 16 both styles)",
        r2.min.y - r1.max.y
    ));
    ui.add_space(4.0);

    // 2c. Self-conditional checkbox-group slot: 12px in both styles.
    ui.label("2c. FieldGroup .checkbox_group(true) gap (expected: 12px both styles)");
    ui.add_space(4.0);
    let (r1, r2) = FieldGroup::new()
        .checkbox_group(true)
        .id_salt("field-s3-s2c-group")
        .show(ui, |ui| {
            let r1 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("Checkbox-group first")
                        .id_salt("field-s3-s2c-l1")
                        .show(ui);
                })
                .response
                .rect;
            let r2 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("Checkbox-group second")
                        .id_salt("field-s3-s2c-l2")
                        .show(ui);
                })
                .response
                .rect;
            (r1, r2)
        })
        .inner;
    ui.label(format!(
        "measured gap: {:.1} px (expected: 12 both styles)",
        r2.min.y - r1.max.y
    ));
    ui.add_space(4.0);

    // 2d. FieldSet default: 24px base / 16px nova.
    ui.label("2d. FieldSet default gap (expected: 24px base / 16px nova)");
    ui.add_space(4.0);
    let (r1, r2) = FieldSet::new()
        .id_salt("field-s3-s2d-set")
        .show(ui, |ui| {
            let r1 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("Set first")
                        .id_salt("field-s3-s2d-l1")
                        .show(ui);
                })
                .response
                .rect;
            let r2 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("Set second")
                        .id_salt("field-s3-s2d-l2")
                        .show(ui);
                })
                .response
                .rect;
            (r1, r2)
        })
        .inner;
    ui.label(format!(
        "measured gap: {:.1} px (expected: 24 base / 16 nova)",
        r2.min.y - r1.max.y
    ));
    ui.add_space(4.0);

    // 2e. FieldSet with a direct RadioGroup child: 12px in both styles.
    ui.label("2e. FieldSet .has_group_child(true) with a direct RadioGroup child (expected: 12px both styles; the rebuilt RadioGroup paints circles + labels)");
    ui.add_space(4.0);
    let (r1, rg, r2) = FieldSet::new()
        .has_group_child(true)
        .id_salt("field-s3-s2e-set")
        .show(ui, |ui| {
            let r1 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("Before group")
                        .id_salt("field-s3-s2e-l1")
                        .show(ui);
                })
                .response
                .rect;
            let rg = RadioGroup::new(&mut state.s2e_sel)
                .id_salt("field-s3-s2e-rg")
                .option("Default")
                .option("Comfortable")
                .option("Compact")
                .show(ui)
                .response
                .rect;
            let r2 = Field::new()
                .show(ui, |ui| {
                    FieldLabel::new("After group")
                        .id_salt("field-s3-s2e-l2")
                        .show(ui);
                })
                .response
                .rect;
            (r1, rg, r2)
        })
        .inner;
    ui.label(format!(
        "measured gaps: field->group {:.1} px, group->field {:.1} px (expected: 12 / 12)",
        rg.min.y - r1.max.y,
        r2.min.y - rg.max.y
    ));
    ui.add_space(4.0);

    // 2f. PROVISIONAL (RATIFIED, gate G1 item 2): a nested group that
    // also carries the checkbox-group slot renders 12px (self wins over
    // the nested 16px).
    ui.label("2f. PROVISIONAL — nested + .checkbox_group(true): self-wins (expected: 12px both styles; RATIFIED as PROVISIONAL, gate G1)");
    ui.add_space(4.0);
    let (r1, r2) = FieldGroup::new()
        .id_salt("field-s3-s2f-outer")
        .show(ui, |ui| {
            FieldGroup::new()
                .checkbox_group(true)
                .id_salt("field-s3-s2f-inner")
                .show(ui, |ui| {
                    let r1 = Field::new()
                        .show(ui, |ui| {
                            FieldLabel::new("Self-wins first")
                                .id_salt("field-s3-s2f-l1")
                                .show(ui);
                        })
                        .response
                        .rect;
                    let r2 = Field::new()
                        .show(ui, |ui| {
                            FieldLabel::new("Self-wins second")
                                .id_salt("field-s3-s2f-l2")
                                .show(ui);
                        })
                        .response
                        .rect;
                    (r1, r2)
                })
                .inner
        })
        .inner;
    ui.label(format!(
        "measured gap: {:.1} px (expected: 12 both styles, PROVISIONAL)",
        r2.min.y - r1.max.y
    ));

    // §3. Description margin rules (spec §2.8). Each case prints the
    // measured offset from the previous sibling's bottom to the
    // description's top; the container gap + legend margin are stated
    // so the frozen margin is derivable. Control cases sit beside
    // their variants so S5 can difference directly.
    ui.add_space(12.0);
    ui.label("3. Description margin rules — legend-follow -6px, second-from-last -4px, default 0 (spec §2.8)");
    ui.add_space(4.0);

    // 3a. Control: legend + description, no margin flag.
    ui.label("3a. Control: legend + description, no margin flag (expected offset: legend mb 12/6 + set gap 24/16 = 36 base / 22 nova)");
    ui.add_space(4.0);
    let (lr, dr) = FieldSet::new()
        .id_salt("field-s3-s3a-set")
        .show(ui, |ui| {
            let lr = FieldLegend::new("Section")
                .id_salt("field-s3-s3a-legend")
                .show(ui)
                .rect;
            let dr = FieldDescription::new("Description with no margin flag.")
                .id_salt("field-s3-s3a-desc")
                .show(ui)
                .rect;
            (lr, dr)
        })
        .inner;
    ui.label(format!(
        "measured offset: {:.1} px (expected: 36 base / 22 nova)",
        dr.min.y - lr.max.y
    ));
    ui.add_space(4.0);

    // 3b. Legend-follow: -6px.
    ui.label("3b. Legend-follow: description .after_legend(true) (expected offset: 36/22 minus 6 = 30 base / 16 nova)");
    ui.add_space(4.0);
    let (lr, dr) = FieldSet::new()
        .id_salt("field-s3-s3b-set")
        .show(ui, |ui| {
            let lr = FieldLegend::new("Section")
                .id_salt("field-s3-s3b-legend")
                .show(ui)
                .rect;
            let dr =
                FieldDescription::new("Description immediately after a legend-variant legend.")
                    .after_legend(true)
                    .id_salt("field-s3-s3b-desc")
                    .show(ui)
                    .rect;
            (lr, dr)
        })
        .inner;
    ui.label(format!(
        "measured offset: {:.1} px (expected: 30 base / 16 nova)",
        dr.min.y - lr.max.y
    ));
    ui.add_space(4.0);

    // 3c. Control: second-from-last position, no flag.
    ui.label("3c. Control: description second-from-last, no flag (expected offset: field gap 12 base / 8 nova)");
    ui.add_space(4.0);
    let (lr, dr) = Field::new()
        .show(ui, |ui| {
            let lr = FieldLabel::new("Name")
                .id_salt("field-s3-s3c-l1")
                .show(ui)
                .rect;
            let dr = FieldDescription::new("Description in second-from-last position, no flag.")
                .id_salt("field-s3-s3c-desc")
                .show(ui)
                .rect;
            FieldLabel::new("Following sibling")
                .id_salt("field-s3-s3c-l2")
                .show(ui);
            (lr, dr)
        })
        .inner;
    ui.label(format!(
        "measured offset: {:.1} px (expected: 12 base / 8 nova)",
        dr.min.y - lr.max.y
    ));
    ui.add_space(4.0);

    // 3d. Second-from-last: -4px.
    ui.label("3d. Second-from-last: description .second_from_last(true) (expected offset: 12/8 minus 4 = 8 base / 4 nova)");
    ui.add_space(4.0);
    let (lr, dr) = Field::new()
        .show(ui, |ui| {
            let lr = FieldLabel::new("Name")
                .id_salt("field-s3-s3d-l1")
                .show(ui)
                .rect;
            let dr = FieldDescription::new("Description in second-from-last position, flagged.")
                .second_from_last(true)
                .id_salt("field-s3-s3d-desc")
                .show(ui)
                .rect;
            FieldLabel::new("Following sibling")
                .id_salt("field-s3-s3d-l2")
                .show(ui);
            (lr, dr)
        })
        .inner;
    ui.label(format!(
        "measured offset: {:.1} px (expected: 8 base / 4 nova)",
        dr.min.y - lr.max.y
    ));
    ui.add_space(4.0);

    // 3e. OPEN-5 co-match: both flags — legend-follow wins (RATIFIED as PROVISIONAL, gate G1 item 8).
    ui.label("3e. OPEN-5 co-match: description .after_legend(true) + .second_from_last(true) — legend-follow wins, -6px (expected offset: 30 base / 16 nova; RATIFIED as PROVISIONAL, gate G1)");
    ui.add_space(4.0);
    let (lr, dr) = FieldSet::new()
        .id_salt("field-s3-s3e-set")
        .show(ui, |ui| {
            let lr = FieldLegend::new("Section")
                .id_salt("field-s3-s3e-legend")
                .show(ui)
                .rect;
            let dr = FieldDescription::new("Description after a legend AND second-from-last.")
                .after_legend(true)
                .second_from_last(true)
                .id_salt("field-s3-s3e-desc")
                .show(ui)
                .rect;
            Field::new().show(ui, |ui| {
                FieldLabel::new("Following field")
                    .id_salt("field-s3-s3e-l1")
                    .show(ui);
            });
            (lr, dr)
        })
        .inner;
    ui.label(format!(
        "measured offset: {:.1} px (expected: 30 base / 16 nova, PROVISIONAL)",
        dr.min.y - lr.max.y
    ));
    ui.add_space(4.0);

    // 3f. Label-variant legend does NOT trigger legend-follow (spec §9).
    ui.label("3f. Label-variant legend + description, no flag — the legend-follow rule keys on data-variant=legend ONLY (expected offset: 36 base / 22 nova, no tightening)");
    ui.add_space(4.0);
    let (lr, dr) = FieldSet::new()
        .id_salt("field-s3-s3f-set")
        .show(ui, |ui| {
            let lr = FieldLegend::new("Sub-section")
                .variant(FieldLegendVariant::Label)
                .id_salt("field-s3-s3f-legend")
                .show(ui)
                .rect;
            let dr = FieldDescription::new("Description after a label-variant legend.")
                .id_salt("field-s3-s3f-desc")
                .show(ui)
                .rect;
            (lr, dr)
        })
        .inner;
    ui.label(format!(
        "measured offset: {:.1} px (expected: 36 base / 22 nova)",
        dr.min.y - lr.max.y
    ));

    // §4. FieldError content algorithm — N13 dedup + render plan
    // (spec §2.10).
    ui.add_space(12.0);
    ui.label("4. FieldError content algorithm — N13 dedup + render plan (spec §2.10)");
    ui.add_space(4.0);

    // 4a. Single error: bare text.
    ui.label("4a. Single error — bare text, no list, no indent (spec §2.10 step 4)");
    ui.add_space(4.0);
    let r = FieldError::new("Your display name cannot be empty.")
        .id_salt("field-s3-s4a")
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: one text line, no indent, no bullet)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);

    // 4b. Multiple unique: list.
    ui.label(
        "4b. Three unique errors — list, 16px indent, disc bullets, 4px gaps (spec §2.10 step 5)",
    );
    ui.add_space(4.0);
    let r = FieldError::errors(vec![
        "Card declined".to_owned(),
        "Insufficient funds".to_owned(),
        "Expired card".to_owned(),
    ])
    .id_salt("field-s3-s4b")
    .show(ui)
    .rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: 3 bullets)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);

    // 4c. Duplicates collapse.
    ui.label("4c. Duplicates collapse — [declined, declined, funds] renders 2 bullets in first-occurrence order (spec §2.10 step 3)");
    ui.add_space(4.0);
    let r = FieldError::entries(vec![
        Some("Card declined".to_owned()),
        Some("Card declined".to_owned()),
        Some("Insufficient funds".to_owned()),
    ])
    .id_salt("field-s3-s4c")
    .show(ui)
    .rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: 2 bullets)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);

    // 4d. Falsy entry in a list.
    ui.label("4d. Falsy entry — [\"\", \"a\"] renders LIST form (deduped count 2) with one visible bullet; the falsy entry renders nothing (spec §2.10 steps 3/5)");
    ui.add_space(4.0);
    let r = FieldError::entries(vec![Some(String::new()), Some("a".to_owned())])
        .id_salt("field-s3-s4d")
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: list form, 1 bullet, 16px indent)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);

    // 4e. All falsy: nothing.
    ui.label("4e. All-falsy entries — renders NOTHING, zero size (spec §2.10 steps 3/4)");
    ui.add_space(4.0);
    let r = FieldError::entries(vec![None, Some(String::new())])
        .id_salt("field-s3-s4e")
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: 0 x 0)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);

    // 4f. Empty list: nothing.
    ui.label("4f. Empty errors list — renders NOTHING, zero size (spec §2.10 step 2)");
    ui.add_space(4.0);
    let r = FieldError::errors(Vec::new())
        .id_salt("field-s3-s4f")
        .show(ui)
        .rect;
    ui.label(format!(
        "measured: {:.1} x {:.1} px (expected: 0 x 0)",
        r.width(),
        r.height()
    ));
    ui.add_space(4.0);

    // 4g. Error after a legend: no margin rule.
    ui.label("4g. Error after a legend — NO negative margin: the description tightening rules do not apply to errors (spec §2.8)");
    ui.add_space(4.0);
    let (lr, er) = FieldSet::new()
        .id_salt("field-s3-s4g-set")
        .show(ui, |ui| {
            let lr = FieldLegend::new("Section")
                .id_salt("field-s3-s4g-legend")
                .show(ui)
                .rect;
            let er = FieldError::new("Something went wrong.")
                .id_salt("field-s3-s4g-err")
                .show(ui)
                .rect;
            (lr, er)
        })
        .inner;
    ui.label(format!(
        "measured offset: {:.1} px (expected: legend mb 12/6 + set gap 24/16 = 36 base / 22 nova, no tightening)",
        er.min.y - lr.max.y
    ));

    // §5. Responsive orientation (spec §2.4, OPEN-1). The switch
    // measures the nearest enclosing FieldGroup's width against 448px.
    ui.add_space(12.0);
    ui.label("5. Responsive orientation — the switch measures the nearest enclosing FieldGroup's width against 448px, a Tailwind v4 framework default, NOT snapshot evidence (OPEN-1; spec §2.4)");
    ui.add_space(4.0);

    // 5a. Wide group: horizontal.
    ui.label("5a. Responsive Field in a 600px group (expected: HORIZONTAL — 600 >= 448)");
    ui.add_space(4.0);
    let (lr, cr) = ui
        .allocate_ui(egui::Vec2::new(600.0, 80.0), |ui| {
            FieldGroup::new()
                .id_salt("field-s3-s5a-group")
                .show(ui, |ui| {
                    Field::new()
                        .orientation(FieldOrientation::Responsive)
                        .show(ui, |ui| {
                            let lr = FieldLabel::new("Wide group")
                                .id_salt("field-s3-s5a-label")
                                .show(ui)
                                .rect;
                            let cr = Switch::new(&mut state.s5_wide)
                                .id_salt("field-s3-s5a-sw")
                                .show(ui)
                                .rect;
                            (lr, cr)
                        })
                        .inner
                })
                .inner
        })
        .inner;
    ui.label(format!(
        "measured orientation: {} (expected: horizontal); on: {}",
        field_s3_orientation(lr, cr),
        state.s5_wide
    ));
    ui.add_space(4.0);

    // 5b. Narrow group: vertical.
    ui.label("5b. Responsive Field in a 300px group (expected: VERTICAL — 300 < 448)");
    ui.add_space(4.0);
    let (lr, cr) = ui
        .allocate_ui(egui::Vec2::new(300.0, 80.0), |ui| {
            FieldGroup::new()
                .id_salt("field-s3-s5b-group")
                .show(ui, |ui| {
                    Field::new()
                        .orientation(FieldOrientation::Responsive)
                        .show(ui, |ui| {
                            let lr = FieldLabel::new("Narrow group")
                                .id_salt("field-s3-s5b-label")
                                .show(ui)
                                .rect;
                            let cr = Switch::new(&mut state.s5_narrow)
                                .id_salt("field-s3-s5b-sw")
                                .show(ui)
                                .rect;
                            (lr, cr)
                        })
                        .inner
                })
                .inner
        })
        .inner;
    ui.label(format!(
        "measured orientation: {} (expected: vertical); on: {}",
        field_s3_orientation(lr, cr),
        state.s5_narrow
    ));
    ui.add_space(4.0);

    // 5c. Nested groups: the nearest container width governs.
    ui.label("5c. Responsive Field in a 300px group nested in a 600px group (expected: VERTICAL — the NEAREST container width governs, spec §9)");
    ui.add_space(4.0);
    let (lr, cr) = ui
        .allocate_ui(egui::Vec2::new(600.0, 100.0), |ui| {
            FieldGroup::new()
                .id_salt("field-s3-s5c-outer")
                .show(ui, |ui| {
                    ui.allocate_ui(egui::Vec2::new(300.0, 80.0), |ui| {
                        FieldGroup::new()
                            .id_salt("field-s3-s5c-inner")
                            .show(ui, |ui| {
                                Field::new()
                                    .orientation(FieldOrientation::Responsive)
                                    .show(ui, |ui| {
                                        let lr = FieldLabel::new("Nested group")
                                            .id_salt("field-s3-s5c-label")
                                            .show(ui)
                                            .rect;
                                        let cr = Switch::new(&mut state.s5_nested)
                                            .id_salt("field-s3-s5c-sw")
                                            .show(ui)
                                            .rect;
                                        (lr, cr)
                                    })
                                    .inner
                            })
                            .inner
                    })
                    .inner
                })
                .inner
        })
        .inner;
    ui.label(format!(
        "measured orientation: {} (expected: vertical); on: {}",
        field_s3_orientation(lr, cr),
        state.s5_nested
    ));
    ui.add_space(4.0);

    // 5d. No enclosing group: always columnar.
    ui.label("5d. Responsive Field with NO enclosing FieldGroup (expected: VERTICAL — no named container exists, the @md rules never match, spec §2.4)");
    ui.add_space(4.0);
    let (lr, cr) = Field::new()
        .orientation(FieldOrientation::Responsive)
        .show(ui, |ui| {
            let lr = FieldLabel::new("No group")
                .id_salt("field-s3-s5d-label")
                .show(ui)
                .rect;
            let cr = Switch::new(&mut state.s5_nogroup)
                .id_salt("field-s3-s5d-sw")
                .show(ui)
                .rect;
            (lr, cr)
        })
        .inner;
    ui.label(format!(
        "measured orientation: {} (expected: vertical); on: {}",
        field_s3_orientation(lr, cr),
        state.s5_nogroup
    ));

    // §6. Horizontal Field with a FieldContent child (spec §2.4,
    // OPEN-6): items-start + the mt-px flag.
    ui.add_space(12.0);
    ui.label("6. Horizontal Field with a FieldContent child — items-start + mt-px flag (spec §2.4, OPEN-6)");
    ui.add_space(4.0);

    // 6a. Content child present: top-aligned; the mt-px flag is SET
    // for the direct-child checkbox (unobservable from the field side).
    ui.label("6a. .has_content_child(true) — top-aligned row; mt-px flag SET for the direct-child checkbox (the 1px offset is consumed — checkbox sits 1px below the content top, ruling 12 §2)");
    ui.add_space(4.0);
    let page_right = ui.max_rect().max.x;
    let field_frame = Field::new()
        .orientation(FieldOrientation::Horizontal)
        .has_content_child(true)
        .show(ui, |ui| {
            let cb = Checkbox::new(&mut state.s6_content_cb)
                .id_salt("field-s3-s6a-cb")
                .show(ui)
                .rect;
            let content = FieldContent::new()
                .id_salt("field-s3-s6a-content")
                .show(ui, |ui| {
                    FieldTitle::new("Notifications")
                        .id_salt("field-s3-s6a-title")
                        .show(ui);
                    FieldDescription::new("Description text so the content column is taller than the checkbox, making the top alignment measurable.")
                        .id_salt("field-s3-s6a-desc")
                        .show(ui);
                })
                .response
                .rect;
            (cb, content)
        });
    let field_rect = field_frame.response.rect;
    let (cb_rect, content_rect) = field_frame.inner;
    ui.label(format!(
        "measured: content-top minus checkbox-top {:.1} px (expected: -1.0, items-start); content right {:.1}, row right {:.1} (expected: equal, flex-1 fills the row)",
        content_rect.min.y - cb_rect.min.y,
        content_rect.max.x,
        page_right
    ));
    ui.label(format!(
        "field rect: {:.1} x {:.1} px; checkbox checked: {}",
        field_rect.width(),
        field_rect.height(),
        state.s6_content_cb
    ));
    ui.add_space(4.0);

    // 6b. Control: horizontal without a content child — centered row.
    ui.label("6b. Control: horizontal Field WITHOUT a content child — centered row, no mt-px flag (spec §2.4)");
    ui.add_space(4.0);
    let (lr, cb) = Field::new()
        .orientation(FieldOrientation::Horizontal)
        .show(ui, |ui| {
            let lr = FieldLabel::new("Centered label")
                .id_salt("field-s3-s6b-label")
                .show(ui)
                .rect;
            let cb = Checkbox::new(&mut state.s6_plain_cb)
                .id_salt("field-s3-s6b-cb")
                .show(ui)
                .rect;
            (lr, cb)
        })
        .inner;
    ui.label(format!(
        "measured: label-center minus checkbox-center {:.1} px (expected: ~0, items-center); checked: {}",
        lr.center().y - cb.center().y,
        state.s6_plain_cb
    ));

    // §7. Choice cards (F2 + N9). Every card section carries the
    // suppression label (ruling 12 §3): the nested control's ring is
    // NOT suppressed in this demo — field S3 cannot wire it, and
    // §4.8 was verified at S10 via scratch merge.
    ui.add_space(12.0);
    ui.label("7. Choice cards — FieldLabel wrapping a nested Field (F2 click forwarding + N9 focus-inside, spec §2.6/§4.5/§7.2)");
    ui.label("Suppression is NOT wired in this demo — field S3 cannot wire it (ruling 12 §3); nested control rings visible in the cards below are expected. Rendered §4.8 verification happened at field S10 via scratch merge.");
    ui.add_space(4.0);

    // 7a. Card + switch (unchecked resting state; N9 target).
    ui.label("7a. Card + switch — unchecked resting state; N9 focus-inside target (nova: border ring + 3px ring/50 halo; base: no card chrome). Suppression NOT wired here (ruling 12 §3); nested control rings expected — §4.8 verified at S10 via scratch merge.");
    ui.add_space(4.0);
    let target = state.s7a_id;
    let frame = FieldLabel::new("")
        .for_id(target)
        .checked(Some(state.s7a_sw))
        .id_salt("field-s3-s7a-label")
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    let resp = Switch::new(&mut state.s7a_sw)
                        .id_salt("field-s3-s7a-sw")
                        .show(ui);
                    if resp.clicked() {
                        state.s7a_act += 1;
                    }
                    let focused = resp.has_focus();
                    state.s7a_id = resp.id;
                    FieldContent::new()
                        .id_salt("field-s3-s7a-content")
                        .show(ui, |ui| {
                            FieldTitle::new("Push notifications")
                                .id_salt("field-s3-s7a-title")
                                .show(ui);
                            FieldDescription::new("Send notifications to this device.")
                                .id_salt("field-s3-s7a-desc")
                                .show(ui);
                        });
                    focused
                })
                .inner
        });
    let outer = frame.response.rect;
    ui.label(format!(
        "card {:.1} x {:.1} px; on: {}; focused: {}; hovered: {}; activations: {} (click the card body: exactly +1 and focus persists; click the switch directly: exactly +1, no re-forward)",
        outer.width(),
        outer.height(),
        state.s7a_sw,
        frame.inner,
        ui.rect_contains_pointer(outer),
        state.s7a_act
    ));
    ui.add_space(4.0);

    // 7b. Card + checkbox (checked resting state).
    ui.label("7b. Card + checkbox — CHECKED resting state (initially on). Suppression NOT wired here (ruling 12 §3); nested control rings expected — §4.8 verified at S10 via scratch merge.");
    ui.add_space(4.0);
    let target = state.s7b_id;
    let frame = FieldLabel::new("")
        .for_id(target)
        .checked(Some(state.s7b_cb))
        .id_salt("field-s3-s7b-label")
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    let resp = Checkbox::new(&mut state.s7b_cb)
                        .id_salt("field-s3-s7b-cb")
                        .show(ui);
                    if resp.clicked() {
                        state.s7b_act += 1;
                    }
                    let focused = resp.has_focus();
                    state.s7b_id = resp.id;
                    FieldContent::new()
                        .id_salt("field-s3-s7b-content")
                        .show(ui, |ui| {
                            FieldTitle::new("Email digest")
                                .id_salt("field-s3-s7b-title")
                                .show(ui);
                            FieldDescription::new("Receive a weekly summary of activity.")
                                .id_salt("field-s3-s7b-desc")
                                .show(ui);
                        });
                    focused
                })
                .inner
        });
    let outer = frame.response.rect;
    ui.label(format!(
        "card {:.1} x {:.1} px; checked: {}; focused: {}; hovered: {}; activations: {} (click the card body: exactly +1 and focus persists; click the checkbox directly: exactly +1, no re-forward)",
        outer.width(),
        outer.height(),
        state.s7b_cb,
        frame.inner,
        ui.rect_contains_pointer(outer),
        state.s7b_act
    ));
    ui.add_space(4.0);

    // 7c. Card + standalone radio item.
    ui.label("7c. Card + standalone radio item — unchecked resting state. Suppression NOT wired here (ruling 12 §3); nested control rings expected — §4.8 verified at S10 via scratch merge.");
    ui.add_space(4.0);
    let target = state.s7c_id;
    let frame = FieldLabel::new("")
        .for_id(target)
        .checked(Some(state.s7c_radio))
        .id_salt("field-s3-s7c-label")
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    let resp = RadioGroupItem::new(&mut state.s7c_radio)
                        .id_salt("field-s3-s7c-radio")
                        .show(ui);
                    if resp.clicked() {
                        state.s7c_act += 1;
                    }
                    let focused = resp.has_focus();
                    state.s7c_id = resp.id;
                    FieldContent::new()
                        .id_salt("field-s3-s7c-content")
                        .show(ui, |ui| {
                            FieldTitle::new("Compact density")
                                .id_salt("field-s3-s7c-title")
                                .show(ui);
                            FieldDescription::new("Use the compact layout density.")
                                .id_salt("field-s3-s7c-desc")
                                .show(ui);
                        });
                    focused
                })
                .inner
        });
    let outer = frame.response.rect;
    ui.label(format!(
        "card {:.1} x {:.1} px; selected: {}; focused: {}; hovered: {}; activations: {} (click the card body: exactly +1 and focus persists; click the item directly: exactly +1, no re-forward)",
        outer.width(),
        outer.height(),
        state.s7c_radio,
        frame.inner,
        ui.rect_contains_pointer(outer),
        state.s7c_act
    ));
    ui.add_space(4.0);

    // 7d. Card + text input: card clicks focus the input (no
    // activation, no text inserted).
    ui.label("7d. Card + text input — card clicks FOCUS the input (no activation, no text inserted). Suppression NOT wired here (ruling 12 §3); nested control rings expected — §4.8 verified at S10 via scratch merge.");
    ui.add_space(4.0);
    let target = state.s7d_id;
    let frame = FieldLabel::new("")
        .for_id(target)
        .id_salt("field-s3-s7d-label")
        .show_with_card(ui, |ui| {
            Field::new()
                .show(ui, |ui| {
                    FieldTitle::new("Reply-to address")
                        .id_salt("field-s3-s7d-title")
                        .show(ui);
                    FieldDescription::new("Replies go to this address.")
                        .id_salt("field-s3-s7d-desc")
                        .show(ui);
                    let resp = Input::new(&mut state.s7d_text)
                        .placeholder("you@example.com")
                        .id_salt("field-s3-s7d-input")
                        .show(ui);
                    let focused = resp.has_focus();
                    state.s7d_id = resp.id;
                    focused
                })
                .inner
        });
    let outer = frame.response.rect;
    ui.label(format!(
        "card {:.1} x {:.1} px; input focused: {}; hovered: {}; text: {:?} (click the card body: input gains focus, text unchanged)",
        outer.width(),
        outer.height(),
        frame.inner,
        ui.rect_contains_pointer(outer),
        state.s7d_text
    ));
    ui.add_space(4.0);

    // 7e. Disabled card: 50% opacity, no hover fill, clicks pass
    // through.
    ui.label("7e. Disabled card — label disabled + switch disabled: 50% opacity, NO hover fill, clicks pass through (spec §4.2/§4.3). Suppression NOT wired here (ruling 12 §3); nested control rings expected — §4.8 verified at S10 via scratch merge.");
    ui.add_space(4.0);
    let target = state.s7e_id;
    let frame = FieldLabel::new("")
        .for_id(target)
        .checked(Some(state.s7e_sw))
        .disabled(true)
        .id_salt("field-s3-s7e-label")
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    let resp = Switch::new(&mut state.s7e_sw)
                        .disabled(true)
                        .id_salt("field-s3-s7e-sw")
                        .show(ui);
                    if resp.clicked() {
                        state.s7e_act += 1;
                    }
                    let focused = resp.has_focus();
                    state.s7e_id = resp.id;
                    FieldContent::new()
                        .id_salt("field-s3-s7e-content")
                        .show(ui, |ui| {
                            FieldTitle::new("Beta features")
                                .id_salt("field-s3-s7e-title")
                                .show(ui);
                            FieldDescription::new("Unavailable on your current plan.")
                                .id_salt("field-s3-s7e-desc")
                                .show(ui);
                        });
                    focused
                })
                .inner
        });
    let outer = frame.response.rect;
    ui.label(format!(
        "card {:.1} x {:.1} px; on: {}; focused: {}; hovered: {}; activations: {} (expected: 0 — clicks pass through)",
        outer.width(),
        outer.height(),
        state.s7e_sw,
        frame.inner,
        ui.rect_contains_pointer(outer),
        state.s7e_act
    ));
    ui.add_space(4.0);

    // 7f. COMBINED focus+checked card — observed without verdict
    // (OPEN-3 provisional precedence). Non-empty label text renders
    // above the wrapped field.
    ui.label("7f. COMBINED focus+checked card — OBSERVED WITHOUT VERDICT (OPEN-3 provisional: focus-inside wins over checked for the border; RATIFIED as PROVISIONAL, gate G1 item 1). Non-empty label text renders above the wrapped field. Suppression NOT wired here (ruling 12 §3); nested control rings expected — §4.8 verified at S10 via scratch merge.");
    ui.add_space(4.0);
    let target = state.s7f_id;
    let frame = FieldLabel::new("Notifications")
        .for_id(target)
        .checked(Some(state.s7f_cb))
        .id_salt("field-s3-s7f-label")
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    let resp = Checkbox::new(&mut state.s7f_cb)
                        .id_salt("field-s3-s7f-cb")
                        .show(ui);
                    if resp.clicked() {
                        state.s7f_act += 1;
                    }
                    let focused = resp.has_focus();
                    state.s7f_id = resp.id;
                    FieldContent::new()
                        .id_salt("field-s3-s7f-content")
                        .show(ui, |ui| {
                            FieldTitle::new("Marketing emails")
                                .id_salt("field-s3-s7f-title")
                                .show(ui);
                            FieldDescription::new("Receive news and special offers.")
                                .id_salt("field-s3-s7f-desc")
                                .show(ui);
                        });
                    focused
                })
                .inner
        });
    let outer = frame.response.rect;
    ui.label(format!(
        "card {:.1} x {:.1} px; checked: {}; focused: {}; hovered: {}; activations: {} (combined case: focus the nested checkbox to observe the focus+checked border — no verdict)",
        outer.width(),
        outer.height(),
        state.s7f_cb,
        frame.inner,
        ui.rect_contains_pointer(outer),
        state.s7f_act
    ));

    // §8. Disabled distinction (RATIFIED as settled, gate G1 item 3, D1): the label's OWN
    // .disabled() is the pointer-events-none condition (clicks pass
    // through); the enclosing Field's flag is the opacity-only
    // condition (clicks still forward). Both render 50% opacity. The
    // checkboxes stay enabled in all four combinations so only the
    // label/field flags vary.
    ui.add_space(12.0);
    ui.label("8. Disabled distinction — own-disabled (pass-through) vs field-disabled (opacity-only, still forwards) (spec §4.2/§7.2; RATIFIED as settled, gate G1 item 3)");
    ui.add_space(4.0);

    ui.label("8a–8d. Plain label + checkbox — click each LABEL: own=false must toggle exactly once; own=true must not toggle (pass-through).");
    ui.add_space(4.0);
    for (i, (own, field)) in [(false, false), (true, false), (false, true), (true, true)]
        .iter()
        .enumerate()
    {
        let salt = format!("field-s3-s8p{i}");
        let (_lr, focused) = field_s3_plain_pair(
            ui,
            &salt,
            &format!("Pair {i} (own-disabled={own}, field-disabled={field})"),
            &mut state.s8_plain[i],
            &mut state.s8_plain_act[i],
            &mut state.s8_plain_id[i],
            *own,
            *field,
        );
        ui.label(format!(
            "pair {i}: checked: {}; focused: {}; activations: {} (expected forwarding: {})",
            state.s8_plain[i],
            focused,
            state.s8_plain_act[i],
            if *own {
                "NONE (pass-through)"
            } else {
                "forwards"
            }
        ));
        ui.add_space(4.0);
    }

    ui.label("8e–8h. Choice card + checkbox — click each CARD BODY: own=false must toggle exactly once; own=true must not toggle (pass-through). Suppression NOT wired here (ruling 12 §3); nested control rings expected — §4.8 verified at S10 via scratch merge.");
    ui.add_space(4.0);
    for (i, (own, field)) in [(false, false), (true, false), (false, true), (true, true)]
        .iter()
        .enumerate()
    {
        let salt = format!("field-s3-s8c{i}");
        let (_outer, focused, hovered) = field_s3_card_pair(
            ui,
            &salt,
            &format!("Card {i}"),
            &format!("own-disabled={own}, field-disabled={field}"),
            &mut state.s8_card[i],
            &mut state.s8_card_act[i],
            &mut state.s8_card_id[i],
            *own,
            *field,
        );
        ui.label(format!(
            "card {i}: checked: {}; focused: {}; hovered: {}; activations: {} (expected forwarding: {})",
            state.s8_card[i],
            focused,
            hovered,
            state.s8_card_act[i],
            if *own { "NONE (pass-through)" } else { "forwards" }
        ));
        ui.add_space(4.0);
    }

    // §9. Plain-label click table (F2 / settled label rule, spec §7.2):
    // a label click fires a click on the bound control AND focuses it;
    // focus persists; exactly once per click.
    ui.add_space(12.0);
    ui.label("9. Plain-label click table — click fires + focuses, exactly once (spec §7.2)");
    ui.add_space(4.0);

    // 9a. Text input: focus, no activation, no text inserted.
    ui.label(
        "9a. Label -> text input: click focuses the input; no text is inserted, no value changes.",
    );
    ui.add_space(4.0);
    let target = state.s9a_id;
    let (lr, focused) = Field::new()
        .orientation(FieldOrientation::Horizontal)
        .show(ui, |ui| {
            let lr = FieldLabel::new("Email")
                .for_id(target)
                .id_salt("field-s3-s9a-label")
                .show(ui)
                .rect;
            let resp = Input::new(&mut state.s9a_text)
                .placeholder("m@example.com")
                .id_salt("field-s3-s9a-input")
                .show(ui);
            let focused = resp.has_focus();
            state.s9a_id = resp.id;
            (lr, focused)
        })
        .inner;
    ui.label(format!(
        "label {:.1} x {:.1} px; input focused: {}; text: {:?}",
        lr.width(),
        lr.height(),
        focused,
        state.s9a_text
    ));
    ui.add_space(4.0);

    // 9b. Checkbox: activation + focus.
    ui.label("9b. Label -> checkbox: click toggles AND focuses, exactly once (clicking the checkbox directly also toggles exactly once).");
    ui.add_space(4.0);
    let target = state.s9b_id;
    let (lr, focused) = Field::new()
        .orientation(FieldOrientation::Horizontal)
        .show(ui, |ui| {
            let lr = FieldLabel::new("Accept terms")
                .for_id(target)
                .id_salt("field-s3-s9b-label")
                .show(ui)
                .rect;
            let resp = Checkbox::new(&mut state.s9b_cb)
                .id_salt("field-s3-s9b-cb")
                .show(ui);
            if resp.clicked() {
                state.s9b_act += 1;
            }
            let focused = resp.has_focus();
            state.s9b_id = resp.id;
            (lr, focused)
        })
        .inner;
    ui.label(format!(
        "label {:.1} x {:.1} px; checked: {}; focused: {}; activations: {}",
        lr.width(),
        lr.height(),
        state.s9b_cb,
        focused,
        state.s9b_act
    ));
    ui.add_space(4.0);

    // 9c. Switch: activation + focus.
    ui.label("9c. Label -> switch: click toggles AND focuses, exactly once.");
    ui.add_space(4.0);
    let target = state.s9c_id;
    let (lr, focused) = Field::new()
        .orientation(FieldOrientation::Horizontal)
        .show(ui, |ui| {
            let lr = FieldLabel::new("Notifications")
                .for_id(target)
                .id_salt("field-s3-s9c-label")
                .show(ui)
                .rect;
            let resp = Switch::new(&mut state.s9c_sw)
                .id_salt("field-s3-s9c-sw")
                .show(ui);
            if resp.clicked() {
                state.s9c_act += 1;
            }
            let focused = resp.has_focus();
            state.s9c_id = resp.id;
            (lr, focused)
        })
        .inner;
    ui.label(format!(
        "label {:.1} x {:.1} px; on: {}; focused: {}; activations: {}",
        lr.width(),
        lr.height(),
        state.s9c_sw,
        focused,
        state.s9c_act
    ));
    ui.add_space(4.0);

    // 9d. Standalone radio item: activation + focus.
    ui.label("9d. Label -> standalone radio item: click selects AND focuses, exactly once.");
    ui.add_space(4.0);
    let target = state.s9d_id;
    let (lr, focused) = Field::new()
        .orientation(FieldOrientation::Horizontal)
        .show(ui, |ui| {
            let lr = FieldLabel::new("Option A")
                .for_id(target)
                .id_salt("field-s3-s9d-label")
                .show(ui)
                .rect;
            let resp = RadioGroupItem::new(&mut state.s9d_radio)
                .id_salt("field-s3-s9d-radio")
                .show(ui);
            if resp.clicked() {
                state.s9d_act += 1;
            }
            let focused = resp.has_focus();
            state.s9d_id = resp.id;
            (lr, focused)
        })
        .inner;
    ui.label(format!(
        "label {:.1} x {:.1} px; selected: {}; focused: {}; activations: {}",
        lr.width(),
        lr.height(),
        state.s9d_radio,
        focused,
        state.s9d_act
    ));
    ui.add_space(4.0);

    // 9e. No bound control: no control-directed effect.
    ui.label(
        "9e. Label with NO bound control: clicks have no control-directed effect (spec §7.2).",
    );
    ui.add_space(4.0);
    Field::new()
        .orientation(FieldOrientation::Horizontal)
        .show(ui, |ui| {
            FieldLabel::new("Unbound label")
                .id_salt("field-s3-s9e-label")
                .show(ui);
            Checkbox::new(&mut state.s9e_cb)
                .id_salt("field-s3-s9e-cb")
                .show(ui);
        });
    ui.label(format!(
        "checked: {} (changes only on direct checkbox clicks)",
        state.s9e_cb
    ));

    // §10. Field-level states (spec §4.4/§4.6). Card checked/hover
    // single-condition states live in §7 (7b checked resting, 7a hover
    // target); the combined focus+checked case is §7f.
    ui.add_space(12.0);
    ui.label("10. Field states — invalid + disabled (spec §4.4/§4.6; card checked/hover states: see §7; combined focus+checked: see §7f)");
    ui.add_space(4.0);

    // 10a. Invalid: label/title destructive; description + error keep
    // their own colours.
    ui.label("10a. Invalid Field — label + title text destructive (inherited); description keeps muted-foreground; error keeps destructive (spec §4.4)");
    ui.add_space(4.0);
    Field::new().invalid(true).show(ui, |ui| {
        FieldLabel::new("Username")
            .id_salt("field-s3-s10a-label")
            .show(ui);
        FieldTitle::new("Account name")
            .id_salt("field-s3-s10a-title")
            .show(ui);
        FieldDescription::new("This description keeps muted-foreground under an invalid field.")
            .id_salt("field-s3-s10a-desc")
            .show(ui);
        Input::new(&mut state.s10a_text)
            .invalid(true)
            .id_salt("field-s3-s10a-input")
            .show(ui);
        FieldError::new("That username is taken.")
            .id_salt("field-s3-s10a-err")
            .show(ui);
    });
    ui.add_space(4.0);

    // 10b. Disabled: label/title 50%; description has NO disabled fade.
    ui.label("10b. Disabled Field — label + title at 50% opacity; description has NO disabled fade (spec §4.2/§4.6)");
    ui.add_space(4.0);
    Field::new().disabled(true).show(ui, |ui| {
        FieldLabel::new("Workspace URL")
            .id_salt("field-s3-s10b-label")
            .show(ui);
        FieldTitle::new("Workspace")
            .id_salt("field-s3-s10b-title")
            .show(ui);
        FieldDescription::new("This description does NOT fade under a disabled field.")
            .id_salt("field-s3-s10b-desc")
            .show(ui);
        Input::new(&mut state.s10b_text)
            .disabled(true)
            .id_salt("field-s3-s10b-input")
            .show(ui);
    });

    // §11. Zero Tab stops (spec §7.4): no section — the statement is
    // the requirement; S6 verifies it in the Tab walk.
    ui.add_space(12.0);
    ui.label("11. Zero Tab stops — no field part takes a Tab stop or shows a focus ring of its own (spec §7.1/§7.4); the card focus-inside ring in §7 is a reflection of a DESCENDANT's focus, not the card's own. S6 verifies zero field stops in the Tab walk.");

    ui.ctx().data_mut(|d| d.insert_temp(store, state));
}

pub fn input_group_demo(ui: &mut egui::Ui) {
    let url_store = egui::Id::unique("ig_demo_url");
    let search_store = egui::Id::unique("ig_demo_search");
    let bio_store = egui::Id::unique("ig_demo_bio");
    let invalid_store = egui::Id::unique("ig_demo_invalid");
    let mut url: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(url_store).unwrap_or_default());
    let mut search: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(search_store).unwrap_or_default());
    let mut bio: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(bio_store).unwrap_or_default());
    let mut bad: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(invalid_store).unwrap_or_default());

    // Header: recipe position for S5 captures (spec §§2–4).
    ui.label("Container 36px base / 32px nova (border-box), radius 8/10px; inner controls frameless — all chrome on the container.");
    ui.add_space(12.0);

    ui.label("§1 — Inline-start text addon + inline-end icon button (Copy, IconXs)");
    ui.add_space(4.0);
    InputGroup::new().id_salt("ig_demo_url").show(ui, |ui| {
        InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
            InputGroupText::new("https://").show(ui);
        });
        InputGroupInput::new(&mut url)
            .placeholder("example.com")
            .show(ui);
        InputGroupAddon::new(AddonAlign::InlineEnd).show(ui, |ui| {
            InputGroupButton::icon(Icon::Copy).show(ui);
        });
    });
    ui.small(format!("url: {url:?}"));

    ui.add_space(12.0);
    ui.label("§2 — Icon addon + KbdGroup (⌘K) — KBD-VIS-02 centred-parent composition");
    ui.add_space(4.0);
    InputGroup::new().id_salt("ig_demo_search").show(ui, |ui| {
        InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
            InputGroupText::new("").icon(Icon::Search).show(ui);
        });
        InputGroupInput::new(&mut search)
            .placeholder("Search anything…")
            .show(ui);
        InputGroupAddon::new(AddonAlign::InlineEnd).show(ui, |ui| {
            KbdGroup::new().show(ui, |ui| {
                Kbd::new("⌘").show(ui);
                Kbd::new("K").show(ui);
            });
        });
    });

    ui.add_space(12.0);
    // Label fix (S1 report §9.3): the column comes from the block addons,
    // not the textarea — the textarea gives auto height, row kept (§3.1).
    ui.label(
        "§3 — Block addons + textarea (column from block addons; textarea auto-height, row kept)",
    );
    ui.add_space(4.0);
    InputGroup::new().id_salt("ig_demo_bio").show(ui, |ui| {
        InputGroupAddon::new(AddonAlign::BlockStart).show(ui, |ui| {
            InputGroupText::new("Bio").icon(Icon::User).show(ui);
        });
        InputGroupTextarea::new(&mut bio)
            .rows(3)
            .placeholder("Tell us about yourself…")
            .show(ui);
        InputGroupAddon::new(AddonAlign::BlockEnd).show(ui, |ui| {
            InputGroupText::new("Markdown supported").show(ui);
        });
    });

    ui.add_space(12.0);
    ui.label("§4 — Invalid input (destructive border; nova 3px ring at rest) — click to focus for the combined state (OPEN-2: invalid wins)");
    ui.add_space(4.0);
    InputGroup::new().id_salt("ig_demo_invalid").show(ui, |ui| {
        InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
            InputGroupText::new("").icon(Icon::CircleAlert).show(ui);
        });
        InputGroupInput::new(&mut bad)
            .invalid(true)
            .placeholder("Invalid URL")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("§5 — Disabled group, mechanism (a) — whole-container 50% fade (nova)");
    ui.add_space(4.0);
    InputGroup::new()
        .id_salt("ig_demo_disabled")
        .width(280.0)
        .disabled(true)
        .show(ui, |ui| {
            InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
                InputGroupText::new("https://").show(ui);
            });
            InputGroupInput::new(&mut url)
                .disabled(true)
                .placeholder("Disabled")
                .show(ui);
        });

    ui.add_space(12.0);
    ui.label("§6 — Button sizes (Sm “Search” + IconXs Copy)");
    ui.add_space(4.0);
    InputGroup::new().id_salt("ig_demo_sizes").show(ui, |ui| {
        InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
            InputGroupText::new("").icon(Icon::Search).show(ui);
        });
        InputGroupInput::new(&mut search)
            .placeholder("Find a workspace…")
            .show(ui);
        InputGroupAddon::new(AddonAlign::InlineEnd).show(ui, |ui| {
            InputGroupButton::new("Search")
                .size(InputGroupButtonSize::Sm)
                .show(ui);
            InputGroupButton::icon(Icon::Copy).show(ui);
        });
    });
    // Carried S1 limitation (§8.6): this addon holds TWO direct children,
    // so the spec `gap-2` (8px) between the buttons is NOT rendered
    // (`item_spacing.x = 0` on the addon row). Judge the button SIZES here;
    // the missing inter-button gap is a recorded limitation, not a defect.
    ui.small("§6 note: the two buttons render with no gap between them (carried S1 limitation, S1 report §8.6).");

    ui.add_space(12.0);
    ui.label("§7 — Fused inside a ButtonGroup (InputGroup First + Button “Go”)");
    ui.add_space(4.0);
    ButtonGroup::new().show(ui, |ui| {
        InputGroup::new()
            .id_salt("ig_demo_fused")
            .width(240.0)
            .show(ui, |ui| {
                InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
                    InputGroupText::new("").icon(Icon::Search).show(ui);
                });
                InputGroupInput::new(&mut search).show(ui);
            });
        Button::new("Go").show(ui);
    });

    ui.add_space(12.0);
    ui.label("§8 — Group-disabled flag (addons fade 50%; container chrome unchanged)");
    ui.add_space(4.0);
    InputGroup::new()
        .id_salt("ig_demo_group_disabled")
        .group_disabled(true)
        .show(ui, |ui| {
            InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
                InputGroupText::new("https://").show(ui);
            });
            InputGroupInput::new(&mut url)
                .placeholder("example.com")
                .show(ui);
        });
    ui.small("§8 note: group flag only (OPEN-3 mechanism (b)) — the control stays enabled; only the addon fades. Compare §5, where the disabled control fades the whole container (mechanism (a), nova).");

    ui.ctx().data_mut(|d| {
        d.insert_temp(url_store, url);
        d.insert_temp(search_store, search);
        d.insert_temp(bio_store, bio);
        d.insert_temp(invalid_store, bad);
    });
}

pub fn card_demo(ui: &mut egui::Ui) {
    // Header: recipe position, stated honestly (spec §§0, 2, 10, 11).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Card — a layout container with no behaviour of its own. Active recipe: {recipe} (style: {}, mode: {mode}).",
        theme.style.name()
    ));
    ui.label("Title weights: 600 base / 500 nova (Inter SemiBold / Medium, embedded). Nova edge: 1px border + 1px ring (foreground/10) + shadow-sm; base edge: 1px border + shadow-sm. Footer children sit with gap 0. Honest notes: D1 — the header action is a top-right overlay, so a taller-than-header action does not grow the header. D3 — base unstyled content 14-vs-16px is a theme-level note, not a card defect. OPEN-1 — the sm title rule (14px) is frozen; its group/card attachment is the port's CardCtx size stack.");
    let title_lh = if nova { 22.0 } else { 16.0 };

    ui.add_space(16.0);
    ui.label("1. Default — two-button footer (gap 0; nova footer band bleeds to the card's bottom inner edge)");
    ui.add_space(4.0);
    let default_card = Card::new().show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Create project").show(ui);
            CardDescription::new("Deploy your new project in one-click.").show(ui);
        });
        CardContent::new().show(ui, |ui| {
            Label::new("Name").show(ui);
            let store = egui::Id::unique("card_demo_name");
            let mut name: String = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or_default());
            Input::new(&mut name)
                .placeholder("Name of your project")
                .show(ui);
            ui.ctx().data_mut(|d| d.insert_temp(store, name));
        });
        CardFooter::new().show(ui, |ui| {
            Button::new("Create").show(ui);
            Button::new("Cancel")
                .variant(ButtonVariant::Outline)
                .show(ui);
        });
    });
    ui.label(format!(
        "measured: {:.1} x {:.1} px",
        default_card.response.rect.width(),
        default_card.response.rect.height()
    ));

    ui.add_space(16.0);
    ui.label("2. With a header action (top-right slot)");
    ui.add_space(4.0);
    Card::new().show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Notifications").show(ui);
            CardDescription::new("Control how you receive updates.").show(ui);
            CardAction::new().show(ui, |ui| {
                Button::new("")
                    .size(ButtonSize::IconSm)
                    .variant(ButtonVariant::Outline)
                    .icon_start(Icon::MoreHorizontal)
                    .show(ui);
            });
        });
        CardContent::new().show(ui, |ui| {
            ui.label("You have 3 unread notifications.");
        });
    });

    ui.add_space(16.0);
    ui.label("3. Destructive action in a card");
    ui.add_space(4.0);
    Card::new().show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Danger zone").show(ui);
            CardDescription::new("Deleting a project is permanent.").show(ui);
        });
        CardContent::new().show(ui, |ui| {
            ui.label("All resources will be removed immediately.");
        });
        CardFooter::new().show(ui, |ui| {
            Button::new("Delete project")
                .variant(ButtonVariant::Destructive)
                .show(ui);
        });
    });

    ui.add_space(16.0);
    ui.label("4. Compact (size=sm) card — nova only: 12px spacing, 14px title; no effect under non-nova styles");
    ui.add_space(4.0);
    let sm_card = Card::new().size(CardSize::Sm).show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Compact card").show(ui);
            CardDescription::new("Tighter spacing and a smaller title.").show(ui);
        });
        CardContent::new().show(ui, |ui| {
            ui.label("Nova shrinks --card-spacing to 12px and the title to 14px.");
        });
    });
    ui.label(format!(
        "measured: {:.1} x {:.1} px",
        sm_card.response.rect.width(),
        sm_card.response.rect.height()
    ));

    ui.add_space(16.0);
    ui.label("5. Card with a top image slot (D2(a) — nova-only full-bleed)");
    ui.add_space(4.0);
    let image_card = Card::new().image_top(card_demo_texture(ui)).show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Landscape").show(ui);
            CardDescription::new("Top image: nova bleeds full-bleed, base sits below the padding.").show(ui);
        });
        CardContent::new().show(ui, |ui| {
            ui.label("Nova: the image bleeds to the card's top edge (pt-0, 14px top corners, clipped). Base: it sits below the 24px top padding with square corners.");
        });
    });
    ui.label(format!(
        "measured: {:.1} x {:.1} px (nova: top padding 0; base: 24px)",
        image_card.response.rect.width(),
        image_card.response.rect.height()
    ));

    ui.add_space(16.0);
    ui.label("6. Image + footer (N1 §3.2) — nova: top 0 / bottom 0; base: 24 / 24");
    ui.add_space(4.0);
    let image_footer_card = Card::new().image_top(card_demo_texture(ui)).show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Landscape with footer").show(ui);
            CardDescription::new("Image above, footer band below.").show(ui);
        });
        CardContent::new().show(ui, |ui| {
            ui.label("The footer band must bleed to the card's bottom inner edge.");
        });
        CardFooter::new().show(ui, |ui| {
            Button::new("Save").show(ui);
            Button::new("Discard")
                .variant(ButtonVariant::Outline)
                .show(ui);
        });
    });
    ui.label(format!(
        "measured: {:.1} x {:.1} px",
        image_footer_card.response.rect.width(),
        image_footer_card.response.rect.height()
    ));

    ui.add_space(16.0);
    ui.label("7. Long title with an action (N2) — the title wraps within the 1fr column (CRD-01); the action stays top-aligned");
    ui.add_space(4.0);
    let avail = ui.available_rect_before_wrap();
    let narrow = ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_size(
                avail.min,
                egui::vec2(360.0, avail.height()),
            ))
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            let mut title_rect = egui::Rect::NOTHING;
            Card::new().show(ui, |ui| {
                CardHeader::new().show(ui, |ui| {
                    title_rect = CardTitle::new(
                        "A very long card title that must wrap across several lines inside the header column while the action stays pinned to the top-right corner",
                    )
                    .show(ui)
                    .rect;
                    CardDescription::new("The action spans both header rows.").show(ui);
                    CardAction::new().show(ui, |ui| {
                        Button::new("")
                            .size(ButtonSize::IconSm)
                            .variant(ButtonVariant::Outline)
                            .icon_start(Icon::MoreHorizontal)
                            .show(ui);
                    });
                });
                CardContent::new().show(ui, |ui| {
                    ui.label("Narrow 360px card, so the wrap is visible at a glance.");
                });
            });
            title_rect
        },
    );
    let title_rect = narrow.inner;
    let lines = (title_rect.height() / title_lh).round().max(1.0);
    ui.label(format!(
        "measured: card {:.1} x {:.1} px; title {:.1} x {:.1} px ({:.0} lines at {:.0}px line-height)",
        narrow.response.rect.width(),
        narrow.response.rect.height(),
        title_rect.width(),
        title_rect.height(),
        lines,
        title_lh
    ));
}

/// A generated gradient texture for the card demo's image slot (cached in
/// temp storage like the avatar demo's). Landscape 4:1, so the image
/// renders ~255px tall at full card width and the whole demo page still
/// fits one 1200x3300 capture; the image's intrinsic aspect is its own
/// (spec §3.5 — height is the image's intrinsic height at that width).
fn card_demo_texture(ui: &mut egui::Ui) -> egui::ImageSource<'static> {
    let tex_store = egui::Id::unique("card_demo_texture");
    let tex_id = ui
        .ctx()
        .data_mut(|d| d.get_temp::<egui::TextureHandle>(tex_store));
    let texture = tex_id.unwrap_or_else(|| {
        let (w, h) = (256, 64);
        let mut pixels = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let t = (x as f32 / w as f32) * 0.6 + (y as f32 / h as f32) * 0.4;
                let r = (40.0 + 160.0 * t) as u8;
                let g = (70.0 + 60.0 * (1.0 - t)) as u8;
                let b = (180.0 + 60.0 * t) as u8;
                pixels.push(egui::Color32::from_rgb(r, g, b));
            }
        }
        let image = egui::ColorImage::new([w, h], pixels);
        ui.ctx()
            .load_texture("card_demo_gradient", image, egui::TextureOptions::default())
    });
    egui::ImageSource::Texture(egui::load::SizedTexture::new(
        texture.id(),
        texture.size_vec2(),
    ))
}

pub fn alert_demo(ui: &mut egui::Ui) {
    // Every section prints the rect the alert actually allocated
    // (`Response.rect`), so S5/S6 can verify geometry from screenshots.
    // Header: recipe position, stated honestly (spec §§0, 3.4, 10, 11).
    let theme = ShadcnTheme::of_ui(ui);
    let nova = theme.style == Style::Nova;
    let recipe = if nova { "nova" } else { "base" };
    let mode = if theme.mode.is_dark() {
        "dark"
    } else {
        "light"
    };
    ui.label(format!(
        "Alert — callout for user attention. Active recipe: {recipe} (style: {}, mode: {mode}). Title 14px/500, 20px line-height; description 14px, 22.75px line-height; padding 10/8 nova, 16/12 base; radius 10px.",
        theme.style.name()
    ));
    ui.label("D1: role=\"alert\" live-region announcement is NOT conveyed (no ARIA tree in egui). D2: tracking-tight (-0.35px/char) is NOT conveyed (no letter-spacing in egui). D4 (decided, S8 §3.2): the action slot is a nova-only source rule; the port renders it in all styles and reserves 72px in all styles — a documented port extension (uniform action placement). D7: the 72px action reservation applies from frame 2 (one-frame settle). No hover/focus/disabled states, no animation (spec §4/§6).");

    // Right padding implied by the laid-out geometry: outer width minus the
    // text column, left padding, icon column + gap, and the 2px border.
    // (The text column width is captured inside the alert closure.)
    let right_clearance = |alert_w: f32, text_w: f32, has_icon: bool| -> f32 {
        let (px, gap) = if nova { (10.0, 8.0) } else { (16.0, 12.0) };
        let (icon_w, icon_gap) = if has_icon { (16.0, gap) } else { (0.0, 0.0) };
        alert_w - text_w - px - icon_w - icon_gap - 2.0
    };
    let expected_right = |has_action: bool| -> f32 {
        if nova && has_action {
            72.0
        } else if nova {
            10.0
        } else {
            16.0
        }
    };
    let long_title = "This alert title is far too long to fit on a single line and must be truncated with a trailing ellipsis, so it keeps going well past the edge of the text column in every layout the gallery can show";

    // 1. Default + destructive (spec §2.2, §3.1, §3.3, §3.5).
    ui.add_space(12.0);
    ui.label("1. Default + destructive — title 20px line-height, description 22.75px (spec §3.3/§3.5); destructive changes text only, fill stays bg-card");
    ui.add_space(4.0);
    let plain = Alert::new().id_salt("alert_demo_plain").show(ui, |ui| {
        let text_w = ui.available_width();
        let t = AlertTitle::new("Heads up!").show(ui).rect;
        let d = AlertDescription::new(
            "You can add components and dependencies to your app using the cli.",
        )
        .show(ui)
        .rect;
        (text_w, t, d)
    });
    let (text_w, t_rect, d_rect) = plain.inner;
    ui.label(format!(
        "measured: default title {:.1} x {:.1} px (expected h=20.0); description {:.1} x {:.1} px (expected h=22.75, one line); right clearance {:.1} px (expected {:.1})",
        t_rect.width(),
        t_rect.height(),
        d_rect.width(),
        d_rect.height(),
        right_clearance(plain.response.rect.width(), text_w, false),
        expected_right(false)
    ));
    ui.add_space(4.0);
    let dest = Alert::new()
        .id_salt("alert_demo_destructive")
        .variant(AlertVariant::Destructive)
        .icon(Icon::CircleAlert)
        .show(ui, |ui| {
            let text_w = ui.available_width();
            let t = AlertTitle::new("Deployment failed").show(ui).rect;
            let d = AlertDescription::new(
                "Your build could not be completed. Check the logs and try again.",
            )
            .show(ui)
            .rect;
            (text_w, t, d)
        });
    let (text_w, t_rect, d_rect) = dest.inner;
    ui.label(format!(
        "measured: destructive title {:.1} x {:.1} px (expected h=20.0); description {:.1} x {:.1} px (expected h=22.75, one line); right clearance {:.1} px (expected {:.1})",
        t_rect.width(),
        t_rect.height(),
        d_rect.width(),
        d_rect.height(),
        right_clearance(dest.response.rect.width(), text_w, true),
        expected_right(false)
    ));

    // Success and info icons (spec §4.2): the default variant keeps icons
    // foreground-coloured (`[&>svg]:text-current`); do not recolor them per
    // semantic.
    ui.add_space(12.0);
    ui.label("Success and info icons — default variant, icons keep text-current (spec §4.2)");
    ui.add_space(4.0);
    Alert::new()
        .id_salt("alert_demo_success")
        .icon(Icon::CircleCheck)
        .show(ui, |ui| {
            AlertTitle::new("Payment received").show(ui);
            AlertDescription::new("Your invoice #1043 was paid in full.").show(ui);
        });
    ui.add_space(4.0);
    Alert::new()
        .id_salt("alert_demo_info")
        .icon(Icon::Info)
        .show(ui, |ui| {
            AlertTitle::new("Scheduled maintenance").show(ui);
            AlertDescription::new("The API will be unavailable Sunday 02:00–04:00 UTC.").show(ui);
        });

    // 2. With an action — 72px TOTAL reservation in all styles
    // (spec §3.4; S8 §3.2 D4: uniform action placement).
    ui.add_space(12.0);
    ui.label("2. With an action — right clearance 72px total in all styles (spec §3.4; S8 §3.2 D4: uniform action placement)");
    ui.add_space(4.0);
    let action = Alert::new()
        .id_salt("alert_demo_action")
        .icon(Icon::CircleAlert)
        .show(ui, |ui| {
            let text_w = ui.available_width();
            let t = AlertTitle::new("Token expiring").show(ui).rect;
            let d = AlertDescription::new("Your API token expires in 3 days.")
                .show(ui)
                .rect;
            AlertAction::new().show(ui, |ui| {
                Button::new("Renew").size(ButtonSize::Sm).show(ui);
            });
            (text_w, t, d)
        });
    let (text_w, t_rect, d_rect) = action.inner;
    ui.label(format!(
        "measured: action title {:.1} x {:.1} px (expected h=20.0); description {:.1} x {:.1} px (expected h=22.75); right clearance {:.1} px (expected {:.1})",
        t_rect.width(),
        t_rect.height(),
        d_rect.width(),
        d_rect.height(),
        right_clearance(action.response.rect.width(), text_w, true),
        expected_right(true)
    ));
    ui.add_space(4.0);
    let action_dest = Alert::new()
        .id_salt("alert_demo_action_destructive")
        .variant(AlertVariant::Destructive)
        .icon(Icon::CircleX)
        .show(ui, |ui| {
            let text_w = ui.available_width();
            let t = AlertTitle::new("Storage quota exceeded").show(ui).rect;
            let d = AlertDescription::new("Delete some files to keep syncing.")
                .show(ui)
                .rect;
            AlertAction::new().show(ui, |ui| {
                Button::new("Free up space")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Destructive)
                    .show(ui);
            });
            (text_w, t, d)
        });
    let (text_w, t_rect, d_rect) = action_dest.inner;
    ui.label(format!(
        "measured: destructive action title {:.1} x {:.1} px; description {:.1} x {:.1} px; right clearance {:.1} px (expected {:.1})",
        t_rect.width(),
        t_rect.height(),
        d_rect.width(),
        d_rect.height(),
        right_clearance(action_dest.response.rect.width(), text_w, true),
        expected_right(true)
    ));

    // 3. Long title (N4) — one line, trailing "…" (spec §3.3, §9.1/§9.2).
    ui.add_space(12.0);
    ui.label("3. Long title (N4) — must truncate to one line with a trailing \"…\", never wrap or overflow (spec §3.3)");
    ui.add_space(4.0);
    let lt = Alert::new()
        .id_salt("alert_demo_long_title")
        .icon(Icon::CircleAlert)
        .show(ui, |ui| {
            let text_w = ui.available_width();
            let t = AlertTitle::new(long_title).show(ui).rect;
            let d = AlertDescription::new(
                "The description below the truncated title still wraps normally.",
            )
            .show(ui)
            .rect;
            (text_w, t, d)
        });
    let (text_w, t_rect, d_rect) = lt.inner;
    ui.label(format!(
        "measured: long title {:.1} x {:.1} px in a {:.1} px text column (expected: one line, h=20.0, w = text column, ends with \"…\"); description {:.1} px tall",
        t_rect.width(),
        t_rect.height(),
        text_w,
        d_rect.height()
    ));
    ui.add_space(4.0);
    let lt_action = Alert::new()
        .id_salt("alert_demo_long_title_action")
        .variant(AlertVariant::Destructive)
        .icon(Icon::CircleX)
        .show(ui, |ui| {
            let text_w = ui.available_width();
            let t = AlertTitle::new(long_title).show(ui).rect;
            AlertDescription::new("Long title with an action: the title truncates before the action; the 72px reservation keeps the text clear of it.")
                .show(ui);
            // Narrow action ("Undo", ~60px): it fits inside the 72px
            // reservation, so the elided title stays clear of it. A wide
            // action would overlap the text column instead (spec §9.12 —
            // same as the browser); the §2 "Free up space" case shows that.
            let btn_rect = AlertAction::new()
                .show(ui, |ui| {
                    Button::new("Undo")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .rect
                })
                .inner;
            (text_w, t, btn_rect)
        });
    let (text_w, t_rect, btn_rect) = lt_action.inner;
    // In nova the 72px reservation keeps the elided title clear of the
    // action; in base (D4) there is no reservation, so the title runs
    // under the button — the readout says so instead of asserting >= 0.
    let action_note = if nova {
        "expected >= 0 — text clears the action"
    } else {
        "D4: action is nova-only in the source; base has no 72px reservation, so the title runs under the button"
    };
    ui.label(format!(
        "measured: long title + action {:.1} x {:.1} px in a {:.1} px text column; right clearance {:.1} px (expected {:.1}); title right -> button left {:.1} px ({})",
        t_rect.width(),
        t_rect.height(),
        text_w,
        right_clearance(lt_action.response.rect.width(), text_w, true),
        expected_right(true),
        btn_rect.left() - t_rect.right(),
        action_note
    ));

    // 4. Multi-paragraph description — 20px gap nova / 4px base (spec §2.6).
    ui.add_space(12.0);
    ui.label("4. Multi-paragraph description — gap 20px nova / 4px base (spec §2.6: 16px mb-4 + 4px grid gap)");
    ui.add_space(4.0);
    let multi = Alert::new()
        .id_salt("alert_demo_multi_para")
        .icon(Icon::Info)
        .show(ui, |ui| {
            let t = AlertTitle::new("Two paragraphs").show(ui).rect;
            let d = AlertDescription::new(
                "First paragraph of the description, kept to one line.\n\nSecond paragraph of the description, also one line.",
            )
            .show(ui)
            .rect;
            (t, d)
        });
    let (t_rect, d_rect) = multi.inner;
    let gap = if nova { 20.0 } else { 4.0 };
    ui.label(format!(
        "measured: two-paragraph description {:.1} px tall; derived gap {:.1} px (expected {:.1}; +0.5px is egui rounding each 22.75px line up to 23.0); title {:.1} px tall",
        d_rect.height(),
        d_rect.height() - 45.5,
        gap,
        t_rect.height()
    ));

    // 5. Narrow alert — no minimum width invented (spec §9.10).
    ui.add_space(12.0);
    ui.label("5. Narrow alert (360px) — title still truncates to one line; the 72px action clearance still holds (spec §9.10)");
    ui.add_space(4.0);
    let (narrow_text_w, narrow_t, narrow_alert_w, narrow_btn) = ui
        .allocate_ui(egui::Vec2::new(360.0, 0.0), |ui| {
            let resp = Alert::new()
                .id_salt("alert_demo_narrow")
                .icon(Icon::CircleAlert)
                .show(ui, |ui| {
                    let text_w = ui.available_width();
                    let t = AlertTitle::new(long_title).show(ui).rect;
                    AlertDescription::new("Narrow description wraps to the available width.")
                        .show(ui);
                    let btn_rect = AlertAction::new()
                        .show(ui, |ui| {
                            Button::new("Renew").size(ButtonSize::Sm).show(ui).rect
                        })
                        .inner;
                    (text_w, t, btn_rect)
                });
            let (text_w, t, btn_rect) = resp.inner;
            (text_w, t, resp.response.rect.width(), btn_rect)
        })
        .inner;
    ui.label(format!(
        "measured: narrow alert {:.1} px wide; long title {:.1} x {:.1} px in a {:.1} px text column; right clearance {:.1} px (expected {:.1}); title right -> button left {:.1} px ({})",
        narrow_alert_w,
        narrow_t.width(),
        narrow_t.height(),
        narrow_text_w,
        right_clearance(narrow_alert_w, narrow_text_w, true),
        expected_right(true),
        narrow_btn.left() - narrow_t.right(),
        action_note
    ));
}

pub fn avatar_demo(ui: &mut egui::Ui) {
    // A generated gradient texture so the demo shows a real image without
    // needing file loaders.
    let tex_store = egui::Id::unique("avatar_demo_texture");
    let tex_id = ui
        .ctx()
        .data_mut(|d| d.get_temp::<egui::TextureHandle>(tex_store));
    let texture = tex_id.unwrap_or_else(|| {
        let size = 64;
        let mut pixels = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let t = (x as f32 / size as f32 + y as f32 / size as f32) * 0.5;
                let r = (60.0 + 140.0 * t) as u8;
                let g = (90.0 + 80.0 * (1.0 - t)) as u8;
                let b = (220.0 - 120.0 * t) as u8;
                pixels.push(egui::Color32::from_rgb(r, g, b));
            }
        }
        let image = egui::ColorImage::new([size, size], pixels);
        ui.ctx().load_texture(
            "avatar_demo_gradient",
            image,
            egui::TextureOptions::default(),
        )
    });
    let src = egui::ImageSource::Texture(egui::load::SizedTexture::new(
        texture.id(),
        texture.size_vec2(),
    ));

    // A wide (128x64) texture with bright stripes at its left/right edges.
    // D2 distinguisher: the source stretches a non-square image to fill the
    // square (the stripes survive near the circle's left/right edges); the
    // port centre-crops (cover_uv), which cuts the stripes off. S5 judges
    // which renders.
    let wide_store = egui::Id::unique("avatar_demo_texture_wide");
    let wide_tex_id = ui
        .ctx()
        .data_mut(|d| d.get_temp::<egui::TextureHandle>(wide_store));
    let wide_texture = wide_tex_id.unwrap_or_else(|| {
        let (w, h) = (128, 64);
        let mut pixels = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let px = if x < 6 || x >= w - 6 {
                    egui::Color32::WHITE
                } else {
                    let t = x as f32 / w as f32;
                    let v = y as f32 / h as f32;
                    egui::Color32::from_rgb(
                        (40.0 + 160.0 * t) as u8,
                        (100.0 + 80.0 * v) as u8,
                        (220.0 - 120.0 * t) as u8,
                    )
                };
                pixels.push(px);
            }
        }
        let image = egui::ColorImage::new([w, h], pixels);
        ui.ctx().load_texture(
            "avatar_demo_gradient_wide",
            image,
            egui::TextureOptions::default(),
        )
    });
    let wide_src = egui::ImageSource::Texture(egui::load::SizedTexture::new(
        wide_texture.id(),
        wide_texture.size_vec2(),
    ));

    ui.small("Avatar is a non-interactive display element: no keys, no Tab stop, clicks do nothing. Nova == base for every slot (spec §2.2). D2: the image centre-crop is a proposed deviation — the source stretches a non-square image to fill the square; judge it on the wide image in §2b (S5 judged centre-crop; S8 §3.3 accepted it). D3: Radix loading/delay states are degraded to texture-or-fallback (end-states only) — S6 fetched the Radix source and settled D3 as a documented equivalence; the transition timing stays OPEN-1. OPEN-2 CLOSED (S8 §3.4): a mixed-size group's count follows the hint-wins rule — the group's .size() hint when set, else 32px (S5/S6-verified: 24 hint-Sm / 32 no-hint).");

    ui.add_space(12.0);
    ui.label("§1 Sizes with fallback initials");
    ui.add_space(4.0);
    let sizes = ui
        .horizontal(|ui| {
            (
                Avatar::new().size(AvatarSize::Sm).fallback("CN").show(ui),
                Avatar::new().fallback("CN").show(ui),
                Avatar::new().size(AvatarSize::Lg).fallback("CN").show(ui),
            )
        })
        .inner;
    ui.small(format!(
        "measured: sm {:.1} x {:.1} px (expected 24), default {:.1} x {:.1} px (expected 32), lg {:.1} x {:.1} px (expected 40); fallback text 12px sm / 14px default+lg",
        sizes.0.rect.width(),
        sizes.0.rect.height(),
        sizes.1.rect.width(),
        sizes.1.rect.height(),
        sizes.2.rect.width(),
        sizes.2.rect.height(),
    ));

    ui.add_space(12.0);
    ui.label("§2 Image, icon fallback and badges");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Avatar::new().image(src.clone()).show(ui);
        Avatar::new()
            .size(AvatarSize::Lg)
            .image(src.clone())
            .show(ui);
        Avatar::new()
            .fallback_icon(Icon::User)
            .badge(AvatarBadge::new())
            .show(ui);
        Avatar::new()
            .fallback("OK")
            .badge(AvatarBadge::new().icon(Icon::Check))
            .show(ui);
        Avatar::new()
            .size(AvatarSize::Lg)
            .fallback_icon(Icon::User)
            .badge(AvatarBadge::new().icon(Icon::Plus))
            .show(ui);
    });
    ui.small("expected: badge 10px with 8px icon (default), 12px with 8px icon (lg), ring-2 ring-background, bottom-right corner; S5-verified in all 4 configs (the ring disc extends ~2px past the avatar rect — a CSS ring-2 outside the badge box, by design).");

    ui.add_space(12.0);
    ui.label("§2b Non-square image (D2) — wide 128x64 texture");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Avatar::new().image(wide_src.clone()).show(ui);
        Avatar::new()
            .size(AvatarSize::Lg)
            .image(wide_src.clone())
            .show(ui);
    });
    ui.small("D2: the white edge stripes survive a stretch near the circle's left/right edges and are cut off by a centre crop — S5 judged centre-crop (t 0.24–0.76, zero white pixels); S8 §3.3 accepted the crop.");

    ui.add_space(12.0);
    ui.label("§3 Groups — count keys off MEMBER sizes (N5)");
    ui.add_space(4.0);
    ui.label("All-default group (count 32)");
    let default_group = AvatarGroup::new().show(ui, |ui| {
        let a = Avatar::new().image(src.clone()).show(ui);
        let b = Avatar::new().fallback("AB").show(ui);
        Avatar::new().fallback("CD").show(ui);
        Avatar::new().fallback_icon(Icon::User).show(ui);
        let c = AvatarGroupCount::new("+2").show(ui);
        (a, b, c)
    });
    let (first, second, default_count) = default_group.inner;
    ui.small(format!(
        "measured: member pitch {:.1} px (expected 24 = 32 - 8 overlap), count {:.1} x {:.1} px (expected 32)",
        second.rect.left() - first.rect.left(),
        default_count.rect.width(),
        default_count.rect.height(),
    ));

    ui.add_space(8.0);
    ui.label("All-sm group (count 24) — members carry .size(Sm)");
    let sm_group = AvatarGroup::new().show(ui, |ui| {
        let a = Avatar::new().size(AvatarSize::Sm).fallback("EF").show(ui);
        let b = Avatar::new().size(AvatarSize::Sm).fallback("GH").show(ui);
        let c = AvatarGroupCount::new("+9").show(ui);
        (a, b, c)
    });
    let (first, second, sm_count) = sm_group.inner;
    ui.small(format!(
        "measured: member pitch {:.1} px (expected 16 = 24 - 8 overlap), count {:.1} x {:.1} px (expected 24)",
        second.rect.left() - first.rect.left(),
        sm_count.rect.width(),
        sm_count.rect.height(),
    ));

    ui.add_space(8.0);
    ui.label("All-lg group with icon count (count 40) — members carry .size(Lg)");
    let lg_group = AvatarGroup::new().show(ui, |ui| {
        let a = Avatar::new()
            .size(AvatarSize::Lg)
            .image(src.clone())
            .show(ui);
        let b = Avatar::new().size(AvatarSize::Lg).fallback("IJ").show(ui);
        let c = AvatarGroupCount::new("")
            .icon(Icon::MoreHorizontal)
            .show(ui);
        (a, b, c)
    });
    let (first, second, lg_count) = lg_group.inner;
    ui.small(format!(
        "measured: member pitch {:.1} px (expected 32 = 40 - 8 overlap), count {:.1} x {:.1} px (expected 40)",
        second.rect.left() - first.rect.left(),
        lg_count.rect.width(),
        lg_count.rect.height(),
    ));

    ui.add_space(12.0);
    ui.label("§4 Mixed-size group (hint-wins — OPEN-2 closed, S8 §3.4)");
    ui.add_space(4.0);
    ui.label("Mixed sm + lg members, group hint Sm");
    let mixed_hint = AvatarGroup::new().size(AvatarSize::Sm).show(ui, |ui| {
        Avatar::new().size(AvatarSize::Sm).fallback("KL").show(ui);
        Avatar::new().size(AvatarSize::Lg).fallback("MN").show(ui);
        AvatarGroupCount::new("+3").show(ui)
    });
    let mixed_hint_count = mixed_hint.inner;
    ui.small(format!(
        "measured: count {:.1} x {:.1} px (expected 24 — the Sm hint wins the tiebreak; hint-wins, OPEN-2 closed S8 §3.4, S5/S6-verified)",
        mixed_hint_count.rect.width(),
        mixed_hint_count.rect.height(),
    ));

    ui.add_space(8.0);
    ui.label("Mixed sm + lg members, no group hint");
    let mixed_plain = AvatarGroup::new().show(ui, |ui| {
        Avatar::new().size(AvatarSize::Sm).fallback("OP").show(ui);
        Avatar::new().size(AvatarSize::Lg).fallback("QR").show(ui);
        AvatarGroupCount::new("+4").show(ui)
    });
    let mixed_plain_count = mixed_plain.inner;
    ui.small(format!(
        "measured: count {:.1} x {:.1} px (expected 32 — base size when no hint is set; hint-wins, OPEN-2 closed S8 §3.4, S5/S6-verified)",
        mixed_plain_count.rect.width(),
        mixed_plain_count.rect.height(),
    ));

    ui.add_space(12.0);
    ui.label("§5 Badges — 8 / 10 / 12 by member size, bottom-right, sm icon hidden");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Avatar::new()
            .size(AvatarSize::Sm)
            .fallback("S")
            .badge(AvatarBadge::new().icon(Icon::Check))
            .show(ui);
        Avatar::new()
            .size(AvatarSize::Sm)
            .fallback("M")
            .badge(AvatarBadge::new())
            .show(ui);
        Avatar::new()
            .fallback("OK")
            .badge(AvatarBadge::new().icon(Icon::Check))
            .show(ui);
        Avatar::new()
            .size(AvatarSize::Lg)
            .fallback("LG")
            .badge(AvatarBadge::new().icon(Icon::Plus))
            .show(ui);
    });
    ui.small("expected: badge 8px (sm, icon hidden) / 10px (default, icon 8px) / 12px (lg, icon 8px), ring-2 ring-background; S5-verified in all 4 configs.");

    ui.add_space(8.0);
    ui.small("URI images load through the context's texture loaders and fall back to initials until resolved. Long fallback text is centre-clipped with no ellipsis (spec §3.3) — not exercisable here: .fallback() reduces names to at most 2 initials, so the clip path is unit-pinned in S1 instead.");
}

pub fn progress_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("progress_demo_value");
    let mut value: f32 = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or(30.0));

    ui.label("Progress — determinate value bar (Radix Progress). Non-interactive: no keys, no Tab stop, no hover/focus/disabled/invalid recipe (spec §7). Track 8px base / 4px nova; track fill primary @ 20% base, muted nova; indicator primary. N6: square cut at partial fill, pill only at 100%. Clamp is port-side (D3, APPROVED defensive deviation, S8 §1). Fill animates 150ms cubic-bezier(0.4, 0, 0.2, 1) (PRG-01, S10-verified). No label/value slots (OPEN-3); no indeterminate state (falsy → empty bar); max has no visual effect.");
    ui.add_space(8.0);

    ui.label("1. Interactive — the fill animates over 150ms (shadcn transition-all; cubic-bezier(0.4, 0, 0.2, 1), PRG-01)");
    ui.add_space(4.0);
    let resp = Progress::new(value).show(ui);
    ui.small(format!(
        "value {:.0}% — track: {:.1} x {:.1} px (expected h = 8 base / 4 nova); settled fill {:.1} px",
        value,
        resp.rect.width(),
        resp.rect.height(),
        value.clamp(0.0, 100.0) / 100.0 * resp.rect.width()
    ));

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if Button::new("-10").size(ButtonSize::Sm).show(ui).clicked() {
            value = (value - 10.0).max(0.0);
        }
        if Button::new("+10").size(ButtonSize::Sm).show(ui).clicked() {
            value = (value + 10.0).min(100.0);
        }
        if Button::new("Reset")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            value = 30.0;
        }
    });

    ui.add_space(12.0);
    ui.label("2. Values — measured track and settled fill width");
    ui.add_space(4.0);
    for v in [0.0, 25.0, 50.0, 75.0, 100.0] {
        // NOTE: no inline "%" label in the bar's row — a trailing label
        // after a full-width bar overflows the row, widens the scroll
        // content, and makes each successive track wider than the last
        // (measured 1053.8 → 1183.5px in the first smoke run, clipping the
        // 100% bar at the window edge). The value is in the readout below.
        let r = Progress::new(v)
            .id_salt(format!("s3-value-{v:.0}"))
            .show(ui);
        ui.small(format!(
            "{:.0}% — track {:.1} x {:.1} px; settled fill {:.1} px",
            v,
            r.rect.width(),
            r.rect.height(),
            v / 100.0 * r.rect.width()
        ));
    }

    ui.add_space(12.0);
    ui.label(
        "3. N6 — square cut at partial fill vs pill at 100% (S5: judge the right edge rendered)",
    );
    ui.add_space(4.0);
    ui.small("50% — square right edge, rounded left edge:");
    let half = Progress::new(50.0)
        .width(400.0)
        .id_salt("s3-n6-half")
        .show(ui);
    ui.small(format!(
        "track {:.1} x {:.1} px; settled fill {:.1} px; right edge square (N6)",
        half.rect.width(),
        half.rect.height(),
        0.5 * half.rect.width()
    ));
    ui.add_space(8.0);
    ui.small("100% — pill, both ends rounded (track rounded-full):");
    let full = Progress::new(100.0)
        .width(400.0)
        .id_salt("s3-n6-full")
        .show(ui);
    ui.small(format!(
        "track {:.1} x {:.1} px; settled fill {:.1} px; right edge pill",
        full.rect.width(),
        full.rect.height(),
        full.rect.width()
    ));

    ui.add_space(12.0);
    ui.label("4. Clamped (D3, port-side) and fixed widths");
    ui.add_space(4.0);
    let over = Progress::new(250.0)
        .width(200.0)
        .id_salt("s3-clamp-over")
        .show(ui);
    ui.small(format!(
        "250.0 — rendered fill: full ({:.1} px of {:.1} px track; clamped to 100, D3)",
        over.rect.width(),
        over.rect.width()
    ));
    ui.add_space(4.0);
    let under = Progress::new(-5.0)
        .width(200.0)
        .id_salt("s3-clamp-under")
        .show(ui);
    ui.small("-5.0 — rendered fill: empty (clamped to 0, D3)");
    let _ = under;
    ui.add_space(4.0);
    let narrow = Progress::new(40.0)
        .width(120.0)
        .id_salt("s3-fixed-120")
        .show(ui);
    ui.small(format!(
        "40% of 120.0 px — settled fill {:.1} px",
        0.4 * narrow.rect.width()
    ));

    ui.ctx().data_mut(|d| d.insert_temp(store, value));
}

pub fn table_demo(ui: &mut egui::Ui) {
    ui.label("Table primitive — header / body / footer, rows, cells, caption. This is the shadcn Table primitive, NOT data-table (Batch G): no sorting, pagination, or filtering here.");
    ui.label("Honest notes: expanded rows render bg-muted/50; the trigger placement is settled (S8 §3.6) — aria-expanded sits on a descendant expander control, never on the <tr> (TanStack RowExpanding.ts 8.21.3), and expanded(bool) is the state carrier (OPEN-5 CLOSED, Batch G owns the expander control + rendering); checkbox cells use the additive .checkbox_cell builders (N8); row recolour is instant, not the 150ms transition-colors fade (limitation D2, easing OPEN-4); the caption's mt-4 is caller-spaced here because the TableCaption slot is Batch A's (D3).");
    ui.add_space(8.0);

    ui.label("§1 Invoices table (hover rows, one selected row, footer totals)");
    ui.add_space(4.0);
    let mut head_h = 0.0;
    let mut body_row_h = 0.0;
    Table::new().show(ui, |ui| {
        TableHeader::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                let head_resp = TableHead::new("Invoice").width(220.0).show(ui);
                head_h = head_resp.rect.height();
                TableHead::new("Status").width(110.0).show(ui);
                TableHead::new("Method").width(130.0).show(ui);
                TableHead::new("Amount").width(90.0).show(ui);
            });
        });
        TableBody::new().show(ui, |ui| {
            let first_row = TableRow::new().show(ui, |ui| {
                TableCell::text("INV-0041").width(220.0).show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Paid").show(ui);
                });
                TableCell::text("Credit Card").width(130.0).show_text(ui);
                TableCell::text("$250.00").width(90.0).show_text(ui);
            });
            body_row_h = first_row.response.rect.height();
            TableRow::new().show(ui, |ui| {
                TableCell::text("INV-0042").width(220.0).show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Pending")
                        .variant(BadgeVariant::Secondary)
                        .show(ui);
                });
                TableCell::text("PayPal").width(130.0).show_text(ui);
                TableCell::text("$150.00").width(90.0).show_text(ui);
            });
            TableRow::new().selected(true).show(ui, |ui| {
                TableCell::text("INV-0043").width(220.0).show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Unpaid").variant(BadgeVariant::Outline).show(ui);
                });
                TableCell::text("Bank Transfer").width(130.0).show_text(ui);
                TableCell::text("$300.00").width(90.0).show_text(ui);
            });
            TableRow::new().show(ui, |ui| {
                TableCell::text("INV-0044").width(220.0).show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Paid").show(ui);
                });
                TableCell::text("Credit Card").width(130.0).show_text(ui);
                TableCell::text("$125.00").width(90.0).show_text(ui);
            });
        });
        TableFooter::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                TableCell::text("Total").width(220.0).show_text(ui);
                TableCell::text("").width(110.0).show_text(ui);
                TableCell::text("").width(130.0).show_text(ui);
                TableCell::text("$825.00").width(90.0).show_text(ui);
            });
        });
    });
    // The source caption is `mt-4` (16px above); TableCaption itself uses
    // the default item spacing, so the demo adds the space explicitly.
    ui.add_space(16.0);
    TableCaption::new("A list of your recent invoices.").show(ui);
    ui.small(format!(
        "measured: head height {head_h:.1} px (h-10 = 40); first body row height {body_row_h:.1} px (tallest cell is the Status badge — see §3 for the plain-text row height); caption gap 16.0 px (mt-4, caller-spaced per D3)"
    ));

    ui.add_space(12.0);
    ui.label("§2 Checkbox cells (N8) — select-all header checkbox + row selection checkboxes");
    ui.add_space(4.0);
    let cb_store = egui::Id::unique("table_demo_checkbox_rows");
    let mut cb_rows: [bool; 3] = ui
        .ctx()
        .data_mut(|d| d.get_temp(cb_store).unwrap_or([false; 3]));
    let mut cb_all = cb_rows.iter().all(|r| *r);
    let cb_all_before = cb_all;
    let mut cb_head_rect = egui::Rect::ZERO;
    let mut cb_body_rect = egui::Rect::ZERO;
    let mut cb_body_row_h = 0.0;
    Table::new().show(ui, |ui| {
        TableHeader::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                let head = TableHead::new("")
                    .width(40.0)
                    .checkbox_cell(true)
                    .show_with(ui, |ui| {
                        Checkbox::new(&mut cb_all)
                            .id_salt("table_demo_select_all")
                            .show(ui)
                    });
                cb_head_rect = head.response.rect;
                TableHead::new("Name").width(220.0).show(ui);
                TableHead::new("Role").width(140.0).show(ui);
            });
        });
        TableBody::new().show(ui, |ui| {
            for (i, (name, role)) in [
                ("Ada Lovelace", "Engineer"),
                ("Grace Hopper", "Admiral"),
                ("Katherine Johnson", "Mathematician"),
            ]
            .into_iter()
            .enumerate()
            {
                let selected_row = cb_rows[i];
                let cb_row = TableRow::new().selected(selected_row).show(ui, |ui| {
                    let cell = TableCell::new()
                        .width(40.0)
                        .checkbox_cell(true)
                        .show(ui, |ui| {
                            Checkbox::new(&mut cb_rows[i])
                                .id_salt(format!("table_demo_cb_{i}"))
                                .show(ui)
                        });
                    if i == 0 {
                        cb_body_rect = cell.response.rect;
                    }
                    TableCell::text(name).width(220.0).show_text(ui);
                    TableCell::text(role).width(140.0).show_text(ui);
                });
                if i == 0 {
                    cb_body_row_h = cb_row.response.rect.height();
                }
            }
        });
    });
    if cb_all != cb_all_before {
        cb_rows = [cb_all; 3];
    }
    ui.ctx().data_mut(|d| d.insert_temp(cb_store, cb_rows));
    ui.small(format!(
        "measured: checkbox head cell {:.1} x {:.1} px (h-10 = 40, pr-0); checkbox body cell {:.1} x {:.1} px; checkbox body row height {:.1} px (the checkbox does not grow the row); checkbox content shifts 2px down (translate-y-[2px])",
        cb_head_rect.width(),
        cb_head_rect.height(),
        cb_body_rect.width(),
        cb_body_rect.height(),
        cb_body_row_h
    ));

    ui.add_space(12.0);
    ui.label("§3 Expanded row (N7) — rendering case only; trigger placement settled (S8 §3.6)");
    ui.add_space(4.0);
    let mut plain_row_h = 0.0;
    Table::new().show(ui, |ui| {
        TableHeader::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                TableHead::new("Invoice").width(220.0).show(ui);
                TableHead::new("Status").width(110.0).show(ui);
            });
        });
        TableBody::new().show(ui, |ui| {
            let plain_row = TableRow::new().show(ui, |ui| {
                TableCell::text("INV-0051").width(220.0).show_text(ui);
                TableCell::text("Paid").width(110.0).show_text(ui);
            });
            plain_row_h = plain_row.response.rect.height();
            TableRow::new().expanded(true).show(ui, |ui| {
                TableCell::text("INV-0052 (expanded)")
                    .width(220.0)
                    .show_text(ui);
                TableCell::text("Pending").width(110.0).show_text(ui);
            });
            TableRow::new().selected(true).show(ui, |ui| {
                TableCell::text("INV-0053 (selected)")
                    .width(220.0)
                    .show_text(ui);
                TableCell::text("Unpaid").width(110.0).show_text(ui);
            });
        });
    });
    ui.small(format!(
        "measured: plain-text body row height {plain_row_h:.1} px (14px text + 8px padding top/bottom). Expanded row fill: bg-muted/50 (same value as hover); selected wins over expanded (opaque bg-muted). No expander control is shown — the expanded state is consumer-owned (S8 §3.6: aria-expanded on a descendant control; expanded(bool) is the state carrier)."
    ));

    ui.add_space(12.0);
    ui.label("§4 Selection state demo (click rows)");
    ui.add_space(4.0);
    let sel_store = egui::Id::unique("table_demo_selected");
    let mut selected: Option<usize> = ui.ctx().data_mut(|d| d.get_temp(sel_store).unwrap_or(None));
    Table::new().show(ui, |ui| {
        TableHeader::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                TableHead::new("Name").width(220.0).show(ui);
                TableHead::new("Role").width(140.0).show(ui);
            });
        });
        TableBody::new().show(ui, |ui| {
            for (i, (name, role)) in [
                ("Ada Lovelace", "Engineer"),
                ("Grace Hopper", "Admiral"),
                ("Katherine Johnson", "Mathematician"),
            ]
            .into_iter()
            .enumerate()
            {
                let mut selected_row = selected == Some(i);
                let resp = TableRow::new()
                    .selected(selected_row)
                    .interactive(true)
                    .id_salt(format!("table_demo_row_{i}"))
                    .show(ui, |ui| {
                        TableCell::text(name).width(220.0).show_text(ui);
                        TableCell::text(role).width(140.0).show_text(ui);
                    });
                if resp.response.clicked() {
                    selected_row = !selected_row;
                }
                selected = if selected_row {
                    Some(i)
                } else if selected == Some(i) {
                    None
                } else {
                    selected
                };
            }
        });
    });
    ui.small(format!("selected row: {selected:?}"));

    ui.ctx().data_mut(|d| d.insert_temp(sel_store, selected));
}

pub fn empty_demo(ui: &mut egui::Ui) {
    ui.label("Bordered empty state with icon media, actions and inputs");
    ui.add_space(4.0);
    Empty::new().bordered(true).show(ui, |ui| {
        EmptyHeader::new().show(ui, |ui| {
            EmptyMedia::icon(Icon::Search).show(ui);
            EmptyTitle::new("No results found").show(ui);
            EmptyDescription::new(
                "We couldn't find anything matching your filters. Try a different search term.",
            )
            .show(ui);
        });
        EmptyContent::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                Button::new("Clear filters").show(ui);
                Button::new("Save search")
                    .variant(ButtonVariant::Outline)
                    .show(ui);
            });
        });
    });

    ui.add_space(16.0);
    ui.label("Form-style empty (new project)");
    ui.add_space(4.0);
    Empty::new().bordered(true).show(ui, |ui| {
        EmptyHeader::new().show(ui, |ui| {
            EmptyMedia::icon(Icon::Plus).show(ui);
            EmptyTitle::new("No projects yet").show(ui);
            EmptyDescription::new("Create your first project to get started.").show(ui);
        });
        EmptyContent::new().show(ui, |ui| {
            let store = egui::Id::unique("empty_demo_project");
            let mut name: String = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or_default());
            Input::new(&mut name)
                .placeholder("Project name")
                .width(240.0)
                .show(ui);
            ui.ctx().data_mut(|d| d.insert_temp(store, name));
            Button::new("Create project").show(ui);
        });
    });

    ui.add_space(16.0);
    ui.label("Bare (no border) and free-form media");
    ui.add_space(4.0);
    Empty::new().show(ui, |ui| {
        EmptyHeader::new().show(ui, |ui| {
            EmptyMedia::new().show_with(ui, |ui| {
                let theme = ShadcnTheme::of_ui(ui);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(64.0, 48.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 8, theme.palette.muted);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "img",
                    egui::FontId::proportional(12.0),
                    theme.palette.muted_foreground,
                );
            });
            EmptyTitle::new("Inbox is empty").show(ui);
            EmptyDescription::new("Messages you receive will show up here.").show(ui);
        });
        EmptyFooter::new().show(ui, |ui| {
            Button::new("Compose message").show(ui);
        });
    });
}

pub fn item_demo(ui: &mut egui::Ui) {
    // A generated texture for image media.
    let tex_store = egui::Id::unique("item_demo_texture");
    let tex_id = ui
        .ctx()
        .data_mut(|d| d.get_temp::<egui::TextureHandle>(tex_store));
    let texture = tex_id.unwrap_or_else(|| {
        let size = 64;
        let mut pixels = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let t = (x as f32 / size as f32 + y as f32 / size as f32) * 0.5;
                pixels.push(egui::Color32::from_rgb(
                    (40.0 + 180.0 * t) as u8,
                    (160.0 - 60.0 * t) as u8,
                    (200.0 - 80.0 * t) as u8,
                ));
            }
        }
        let image = egui::ColorImage::new([size, size], pixels);
        ui.ctx()
            .load_texture("item_demo_gradient", image, egui::TextureOptions::default())
    });
    let src: egui::ImageSource<'static> = egui::ImageSource::Texture(
        egui::load::SizedTexture::new(texture.id(), texture.size_vec2()),
    );

    ui.label("Outline items in a group (media/content/actions)");
    ui.add_space(4.0);
    ItemGroup::new().show(ui, |ui| {
        Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
            ItemMedia::image(src.clone()).show(ui);
            ItemContent::new().show(ui, |ui| {
                ItemTitle::new("Morning Ritual").show(ui);
                ItemDescription::new("Lofi meditation — 3:24").show(ui);
            });
            ItemActions::new().show(ui, |ui| {
                Button::new("")
                    .size(ButtonSize::IconSm)
                    .variant(ButtonVariant::Ghost)
                    .icon_start(Icon::MoreHorizontal)
                    .show(ui);
            });
        });
        Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
            ItemMedia::icon(Icon::User).show(ui);
            ItemContent::new().show(ui, |ui| {
                ItemTitle::new("Ada Lovelace").show(ui);
                ItemDescription::new("Engineer — writing the first programs.").show(ui);
            });
            ItemActions::new().show(ui, |ui| {
                Button::new("Follow").size(ButtonSize::Sm).show(ui);
            });
        });
        Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
            ItemContent::new().show(ui, |ui| {
                ItemTitle::new("Server credits due").show(ui);
                ItemDescription::new("Payment for server credits will be charged on the 14th.")
                    .show(ui);
            });
            ItemActions::new().show(ui, |ui| {
                Button::new("Pay")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Outline)
                    .show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("Muted and default variants");
    ui.add_space(4.0);
    ItemGroup::new().show(ui, |ui| {
        Item::new().variant(ItemVariant::Muted).show(ui, |ui| {
            ItemMedia::icon(Icon::CircleCheck).show(ui);
            ItemContent::new().show(ui, |ui| {
                ItemTitle::new("Deployment passed").show(ui);
                ItemDescription::new("All checks green on main.").show(ui);
            });
        });
        Item::new().show(ui, |ui| {
            ItemContent::new().show(ui, |ui| {
                ItemTitle::new("Transparent item").show(ui);
                ItemDescription::new("No fill, no border.").show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("Header and footer rows (actions right-aligned)");
    ui.add_space(4.0);
    Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
        ItemHeader::new().show(ui, |ui| {
            ItemTitle::new("Team settings").show(ui);
            ItemActions::new().show(ui, |ui| {
                Badge::new("Admin").show(ui);
            });
        });
        ItemContent::new().show(ui, |ui| {
            ItemDescription::new("Invite members and manage roles for this workspace.").show(ui);
        });
        ItemFooter::new().show(ui, |ui| {
            ui.label("12 members");
            ItemActions::new().show(ui, |ui| {
                Button::new("Manage").size(ButtonSize::Sm).show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("Interactive item (hover fill + focus ring, click to activate)");
    ui.add_space(4.0);
    let item_active_store = egui::Id::unique("item_demo_active");
    let mut active: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(item_active_store).unwrap_or(false));
    let item = Item::new()
        .interactive(true)
        .id_salt("item_demo_clickable")
        .show(ui, |ui| {
            ItemMedia::icon(Icon::ChevronsRight).show(ui);
            ItemContent::new().show(ui, |ui| {
                ItemTitle::new("Launch checklist").show_with(ui, |ui| {
                    Badge::new("Ready").show(ui);
                });
                ItemDescription::new("The item hovers like the source's anchor rows.").show(ui);
            });
            ItemActions::new().show(ui, |ui| {
                Button::new("Launch").size(ButtonSize::Sm).show(ui);
            });
        });
    if item.response.clicked() {
        active = !active;
    }
    ui.small(format!("activated: {active}"));
    ui.ctx()
        .data_mut(|d| d.insert_temp(item_active_store, active));

    ui.add_space(12.0);
    ui.label("Separator between items (nova my-2)");
    ui.add_space(4.0);
    Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
        ItemContent::new().show(ui, |ui| {
            ItemTitle::new("Above").show(ui);
        });
    });
    ItemSeparator::new().show(ui);
    Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
        ItemContent::new().show(ui, |ui| {
            ItemTitle::new("Below").show(ui);
        });
    });

    ui.add_space(12.0);
    ui.label("Sizes (group data-size inheritance)");
    ui.add_space(4.0);
    ItemGroup::new().size(ItemSize::Sm).show(ui, |ui| {
        Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
            ItemMedia::icon(Icon::Sparkles).show(ui);
            ItemContent::new().show(ui, |ui| {
                ItemTitle::new("Small item").show(ui);
            });
        });
        Item::new()
            .variant(ItemVariant::Outline)
            .size(ItemSize::Xs)
            .show(ui, |ui| {
                ItemContent::new().show(ui, |ui| {
                    ItemTitle::new("Extra small item").show(ui);
                });
            });
    });
}

pub fn scroll_area_demo(ui: &mut egui::Ui) {
    ui.label("Vertical (fixed height, themed 10px scrollbar)");
    ui.add_space(4.0);
    ScrollArea::new()
        .height(160.0)
        .id_salt("scroll_demo_vertical")
        .show(ui, |ui| {
            for i in 1..=40 {
                ui.label(format!("Item {i:02} — scroll me"));
            }
        });

    ui.add_space(12.0);
    ui.label("Horizontal");
    ui.add_space(4.0);
    ScrollArea::new()
        .vertical(false)
        .horizontal(true)
        .height(64.0)
        .id_salt("scroll_demo_horizontal")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for i in 1..=30 {
                    Button::new(format!("Tag {i}"))
                        .variant(ButtonVariant::Outline)
                        .show(ui);
                }
            });
        });

    ui.add_space(12.0);
    ui.label("Both axes");
    ui.add_space(4.0);
    ScrollArea::new()
        .horizontal(true)
        .height(140.0)
        .width(420.0)
        .id_salt("scroll_demo_both")
        .show(ui, |ui| {
            for row in 1..=20 {
                ui.horizontal(|ui| {
                    for col in 1..=20 {
                        ui.label(format!("{row}×{col}"));
                    }
                });
            }
        });

    ui.add_space(12.0);
    ui.label("Short content (no scrollbar needed)");
    ui.add_space(4.0);
    ScrollArea::new()
        .height(80.0)
        .id_salt("scroll_demo_short")
        .show(ui, |ui| {
            ui.label("Just a couple of lines — the scrollbar only appears when needed.");
            ui.label("Second line.");
        });
}

pub fn tabs_demo(ui: &mut egui::Ui) {
    let controlled_store = egui::Id::unique("tabs_demo_controlled");
    let mut controlled: String = ui.ctx().data_mut(|d| {
        d.get_temp(controlled_store)
            .unwrap_or_else(|| "account".to_owned())
    });

    ui.label("Default variant (bg-muted pill list)");
    ui.add_space(4.0);
    Tabs::new().id_salt("tabs_demo_default").show(ui, |ui| {
        TabsList::new().show(ui, |ui| {
            TabsTrigger::new("account", "Account").show(ui);
            TabsTrigger::new("password", "Password").show(ui);
        });
        TabsContent::new("account").show(ui, |ui| {
            ui.label("Make changes to your account here.");
        });
        TabsContent::new("password").show(ui, |ui| {
            ui.label("Change your password here.");
        });
    });

    ui.add_space(12.0);
    ui.label("Line variant (underline indicator, gap-1 triggers)");
    ui.add_space(4.0);
    Tabs::new().id_salt("tabs_demo_line").show(ui, |ui| {
        TabsList::new().variant(TabsVariant::Line).show(ui, |ui| {
            TabsTrigger::new("preview", "Preview").show(ui);
            TabsTrigger::new("code", "Code").show(ui);
            TabsTrigger::new("logs", "Logs").icon(Icon::Clock).show(ui);
        });
        TabsContent::new("preview").show(ui, |ui| {
            ui.label("Rendered output preview.");
        });
        TabsContent::new("code").show(ui, |ui| {
            ui.label("Source code panel.");
        });
        TabsContent::new("logs").show(ui, |ui| {
            ui.label("Build logs panel.");
        });
    });

    ui.add_space(12.0);
    ui.label(format!("Controlled (selected: {controlled:?})"));
    ui.add_space(4.0);
    Tabs::controlled(&mut controlled)
        .id_salt("tabs_demo_controlled")
        .show(ui, |ui| {
            TabsList::new().show(ui, |ui| {
                TabsTrigger::new("account", "Account").show(ui);
                TabsTrigger::new("password", "Password").show(ui);
            });
            TabsContent::new("account").show(ui, |ui| {
                ui.label("The selected String mirrors the active tab each frame.");
            });
            TabsContent::new("password").show(ui, |ui| {
                ui.label("Switched programmatically or by clicking a trigger.");
            });
        });
    ui.small(format!("value = {controlled:?}"));

    ui.add_space(12.0);
    ui.label("Vertical orientation");
    ui.add_space(4.0);
    Tabs::new()
        .id_salt("tabs_demo_vertical")
        .orientation(TabsOrientation::Vertical)
        .show(ui, |ui| {
            TabsList::new().show(ui, |ui| {
                TabsTrigger::new("general", "General").show(ui);
                TabsTrigger::new("security", "Security").show(ui);
                TabsTrigger::new("notifications", "Notifications").show(ui);
            });
            TabsContent::new("general").show(ui, |ui| {
                ui.label("General settings.");
            });
            TabsContent::new("security").show(ui, |ui| {
                ui.label("Security settings.");
            });
            TabsContent::new("notifications").show(ui, |ui| {
                ui.label("Notification settings.");
            });
        });

    ui.add_space(12.0);
    ui.label("Line + vertical (side indicator on the active trigger)");
    ui.add_space(4.0);
    Tabs::new()
        .id_salt("tabs_demo_vertical_line")
        .orientation(TabsOrientation::Vertical)
        .show(ui, |ui| {
            TabsList::new().variant(TabsVariant::Line).show(ui, |ui| {
                TabsTrigger::new("outline", "Outline").show(ui);
                TabsTrigger::new("history", "History").show(ui);
                TabsTrigger::new("trash", "Trash").show(ui);
            });
            TabsContent::new("outline").show(ui, |ui| {
                ui.label("Document outline.");
            });
            TabsContent::new("history").show(ui, |ui| {
                ui.label("Version history.");
            });
            TabsContent::new("trash").show(ui, |ui| {
                ui.label("Deleted items.");
            });
        });

    ui.add_space(12.0);
    ui.label("With icons and a disabled trigger");
    ui.add_space(4.0);
    Tabs::new().id_salt("tabs_demo_icons").show(ui, |ui| {
        TabsList::new().show(ui, |ui| {
            TabsTrigger::new("files", "Files")
                .icon(Icon::Paperclip)
                .show(ui);
            TabsTrigger::new("search", "Search")
                .icon(Icon::Search)
                .show(ui);
            TabsTrigger::new("locked", "Locked").disabled(true).show(ui);
        });
        TabsContent::new("files").show(ui, |ui| {
            ui.label("Attachments live here.");
        });
        TabsContent::new("search").show(ui, |ui| {
            ui.label("Search across the workspace.");
        });
        TabsContent::new("locked").show(ui, |ui| {
            ui.label("Requires a higher plan.");
        });
    });

    ui.add_space(12.0);
    ui.label(
        "Keyboard: Arrow keys move focus between triggers and select them (automatic activation); \
         Home/End jump; Enter or Space activates the focused trigger.",
    );

    ui.ctx()
        .data_mut(|d| d.insert_temp(controlled_store, controlled));
}

pub fn breadcrumb_demo(ui: &mut egui::Ui) {
    ui.label("Default trail (links, separators, current page)");
    ui.add_space(4.0);
    Breadcrumb::new().show(ui, |ui| {
        BreadcrumbList::new().show(ui, |ui| {
            BreadcrumbItem::new().show(ui, |ui| {
                BreadcrumbLink::new("Home").show(ui);
            });
            BreadcrumbSeparator::new().show(ui);
            BreadcrumbItem::new().show(ui, |ui| {
                BreadcrumbLink::new("Components").show(ui);
            });
            BreadcrumbSeparator::new().show(ui);
            BreadcrumbItem::new().show(ui, |ui| {
                BreadcrumbPage::new("Breadcrumb").show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("With an ellipsis slot for collapsed levels");
    ui.add_space(4.0);
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
                BreadcrumbLink::new("Design System").show(ui);
            });
            BreadcrumbSeparator::new().show(ui);
            BreadcrumbItem::new().show(ui, |ui| {
                BreadcrumbLink::new("Components").show(ui);
            });
            BreadcrumbSeparator::new().show(ui);
            BreadcrumbItem::new().show(ui, |ui| {
                BreadcrumbPage::new("Breadcrumb").show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("Custom separator icon (the source accepts arbitrary children)");
    ui.add_space(4.0);
    Breadcrumb::new().show(ui, |ui| {
        BreadcrumbList::new().show(ui, |ui| {
            BreadcrumbItem::new().show(ui, |ui| {
                BreadcrumbLink::new("Home").show(ui);
            });
            BreadcrumbSeparator::new().icon(Icon::Minus).show(ui);
            BreadcrumbItem::new().show(ui, |ui| {
                BreadcrumbPage::new("Breadcrumb").show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("Hover a link — it brightens (hover:text-foreground).");

    let clicked_store = egui::Id::unique("breadcrumb_demo_clicked");
    let mut clicks: u32 = ui
        .ctx()
        .data_mut(|d| d.get_temp(clicked_store).unwrap_or(0));
    ui.add_space(8.0);
    if BreadcrumbLink::new("Clickable link (counts clicks)")
        .show(ui)
        .clicked()
    {
        clicks += 1;
    }
    if clicks > 0 {
        ui.small(format!("link clicked {clicks} time(s)"));
    }
    ui.ctx().data_mut(|d| d.insert_temp(clicked_store, clicks));
}

pub fn pagination_demo(ui: &mut egui::Ui) {
    let page_store = egui::Id::unique("pagination_demo_page");
    let total_pages = 9usize;
    let mut page: usize = ui.ctx().data_mut(|d| d.get_temp(page_store).unwrap_or(2));

    ui.label("Interactive (click pages, Previous and Next)");
    ui.add_space(4.0);
    Pagination::new().show(ui, |ui| {
        PaginationContent::new().show(ui, |ui| {
            PaginationItem::new().show(ui, |ui| {
                if PaginationPrevious::new()
                    .disabled(page == 1)
                    .show(ui)
                    .clicked()
                {
                    page = page.saturating_sub(1).max(1);
                }
            });
            for p in 1..=4usize {
                PaginationItem::new().show(ui, |ui| {
                    if PaginationLink::new(p.to_string())
                        .active(p == page)
                        .show(ui)
                        .clicked()
                    {
                        page = p;
                    }
                });
            }
            PaginationItem::new().show(ui, |ui| {
                PaginationEllipsis::new().show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                if PaginationLink::new(total_pages.to_string())
                    .active(page == total_pages)
                    .show(ui)
                    .clicked()
                {
                    page = total_pages;
                }
            });
            PaginationItem::new().show(ui, |ui| {
                if PaginationNext::new()
                    .disabled(page == total_pages)
                    .show(ui)
                    .clicked()
                {
                    page = (page + 1).min(total_pages);
                }
            });
        });
    });
    ui.small(format!("current page: {page} of {total_pages}"));

    ui.add_space(12.0);
    ui.label("Disabled states (first page)");
    ui.add_space(4.0);
    Pagination::new().show(ui, |ui| {
        PaginationContent::new().show(ui, |ui| {
            PaginationItem::new().show(ui, |ui| {
                PaginationPrevious::new().disabled(true).show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationLink::new("1").active(true).show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationLink::new("2").show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationLink::new("3").show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationEllipsis::new().show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationNext::new().show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("Last page (Next disabled, wide page label)");
    ui.add_space(4.0);
    Pagination::new().show(ui, |ui| {
        PaginationContent::new().show(ui, |ui| {
            PaginationItem::new().show(ui, |ui| {
                PaginationPrevious::new().show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationEllipsis::new().show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationLink::new("98").show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationLink::new("99").show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationLink::new("100")
                    .active(true)
                    .size(ButtonSize::Sm)
                    .show(ui);
            });
            PaginationItem::new().show(ui, |ui| {
                PaginationNext::new().disabled(true).show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("The active page uses the Outline button variant, the rest Ghost.");

    ui.ctx().data_mut(|d| d.insert_temp(page_store, page));
}

pub fn collapsible_demo(ui: &mut egui::Ui) {
    let controlled_store = egui::Id::unique("collapsible_demo_controlled");
    let mut controlled: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(controlled_store).unwrap_or(false));

    ui.label("Uncontrolled (state persisted per id_salt)");
    ui.add_space(4.0);
    Collapsible::new()
        .id_salt("collapsible_demo_1")
        .show(ui, |ui| {
            CollapsibleTrigger::new("@peduarte starred 3 repositories")
                .icon_end(Icon::ChevronDown)
                .show(ui);
            CollapsibleContent::new().show(ui, |ui| {
                ui.label("@radix-ui/primitives");
                ui.label("@radix-ui/colors");
                ui.label("@stitches/react");
            });
        });

    ui.add_space(12.0);
    ui.label("Icon trigger, open by default (demo-style bordered rows)");
    ui.add_space(4.0);
    Collapsible::new()
        .id_salt("collapsible_demo_2")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("@peduarte starred 3 repositories");
                CollapsibleTrigger::new("")
                    .icon_start(Icon::ChevronDown)
                    .id_salt("collapsible_demo_2_trigger")
                    .show(ui);
            });
            CollapsibleContent::new().show(ui, |ui| {
                for name in ["@radix-ui/primitives", "@radix-ui/colors"] {
                    ui.add_space(2.0);
                    BorderRow::draw(ui, name);
                }
            });
        });

    ui.add_space(12.0);
    ui.label(format!("Controlled (open: {controlled})"));
    ui.add_space(4.0);
    Collapsible::controlled(&mut controlled)
        .id_salt("collapsible_demo_3")
        .show(ui, |ui| {
            CollapsibleTrigger::new("Toggle details").show(ui);
            CollapsibleContent::new().show(ui, |ui| {
                ui.label("Anything inside the content region animates open and closed.");
                ui.label("The `controlled` bool mirrors the open state each frame.");
            });
        });
    ui.small(format!("open = {controlled}"));

    ui.add_space(12.0);
    ui.label("Disabled trigger");
    ui.add_space(4.0);
    Collapsible::new()
        .id_salt("collapsible_demo_4")
        .show(ui, |ui| {
            CollapsibleTrigger::new("Cannot be opened")
                .disabled(true)
                .show(ui);
            CollapsibleContent::new().show(ui, |ui| {
                ui.label("Hidden content.");
            });
        });

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a trigger, press Space or Enter to toggle.");

    ui.ctx()
        .data_mut(|d| d.insert_temp(controlled_store, controlled));
}

/// A tiny helper that mimics the demo's `rounded-md border px-4 py-2` row.
struct BorderRow;

impl BorderRow {
    fn draw(ui: &mut egui::Ui, text: &str) {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width().min(350.0), 36.0),
            egui::Sense::hover(),
        );
        ui.painter().rect_stroke(
            rect,
            theme.radius.md_u8(),
            egui::Stroke::new(1.0, theme.palette.border),
            egui::epaint::StrokeKind::Inside,
        );
        ui.painter().text(
            egui::Pos2::new(rect.min.x + 16.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            egui::FontId::monospace(14.0),
            theme.palette.foreground,
        );
    }
}

pub fn resizable_demo(ui: &mut egui::Ui) {
    ui.label("Horizontal group (drag the handle; grip chip shown)");
    ui.add_space(4.0);
    let store = egui::Id::unique("resizable_demo_left");
    let mut left_count: usize = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or(3));
    ResizablePanelGroup::new()
        .id_salt("resizable_demo_h")
        .length(440.0)
        .show(ui, |ui| {
            ResizablePanel::new().min(0.2).max(0.7).show(ui, |ui| {
                ui.heading("Left panel");
                for i in 1..=left_count {
                    ui.label(format!("Item {i}"));
                }
                if Button::new("Add item")
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    left_count += 1;
                }
            });
            ResizableHandle::new().with_handle(true).show(ui);
            ResizablePanel::new().show(ui, |ui| {
                ui.heading("Right panel");
                ui.label("Drag the divider between us.");
                ui.small("Panel sizes persist across frames (per group id).");
            });
        });
    ui.ctx().data_mut(|d| d.insert_temp(store, left_count));

    ui.add_space(12.0);
    ui.label("Vertical group (thin handles, arrow keys when focused)");
    ui.add_space(4.0);
    ResizablePanelGroup::new()
        .id_salt("resizable_demo_v")
        .orientation(ResizableOrientation::Vertical)
        .length(200.0)
        .show(ui, |ui| {
            ResizablePanel::new().min(0.15).show(ui, |ui| {
                ui.heading("Top");
                ui.label("Tab to a handle and use Arrow Up / Arrow Down.");
            });
            ResizableHandle::new().show(ui);
            ResizablePanel::new().show(ui, |ui| {
                ui.heading("Middle");
                ui.label("20% minimum via .min(0.15).");
            });
            ResizableHandle::new().with_handle(true).show(ui);
            ResizablePanel::new().show(ui, |ui| {
                ui.heading("Bottom");
            });
        });

    ui.add_space(12.0);
    ui.label("Three panels with clamps (middle capped at 0.4)");
    ui.add_space(4.0);
    ResizablePanelGroup::new()
        .id_salt("resizable_demo_3")
        .length(440.0)
        .show(ui, |ui| {
            ResizablePanel::new().min(0.15).show(ui, |ui| {
                ui.label("Sidebar");
            });
            ResizableHandle::new().show(ui);
            ResizablePanel::new().max(0.4).show(ui, |ui| {
                ui.label("Content (max 40%)");
            });
            ResizableHandle::new().show(ui);
            ResizablePanel::new().min(0.2).show(ui, |ui| {
                ui.label("Details");
            });
        });
}

pub fn dropdown_menu_demo(ui: &mut egui::Ui) {
    let state = egui::Id::unique("dropdown_menu_demo_state");
    let action_store = egui::Id::unique("dropdown_menu_demo_action");
    let ctrl_store = egui::Id::unique("dropdown_menu_demo_controlled");
    let (mut show_status_bar, mut show_activity, mut show_panel, mut theme): (
        bool,
        bool,
        bool,
        Option<usize>,
    ) = ui
        .ctx()
        .data_mut(|d| d.get_temp(state).unwrap_or((true, false, false, Some(0))));
    let mut last_action: Option<String> = ui
        .ctx()
        .data_mut(|d| d.get_temp(action_store).unwrap_or(None));

    ui.label("Click a trigger to open. ArrowUp/Down navigate, ArrowRight opens sub-menus, Enter/Space activate, Escape closes, typing jumps to matching items.");
    ui.add_space(8.0);

    ui.label("Basic (docs demo, w-56, align start)");
    ui.add_space(4.0);
    let res = DropdownMenu::new("Open")
        .align(MenuAlign::Start)
        .width(224.0)
        .id_salt("dropdown_demo_basic")
        .show(ui, |m| {
            m.label("My Account");
            m.separator();
            m.item(MenuItem::new("Profile").shortcut("⇧⌘P"));
            m.item(MenuItem::new("Billing").shortcut("⌘B"));
            m.item(MenuItem::new("Settings").shortcut("⌘S"));
            m.item(MenuItem::new("Keyboard shortcuts").shortcut("⌘K"));
            m.separator();
            m.item(MenuItem::new("Team").icon(Icon::User));
            m.sub_menu("Invite users", |m| {
                m.item(MenuItem::new("Email").icon(Icon::Send));
                m.item(MenuItem::new("Message"));
                m.separator();
                m.item(MenuItem::new("More…"));
            });
            m.item(MenuItem::new("New Team").shortcut("⌘+T"));
            m.separator();
            m.item(MenuItem::new("GitHub"));
            m.item(MenuItem::new("Support"));
            m.item(MenuItem::new("API").disabled(true));
            m.separator();
            m.item(MenuItem::new("Log out").shortcut("⇧⌘Q").destructive(true));
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }

    ui.add_space(12.0);
    ui.label("Checkbox items (one disabled)");
    ui.add_space(4.0);
    let res = DropdownMenu::new("Checkboxes")
        .align(MenuAlign::Start)
        .width(224.0)
        .id_salt("dropdown_demo_checkboxes")
        .show(ui, |m| {
            m.label("Appearance");
            m.separator();
            m.checkbox_item(CheckboxItem::new("Status Bar", &mut show_status_bar));
            m.checkbox_item(CheckboxItem::new("Activity Bar", &mut show_activity).disabled(true));
            m.checkbox_item(CheckboxItem::new("Panel", &mut show_panel));
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }

    ui.add_space(12.0);
    ui.label("Radio group + destructive item with icon");
    ui.add_space(4.0);
    let res = DropdownMenu::new("Radio group")
        .align(MenuAlign::Start)
        .width(224.0)
        .id_salt("dropdown_demo_radio")
        .show(ui, |m| {
            m.label("Theme");
            m.radio_group(&mut theme, |g| {
                g.radio_items([
                    RadioItem::new("Light").shortcut("⌘1"),
                    RadioItem::new("Dark").shortcut("⌘2"),
                    RadioItem::new("System").shortcut("⌘3"),
                ]);
            });
            m.separator();
            m.item(
                MenuItem::new("Delete all data")
                    .icon(Icon::Trash)
                    .destructive(true),
            );
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }

    ui.add_space(12.0);
    ui.label("Alignment against the trigger");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        DropdownMenu::new("Start")
            .align(MenuAlign::Start)
            .id_salt("dropdown_demo_align_start")
            .show(ui, |m| m.item(MenuItem::new("Aligned start")));
        DropdownMenu::new("Center")
            .align(MenuAlign::Center)
            .id_salt("dropdown_demo_align_center")
            .show(ui, |m| m.item(MenuItem::new("Aligned center")));
        DropdownMenu::new("End")
            .align(MenuAlign::End)
            .id_salt("dropdown_demo_align_end")
            .show(ui, |m| m.item(MenuItem::new("Aligned end")));
    });

    ui.add_space(12.0);
    ui.label("Inset sub-menu trigger and a long list (clamped to the window, scrollable)");
    ui.add_space(4.0);
    let res = DropdownMenu::new("Open")
        .align(MenuAlign::Start)
        .id_salt("dropdown_demo_scroll")
        .show(ui, |m| {
            m.sub_menu_inset("Nested preferences", |m| {
                m.item(MenuItem::new("Editor font"));
                m.item(MenuItem::new("Terminal font"));
            });
            m.separator();
            for i in 1..=30 {
                m.item(MenuItem::new(format!("Entry {i}")));
            }
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }

    ui.add_space(12.0);
    let mut open: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctrl_store).unwrap_or(false));
    ui.horizontal(|ui| {
        ui.label("Controlled open state:");
        DropdownMenu::controlled("Controlled", &mut open)
            .align(MenuAlign::Start)
            .id_salt("dropdown_demo_controlled")
            .show(ui, |m| {
                m.item(MenuItem::new("Refresh").shortcut("⌘R"));
                m.item(MenuItem::new("Reload").shortcut("⇧⌘R").disabled(true));
            });
        ui.label(format!("open = {open}"));
    });
    ui.ctx().data_mut(|d| d.insert_temp(ctrl_store, open));

    ui.add_space(12.0);
    if let Some(action) = &last_action {
        ui.label(format!("Last activated item: {action}"));
    }
    ui.ctx().data_mut(|d| {
        d.insert_temp(state, (show_status_bar, show_activity, show_panel, theme));
        d.insert_temp(action_store, last_action);
    });
}

pub fn context_menu_demo(ui: &mut egui::Ui) {
    let state = egui::Id::unique("context_menu_demo_state");
    let action_store = egui::Id::unique("context_menu_demo_action");
    let ctrl_store = egui::Id::unique("context_menu_demo_controlled");
    let (mut show_bookmarks, mut show_full_urls, mut people): (bool, bool, Option<usize>) = ui
        .ctx()
        .data_mut(|d| d.get_temp(state).unwrap_or((true, false, Some(1))));
    let mut last_action: Option<String> = ui
        .ctx()
        .data_mut(|d| d.get_temp(action_store).unwrap_or(None));

    ui.label("Right-click inside a box to open its menu. Escape closes, Arrow keys navigate, Enter activates, typing jumps to matching items.");
    ui.add_space(8.0);

    ui.label("Basic");
    ui.add_space(4.0);
    ui.group(|ui| {
        ui.set_min_size(egui::vec2(280.0, 64.0));
        ui.label("Right-click here");
        ui.label("Back / Forward / Save / Print…");
    });
    let res = ContextMenu::new()
        .id_salt("context_menu_demo_basic")
        .show(ui, |m| {
            m.item(MenuItem::new("Back").shortcut("⌘[").icon(Icon::ArrowLeft));
            m.item(MenuItem::new("Forward").shortcut("⌘]").disabled(true));
            m.separator();
            m.sub_menu("Share", |m| {
                m.item(MenuItem::new("Email").icon(Icon::Send));
                m.item(MenuItem::new("Message"));
                m.separator();
                m.item(MenuItem::new("More…"));
            });
            m.separator();
            m.item(MenuItem::new("Save").shortcut("⌘S"));
            m.item(MenuItem::new("Print…").shortcut("⌘P"));
            m.separator();
            m.item(
                MenuItem::new("Delete")
                    .shortcut("⌘⌫")
                    .destructive(true)
                    .icon(Icon::Trash),
            );
            m.item(MenuItem::new("Disabled entry").disabled(true));
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }

    ui.add_space(12.0);
    ui.label("Checkbox + radio items (menus stay open while toggling is on-screen via re-open)");
    ui.add_space(4.0);
    ui.group(|ui| {
        ui.set_min_size(egui::vec2(280.0, 64.0));
        ui.label("Right-click here for checkboxes and a radio group");
    });
    let res = ContextMenu::new()
        .id_salt("context_menu_demo_checks")
        .show(ui, |m| {
            m.label("Appearance");
            m.separator();
            m.checkbox_item(CheckboxItem::new("Show Bookmarks Bar", &mut show_bookmarks));
            m.checkbox_item(
                CheckboxItem::new("Always Show Full URLs", &mut show_full_urls).disabled(true),
            );
            m.separator();
            m.label_inset("People");
            m.radio_group(&mut people, |g| {
                g.radio_items([
                    RadioItem::new("Eduardo Lavaque").inset(true),
                    RadioItem::new("Shadcn").inset(true),
                    RadioItem::new("Egui").disabled(true),
                ]);
            });
            m.separator();
            m.item(MenuItem::new("Reset").destructive(true));
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }

    ui.add_space(12.0);
    let mut open: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctrl_store).unwrap_or(false));
    ui.horizontal(|ui| {
        ui.label("Controlled open state:");
        ui.label(format!("open = {open}"));
    });
    ui.group(|ui| {
        ui.set_min_size(egui::vec2(280.0, 48.0));
        ui.label("Right-click here (controlled)");
    });
    let res = ContextMenu::controlled(&mut open)
        .id_salt("context_menu_demo_controlled")
        .show(ui, |m| {
            m.item(MenuItem::new("Refresh").shortcut("⌘R"));
            m.item(MenuItem::new("Copy location").shortcut("⌥⌘C"));
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }
    ui.ctx().data_mut(|d| d.insert_temp(ctrl_store, open));

    ui.add_space(12.0);
    if let Some(action) = &last_action {
        ui.label(format!("Last activated item: {action}"));
    }
    ui.ctx().data_mut(|d| {
        d.insert_temp(state, (show_bookmarks, show_full_urls, people));
        d.insert_temp(action_store, last_action);
    });
}

pub fn menubar_demo(ui: &mut egui::Ui) {
    let state = egui::Id::unique("menubar_demo_state");
    let action_store = egui::Id::unique("menubar_demo_action");
    let ctrl_store = egui::Id::unique("menubar_demo_controlled");
    let (mut bookmarks, mut full_urls, mut theme, mut zoom): (
        bool,
        bool,
        Option<usize>,
        Option<usize>,
    ) = ui
        .ctx()
        .data_mut(|d| d.get_temp(state).unwrap_or((true, false, Some(0), Some(1))));
    let mut last_action: Option<String> = ui
        .ctx()
        .data_mut(|d| d.get_temp(action_store).unwrap_or(None));

    ui.label("Click a menu to open it. Hovering another menu switches while one is open. Tab reaches the triggers when nothing is open (ArrowDown or Enter opens the focused one), ArrowLeft/Right switch menus, ArrowUp/Down navigate items, typing jumps to matching items, Enter activates, Escape closes.");
    ui.add_space(8.0);

    let res = Menubar::new().id_salt("menubar_demo").show(ui, |bar| {
        bar.menu("File", |m| {
            m.item(MenuItem::new("New Tab").shortcut("⌘T"));
            m.item(MenuItem::new("New Window").shortcut("⌘N"));
            m.item(MenuItem::new("New Incognito Window").disabled(true));
            m.separator();
            m.sub_menu("Share", |m| {
                m.item(MenuItem::new("Email link"));
                m.item(MenuItem::new("Messages"));
                m.item(MenuItem::new("Notes"));
            });
            m.separator();
            m.item(MenuItem::new("Print…").shortcut("⌘P"));
        });
        bar.menu("Edit", |m| {
            m.item(MenuItem::new("Undo").shortcut("⌘Z"));
            m.item(MenuItem::new("Redo").shortcut("⇧⌘Z"));
            m.separator();
            m.sub_menu("Find", |m| {
                m.item(MenuItem::new("Search the web"));
                m.separator();
                m.item(MenuItem::new("Find…"));
                m.item(MenuItem::new("Find Next"));
                m.item(MenuItem::new("Find Previous"));
            });
            m.separator();
            m.item(MenuItem::new("Cut"));
            m.item(MenuItem::new("Copy"));
            m.item(MenuItem::new("Paste"));
        });
        bar.menu("View", |m| {
            m.checkbox_item(CheckboxItem::new(
                "Always Show Bookmarks Bar",
                &mut bookmarks,
            ));
            m.checkbox_item(
                CheckboxItem::new("Always Show Full URLs", &mut full_urls).disabled(true),
            );
            m.separator();
            m.label_inset("Theme");
            m.radio_group(&mut theme, |g| {
                g.radio_items([
                    RadioItem::new("Light").inset(true),
                    RadioItem::new("Dark").inset(true),
                    RadioItem::new("System").inset(true),
                ]);
            });
            m.separator();
            m.item(MenuItem::new("Reload").shortcut("⌘R").inset(true));
        });
        bar.menu("Zoom", |m| {
            m.radio_group(&mut zoom, |g| {
                g.radio_items([
                    RadioItem::new("100%").shortcut("⌘0"),
                    RadioItem::new("125%"),
                    RadioItem::new("150%"),
                ]);
            });
            m.separator();
            m.item(MenuItem::new("Reset zoom").destructive(true));
            m.item(MenuItem::new("Zoom In").shortcut("⌘+"));
            m.item(MenuItem::new("Zoom Out").shortcut("⌘-"));
        });
    });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }

    ui.add_space(12.0);
    let mut open_index: Option<usize> = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctrl_store).unwrap_or(None));
    ui.label("Controlled (open menu index owned by the app):");
    ui.add_space(4.0);
    let res = Menubar::controlled(&mut open_index)
        .id_salt("menubar_demo_controlled")
        .show(ui, |bar| {
            bar.menu("Actions", |m| {
                m.item(MenuItem::new("Refresh").shortcut("⌘R"));
                m.item(MenuItem::new("Duplicate").shortcut("⌘D").disabled(true));
            });
            bar.menu("Help", |m| {
                m.item(MenuItem::new("Docs"));
                m.item(MenuItem::new("Changelog"));
                m.separator();
                m.item(MenuItem::new("Report issue").destructive(true));
            });
        });
    if let Some(a) = res.activated {
        last_action = Some(a.id);
    }
    if let Some(i) = open_index {
        ui.label(format!("Open menu: {i}"));
    }
    ui.ctx().data_mut(|d| d.insert_temp(ctrl_store, open_index));

    ui.add_space(12.0);
    if let Some(action) = &last_action {
        ui.label(format!("Last activated item: {action}"));
    }
    ui.ctx().data_mut(|d| {
        d.insert_temp(state, (bookmarks, full_urls, theme, zoom));
        d.insert_temp(action_store, last_action);
    });
}

pub fn command_demo(ui: &mut egui::Ui) {
    ui.heading("Command");
    ui.add_space(8.0);

    // Standalone palette with groups, icons, shortcuts, disabled items.
    let state_id = egui::Id::unique("command_demo_state");
    let (mut activated, mut open) = ui.ctx().data_mut(|d| {
        d.get_temp::<(Option<String>, bool)>(state_id)
            .unwrap_or((None, false))
    });
    let res = Command::new().id_salt("command_demo").show(ui, |c| {
        c.input(CommandInput::new().placeholder("Type a command or search..."));
        c.group("Suggestions", |g| {
            g.item(
                CommandItem::new("Calendar")
                    .icon(Icon::Calendar)
                    .shortcut("⌘E"),
            );
            g.item(CommandItem::new("Search Emoji"));
            g.item(CommandItem::new("Calculator").disabled(true));
        });
        c.separator();
        c.group("Settings", |g| {
            g.item(CommandItem::new("Theme").shortcut("⌘T"));
            g.item(CommandItem::new("Profile").shortcut("⌘P"));
        });
    });
    if res.activated.is_some() {
        activated = res.activated.as_ref().map(|a| a.label.clone());
    }
    ui.add_space(4.0);
    ui.label(format!(
        "Query: {:?} · Last activated: {}",
        res.query,
        activated.as_deref().unwrap_or("(none)")
    ));

    ui.add_space(12.0);
    ui.label("Dialog palette (⌘K style)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if Button::new("Open palette")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            open = true;
        }
        ui.label("(or press the button — Escape and backdrop close it)");
    });
    let dlg = CommandDialog::controlled(&mut open)
        .id_salt("command_demo_dialog")
        .show(ui, |c| {
            c.input(CommandInput::new().placeholder("Type a command or search..."));
            c.group("Actions", |g| {
                g.item(CommandItem::new("Log out").shortcut("⇧⌘Q"));
                g.item(CommandItem::new("Delete account").disabled(true));
            });
            c.group("Search", |g| {
                g.item(CommandItem::new("Docs"));
                g.item(CommandItem::new("Changelog"));
            });
        });
    if let Some(a) = dlg.activated {
        activated = Some(a.label);
    }

    ui.ctx()
        .data_mut(|d| d.insert_temp(state_id, (activated, open)));
}

pub fn combobox_demo(ui: &mut egui::Ui) {
    ui.heading("Combobox");
    ui.add_space(8.0);

    let store = egui::Id::unique("combobox_demo_state");
    let (mut framework, mut picked, mut fruit) = ui.ctx().data_mut(|d| {
        d.get_temp::<(Option<String>, Vec<String>, String)>(store)
            .unwrap_or((None, Vec::new(), String::new()))
    });

    ui.label("Single select (with clear)");
    ui.add_space(4.0);
    let res = Combobox::new(&mut framework)
        .placeholder("Select a framework...")
        .id_salt("combobox_demo_single")
        .show_clear(true)
        .options(["Next.js", "SvelteKit", "Nuxt.js", "Remix", "Astro"])
        .show_seeded(ui);
    ui.add_space(4.0);
    ui.label(format!(
        "Value: {}{}",
        res.value.as_deref().unwrap_or("(none)"),
        if res.changed { " (changed)" } else { "" }
    ));

    ui.add_space(12.0);
    ui.label("Multi select (wrapped chips, removable)");
    ui.add_space(4.0);
    Combobox::multi(&mut picked)
        .placeholder("Frameworks...")
        .id_salt("combobox_demo_multi")
        .options([
            "Next.js",
            "SvelteKit",
            "Nuxt.js",
            "Remix",
            "Astro",
            "Vite",
            "Angular",
        ])
        .show_seeded(ui);

    ui.add_space(12.0);
    ui.label("States");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Combobox::uncontrolled()
            .placeholder("Disabled")
            .disabled(true)
            .id_salt("combobox_demo_disabled")
            .options(["A", "B"])
            .show_seeded(ui);
        Combobox::uncontrolled()
            .placeholder("Invalid")
            .invalid(true)
            .id_salt("combobox_demo_invalid")
            .options(["A", "B"])
            .show_seeded(ui);
        Combobox::bind(&mut fruit)
            .placeholder("String-bound")
            .id_salt("combobox_demo_string")
            .options(["Apple", "Pear", "Cherry"])
            .show_seeded(ui);
    });

    ui.add_space(12.0);
    ui.label("With disabled options and wide list");
    ui.add_space(4.0);
    Combobox::uncontrolled()
        .placeholder("Pick a number")
        .id_salt("combobox_demo_scroll")
        .width(260.0)
        .options((1..=40).map(|i| format!("Option {i}")))
        .show_seeded(ui);

    ui.ctx()
        .data_mut(|d| d.insert_temp(store, (framework, picked, fruit)));
}

pub fn navigation_menu_demo(ui: &mut egui::Ui) {
    ui.heading("Navigation Menu");
    ui.add_space(8.0);

    let store = egui::Id::unique("nav_demo_state");
    let (mut open_label, mut link_clicked) = ui.ctx().data_mut(|d| {
        d.get_temp::<(Option<String>, Option<String>)>(store)
            .unwrap_or_default()
    });

    ui.label("Horizontal (shared viewport)");
    ui.add_space(4.0);
    let res = NavigationMenu::new().id_salt("nav_demo").show(ui, |nav| {
        nav.link("Introduction");
        nav.trigger("Learn", |c| {
            c.link("Library", "Structured access to collection data.");
            c.link(
                "Components",
                "Building blocks for composing interfaces quickly.",
            );
        });
        nav.trigger("Examples", |c| {
            c.link("Email", "Send and receive email.");
            c.link_plain("Docs");
        });
        nav.trigger_disabled("Enterprise");
    });
    if res.open.is_some() {
        open_label = res.open;
    }
    if res.link_clicked.is_some() {
        link_clicked = res.link_clicked;
    }

    ui.add_space(12.0);
    ui.label("Inline panels (viewport = false)");
    ui.add_space(4.0);
    NavigationMenu::new()
        .viewport(false)
        .id_salt("nav_demo_inline")
        .show(ui, |nav| {
            nav.trigger("Playground", |c| {
                c.link("History", "Browse recent activity.");
                c.link("Settings", "Tune your workspace.");
            });
            nav.trigger("Community", |c| {
                c.link("Forum", "Ask questions and share work.");
            });
        });

    ui.add_space(12.0);
    ui.label("Vertical");
    ui.add_space(4.0);
    NavigationMenu::new()
        .orientation(NavOrientation::Vertical)
        .id_salt("nav_demo_vertical")
        .show(ui, |nav| {
            nav.trigger("Learn", |c| {
                c.link("Library", "Structured access to collection data.");
            });
            nav.trigger("Examples", |c| {
                c.link("Email", "Send and receive email.");
            });
            nav.link("Introduction");
        });

    ui.add_space(12.0);
    ui.label(format!(
        "Open: {} · Last link: {}",
        open_label.as_deref().unwrap_or("(none)"),
        link_clicked.as_deref().unwrap_or("(none)")
    ));
    ui.ctx()
        .data_mut(|d| d.insert_temp(store, (open_label, link_clicked)));
}

pub fn calendar_demo(ui: &mut egui::Ui) {
    use egui_shadcn_ui::components::calendar::{CalendarDate, WeekStart};

    let store = egui::Id::unique("calendar_demo_values");
    #[derive(Clone, Default)]
    struct DemoState {
        single: Option<CalendarDate>,
        multiple: Vec<CalendarDate>,
        range: Option<(CalendarDate, CalendarDate)>,
    }
    let mut state: DemoState = ui
        .ctx()
        .data_mut(|d| d.get_temp::<DemoState>(store).unwrap_or_default());

    ui.label("Single");
    ui.add_space(4.0);
    let res = Calendar::single(&mut state.single).show(ui);
    ui.add_space(4.0);
    ui.label(match res.clicked_day {
        Some(d) => format!("Clicked: {}", d.format_ppp()),
        None => match state.single {
            Some(d) => format!("Selected: {}", d.format_ppp()),
            None => "No date selected".to_owned(),
        },
    });

    ui.add_space(16.0);
    ui.label("Multiple");
    ui.add_space(4.0);
    Calendar::multiple(&mut state.multiple)
        .week_starts_on(WeekStart::Monday)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Range");
    ui.add_space(4.0);
    Calendar::range(&mut state.range).show(ui);

    ui.add_space(16.0);
    ui.label("Without outside days, from a fixed month");
    ui.add_space(4.0);
    let mut fixed = Some(CalendarDate::new(2026, 11, 10).unwrap());
    Calendar::single(&mut fixed)
        .default_month(CalendarDate::new(2026, 11, 1).unwrap())
        .show_outside_days(false)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Disabled weekends");
    ui.add_space(4.0);
    let mut weekend = None;
    Calendar::single(&mut weekend)
        .disabled_when(|d| matches!(d.weekday(), 0 | 6))
        .show(ui);

    ui.add_space(16.0);
    ui.label("Dropdown captions (captionLayout=\"dropdown\")");
    ui.add_space(4.0);
    let mut dropdown = None;
    Calendar::single(&mut dropdown)
        .caption_layout(egui_shadcn_ui::components::calendar::CaptionLayout::Dropdown)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Week numbers (Monday start)");
    ui.add_space(4.0);
    let mut week_numbers = None;
    Calendar::single(&mut week_numbers)
        .week_starts_on(WeekStart::Monday)
        .show_week_numbers(true)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Bounded navigation (Oct 2026 – Feb 2027)");
    ui.add_space(4.0);
    let mut bounded = None;
    Calendar::single(&mut bounded)
        .default_month(CalendarDate::new(2026, 11, 1).unwrap())
        .min_month(CalendarDate::new(2026, 10, 1).unwrap())
        .max_month(CalendarDate::new(2027, 2, 1).unwrap())
        .show(ui);

    ui.ctx().data_mut(|d| d.insert_temp(store, state));
}

pub fn date_picker_demo(ui: &mut egui::Ui) {
    use egui_shadcn_ui::components::calendar::CalendarDate;
    use egui_shadcn_ui::components::date_picker::DatePickerPreset;

    let store = egui::Id::unique("date_picker_demo_values");
    #[derive(Clone, Default)]
    struct DemoState {
        empty: Option<CalendarDate>,
        preset: Option<CalendarDate>,
        presets_picked: Option<CalendarDate>,
        range: Option<(CalendarDate, CalendarDate)>,
    }
    let mut state: DemoState = ui
        .ctx()
        .data_mut(|d| d.get_temp::<DemoState>(store).unwrap_or_default());

    ui.label("Default");
    ui.add_space(4.0);
    let changed = DatePicker::new(&mut state.empty).id_salt("demo").show(ui);
    ui.add_space(4.0);
    ui.label(match state.empty {
        Some(d) => format!("Picked: {}", d.format_ppp()),
        None => "No date picked yet".to_owned(),
    });

    ui.add_space(16.0);
    ui.label("With a preselected date");
    ui.add_space(4.0);
    if state.preset.is_none() {
        state.preset = Some(CalendarDate::new(2026, 11, 10).unwrap());
    }
    DatePicker::new(&mut state.preset)
        .id_salt("demo_preset")
        .show(ui);

    ui.add_space(16.0);
    ui.label("With presets (Select above the calendar)");
    ui.add_space(4.0);
    DatePicker::new(&mut state.presets_picked)
        .id_salt("demo_presets")
        .with_presets([
            DatePickerPreset::new("Today", 0),
            DatePickerPreset::new("Tomorrow", 1),
            DatePickerPreset::new("In 3 days", 3),
            DatePickerPreset::new("In a week", 7),
        ])
        .show(ui);

    ui.add_space(16.0);
    ui.label("Range mode (stays open until both ends are chosen)");
    ui.add_space(4.0);
    DatePicker::range(&mut state.range)
        .id_salt("demo_range")
        .width(300.0)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Disabled");
    ui.add_space(4.0);
    let mut disabled_date = None;
    DatePicker::new(&mut disabled_date)
        .id_salt("demo_disabled")
        .disabled(true)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Monday week start, no outside days, disabled weekends");
    ui.add_space(4.0);
    DatePicker::new(&mut state.empty)
        .id_salt("demo_options")
        .week_starts_on(egui_shadcn_ui::components::calendar::WeekStart::Monday)
        .show_outside_days(false)
        .disabled_when(|d| matches!(d.weekday(), 0 | 6))
        .show(ui);
    let _ = changed;

    ui.ctx().data_mut(|d| d.insert_temp(store, state));
}

pub fn carousel_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("carousel_demo_values");
    let mut selected: usize = ui
        .ctx()
        .data_mut(|d| d.get_temp::<usize>(store).unwrap_or(0));

    ui.label("Horizontal (drag, arrows, ArrowLeft/Right; opt-in dots)");
    ui.add_space(4.0);
    let res = Carousel::new()
        .id_salt("demo")
        .show_dots(true)
        .show(ui, |ui| {
            for i in 0..5 {
                CarouselItem::new().show(ui, |ui| {
                    ui.add_space(4.0);
                    Card::new().show(ui, |ui| {
                        CardContent::new().show(ui, |ui| {
                            ui.allocate_ui(egui::Vec2::new(ui.available_width(), 160.0), |ui| {
                                ui.with_layout(
                                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{}", i + 1))
                                                .size(32.0)
                                                .strong(),
                                        );
                                    },
                                );
                            });
                        });
                    });
                });
            }
        });
    if res.changed {
        selected = res.selected;
    }
    ui.add_space(4.0);
    // The carousel-api demo's indicator: `py-2 text-center text-sm
    // text-muted-foreground`.
    let theme = egui_shadcn_ui::theme::ShadcnTheme::of_ui(ui);
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("Slide {} of 5", selected + 1))
                .size(14.0)
                .color(theme.palette.muted_foreground),
        );
    });

    ui.add_space(16.0);
    ui.label("Size — two slides per view (md:basis-1/2)");
    ui.add_space(4.0);
    Carousel::new()
        .id_salt("demo_half")
        .per_view(2)
        .show(ui, |ui| {
            for i in 0..5 {
                CarouselItem::new().show(ui, |ui| {
                    Card::new().show(ui, |ui| {
                        CardContent::new().show(ui, |ui| {
                            ui.set_min_height(72.0);
                            ui.vertical_centered(|ui| {
                                ui.label(format!("Slide {}", i + 1));
                            });
                        });
                    });
                });
            }
        });

    ui.add_space(16.0);
    ui.label("Size — three slides per view, tight 4 px gutters (carousel-spacing)");
    ui.add_space(4.0);
    Carousel::new()
        .id_salt("demo_third")
        .per_view(3)
        .item_gap(4.0)
        .show(ui, |ui| {
            for i in 0..6 {
                CarouselItem::new().show(ui, |ui| {
                    Card::new().show(ui, |ui| {
                        CardContent::new().show(ui, |ui| {
                            ui.set_min_height(72.0);
                            ui.vertical_centered(|ui| {
                                ui.label(format!("Slide {}", i + 1));
                            });
                        });
                    });
                });
            }
        });

    ui.add_space(16.0);
    ui.label("Vertical (h-[200px] viewport, two slides per view, ArrowUp/ArrowDown)");
    ui.add_space(4.0);
    Carousel::new()
        .id_salt("demo_vertical")
        .orientation(CarouselOrientation::Vertical)
        .viewport_height(200.0)
        .per_view(2)
        .show(ui, |ui| {
            for i in 0..3 {
                CarouselItem::new().show(ui, |ui| {
                    Card::new().show(ui, |ui| {
                        CardContent::new().show(ui, |ui| {
                            ui.set_min_height(72.0);
                            ui.set_min_width(ui.available_width());
                            ui.vertical_centered(|ui| {
                                ui.label(format!("Vertical slide {}", i + 1));
                            });
                        });
                    });
                });
            }
        });

    ui.add_space(16.0);
    ui.label("Without arrows (dots are opt-in and off by default)");
    ui.add_space(4.0);
    Carousel::new()
        .id_salt("demo_plain")
        .show_arrows(false)
        .show(ui, |ui| {
            for i in 0..3 {
                CarouselItem::new().show(ui, |ui| {
                    Card::new().show(ui, |ui| {
                        CardContent::new().show(ui, |ui| {
                            ui.set_min_height(72.0);
                            ui.set_min_width(ui.available_width());
                            ui.vertical_centered(|ui| {
                                ui.label(format!("Plain slide {}", i + 1));
                            });
                        });
                    });
                });
            }
        });

    ui.ctx().data_mut(|d| d.insert_temp(store, selected));
}

pub fn attachment_demo(ui: &mut egui::Ui) {
    ui.label("States");
    ui.add_space(4.0);
    let states: [(&str, AttachmentState, bool, Icon, &str, bool); 5] = [
        (
            "selected-file.pdf",
            AttachmentState::Idle,
            false,
            Icon::Clock,
            "Ready to upload",
            false,
        ),
        (
            "design-system.zip",
            AttachmentState::Uploading,
            true,
            Icon::FileText,
            "Uploading · 64%",
            false,
        ),
        (
            "market-research.pdf",
            AttachmentState::Processing,
            false,
            Icon::FileText,
            "Processing document",
            false,
        ),
        (
            "financial-model.xlsx",
            AttachmentState::Error,
            false,
            Icon::CircleAlert,
            "Upload failed. Try again.",
            true,
        ),
        (
            "uploaded-report.pdf",
            AttachmentState::Done,
            false,
            Icon::CircleCheck,
            "Uploaded · 1.8 MB",
            false,
        ),
    ];
    for (title, state, spin, media_icon, desc, retry) in states {
        let mut removed = false;
        Attachment::new().state(state).show(ui, |ui| {
            AttachmentMedia::new()
                .spinner(spin)
                .icon(media_icon)
                .show(ui);
            AttachmentContent::new().show(ui, |ui| {
                AttachmentTitle::new(title).show(ui);
                AttachmentDescription::new(desc).show(ui);
            });
            AttachmentActions::new().show(ui, |ui| {
                if retry && AttachmentAction::new(Icon::RefreshCw).show(ui).clicked() {
                    ui.ctx().request_repaint();
                }
                if AttachmentAction::new(Icon::X).show(ui).clicked() {
                    removed = true;
                }
            });
        });
        if removed {
            ui.label(format!("{title} — remove clicked"));
        }
        ui.add_space(8.0);
    }

    ui.add_space(12.0);
    ui.label("Sizes");
    ui.add_space(4.0);
    for (size, title) in [
        (AttachmentSize::Default, "Default attachment"),
        (AttachmentSize::Sm, "Small attachment"),
        (AttachmentSize::Xs, "Extra small attachment"),
    ] {
        Attachment::new().size(size).show(ui, |ui| {
            AttachmentMedia::new().icon(Icon::FileText).show(ui);
            AttachmentContent::new().show(ui, |ui| {
                AttachmentTitle::new(title).show(ui);
                AttachmentDescription::new("PDF · 2.4 MB").show(ui);
            });
        });
        ui.add_space(8.0);
    }

    ui.add_space(12.0);
    ui.label("Vertical orientation");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        for (state, title) in [
            (AttachmentState::Uploading, "clip.mp4"),
            (AttachmentState::Error, "broken.png"),
            (AttachmentState::Done, "report.pdf"),
        ] {
            Attachment::new()
                .state(state)
                .orientation(AttachmentOrientation::Vertical)
                .show(ui, |ui| {
                    AttachmentMedia::new()
                        .spinner(state == AttachmentState::Uploading)
                        .icon(if state == AttachmentState::Uploading {
                            Icon::LoaderCircle
                        } else {
                            Icon::FileText
                        })
                        .show(ui);
                    AttachmentContent::new().show(ui, |ui| {
                        AttachmentTitle::new(title).show(ui);
                        if state == AttachmentState::Error {
                            AttachmentDescription::new("Failed").show(ui);
                        }
                    });
                    AttachmentActions::new().show(ui, |ui| {
                        AttachmentAction::new(Icon::X).show(ui);
                    });
                });
        }
    });

    ui.add_space(12.0);
    ui.label("Group (horizontal scroll strip) + trigger");
    ui.add_space(4.0);
    AttachmentGroup::new().show(ui, |ui| {
        for name in [
            "quarterly-report.pdf",
            "budget-2026.xlsx",
            "design-tokens.json",
            "architecture-notes.md",
        ] {
            Attachment::new().trigger(true).show(ui, |ui| {
                AttachmentMedia::new().icon(Icon::FileText).show(ui);
                AttachmentContent::new().show(ui, |ui| {
                    AttachmentTitle::new(name).show(ui);
                    AttachmentDescription::new("Click to open").show(ui);
                });
            });
        }
    });

    ui.add_space(12.0);
    ui.label("Image media + extra-small vertical chip");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        Attachment::new().show(ui, |ui| {
            AttachmentMedia::new().image(card_demo_texture(ui)).show(ui);
            AttachmentContent::new().show(ui, |ui| {
                AttachmentTitle::new("photo.png").show(ui);
                AttachmentDescription::new("Image · 3.1 MB").show(ui);
            });
            AttachmentActions::new().show(ui, |ui| {
                AttachmentAction::new(Icon::X).show(ui);
            });
        });
        Attachment::new()
            .size(AttachmentSize::Xs)
            .orientation(AttachmentOrientation::Vertical)
            .show(ui, |ui| {
                AttachmentMedia::new().icon(Icon::FileText).show(ui);
                AttachmentContent::new().show(ui, |ui| {
                    AttachmentTitle::new("note.md").show(ui);
                });
            });
    });
}
pub fn bubble_demo(ui: &mut egui::Ui) {
    ui.label("Variants");
    ui.add_space(4.0);
    for (variant, label) in [
        (BubbleVariant::Default, "Default (bg-primary)"),
        (BubbleVariant::Secondary, "Secondary"),
        (BubbleVariant::Muted, "Muted"),
        (BubbleVariant::Tinted, "Tinted (OKLCH tint of primary)"),
        (BubbleVariant::Outline, "Outline"),
        (BubbleVariant::Ghost, "Ghost (frameless, full width)"),
        (BubbleVariant::Destructive, "Destructive"),
    ] {
        Bubble::new().variant(variant).show(ui, |ui| {
            BubbleContent::new(format!(
                "{label} — a chat bubble with wrapping text that demonstrates the pill shape."
            ))
            .show(ui);
        });
        ui.add_space(8.0);
    }

    ui.add_space(12.0);
    ui.label("Alignment (start vs end, max-w 80%)");
    ui.add_space(4.0);
    Bubble::new().show(ui, |ui| {
        BubbleContent::new("Start-aligned bubble.").show(ui);
    });
    Bubble::new()
        .align(BubbleAlign::End)
        .variant(BubbleVariant::Secondary)
        .show(ui, |ui| {
            BubbleContent::new("End-aligned bubble.").show(ui);
        });

    ui.add_space(12.0);
    ui.label("Custom content (show_with) — the pill recolors it to the variant's text token");
    ui.add_space(4.0);
    Bubble::new().show(ui, |ui| {
        BubbleContent::new("").show_with(ui, |ui| {
            ui.label(
                egui::RichText::new("Custom content inside the pill —")
                    .size(14.0)
                    .strong(),
            );
            ui.label(
                egui::RichText::new("built-in labels and closures both follow the variant.")
                    .size(14.0),
            );
        });
    });
    ui.add_space(8.0);
    ui.label("End-aligned via the enclosing Message (group-data-[align=end]/message:self-end)");
    ui.add_space(4.0);
    Message::new().align(MessageAlign::End).show(ui, |ui| {
        MessageContent::new().show(ui, |ui| {
            Bubble::new()
                .variant(BubbleVariant::Default)
                .show(ui, |ui| {
                    BubbleContent::new("Right-aligned by the message — no .align(End) needed.")
                        .show(ui);
                });
        });
    });

    ui.add_space(12.0);
    ui.label("Group");
    ui.add_space(4.0);
    BubbleGroup::new().show(ui, |ui| {
        Bubble::new().show(ui, |ui| {
            BubbleContent::new("First message in a group.").show(ui);
        });
        Bubble::new().show(ui, |ui| {
            BubbleContent::new("Second message — same side, 8px gap.").show(ui);
        });
        Bubble::new()
            .variant(BubbleVariant::Muted)
            .align(BubbleAlign::End)
            .show(ui, |ui| {
                BubbleContent::new("Reply from the other side.").show(ui);
            });
    });

    ui.add_space(12.0);
    ui.label("Reactions (bottom/end default, top/start variant)");
    ui.add_space(4.0);
    Bubble::new().variant(BubbleVariant::Muted).show(ui, |ui| {
        BubbleContent::new("Hover me in Slack, apparently.").show(ui);
        BubbleReactions::new("👍 3").show(ui);
    });
    ui.add_space(16.0);
    Bubble::new().show(ui, |ui| {
        BubbleContent::new("Reaction on the top edge.").show(ui);
        BubbleReactions::new("🎉")
            .side(BubbleReactionSide::Top)
            .align(BubbleReactionAlign::Start)
            .show(ui);
    });
}
pub fn message_demo(ui: &mut egui::Ui) {
    ui.label("Grouped transcript (avatars bottom-aligned, headers/footers)");
    ui.add_space(4.0);
    MessageGroup::new().show(ui, |ui| {
        Message::new().show(ui, |ui| {
            MessageAvatar::new().show(ui, |ui| {
                Avatar::new().fallback("RB").show(ui);
            });
            MessageContent::new().show(ui, |ui| {
                MessageHeader::new("Rabbit").show(ui);
                Bubble::new().variant(BubbleVariant::Muted).show(ui, |ui| {
                    BubbleContent::new("It's 4:55 PM. On a Friday.").show(ui);
                });
                MessageFooter::new("9:55 AM").show(ui);
            });
        });
        Message::new().align(MessageAlign::End).show(ui, |ui| {
            MessageAvatar::new()
                .raised(true) // footer below → -translate-y-8
                .show(ui, |ui| {
                    Avatar::new().fallback("ME").show(ui);
                });
            MessageContent::new().show(ui, |ui| {
                // Header stays left-aligned in end-aligned messages.
                MessageHeader::new("Me").show(ui);
                Bubble::new().show(ui, |ui| {
                    BubbleContent::new("Deploying to prod real quick.").show(ui);
                });
                MessageFooter::new("Delivered").show(ui);
            });
        });
        Message::new().show(ui, |ui| {
            MessageAvatar::new().show(ui, |ui| {
                Avatar::new().fallback("RB").show(ui);
            });
            MessageContent::new().show(ui, |ui| {
                Bubble::new().variant(BubbleVariant::Muted).show(ui, |ui| {
                    BubbleContent::new("It's always a one-line change.").show(ui);
                });
                Bubble::new().variant(BubbleVariant::Muted).show(ui, |ui| {
                    BubbleContent::new("Alright, let me take a look.").show(ui);
                });
            });
        });
        Message::new().show(ui, |ui| {
            MessageAvatar::new().show(ui, |ui| {
                Avatar::new().fallback_icon(Icon::User).show(ui);
            });
            MessageContent::new().show(ui, |ui| {
                Bubble::new().variant(BubbleVariant::Ghost).show(ui, |ui| {
                    BubbleContent::new(
                        "Ghost bubbles take the full width with no frame and no padding.",
                    )
                    .show(ui);
                });
                MessageFooter::new("flush footer (group-has-ghost:px-0)")
                    .flush(true)
                    .show(ui);
            });
        });
    });
    ui.add_space(8.0);
    MessageHeader::new("flush header (group-has-ghost:px-0)")
        .flush(true)
        .show(ui);
}
pub fn message_scroller_demo(ui: &mut egui::Ui) {
    ui.label("Auto-follow chat viewport (scroll up to see the jump button)");
    ui.add_space(4.0);
    let store = egui::Id::unique("demo_scroller_transcript");
    let mut transcript: Vec<(bool, String)> =
        ui.ctx().data_mut(|d| d.get_temp(store)).unwrap_or_else(|| {
            (0..8)
                .map(|i| {
                    (
                        i % 2 == 1,
                        format!("Seeded message {i} — padding the log so it overflows."),
                    )
                })
                .collect()
        });

    ui.horizontal(|ui| {
        if Button::new("Append user message").show(ui).clicked() {
            transcript.push((true, format!("New user message #{}", transcript.len())));
        }
        if Button::new("Append reply")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            transcript.push((
                false,
                format!(
                    "Auto-scroll follows only while you're pinned to the bottom (#{}).",
                    transcript.len()
                ),
            ));
        }
        if Button::new("Reset")
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            transcript.truncate(8);
        }
    });
    ui.add_space(8.0);

    let scroller = MessageScroller::new()
        .height(320.0)
        .id_salt("demo_scroller")
        .show(ui, |ui| {
            for (is_user, text) in &transcript {
                Message::new()
                    .align(if *is_user {
                        MessageAlign::End
                    } else {
                        MessageAlign::Start
                    })
                    .show(ui, |ui| {
                        MessageAvatar::new().show(ui, |ui| {
                            Avatar::new()
                                .fallback(if *is_user { "ME" } else { "RB" })
                                .show(ui);
                        });
                        MessageContent::new().show(ui, |ui| {
                            Bubble::new()
                                .variant(if *is_user {
                                    BubbleVariant::Default
                                } else {
                                    BubbleVariant::Muted
                                })
                                .show(ui, |ui| {
                                    BubbleContent::new(text.clone()).show(ui);
                                });
                        });
                    });
            }
        });
    let resp = scroller.inner;
    ui.add_space(4.0);
    ui.label(format!(
        "at_bottom: {} · at_top: {} · auto-follow: {}",
        resp.at_bottom, resp.at_top, resp.following
    ));

    ui.add_space(16.0);
    ui.label(
        "Second instance, jump-to-start button (direction=Start — hover it, then scroll down)",
    );
    ui.add_space(4.0);
    let start_resp = MessageScroller::new()
        .height(160.0)
        .direction(MessageScrollerDirection::Start)
        .id_salt("demo_scroller_start")
        .show(ui, |ui| {
            for i in 0..12 {
                Message::new().show(ui, |ui| {
                    MessageContent::new().show(ui, |ui| {
                        Bubble::new().variant(BubbleVariant::Muted).show(ui, |ui| {
                            BubbleContent::new(format!("Start-direction scroller message {i}."))
                                .show(ui);
                        });
                    });
                });
            }
        })
        .inner;
    ui.add_space(4.0);
    ui.label(format!(
        "at_top: {} · at_bottom: {}",
        start_resp.at_top, start_resp.at_bottom
    ));

    ui.ctx().data_mut(|d| d.insert_temp(store, transcript));
}
pub fn direction_demo(ui: &mut egui::Ui) {
    ui.label("LTR (default)");
    ui.add_space(4.0);
    DirectionProvider::ltr().show(ui, |ui| {
        ui.label("Layout flows left to right and text hugs the left edge.");
        ui.with_layout(row_layout(ui), |ui| {
            Button::new("First").show(ui);
            Button::new("Second").show(ui);
            Button::new("Third").show(ui);
        });
        ui.label(format!("current direction: {:?}", current(ui)));
    });

    ui.add_space(16.0);
    ui.label("RTL (scoped flip: right-aligned text + right-to-left rows)");
    ui.add_space(4.0);
    DirectionProvider::rtl().show(ui, |ui| {
        ui.label("النص يتدفق من اليمين إلى اليسار ومحاذاته إلى اليمين.");
        ui.with_layout(row_layout(ui), |ui| {
            Button::new("الأول").show(ui);
            Button::new("الثاني").show(ui);
            Button::new("الثالث").show(ui);
        });
        ui.label(format!("current direction: {:?}", current(ui)));
    });

    ui.add_space(16.0);
    ui.label("Convenience form (Direction::rtl)");
    ui.add_space(4.0);
    Direction::rtl(ui, |ui| {
        ui.label("مرحبا — scoped via Direction::rtl");
    });

    ui.add_space(16.0);
    ui.label("Nesting (RTL scope containing an LTR island)");
    ui.add_space(4.0);
    DirectionProvider::rtl().show(ui, |ui| {
        ui.label("يمين");
        DirectionProvider::ltr().show(ui, |ui| {
            ui.label("Inner island back to LTR");
            ui.label(format!("current: {:?}", current(ui)));
        });
        ui.label("وإلى اليمين مرة أخرى");
    });

    ui.add_space(16.0);
    ui.label("RTL popups (dropdown-menu: Start hugs the right edge, sub-menus open left)");
    ui.add_space(4.0);
    DirectionProvider::rtl().show(ui, |ui| {
        DropdownMenu::new("القائمة")
            .align(MenuAlign::Start)
            .show(ui, |m| {
                m.item(MenuItem::new("الأول"));
                m.item(MenuItem::new("الثاني"));
                m.sub_menu("فرعي", |s| {
                    s.item(MenuItem::new("داخلي"));
                });
            });
    });
}
