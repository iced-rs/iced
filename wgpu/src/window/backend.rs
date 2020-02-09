use crate::{window::SwapChain, Renderer, Settings, Target};

use iced_native::MouseCursor;
use raw_window_handle::HasRawWindowHandle;

/// A window graphics backend for iced powered by `wgpu`.
#[derive(Debug)]
pub struct Backend {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl iced_native::window::Backend for Backend {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = wgpu::Surface;
    type SwapChain = SwapChain;

    fn new(settings: Self::Settings) -> (Backend, Renderer) {
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::all(),
        })
        .expect("Request adapter");

        let (mut device, queue) =
            adapter.request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits { max_bind_groups: 2 },
            });

        let renderer = Renderer::new(settings, &mut device);

        (Backend { device, queue }, renderer)
    }

    fn create_surface<W: HasRawWindowHandle>(
        &mut self,
        window: &W,
    ) -> wgpu::Surface {
        wgpu::Surface::create(window)
    }

    fn create_swap_chain(
        &mut self,
        surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> SwapChain {
        SwapChain::new(&self.device, surface, width, height)
    }

    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut SwapChain,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        scale_factor: f64,
        overlay: &[T],
    ) -> MouseCursor {
        let (frame, viewport) = swap_chain.next_frame();

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { todo: 0 },
        );

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        });

        let mouse_cursor = renderer.draw(
            &mut self.device,
            &mut encoder,
            Target {
                texture: &frame.view,
                viewport,
            },
            output,
            scale_factor,
            overlay,
        );

        self.queue.submit(&[encoder.finish()]);

        mouse_cursor
    }
}
