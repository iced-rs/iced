/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message, Theme = iced_native::Theme> =
    crate::runtime::Element<'a, Message, crate::Renderer<Theme>>;
