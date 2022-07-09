use iced_core::{Background, Color};

use crate::container;
use crate::menu;
use crate::scrollable;

/// The appearance of a pick list.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub text_color: Color,
    pub placeholder_color: Color,
    pub background: Background,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub icon_size: f32,
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet:
    container::StyleSheet + menu::StyleSheet + scrollable::StyleSheet
{
    type Style: Default + Copy + Into<<Self as menu::StyleSheet>::Style>;

    fn active(&self, style: <Self as StyleSheet>::Style) -> Appearance;

    /// Produces the style of a container.
    fn hovered(&self, style: <Self as StyleSheet>::Style) -> Appearance;
}
