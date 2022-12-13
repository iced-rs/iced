//! Display interactive elements on top of other widgets.

/// A generic [`Overlay`].
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
///
/// [`Overlay`]: iced_native::Overlay
pub type Element<'a, Message, Renderer = crate::Renderer> =
    iced_native::overlay::Element<'a, Message, Renderer>;

pub mod menu {
    //! Build and show dropdown menus.
    pub use iced_native::overlay::menu::{Appearance, State, StyleSheet};

    /// A widget that produces a message when clicked.
    pub type Menu<'a, Message, Renderer = crate::Renderer> =
        iced_native::overlay::Menu<'a, Message, Renderer>;
}
