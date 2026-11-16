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
    let store = egui::Id::unique("input_demo_values");
    let mut values: Vec<String> = ui.ctx().data_mut(|d| {
        d.get_temp::<Vec<String>>(store)
            .unwrap_or_else(|| vec![String::new(); 6])
    });

    ui.label("Default (w-full)");
    ui.add_space(4.0);
    Input::new(&mut values[0]).show(ui);

    ui.add_space(12.0);
    ui.label("With placeholder and label");
    ui.add_space(4.0);
    Label::new("Email").show(ui);
    Input::new(&mut values[1])
        .placeholder("m@example.com")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Password");
    ui.add_space(4.0);
    Input::new(&mut values[2])
        .password(true)
        .width(280.0)
        .show(ui);

    ui.add_space(12.0);
    ui.label("Fixed width");
    ui.add_space(4.0);
    Input::new(&mut values[3]).width(240.0).show(ui);

    ui.add_space(12.0);
    ui.label("Disabled");
    ui.add_space(4.0);
    Input::new(&mut values[4])
        .disabled(true)
        .width(240.0)
        .placeholder("Disabled field")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Invalid (aria-invalid)");
    ui.add_space(4.0);
    Input::new(&mut values[5])
        .invalid(true)
        .width(240.0)
        .placeholder("Invalid email")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Fused inside a ButtonGroup");
    ui.add_space(4.0);
    ButtonGroup::new().show(ui, |ui| {
        Input::new(&mut values[0]).width(200.0).show(ui);
        Button::new("Subscribe").show(ui);
    });

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn textarea_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("textarea_demo_values");
    let mut values: Vec<String> = ui.ctx().data_mut(|d| {
        d.get_temp::<Vec<String>>(store)
            .unwrap_or_else(|| vec![String::new(); 4])
    });

    ui.label("Default (min-h-16, w-full)");
    ui.add_space(4.0);
    Textarea::new(&mut values[0]).show(ui);

    ui.add_space(12.0);
    ui.label("With placeholder");
    ui.add_space(4.0);
    Label::new("Your message").show(ui);
    Textarea::new(&mut values[1])
        .placeholder("Type your message here.")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Rows (grows with content)");
    ui.add_space(4.0);
    Textarea::new(&mut values[2]).rows(5).width(360.0).show(ui);

    ui.add_space(12.0);
    ui.label("States");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Textarea::new(&mut values[3])
            .disabled(true)
            .width(200.0)
            .placeholder("Disabled")
            .show(ui);
        Textarea::new(&mut values[3])
            .invalid(true)
            .width(200.0)
            .placeholder("Invalid")
            .show(ui);
    });

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn checkbox_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("checkbox_demo_values");
    let mut values: [bool; 6] = ui.ctx().data_mut(|d| {
        d.get_temp(store)
            .unwrap_or([true, false, false, false, false, false])
    });

    ui.label("Default");
    ui.add_space(4.0);
    Checkbox::new(&mut values[0]).id_salt("cb_demo_1").show(ui);

    ui.add_space(12.0);
    ui.label("With a bound label (click the text to toggle)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let id = egui::Id::unique("cb_demo_2");
        Checkbox::new(&mut values[1]).id_salt("cb_demo_2").show(ui);
        Label::new("Enable notifications").for_id(id).show(ui);
    });

    ui.add_space(12.0);
    ui.label("Indeterminate");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Checkbox::new(&mut values[2])
            .indeterminate(true)
            .id_salt("cb_demo_3")
            .show(ui);
        ui.label("Select all (mixed state)");
    });

    ui.add_space(12.0);
    ui.label("Disabled");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Checkbox::new(&mut values[3]).disabled(true).show(ui);
        Checkbox::new(&mut values[4])
            .disabled(true)
            .id_salt("cb_demo_5")
            .show(ui);
        ui.label("Disabled unchecked");
    });

    ui.add_space(12.0);
    ui.label("Invalid");
    ui.add_space(4.0);
    Checkbox::new(&mut values[5]).invalid(true).show(ui);

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a checkbox, press Space or Enter to toggle.");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn radio_group_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("radio_demo_values");
    let mut values: [Option<usize>; 5] = ui.ctx().data_mut(|d| {
        d.get_temp(store)
            .unwrap_or([Some(0), None, Some(1), Some(0), Some(0)])
    });

    ui.label("Vertical (default)");
    ui.add_space(4.0);
    RadioGroup::new(&mut values[0])
        .id_salt("radio_demo_default")
        .option("Default")
        .option("Comfortable")
        .option("Compact")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Horizontal");
    ui.add_space(4.0);
    RadioGroup::new(&mut values[2])
        .id_salt("radio_demo_horizontal")
        .orientation(GroupOrientation::Horizontal)
        .option("Day")
        .option("Month")
        .option("Year")
        .show(ui);

    ui.add_space(12.0);
    ui.label("No initial selection, one disabled option");
    ui.add_space(4.0);
    RadioGroup::new(&mut values[1])
        .id_salt("radio_demo_disabled")
        .option("Push notifications")
        .option_disabled("Email digest (unavailable)")
        .option("Nothing")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Disabled group and invalid items");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        RadioGroup::new(&mut values[3])
            .id_salt("radio_demo_disabled_group")
            .disabled(true)
            .option("A")
            .option("B")
            .show(ui);
        RadioGroup::new(&mut values[4])
            .id_salt("radio_demo_invalid")
            .invalid(true)
            .option("A")
            .option("B")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Standalone RadioGroupItem + label");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let mut standalone = ui.ctx().data_mut(|d| {
            d.get_temp(egui::Id::unique("radio_demo_standalone"))
                .unwrap_or(false)
        });
        RadioGroupItem::new(&mut standalone)
            .id_salt("radio_demo_item")
            .show(ui);
        ui.label("Standalone item");
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::unique("radio_demo_standalone"), standalone));
    });

    ui.add_space(12.0);
    ui.label("Keyboard: arrows move the selection and focus (no wrap-around).");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
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
    let store = egui::Id::unique("slider_demo_values");
    let mut values: (f32, f32, f32, f32, f32, f32) = ui.ctx().data_mut(|d| {
        d.get_temp(store)
            .unwrap_or((30.0, 20.0, 80.0, 25.0, 50.0, 60.0))
    });

    ui.label("Single value (drag the thumb, click the track, use Arrow keys)");
    ui.add_space(4.0);
    let (v0, _, _, _, _, _) = values;
    let mut single = v0;
    let resp = Slider::new(&mut single).min(0.0).max(100.0).show(ui);
    values.0 = single;
    if resp.changed() {
        ui.ctx().request_repaint();
    }
    ui.small(format!("value: {:.0}", values.0));

    ui.add_space(12.0);
    ui.label("Range (two thumbs)");
    ui.add_space(4.0);
    let (a, b, _, _, _, _) = values;
    let mut pair = (a, b);
    Slider::range(&mut pair).min(0.0).max(100.0).show(ui);
    values.1 = pair.0;
    values.2 = pair.1;
    ui.small(format!("range: {:.0} – {:.0}", pair.0, pair.1));

    ui.add_space(12.0);
    ui.label("Stepped (step = 10)");
    ui.add_space(4.0);
    let (_, _, _, c, _, _) = values;
    let mut stepped = c;
    Slider::new(&mut stepped)
        .min(0.0)
        .max(100.0)
        .step(10.0)
        .show(ui);
    values.3 = stepped;
    ui.small(format!("value: {:.0}", stepped));

    ui.add_space(12.0);
    ui.label("Disabled");
    ui.add_space(4.0);
    let (_, _, _, _, e, _) = values;
    let mut frozen = e;
    Slider::new(&mut frozen)
        .min(0.0)
        .max(100.0)
        .disabled(true)
        .show(ui);
    values.4 = frozen;

    ui.add_space(12.0);
    ui.label("Vertical (Arrow Up/Down move the thumb)");
    ui.add_space(4.0);
    let (_, _, _, _, _, f) = values;
    let mut vertical = f;
    ui.horizontal(|ui| {
        Slider::new(&mut vertical)
            .min(0.0)
            .max(100.0)
            .orientation(GroupOrientation::Vertical)
            .show(ui);
        let mut pair = (values.1, values.2);
        Slider::range(&mut pair)
            .min(0.0)
            .max(100.0)
            .orientation(GroupOrientation::Vertical)
            .show(ui);
        values.1 = pair.0;
        values.2 = pair.1;
    });
    values.5 = vertical;

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a thumb, use Arrow keys, Home and End.");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn toggle_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("toggle_demo_values");
    let mut values: [bool; 3] = ui
        .ctx()
        .data_mut(|d| d.get_temp(store).unwrap_or([true, false, false]));

    ui.label("Default variant (on / off)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Toggle::new(&mut values[0]).text("Bold").show(ui);
        Toggle::new(&mut values[1]).text("Italic").show(ui);
        Toggle::new(&mut values[2]).text("Underline").show(ui);
    });

    ui.add_space(12.0);
    ui.label("Outline variant");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Toggle::new(&mut values[0])
            .variant(ToggleVariant::Outline)
            .text("Bold")
            .show(ui);
        Toggle::new(&mut values[1])
            .variant(ToggleVariant::Outline)
            .text("Italic")
            .show(ui);
        Toggle::new(&mut values[2])
            .variant(ToggleVariant::Outline)
            .text("Underline")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Sizes");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Toggle::new(&mut values[0])
            .size(ToggleSize::Sm)
            .text("Small")
            .show(ui);
        Toggle::new(&mut values[0]).text("Default").show(ui);
        Toggle::new(&mut values[0])
            .size(ToggleSize::Lg)
            .text("Large")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("With icons");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Toggle::new(&mut values[0])
            .variant(ToggleVariant::Outline)
            .icon(Icon::Search)
            .show(ui);
        Toggle::new(&mut values[1])
            .icon(Icon::Sparkles)
            .text("Starred")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Disabled and invalid");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Toggle::new(&mut values[2])
            .disabled(true)
            .text("Disabled")
            .show(ui);
        Toggle::new(&mut values[2])
            .variant(ToggleVariant::Outline)
            .disabled(true)
            .text("Disabled outline")
            .show(ui);
        Toggle::new(&mut values[2])
            .variant(ToggleVariant::Outline)
            .invalid(true)
            .text("Invalid")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a toggle, press Space or Enter to press it.");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn toggle_group_demo(ui: &mut egui::Ui) {
    let single_store = egui::Id::unique("toggle_group_demo_single");
    let multi_store = egui::Id::unique("toggle_group_demo_multi");
    let mut single: Option<usize> = ui
        .ctx()
        .data_mut(|d| d.get_temp(single_store).unwrap_or(None));
    let mut multi: Vec<bool> = ui
        .ctx()
        .data_mut(|d| d.get_temp(multi_store).unwrap_or(vec![false; 3]));

    ui.label("Single (default: pressing the pressed item deselects)");
    ui.add_space(4.0);
    ToggleGroup::single(&mut single)
        .id_salt("tg_demo_single")
        .item("Bold")
        .item("Italic")
        .item("Underline")
        .show(ui);
    ui.small(format!("selected: {:?}", single));

    ui.add_space(12.0);
    ui.label("Multiple, outline variant");
    ui.add_space(4.0);
    ToggleGroup::multiple(&mut multi)
        .id_salt("tg_demo_multi")
        .variant(ToggleVariant::Outline)
        .item("Bold")
        .item("Italic")
        .item("Underline")
        .show(ui);
    ui.small(format!("pressed: {:?}", multi));

    ui.add_space(12.0);
    ui.label("With icons and sizes");
    ui.add_space(4.0);
    ToggleGroup::single(&mut single)
        .id_salt("tg_demo_icons")
        .item_with_icon("Left", Icon::ChevronLeft)
        .item("Center")
        .item_with_icon("Right", Icon::ChevronRight)
        .size(ToggleSize::Sm)
        .show(ui);
    ui.add_space(4.0);
    ToggleGroup::single(&mut single)
        .id_salt("tg_demo_lg")
        .item("Day")
        .item("Month")
        .item("Year")
        .size(ToggleSize::Lg)
        .show(ui);

    ui.add_space(12.0);
    ui.label("Positive spacing (individually rounded)");
    ui.add_space(4.0);
    ToggleGroup::multiple(&mut multi)
        .id_salt("tg_demo_spaced")
        .spacing(8.0)
        .variant(ToggleVariant::Outline)
        .item("Filter A")
        .item("Filter B")
        .item("Filter C")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Vertical, disabled");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ToggleGroup::single(&mut single)
            .id_salt("tg_demo_vertical")
            .orientation(GroupOrientation::Vertical)
            .item("Top")
            .item("Middle")
            .item("Bottom")
            .show(ui);
        ToggleGroup::single(&mut single)
            .id_salt("tg_demo_disabled")
            .disabled(true)
            .item("Cut")
            .item("Copy")
            .item("Paste")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Keyboard: Tab and Arrow keys move focus between items.");

    ui.ctx().data_mut(|d| {
        d.insert_temp(single_store, single);
        d.insert_temp(multi_store, multi);
    });
}

