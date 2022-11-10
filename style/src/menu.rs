//! Change the appearance of menus.
use iced_core::{Background, Color};

/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The text [`Color`] of the menu.
    pub text_color: Color,
    /// The [`Background`] of the menu.
    pub background: Background,
    /// The border width of the menu.
    pub border_width: f32,
    /// The border radius of the menu.
    pub border_radius: f32,
    /// The border [`Color`] of the menu.
    pub border_color: Color,
    /// The text [`Color`] of a selected option in the menu.
    pub selected_text_color: Color,
    /// The background [`Color`] of a selected option in the menu.
    pub selected_background: Background,
}

/// The style sheet of a menu.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default + Clone;

    /// Produces the [`Appearance`] of a menu.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
