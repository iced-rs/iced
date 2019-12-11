use crate::Transformation;
use iced_native::{Hasher, Rectangle};

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::{Hash, Hasher as _},
    mem,
    path::PathBuf,
    rc::Rc,
    u32,
};

#[derive(Debug)]
pub struct Pipeline {
    cache: RefCell<Cache>,

    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
    constants: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                ],
            });

        let uniforms = Uniforms {
            transform: Transformation::identity().into(),
        };

        let uniforms_buffer = device
            .create_buffer_mapped(
                1,
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            )
            .fill_from_slice(&[uniforms]);

        let constant_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &constant_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &uniforms_buffer,
                            range: 0..std::mem::size_of::<Uniforms>() as u64,
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                }],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&constant_layout, &texture_layout],
            });

        let vs = include_bytes!("shader/image.vert.spv");
        let vs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&vs[..]))
                .expect("Read image vertex shader as SPIR-V"),
        );

        let fs = include_bytes!("shader/image.frag.spv");
        let fs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&fs[..]))
                .expect("Read image fragment shader as SPIR-V"),
        );

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: &layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[wgpu::VertexAttributeDescriptor {
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float2,
                            offset: 0,
                        }],
                    },
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float2,
                                offset: 0,
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float2,
                                offset: 4 * 2,
                            },
                        ],
                    },
                ],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        let vertices = device
            .create_buffer_mapped(QUAD_VERTS.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&QUAD_VERTS);

        let indices = device
            .create_buffer_mapped(QUAD_INDICES.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&QUAD_INDICES);

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            size: mem::size_of::<Instance>() as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        Pipeline {
            cache: RefCell::new(Cache::new()),

            pipeline,
            uniforms: uniforms_buffer,
            vertices,
            indices,
            instances,
            constants: constant_bind_group,
            texture_layout,
        }
    }

    fn load(&self, handle: &Handle) {
        if !self.cache.borrow().contains(&handle) {
            if !handle.path.is_file() {
                let mem = Memory::NotFound;

                let _ = self.cache.borrow_mut().insert(&handle, mem);
            }

            let mut opt = resvg::Options::default();
            opt.usvg.path = Some(handle.path.clone());
            opt.usvg.dpi = handle.dpi as f64;
            opt.usvg.font_size = handle.font_size as f64;

            let mem =
                match resvg::usvg::Tree::from_file(&handle.path, &opt.usvg) {
                    Ok(tree) => Memory::Host { tree },
                    Err(_) => Memory::Invalid,
                };

            let _ = self.cache.borrow_mut().insert(&handle, mem);
        }
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        svgs: &[Svg],
        transformation: Transformation,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        dpi: u16,
        font_size: u16,
    ) {
        let uniforms_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&[Uniforms {
                transform: transformation.into(),
            }]);

        encoder.copy_buffer_to_buffer(
            &uniforms_buffer,
            0,
            &self.uniforms,
            0,
            std::mem::size_of::<Uniforms>() as u64,
        );

        // TODO: Batch draw calls using a texture atlas
        // Guillotière[1] by @nical can help us a lot here.
        //
        // [1]: https://github.com/nical/guillotiere
        for svg in svgs {
            let mut handle = svg.handle.clone();
            handle.set_dpi(dpi);
            handle.set_font_size(font_size);

            self.load(&handle);

            if let Some(texture) =
                self.cache.borrow_mut().get(&handle).unwrap().upload(
                    device,
                    encoder,
                    &self.texture_layout,
                    svg.scale[0] as u32,
                    svg.scale[1] as u32,
                )
            {
                let instance_buffer = device
                    .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
                    .fill_from_slice(&[Instance {
                        _position: svg.position,
                        _scale: svg.scale,
                    }]);

                encoder.copy_buffer_to_buffer(
                    &instance_buffer,
                    0,
                    &self.instances,
                    0,
                    mem::size_of::<Instance>() as u64,
                );

                {
                    let mut render_pass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            color_attachments: &[
                                wgpu::RenderPassColorAttachmentDescriptor {
                                    attachment: target,
                                    resolve_target: None,
                                    load_op: wgpu::LoadOp::Load,
                                    store_op: wgpu::StoreOp::Store,
                                    clear_color: wgpu::Color::TRANSPARENT,
                                },
                            ],
                            depth_stencil_attachment: None,
                        },
                    );

                    render_pass.set_pipeline(&self.pipeline);
                    render_pass.set_bind_group(0, &self.constants, &[]);
                    render_pass.set_bind_group(1, &texture, &[]);
                    render_pass.set_index_buffer(&self.indices, 0);
                    render_pass.set_vertex_buffers(
                        0,
                        &[(&self.vertices, 0), (&self.instances, 0)],
                    );
                    render_pass.set_scissor_rect(
                        bounds.x,
                        bounds.y,
                        bounds.width,
                        bounds.height,
                    );

                    render_pass.draw_indexed(
                        0..QUAD_INDICES.len() as u32,
                        0,
                        0..1 as u32,
                    );
                }
            }
        }
    }

    pub fn trim_cache(&mut self) {
        self.cache.borrow_mut().trim();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Handle {
    id: u64,
    path: PathBuf,
    dpi: u16,
    font_size: u16,
}

impl Handle {
    /// Returns the unique id of this [`Handle`]
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Creates a svg [`Handle`] pointing to the svg icon of the given path.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        let path = path.into();
        let dpi = 96;
        let font_size = 20;
        let mut hasher = Hasher::default();
        path.hash(&mut hasher);
        dpi.hash(&mut hasher);
        font_size.hash(&mut hasher);

        Self {
            id: hasher.finish(),
            path,
            dpi,
            font_size,
        }
    }

    fn set_dpi(&mut self, dpi: u16) {
        if self.dpi == dpi {
            return;
        }

        self.dpi = dpi;

        let mut hasher = Hasher::default();
        self.path.hash(&mut hasher);
        self.dpi.hash(&mut hasher);
        self.font_size.hash(&mut hasher);

        self.id = hasher.finish();
    }

    fn set_font_size(&mut self, font_size: u16) {
        if self.font_size == font_size {
            return;
        }

        self.font_size = font_size;

        let mut hasher = Hasher::default();
        self.path.hash(&mut hasher);
        self.dpi.hash(&mut hasher);
        self.font_size.hash(&mut hasher);

        self.id = hasher.finish();
    }
}

