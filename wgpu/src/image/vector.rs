use crate::core::svg;
use crate::core::{Color, Size};
use crate::image::atlas::{self, Atlas};

use resvg::tiny_skia;
use resvg::usvg;
use std::collections::{HashMap, HashSet};
use std::fs;

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
                let size = tree.size;

                Size::new(size.width() as u32, size.height() as u32)
            }
            Svg::NotFound => Size::new(1, 1),
        }
    }
}

/// Caches svg vector and raster data
#[derive(Debug, Default)]
pub struct Cache {
    svgs: HashMap<u64, Svg>,
    rasterized: HashMap<(u64, u32, u32, ColorFilter), atlas::Entry>,
    svg_hits: HashSet<u64>,
    rasterized_hits: HashSet<(u64, u32, u32, ColorFilter)>,
}

type ColorFilter = Option<[u8; 4]>;

impl Cache {
    /// Load svg
    pub fn load(&mut self, handle: &svg::Handle) -> &Svg {
        use usvg::TreeParsing;

        if self.svgs.contains_key(&handle.id()) {
            return self.svgs.get(&handle.id()).unwrap();
        }

        let svg = match handle.data() {
            svg::Data::Path(path) => {
                let tree = fs::read_to_string(path).ok().and_then(|contents| {
                    usvg::Tree::from_str(&contents, &usvg::Options::default())
                        .ok()
                });

                tree.map(Svg::Loaded).unwrap_or(Svg::NotFound)
            }
            svg::Data::Bytes(bytes) => {
                match usvg::Tree::from_data(bytes, &usvg::Options::default()) {
                    Ok(tree) => Svg::Loaded(tree),
                    Err(_) => Svg::NotFound,
                }
            }
        };

        let _ = self.svgs.insert(handle.id(), svg);
        self.svgs.get(&handle.id()).unwrap()
    }

    /// Load svg and upload raster data
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        handle: &svg::Handle,
        color: Option<Color>,
        [width, height]: [f32; 2],
        scale: f32,
        atlas: &mut Atlas,
    ) -> Option<&atlas::Entry> {
        let id = handle.id();

        let (width, height) = (
            (scale * width).ceil() as u32,
            (scale * height).ceil() as u32,
        );

        let color = color.map(Color::into_rgba8);
        let key = (id, width, height, color);

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
                if width == 0 || height == 0 {
                    return None;
                }

                // TODO: Optimize!
                // We currently rerasterize the SVG when its size changes. This is slow
                // as heck. A GPU rasterizer like `pathfinder` may perform better.
                // It would be cool to be able to smooth resize the `svg` example.
                let mut img = tiny_skia::Pixmap::new(width, height)?;

                let tree_size = tree.size.to_int_size();

                let target_size = if width > height {
                    tree_size.scale_to_width(width)
                } else {
                    tree_size.scale_to_height(height)
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

                resvg::Tree::from_usvg(tree)
                    .render(transform, &mut img.as_mut());

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
                    atlas.upload(device, encoder, width, height, &rgba)?;

                log::debug!("allocating {} {}x{}", id, width, height);

                let _ = self.svg_hits.insert(id);
                let _ = self.rasterized_hits.insert(key);
                let _ = self.rasterized.insert(key, allocation);

                self.rasterized.get(&key)
            }
            Svg::NotFound => None,
        }
    }

    /// Load svg and upload raster data
    pub fn trim(&mut self, atlas: &mut Atlas) {
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
