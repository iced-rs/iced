use crate::core::svg::{Data, Handle};
use crate::core::{Color, Rectangle, Size};

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
        color: Option<Color>,
        bounds: Rectangle,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::Mask>,
    ) {
        if let Some(image) = self.cache.borrow_mut().draw(
            handle,
            color,
            Size::new(bounds.width as u32, bounds.height as u32),
        ) {
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
    rasters: FxHashMap<RasterKey, tiny_skia::Pixmap>,
    raster_hits: FxHashSet<RasterKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RasterKey {
    id: u64,
    color: Option<[u8; 4]>,
    size: Size<u32>,
}

impl Cache {
    fn load(&mut self, handle: &Handle) -> Option<&usvg::Tree> {
        use usvg::TreeParsing;

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
        color: Option<Color>,
        size: Size<u32>,
    ) -> Option<tiny_skia::PixmapRef<'_>> {
        if size.width == 0 || size.height == 0 {
            return None;
        }

        let key = RasterKey {
            id: handle.id(),
            color: color.map(Color::into_rgba8),
            size,
        };

        #[allow(clippy::map_entry)]
        if !self.rasters.contains_key(&key) {
            let tree = self.load(handle)?;

            let mut image = tiny_skia::Pixmap::new(size.width, size.height)?;

            let tree_size = tree.size.to_int_size();
            let target_size;
            if size.width > size.height {
                target_size = tree_size.scale_to_width(size.width);
            } else {
                target_size = tree_size.scale_to_height(size.height);
            }
            let transform;
            if let Some(target_size) = target_size {
                let tree_size = tree_size.to_size();
                let target_size = target_size.to_size();
                transform = tiny_skia::Transform::from_scale(
                    target_size.width() / tree_size.width(),
                    target_size.height() / tree_size.height(),
                );
            } else {
                transform = tiny_skia::Transform::default();
            }

            resvg::Tree::from_usvg(tree).render(transform, &mut image.as_mut());

            if let Some([r, g, b, _]) = key.color {
                // Apply color filter
                for pixel in
                    bytemuck::cast_slice_mut::<u8, u32>(image.data_mut())
                {
                    *pixel = bytemuck::cast(
                        tiny_skia::ColorU8::from_rgba(
                            b,
                            g,
                            r,
                            (*pixel >> 24) as u8,
                        )
                        .premultiply(),
                    );
                }
            } else {
                // Swap R and B channels for `softbuffer` presentation
                for pixel in
                    bytemuck::cast_slice_mut::<u8, u32>(image.data_mut())
                {
                    *pixel = *pixel & 0xFF00FF00
                        | ((0x000000FF & *pixel) << 16)
                        | ((0x00FF0000 & *pixel) >> 16);
                }
            }

            self.rasters.insert(key, image);
        }

        self.raster_hits.insert(key);
        self.rasters.get(&key).map(tiny_skia::Pixmap::as_ref)
    }

    fn trim(&mut self) {
        self.trees.retain(|key, _| self.tree_hits.contains(key));
        self.rasters.retain(|key, _| self.raster_hits.contains(key));

        self.tree_hits.clear();
        self.raster_hits.clear();
    }
}
