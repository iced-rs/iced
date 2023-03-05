//! Change the appearance of a pick list.
use iced_core::{Background, Color};
use std::fmt::Debug;

/// The appearance of a pick list.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The text [`Color`] of the pick list.
    pub text_color: Color,
    /// The placeholder [`Color`] of the pick list.
    pub placeholder_color: Color,
    /// The handle [`Color`] of the pick list.
    pub handle_color: Color,
    /// The [`Background`] of the pick list.
    pub background: Background,
    /// The border radius of the pick list.
    pub border_radius: f32,
    /// The border width of the pick list.
    pub border_width: f32,
    /// The border color of the pick list.
    pub border_color: Color,
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet: Debug {
    /// The supported style of the [`StyleSheet`].
    type Style: Default + Clone;

    /// Produces the active [`Appearance`] of a pick list.
    fn active(&self, style: &<Self as StyleSheet>::Style) -> Appearance;

    /// Produces the hovered [`Appearance`] of a pick list.
    fn hovered(&self, style: &<Self as StyleSheet>::Style) -> Appearance;
}
