//! Decorate content and apply alignment.
use iced_core::{Background, Color};

/// The appearance of a container.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
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
    type Style: Default + Copy;

    /// Produces the [`Appearance`] of a container.
    fn appearance(&self, style: Self::Style) -> Appearance;
}
