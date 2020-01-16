use crate::image::{TextureArray, ImageAllocation};
use iced_native::svg;
use std::{
    collections::{HashMap, HashSet},
};
use guillotiere::Size;
use debug_stub_derive::*;

#[derive(DebugStub)]
pub enum Svg {
    Loaded(
        #[debug_stub="ReplacementValue"]
        resvg::usvg::Tree
    ),
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

#[derive(DebugStub)]
pub struct Cache {
    svgs: HashMap<u64, Svg>,
    #[debug_stub="ReplacementValue"]
    rasterized: HashMap<(u64, u32, u32), ImageAllocation>,
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

        let svg = match resvg::usvg::Tree::from_file(handle.path(), &opt.usvg) {
            Ok(tree) => Svg::Loaded(tree),
            Err(_) => Svg::NotFound,
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
        texture_array: &mut TextureArray,
    ) -> Option<&ImageAllocation> {
        let id = handle.id();

        let (width, height) = (
            (scale * width).round() as u32,
            (scale * height).round() as u32,
        );

        // TODO: Optimize!
        // We currently rerasterize the SVG when its size changes. This is slow
        // as heck. A GPU rasterizer like `pathfinder` may perform better.
        // It would be cool to be able to smooth resize the `svg` example.
        if self.rasterized.get(&(id, width, height)).is_some() {
            let _ = self.svg_hits.insert(id);
            let _ = self.rasterized_hits.insert((id, width, height));

            return self.rasterized.get(&(id, width, height));
        }

        let _ = self.load(handle);

        match self.svgs.get(&handle.id()).unwrap() {
            Svg::Loaded(tree) => {
                if width == 0 || height == 0 {
                    return None;
                }

                let size = Size::new(width as i32, height as i32);

                let array_allocation = texture_array.allocate(size);

                // TODO: Optimize!
                // We currently rerasterize the SVG when its size changes. This is slow
                // as heck. A GPU rasterizer like `pathfinder` may perform better.
                // It would be cool to be able to smooth resize the `svg` example.
                let screen_size =
                    resvg::ScreenSize::new(width, height).unwrap();

                let mut canvas = resvg::raqote::DrawTarget::new(
                    width as i32,
                    height as i32,
                );

                resvg::backend_raqote::render_to_canvas(
                    tree,
                    &resvg::Options::default(),
                    screen_size,
                    &mut canvas,
                );

                texture_array.upload(&canvas, &array_allocation, device, encoder);

                let _ = self.svg_hits.insert(id);
                let _ = self.rasterized_hits.insert((id, width, height));
                let _ = self
                    .rasterized
                    .insert((id, width, height), array_allocation);

                self.rasterized.get(&(id, width, height))
            }
            Svg::NotFound => None
        }
    }

    pub fn trim(&mut self, texture_array: &mut TextureArray) {
        let svg_hits = &self.svg_hits;
        let rasterized_hits = &self.rasterized_hits;

        for (k, allocation) in &self.rasterized {
            if !rasterized_hits.contains(k) {
                texture_array.deallocate(allocation);
            }
        }

        self.svgs.retain(|k, _| svg_hits.contains(k));
        self.rasterized.retain(|k, _| rasterized_hits.contains(k));
        self.svg_hits.clear();
        self.rasterized_hits.clear();
    }
}
