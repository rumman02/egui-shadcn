//! Shadow presets matching Tailwind's `shadow-xs/sm/md/lg` as used in the
//! shadcn source.

use egui::epaint::Shadow;
use egui::Color32;

/// `shadow-xs` — the subtle border shadow used by buttons and inputs.
pub fn shadow_xs(dark: bool) -> Shadow {
    Shadow {
        offset: [0, 1],
        blur: 2,
        spread: 0,
        color: color(dark, 50),
    }
}

/// `shadow-sm`
pub fn shadow_sm(dark: bool) -> Shadow {
    Shadow {
        offset: [0, 1],
        blur: 4,
        spread: 0,
        color: color(dark, 70),
    }
}

/// `shadow-md`
pub fn shadow_md(dark: bool) -> Shadow {
    Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: color(dark, 80),
    }
}

/// `shadow-lg` — popovers, dropdowns, dialogs.
pub fn shadow_lg(dark: bool) -> Shadow {
    Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: color(dark, 110),
    }
}

/// `shadow-xl` — the chart tooltip and other large floating panels (Tailwind
/// xl = y-offset 20, blur 25): a deeper `shadow-lg`.
pub fn shadow_xl(dark: bool) -> Shadow {
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
