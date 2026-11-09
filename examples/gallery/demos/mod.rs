//! One demo module per ported component.

pub mod alert_dialog_demo;
pub mod dialog_demo;
pub mod drawer_demo;
pub mod hover_card_demo;
pub mod popover_demo;
pub mod sheet_demo;
pub mod sonner_demo;
pub mod tooltip_demo;

use egui_shadcn_ui::prelude::*;

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
    ui.label("States");
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
        Button::new("Invalid")
            .variant(ButtonVariant::Outline)
            .invalid(true)
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a button, press Enter or Space to activate.");
}

pub fn badge_demo(ui: &mut egui::Ui) {
    ui.label("Variants");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Badge::new("Badge").show(ui);
        Badge::new("Secondary")
            .variant(BadgeVariant::Secondary)
            .show(ui);
        Badge::new("Destructive")
            .variant(BadgeVariant::Destructive)
            .show(ui);
        Badge::new("Outline")
            .variant(BadgeVariant::Outline)
            .show(ui);
        Badge::new("Ghost").variant(BadgeVariant::Ghost).show(ui);
        Badge::new("Link").variant(BadgeVariant::Link).show(ui);
    });

    ui.add_space(12.0);
    ui.label("Inline with text");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Build status:");
        Badge::new("Passing").show(ui);
        ui.label("Version:");
        Badge::new("v2.0.0")
            .variant(BadgeVariant::Secondary)
            .show(ui);
        ui.label("Coverage:");
        Badge::new("98%").variant(BadgeVariant::Outline).show(ui);
    });

    ui.add_space(12.0);
    ui.label("Disabled");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Badge::new("Disabled").disabled(true).show(ui);
        Badge::new("Disabled outline")
            .variant(BadgeVariant::Outline)
            .disabled(true)
            .show(ui);
    });
}

pub fn label_demo(ui: &mut egui::Ui) {
    ui.label("Plain labels");
    ui.add_space(4.0);
    Label::new("Email address").show(ui);
    ui.add_space(4.0);
    Label::new("Project name").show(ui);

    ui.add_space(12.0);
    ui.label("Bound to a control: click the label to focus the field.");
    ui.add_space(4.0);
    let id = egui::Id::unique("demo_email_field");
    let store = egui::Id::unique("demo_email_value");
    Label::new("Email").for_id(id).show(ui);
    let mut email: String = ui.ctx().data_mut(|d| {
        d.get_temp(store)
            .unwrap_or_else(|| "m@example.com".to_owned())
    });
    let resp = egui::TextEdit::singleline(&mut email)
        .id_salt(id)
        .desired_width(280.0)
        .show(ui);
    if resp.response.has_focus() {
        ui.label("(focused via label click)");
    }
    ui.ctx().data_mut(|d| d.insert_temp(store, email));

    ui.add_space(12.0);
    ui.label("Disabled");
    ui.add_space(4.0);
    Label::new("Disabled label").disabled(true).show(ui);
}

pub fn separator_demo(ui: &mut egui::Ui) {
    ui.label("Horizontal");
    ui.add_space(4.0);
    Separator::new().show(ui);
    ui.add_space(8.0);
    ui.label("Content below the separator.");
    ui.add_space(4.0);
    Separator::new().show(ui);

    ui.add_space(12.0);
    ui.label("Vertical (in a row)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.set_min_height(36.0);
        ui.label("Left");
        Separator::new().vertical(true).length(36.0).show(ui);
        ui.label("Middle");
        Separator::new().vertical(true).length(36.0).show(ui);
        ui.label("Right");
    });

    ui.add_space(12.0);
    ui.label("Fixed length");
    ui.add_space(4.0);
    Separator::new().length(160.0).show(ui);
}

pub fn kbd_demo(ui: &mut egui::Ui) {
    ui.label("Single keys");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Kbd::new("⌘").show(ui);
        Kbd::new("⇧").show(ui);
        Kbd::new("K").show(ui);
        Kbd::new("Enter").show(ui);
        Kbd::new("Esc").show(ui);
    });

    ui.add_space(12.0);
    ui.label("Groups (gap-1)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Command palette:");
        KbdGroup::new().show(ui, |ui| {
            Kbd::new("⌘").show(ui);
            Kbd::new("K").show(ui);
        });
    });
    ui.horizontal(|ui| {
        ui.label("Undo:");
        KbdGroup::new().show(ui, |ui| {
            Kbd::new("⌘").show(ui);
            Kbd::new("⇧").show(ui);
            Kbd::new("Z").show(ui);
        });
    });

    ui.add_space(12.0);
    ui.label("Inline with text");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Press");
        Kbd::new("Tab").show(ui);
        ui.label("to move focus, or");
        Kbd::new("Esc").show(ui);
        ui.label("to dismiss.");
    });
}

