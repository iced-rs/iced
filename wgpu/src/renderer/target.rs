use crate::{Renderer, Transformation};
use iced_native::window;

use raw_window_handle::HasRawWindowHandle;

/// A rendering target.
#[derive(Debug)]
pub struct Target {
    surface: wgpu::Surface,
    width: u16,
    height: u16,
    dpi: f32,
    transformation: Transformation,
    swap_chain: wgpu::SwapChain,
}

impl Target {
    pub(crate) fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub(crate) fn dpi(&self) -> f32 {
        self.dpi
    }

    pub(crate) fn transformation(&self) -> Transformation {
        self.transformation
    }

    pub(crate) fn next_frame(&mut self) -> wgpu::SwapChainOutput<'_> {
        self.swap_chain.get_next_texture()
    }
}

impl window::Target for Target {
    type Renderer = Renderer;

    fn new<W: HasRawWindowHandle>(
        window: &W,
        width: u16,
        height: u16,
        dpi: f32,
        renderer: &Renderer,
    ) -> Target {
        let surface = wgpu::Surface::create(window);
        let swap_chain =
            new_swap_chain(&surface, width, height, &renderer.device);

        Target {
            surface,
            width,
            height,
            dpi,
            transformation: Transformation::orthographic(width, height),
            swap_chain,
        }
    }

    fn resize(
        &mut self,
        width: u16,
        height: u16,
        dpi: f32,
        renderer: &Renderer,
    ) {
        self.width = width;
        self.height = height;
        self.dpi = dpi;
        self.transformation = Transformation::orthographic(width, height);
        self.swap_chain =
            new_swap_chain(&self.surface, width, height, &renderer.device);
    }
}

fn new_swap_chain(
    surface: &wgpu::Surface,
    width: u16,
    height: u16,
    device: &wgpu::Device,
) -> wgpu::SwapChain {
    device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: u32::from(width),
            height: u32::from(height),
            present_mode: wgpu::PresentMode::Vsync,
        },
    )
}
