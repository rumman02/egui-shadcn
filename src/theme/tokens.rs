//! Design tokens translated from Tailwind CSS v4 defaults as used by the
//! shadcn/ui source: the radius scale derived from `--radius`, spacing in CSS
//! pixels, and the type scale.

/// Radius tokens derived from `--radius` (default `0.625rem` = 10px).
///
/// The source defines: `sm = 0.6r, md = 0.8r, lg = r, xl = 1.4r, 2xl = 1.8r,
/// 3xl = 2.2r, 4xl = 2.6r`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadiusTokens {
    /// `--radius` in pixels (e.g. 10.0).
    pub base: f32,
}

impl RadiusTokens {
    /// Builds the scale from `--radius` in pixels.
    pub fn new(base: f32) -> Self {
        Self { base }
    }

    /// `rounded-sm` = 0.6 × base.
    pub fn sm(&self) -> f32 {
        self.base * 0.6
    }

    /// `rounded-md` = 0.8 × base. The most common component radius.
    pub fn md(&self) -> f32 {
        self.base * 0.8
    }

    /// `rounded-lg` = base.
    pub fn lg(&self) -> f32 {
        self.base
    }

    /// `rounded-xl` = 1.4 × base.
    pub fn xl(&self) -> f32 {
        self.base * 1.4
    }

    /// `rounded-2xl` = 1.8 × base.
    pub fn x2l(&self) -> f32 {
        self.base * 1.8
    }

    /// `rounded-3xl` = 2.2 × base.
    pub fn x3l(&self) -> f32 {
        self.base * 2.2
    }

    /// `rounded-4xl` = 2.6 × base (used by badges).
    pub fn x4l(&self) -> f32 {
        self.base * 2.6
    }

    /// Fully rounded (`rounded-full`).
    pub fn full(&self) -> f32 {
        f32::MAX / 4.0
    }

    /// [`Self::md`] rounded for egui's `CornerRadius` (u8 pixels).
    pub fn md_u8(&self) -> u8 {
        self.md().round() as u8
    }

    /// [`Self::lg`] rounded for egui's `CornerRadius` (u8 pixels).
    pub fn lg_u8(&self) -> u8 {
        self.lg().round() as u8
    }

    /// [`Self::xl`] rounded for egui's `CornerRadius` (u8 pixels).
    pub fn xl_u8(&self) -> u8 {
        self.xl().round() as u8
    }

    /// [`Self::full`] rounded for egui's `CornerRadius` (u8 pixels).
    pub fn full_u8(&self) -> u8 {
        u8::MAX
    }
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self::new(10.0)
    }
}

/// Spacing constants in CSS pixels matching the Tailwind scale used in the
/// source (`1 unit = 0.25rem = 4px`).
pub mod spacing {
    /// `gap-0.5`, `p-0.5`
    pub const HALF: f32 = 2.0;
    /// `gap-1`, `p-1`
    pub const XS: f32 = 4.0;
    /// `gap-1.5`, `p-1.5`
    pub const XS_HALF: f32 = 6.0;
    /// `gap-2`, `p-2`
    pub const SM: f32 = 8.0;
    /// `gap-2.5`, `p-2.5`
    pub const SM_HALF: f32 = 10.0;
    /// `gap-3`, `p-3`
    pub const MD: f32 = 12.0;
    /// `gap-4`, `px-4`, `p-4`
    pub const LG: f32 = 16.0;
    /// `gap-6`, `p-6`
    pub const XL: f32 = 24.0;
    /// `gap-8`, `p-8`
    pub const X2L: f32 = 32.0;
    /// Standard control heights.
    pub const CONTROL_XS: f32 = 24.0; // h-6
    /// `h-8`
    pub const CONTROL_SM: f32 = 32.0;
    /// `h-9` — the default button/input height.
    pub const CONTROL: f32 = 36.0;
    /// `h-10`
    pub const CONTROL_LG: f32 = 40.0;
}

/// Type scale in pixels matching Tailwind text sizes (`text-sm` etc.).
pub mod type_scale {
    /// `text-xs` (12px, line-height 16px).
    pub const XS: f32 = 12.0;
    /// `text-sm` (14px) — the default UI text size.
    pub const SM: f32 = 14.0;
    /// `text-base` (16px).
    pub const BASE: f32 = 16.0;
    /// `text-lg` (18px).
    pub const LG: f32 = 18.0;
    /// `text-xl` (20px).
    pub const XL: f32 = 20.0;
    /// `text-2xl` (24px).
    pub const X2L: f32 = 24.0;
    /// `text-3xl` (30px).
    pub const X3L: f32 = 30.0;
    /// `text-4xl` (36px).
    pub const X4L: f32 = 36.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_radius_scale_matches_source() {
        let r = RadiusTokens::default();
        assert!((r.md() - 8.0).abs() < 1e-4);
        assert!((r.lg() - 10.0).abs() < 1e-4);
        assert!((r.xl() - 14.0).abs() < 1e-4);
        assert!((r.x4l() - 26.0).abs() < 1e-4);
    }
}
