use crate::core::image;
use crate::core::Size;
use crate::graphics;
use crate::graphics::image::image_rs;
use crate::image::atlas::{self, Atlas};

use std::collections::{HashMap, HashSet};

/// Entry in cache corresponding to an image handle
#[derive(Debug)]
pub enum Memory {
    /// Image data on host
    Host(image_rs::ImageBuffer<image_rs::Rgba<u8>, Vec<u8>>),
    /// Storage entry
    Device(atlas::Entry),
    /// Image not found
    NotFound,
    /// Invalid image data
    Invalid,
}

impl Memory {
    /// Width and height of image
    pub fn dimensions(&self) -> Size<u32> {
        match self {
            Memory::Host(image) => {
                let (width, height) = image.dimensions();

                Size::new(width, height)
            }
            Memory::Device(entry) => entry.size(),
            Memory::NotFound => Size::new(1, 1),
            Memory::Invalid => Size::new(1, 1),
        }
    }
}

/// Caches image raster data
#[derive(Debug, Default)]
pub struct Cache {
    map: HashMap<u64, Memory>,
    hits: HashSet<u64>,
}

impl Cache {
    /// Load image
    pub fn load(&mut self, handle: &image::Handle) -> &mut Memory {
        if self.contains(handle) {
            return self.get(handle).unwrap();
        }

        let memory = match graphics::image::load(handle) {
            Ok(image) => Memory::Host(image.to_rgba8()),
            Err(image_rs::error::ImageError::IoError(_)) => Memory::NotFound,
            Err(_) => Memory::Invalid,
        };

        self.insert(handle, memory);
        self.get(handle).unwrap()
    }

    /// Load image and upload raster data
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        handle: &image::Handle,
        atlas: &mut Atlas,
    ) -> Option<&atlas::Entry> {
        let memory = self.load(handle);

        if let Memory::Host(image) = memory {
            let (width, height) = image.dimensions();

            let entry = atlas.upload(device, encoder, width, height, image)?;

            *memory = Memory::Device(entry);
        }

        if let Memory::Device(allocation) = memory {
            Some(allocation)
        } else {
            None
        }
    }

    /// Trim cache misses from cache
    pub fn trim(&mut self, atlas: &mut Atlas) {
        let hits = &self.hits;

        self.map.retain(|k, memory| {
            let retain = hits.contains(k);

            if !retain {
                if let Memory::Device(entry) = memory {
                    atlas.remove(entry);
                }
            }

            retain
        });

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
