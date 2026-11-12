//! Shared helpers: animation, focus ring, shadow, ids.

pub mod animation;
pub mod focus;
pub mod id;
pub mod shadow;

pub use animation::{
    animate_value, mix_oklab, nova_overlay_progress, open_progress, overlay_progress,
    transition_alpha, transition_color, transition_progress, AlphaTransition, ColorTransition,
    Duration, Easing, OverlayProgress, Transition, TransitionSpec,
};
pub use focus::{is_focused, paint_focus_ring, paint_ring, request_focus};
pub use id::{id_path, unique_id};
pub use shadow::{shadow_lg, shadow_md, shadow_sm, shadow_xs};
