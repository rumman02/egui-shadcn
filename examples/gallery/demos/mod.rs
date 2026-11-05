//! One demo module per ported component.

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
        Checkbox::new(&mut values[1])
            .id_salt("cb_demo_2")
            .show(ui);
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
