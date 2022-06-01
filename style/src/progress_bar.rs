//! Provide progress feedback to your users.
use iced_core::Background;

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub background: Background,
    pub bar: Background,
    pub border_radius: f32,
}

/// A set of rules that dictate the style of a progress bar.
pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}
