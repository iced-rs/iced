//! Display a dropdown list of selectable values.
use crate::Renderer;

pub use iced_native::pick_list::State;
pub use iced_style::pick_list::{Style, StyleSheet};

/// A widget allowing the selection of a single value from a list of options.
pub type PickList<'a, T, Message, Backend> =
    iced_native::PickList<'a, T, Message, Renderer<Backend>>;
