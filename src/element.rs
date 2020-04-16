/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
pub type Element<'a, Message> =
    crate::runtime::Element<'a, Message, crate::renderer::Renderer>;

#[cfg(target_os = "ios")]
pub use iced_ios::Element;

#[cfg(target_arch = "wasm32")]
pub use iced_web::Element;

