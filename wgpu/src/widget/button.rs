//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: type.Button.html
//! [`State`]: struct.State.html
use crate::Renderer;
use iced_native::{Background, Color};

pub use iced_native::button::State;

/// A widget that produces a message when clicked.
///
/// This is an alias of an `iced_native` button with an `iced_wgpu::Renderer`.
pub type Button<'a, Message> = iced_native::Button<'a, Message, Renderer>;

#[derive(Debug)]
pub struct Style {
    pub shadow_offset: f32,
    pub background: Option<Background>,
    pub border_radius: u16,
    pub text_color: Color,
}

pub trait StyleSheet {
    fn active(&self) -> Style;

    fn hovered(&self) -> Style {
        let active = self.active();

        Style {
            shadow_offset: active.shadow_offset + 1.0,
            ..active
        }
    }

    fn pressed(&self) -> Style {
        Style {
            shadow_offset: 0.0,
            ..self.active()
        }
    }

    fn disabled(&self) -> Style {
        let active = self.active();

        Style {
            shadow_offset: 0.0,
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Style {
        Style {
            shadow_offset: 1.0,
            background: Some(Background::Color([0.5, 0.5, 0.5].into())),
            border_radius: 5,
            text_color: Color::BLACK,
        }
    }
}

impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
