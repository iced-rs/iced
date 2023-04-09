//! Display a widget over another.
/// A widget allowing the selection of a single value from a list of options.
pub type Tooltip<'a, Message> =
    iced_native::Tooltip<'a, Message, crate::Renderer>;

pub use iced_native::tooltip::Position;
