//! Vector image loading and caching
use crate::image::Storage;

use iced_native::svg;
use iced_native::Size;

use std::collections::{HashMap, HashSet};
use std::fs;

type Fill = Option<[u8; 4]>;

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
                let size = tree.svg_node().size;

                Size::new(size.width() as u32, size.height() as u32)
            }
            Svg::NotFound => Size::new(1, 1),
        }
    }
}

/// Caches svg vector and raster data
#[derive(Debug)]
pub struct Cache<T: Storage> {
    svgs: HashMap<u64, Svg>,
    rasterized: HashMap<(u64, u32, u32, Fill), T::Entry>,
    svg_hits: HashSet<u64>,
    rasterized_hits: HashSet<(u64, u32, u32, Fill)>,
}

impl<T: Storage> Cache<T> {
    /// Load svg
    pub fn load(&mut self, handle: &svg::Handle) -> &Svg {
        if self.svgs.contains_key(&handle.id()) {
            return self.svgs.get(&handle.id()).unwrap();
        }

        let svg = match handle.data() {
            svg::Data::Path(path) => {
                let tree = fs::read_to_string(path).ok().and_then(|contents| {
                    usvg::Tree::from_str(
                        &contents,
                        &usvg::Options::default().to_ref(),
                    )
                    .ok()
                });

                tree.map(Svg::Loaded).unwrap_or(Svg::NotFound)
            }
            svg::Data::Bytes(bytes) => {
                match usvg::Tree::from_data(
                    bytes,
                    &usvg::Options::default().to_ref(),
                ) {
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
        handle: &svg::Handle,
        [width, height]: [f32; 2],
        scale: f32,
        state: &mut T::State<'_>,
        storage: &mut T,
    ) -> Option<&T::Entry> {
        let id = handle.id();

        let (width, height) = (
            (scale * width).ceil() as u32,
            (scale * height).ceil() as u32,
        );

        let appearance = handle.appearance();
        let fill = appearance.fill.map(crate::Color::into_rgba8);

        // TODO: Optimize!
        // We currently rerasterize the SVG when its size changes. This is slow
        // as heck. A GPU rasterizer like `pathfinder` may perform better.
        // It would be cool to be able to smooth resize the `svg` example.
        if self.rasterized.contains_key(&(id, width, height, fill)) {
            let _ = self.svg_hits.insert(id);
            let _ = self.rasterized_hits.insert((id, width, height, fill));

            return self.rasterized.get(&(id, width, height, fill));
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

                resvg::render(
                    tree,
                    if width > height {
                        usvg::FitTo::Width(width)
                    } else {
                        usvg::FitTo::Height(height)
                    },
                    img.as_mut(),
                )?;

                let mut rgba = img.take();

                if let Some(color) = fill {
                    rgba.chunks_exact_mut(4).for_each(|rgba| {
                        if rgba[3] > 0 {
                            rgba[0] = color[0];
                            rgba[1] = color[1];
                            rgba[2] = color[2];
                        }
                    });
                }

                let allocation = storage.upload(width, height, &rgba, state)?;
                log::debug!("allocating {} {}x{}", id, width, height);

                let _ = self.svg_hits.insert(id);
                let _ = self.rasterized_hits.insert((id, width, height, fill));
                let _ = self
                    .rasterized
                    .insert((id, width, height, fill), allocation);

                self.rasterized.get(&(id, width, height, fill))
            }
            Svg::NotFound => None,
        }
    }

    /// Load svg and upload raster data
    pub fn trim(&mut self, storage: &mut T, state: &mut T::State<'_>) {
        let svg_hits = &self.svg_hits;
        let rasterized_hits = &self.rasterized_hits;

        self.svgs.retain(|k, _| svg_hits.contains(k));
        self.rasterized.retain(|k, entry| {
            let retain = rasterized_hits.contains(k);

            if !retain {
                storage.remove(entry, state);
            }

            retain
        });
        self.svg_hits.clear();
        self.rasterized_hits.clear();
    }
}

impl<T: Storage> Default for Cache<T> {
    fn default() -> Self {
        Self {
            svgs: HashMap::new(),
            rasterized: HashMap::new(),
            svg_hits: HashSet::new(),
            rasterized_hits: HashSet::new(),
        }
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
