use iced_native::image;
use std::{
    collections::{HashMap, HashSet},
};
use guillotiere::{Allocation, AtlasAllocator, Size};
use debug_stub_derive::*;

#[derive(DebugStub)]
pub enum Memory {
    Host(::image::ImageBuffer<::image::Bgra<u8>, Vec<u8>>),
    Device(#[debug_stub="ReplacementValue"]Allocation),
    NotFound,
    Invalid,
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host(image) => image.dimensions(),
            Memory::Device(allocation) => {
                let size = &allocation.rectangle.size();
                (size.width as u32, size.height as u32)
            },
            Memory::NotFound => (1, 1),
            Memory::Invalid => (1, 1),
        }
    }
}

#[derive(Debug)]
pub struct Cache {
    map: HashMap<u64, Memory>,
    hits: HashSet<u64>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hits: HashSet::new(),
        }
    }

    pub fn load(&mut self, handle: &image::Handle) -> &mut Memory {
        if self.contains(handle) {
            return self.get(handle).unwrap();
        }

        let memory = match handle.data() {
            image::Data::Path(path) => {
                if let Ok(image) = ::image::open(path) {
                    Memory::Host(image.to_bgra())
                } else {
                    Memory::NotFound
                }
            }
            image::Data::Bytes(bytes) => {
                if let Ok(image) = ::image::load_from_memory(&bytes) {
                    Memory::Host(image.to_bgra())
                } else {
                    Memory::Invalid
                }
            }
        };

        self.insert(handle, memory);
        self.get(handle).unwrap()
    }

    pub fn upload(
        &mut self,
        handle: &image::Handle,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        allocator: &mut AtlasAllocator,
        atlas: &mut wgpu::Texture,
    ) -> &Memory {
        let _ = self.load(handle);

        let memory = self.map.get_mut(&handle.id()).unwrap();

        if let Memory::Host(image) = memory {
            let (width, height) = image.dimensions();
            let size = Size::new(width as i32, height as i32);

            let old_atlas_size = allocator.size();
            let allocation;

            loop {
                if let Some(a) = allocator.allocate(size) {
                    allocation = a;
                    break;
                }

                allocator.grow(allocator.size() * 2);
            }

            let new_atlas_size = allocator.size();

            if new_atlas_size != old_atlas_size {
                let new_atlas = device.create_texture(&wgpu::TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: new_atlas_size.width as u32,
                        height: new_atlas_size.height as u32,
                        depth: 1,
                    },
                    array_layer_count: 1,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUsage::COPY_DST
                        | wgpu::TextureUsage::COPY_SRC
                        | wgpu::TextureUsage::SAMPLED,
                });

                encoder.copy_texture_to_texture(
                    wgpu::TextureCopyView {
                        texture: atlas,
                        array_layer: 0,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    wgpu::TextureCopyView {
                        texture: &new_atlas,
                        array_layer: 0,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    wgpu::Extent3d {
                        width: old_atlas_size.width as u32,
                        height: old_atlas_size.height as u32,
                        depth: 1,
                    }
                );

                *atlas = new_atlas;
            }

            let extent = wgpu::Extent3d {
                width,
                height,
                depth: 1,
            };

            let temp_buf = {
                let flat_samples = image.as_flat_samples();
                let slice = flat_samples.as_slice();

                device
                    .create_buffer_mapped(
                        slice.len(),
                        wgpu::BufferUsage::COPY_SRC,
                    )
                    .fill_from_slice(slice)
            };

            encoder.copy_buffer_to_texture(
                wgpu::BufferCopyView {
                    buffer: &temp_buf,
                    offset: 0,
                    row_pitch: 4 * width,
                    image_height: height,
                },
                wgpu::TextureCopyView {
                    texture: atlas,
                    array_layer: 0,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: allocation.rectangle.min.x as f32,
                        y: allocation.rectangle.min.y as f32,
                        z: 0.0,
                    },
                },
                extent,
            );

            *memory = Memory::Device(allocation);
        }

        memory
    }

    pub fn trim(&mut self, allocator: &mut AtlasAllocator) {
        let hits = &self.hits;

        for (id, mem) in &mut self.map {
            if let Memory::Device(allocation) = mem {
                if !hits.contains(&id) {
                    allocator.deallocate(allocation.id);
                }
            }
        }

        self.map.retain(|k, _| hits.contains(k));
        self.hits.clear();
    }

    fn get(&mut self, handle: &image::Handle) -> Option<&mut Memory> {
        let _ = self.hits.insert(handle.id());

        self.map.get_mut(&handle.id())
    }

    fn insert(&mut self, handle: &image::Handle, memory: Memory) {
        let _ = self.map.insert(handle.id(), memory);
    }

    fn contains(&self, handle: &image::Handle) -> bool {
        self.map.contains_key(&handle.id())
    }
}
