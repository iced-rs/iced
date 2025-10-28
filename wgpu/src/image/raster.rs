use crate::core::Size;
use crate::core::image;
use crate::graphics;
use crate::image::atlas::{self, Atlas};

use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::{Arc, Weak};

pub type Image = graphics::image::Buffer;

/// Entry in cache corresponding to an image handle
#[derive(Debug)]
pub enum Memory {
    /// Image data on host
    Host(Image),
    /// Storage entry
    Device {
        entry: atlas::Entry,
        bind_group: Option<Arc<wgpu::BindGroup>>,
        allocation: Option<Weak<image::Memory>>,
    },
    Error(image::Error),
}

impl Memory {
    pub fn load(handle: &image::Handle) -> Self {
        match graphics::image::load(handle) {
            Ok(image) => Self::Host(image),
            Err(error) => Self::Error(error),
        }
    }

    pub fn dimensions(&self) -> Size<u32> {
        match self {
            Memory::Host(image) => {
                let (width, height) = image.dimensions();

                Size::new(width, height)
            }
            Memory::Device { entry, .. } => entry.size(),
            Memory::Error(_) => Size::new(1, 1),
        }
    }

    pub fn host(&self) -> Option<Image> {
        match self {
            Memory::Host(image) => Some(image.clone()),
            Memory::Device { .. } | Memory::Error(_) => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Cache {
    map: FxHashMap<image::Id, Memory>,
    hits: FxHashSet<image::Id>,
    should_trim: bool,
}

impl Cache {
    pub fn get_mut(&mut self, handle: &image::Handle) -> Option<&mut Memory> {
        let _ = self.hits.insert(handle.id());

        self.map.get_mut(&handle.id())
    }

    pub fn insert(&mut self, handle: &image::Handle, memory: Memory) {
        let _ = self.map.insert(handle.id(), memory);
        let _ = self.hits.insert(handle.id());

        self.should_trim = true;
    }

    pub fn contains(&self, handle: &image::Handle) -> bool {
        self.map.contains_key(&handle.id())
    }

    pub fn trim(
        &mut self,
        atlas: &mut Atlas,
        on_drop: impl Fn(Arc<wgpu::BindGroup>),
    ) {
        // Only trim if new entries have landed in the `Cache`
        if !self.should_trim {
            return;
        }

        let hits = &self.hits;

        self.map.retain(|id, memory| {
            // Retain active allocations
            if let Memory::Device { allocation, .. } = memory
                && allocation
                    .as_ref()
                    .is_some_and(|allocation| allocation.strong_count() > 0)
            {
                return true;
            }

            let retain = hits.contains(id);

            if !retain {
                log::debug!("Dropping image allocation: {id:?}");

                if let Memory::Device {
                    entry, bind_group, ..
                } = memory
                {
                    if let Some(bind_group) = bind_group.take() {
                        on_drop(bind_group);
                    } else {
                        atlas.remove(entry);
                    }
                }
            }

            retain
        });

        self.hits.clear();
        self.should_trim = false;
    }
}
