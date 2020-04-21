use crate::Viewport;

/// The rendering target of a window.
///
/// It represents a series of virtual framebuffers with a scale factor.
#[derive(Debug)]
pub struct SwapChain {
    viewport: Viewport,
}

impl SwapChain {}

impl SwapChain {
    /// Creates a new [`SwapChain`] for the given surface.
    ///
    /// [`SwapChain`]: struct.SwapChain.html
    pub fn new(_surface: &(), width: u32, height: u32) -> SwapChain {
        SwapChain {
            viewport: Viewport::new(width, height),
        }
    }

    /// Returns the next frame of the [`SwapChain`] alongside its [`Viewport`].
    ///
    /// [`SwapChain`]: struct.SwapChain.html
    /// [`Viewport`]: ../struct.Viewport.html
    pub fn next_frame(&mut self) -> Result<((), &Viewport), ()> {
        let viewport = &self.viewport;
        Ok(((), viewport))
    }
}
