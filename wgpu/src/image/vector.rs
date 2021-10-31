use crate::image::atlas::{self, Atlas};

use iced_native::svg;

use std::collections::{HashMap, HashSet};
use std::fs;

pub enum Svg {
    Loaded(usvg::Tree),
    NotFound,
}

impl Svg {
    pub fn viewport_dimensions(&self) -> (u32, u32) {
        match self {
            Svg::Loaded(tree) => {
                let size = tree.svg_node().size;

                (size.width() as u32, size.height() as u32)
            }
            Svg::NotFound => (1, 1),
        }
    }
}

#[derive(Debug)]
pub struct Cache {
    svgs: HashMap<u64, Svg>,
    rasterized: HashMap<(u64, u32, u32), atlas::Entry>,
    svg_hits: HashSet<u64>,
    rasterized_hits: HashSet<(u64, u32, u32)>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            svgs: HashMap::new(),
            rasterized: HashMap::new(),
            svg_hits: HashSet::new(),
            rasterized_hits: HashSet::new(),
        }
    }

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
                    &bytes,
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

    pub fn upload(
        &mut self,
        handle: &svg::Handle,
        [width, height]: [f32; 2],
        scale: f32,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_atlas: &mut Atlas,
    ) -> Option<&atlas::Entry> {
        let id = handle.id();

        let (width, height) = (
            (scale * width).ceil() as u32,
            (scale * height).ceil() as u32,
        );

        // TODO: Optimize!
        // We currently rerasterize the SVG when its size changes. This is slow
        // as heck. A GPU rasterizer like `pathfinder` may perform better.
        // It would be cool to be able to smooth resize the `svg` example.
        if self.rasterized.contains_key(&(id, width, height)) {
            let _ = self.svg_hits.insert(id);
            let _ = self.rasterized_hits.insert((id, width, height));

            return self.rasterized.get(&(id, width, height));
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

                let _ = resvg::render(
                    tree,
                    if width > height {
                        usvg::FitTo::Width(width)
                    } else {
                        usvg::FitTo::Height(height)
                    },
                    img.as_mut(),
                )?;

                let mut rgba = img.take();
                rgba.chunks_exact_mut(4).for_each(|rgba| rgba.swap(0, 2));

                let allocation = texture_atlas.upload(
                    width,
                    height,
                    bytemuck::cast_slice(rgba.as_slice()),
                    device,
                    encoder,
                )?;
                log::debug!("allocating {} {}x{}", id, width, height);

                let _ = self.svg_hits.insert(id);
                let _ = self.rasterized_hits.insert((id, width, height));
                let _ = self.rasterized.insert((id, width, height), allocation);

                self.rasterized.get(&(id, width, height))
            }
            Svg::NotFound => None,
        }
    }

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
