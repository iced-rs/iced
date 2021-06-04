//! Display a dropdown list of selectable values.
use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::{
    mouse, Font, HorizontalAlignment, Padding, Point, Rectangle,
    VerticalAlignment,
};
use iced_style::menu;

pub use iced_native::selection_list::State;
pub use iced_style::selection_list::StyleSheet;

/// A widget allowing the selection of a single value from a list of options.
pub type SelectionList<'a, T, Message, Backend> =
    iced_native::SelectionList<'a, T, Message, Renderer<Backend>>;

impl<B> iced_native::selection_list::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    const DEFAULT_PADDING: Padding = Padding::new(5);

    type Style = Box<dyn StyleSheet>;

    fn menu_style(style: &Box<dyn StyleSheet>) -> menu::Style {
        style.menu()
    }

    fn draw(&mut self) -> Self::Output {
        (Primitive::None, mouse::Interaction::default())
    }
}
