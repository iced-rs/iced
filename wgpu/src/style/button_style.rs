use iced_native::{Background, Color, Palette};

/// Style for the [`Button`] widget.
///
/// [`Button`]: ../widget/button/type.Button.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonStyle {
    /// Background of the widget.
    pub background: Option<Background>,

    /// Background of the widget when the mouse hovers over it.
    /// If set to None, `background` will be used.
    pub hovered_background: Option<Background>,

    /// Background of the widget when the mouse is pressed over it.
    /// If set to None, `hovered_background` will be used.
    pub pressed_background: Option<Background>,

    /// Radius of the widget's borders.
    pub border_radius: u16,

    /// Width of the widget's borders.
    pub border_width: u16,

    /// Color of the widget's borders.
    pub border_color: Color,

    /// Color of the widget's borders when the mouse hovers over it.
    /// If set to None, `border_color` will be used.
    pub hovered_border_color: Option<Color>,

    /// Color of the widget's borders when the mouse is pressed over it.
    /// If set to None, `border_hovered_color` will be used.
    pub pressed_border_color: Option<Color>,
}

impl ButtonStyle {
    /// Get the background for when the mouse hovers over the widget.
    pub fn get_hovered_background(&self) -> Option<Background> {
        if self.hovered_background.is_some() {
            self.hovered_background
        } else {
            self.background
        }
    }

    /// Get the background for when the mouse is pressed over the widget.
    pub fn get_pressed_background(&self) -> Option<Background> {
        if self.pressed_background.is_some() {
            self.pressed_background
        } else {
            self.get_hovered_background()
        }
    }

    /// Get the border color for when the mouse hovers over the widget.
    pub fn get_hovered_border_color(&self) -> Color {
        if let Some(hovered_border_color) = self.hovered_border_color {
            hovered_border_color
        } else {
            self.border_color
        }
    }

    /// Get the border color for when the mouse is pressed over the widget.
    pub fn get_pressed_border_color(&self) -> Color {
        if let Some(pressed_border_color) = self.pressed_border_color {
            pressed_border_color
        } else {
            self.get_hovered_border_color()
        }
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self::from(&Palette::default())
    }
}

impl From<&Palette> for ButtonStyle {
    fn from(palette: &Palette) -> Self {
        Self {
            background: Some(Background::Color(palette.button)),
            hovered_background: None,
            pressed_background: Some(Background::Color(palette.highlight)),
            border_radius: 3,
            border_width: 1,
            border_color: palette.text,
            hovered_border_color: Some(palette.highlight),
            pressed_border_color: None,
        }
    }
}
