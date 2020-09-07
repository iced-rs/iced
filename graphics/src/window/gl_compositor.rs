use crate::{Color, Error, Size, Viewport};
use iced_native::mouse;

use core::ffi::c_void;

/// A basic OpenGL compositor.
///
/// A compositor is responsible for initializing a renderer and managing window
/// surfaces.
///
/// For now, this compositor only deals with a single global surface
/// for drawing. However, the trait will most likely change in the near future
/// to handle multiple surfaces at once.
///
/// If you implement an OpenGL renderer, you can implement this trait to ease
/// integration with existing windowing shells, like `iced_glutin`.
pub trait GLCompositor: Sized {
    /// The renderer of the [`Compositor`].
    ///
    /// This should point to your renderer type, which could be a type alias
    /// of the [`Renderer`] provided in this crate with with a specific
    /// [`Backend`].
    ///
    /// [`Compositor`]: trait.Compositor.html
    /// [`Renderer`]: ../struct.Renderer.html
    /// [`Backend`]: ../backend/trait.Backend.html
    type Renderer: iced_native::Renderer;

    /// The settings of the [`Compositor`].
    ///
    /// It's up to you to decide the configuration supported by your renderer!
    type Settings: Default;

    /// Creates a new [`Compositor`] and [`Renderer`] with the given
    /// [`Settings`] and an OpenGL address loader function.
    ///
    /// [`Compositor`]: trait.Compositor.html
    /// [`Renderer`]: #associatedtype.Renderer
    /// [`Backend`]: ../backend/trait.Backend.html
    #[allow(unsafe_code)]
    unsafe fn new(
        settings: Self::Settings,
        loader_function: impl FnMut(&str) -> *const c_void,
    ) -> Result<(Self, Self::Renderer), Error>;

    /// Returns the amount of samples that should be used when configuring
    /// an OpenGL context for this [`Compositor`].
    ///
    /// [`Compositor`]: trait.Compositor.html
    fn sample_count(settings: &Self::Settings) -> u32;

    /// Resizes the viewport of the [`Compositor`].
    ///
    /// [`Compositor`]: trait.Compositor.html
    fn resize_viewport(&mut self, physical_size: Size<u32>);

    /// Draws the provided output with the given [`Renderer`].
    ///
    /// [`Compositor`]: trait.Compositor.html
    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        overlay: &[T],
    ) -> mouse::Interaction;
}
