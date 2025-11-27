pub mod entry;

mod allocation;
mod allocator;
mod layer;

pub use allocation::Allocation;
pub use entry::Entry;
pub use layer::Layer;

use allocator::Allocator;

pub const DEFAULT_SIZE: u32 = 2048;
pub const MAX_SIZE: u32 = 2048;

use crate::core::Size;
use crate::graphics::color;

use std::sync::Arc;

#[derive(Debug)]
pub struct Atlas {
    size: u32,
    backend: wgpu::Backend,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_bind_group: Arc<wgpu::BindGroup>,
    texture_layout: wgpu::BindGroupLayout,
    layers: Vec<Layer>,
}

impl Atlas {
    pub fn new(
        device: &wgpu::Device,
        backend: wgpu::Backend,
        texture_layout: wgpu::BindGroupLayout,
    ) -> Self {
        Self::with_size(device, backend, texture_layout, DEFAULT_SIZE)
    }

    pub fn with_size(
        device: &wgpu::Device,
        backend: wgpu::Backend,
        texture_layout: wgpu::BindGroupLayout,
        size: u32,
    ) -> Self {
        let size = size.min(MAX_SIZE);

        let layers = match backend {
            // On the GL backend we start with 2 layers, to help wgpu figure
            // out that this texture is `GL_TEXTURE_2D_ARRAY` rather than `GL_TEXTURE_2D`
            // https://github.com/gfx-rs/wgpu/blob/004e3efe84a320d9331371ed31fa50baa2414911/wgpu-hal/src/gles/mod.rs#L371
            wgpu::Backend::Gl => vec![Layer::Empty, Layer::Empty],
            _ => vec![Layer::Empty],
        };

        let extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers.len() as u32,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_wgpu::image texture atlas"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if color::GAMMA_CORRECTION {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::image texture atlas bind group"),
                layout: &texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                }],
            });

        Atlas {
            size,
            backend,
            texture,
            texture_view,
            texture_bind_group: Arc::new(texture_bind_group),
            texture_layout,
            layers,
        }
    }

    pub fn bind_group(&self) -> &Arc<wgpu::BindGroup> {
        &self.texture_bind_group
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Option<Entry> {
        let entry = {
            let current_size = self.layers.len();
            let entry = self.allocate(width, height)?;

            // We grow the internal texture after allocating if necessary
            let new_layers = self.layers.len() - current_size;
            self.grow(new_layers, device, encoder, self.backend);

            entry
        };

        log::debug!("Allocated atlas entry: {entry:?}");

        match &entry {
            Entry::Contiguous(allocation) => {
                self.upload_allocation(
                    pixels, width, 0, allocation, device, encoder, belt,
                );
            }
            Entry::Fragmented { fragments, .. } => {
                for fragment in fragments {
                    let (x, y) = fragment.position;
                    let offset = 4 * (y * width + x) as usize;

                    self.upload_allocation(
                        pixels,
                        width,
                        offset,
                        &fragment.allocation,
                        device,
                        encoder,
                        belt,
                    );
                }
            }
        }

        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "Atlas layers: {} (busy: {}, allocations: {})",
                self.layers.len(),
                self.layers.iter().filter(|layer| !layer.is_empty()).count(),
                self.layers.iter().map(Layer::allocations).sum::<usize>(),
            );
        }

        Some(entry)
    }

    pub fn remove(&mut self, entry: &Entry) {
        log::debug!("Removing atlas entry: {entry:?}");

        match entry {
            Entry::Contiguous(allocation) => {
                self.deallocate(allocation);
            }
            Entry::Fragmented { fragments, .. } => {
                for fragment in fragments {
                    self.deallocate(&fragment.allocation);
                }
            }
        }
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<Entry> {
        // Allocate one layer if texture fits perfectly
        if width == self.size && height == self.size {
            let mut empty_layers = self
                .layers
                .iter_mut()
                .enumerate()
                .filter(|(_, layer)| layer.is_empty());

            if let Some((i, layer)) = empty_layers.next() {
                *layer = Layer::Full;

                return Some(Entry::Contiguous(Allocation::Full {
                    layer: i,
                    size: self.size,
                }));
            }

            self.layers.push(Layer::Full);

            return Some(Entry::Contiguous(Allocation::Full {
                layer: self.layers.len() - 1,
                size: self.size,
            }));
        }

        // Split big textures across multiple layers
        if width > self.size || height > self.size {
            let mut fragments = Vec::new();
            let mut y = 0;

            while y < height {
                let height = std::cmp::min(height - y, self.size);
                let mut x = 0;

                while x < width {
                    let width = std::cmp::min(width - x, self.size);

                    let allocation = self.allocate(width, height)?;

                    if let Entry::Contiguous(allocation) = allocation {
                        fragments.push(entry::Fragment {
                            position: (x, y),
                            allocation,
                        });
                    }

                    x += width;
                }

                y += height;
            }

            return Some(Entry::Fragmented {
                size: Size::new(width, height),
                fragments,
            });
        }

        // Try allocating on an existing layer
        for (i, layer) in self.layers.iter_mut().enumerate() {
            match layer {
                Layer::Empty => {
                    let mut allocator = Allocator::new(self.size);

                    if let Some(region) = allocator.allocate(width, height) {
                        *layer = Layer::Busy(allocator);

                        return Some(Entry::Contiguous(Allocation::Partial {
                            region,
                            layer: i,
                            atlas_size: self.size,
                        }));
                    }
                }
                Layer::Busy(allocator) => {
                    if let Some(region) = allocator.allocate(width, height) {
                        return Some(Entry::Contiguous(Allocation::Partial {
                            region,
                            layer: i,
                            atlas_size: self.size,
                        }));
                    }
                }
                Layer::Full => {}
            }
        }

        // Create new layer with atlas allocator
        let mut allocator = Allocator::new(self.size);

        if let Some(region) = allocator.allocate(width, height) {
            self.layers.push(Layer::Busy(allocator));

            return Some(Entry::Contiguous(Allocation::Partial {
                region,
                layer: self.layers.len() - 1,
                atlas_size: self.size,
            }));
        }

        // We ran out of memory (?)
        None
    }

    fn deallocate(&mut self, allocation: &Allocation) {
        log::debug!("Deallocating atlas: {allocation:?}");

        match allocation {
            Allocation::Full { layer, .. } => {
                self.layers[*layer] = Layer::Empty;
            }
            Allocation::Partial { layer, region, .. } => {
                let layer = &mut self.layers[*layer];

                if let Layer::Busy(allocator) = layer {
                    allocator.deallocate(region);

                    if allocator.is_empty() {
                        *layer = Layer::Empty;
                    }
                }
            }
        }
    }

    fn upload_allocation(
        &self,
        pixels: &[u8],
        image_width: u32,
        offset: usize,
        allocation: &Allocation,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
    ) {
        let (x, y) = allocation.position();
        let Size { width, height } = allocation.size();
        let layer = allocation.layer();
        let padding = allocation.padding();

        // It is a webgpu requirement that:
        //   BufferCopyView.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
        // So we calculate bytes_per_row by rounding width up to the next
        // multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
        let bytes_per_row = (4 * (width + padding.width * 2))
            .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            as usize;
        let total_bytes =
            bytes_per_row * (height + padding.height * 2) as usize;

        let buffer_slice = belt.allocate(
            wgpu::BufferSize::new(total_bytes as u64).unwrap(),
            wgpu::BufferSize::new(8 * 4).unwrap(),
            device,
        );

        const PIXEL: usize = 4;

        let mut fragment = buffer_slice.get_mapped_range_mut();
        let w = width as usize;
        let h = height as usize;
        let pad_w = padding.width as usize;
        let pad_h = padding.height as usize;
        let stride = PIXEL * w;

        // Copy image rows
        for row in 0..h {
            let src = offset + row * PIXEL * image_width as usize;
            let dst = (row + pad_h) * bytes_per_row;

            fragment[dst + PIXEL * pad_w..dst + PIXEL * pad_w + stride]
                .copy_from_slice(&pixels[src..src + stride]);

            // Add padding to the sides, if needed
            for i in 0..pad_w {
                fragment[dst + PIXEL * i..dst + PIXEL * (i + 1)]
                    .copy_from_slice(&pixels[src..src + PIXEL]);

                fragment[dst + stride + PIXEL * (pad_w + i)
                    ..dst + stride + PIXEL * (pad_w + i + 1)]
                    .copy_from_slice(
                        &pixels[src + stride - PIXEL..src + stride],
                    );
            }
        }

        // Add padding on top and bottom
        for row in 0..pad_h {
            let dst_top = row * bytes_per_row;
            let dst_bottom = (pad_h + h + row) * bytes_per_row;
            let src_top = offset;
            let src_bottom = offset + (h - 1) * PIXEL * image_width as usize;

            // Top
            fragment[dst_top + PIXEL * pad_w..dst_top + PIXEL * (pad_w + w)]
                .copy_from_slice(&pixels[src_top..src_top + PIXEL * w]);

            // Bottom
            fragment
                [dst_bottom + PIXEL * pad_w..dst_bottom + PIXEL * (pad_w + w)]
                .copy_from_slice(&pixels[src_bottom..src_bottom + PIXEL * w]);

            // Corners
            for i in 0..pad_w {
                // Top left
                fragment[dst_top + PIXEL * i..dst_top + PIXEL * (i + 1)]
                    .copy_from_slice(&pixels[offset..offset + PIXEL]);

                // Top right
                fragment[dst_top + PIXEL * (w + pad_w + i)
                    ..dst_top + PIXEL * (w + pad_w + i + 1)]
                    .copy_from_slice(
                        &pixels[offset + PIXEL * (w - 1)..offset + PIXEL * w],
                    );

                // Bottom left
                fragment[dst_bottom + PIXEL * i..dst_bottom + PIXEL * (i + 1)]
                    .copy_from_slice(&pixels[src_bottom..src_bottom + PIXEL]);

                // Bottom right
                fragment[dst_bottom + PIXEL * (w + pad_w + i)
                    ..dst_bottom + PIXEL * (w + pad_w + i + 1)]
                    .copy_from_slice(
                        &pixels[src_bottom + PIXEL * (w - 1)
                            ..src_bottom + PIXEL * w],
                    );
            }
        }

        // Copy actual image
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: buffer_slice.buffer(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: buffer_slice.offset(),
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: Some(height + padding.height * 2),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x - padding.width,
                    y: y - padding.height,
                    z: layer as u32,
                },
                aspect: wgpu::TextureAspect::default(),
            },
            wgpu::Extent3d {
                width: width + padding.width * 2,
                height: height + padding.height * 2,
                depth_or_array_layers: 1,
            },
        );
    }

    fn grow(
        &mut self,
        amount: usize,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        backend: wgpu::Backend,
    ) {
        if amount == 0 {
            return;
        }

        // On the GL backend if layers.len() == 6 we need to help wgpu figure out that this texture
        // is still a `GL_TEXTURE_2D_ARRAY` rather than `GL_TEXTURE_CUBE_MAP`. This will over-allocate
        // some unused memory on GL, but it's better than not being able to grow the atlas past a depth
        // of 6!
        // https://github.com/gfx-rs/wgpu/blob/004e3efe84a320d9331371ed31fa50baa2414911/wgpu-hal/src/gles/mod.rs#L371
        let depth_or_array_layers = match backend {
            wgpu::Backend::Gl if self.layers.len() == 6 => 7,
            _ => self.layers.len() as u32,
        };

        let new_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_wgpu::image texture atlas"),
            size: wgpu::Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if color::GAMMA_CORRECTION {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let amount_to_copy = self.layers.len() - amount;

        for (i, layer) in
            self.layers.iter_mut().take(amount_to_copy).enumerate()
        {
            if layer.is_empty() {
                continue;
            }

            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::default(),
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &new_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::default(),
                },
                wgpu::Extent3d {
                    width: self.size,
                    height: self.size,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.texture = new_texture;
        self.texture_view =
            self.texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });

        self.texture_bind_group =
            Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::image texture atlas bind group"),
                layout: &self.texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.texture_view,
                    ),
                }],
            }));
    }
}
