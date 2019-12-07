use iced_native::{Background, Color, Palette};

/// Style for the [`Slider`] widget.
///
/// [`Slider`]: ../widget/slider/struct.Slider.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderStyle {
    /// Color of the top half of the rail.
    pub rail_top_color: Color,

    /// Color of the bottom half of the rail.
    pub rail_bottom_color: Color,

    /// Width of the slider handle.
    pub handle_width: f32,

    /// Height of the slider handle.
    pub handle_height: f32,

    /// Width of the slider handle's borders.
    pub handle_border_width: u16,

    /// Corner radius of the slider handle.
    pub handle_corner_radius: u16,

    /// Background of the slider handle.
    pub handle_background: Background,

    /// Background of the slider handle when the mouse hovers over it.
    /// If set to None, `handle_background` will be used.
    pub handle_hovered_background: Option<Background>,

    /// Background of the slider handle when the mouse is pressed over it.
    /// If set to None, `handle_hovered_background` will be used.
    pub handle_pressed_background: Option<Background>,

    /// Color of the slider handle's borders.
    pub handle_border_color: Color,

    /// Color of the slider handle's borders when the mouse hovers over it.
    /// If set to None, `handle_border_color` will be used.
    pub handle_hovered_border_color: Option<Color>,

    /// Color of the slider handle's borders when the mouse is pressed over it.
    /// If set to None, `handle_hovered_border_color` will be used.
    pub handle_pressed_border_color: Option<Color>,
}

impl SliderStyle {
    /// Get the background for when the mouse hovers over the widget.
    pub fn get_handle_hovered_background(&self) -> Background {
        if let Some(handle_hovered_background) = self.handle_hovered_background
        {
            handle_hovered_background
        } else {
            self.handle_background
        }
    }

    /// Get the background for when the mouse is pressed over the widget.
    pub fn get_handle_pressed_background(&self) -> Background {
        if let Some(handle_pressed_background) = self.handle_pressed_background
        {
            handle_pressed_background
        } else {
            self.get_handle_hovered_background()
        }
    }

    /// Get the border color for when the mouse hovers over the widget.
    pub fn get_handle_hovered_border_color(&self) -> Color {
        if let Some(handle_hovered_border_color) =
            self.handle_hovered_border_color
        {
            handle_hovered_border_color
        } else {
            self.handle_border_color
        }
    }

    /// Get the border color for when the mouse is pressed over the widget.
    pub fn get_handle_pressed_border_color(&self) -> Color {
        if let Some(handle_pressed_border_color) =
            self.handle_pressed_border_color
        {
            handle_pressed_border_color
        } else {
            self.get_handle_hovered_border_color()
        }
    }
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self::from(&Palette::default())
    }
}

impl From<&Palette> for SliderStyle {
    fn from(palette: &Palette) -> Self {
        Self {
            rail_top_color: palette.placeholder_text,
            rail_bottom_color: palette.placeholder_text,
            handle_width: 20.0,
            handle_height: 20.0,
            handle_border_width: 1,
            handle_corner_radius: 10,
            handle_background: Background::Color(palette.button),
            handle_hovered_background: None,
            handle_pressed_background: None,
            handle_border_color: palette.placeholder_text,
            handle_hovered_border_color: Some(palette.highlight),
            handle_pressed_border_color: None,
        }
    }
}

