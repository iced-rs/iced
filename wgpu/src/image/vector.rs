use crate::core::svg;
use crate::core::{Color, Size};
use crate::image::atlas::{self, Atlas};

use resvg::tiny_skia;
use resvg::usvg;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fs;
use std::panic;
#[cfg(feature = "svg-text")]
use std::sync::Arc;

/// Entry in cache corresponding to an svg handle
pub enum Svg {
    /// Parsed svg
    Loaded(usvg::Tree),
    /// Svg not found or failed to parse
    NotFound,
}

impl Svg {
    /// Viewport width and height
    pub fn viewport_dimensions(&self) -> Size<u32> {
        match self {
            Svg::Loaded(tree) => {
                let size = tree.size();

                Size::new(size.width() as u32, size.height() as u32)
            }
            Svg::NotFound => Size::new(1, 1),
        }
    }
}

/// Caches svg vector and raster data
#[derive(Debug, Default)]
pub struct Cache {
    svgs: FxHashMap<u64, Svg>,
    rasterized: FxHashMap<(u64, u32, u32, ColorFilter), atlas::Entry>,
    svg_hits: FxHashSet<u64>,
    rasterized_hits: FxHashSet<(u64, u32, u32, ColorFilter)>,
    should_trim: bool,
    #[cfg(feature = "svg-text")]
    fontdb: Option<Arc<usvg::fontdb::Database>>,
}

type ColorFilter = Option<[u8; 4]>;

impl Cache {
    /// Load svg
    pub fn load(&mut self, handle: &svg::Handle) -> &Svg {
        if self.svgs.contains_key(&handle.id()) {
            return self.svgs.get(&handle.id()).unwrap();
        }

        // TODO: Reuse `cosmic-text` font database
        #[cfg(feature = "svg-text")]
        if self.fontdb.is_none() {
            let mut fontdb = usvg::fontdb::Database::new();
            fontdb.load_system_fonts();

            self.fontdb = Some(Arc::new(fontdb));
        }

        let options = usvg::Options {
            #[cfg(feature = "svg-text")]
            fontdb: self
                .fontdb
                .as_ref()
                .expect("fontdb must be initialized")
                .clone(),
            ..usvg::Options::default()
        };

        let svg = match handle.data() {
            svg::Data::Path(path) => fs::read_to_string(path)
                .ok()
                .and_then(|contents| usvg::Tree::from_str(&contents, &options).ok())
                .map(Svg::Loaded)
                .unwrap_or(Svg::NotFound),
            svg::Data::Bytes(bytes) => match usvg::Tree::from_data(bytes, &options) {
                Ok(tree) => Svg::Loaded(tree),
                Err(_) => Svg::NotFound,
            },
        };

        self.should_trim = true;

        let _ = self.svgs.insert(handle.id(), svg);
        self.svgs.get(&handle.id()).unwrap()
    }

    /// Load svg and upload raster data
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        handle: &svg::Handle,
        color: Option<Color>,
        size: Size<u32>,
        atlas: &mut Atlas,
    ) -> Option<&atlas::Entry> {
        let id = handle.id();

        let color = color.map(Color::into_rgba8);
        let key = (id, size.width, size.height, color);

        // TODO: Optimize!
        // We currently rerasterize the SVG when its size changes. This is slow
        // as heck. A GPU rasterizer like `pathfinder` may perform better.
        // It would be cool to be able to smooth resize the `svg` example.
        if self.rasterized.contains_key(&key) {
            let _ = self.svg_hits.insert(id);
            let _ = self.rasterized_hits.insert(key);

            return self.rasterized.get(&key);
        }

        match self.load(handle) {
            Svg::Loaded(tree) => {
                // TODO: Optimize!
                // We currently rerasterize the SVG when its size changes. This is slow
                // as heck. A GPU rasterizer like `pathfinder` may perform better.
                // It would be cool to be able to smooth resize the `svg` example.
                let mut img = tiny_skia::Pixmap::new(size.width, size.height)?;

                let tree_size = tree.size().to_int_size();

                let target_size = if size.width > size.height {
                    tree_size.scale_to_height(size.height)
                } else {
                    tree_size.scale_to_width(size.width)
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
                    resvg::render(tree, transform, &mut img.as_mut());
                }));

                if let Err(error) = render {
                    log::warn!("SVG rendering for {handle:?} panicked: {error:?}");
                }

                let mut rgba = img.take();

                if let Some(color) = color {
                    rgba.chunks_exact_mut(4).for_each(|rgba| {
                        if rgba[3] > 0 {
                            rgba[0] = color[0];
                            rgba[1] = color[1];
                            rgba[2] = color[2];
                        }
                    });
                }

                let allocation =
                    atlas.upload(device, encoder, belt, size.width, size.height, &rgba)?;

                log::debug!("allocating {id} {}x{}", size.width, size.height);

                let _ = self.svg_hits.insert(id);
                let _ = self.rasterized_hits.insert(key);
                let _ = self.rasterized.insert(key, allocation);
                self.should_trim = true;

                self.rasterized.get(&key)
            }
            Svg::NotFound => None,
        }
    }

    /// Load svg and upload raster data
    pub fn trim(&mut self, atlas: &mut Atlas) {
        if !self.should_trim {
            return;
        }

        let svg_hits = &self.svg_hits;
        let rasterized_hits = &self.rasterized_hits;

        self.svgs.retain(|k, _| svg_hits.contains(k));
        self.rasterized.retain(|k, entry| {
            let retain = rasterized_hits.contains(k);

            if !retain {
                atlas.remove(entry);
            }

            retain
        });
        self.svg_hits.clear();
        self.rasterized_hits.clear();
        self.should_trim = false;
    }
}

impl std::fmt::Debug for Svg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Svg::Loaded(_) => write!(f, "Svg::Loaded"),
            Svg::NotFound => write!(f, "Svg::NotFound"),
        }
    }
}
