use crate::{Backend, Color, Renderer, Settings};

use iced_graphics::Viewport;
use iced_native::{futures, mouse};
use raw_window_handle::HasRawWindowHandle;

/// A window graphics backend for iced powered by `wgpu`.
#[derive(Debug)]
pub struct Compositor {
    settings: Settings,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Compositor {
    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    ///
    /// [`Compositor`]: struct.Compositor.html
    /// [`Settings`]: struct.Settings.html
    pub async fn request(settings: Settings) -> Option<Self> {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: if settings.antialiasing.is_none() {
                    wgpu::PowerPreference::Default
                } else {
                    wgpu::PowerPreference::HighPerformance
                },
                compatible_surface: None,
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits { max_bind_groups: 2 },
            })
            .await;

        Some(Compositor {
            settings,
            device,
            queue,
        })
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    ///
    /// [`Compositor`]: struct.Compositor.html
    /// [`Backend`]: struct.Backend.html
    pub fn create_backend(&self) -> Backend {
        Backend::new(&self.device, self.settings)
    }
}

impl iced_graphics::window::Compositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = wgpu::Surface;
    type SwapChain = wgpu::SwapChain;

    fn new(settings: Self::Settings) -> (Self, Renderer) {
        let compositor = futures::executor::block_on(Self::request(settings))
            .expect("Could not find a suitable graphics adapter");

        let backend = compositor.create_backend();

        (compositor, Renderer::new(backend))
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
    ) -> Self::SwapChain {
        self.device.create_swap_chain(
            surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: self.settings.format,
                width,
                height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        )
    }

    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut Self::SwapChain,
        viewport: &Viewport,
        background_color: Color,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        overlay: &[T],
    ) -> mouse::Interaction {
        let frame = swap_chain.get_next_texture().expect("Next frame");

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: {
                    let [r, g, b, a] = background_color.into_linear();

                    wgpu::Color {
                        r: f64::from(r),
                        g: f64::from(g),
                        b: f64::from(b),
                        a: f64::from(a),
                    }
                },
            }],
            depth_stencil_attachment: None,
        });

        let mouse_interaction = renderer.backend_mut().draw(
            &mut self.device,
            &mut encoder,
            &frame.view,
            viewport,
            output,
            overlay,
        );

        self.queue.submit(&[encoder.finish()]);

        mouse_interaction
    }
}
