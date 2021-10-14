use crate::{Color, Error, Viewport};

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

    /// Configures a new [`Surface`] with the given dimensions.
    ///
    /// [`Surface`]: Self::Surface
    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    );

    /// Presents the [`Renderer`] primitives to the next frame of the given [`Surface`].
    ///
    /// [`SwapChain`]: Self::SwapChain
    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), SurfaceError>;
}

/// Result of an unsuccessful call to [`Compositor::draw`].
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum SurfaceError {
    /// A timeout was encountered while trying to acquire the next frame.
    #[error(
        "A timeout was encountered while trying to acquire the next frame"
    )]
    Timeout,
    /// The underlying surface has changed, and therefore the surface must be updated.
    #[error(
        "The underlying surface has changed, and therefore the surface must be updated."
    )]
    Outdated,
    /// The swap chain has been lost and needs to be recreated.
    #[error("The surface has been lost and needs to be recreated")]
    Lost,
    /// There is no more memory left to allocate a new frame.
    #[error("There is no more memory left to allocate a new frame")]
    OutOfMemory,
}