pub fn input_otp_demo(ui: &mut egui::Ui) {
    let simple_store = egui::Id::unique("otp_demo_simple");
    let custom_store = egui::Id::unique("otp_demo_custom");
    let disabled_store = egui::Id::unique("otp_demo_disabled");
    let mut code: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(simple_store).unwrap_or_default());
    let mut custom: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(custom_store).unwrap_or_default());
    let mut frozen: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(disabled_store).unwrap_or_default());

    ui.label("Default (single group, numeric)");
    ui.add_space(4.0);
    let out = InputOTP::new(&mut code)
        .length(6)
        .numeric(true)
        .id_salt("otp_demo_simple")
        .show(ui);
    if out.changed {
        ui.ctx().request_repaint();
    }
    ui.small(format!("code: {:?}", code));

    ui.add_space(12.0);
    ui.label("Pattern with separator (3+3)");
    ui.add_space(4.0);
    InputOTP::new(&mut custom)
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

    ui.add_space(12.0);
    ui.label("Disabled and invalid");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        InputOTP::new(&mut frozen)
            .length(4)
            .numeric(true)
            .disabled(true)
            .id_salt("otp_demo_disabled")
            .show(ui);
        InputOTP::new(&mut frozen)
            .length(4)
            .numeric(true)
            .invalid(true)
            .id_salt("otp_demo_invalid")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label(
        "Keyboard: digits fill the focused slot, Backspace clears, Arrow keys move, paste spreads.",
    );

    ui.ctx().data_mut(|d| {
        d.insert_temp(simple_store, code);
        d.insert_temp(custom_store, custom);
        d.insert_temp(disabled_store, frozen);
    });
}

