use crate::Transformation;
use iced_native::{
    image::{Data, Handle},
    Rectangle,
};

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
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

    pub fn dimensions(&self, handle: &Handle) -> (u32, u32) {
        self.load(handle);

        self.cache.borrow_mut().get(handle).unwrap().dimensions()
    }

    fn load(&self, handle: &Handle) {
        if !self.cache.borrow().contains(&handle) {
            let memory = match handle.data() {
                Data::Path(path) => {
                    if let Ok(image) = image::open(path) {
                        Memory::Host {
                            image: image.to_bgra(),
                        }
                    } else {
                        Memory::NotFound
                    }
                }
                Data::Bytes(bytes) => {
                    if let Ok(image) = image::load_from_memory(&bytes) {
                        Memory::Host {
                            image: image.to_bgra(),
                        }
                    } else {
                        Memory::Invalid
                    }
                }
            };

            let _ = self.cache.borrow_mut().insert(&handle, memory);
        }
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        instances: &[Image],
        transformation: Transformation,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
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
        for image in instances {
            self.load(&image.handle);

            if let Some(texture) = self
                .cache
                .borrow_mut()
                .get(&image.handle)
                .unwrap()
                .upload(device, encoder, &self.texture_layout)
            {
                let instance_buffer = device
                    .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
                    .fill_from_slice(&[Instance {
                        _position: image.position,
                        _scale: image.scale,
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
                                    clear_color: wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 0.0,
                                    },
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

#[derive(Debug)]
enum Memory {
    Host {
        image: image::ImageBuffer<image::Bgra<u8>, Vec<u8>>,
    },
    Device {
        bind_group: Rc<wgpu::BindGroup>,
        width: u32,
        height: u32,
    },
    NotFound,
    Invalid,
}

impl Memory {
    fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host { image } => image.dimensions(),
            Memory::Device { width, height, .. } => (*width, *height),
            Memory::NotFound => (1, 1),
            Memory::Invalid => (1, 1),
        }
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Option<Rc<wgpu::BindGroup>> {
        match self {
            Memory::Host { image } => {
                let (width, height) = image.dimensions();

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

                let slice = image.clone().into_raw();

                let temp_buf = device
                    .create_buffer_mapped(
                        slice.len(),
                        wgpu::BufferUsage::COPY_SRC,
                    )
                    .fill_from_slice(&slice[..]);

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

                *self = Memory::Device {
                    bind_group: bind_group.clone(),
                    width,
                    height,
                };

                Some(bind_group)
            }
            Memory::Device { bind_group, .. } => Some(bind_group.clone()),
            Memory::NotFound => None,
            Memory::Invalid => None,
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

pub struct Image {
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
