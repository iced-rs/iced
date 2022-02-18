use iced_core::{Background, Color};

/// The appearance of a menu.
#[derive(Debug, Clone)]
pub struct Style {
    pub text_color: Color,
    pub background: Background,
    pub border_width: f32,
    pub border_color: Color,
    pub selected_text_color: Color,
    pub selected_background: Background,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            text_color: Color::BLACK,
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            selected_text_color: Color::WHITE,
            selected_background: Background::Color([0.4, 0.4, 1.0].into()),
        }
    }
}
