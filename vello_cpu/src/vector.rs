use crate::core::svg::{Data, Handle};
use crate::core::{Color, Rectangle, Size};

use resvg::usvg;
use rustc_hash::{FxHashMap, FxHashSet};
use tiny_skia::Transform;

use std::cell::RefCell;
use std::collections::hash_map;
use std::fs;
use std::sync::Arc;

#[derive(Debug)]
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
        opacity: f32,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        transform: Transform,
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
                &tiny_skia::PixmapPaint {
                    opacity,
                    ..tiny_skia::PixmapPaint::default()
                },
                transform,
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
    fontdb: Option<Arc<usvg::fontdb::Database>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RasterKey {
    id: u64,
    color: Option<[u8; 4]>,
    size: Size<u32>,
}

impl Cache {
    fn load(&mut self, handle: &Handle) -> Option<&usvg::Tree> {
        let id = handle.id();

        // TODO: Reuse `cosmic-text` font database
        if self.fontdb.is_none() {
            let mut fontdb = usvg::fontdb::Database::new();
            fontdb.load_system_fonts();

            self.fontdb = Some(Arc::new(fontdb));
        }

        let options = usvg::Options {
            fontdb: self
                .fontdb
                .as_ref()
                .expect("fontdb must be initialized")
                .clone(),
            ..usvg::Options::default()
        };

        if let hash_map::Entry::Vacant(entry) = self.trees.entry(id) {
            let svg = match handle.data() {
                Data::Path(path) => {
                    fs::read_to_string(path).ok().and_then(|contents| {
                        usvg::Tree::from_str(&contents, &options).ok()
                    })
                }
                Data::Bytes(bytes) => {
                    usvg::Tree::from_data(bytes, &options).ok()
                }
            };

            let _ = entry.insert(svg);
        }

        let _ = self.tree_hits.insert(id);
        self.trees.get(&id).unwrap().as_ref()
    }

    fn viewport_dimensions(&mut self, handle: &Handle) -> Option<Size<u32>> {
        let tree = self.load(handle)?;
        let size = tree.size();

        Some(Size::new(size.width() as u32, size.height() as u32))
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

            let tree_size = tree.size().to_int_size();

            let target_size = if size.width > size.height {
                tree_size.scale_to_width(size.width)
            } else {
                tree_size.scale_to_height(size.height)
            };

            let transform = if let Some(target_size) = target_size {
                let tree_size = tree_size.to_size();
                let target_size = target_size.to_size();

                tiny_skia::Transform::from_scale(
                    target_size.width() / tree_size.width(),
                    target_size.height() / tree_size.height(),
                )
            } else {
                tiny_skia::Transform::default()
            };

            resvg::render(tree, transform, &mut image.as_mut());

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
                    *pixel = *pixel & 0xFF00_FF00
                        | ((0x0000_00FF & *pixel) << 16)
                        | ((0x00FF_0000 & *pixel) >> 16);
                }
            }

            let _ = self.rasters.insert(key, image);
        }

        let _ = self.raster_hits.insert(key);
        self.rasters.get(&key).map(tiny_skia::Pixmap::as_ref)
    }

    fn trim(&mut self) {
        self.trees.retain(|key, _| self.tree_hits.contains(key));
        self.rasters.retain(|key, _| self.raster_hits.contains(key));

        self.tree_hits.clear();
        self.raster_hits.clear();
    }
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cache")
            .field("tree_hits", &self.tree_hits)
            .field("rasters", &self.rasters)
            .field("raster_hits", &self.raster_hits)
            .finish_non_exhaustive()
    }
}
