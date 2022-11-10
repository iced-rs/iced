//! Change the apperance of a slider.
use iced_core::Color;

/// The appearance of a slider.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The colors of the rail of the slider.
    pub rail_colors: (Color, Color),
    /// The appearance of the [`Handle`] of the slider.
    pub handle: Handle,
}

/// The appearance of the handle of a slider.
#[derive(Debug, Clone, Copy)]
pub struct Handle {
    /// The shape of the handle.
    pub shape: HandleShape,
    /// The [`Color`] of the handle.
    pub color: Color,
    /// The border width of the handle.
    pub border_width: f32,
    /// The border [`Color`] of the handle.
    pub border_color: Color,
}

/// The shape of the handle of a slider.
#[derive(Debug, Clone, Copy)]
pub enum HandleShape {
    /// A circular handle.
    Circle {
        /// The radius of the circle.
        radius: f32,
    },
    /// A rectangular shape.
    Rectangle {
        /// The width of the rectangle.
        width: u16,
        /// The border radius of the corners of the rectangle.
        border_radius: f32,
    },
}

/// A set of rules that dictate the style of a slider.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the style of an active slider.
    fn active(&self, style: &Self::Style) -> Appearance;

    /// Produces the style of an hovered slider.
    fn hovered(&self, style: &Self::Style) -> Appearance;

    /// Produces the style of a slider that is being dragged.
    fn dragging(&self, style: &Self::Style) -> Appearance;
}