pub fn spinner_demo(ui: &mut egui::Ui) {
    ui.label("Sizes (default size-4 = 16px)");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Spinner::new().size(12.0).show(ui);
        Spinner::new().show(ui);
        Spinner::new().size(24.0).show(ui);
        Spinner::new().size(32.0).show(ui);
    });

    ui.add_space(12.0);
    ui.label("With colors and inside a button");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let theme = ShadcnTheme::of_ui(ui);
        Spinner::new().color(theme.palette.primary).show(ui);
        Spinner::new().color(theme.palette.destructive).show(ui);
        Spinner::new()
            .color(theme.palette.muted_foreground)
            .show(ui);
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let theme = ShadcnTheme::of_ui(ui);
        Button::new("Loading")
            .icon_start(Icon::LoaderCircle)
            .disabled(true)
            .show(ui);
        Spinner::new()
            .color(theme.palette.primary_foreground)
            .show(ui);
    });
}

pub fn skeleton_demo(ui: &mut egui::Ui) {
    ui.label("Text-line placeholders");
    ui.add_space(4.0);
    Skeleton::new()
        .size(ui.available_width().min(320.0), 16.0)
        .show(ui);
    Skeleton::new()
        .size(ui.available_width().min(280.0), 16.0)
        .show(ui);
    Skeleton::new()
        .size(ui.available_width().min(300.0), 16.0)
        .show(ui);

    ui.add_space(12.0);
    ui.label("Card placeholder");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Skeleton::new()
            .size(48.0, 48.0)
            .corner_radius(theme_radius_full(ui))
            .show(ui);
        ui.vertical(|ui| {
            Skeleton::new().size(180.0, 16.0).show(ui);
            ui.add_space(6.0);
            Skeleton::new().size(140.0, 12.0).show(ui);
        });
    });

    ui.add_space(12.0);
    ui.label("Large block (h-[125px] w-[250px])");
    ui.add_space(4.0);
    Skeleton::new().size(250.0, 125.0).show(ui);

    ui.add_space(12.0);
    ui.label("Static (animation off)");
    ui.add_space(4.0);
    Skeleton::new().size(200.0, 20.0).animate(false).show(ui);
}

fn theme_radius_full(ui: &egui::Ui) -> u8 {
    ShadcnTheme::of_ui(ui).radius.full_u8()
}

pub fn typography_demo(ui: &mut egui::Ui) {
    H1::new("Taxing Laughter: The Joke Tax Chronicles").show(ui);
    ui.add_space(8.0);
    P::new("The joke tax is a levy imposed on the punchline economy. When the people discovered the jokes were taxable, laughter became a ledger entry — and humor learned to hide.").show(ui);
    ui.add_space(12.0);
    H2::new("The King's Plan").show(ui);
    ui.add_space(8.0);
    P::new("The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax.").show(ui);
    ui.add_space(12.0);
    Blockquote::new("After the tax was repealed, the court jester's salary doubled, and the kingdom's GDP of giggles soared.").show(ui);
    ui.add_space(12.0);
    H3::new("The Joke Tax Chronicles").show(ui);
    ui.add_space(8.0);
    P::new("As implemented in the royal decree, the tax collected a flat rate on all punchlines above three words.").show(ui);
    ui.add_space(12.0);
    H4::new("Exemptions").show(ui);
    ui.add_space(8.0);
    Ul::new(&[
        "Puns under one second",
        "Knock-knock jokes",
        "Sighs and groans",
    ])
    .show(ui);
    ui.add_space(12.0);
    Ol::new(&[
        "File form LOL-1",
        "Attach proof of laughter",
        "Pay within 30 days",
    ])
    .show(ui);
    ui.add_space(12.0);
    Lead::new(
        "A modal dialog that interrupts the user with important content and expects a response.",
    )
    .show(ui);
    ui.add_space(8.0);
    Large::new("Display a prominent message in a large size.").show(ui);
    ui.add_space(8.0);
    Small::new("Email address").show(ui);
    ui.add_space(4.0);
    Muted::new("Enter your email to subscribe to the joke tax digest.").show(ui);
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.label("Press");
        InlineCode::new("Cmd + K").show(ui);
        ui.label("to open the palette. Kingdom GDP figure:");
        InlineCode::new("GDP_giggles").show(ui);
    });
    ui.add_space(12.0);
    CodeBlock::new(
        "fn levy(punchline: &Punchline) -> Tax {
    Tax::new(punchline.words()).rate(0.2)
}",
    )
    .show(ui);
    ui.add_space(8.0);
    TableCaption::new("Table 1 — joke revenue by quarter.").show(ui);
}

