use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use futures::task::SpawnExt;
use iced_native::futures;
use raw_window_handle::HasRawWindowHandle;

/// A window graphics backend for iced powered by `wgpu`.
#[allow(missing_debug_implementations)]
pub struct Compositor {
    settings: Settings,
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    format: wgpu::TextureFormat,
}

impl Compositor {
    const CHUNK_SIZE: u64 = 10 * 1024;

    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    pub async fn request<W: HasRawWindowHandle>(
        settings: Settings,
        compatible_window: Option<&W>,
    ) -> Option<Self> {
        let instance = wgpu::Instance::new(settings.internal_backend);

        #[allow(unsafe_code)]
        let compatible_surface = compatible_window
            .map(|window| unsafe { instance.create_surface(window) });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: if settings.antialiasing.is_none() {
                    wgpu::PowerPreference::LowPower
                } else {
                    wgpu::PowerPreference::HighPerformance
                },
                compatible_surface: compatible_surface.as_ref(),
                force_fallback_adapter: false,
            })
            .await?;

        let format = compatible_surface
            .as_ref()
            .and_then(|surface| surface.get_preferred_format(&adapter))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(
                        "iced_wgpu::window::compositor device descriptor",
                    ),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits {
                        max_bind_groups: 2,
                        ..wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .ok()?;

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();

        Some(Compositor {
            instance,
            settings,
            device,
            queue,
            staging_belt,
            local_pool,
            format,
        })
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    pub fn create_backend(&self) -> Backend {
        Backend::new(&self.device, self.settings, self.format)
    }
}

impl iced_graphics::window::Compositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = wgpu::Surface;

    fn new<W: HasRawWindowHandle>(
        settings: Self::Settings,
        compatible_window: Option<&W>,
    ) -> Result<(Self, Renderer), Error> {
        let compositor = futures::executor::block_on(Self::request(
            settings,
            compatible_window,
        ))
        .ok_or(Error::AdapterNotFound)?;

        let backend = compositor.create_backend();

        Ok((compositor, Renderer::new(backend)))
    }

    fn create_surface<W: HasRawWindowHandle>(
        &mut self,
        window: &W,
    ) -> wgpu::Surface {
        #[allow(unsafe_code)]
        unsafe {
            self.instance.create_surface(window)
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    ) {
        surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                present_mode: self.settings.present_mode,
                width,
                height,
            },
        );
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), iced_graphics::window::SurfaceError> {
        match surface.get_current_texture() {
            Ok(frame) => {
                let mut encoder = self.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("iced_wgpu encoder"),
                    },
                );

                let view = &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let _ =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(
                            "iced_wgpu::window::Compositor render pass",
                        ),
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear({
                                    let [r, g, b, a] =
                                        background_color.into_linear();

                                    wgpu::Color {
                                        r: f64::from(r),
                                        g: f64::from(g),
                                        b: f64::from(b),
                                        a: f64::from(a),
                                    }
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                renderer.with_primitives(|backend, primitives| {
                    backend.present(
                        &mut self.device,
                        &mut self.staging_belt,
                        &mut encoder,
                        view,
                        primitives,
                        viewport,
                        overlay,
                    );
                });

                // Submit work
                self.staging_belt.finish();
                self.queue.submit(Some(encoder.finish()));
                frame.present();

                // Recall staging buffers
                self.local_pool
                    .spawner()
                    .spawn(self.staging_belt.recall())
                    .expect("Recall staging belt");

                self.local_pool.run_until_stalled();

                Ok(())
            }
            Err(error) => match error {
                wgpu::SurfaceError::Timeout => {
                    Err(iced_graphics::window::SurfaceError::Timeout)
                }
                wgpu::SurfaceError::Outdated => {
                    Err(iced_graphics::window::SurfaceError::Outdated)
                }
                wgpu::SurfaceError::Lost => {
                    Err(iced_graphics::window::SurfaceError::Lost)
                }
                wgpu::SurfaceError::OutOfMemory => {
                    Err(iced_graphics::window::SurfaceError::OutOfMemory)
                }
            },
        }
    }
}
