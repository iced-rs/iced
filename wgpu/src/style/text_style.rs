use iced_native::{Color, Palette};

/// Style for the [`Text`] widget.
///
/// [`Text`]: ../widget/struct.Text.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// The text color of the widget.
    pub text_color: Color,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::from(&Palette::default())
    }
}

impl From<&Palette> for TextStyle {
    fn from(palette: &Palette) -> Self {
        Self {
            text_color: palette.text,
        }
    }
}

