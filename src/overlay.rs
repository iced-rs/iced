//! Build and show dropdown menus.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub mod menu {
    pub use iced_native::overlay::menu::{Appearance, State, StyleSheet};

    /// A widget that produces a message when clicked.
    pub type Menu<'a, Message, Renderer = crate::Renderer> =
        iced_native::overlay::Menu<'a, Message, Renderer>;
}
