pub mod entry;

mod allocation;
mod allocator;
mod layer;

pub use allocation::Allocation;
pub use entry::Entry;
pub use layer::Layer;

use allocator::Allocator;

pub const SIZE: u32 = 4096;

#[derive(Debug)]
pub struct Atlas {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    layers: Vec<Layer>,
}

impl Atlas {
    pub fn new(device: &wgpu::Device) -> Self {
        let extent = wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: 2,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        let texture_view = texture.create_default_view();

        Atlas {
            texture,
            texture_view,
            layers: vec![Layer::Empty, Layer::Empty],
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn upload<C>(
        &mut self,
        width: u32,
        height: u32,
        data: &[C],
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<Entry>
    where
        C: Copy + 'static,
    {
        let memory = {
            let current_size = self.layers.len();
            let memory = self.allocate(width, height)?;

            // We grow the internal texture after allocating if necessary
            let new_layers = self.layers.len() - current_size;
            self.grow(new_layers, device, encoder);

            memory
        };

        dbg!(&memory);

        let buffer = device
            .create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(data);

        match &memory {
            Entry::Contiguous(allocation) => {
                self.upload_texture(&buffer, 0, &allocation, encoder);
            }
            Entry::Fragmented { fragments, .. } => {
                for fragment in fragments {
                    let (x, y) = fragment.allocation.position();

                    let offset =
                        (y * height + x) as usize * std::mem::size_of::<C>();

                    self.upload_texture(
                        &buffer,
                        offset as u64,
                        &fragment.allocation,
                        encoder,
                    );
                }
            }
        }

        Some(memory)
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<Entry> {
        // Allocate one layer if texture fits perfectly
        if width == SIZE && height == SIZE {
            let mut empty_layers = self
                .layers
                .iter_mut()
                .enumerate()
                .filter(|(_, layer)| layer.is_empty());

            if let Some((i, layer)) = empty_layers.next() {
                *layer = Layer::Full;

                return Some(Entry::Contiguous(Allocation::Full { layer: i }));
            }

            self.layers.push(Layer::Full);

            return Some(Entry::Contiguous(Allocation::Full {
                layer: self.layers.len() - 1,
            }));
        }

        // Split big textures across multiple layers
        if width > SIZE || height > SIZE {
            let mut fragments = Vec::new();
            let mut y = 0;

            while y < height {
                let height = std::cmp::min(height - y, SIZE);
                let mut x = 0;

                while x < width {
                    let width = std::cmp::min(width - x, SIZE);

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
                size: (width, height),
                fragments,
            });
        }

        // Try allocating on an existing layer
        for (i, layer) in self.layers.iter_mut().enumerate() {
            match layer {
                Layer::Empty => {
                    let mut allocator = Allocator::new(SIZE);

                    if let Some(region) = allocator.allocate(width, height) {
                        *layer = Layer::Busy(allocator);

                        return Some(Entry::Contiguous(Allocation::Partial {
                            region,
                            layer: i,
                        }));
                    }
                }
                Layer::Busy(allocator) => {
                    if let Some(region) = allocator.allocate(width, height) {
                        return Some(Entry::Contiguous(Allocation::Partial {
                            region,
                            layer: i,
                        }));
                    }
                }
                _ => {}
            }
        }

        // Create new layer with atlas allocator
        let mut allocator = Allocator::new(SIZE);

        if let Some(region) = allocator.allocate(width, height) {
            self.layers.push(Layer::Busy(allocator));

            return Some(Entry::Contiguous(Allocation::Partial {
                region,
                layer: self.layers.len() - 1,
            }));
        }

        // We ran out of memory (?)
        None
    }

    fn upload_texture(
        &mut self,
        buffer: &wgpu::Buffer,
        offset: u64,
        allocation: &Allocation,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let (x, y) = allocation.position();
        let (width, height) = allocation.size();
        let layer = allocation.layer();

        let extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer,
                offset,
                row_pitch: 4 * width,
                image_height: height,
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                array_layer: layer as u32,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as f32,
                    y: y as f32,
                    z: 0.0,
                },
            },
            extent,
        );
    }

    fn grow(
        &mut self,
        amount: usize,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if amount == 0 {
            return;
        }

        let new_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: SIZE,
                height: SIZE,
                depth: 1,
            },
            array_layer_count: (self.layers.len() + amount) as u32,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        for (i, layer) in self.layers.iter_mut().enumerate() {
            if layer.is_empty() {
                continue;
            }

            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &self.texture,
                    array_layer: i as u32,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                wgpu::TextureCopyView {
                    texture: &new_texture,
                    array_layer: i as u32,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                wgpu::Extent3d {
                    width: SIZE,
                    height: SIZE,
                    depth: 1,
                },
            );
        }

        for _ in 0..amount {
            self.layers.push(Layer::Empty);
        }

        self.texture = new_texture;
    }
}
