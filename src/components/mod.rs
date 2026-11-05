//! Components are added here as they are ported.

pub mod aspect_ratio;
pub mod badge;
pub mod button;
pub mod kbd;
pub mod label;
pub mod marker;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod typography;

pub use aspect_ratio::AspectRatio;
pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonSize, ButtonVariant, GroupOrientation, GroupPosition};
pub use kbd::{Kbd, KbdGroup};
pub use label::Label;
pub use marker::{Marker, MarkerContent, MarkerIcon, MarkerVariant};
pub use separator::{Separator, SeparatorOrientation};
pub use skeleton::Skeleton;
pub use spinner::Spinner;
pub use typography::{
    Blockquote, CodeBlock, InlineCode, Large, Lead, ListMarker, Muted, Ol, Small, TableCaption, Ul,
    H1, H2, H3, H4, P,
};
