use std::rc::Rc;

pub enum Memory {
    Host {
        image: image::ImageBuffer<image::Bgra<u8>, Vec<u8>>,
    },
    Device {
        bind_group: Rc<wgpu::BindGroup>,
        width: u32,
        height: u32,
    },
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host { image } => image.dimensions(),
            Memory::Device { width, height, .. } => (*width, *height),
        }
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Rc<wgpu::BindGroup> {
        match self {
            Memory::Host { image } => {
                let (width, height) = image.dimensions();

                let extent = wgpu::Extent3d {
                    width,
                    height,
                    depth: 1,
                };

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: extent,
                    array_layer_count: 1,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUsage::COPY_DST
                        | wgpu::TextureUsage::SAMPLED,
                });

                let slice = image.clone().into_raw();

                let temp_buf = device
                    .create_buffer_mapped(
                        slice.len(),
                        wgpu::BufferUsage::COPY_SRC,
                    )
                    .fill_from_slice(&slice[..]);

                encoder.copy_buffer_to_texture(
                    wgpu::BufferCopyView {
                        buffer: &temp_buf,
                        offset: 0,
                        row_pitch: 4 * width as u32,
                        image_height: height as u32,
                    },
                    wgpu::TextureCopyView {
                        texture: &texture,
                        array_layer: 0,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    extent,
                );

                let bind_group =
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: texture_layout,
                        bindings: &[wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &texture.create_default_view(),
                            ),
                        }],
                    });

                let bind_group = Rc::new(bind_group);

                *self = Memory::Device {
                    bind_group: bind_group.clone(),
                    width,
                    height,
                };

                bind_group
            }
            Memory::Device { bind_group, .. } => bind_group.clone(),
        }
    }
}
