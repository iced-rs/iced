//! Change the appearance of a container.
use iced_core::{Background, Color};

/// The appearance of a container.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The text [`Color`] of the container.
    pub text_color: Option<Color>,
    /// The [`Background`] of the container.
    pub background: Option<Background>,
    /// The border radius of the container.
    pub border_radius: f32,
    /// The border width of the container.
    pub border_width: f32,
    /// The border [`Color`] of the container.
    pub border_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// A set of rules that dictate the [`Appearance`] of a container.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of a container.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
