/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message, Styling> =
    crate::runtime::Element<'a, Message, crate::renderer::Renderer, Styling>;
