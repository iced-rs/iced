//! Display an interactive selector of a single value from a range of values.
use iced_core::Color;

/// The appearance of a slider.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub rail_colors: (Color, Color),
    pub handle: Handle,
}

/// The appearance of the handle of a slider.
#[derive(Debug, Clone, Copy)]
pub struct Handle {
    pub shape: HandleShape,
    pub color: Color,
    pub border_width: f32,
    pub border_color: Color,
}

/// The shape of the handle of a slider.
#[derive(Debug, Clone, Copy)]
pub enum HandleShape {
    Circle { radius: f32 },
    Rectangle { width: u16, border_radius: f32 },
}

/// A set of rules that dictate the style of a slider.
pub trait StyleSheet {
    type Style: Default + Copy;

    /// Produces the style of an active slider.
    fn active(&self, style: Self::Style) -> Appearance;

    /// Produces the style of an hovered slider.
    fn hovered(&self, style: Self::Style) -> Appearance;

    /// Produces the style of a slider that is being dragged.
    fn dragging(&self, style: Self::Style) -> Appearance;
}
