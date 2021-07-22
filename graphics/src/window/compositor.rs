use crate::{Color, Error, Viewport};
use iced_native::mouse;
use raw_window_handle::HasRawWindowHandle;

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
    ) -> Result<mouse::Interaction, CompositorDrawError>;
}

/// Result of an unsuccessful call to [`Compositor::draw`].
#[derive(Debug)]
pub enum CompositorDrawError {
    /// The swapchain is outdated. Try rendering again next frame.
    SwapchainOutdated(Box<dyn std::error::Error>),
    /// A fatal swapchain error occured. Rendering cannot continue.
    FatalSwapchainError(Box<dyn std::error::Error>),
}

impl std::error::Error for CompositorDrawError {}

impl std::fmt::Display for CompositorDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositorDrawError::SwapchainOutdated(e) => write!(
                f,
                "Swapchain is outdated: {}. Try rendering next frame.",
                e
            ),
            CompositorDrawError::FatalSwapchainError(e) => write!(
                f,
                "Fatal swapchain error: {}. Rendering cannot continue.",
                e
            ),
        }
    }
}