impl Hash for Handle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.dpi.hash(state);
        self.font_size.hash(state);
    }
}

enum Memory {
    Host { tree: resvg::usvg::Tree },
    Device { bind_group: Rc<wgpu::BindGroup> },
    NotFound,
    Invalid,
}

impl Memory {
    fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Option<Rc<wgpu::BindGroup>> {
        match self {
            Memory::Host { tree } => {
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

                let mut canvas =
                    resvg::raqote::DrawTarget::new(width as i32, height as i32);
                let opt = resvg::Options::default();
                let screen_size =
                    resvg::ScreenSize::new(width, height).unwrap();
                resvg::backend_raqote::render_to_canvas(
                    tree,
                    &opt,
                    screen_size,
                    &mut canvas,
                );
                let slice = canvas.get_data();
                let temp_buf = device
                    .create_buffer_mapped(
                        slice.len(),
                        wgpu::BufferUsage::COPY_SRC,
                    )
                    .fill_from_slice(slice);

                encoder.copy_buffer_to_texture(
                    wgpu::BufferCopyView {
                        buffer: &temp_buf,
                        offset: 0,
                        row_pitch: width * 4,
                        image_height: height,
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

                *self = Memory::Device {
                    bind_group: bind_group.clone(),
                };

                Some(bind_group)
            }
            Memory::Device { bind_group, .. } => Some(bind_group.clone()),
            Memory::NotFound => None,
            Memory::Invalid => None,
        }
    }
}

impl Debug for Memory {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        match self {
            Memory::Host { .. } => write!(f, "Memory::Host"),
            Memory::Device { .. } => write!(f, "Memory::Device"),
            Memory::NotFound => write!(f, "Memory::NotFound"),
            Memory::Invalid => write!(f, "Memory::Invalid"),
        }
    }
}

#[derive(Debug)]
struct Cache {
    map: HashMap<u64, Memory>,
    hits: HashSet<u64>,
}

impl Cache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            hits: HashSet::new(),
        }
    }

    fn contains(&self, handle: &Handle) -> bool {
        self.map.contains_key(&handle.id())
    }

    fn get(&mut self, handle: &Handle) -> Option<&mut Memory> {
        let _ = self.hits.insert(handle.id());

        self.map.get_mut(&handle.id())
    }

    fn insert(&mut self, handle: &Handle, memory: Memory) {
        let _ = self.map.insert(handle.id(), memory);
    }

    fn trim(&mut self) {
        let hits = &self.hits;

        self.map.retain(|k, _| hits.contains(k));
        self.hits.clear();
    }
}

#[derive(Debug)]
pub struct Svg {
    pub handle: Handle,
    pub position: [f32; 2],
    pub scale: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    _position: [f32; 2],
}

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        _position: [0.0, 0.0],
    },
    Vertex {
        _position: [1.0, 0.0],
    },
    Vertex {
        _position: [1.0, 1.0],
    },
    Vertex {
        _position: [0.0, 1.0],
    },
];

#[repr(C)]
#[derive(Clone, Copy)]
struct Instance {
    _position: [f32; 2],
    _scale: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
}
