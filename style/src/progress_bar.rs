//! Change the appearance of a progress bar.
use iced_core::Background;

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the progress bar.
    pub background: Background,
    /// The [`Background`] of the bar of the progress bar.
    pub bar: Background,
    /// The border radius of the progress bar.
    pub border_radius: f32,
}

/// A set of rules that dictate the style of a progress bar.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of the progress bar.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
