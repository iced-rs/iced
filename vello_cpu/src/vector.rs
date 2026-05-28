use crate::core::svg::{Data, Handle, Svg};
use crate::core::{Color, Rectangle, Size};

use resvg::tiny_skia;
use resvg::usvg;
use rustc_hash::{FxHashMap, FxHashSet};

use std::cell::RefCell;
use std::collections::hash_map;
use std::fs;
use std::panic;
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
        svg: &Svg,
        bounds: Rectangle,
        renderer: &mut vello_cpu::RenderContext,
        scale_factor: f32,
    ) {
        let raster = self.cache.borrow_mut().draw(
            &svg.handle,
            svg.color,
            Size::new(
                (bounds.width * scale_factor) as u32,
                (bounds.height * scale_factor) as u32,
            ),
        );

        let Some(raster) = raster else {
            return;
        };

        let width = f32::from(raster.width());
        let height = f32::from(raster.height());
        let width_scale = bounds.width / width;
        let height_scale = bounds.height / height;

        let transform = vello_cpu::kurbo::Affine::translate(vello_cpu::kurbo::Vec2::new(
            -f64::from(width) / 2.0,
            -f64::from(height) / 2.0,
        ))
        .then_rotate(f64::from(svg.rotation.0))
        .then_translate(vello_cpu::kurbo::Vec2::new(
            f64::from(width) / 2.0,
            f64::from(height) / 2.0,
        ))
        .then_scale_non_uniform(f64::from(width_scale), f64::from(height_scale))
        .then_translate(vello_cpu::kurbo::Vec2::new(
            f64::from(bounds.x),
            f64::from(bounds.y),
        ))
        .then_scale(f64::from(scale_factor));

        renderer.set_paint(vello_cpu::peniko::Brush::Image(
            vello_cpu::peniko::ImageBrush {
                image: vello_cpu::ImageSource::Pixmap(raster),
                sampler: vello_cpu::peniko::ImageSampler::new()
                    .with_quality(vello_cpu::peniko::ImageQuality::Low),
                // .with_alpha(image.opacity), TODO: Enable once `vello_cpu` supports it
            },
        ));

        renderer.set_transform(transform);

        renderer.fill_rect(&crate::into_rect(Rectangle {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }));

        renderer.reset_transform();
    }

    pub fn trim_cache(&mut self) {
        self.cache.borrow_mut().trim();
    }
}

#[derive(Default)]
struct Cache {
    trees: FxHashMap<u64, Option<resvg::usvg::Tree>>,
    tree_hits: FxHashSet<u64>,
    rasters: FxHashMap<RasterKey, Arc<vello_cpu::Pixmap>>,
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
                Data::Path(path) => fs::read_to_string(path)
                    .ok()
                    .and_then(|contents| usvg::Tree::from_str(&contents, &options).ok()),
                Data::Bytes(bytes) => usvg::Tree::from_data(bytes, &options).ok(),
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
    ) -> Option<Arc<vello_cpu::Pixmap>> {
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

            let mut image = vello_cpu::Pixmap::new(size.width as u16, size.height as u16);

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

            // SVG rendering can panic on malformed or complex vectors.
            // We catch panics to prevent crashes and continue gracefully.
            let render = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                resvg::render(
                    tree,
                    transform,
                    &mut tiny_skia::PixmapMut::from_bytes(
                        image.data_as_u8_slice_mut(),
                        size.width,
                        size.height,
                    )
                    .expect("Build mutable pixmap"),
                );
            }));

            if let Err(error) = render {
                log::warn!("SVG rendering for {handle:?} panicked: {error:?}");
            }

            if let Some([r, g, b, _]) = key.color {
                // Apply color filter
                for pixel in image.data_mut() {
                    *pixel =
                        crate::into_color(Color::from_rgba8(r, g, b, f32::from(pixel.a) / 255.0))
                            .premultiply()
                            .to_rgba8();
                }
            } else {
                // Swap R and B channels for `softbuffer` presentation
                for vello_cpu::color::PremulRgba8 { r, b, .. } in image.data_mut() {
                    std::mem::swap(r, b);
                }
            }

            let _ = self.rasters.insert(key, Arc::new(image));
        }

        let _ = self.raster_hits.insert(key);
        self.rasters.get(&key).map(Arc::clone)
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
