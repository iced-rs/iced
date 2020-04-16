use crate::Viewport;

/// The rendering target of a window.
///
/// It represents a series of virtual framebuffers with a scale factor.
#[derive(Debug)]
pub struct SwapChain {
    raw: wgpu::SwapChain,
    viewport: Viewport,
}

impl SwapChain {}

impl SwapChain {
    /// Creates a new [`SwapChain`] for the given surface.
    ///
    /// [`SwapChain`]: struct.SwapChain.html
    pub fn new(
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> SwapChain {
        SwapChain {
            raw: new_swap_chain(surface, format, width, height, device),
            viewport: Viewport::new(width, height),
        }
    }

    /// Returns the next frame of the [`SwapChain`] alongside its [`Viewport`].
    ///
    /// [`SwapChain`]: struct.SwapChain.html
    /// [`Viewport`]: ../struct.Viewport.html
    pub fn next_frame(
        &mut self,
    ) -> Result<(wgpu::SwapChainOutput, &Viewport), wgpu::TimeOut> {
        let viewport = &self.viewport;

        self.raw.get_next_texture().map(|output| (output, viewport))
    }
}

fn new_swap_chain(
    surface: &wgpu::Surface,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    device: &wgpu::Device,
) -> wgpu::SwapChain {
    device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    )
}
