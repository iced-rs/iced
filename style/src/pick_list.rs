use iced_core::{Background, Color};

/// The appearance of a pick list.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub text_color: Color,
    pub placeholder_color: Color,
    pub background: Background,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub icon_size: f32,
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    type Style: Default + Clone;

    fn active(&self, style: &<Self as StyleSheet>::Style) -> Appearance;

    fn hovered(&self, style: &<Self as StyleSheet>::Style) -> Appearance;
}
