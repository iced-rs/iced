use crate::mouse;

use raw_window_handle::HasRawWindowHandle;

/// A graphics backend that can render to windows.
pub trait Backend: Sized {
    /// The settings of the backend.
    type Settings: Default;

    /// The iced renderer of the backend.
    type Renderer: crate::Renderer;

    /// The surface of the backend.
    type Surface;

    /// The swap chain of the backend.
    type SwapChain;

    /// Creates a new [`Backend`] and an associated iced renderer.
    ///
    /// [`Backend`]: trait.Backend.html
    fn new(settings: Self::Settings) -> (Self, Self::Renderer);

    /// Crates a new [`Surface`] for the given window.
    ///
    /// [`Surface`]: #associatedtype.Surface
    fn create_surface<W: HasRawWindowHandle>(
        &mut self,
        window: &W,
    ) -> Self::Surface;

    /// Crates a new [`SwapChain`] for the given [`Surface`].
    ///
    /// [`SwapChain`]: #associatedtype.SwapChain
    /// [`Surface`]: #associatedtype.Surface
    fn create_swap_chain(
        &mut self,
        surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> Self::SwapChain;

    /// Draws the output primitives to the next frame of the given [`SwapChain`].
    ///
    /// [`SwapChain`]: #associatedtype.SwapChain
    /// [`Surface`]: #associatedtype.Surface
    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut Self::SwapChain,
        output: &<Self::Renderer as crate::Renderer>::Output,
        scale_factor: f64,
        overlay: &[T],
    ) -> mouse::Interaction;
}
