use iced_native::{Color, Palette};

/// Style for the [`Text`] widget.
///
/// [`Text`]: ../widget/text/type.Text.html
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// The text color of the widget.
    pub text_color: Color,
}

impl TextStyle {
    /// Creates a new [`TextStyle`] from the given [`Palette`].
    ///
    /// [`TextStyle`]: struct.TextStyle.html
    /// [`Palette`]: ../struct.Palette.html
    pub fn from_palette(palette: &Palette) -> Self {
        Self {
            text_color: palette.text,
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::from_palette(Palette::default())
    }
}


