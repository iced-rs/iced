use crate::image::{TextureArray, ImageAllocation};
use iced_native::image;
use std::{
    collections::{HashMap, HashSet},
};
use guillotiere::Size;
use debug_stub_derive::*;

#[derive(DebugStub)]
pub enum Memory {
    Host(::image::ImageBuffer<::image::Bgra<u8>, Vec<u8>>),
    Device(ImageAllocation),
    NotFound,
    Invalid,
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host(image) => image.dimensions(),
            Memory::Device(allocation) => allocation.size(),
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
        atlas_array: &mut TextureArray,
    ) -> &Memory {
        let _ = self.load(handle);

        let memory = self.map.get_mut(&handle.id()).unwrap();

        if let Memory::Host(image) = memory {
            let (width, height) = image.dimensions();
            let size = Size::new(width as i32, height as i32);

            let allocation = atlas_array.allocate(size);

            atlas_array.upload(image, &allocation, device, encoder);

            *memory = Memory::Device(allocation);
        }

        memory
    }

    pub fn trim(&mut self, texture_array: &mut TextureArray) {
        let hits = &self.hits;

        for (id, mem) in &self.map {
            if let Memory::Device(allocation) = mem {
                if !hits.contains(&id) {
                    texture_array.deallocate(allocation);
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
