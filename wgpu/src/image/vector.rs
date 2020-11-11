use crate::image::atlas::{self, Atlas};
use iced_native::svg;
use std::collections::{HashMap, HashSet};

pub enum Svg {
    Loaded(resvg::usvg::Tree),
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

        let opt = resvg::Options::default();

        let svg = match handle.data() {
            svg::Data::Path(path) => {
                match resvg::usvg::Tree::from_file(path, &opt.usvg) {
                    Ok(tree) => Svg::Loaded(tree),
                    Err(_) => Svg::NotFound,
                }
            }
            svg::Data::Bytes(bytes) => {
                match resvg::usvg::Tree::from_data(&bytes, &opt.usvg) {
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
            (scale * width).round() as u32,
            (scale * height).round() as u32,
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
                let screen_size =
                    resvg::ScreenSize::new(width, height).unwrap();

                let mut canvas =
                    resvg::raqote::DrawTarget::new(width as i32, height as i32);

                resvg::backend_raqote::render_to_canvas(
                    tree,
                    &resvg::Options::default(),
                    screen_size,
                    &mut canvas,
                );

                let allocation = texture_atlas.upload(
                    width,
                    height,
                    bytemuck::cast_slice(canvas.get_data()),
                    device,
                    encoder,
                )?;

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