pub fn native_select_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("native_select_demo_values");
    let mut values: [Option<usize>; 3] = ui
        .ctx()
        .data_mut(|d| d.get_temp(store).unwrap_or([None, Some(0), Some(1)]));

    ui.label("Default");
    ui.add_space(4.0);
    NativeSelect::new(&mut values[0])
        .id_salt("ns_demo_1")
        .options(["Light", "Dark", "System"])
        .placeholder("Pick a theme")
        .show(ui);
    ui.small(format!("selected: {:?}", values[0]));

    ui.add_space(12.0);
    ui.label("Small with a selection");
    ui.add_space(4.0);
    NativeSelect::new(&mut values[2])
        .id_salt("ns_demo_2")
        .size(NativeSelectSize::Sm)
        .width(240.0)
        .option("engine")
        .option("web")
        .option("native")
        .show(ui);

    ui.add_space(12.0);
    ui.label("Disabled and invalid");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        NativeSelect::new(&mut values[1])
            .id_salt("ns_demo_disabled")
            .disabled(true)
            .options(["Light", "Dark"])
            .show(ui);
        NativeSelect::new(&mut values[1])
            .id_salt("ns_demo_invalid")
            .invalid(true)
            .options(["Light", "Dark"])
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Options with one disabled");

    ui.add_space(4.0);
    NativeSelect::new(&mut values[2])
        .id_salt("ns_demo_disabled_opt")
        .width(220.0)
        .option("Push notifications")
        .option_disabled("Email digest (unavailable)")
        .option("Nothing")
        .show(ui);

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn field_demo(ui: &mut egui::Ui) {
    let notif_store = egui::Id::unique("field_demo_notif");
    let email_store = egui::Id::unique("field_demo_email");
    let name_store = egui::Id::unique("field_demo_name");
    let plan_store = egui::Id::unique("field_demo_plan");
    let mut notifications: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(notif_store).unwrap_or(false));
    let mut email: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(email_store).unwrap_or_default());
    let mut name: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(name_store).unwrap_or_default());
    let mut plan: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(plan_store).unwrap_or(true));

    FieldSet::new().show(ui, |ui| {
        FieldLegend::new("Notification settings").show(ui);
        FieldDescription::new("Choose how you want to be contacted. These apply to all projects.")
            .show(ui);

        FieldGroup::new().show(ui, |ui| {
            Field::new().show(ui, |ui| {
                let id = egui::Id::unique("field_demo_notif");
                FieldLabel::new("Push notifications").for_id(id).show(ui);
                FieldDescription::new("Send notifications to device.").show(ui);
                Checkbox::new(&mut notifications)
                    .id_salt("field_demo_notif")
                    .show(ui);
            });

            Field::new().show(ui, |ui| {
                FieldLabel::new("Email").show(ui);
                FieldDescription::new("We use your email for the digest only.").show(ui);
                Input::new(&mut email).placeholder("m@example.com").show(ui);
            });

            Field::new().invalid(true).show(ui, |ui| {
                FieldLabel::new("Name").show(ui);
                Input::new(&mut name).invalid(true).show(ui);
                FieldError::new("Your display name cannot be empty.").show(ui);
            });
        });

        FieldSeparator::content("or continue with").show(ui);

        Field::new().show(ui, |ui| {
            FieldLabel::new("Billing plan").show(ui);
            FieldContent::new().show(ui, |ui| {
                FieldDescription::new("Updates your plan to Pro.").show(ui);
                FieldError::errors(vec![
                    "Card declined".to_owned(),
                    "Insufficient funds".to_owned(),
                ])
                .show(ui);
            });
        });
    });

    ui.add_space(12.0);
    ui.label("Horizontal orientation (label beside the control)");
    ui.add_space(4.0);
    Field::new()
        .orientation(FieldOrientation::Horizontal)
        .show(ui, |ui| {
            FieldLabel::new("Enable notifications").show(ui);
            Switch::new(&mut notifications)
                .id_salt("field_demo_switch")
                .show(ui);
        });

    ui.add_space(12.0);
    ui.label("Field title with icon");
    ui.add_space(4.0);
    Field::new().show(ui, |ui| {
        FieldTitle::new("Account").icon(Icon::User).show(ui);
        FieldDescription::new("Manage your account details.").show(ui);
    });

    ui.add_space(12.0);
    ui.label("Responsive orientation (horizontal when the container is wide)");
    ui.add_space(4.0);
    Field::new()
        .orientation(FieldOrientation::Responsive)
        .show(ui, |ui| {
            FieldLabel::new("Compact density").show(ui);
            Switch::new(&mut notifications)
                .id_salt("field_demo_switch_responsive")
                .show(ui);
        });

    ui.add_space(12.0);
    ui.label("Choice card (FieldLabel wrapping a nested Field)");
    ui.add_space(4.0);
    FieldLabel::new("")
        .checked(Some(plan))
        .show_with_card(ui, |ui| {
            Field::new()
                .orientation(FieldOrientation::Horizontal)
                .show(ui, |ui| {
                    Checkbox::new(&mut plan).id_salt("field_demo_plan").show(ui);
                    FieldContent::new().show(ui, |ui| {
                        FieldTitle::new("Enable notifications").show(ui);
                        FieldDescription::new(
                            "You can enable or disable notifications at any time.",
                        )
                        .show(ui);
                    });
                });
        });

    ui.add_space(12.0);
    ui.label("Legend variants");
    ui.add_space(4.0);
    FieldSet::new().show(ui, |ui| {
        FieldLegend::new("Section").show(ui);
        FieldLegend::new("Sub-section")
            .variant(FieldLegendVariant::Label)
            .show(ui);
        FieldDescription::new("The label variant renders at the smaller text size.").show(ui);
    });

    ui.add_space(12.0);
    ui.label("Disabled field (label dims to 50%)");
    ui.add_space(4.0);
    Field::new().disabled(true).show(ui, |ui| {
        FieldLabel::new("Workspace URL").show(ui);
        Input::new(&mut email).disabled(true).show(ui);
    });

    ui.ctx().data_mut(|d| {
        d.insert_temp(notif_store, notifications);
        d.insert_temp(email_store, email);
        d.insert_temp(name_store, name);
        d.insert_temp(plan_store, plan);
    });
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

    ui.label("Inline start text addon + inline end icon button");
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
    ui.label("Icon addon and kbd hint");
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
    ui.label("Textarea control (auto-height column layout)");
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
    ui.label("Invalid control (container border turns destructive)");
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
    ui.label("Disabled");
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
    ui.label("Button sizes (xs chip and sm action)");
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

    ui.add_space(12.0);
    ui.label("Fused inside a ButtonGroup");
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

    ui.ctx().data_mut(|d| {
        d.insert_temp(url_store, url);
        d.insert_temp(search_store, search);
        d.insert_temp(bio_store, bio);
        d.insert_temp(invalid_store, bad);
    });
}

