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
