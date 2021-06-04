//! Display a dropdown list of selectable values.
pub use iced_native::selection_list::State;

pub use iced_graphics::overlay::menu::Style as Menu;
pub use iced_graphics::selection_list::StyleSheet;

/// A widget allowing the selection of a single value from a list of options.
pub type SelectionList<'a, T, Message> =
    iced_native::SelectionList<'a, T, Message, crate::Renderer>;
