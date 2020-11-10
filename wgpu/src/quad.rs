use crate::Transformation;
use iced_graphics::layer;
use iced_native::Rectangle;

use bytemuck::{Pod, Zeroable};
use std::mem;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipeline {
        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::quad uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<Uniforms>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let constants_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::quad uniforms buffer"),
            size: mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::quad uniforms bind group"),
            layout: &constant_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    constants_buffer.slice(..),
                ),
            }],
        });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::quad pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&constant_layout],
            });

        let vs_module = device
            .create_shader_module(wgpu::include_spirv!("shader/quad.vert.spv"));

        let fs_module = device
            .create_shader_module(wgpu::include_spirv!("shader/quad.frag.spv"));

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::quad pipeline"),
                layout: Some(&layout),
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
                    ..Default::default()
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format,
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
                vertex_state: wgpu::VertexStateDescriptor {
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
                            stride: mem::size_of::<layer::Quad>() as u64,
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
                                    format: wgpu::VertexFormat::Float4,
                                    offset: 4 * (2 + 2),
                                },
                                wgpu::VertexAttributeDescriptor {
                                    shader_location: 4,
                                    format: wgpu::VertexFormat::Float4,
                                    offset: 4 * (2 + 2 + 4),
                                },
                                wgpu::VertexAttributeDescriptor {
                                    shader_location: 5,
                                    format: wgpu::VertexFormat::Float,
                                    offset: 4 * (2 + 2 + 4 + 4),
                                },
                                wgpu::VertexAttributeDescriptor {
                                    shader_location: 6,
                                    format: wgpu::VertexFormat::Float,
                                    offset: 4 * (2 + 2 + 4 + 4 + 1),
                                },
                            ],
                        },
                    ],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad vertex buffer"),
                contents: bytemuck::cast_slice(&QUAD_VERTS),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad index buffer"),
                contents: bytemuck::cast_slice(&QUAD_INDICES),
                usage: wgpu::BufferUsage::INDEX,
            });

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::quad instance buffer"),
            size: mem::size_of::<layer::Quad>() as u64 * MAX_INSTANCES as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Pipeline {
            pipeline,
            constants,
            constants_buffer,
            vertices,
            indices,
            instances,
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        instances: &[layer::Quad],
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
    ) {
        let uniforms = Uniforms::new(transformation, scale);

        {
            let mut constants_buffer = staging_belt.write_buffer(
                encoder,
                &self.constants_buffer,
                0,
                wgpu::BufferSize::new(mem::size_of::<Uniforms>() as u64)
                    .unwrap(),
                device,
            );

            constants_buffer.copy_from_slice(bytemuck::bytes_of(&uniforms));
        }

        let mut i = 0;
        let total = instances.len();

        while i < total {
            let end = (i + MAX_INSTANCES).min(total);
            let amount = end - i;

            let instance_bytes = bytemuck::cast_slice(&instances[i..end]);

            let mut instance_buffer = staging_belt.write_buffer(
                encoder,
                &self.instances,
                0,
                wgpu::BufferSize::new(instance_bytes.len() as u64).unwrap(),
                device,
            );

            instance_buffer.copy_from_slice(instance_bytes);

            {
                let mut render_pass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[
                            wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: target,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: true,
                                },
                            },
                        ],
                        depth_stencil_attachment: None,
                    });

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.constants, &[]);
                render_pass.set_index_buffer(self.indices.slice(..));
                render_pass.set_vertex_buffer(0, self.vertices.slice(..));
                render_pass.set_vertex_buffer(1, self.instances.slice(..));
                render_pass.set_scissor_rect(
                    bounds.x,
                    bounds.y,
                    bounds.width,
                    // TODO: Address anti-aliasing adjustments properly
                    bounds.height + 1,
                );

                render_pass.draw_indexed(
                    0..QUAD_INDICES.len() as u32,
                    0,
                    0..amount as u32,
                );
            }

            i += MAX_INSTANCES;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
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

const MAX_INSTANCES: usize = 100_000;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
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