pub fn card_demo(ui: &mut egui::Ui) {
    ui.label("Default (display-only card)");
    ui.add_space(4.0);
    Card::new().show(ui, |ui| {
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

    ui.add_space(16.0);
    ui.label("With a header action (top-right slot)");
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
    ui.label("Destructive action in a card");
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
    ui.label("Compact (size=sm) card");
    ui.add_space(4.0);
    Card::new().size(CardSize::Sm).show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Compact card").show(ui);
            CardDescription::new("Tighter spacing and a smaller title.").show(ui);
        });
        CardContent::new().show(ui, |ui| {
            ui.label("Nova shrinks --card-spacing to 12px and the title to 14px.");
        });
    });

    ui.add_space(16.0);
    ui.label("Card with a top image slot");
    ui.add_space(4.0);
    Card::new().image_top(card_demo_texture(ui)).show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Landscape").show(ui);
            CardDescription::new("The image bleeds to the rounded top corners.").show(ui);
        });
        CardContent::new().show(ui, |ui| {
            ui.label("The media slot removes the card's top padding.");
        });
    });
}

/// A generated gradient texture for the card demo's image slot (cached in
/// temp storage like the avatar demo's).
fn card_demo_texture(ui: &mut egui::Ui) -> egui::ImageSource<'static> {
    let tex_store = egui::Id::unique("card_demo_texture");
    let tex_id = ui
        .ctx()
        .data_mut(|d| d.get_temp::<egui::TextureHandle>(tex_store));
    let texture = tex_id.unwrap_or_else(|| {
        let size = 128;
        let mut pixels = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let t = (x as f32 / size as f32) * 0.6 + (y as f32 / size as f32) * 0.4;
                let r = (40.0 + 160.0 * t) as u8;
                let g = (70.0 + 60.0 * (1.0 - t)) as u8;
                let b = (180.0 + 60.0 * t) as u8;
                pixels.push(egui::Color32::from_rgb(r, g, b));
            }
        }
        let image = egui::ColorImage::new([size, size], pixels);
        ui.ctx()
            .load_texture("card_demo_gradient", image, egui::TextureOptions::default())
    });
    egui::ImageSource::Texture(egui::load::SizedTexture::new(
        texture.id(),
        texture.size_vec2(),
    ))
}

