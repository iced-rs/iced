use crate::core::image as raster;
use crate::core::{Rectangle, Size};
use crate::graphics;

use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::collections::hash_map;
use std::fmt;
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

    pub fn load(&self, handle: &raster::Handle) -> Result<raster::Allocation, raster::Error> {
        let mut cache = self.cache.borrow_mut();
        let image = cache.allocate(handle)?;

        #[allow(unsafe_code)]
        Ok(unsafe { raster::allocate(handle, Size::new(image.width(), image.height())) })
    }

    pub fn dimensions(&self, handle: &raster::Handle) -> Option<Size<u32>> {
        let mut cache = self.cache.borrow_mut();
        let image = cache.allocate(handle).ok()?;

        Some(Size::new(image.width(), image.height()))
    }

    pub fn draw(
        &mut self,
        handle: &raster::Handle,
        filter_method: raster::FilterMethod,
        bounds: Rectangle,
        opacity: f32,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        transform: tiny_skia::Transform,
        clip_mask: Option<&tiny_skia::Mask>,
    ) {
        let mut cache = self.cache.borrow_mut();

        let Ok(image) = cache.allocate(handle) else {
            return;
        };

        let width_scale = bounds.width / image.width() as f32;
        let height_scale = bounds.height / image.height() as f32;

        let transform = transform.pre_scale(width_scale, height_scale);

        let quality = match filter_method {
            raster::FilterMethod::Linear => tiny_skia::FilterQuality::Bilinear,
            raster::FilterMethod::Nearest => tiny_skia::FilterQuality::Nearest,
        };

        pixels.draw_pixmap(
            (bounds.x / width_scale) as i32,
            (bounds.y / height_scale) as i32,
            image,
            &tiny_skia::PixmapPaint {
                quality,
                opacity,
                ..Default::default()
            },
            transform,
            clip_mask,
        );
    }

    pub fn trim_cache(&mut self) {
        self.cache.borrow_mut().trim();
    }
}

#[derive(Debug, Default)]
struct Cache {
    entries: FxHashMap<raster::Id, EntryState>,
    hits: FxHashSet<raster::Id>,
}

#[derive(Debug)]
enum EntryState {
    Ready(Entry),
    Error(raster::Error),
}

impl Cache {
    pub fn allocate(
        &mut self,
        handle: &raster::Handle,
    ) -> Result<tiny_skia::PixmapRef<'_>, raster::Error> {
        let id = handle.id();

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(id) {
            let image = match graphics::image::load(handle) {
                Ok(image) => image,
                Err(error) => {
                    let _ = entry.insert(EntryState::Error(error.clone()));
                    return Err(error);
                }
            };

            let error = if image.width() == 0 || image.height() == 0 {
                Some(raster::Error::Empty)
            } else {
                None
            };

            if let Some(error) = error {
                let _ = entry.insert(EntryState::Error(error.clone()));
                return Err(error);
            }

            let mut buffer = vec![0u32; image.width() as usize * image.height() as usize];

            for (i, pixel) in image.pixels().enumerate() {
                let [r, g, b, a] = pixel.0;

                buffer[i] = bytemuck::cast(
                    tiny_skia::ColorU8::from_rgba(b, g, r, a).premultiply(),
                );
            }

            let _ = entry.insert(EntryState::Ready(Entry {
                width: image.width(),
                height: image.height(),
                pixels: buffer,
            }));
        }

        let _ = self.hits.insert(id);

        match self.entries.get(&id) {
            Some(EntryState::Ready(entry)) => {
                tiny_skia::PixmapRef::from_bytes(
                    bytemuck::cast_slice(&entry.pixels),
                    entry.width,
                    entry.height,
                )
                .ok_or_else(|| {
                    raster::Error::Invalid(Arc::new(PixmapBuildError {
                        width: entry.width,
                        height: entry.height,
                        pixels_len: entry.pixels.len(),
                    }))
                })
            }
            Some(EntryState::Error(error)) => Err(error.clone()),
            None => Err(raster::Error::Invalid(Arc::new(MissingCacheEntryError(id)))),
        }
    }

    fn trim(&mut self) {
        self.entries.retain(|key, _| self.hits.contains(key));
        self.hits.clear();
    }
}

#[derive(Debug)]
struct Entry {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

#[derive(Debug)]
struct PixmapBuildError {
    width: u32,
    height: u32,
    pixels_len: usize,
}

impl fmt::Display for PixmapBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to build tiny-skia pixmap (width={}, height={}, pixels_len_u32={})",
            self.width, self.height, self.pixels_len
        )
    }
}

impl std::error::Error for PixmapBuildError {}

#[derive(Debug)]
struct MissingCacheEntryError(raster::Id);

impl fmt::Display for MissingCacheEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tiny-skia raster cache missing entry for image id {:?}", self.0)
    }
}

impl std::error::Error for MissingCacheEntryError {}

#[cfg(test)]
mod tests {
    use super::{Cache, Entry, EntryState};
    use crate::core::image as raster;

    #[test]
    fn allocate_returns_error_instead_of_panicking_on_invalid_pixmap_bytes() {
        let mut cache = Cache::default();

        let handle = raster::Handle::from_rgba(1, 1, vec![0u8; 4]);
        let id = handle.id();

        let _ = cache.entries.insert(
            id,
            EntryState::Ready(Entry {
                width: 1,
                height: 1,
                // Invalid: should be width * height pixels
                pixels: vec![],
            }),
        );

        let result = cache.allocate(&handle);
        assert!(matches!(result, Err(raster::Error::Invalid(_))));
    }

    #[test]
    fn allocate_reuses_cached_error_instead_of_panicking() {
        let mut cache = Cache::default();

        let handle = raster::Handle::from_rgba(1, 1, vec![0u8; 4]);
        let id = handle.id();

        let _ = cache.entries.insert(id, EntryState::Error(raster::Error::Empty));

        let result = cache.allocate(&handle);
        assert!(matches!(result, Err(raster::Error::Empty)));
    }
}
