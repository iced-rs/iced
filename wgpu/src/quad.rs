use crate::Transformation;
use iced_native::Rectangle;

use std::{mem, cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Pipeline {
    cache: RefCell<ColorCache>,

    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
    texture_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(device: &mut wgpu::Device) -> Pipeline {
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

        let constants_buffer = device
            .create_buffer_mapped(
                1,
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            )
            .fill_from_slice(&[Uniforms::default()]);

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &constant_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &constants_buffer,
                        range: 0..std::mem::size_of::<Uniforms>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&constant_layout, &texture_layout],
            });

        let vs = include_bytes!("shader/quad.vert.spv");
        let vs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&vs[..]))
                .expect("Read quad vertex shader as SPIR-V"),
        );

        let fs = include_bytes!("shader/quad.frag.spv");
        let fs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&fs[..]))
                .expect("Read quad fragment shader as SPIR-V"),
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
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float2,
                                offset: 4 * (2 + 2),
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float,
                                offset: 4 * (2 + 2 + 2),
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
            size: mem::size_of::<Instance>() as u64 * Quad::MAX as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        Pipeline {
            cache: RefCell::new(ColorCache::default()),

            pipeline,
            constants,
            constants_buffer,
            vertices,
            indices,
            instances,
            texture_layout,
        }
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        instances: &[Quad],
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
    ) {
        // load all colors into a single texture
        let instances = {
            let mut cache = self.cache.borrow_mut();
            instances.iter()
                .map(|quad| {
                    let colors = quad.colors.iter()
                        .map(|c| to_bgra(*c))
                        .collect();
                    let offset = cache.load_gradient(colors);
                    Instance {
                        _position: quad.position,
                        _scale: quad.scale,
                        _color_offset: offset as f32,
                        _colors: quad.colors.len() as f32,
                        _border_radius: quad.border_radius,
                    }
                })
                .collect::<Vec<_>>()
        };

        let uniforms = Uniforms::new(transformation, scale);

        let constants_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&[uniforms]);

        encoder.copy_buffer_to_buffer(
            &constants_buffer,
            0,
            &self.constants_buffer,
            0,
            std::mem::size_of::<Uniforms>() as u64,
        );

        let mut i = 0;
        let total = instances.len();

        while i < total {
            let color_texture = self.cache.borrow_mut()
                .upload(device, encoder, &self.texture_layout);

            let end = (i + Quad::MAX).min(total);
            let amount = end - i;

            let instance_buffer = device
                .create_buffer_mapped(amount, wgpu::BufferUsage::COPY_SRC)
                .fill_from_slice(&instances[i..end]);

            encoder.copy_buffer_to_buffer(
                &instance_buffer,
                0,
                &self.instances,
                0,
                (mem::size_of::<Instance>() * amount) as u64,
            );

            {
                let mut render_pass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    });

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.constants, &[]);
                render_pass.set_bind_group(1, &color_texture, &[]);
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
                    0..amount as u32,
                );
            }

            i += Quad::MAX;
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct Quad {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub colors: Vec<[f32; 4]>,
    pub border_radius: f32,
}

impl Quad {
    const MAX: usize = 100_000;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
}

impl Uniforms {
    fn new(transformation: Transformation, scale: f32) -> Uniforms {
        Self {
            transform: *transformation.as_ref(),
            scale,
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
            scale: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Instance {
    _position: [f32; 2],
    _scale: [f32; 2],
    _color_offset: f32,
    _colors: f32,
    _border_radius: f32,
}

fn to_bgra(color: [f32; 4]) -> [u8; 4] {
    [
        (color[2] * 255.0) as u8, 
        (color[1] * 255.0) as u8, 
        (color[0] * 255.0) as u8, 
        (color[3] * 255.0) as u8
    ]
}

#[derive(Debug, Default)]
struct ColorCache {
    texture: Vec<u8>,
    stored_colors: HashMap<Vec<[u8; 4]>, usize>,
    bind_group: Option<Rc<wgpu::BindGroup>>,
}

impl ColorCache {
    fn load_gradient(&mut self, colors: Vec<[u8; 4]>) -> usize {
        if let Some(offset) = self.stored_colors.get(&colors) {
            return *offset;
        }

        self.bind_group = None;
        let position = self.texture.len() / 4;

        let values = colors.iter()
            .flat_map(|c| c.iter())
            .map(|v| *v);
        self.texture.extend(values);
        let _ = self.stored_colors.insert(colors, position);
        position
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Rc<wgpu::BindGroup> {
        if let Some(ref bind_group) = self.bind_group {
            return bind_group.clone();
        }

        let texture_extent = wgpu::Extent3d {
            width: 1,
            height: (self.texture.len() / 4) as u32,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::COPY_DST,
        });
        let temp_buf = device.create_buffer_mapped(
            self.texture.len(),
            wgpu::BufferUsage::COPY_SRC,
        ).fill_from_slice(&self.texture);


        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4,
                image_height: self.texture.len() as u32,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            texture_extent,
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
        self.bind_group = Some(bind_group.clone());
        bind_group
    }
}
