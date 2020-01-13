use crate::image::AtlasArray;
use iced_native::image;
use std::{
    collections::{HashMap, HashSet},
};
use guillotiere::{Allocation, Size};
use debug_stub_derive::*;

#[derive(DebugStub)]
pub enum Memory {
    Host(::image::ImageBuffer<::image::Bgra<u8>, Vec<u8>>),
    Device {
        layer: u32,
        #[debug_stub="ReplacementValue"]
        allocation: Allocation,
    },
    NotFound,
    Invalid,
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host(image) => image.dimensions(),
            Memory::Device { allocation, .. } => {
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
        atlas_array: &mut AtlasArray,
    ) -> &Memory {
        let _ = self.load(handle);

        let memory = self.map.get_mut(&handle.id()).unwrap();

        if let Memory::Host(image) = memory {
            let (width, height) = image.dimensions();
            let size = Size::new(width as i32, height as i32);

            let (layer, allocation) = atlas_array.allocate(size).unwrap_or_else(|| {
                atlas_array.grow(1, device, encoder);
                atlas_array.allocate(size).unwrap()
            });

            let flat_samples = image.as_flat_samples();
            let slice = flat_samples.as_slice();

            atlas_array.upload(slice, layer, &allocation, device, encoder);

            *memory = Memory::Device { layer, allocation };
        }

        memory
    }

    pub fn trim(&mut self, atlas_array: &mut AtlasArray) {
        let hits = &self.hits;

        for (id, mem) in &self.map {
            if let Memory::Device { layer, allocation } = mem {
                if !hits.contains(&id) {
                    atlas_array.deallocate(*layer, allocation);
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
