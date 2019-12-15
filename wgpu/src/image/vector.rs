use iced_native::svg;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct Svg {
    tree: resvg::usvg::Tree,
}

impl Svg {
    pub fn viewport_dimensions(&self) -> (u32, u32) {
        let size = self.tree.svg_node().size;

        (size.width() as u32, size.height() as u32)
    }
}

impl std::fmt::Debug for Svg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Svg")
    }
}

#[derive(Debug)]
pub struct Cache {
    svgs: HashMap<u64, Svg>,
    rasterized: HashMap<(u64, u32, u32), Rc<wgpu::BindGroup>>,
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

    pub fn load(&mut self, handle: &svg::Handle) -> Option<&Svg> {
        if self.svgs.contains_key(&handle.id()) {
            return self.svgs.get(&handle.id());
        }

        let opt = resvg::Options::default();

        match resvg::usvg::Tree::from_file(handle.path(), &opt.usvg) {
            Ok(tree) => {
                let _ = self.svgs.insert(handle.id(), Svg { tree });
            }
            Err(_) => {}
        };

        self.svgs.get(&handle.id())
    }

    pub fn upload(
        &mut self,
        handle: &svg::Handle,
        [width, height]: [f32; 2],
        scale: f32,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Option<Rc<wgpu::BindGroup>> {
        let id = handle.id();

        let (width, height) = (
            (scale * width).round() as u32,
            (scale * height).round() as u32,
        );

        // TODO: Optimize!
        // We currently rerasterize the SVG when its size changes. This is slow
        // as heck. A GPU rasterizer like `pathfinder` may perform better.
        // It would be cool to be able to smooth resize the `svg` example.
        if let Some(bind_group) = self.rasterized.get(&(id, width, height)) {
            let _ = self.svg_hits.insert(id);
            let _ = self.rasterized_hits.insert((id, width, height));

            return Some(bind_group.clone());
        }

        match self.load(handle) {
            Some(svg) => {
                let extent = wgpu::Extent3d {
                    width,
                    height,
                    depth: 1,
                };

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: extent,
                    array_layer_count: 1,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUsage::COPY_DST
                        | wgpu::TextureUsage::SAMPLED,
                });

                let temp_buf = {
                    let screen_size =
                        resvg::ScreenSize::new(width, height).unwrap();

                    let mut canvas = resvg::raqote::DrawTarget::new(
                        width as i32,
                        height as i32,
                    );

                    resvg::backend_raqote::render_to_canvas(
                        &svg.tree,
                        &resvg::Options::default(),
                        screen_size,
                        &mut canvas,
                    );

                    let slice = canvas.get_data();

                    device
                        .create_buffer_mapped(
                            slice.len(),
                            wgpu::BufferUsage::COPY_SRC,
                        )
                        .fill_from_slice(slice)
                };

                encoder.copy_buffer_to_texture(
                    wgpu::BufferCopyView {
                        buffer: &temp_buf,
                        offset: 0,
                        row_pitch: 4 * width as u32,
                        image_height: height as u32,
                    },
                    wgpu::TextureCopyView {
                        texture: &texture,
                        array_layer: 0,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    extent,
                );

                let bind_group =
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: texture_layout,
                        bindings: &[wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &texture.create_default_view(),
                            ),
                        }],
                    });

                let bind_group = Rc::new(bind_group);

                let _ = self
                    .rasterized
                    .insert((id, width, height), bind_group.clone());

                let _ = self.svg_hits.insert(id);
                let _ = self.rasterized_hits.insert((id, width, height));

                Some(bind_group)
            }
            None => None,
        }
    }

    pub fn trim(&mut self) {
        let svg_hits = &self.svg_hits;
        let rasterized_hits = &self.rasterized_hits;

        self.svgs.retain(|k, _| svg_hits.contains(k));
        self.rasterized.retain(|k, _| rasterized_hits.contains(k));
        self.svg_hits.clear();
        self.rasterized_hits.clear();
    }
}