pub fn aspect_ratio_demo(ui: &mut egui::Ui) {
    ui.label("16 / 9 (full width)");
    ui.add_space(4.0);
    AspectRatio::new(16.0 / 9.0).show(ui, |ui| {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
        ui.painter().rect_filled(rect, 8, theme.palette.muted);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "16:9 content",
            egui::FontId::proportional(14.0),
            theme.palette.muted_foreground,
        );
    });

    ui.add_space(12.0);
    ui.label("1 / 1, max width 240");
    ui.add_space(4.0);
    AspectRatio::new(1.0).max_width(240.0).show(ui, |ui| {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
        ui.painter().rect_filled(rect, 8, theme.palette.accent);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "1:1",
            egui::FontId::proportional(14.0),
            theme.palette.accent_foreground,
        );
    });

    ui.add_space(12.0);
    ui.label("4 / 3, max width 320");
    ui.add_space(4.0);
    AspectRatio::new(4.0 / 3.0).max_width(320.0).show(ui, |ui| {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
        ui.painter().rect_filled(rect, 8, theme.palette.secondary);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "4:3",
            egui::FontId::proportional(14.0),
            theme.palette.secondary_foreground,
        );
    });
}

pub fn marker_demo(ui: &mut egui::Ui) {
    ui.label("Default");
    ui.add_space(4.0);
    Marker::new().show(ui, |ui| {
        MarkerIcon::new(Icon::Sparkles).show(ui);
        MarkerContent::new("Heads up — AI usage is trending up this week").show(ui);
    });

    ui.add_space(12.0);
    ui.label("Separator (content centered between rules)");
    ui.add_space(4.0);
    Marker::new()
        .variant(MarkerVariant::Separator)
        .show(ui, |ui| {
            MarkerContent::new("or continue with").show(ui);
        });
    Marker::new()
        .variant(MarkerVariant::Separator)
        .show(ui, |ui| {
            MarkerIcon::new(Icon::Plus).show(ui);
            MarkerContent::new("add item").show(ui);
        });

    ui.add_space(12.0);
    ui.label("Border (bottom rule)");
    ui.add_space(4.0);
    Marker::new().variant(MarkerVariant::Border).show(ui, |ui| {
        MarkerIcon::new(Icon::CircleAlert).show(ui);
        MarkerContent::new("Deployment pending review").show(ui);
    });
    Marker::new().variant(MarkerVariant::Border).show(ui, |ui| {
        MarkerContent::new("Last synced 2 minutes ago").show(ui);
    });
}

