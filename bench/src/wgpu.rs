use iced::futures::executor::block_on;
use iced_wgpu::wgpu;
use std::num::NonZeroU32;

fn wgpu_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter = block_on(wgpu::util::initialize_adapter_from_env_or_default(
        &instance,
        wgpu::Backends::PRIMARY,
        None,
    ))
    .unwrap();
    block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: adapter.features() & wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap()
}

fn create_wgpu_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> wgpu::Texture {
    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    };
    device.create_texture(&texture_desc)
}

pub struct WgpuBench {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    renderer: iced_wgpu::Renderer<iced::Theme>,
    viewport: iced_graphics::Viewport,
    staging_belt: wgpu::util::StagingBelt,
    width: u32,
    height: u32,
}

impl WgpuBench {
    pub fn new(width: u32, height: u32) -> Self {
        let (device, queue) = wgpu_device();
        let texture = create_wgpu_texture(&device, width, height);
        let renderer =
            iced_wgpu::Renderer::<iced::Theme>::new(iced_wgpu::Backend::new(
                &device,
                Default::default(),
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ));
        let viewport = iced_graphics::Viewport::with_physical_size(
            iced::Size::new(width, height),
            1.0,
        );
        let staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
        Self {
            device,
            queue,
            texture,
            renderer,
            viewport,
            staging_belt,
            width,
            height,
        }
    }
}

impl super::Bench for WgpuBench {
    type Backend = iced_wgpu::Backend;
    type RenderState = RenderState;
    const BACKEND_NAME: &'static str = "wgpu";

    fn clear(&self) -> RenderState {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );
        let view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        RenderState { encoder, view }
    }

    fn present(&mut self, mut state: RenderState) {
        self.renderer.with_primitives(|backend, primitive| {
            backend.present::<&str>(
                &self.device,
                &mut self.staging_belt,
                &mut state.encoder,
                &state.view,
                primitive,
                &self.viewport,
                &[],
            );
        });

        self.staging_belt.finish();
        self.queue.submit(Some(state.encoder.finish()));
        self.staging_belt.recall();

        self.device.poll(wgpu::MaintainBase::Wait);
    }

    fn read_pixels(&self) -> image_rs::RgbaImage {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );
        let real_bytes_per_row = self.width * 4;
        let bytes_per_row =
            if (real_bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) == 0 {
                real_bytes_per_row
            } else {
                (real_bytes_per_row / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT + 1)
                    * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            };
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(bytes_per_row * self.height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(bytes_per_row),
            rows_per_image: None,
        };
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout,
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |res| res.unwrap());
        self.device.poll(wgpu::MaintainBase::Wait);
        let range = slice.get_mapped_range();
        let pixels = if bytes_per_row == real_bytes_per_row {
            range.to_owned()
        } else {
            range
                .chunks(bytes_per_row as usize)
                .flat_map(|chunk| {
                    chunk.iter().take(real_bytes_per_row as usize)
                })
                .copied()
                .collect()
        };
        image_rs::RgbaImage::from_vec(self.width, self.height, pixels).unwrap()
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn renderer(&mut self) -> &mut iced_wgpu::Renderer<iced::Theme> {
        &mut self.renderer
    }
}

pub struct RenderState {
    encoder: wgpu::CommandEncoder,
    view: wgpu::TextureView,
}
