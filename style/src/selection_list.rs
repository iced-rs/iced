use crate::menu;

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    fn menu(&self) -> menu::Style;
}

struct Default;

impl StyleSheet for Default {
    fn menu(&self) -> menu::Style {
        menu::Style::default()
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
