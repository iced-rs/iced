use crate::TextStyle;

use iced_native::{Background, Color, Palette};

/// Style for the [`Radio`] widget.
///
/// [`Radio`]: ../widget/struct.Radio.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadioStyle {
    /// The style for the widget's label.
    pub label_style: TextStyle,

    /// Background of the widget.
    pub background: Background,

    /// Width of the widget's borders.
    pub border_width: u16,

    /// Color of the widget's borders.
    pub border_color: Color,

    /// Color of the widget's borders when the mouse hovers over it.
    /// If set to None, border_color will be used.
    pub border_hovered_color: Option<Color>,

    /// Background of the checked indicator dot.
    pub dot_background: Background,
}

impl RadioStyle {
    /// Get the border color for when the mouse hovers over the [`RadioStyle`].
    ///
    /// [`RadioStyle`]: struct.RadioStyle.html
    pub fn get_border_hovered_color(&self) -> Color {
        if let Some(border_hovered_color) = self.border_hovered_color {
            border_hovered_color
        } else {
            self.border_color
        }
    }
}

impl Default for RadioStyle {
    fn default() -> Self {
        Self::from(&Palette::default())
    }
}

impl From<&Palette> for RadioStyle {
    fn from(palette: &Palette) -> Self {
        Self {
            label_style: TextStyle::from(palette),
            background: palette.button.into(),
            border_width: 1,
            border_color: palette.text,
            border_hovered_color: Some(palette.highlight),
            dot_background: palette.button_text.into(),
        }
    }
}

impl AsRef<TextStyle> for RadioStyle {
    fn as_ref(&self) -> &TextStyle {
        &self.label_style
    }
}

impl AsMut<TextStyle> for RadioStyle {
    fn as_mut(&mut self) -> &mut TextStyle {
        &mut self.label_style
    }
}