pub fn button_group_demo(ui: &mut egui::Ui) {
    ui.label("Horizontal (fused corners and borders)");
    ui.add_space(4.0);
    ButtonGroup::new().show(ui, |ui| {
        Button::new("Day").show(ui);
        Button::new("Month").show(ui);
        Button::new("Year").show(ui);
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ButtonGroup::new().show(ui, |ui| {
            Button::new("Outline left")
                .variant(ButtonVariant::Outline)
                .show(ui);
            Button::new("Outline right")
                .variant(ButtonVariant::Outline)
                .show(ui);
        });
    });

    ui.add_space(8.0);
    ui.label("With separators");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ButtonGroup::new().show(ui, |ui| {
            Button::new("")
                .size(ButtonSize::Icon)
                .variant(ButtonVariant::Outline)
                .icon_start(Icon::ChevronLeft)
                .show(ui);
            ButtonGroupSeparator::new().show(ui);
            Button::new("")
                .size(ButtonSize::Icon)
                .variant(ButtonVariant::Outline)
                .icon_start(Icon::MoreHorizontal)
                .show(ui);
            ButtonGroupSeparator::new().show(ui);
            Button::new("")
                .size(ButtonSize::Icon)
                .variant(ButtonVariant::Outline)
                .icon_start(Icon::ChevronRight)
                .show(ui);
        });
    });

    ui.add_space(12.0);
    ui.label("With a text slot");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ButtonGroup::new().show(ui, |ui| {
            ButtonGroupText::new("https://example.com").show(ui);
            Button::new("")
                .size(ButtonSize::Icon)
                .variant(ButtonVariant::Outline)
                .icon_start(Icon::Copy)
                .show(ui);
        });
    });

    ui.add_space(12.0);
    ui.label("Vertical");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ButtonGroup::new()
            .orientation(GroupOrientation::Vertical)
            .id_salt("vertical_group")
            .show(ui, |ui| {
                Button::new("Cut").show(ui);
                Button::new("Copy").show(ui);
                Button::new("Paste").show(ui);
            });
        ButtonGroup::new()
            .orientation(GroupOrientation::Vertical)
            .id_salt("vertical_group_outline")
            .show(ui, |ui| {
                Button::new("Top").variant(ButtonVariant::Outline).show(ui);
                ButtonGroupSeparator::new()
                    .orientation(GroupOrientation::Vertical)
                    .show(ui);
                Button::new("Middle")
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                ButtonGroupSeparator::new()
                    .orientation(GroupOrientation::Vertical)
                    .show(ui);
                Button::new("Bottom")
                    .variant(ButtonVariant::Outline)
                    .show(ui);
            });
    });

    ui.add_space(12.0);
    ui.label("Sizes and states");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ButtonGroup::new().show(ui, |ui| {
            Button::new("Sm").size(ButtonSize::Sm).show(ui);
            Button::new("Sm")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Secondary)
                .show(ui);
        });
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ButtonGroup::new().show(ui, |ui| {
            Button::new("Disabled").disabled(true).show(ui);
            Button::new("Active").show(ui);
        });
    });
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
    let mut values: [bool; 5] = ui.ctx().data_mut(|d| {
        d.get_temp(store)
            .unwrap_or([true, false, false, false, false])
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
    Checkbox::new(&mut values[4]).invalid(true).show(ui);

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a checkbox, press Space or Enter to toggle.");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn radio_group_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("radio_demo_values");
    let mut values: [Option<usize>; 3] = ui
        .ctx()
        .data_mut(|d| d.get_temp(store).unwrap_or([Some(0), None, Some(1)]));

    ui.label("Vertical (default, gap-3)");
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
        RadioGroup::new(&mut values[0])
            .id_salt("radio_demo_disabled_group")
            .disabled(true)
            .option("A")
            .option("B")
            .show(ui);
        RadioGroup::new(&mut values[0])
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
    ui.label("Keyboard: arrows move the selection and focus (wrapping).");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn switch_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("switch_demo_values");
    let mut values: [bool; 4] = ui
        .ctx()
        .data_mut(|d| d.get_temp(store).unwrap_or([false, true, false, true]));

    ui.label("Default");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Switch::new(&mut values[0]).id_salt("sw_demo_1").show(ui);
        Switch::new(&mut values[1]).id_salt("sw_demo_2").show(ui);
    });

    ui.add_space(12.0);
    ui.label("Small");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Switch::new(&mut values[0])
            .size(SwitchSize::Sm)
            .id_salt("sw_demo_sm_1")
            .show(ui);
        Switch::new(&mut values[1])
            .size(SwitchSize::Sm)
            .id_salt("sw_demo_sm_2")
            .show(ui);
    });

    ui.add_space(12.0);
    ui.label("With label");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Switch::new(&mut values[2])
            .id_salt("sw_demo_airplane")
            .show(ui);
        ui.label("Airplane Mode");
    });

    ui.add_space(12.0);
    ui.label("Disabled");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        Switch::new(&mut values[2]).disabled(true).show(ui);
        Switch::new(&mut values[3]).disabled(true).show(ui);
    });

    ui.add_space(12.0);
    ui.label("Invalid");
    ui.add_space(4.0);
    Switch::new(&mut values[3]).invalid(true).show(ui);

    ui.add_space(12.0);
    ui.label("Keyboard: Tab to a switch, press Space or Enter to toggle.");

    ui.ctx().data_mut(|d| d.insert_temp(store, values));
}

