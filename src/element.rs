/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
#[cfg(all(feature = "iced_sctk", feature = "iced_shm"))]
pub type Element<'a, Message> =
    iced_sctk::Element<'a, Message, iced_shm::Renderer>;

#[cfg(all(feature = "iced_winit", feature = "iced_wgpu"))]
pub type Element<'a, Message> =
    iced_winit::Element<'a, Message, iced_wgpu::Renderer>;

#[cfg(target_arch = "wasm32")]
pub use iced_web::Element;
