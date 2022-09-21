//! Display interactive elements on top of other widgets.
pub mod menu {
    //! Build and show dropdown menus.
    pub use iced_native::overlay::menu::{Appearance, State, StyleSheet};

    /// A widget that produces a message when clicked.
    pub type Menu<'a, Message, Renderer = crate::Renderer> =
        iced_native::overlay::Menu<'a, Message, Renderer>;
}
