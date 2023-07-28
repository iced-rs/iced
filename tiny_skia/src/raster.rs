use crate::core::image as raster;
use crate::core::{Rectangle, Size};
use crate::graphics;

use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::collections::hash_map;

pub struct Pipeline {
    cache: RefCell<Cache>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(Cache::default()),
        }
    }

    pub fn dimensions(&self, handle: &raster::Handle) -> Size<u32> {
        if let Some(image) = self.cache.borrow_mut().allocate(handle) {
            Size::new(image.width(), image.height())
        } else {
            Size::new(0, 0)
        }
    }

    pub fn draw(
        &mut self,
        handle: &raster::Handle,
        bounds: Rectangle,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        transform: tiny_skia::Transform,
        clip_mask: Option<&tiny_skia::Mask>,
    ) {
        if let Some(image) = self.cache.borrow_mut().allocate(handle) {
            let width_scale = bounds.width / image.width() as f32;
            let height_scale = bounds.height / image.height() as f32;

            let transform = transform.pre_scale(width_scale, height_scale);

            pixels.draw_pixmap(
                (bounds.x / width_scale) as i32,
                (bounds.y / height_scale) as i32,
                image,
                &tiny_skia::PixmapPaint {
                    quality: tiny_skia::FilterQuality::Bilinear,
                    ..Default::default()
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
    entries: FxHashMap<u64, Option<Entry>>,
    hits: FxHashSet<u64>,
}

impl Cache {
    pub fn allocate(
        &mut self,
        handle: &raster::Handle,
    ) -> Option<tiny_skia::PixmapRef<'_>> {
        let id = handle.id();

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(id) {
            let image = graphics::image::load(handle).ok()?.into_rgba8();

            let mut buffer =
                vec![0u32; image.width() as usize * image.height() as usize];

            for (i, pixel) in image.pixels().enumerate() {
                let [r, g, b, a] = pixel.0;

                buffer[i] = bytemuck::cast(
                    tiny_skia::ColorU8::from_rgba(b, g, r, a).premultiply(),
                );
            }

            entry.insert(Some(Entry {
                width: image.width(),
                height: image.height(),
                pixels: buffer,
            }));
        }

        self.hits.insert(id);
        self.entries.get(&id).unwrap().as_ref().map(|entry| {
            tiny_skia::PixmapRef::from_bytes(
                bytemuck::cast_slice(&entry.pixels),
                entry.width,
                entry.height,
            )
            .expect("Build pixmap from image bytes")
        })
    }

    fn trim(&mut self) {
        self.entries.retain(|key, _| self.hits.contains(key));
        self.hits.clear();
    }
}

struct Entry {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}
