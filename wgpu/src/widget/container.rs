use iced_native::{Background, Color};

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: u16,
}

pub trait StyleSheet {
    fn style(&self) -> Style {
        Style {
            text_color: None,
            background: None,
            border_radius: 0,
        }
    }
}

struct Default;

impl StyleSheet for Default {}

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
