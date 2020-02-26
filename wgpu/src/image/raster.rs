use crate::texture::atlas::{self, Atlas};
use iced_native::image;
use std::collections::{HashMap, HashSet};

pub enum Memory {
    Host(::image::ImageBuffer<::image::Bgra<u8>, Vec<u8>>),
    Device(atlas::Entry),
    NotFound,
    Invalid,
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host(image) => image.dimensions(),
            Memory::Device(entry) => entry.size(),
            Memory::NotFound => (1, 1),
            Memory::Invalid => (1, 1),
        }
    }
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Memory::Host(_) => write!(f, "Memory::Host"),
            Memory::Device(_) => write!(f, "Memory::Device"),
            Memory::NotFound => write!(f, "Memory::NotFound"),
            Memory::Invalid => write!(f, "Memory::Invalid"),
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
        atlas: &mut Atlas,
    ) -> Option<&atlas::Entry> {
        let memory = self.load(handle);

        if let Memory::Host(image) = memory {
            let (width, height) = image.dimensions();

            let allocation =
                atlas.upload(width, height, &image, device, encoder)?;

            dbg!("Uploaded");

            *memory = Memory::Device(allocation);
        }

        if let Memory::Device(allocation) = memory {
            Some(allocation)
        } else {
            None
        }
    }

    pub fn trim(&mut self) {
        let hits = &self.hits;

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
