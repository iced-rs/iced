//! Allow your users to perform actions by pressing a button.
use iced_core::{Background, Color, Vector};

/// The appearance of a button.
#[derive(Debug, Clone)]
pub struct Style {
    pub shadow_offset: Vector,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Color,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            shadow_offset: Vector::default(),
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
        }
    }
}

/// A set of rules that dictate the style of a button.
pub trait StyleSheet {
    fn active(&self) -> Style;

    fn hovered(&self) -> Style {
        let active = self.active();

        Style {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self) -> Style {
        Style {
            shadow_offset: Vector::default(),
            ..self.active()
        }
    }

    fn disabled(&self) -> Style {
        let active = self.active();

        Style {
            shadow_offset: Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                Background::Image(_hdl, _alignment) => {
                    Background::Color(Color::BLACK)
                }
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
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color([0.87, 0.87, 0.87].into())),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            text_color: Color::BLACK,
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
