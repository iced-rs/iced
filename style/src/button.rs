//! Allow your users to perform actions by pressing a button.
use iced_core::{Background, Color, Vector};
use std::fmt::Debug;

/// The appearance of a button.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub shadow_offset: Vector,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a button.
pub trait StyleSheet {
    type Theme;

    fn get_style(
        &self,
        theme: &Self::Theme,
        is_disabled: bool,
        is_mouse_over: bool,
        is_pressed: bool,
    ) -> Style {
        if is_disabled {
            self.disabled(theme)
        } else if is_mouse_over {
            if is_pressed {
                self.pressed(theme)
            } else {
                self.hovered(theme)
            }
        } else {
            self.active(theme)
        }
    }

    fn active(&self, theme: &Self::Theme) -> Style;

    fn hovered(&self, theme: &Self::Theme) -> Style;

    fn pressed(&self, theme: &Self::Theme) -> Style;

    fn disabled(&self, theme: &Self::Theme) -> Style;
}

// impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
//     fn default() -> Self {
//         Box::new(Default)
//     }
// }
impl<'a, T, S> From<T> for Box<dyn StyleSheet<Theme = S> + 'a>
where
    T: StyleSheet<Theme = S> + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
