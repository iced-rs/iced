//! Display a dropdown list of selectable values.
pub use iced_native::combo_box::State;

pub use iced_graphics::combo_box::{Style, StyleSheet};
pub use iced_graphics::overlay::menu::Style as Menu;

/// A widget allowing the selection of a single value from a list of options.
pub type ComboBox<'a, T, Message> =
    iced_native::ComboBox<'a, T, Message, crate::Renderer>;
