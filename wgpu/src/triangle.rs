//! Draw meshes of triangles.
use crate::Transformation;
use iced_native::{Point, Rectangle};
use std::{mem, sync::Arc};

#[derive(Debug)]
pub(crate) struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
}

impl Pipeline {
    pub fn new(device: &mut wgpu::Device) -> Pipeline {
        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
            });

        let constants_buffer = device
            .create_buffer_mapped(
                1,
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            )
            .fill_from_slice(&[Uniforms::default()]);

        let constant_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &constant_layout,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &constants_buffer,
                        range: 0..std::mem::size_of::<Uniforms>() as u64,
                    },
                }],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&constant_layout],
            });

        let vs = include_bytes!("shader/triangle.vert.spv");
        let vs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&vs[..]))
                .expect("Read triangle vertex shader as SPIR-V"),
        );

        let fs = include_bytes!("shader/triangle.frag.spv");
        let fs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&fs[..]))
                .expect("Read triangle fragment shader as SPIR-V"),
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
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<Vertex2D>() as u64,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        // Position
                        wgpu::VertexAttributeDescriptor {
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float2,
                            offset: 0,
                        },
                        // Color
                        wgpu::VertexAttributeDescriptor {
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float4,
                            offset: 4 * 2,
                        },
                    ],
                }],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Pipeline {
            pipeline,
            constants: constant_bind_group,
            constants_buffer,
        }
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        transformation: Transformation,
        meshes: &Vec<(Point, Arc<Mesh2D>)>,
        bounds: Rectangle<u32>,
    ) {
        for (origin, mesh) in meshes {
            let uniforms = Uniforms {
                transform: (transformation
                    * Transformation::translate(origin.x, origin.y))
                .into(),
            };

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

            let vertices_buffer = device
                .create_buffer_mapped(
                    mesh.vertices.len(),
                    wgpu::BufferUsage::VERTEX,
                )
                .fill_from_slice(&mesh.vertices);

            let indices_buffer = device
                .create_buffer_mapped(
                    mesh.indices.len(),
                    wgpu::BufferUsage::INDEX,
                )
                .fill_from_slice(&mesh.indices);

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
            render_pass.set_index_buffer(&indices_buffer, 0);
            render_pass.set_vertex_buffers(0, &[(&vertices_buffer, 0)]);
            render_pass.set_scissor_rect(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );

            render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
        }
    }
}

/// A two-dimensional vertex with some color in __linear__ RGBA.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex2D {
    /// The vertex position
    pub position: [f32; 2],
    /// The vertex color in __linear__ RGBA.
    pub color: [f32; 4],
}

/// A set of [`Vertex2D`] and indices representing a list of triangles.
///
/// [`Vertex2D`]: struct.Vertex2D.html
#[derive(Clone, Debug)]
pub struct Mesh2D {
    /// The vertices of the mesh
    pub vertices: Vec<Vertex2D>,
    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of 3.
    pub indices: Vec<u16>,
}
