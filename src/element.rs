/// A generic widget.
///
/// This selects the `Renderer` for `iced_native` elements
#[cfg(not(target_arch = "wasm32"))]
pub type Element<'a, Message> = crate::runtime::Element<'a, Message, crate::renderer::Renderer>;

#[cfg(target_arch = "wasm32")]
pub use iced_web::Element;