pub fn alert_demo(ui: &mut egui::Ui) {
    ui.label("Default (no icon)");
    ui.add_space(4.0);
    Alert::new().id_salt("alert_demo_plain").show(ui, |ui| {
        AlertTitle::new("Heads up!").show(ui);
        AlertDescription::new("You can add components and dependencies to your app using the cli.")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Destructive with icon");
    ui.add_space(4.0);
    Alert::new()
        .id_salt("alert_demo_destructive")
        .variant(AlertVariant::Destructive)
        .icon(Icon::CircleAlert)
        .show(ui, |ui| {
            AlertTitle::new("Deployment failed").show(ui);
            AlertDescription::new(
                "Your build could not be completed. Check the logs and try again.",
            )
            .show(ui);
        });

    ui.add_space(12.0);
    ui.label("Success and info icons");
    ui.add_space(4.0);
    // These use the default variant: the source's `[&>svg]:text-current`
    // keeps icons foreground-colored unless the variant overrides it, so do
    // not recolor them per semantic.
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

    ui.add_space(12.0);
    ui.label("With an action (top-right slot)");
    ui.add_space(4.0);
    Alert::new()
        .id_salt("alert_demo_action")
        .icon(Icon::CircleAlert)
        .show(ui, |ui| {
            AlertTitle::new("Token expiring").show(ui);
            AlertDescription::new("Your API token expires in 3 days.").show(ui);
            AlertAction::new().show(ui, |ui| {
                Button::new("Renew").size(ButtonSize::Sm).show(ui);
            });
        });
    ui.add_space(4.0);
    Alert::new()
        .id_salt("alert_demo_action_destructive")
        .variant(AlertVariant::Destructive)
        .icon(Icon::CircleX)
        .show(ui, |ui| {
            AlertTitle::new("Storage quota exceeded").show(ui);
            AlertDescription::new("Delete some files to keep syncing.").show(ui);
            AlertAction::new().show(ui, |ui| {
                Button::new("Free up space")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Destructive)
                    .show(ui);
            });
        });
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

    ui.label("Sizes with fallback initials");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Avatar::new().size(AvatarSize::Sm).fallback("CN").show(ui);
        Avatar::new().fallback("CN").show(ui);
        Avatar::new().size(AvatarSize::Lg).fallback("CN").show(ui);
    });

    ui.add_space(12.0);
    ui.label("With an image (texture), icon fallback and badges");
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

    ui.add_space(12.0);
    ui.label("Small badges hide their icon");
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
    });

    ui.add_space(12.0);
    ui.label("Group (-space-x-2 overlap with separation rings)");
    ui.add_space(4.0);
    AvatarGroup::new().show(ui, |ui| {
        Avatar::new().image(src.clone()).show(ui);
        Avatar::new().fallback("AB").show(ui);
        Avatar::new().fallback("CD").show(ui);
        Avatar::new().fallback_icon(Icon::User).show(ui);
        AvatarGroupCount::new("+2").show(ui);
    });

    ui.add_space(8.0);
    ui.label("Small group");
    ui.add_space(4.0);
    AvatarGroup::new().size(AvatarSize::Sm).show(ui, |ui| {
        Avatar::new().fallback("EF").show(ui);
        Avatar::new().fallback("GH").show(ui);
        AvatarGroupCount::new("+9").show(ui);
    });

    ui.add_space(8.0);
    ui.label("Large group with icons");
    ui.add_space(4.0);
    AvatarGroup::new().size(AvatarSize::Lg).show(ui, |ui| {
        Avatar::new().image(src.clone()).show(ui);
        Avatar::new().fallback("IJ").show(ui);
        AvatarGroupCount::new("")
            .icon(Icon::MoreHorizontal)
            .show(ui);
    });

    ui.add_space(8.0);
    ui.small("URI images load through the context's texture loaders and fall back to initials until resolved.");
}

