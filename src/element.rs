/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
#[cfg(not(feature = "web"))]
pub type Element<'a, Message> =
    crate::runtime::Element<'a, Message, crate::renderer::Renderer>;

#[cfg(feature = "web")]
pub use iced_web::Element;
