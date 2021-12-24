use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use futures::{executor::block_on, task::SpawnExt};
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
    frame_buffer: Option<Framebuffer>,
    size: BufferDimensions,
    surface: Option<wgpu::Surface>,
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

        #[cfg(target_arch = "wasm32")]
        let limits = wgpu::Limits::downlevel_webgl2_defaults()
            .using_resolution(adapter.limits());

        #[cfg(not(target_arch = "wasm32"))]
        let limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(
                        "iced_wgpu::window::compositor device descriptor",
                    ),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits {
                        max_bind_groups: 2,
                        ..limits
                    },
                },
                None,
            )
            .await
            .ok()?;

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();

        let frame_buffer = None;
        let surface = None;
        Some(Compositor {
            instance,
            settings,
            device,
            queue,
            staging_belt,
            local_pool,
            format,
            frame_buffer,
            surface,
            size: BufferDimensions::default(),
        })
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    pub fn create_backend(&self) -> Backend {
        Backend::new(&self.device, self.settings, self.format)
    }

    fn create_buffer(&self) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (self.size.padded_bytes_per_row * self.size.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn copy_texture_to_buffer(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        buffer: &wgpu::Buffer,
    ) {
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::default(),
            },
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(
                            self.size.padded_bytes_per_row as u32,
                        )
                        .unwrap(),
                    ),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: self.size.width as u32,
                height: self.size.height as u32,
                depth_or_array_layers: 1,
            },
        )
    }
    fn resize_framebuffer(&mut self, width: u32, height: u32) {
        let framebuffer = {
            let size = BufferDimensions::new(width as usize, height as usize);
            let output = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (size.padded_bytes_per_row * size.height) as u64,
                usage: wgpu::BufferUsages::MAP_READ
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let target = self.device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // format: wgpu::TextureFormat::Rgba8UnormSrgb,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: None,
            });

            self.size = size;
            Some(Framebuffer { target, output })
        };
        self.frame_buffer = framebuffer;
    }
}

impl iced_graphics::window::VirtualCompositor for Compositor {
    fn read(&self) -> Option<Vec<u8>> {
        let mut rv = Vec::new();
        if let Some(frame) = &self.frame_buffer {
            let buffer_slice = frame.output.slice(..);
            let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
            self.device.poll(wgpu::Maintain::Wait);

            if let Ok(()) = block_on(buffer_future) {
                rv.extend_from_slice(&buffer_slice.get_mapped_range());
            }

            frame.output.unmap();
            Some(rv)
        } else {
            let mut encoder = self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("iced_wgpu encoder"),
                },
            );

            let surface =
                self.surface.as_ref().expect("Surface not initialized");
            let buffer = self.create_buffer();
            let frame = surface.get_current_texture().unwrap();
                            self.copy_texture_to_buffer(&mut encoder, &frame.texture, &buffer);
            let buffer_slice = buffer.slice(..);
            let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
            self.device.poll(wgpu::Maintain::Wait);

            if let Ok(()) = block_on(buffer_future) {
                rv.extend_from_slice(&buffer_slice.get_mapped_range());
            }

            buffer.unmap();
            Some(rv)
        }
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

    fn initialize_surface<W: HasRawWindowHandle>(&mut self, window: &W) {
        #[allow(unsafe_code)]
        let surface = unsafe { self.instance.create_surface(window) };
        self.surface = Some(surface);
    }

    fn configure_surface(&mut self, width: u32, height: u32) {
        let surface = self.surface.as_mut().unwrap();
        self.size = BufferDimensions::new(width as usize, height as usize);
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
        if self.settings.headless {
            self.resize_framebuffer(width, height);
        }
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), iced_graphics::window::SurfaceError> {
        let surface = self.surface.as_mut().expect("Surface not initialized");
        match surface.get_current_texture() {
            Ok(frame) => {
                let mut encoder = self.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("iced_wgpu encoder"),
                    },
                );

                let texture_viewer = wgpu::TextureViewDescriptor::default();
                let owned_view =
                    if let Some(ref frame_buffer) = self.frame_buffer {
                        frame_buffer.target.create_view(&texture_viewer)
                    } else {
                        frame.texture.create_view(&texture_viewer)
                    };

                let view = &owned_view;

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
                if let Some(ref frame_buffer) = self.frame_buffer {
                    self.copy_texture_to_buffer(
                        &mut encoder,
                        &frame_buffer.target,
                        &frame_buffer.output,
                    );
                }

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

// TODO: This struct and Swapchain should be interchangeable, maybe an enum?
struct Framebuffer {
    target: wgpu::Texture,
    output: wgpu::Buffer,
}

// from https://github.com/gfx-rs/wgpu-rs/blob/master/examples/capture/main.rs
#[derive(Default)]
struct BufferDimensions {
    width: usize,
    height: usize,
    //unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = std::mem::size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding =
            (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row =
            unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            //unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}
