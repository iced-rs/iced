/// A generic widget.
///
/// This selects the shm `Renderer` for `iced_native` elements
#[cfg(feature = "iced_shm")]
pub type Element<'a, Message> =
    crate::runtime::Element<'a, Message, iced_shm::Renderer>;

/// A generic widget.
///
/// This selects the wgpu `Renderer` for `iced_native` elements
#[cfg(feature = "iced_wgpu")]
pub type Element<'a, Message> =
    crate::runtime::Element<'a, Message, iced_wgpu::Renderer>;

#[cfg(target_arch = "wasm32")]
pub use iced_web::Element;
