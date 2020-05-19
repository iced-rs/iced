use iced_native::Color;

/// The style used to fill geometry.
#[derive(Debug, Clone, Copy)]
pub enum Fill {
    /// Fill with a color.
    Color(Color),
}

impl Default for Fill {
    fn default() -> Fill {
        Fill::Color(Color::BLACK)
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Fill {
        Fill::Color(color)
    }
}
