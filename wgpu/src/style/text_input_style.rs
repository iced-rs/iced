use iced_native::{Background, Color, Font, Palette};

/// Style for the [`TextInput`] widget.
///
/// [`TextInput`]: ../widget/text_input/struct.TextInput.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextInputStyle {
    /// The text color of the widget.
    pub text_color: Color,

    /// The placeholder text color of the widget.
    pub placeholder_color: Color,

    /// The text font of the widget.
    pub font: Font,

    /// Background of the widget.
    pub background: Option<Background>,

    /// Radius of the widget's borders.
    pub border_radius: u16,

    /// Width of the widget's borders.
    pub border_width: u16,

    /// Color of the widget's borders.
    pub border_color: Color,

    /// Color of the widget's borders when the mouse hovers over it.
    /// If set to None, border_color will be used.
    pub border_hovered_color: Option<Color>,
}

impl TextInputStyle {
    /// Get the border color for when the mouse hovers over the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn get_border_hovered_color(&self) -> Color {
        if let Some(border_hovered_color) = self.border_hovered_color {
            border_hovered_color
        } else {
            self.border_color
        }
    }
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self::from(&Palette::default())
    }
}

impl From<&Palette> for TextInputStyle {
    fn from(palette: &Palette) -> Self {
        Self {
            text_color: palette.text,
            placeholder_color: palette.placeholder_text,
            font: Font::Default,
            background: Some(Background::Color(palette.base)),
            border_radius: 3,
            border_width: 1,
            border_color: palette.placeholder_text,
            border_hovered_color: Some(palette.highlight),
        }
    }
}

