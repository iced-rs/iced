/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
#[cfg(not(all(feature = "web", target_arch = "wasm32")))]
pub type Element<'a, Message> =
    crate::runtime::Element<'a, Message, crate::renderer::Renderer>;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use iced_web::Element;
