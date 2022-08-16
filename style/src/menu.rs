use iced_core::{Background, Color};

/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub text_color: Color,
    pub background: Background,
    pub border_width: f32,
    pub border_radius: f32,
    pub border_color: Color,
    pub selected_text_color: Color,
    pub selected_background: Background,
}

pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}
