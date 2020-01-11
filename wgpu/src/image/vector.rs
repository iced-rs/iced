use iced_native::svg;
use std::{
    collections::{HashMap, HashSet},
};
use guillotiere::{Allocation, AtlasAllocator, Size};
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
    rasterized: HashMap<(u64, u32, u32), Allocation>,
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
        allocator: &mut AtlasAllocator,
        atlas: &mut wgpu::Texture,
    ) -> Option<&Allocation> {
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
                let old_atlas_size = allocator.size();
                let allocation;

                loop {
                    if let Some(a) = allocator.allocate(size) {
                        allocation = a;
                        break;
                    }

                    allocator.grow(allocator.size() * 2);
                }

                let new_atlas_size = allocator.size();

                if new_atlas_size != old_atlas_size {
                    let new_atlas = device.create_texture(&wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: new_atlas_size.width as u32,
                            height: new_atlas_size.height as u32,
                            depth: 1,
                        },
                        array_layer_count: 1,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        usage: wgpu::TextureUsage::COPY_DST
                            | wgpu::TextureUsage::COPY_SRC
                            | wgpu::TextureUsage::SAMPLED,
                    });

                    encoder.copy_texture_to_texture(
                        wgpu::TextureCopyView {
                            texture: atlas,
                            array_layer: 0,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        },
                        wgpu::TextureCopyView {
                            texture: &new_atlas,
                            array_layer: 0,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        },
                        wgpu::Extent3d {
                            width: old_atlas_size.width as u32,
                            height: old_atlas_size.height as u32,
                            depth: 1,
                        }
                    );

                    *atlas = new_atlas;
                }

                // TODO: Optimize!
                // We currently rerasterize the SVG when its size changes. This is slow
                // as heck. A GPU rasterizer like `pathfinder` may perform better.
                // It would be cool to be able to smooth resize the `svg` example.
                let temp_buf = {
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
                        texture: atlas,
                        array_layer: 0,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: allocation.rectangle.min.x as f32,
                            y: allocation.rectangle.min.y as f32,
                            z: 0.0,
                        },
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth: 1,
                    },
                );

                let _ = self.svg_hits.insert(id);
                let _ = self.rasterized_hits.insert((id, width, height));
                let _ = self
                    .rasterized
                    .insert((id, width, height), allocation);

                self.rasterized.get(&(id, width, height))
            }
            Svg::NotFound => None
        }
    }

    pub fn trim(&mut self, allocator: &mut AtlasAllocator) {
        let svg_hits = &self.svg_hits;
        let rasterized_hits = &self.rasterized_hits;

        for (k, alloc) in &mut self.rasterized {
            if !rasterized_hits.contains(&k) {
                allocator.deallocate(alloc.id);
            }
        }

        self.svgs.retain(|k, _| svg_hits.contains(k));
        self.rasterized.retain(|k, _| rasterized_hits.contains(k));
        self.svg_hits.clear();
        self.rasterized_hits.clear();
    }
}
