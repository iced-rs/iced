use crate::{Color, Error, Size, Viewport};

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
    /// The renderer of the [`GLCompositor`].
    ///
    /// This should point to your renderer type, which could be a type alias
    /// of the [`Renderer`] provided in this crate with with a specific
    /// [`Backend`].
    ///
    /// [`Renderer`]: crate::Renderer
    /// [`Backend`]: crate::Backend
    type Renderer: iced_native::Renderer;

    /// The settings of the [`GLCompositor`].
    ///
    /// It's up to you to decide the configuration supported by your renderer!
    type Settings: Default;

    /// Creates a new [`GLCompositor`] and [`Renderer`] with the given
    /// [`Settings`] and an OpenGL address loader function.
    ///
    /// [`Renderer`]: crate::Renderer
    /// [`Backend`]: crate::Backend
    /// [`Settings`]: Self::Settings
    #[allow(unsafe_code)]
    unsafe fn new(
        settings: Self::Settings,
        loader_function: impl FnMut(&str) -> *const c_void,
    ) -> Result<(Self, Self::Renderer), Error>;

    /// Returns the amount of samples that should be used when configuring
    /// an OpenGL context for this [`GLCompositor`].
    fn sample_count(settings: &Self::Settings) -> u32;

    /// Resizes the viewport of the [`GLCompositor`].
    fn resize_viewport(&mut self, physical_size: Size<u32>);

    /// Presents the primitives of the [`Renderer`] to the next frame of the
    /// [`GLCompositor`].
    ///
    /// [`Renderer`]: crate::Renderer
    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    );
}
