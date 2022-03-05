//! Decorate content and apply alignment.
use std::fmt::Debug;
use dyn_clone::DynClone;
use iced_core::{Background, Color};

/// The appearance of a container.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet: Debug + DynClone {
    /// Produces the style of a container.
    fn style(&self) -> Style;
}

dyn_clone::clone_trait_object!(StyleSheet);

#[derive(Debug, Clone)]
struct Default;

impl StyleSheet for Default {
    fn style(&self) -> Style {
        Style {
            text_color: None,
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet + 'a>
where
    T: StyleSheet + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
