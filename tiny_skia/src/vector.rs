use crate::core::svg::{Data, Handle};
use crate::core::{Rectangle, Size};

use resvg::usvg;
use rustc_hash::{FxHashMap, FxHashSet};

use std::cell::RefCell;
use std::collections::hash_map;
use std::fs;

pub struct Pipeline {
    cache: RefCell<Cache>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(Cache::default()),
        }
    }

    pub fn viewport_dimensions(&self, handle: &Handle) -> Size<u32> {
        self.cache
            .borrow_mut()
            .viewport_dimensions(handle)
            .unwrap_or(Size::new(0, 0))
    }

    pub fn draw(
        &mut self,
        handle: &Handle,
        bounds: Rectangle,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::ClipMask>,
    ) {
        if let Some(image) = self
            .cache
            .borrow_mut()
            .draw(handle, Size::new(bounds.width as u32, bounds.height as u32))
        {
            pixels.draw_pixmap(
                bounds.x as i32,
                bounds.y as i32,
                image,
                &tiny_skia::PixmapPaint::default(),
                tiny_skia::Transform::identity(),
                clip_mask,
            );
        }
    }

    pub fn trim_cache(&mut self) {
        self.cache.borrow_mut().trim();
    }
}

#[derive(Default)]
struct Cache {
    trees: FxHashMap<u64, Option<resvg::usvg::Tree>>,
    tree_hits: FxHashSet<u64>,
    rasters: FxHashMap<(u64, Size<u32>), tiny_skia::Pixmap>,
    raster_hits: FxHashSet<(u64, Size<u32>)>,
}

impl Cache {
    fn load(&mut self, handle: &Handle) -> Option<&usvg::Tree> {
        let id = handle.id();

        if let hash_map::Entry::Vacant(entry) = self.trees.entry(id) {
            let svg = match handle.data() {
                Data::Path(path) => {
                    fs::read_to_string(path).ok().and_then(|contents| {
                        usvg::Tree::from_str(
                            &contents,
                            &usvg::Options::default(),
                        )
                        .ok()
                    })
                }
                Data::Bytes(bytes) => {
                    usvg::Tree::from_data(bytes, &usvg::Options::default()).ok()
                }
            };

            entry.insert(svg);
        }

        self.tree_hits.insert(id);
        self.trees.get(&id).unwrap().as_ref()
    }

    fn viewport_dimensions(&mut self, handle: &Handle) -> Option<Size<u32>> {
        let tree = self.load(handle)?;

        Some(Size::new(
            tree.size.width() as u32,
            tree.size.height() as u32,
        ))
    }

    fn draw(
        &mut self,
        handle: &Handle,
        size: Size<u32>,
    ) -> Option<tiny_skia::PixmapRef<'_>> {
        if size.width == 0 || size.height == 0 {
            return None;
        }

        let id = handle.id();

        if !self.rasters.contains_key(&(id, size)) {
            let tree = self.load(handle)?;

            let mut image = tiny_skia::Pixmap::new(size.width, size.height)?;

            resvg::render(
                tree,
                if size.width > size.height {
                    usvg::FitTo::Width(size.width)
                } else {
                    usvg::FitTo::Height(size.height)
                },
                tiny_skia::Transform::default(),
                image.as_mut(),
            )?;

            // Swap R and B channels for `softbuffer` presentation
            for pixel in bytemuck::cast_slice_mut::<u8, u32>(image.data_mut()) {
                *pixel = *pixel & 0xFF00FF00
                    | ((0x000000FF & *pixel) << 16)
                    | ((0x00FF0000 & *pixel) >> 16);
            }

            self.rasters.insert((id, size), image);
        }

        self.raster_hits.insert((id, size));
        self.rasters.get(&(id, size)).map(tiny_skia::Pixmap::as_ref)
    }

    fn trim(&mut self) {
        self.trees.retain(|key, _| self.tree_hits.contains(key));
        self.rasters.retain(|key, _| self.raster_hits.contains(key));

        self.tree_hits.clear();
        self.raster_hits.clear();
    }
}
