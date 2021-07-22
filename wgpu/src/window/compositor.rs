use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use futures::{executor::block_on, task::SpawnExt};
use iced_native::{futures, mouse};
use iced_graphics::{Size, Rectangle};
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
    framebuffer: Option<Framebuffer>,
}

impl Compositor {
    const CHUNK_SIZE: u64 = 10 * 1024;

    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    pub async fn request<W: HasRawWindowHandle>(
        settings: Settings,
        viewport_size: Size<u32>,
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
            })
            .await?;

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

        #[cfg(feature = "offscreen")]
        let framebuffer = {
            let size = BufferDimensions::new(viewport_size.width as usize, viewport_size.height as usize);
            let output = device.create_buffer(&wgpu::BufferDescriptor{
                label: None,
                size: (size.padded_bytes_per_row * size.height) as u64,
                usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: false,
            });

            let target = device.create_texture(&wgpu::TextureDescriptor{
                size: wgpu::Extent3d{
                    width: size.width as u32,
                    height: size.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // format: wgpu::TextureFormat::Rgba8UnormSrgb,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::RENDER_ATTACHMENT,
                label: None,
            });

            Some(
                Framebuffer{
                    target,
                    output,
                    size
                }
            )
        };

        #[cfg(not(feature = "offscreen"))]
        let framebuffer = None;

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();

        Some(Compositor {
            instance,
            settings,
            device,
            queue,
            staging_belt,
            local_pool,
            framebuffer,
        })
    }

    /// Creates a new rendering [`Backend`] for this [`Compositor`].
    pub fn create_backend(&self) -> Backend {
        Backend::new(&self.device, self.settings)
    }
}

impl iced_graphics::window::Compositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = wgpu::Surface;
    type SwapChain = wgpu::SwapChain;

    fn new<W: HasRawWindowHandle>(
        settings: Self::Settings,
        viewport_size: Size<u32>,
        compatible_window: Option<&W>,
    ) -> Result<(Self, Renderer), Error> {
        let compositor = futures::executor::block_on(Self::request(
            settings,
            viewport_size,
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

    fn create_swap_chain(
        &mut self,
        surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> Self::SwapChain {
        self.device.create_swap_chain(
            surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                format: self.settings.format,
                present_mode: self.settings.present_mode,
                width,
                height,
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
        #[cfg(not(feature = "offscreen"))]
        let frame = swap_chain.get_current_frame().expect("Next frame");
        #[cfg(feature = "offscreen")]
        let frame = self.framebuffer.as_ref().expect("No framebuffer");
        
        #[cfg(not(feature = "offscreen"))]
        let view = &frame.output.view;

        #[cfg(feature = "offscreen")]
        let view = &frame.target.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("iced_wgpu encoder"),
            },
        );

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_wgpu::window::Compositor render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear({
                        let [r, g, b, a] = background_color.into_linear();

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

        let mouse_interaction = renderer.backend_mut().draw(
            &mut self.device,
            &mut self.staging_belt,
            &mut encoder,
            view,
            viewport,
            output,
            overlay,
        );

        #[cfg(feature = "offscreen")]
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &frame.target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &frame.output,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(frame.size.padded_bytes_per_row as u32)
                            .unwrap(),
                    ),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d{
                width: frame.size.width as u32,
                height: frame.size.height as u32,
                depth_or_array_layers: 1,
            },
        );

        // Submit work
        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));

        // Recall staging buffers
        self.local_pool
            .spawner()
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();

        mouse_interaction
    }

    fn read(&self, buffer: &mut [u8]){
        if let Some(frame) = &self.framebuffer{
            let buffer_slice = frame.output.slice(..);
            let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
            self.device.poll(wgpu::Maintain::Wait);

            if let Ok(()) = block_on(buffer_future){
                buffer.copy_from_slice(&buffer_slice.get_mapped_range());
            }
            
            frame.output.unmap();
        }
    }
}

// TODO: This struct and Swapchain should be interchangeable, maybe an enum?
struct Framebuffer{
    target: wgpu::Texture,
    output: wgpu::Buffer,   
    size: BufferDimensions,
}

// from https://github.com/gfx-rs/wgpu-rs/blob/master/examples/capture/main.rs
struct BufferDimensions {
    width: usize,
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = std::mem::size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}
