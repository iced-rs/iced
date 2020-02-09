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
    pub fn new(
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        width: u32,
        height: u32,
        scale_factor: f64,
    ) -> SwapChain {
        SwapChain {
            raw: new_swap_chain(surface, width, height, device),
            viewport: Viewport::new(width, height, scale_factor),
        }
    }

    pub fn next_frame(&mut self) -> (wgpu::SwapChainOutput<'_>, &Viewport) {
        (self.raw.get_next_texture(), &self.viewport)
    }
}

fn new_swap_chain(
    surface: &wgpu::Surface,
    width: u32,
    height: u32,
    device: &wgpu::Device,
) -> wgpu::SwapChain {
    device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::Vsync,
        },
    )
}
