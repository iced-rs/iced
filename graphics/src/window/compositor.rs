use crate::{Color, Error, Viewport};

use iced_native::mouse;

use raw_window_handle::HasRawWindowHandle;
use thiserror::Error;

/// A graphics compositor that can draw to windows.
pub trait Compositor: Sized {
    /// The settings of the backend.
    type Settings: Default;

    /// The iced renderer of the backend.
    type Renderer: iced_native::Renderer;

    /// The surface of the backend.
    type Surface;

    /// The swap chain of the backend.
    type SwapChain;

    /// Creates a new [`Compositor`].
    fn new<W: HasRawWindowHandle>(
        settings: Self::Settings,
        compatible_window: Option<&W>,
    ) -> Result<(Self, Self::Renderer), Error>;

    /// Crates a new [`Surface`] for the given window.
    ///
    /// [`Surface`]: Self::Surface
    fn create_surface<W: HasRawWindowHandle>(
        &mut self,
        window: &W,
    ) -> Self::Surface;

    /// Crates a new [`SwapChain`] for the given [`Surface`].
    ///
    /// [`SwapChain`]: Self::SwapChain
    /// [`Surface`]: Self::Surface
    fn create_swap_chain(
        &mut self,
        surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> Self::SwapChain;

    /// Draws the output primitives to the next frame of the given [`SwapChain`].
    ///
    /// [`SwapChain`]: Self::SwapChain
    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut Self::SwapChain,
        viewport: &Viewport,
        background_color: Color,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        overlay: &[T],
    ) -> Result<mouse::Interaction, SwapChainError>;
}

/// Result of an unsuccessful call to [`Compositor::draw`].
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum SwapChainError {
    /// A timeout was encountered while trying to acquire the next frame.
    #[error(
        "A timeout was encountered while trying to acquire the next frame"
    )]
    Timeout,
    /// The underlying surface has changed, and therefore the swap chain must be updated.
    #[error(
        "The underlying surface has changed, and therefore the swap chain must be updated."
    )]
    Outdated,
    /// The swap chain has been lost and needs to be recreated.
    #[error("The swap chain has been lost and needs to be recreated")]
    Lost,
    /// There is no more memory left to allocate a new frame.
    #[error("There is no more memory left to allocate a new frame")]
    OutOfMemory,
}