pub fn slider_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("slider_demo_values");
    let mut values: (f32, f32, f32, f32, f32) = ui
        .ctx()
        .data_mut(|d| d.get_temp(store).unwrap_or((30.0, 20.0, 80.0, 25.0, 50.0)));

    ui.label("Single value (drag the thumb, click the track, use Arrow keys)");
    ui.add_space(4.0);
    let (v0, _, _, _, _) = values;
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
    let (a, b, _, _, _) = values;
    let mut pair = (a, b);
    Slider::range(&mut pair).min(0.0).max(100.0).show(ui);
    values.1 = pair.0;
    values.2 = pair.1;
    ui.small(format!("range: {:.0} – {:.0}", pair.0, pair.1));

    ui.add_space(12.0);
    ui.label("Stepped (step = 10)");
    ui.add_space(4.0);
    let (_, _, _, c, _) = values;
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
    let (_, _, _, _, e) = values;
    let mut frozen = e;
    Slider::new(&mut frozen)
        .min(0.0)
        .max(100.0)
        .disabled(true)
        .show(ui);
    values.4 = frozen;

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
    let mut notifications: bool = ui
        .ctx()
        .data_mut(|d| d.get_temp(notif_store).unwrap_or(false));
    let mut email: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(email_store).unwrap_or_default());
    let mut name: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(name_store).unwrap_or_default());

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

    ui.ctx().data_mut(|d| {
        d.insert_temp(notif_store, notifications);
        d.insert_temp(email_store, email);
        d.insert_temp(name_store, name);
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
    ui.label("Destructive alert inside a card");
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

pub fn accordion_demo(ui: &mut egui::Ui) {
    let multi_store = egui::Id::unique("accordion_demo_multi");
    let mut multi: Vec<bool> = ui
        .ctx()
        .data_mut(|d| d.get_temp(multi_store).unwrap_or(vec![true, false, false]));

    ui.label("Single, collapsible (first item open by default)");
    ui.add_space(4.0);
    Accordion::new()
        .id_salt("accordion_demo_single")
        .default_open(0)
        .show(ui, |ui| {
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Product Information").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label(
                        "Our flagship product combines cutting-edge technology with sleek design. \
                     Built with premium materials, it offers unparalleled performance.",
                    );
                    ui.label(
                        "Key features include advanced processing capabilities and an intuitive \
                     user interface designed for both beginners and experts.",
                    );
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Shipping Details").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label(
                        "We offer worldwide shipping through trusted courier partners. Standard \
                     delivery takes 3-5 business days, express shipping 1-2 business days.",
                    );
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Return Policy").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label(
                        "We stand behind our products with a comprehensive 30-day return policy. \
                     Free return shipping and full refunds processed within 48 hours.",
                    );
                });
            });
        });

    ui.add_space(12.0);
    ui.label(format!("Multiple (open flags: {multi:?})"));
    ui.add_space(4.0);
    Accordion::multiple(&mut multi)
        .id_salt("accordion_demo_multi")
        .show(ui, |ui| {
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Billing").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Invoices are issued monthly and can be downloaded as PDFs.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Team members").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Invite unlimited teammates on the Pro plan.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("API access").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Generate personal access tokens from the dashboard.");
                });
            });
        });

    ui.add_space(12.0);
    ui.label("Disabled trigger and non-collapsible single");
    ui.add_space(4.0);
    Accordion::new()
        .id_salt("accordion_demo_locked")
        .default_open(0)
        .collapsible(false)
        .show(ui, |ui| {
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Always open (collapsible = false)").show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("Clicking this trigger keeps the section open.");
                });
            });
            AccordionItem::new().show(ui, |ui| {
                AccordionTrigger::new("Unavailable").disabled(true).show(ui);
                AccordionContent::new().show(ui, |ui| {
                    ui.label("You cannot open this section.");
                });
            });
        });

    ui.add_space(12.0);
    ui.label("Keyboard: Tab and Arrow keys move focus; Enter or Space toggles.");

    ui.ctx().data_mut(|d| d.insert_temp(multi_store, multi));
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
    ui.label("Line variant (underline indicator)");
    ui.add_space(4.0);
    Tabs::new().id_salt("tabs_demo_line").show(ui, |ui| {
        TabsList::new().variant(TabsVariant::Line).show(ui, |ui| {
            TabsTrigger::new("preview", "Preview").show(ui);
            TabsTrigger::new("code", "Code").show(ui);
            TabsTrigger::new("logs", "Logs").show(ui);
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
    ui.label("Keyboard: Tab and Arrow keys move focus; Enter or Space selects.");

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

    ui.label("Click a trigger to open. ArrowUp/Down navigate, ArrowRight opens sub-menus, Enter/Space activate, Escape closes.");
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