pub fn progress_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("progress_demo_value");
    let mut value: f32 = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or(30.0));

    ui.label("Default");
    ui.add_space(4.0);
    let resp = ui
        .horizontal(|ui| {
            let r = Progress::new(value).show(ui);
            ui.small(format!("{:.0}%", value));
            r
        })
        .inner;
    let _ = resp;

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
    ui.label("Values");
    ui.add_space(4.0);
    for v in [0.0, 25.0, 50.0, 75.0, 100.0] {
        ui.horizontal(|ui| {
            Progress::new(v).show(ui);
            ui.small(format!("{:.0}%", v));
        });
    }

    ui.add_space(12.0);
    ui.label("Clamped and fixed width");
    ui.add_space(4.0);
    Progress::new(250.0).width(200.0).show(ui);
    Progress::new(-5.0).width(200.0).show(ui);
    ui.add_space(4.0);
    Progress::new(40.0).width(120.0).show(ui);

    ui.add_space(12.0);
    ui.label("The fill animates over 150ms (shadcn transition-all).");

    ui.ctx().data_mut(|d| d.insert_temp(store, value));
}

pub fn table_demo(ui: &mut egui::Ui) {
    ui.label("Invoices table (hover rows, one selected row, footer totals)");
    ui.add_space(4.0);
    Table::new().show(ui, |ui| {
        TableHeader::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                TableHead::new("Invoice").show(ui);
                TableHead::new("Status").width(110.0).show(ui);
                TableHead::new("Method").width(130.0).show(ui);
                TableHead::new("Amount").width(90.0).show(ui);
            });
        });
        TableBody::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                TableCell::text("INV-0041").show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Paid").show(ui);
                });
                TableCell::text("Credit Card").width(130.0).show_text(ui);
                TableCell::text("$250.00").width(90.0).show_text(ui);
            });
            TableRow::new().show(ui, |ui| {
                TableCell::text("INV-0042").show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Pending")
                        .variant(BadgeVariant::Secondary)
                        .show(ui);
                });
                TableCell::text("PayPal").width(130.0).show_text(ui);
                TableCell::text("$150.00").width(90.0).show_text(ui);
            });
            TableRow::new().selected(true).show(ui, |ui| {
                TableCell::text("INV-0043").show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Unpaid").variant(BadgeVariant::Outline).show(ui);
                });
                TableCell::text("Bank Transfer").width(130.0).show_text(ui);
                TableCell::text("$300.00").width(90.0).show_text(ui);
            });
            TableRow::new().show(ui, |ui| {
                TableCell::text("INV-0044").show_text(ui);
                TableCell::new().width(110.0).show(ui, |ui| {
                    Badge::new("Paid").show(ui);
                });
                TableCell::text("Credit Card").width(130.0).show_text(ui);
                TableCell::text("$125.00").width(90.0).show_text(ui);
            });
        });
        TableFooter::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                TableCell::text("Total").show_text(ui);
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

    ui.add_space(12.0);
    ui.label("Selection state demo (click rows)");
    ui.add_space(4.0);
    let sel_store = egui::Id::unique("table_demo_selected");
    let mut selected: Option<usize> = ui.ctx().data_mut(|d| d.get_temp(sel_store).unwrap_or(None));
    Table::new().show(ui, |ui| {
        TableHeader::new().show(ui, |ui| {
            TableRow::new().show(ui, |ui| {
                TableHead::new("Name").show(ui);
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
                        TableCell::text(name).show_text(ui);
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
