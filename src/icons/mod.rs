//! Real Lucide icons, rasterized once and cached as textures.
//! See `icons::Icon`; geometry lives in the generated `data` module.

// The module keeps the shadcn-style `icons` name for discoverability.
#[allow(clippy::module_inception)]
pub mod icons;

mod data;

pub use icons::{El, Icon};
