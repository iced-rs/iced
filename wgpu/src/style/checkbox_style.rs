use crate::TextStyle;

use iced_native::{Background, Color, Palette};

/// Style for the [`Checkbox`] widget.
///
/// [`Checkbox`]: ../widget/struct.Checkbox.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxStyle {
    /// The style for the widget's label.
    pub label_style: TextStyle,

    /// The widget's background.
    pub background: Background,

    /// The radius of the widget's borders.
    pub border_radius: u16,

    /// The width of the widget's borders.
    pub border_width: u16,

    /// The color of the widget's borders.
    pub border_color: Color,

    /// The color of the widget's borders when the mouse hovers over it.
    /// If set to None, border_color will be used.
    pub border_hovered_color: Option<Color>,

    /// The color of the checked indicator.
    pub checked_color: Color,
}

impl CheckboxStyle {
    /// Get the border color for when the mouse hovers over the [`CheckboxStyle`].
    ///
    /// [`CheckboxStyle`]: struct.CheckboxStyle.html
    pub fn get_border_hovered_color(&self) -> Color {
        if let Some(border_hovered_color) = self.border_hovered_color {
            border_hovered_color
        } else {
            self.border_color
        }
    }
}

impl Default for CheckboxStyle {
    fn default() -> Self {
        Self::from(&Palette::default())
    }
}

impl From<&Palette> for CheckboxStyle {
    fn from(palette: &Palette) -> Self {
        Self {
            label_style: TextStyle::from(palette),
            background: palette.button.into(),
            border_radius: 6,
            border_width: 1,
            border_color: palette.text,
            border_hovered_color: Some(palette.highlight),
            checked_color: palette.button_text,
        }
    }
}

impl AsRef<TextStyle> for CheckboxStyle {
    fn as_ref(&self) -> &TextStyle {
        &self.label_style
    }
}

impl AsMut<TextStyle> for CheckboxStyle {
    fn as_mut(&mut self) -> &mut TextStyle {
        &mut self.label_style
    }
}
