use iced_core::{Background, Color};

pub struct Style {
   pub button_background: Option<Background>,
   pub icon_color: Color,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            button_background: None,
            icon_color: Color::BLACK,
        }
    }
}

pub trait StyleSheet {
    fn active(&self) -> Style;

    fn pressed(&self) -> Style {
        self.active()
    }

    fn disabled(&self) -> Style {
        let active = self.active();
        Style {
            button_background: active.button_background.map(|bg| match bg {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            icon_color: Color {
                a: active.icon_color.a * 0.5,
                ..active.icon_color
            },
            ..active
        }
    }
}

struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Style {
        Style::default()
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
