//! Display a dropdown list of selectable values.
use crate::backend::{self, Backend};
use crate::Renderer;

use iced_native::Padding;
use iced_style::menu;

pub use iced_native::pick_list::State;
pub use iced_style::pick_list::{Style, StyleSheet};

/// A widget allowing the selection of a single value from a list of options.
pub type PickList<'a, T, Message, Backend> =
    iced_native::PickList<'a, T, Message, Renderer<Backend>>;

impl<B> iced_native::pick_list::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_PADDING: Padding = Padding::new(5);

    fn menu_style(style: &Box<dyn StyleSheet>) -> menu::Style {
        style.menu()
    }
}
