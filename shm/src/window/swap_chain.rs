use crate::Viewport;
use smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface;
#[derive(derive_more::Deref, derive_more::DerefMut)] struct WlSurface_(WlSurface);
impl std::fmt::Debug for WlSurface_ { fn fmt(&self, _: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> { unimplemented!() } }

#[derive(Debug)]
///
pub struct SwapChain {
    surface: WlSurface_,
    viewport: Viewport,
}

impl SwapChain {
    /// Creates a new `SwapChain` for the given surface.
    pub fn new(surface: WlSurface, width: u32, height: u32) -> SwapChain { SwapChain { surface: WlSurface_(surface), viewport: Viewport::new(width, height)} }
    ///
    pub fn next_frame(&mut self) -> Result<(&WlSurface, &Viewport), ()> {
        Ok((&self.surface.0, &self.viewport))
    }
}
