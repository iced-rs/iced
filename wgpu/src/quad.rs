use crate::core::Rectangle;
use crate::graphics::Transformation;
use crate::layer;
use crate::Buffer;

use wgpu::util::DeviceExt;

#[cfg(feature = "tracing")]
use tracing::info_span;

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constant_layout: wgpu::BindGroupLayout,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    layers: Vec<Layer>,
    prepare_layer: usize,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipeline {
        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::quad uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<Uniforms>() as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::quad pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&constant_layout],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu::quad::shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader/quad.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::quad pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: mem::size_of::<Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: mem::size_of::<layer::Quad>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array!(
                                1 => Float32x2,
                                2 => Float32x2,
                                3 => Float32x4,
                                4 => Float32x4,
                                5 => Float32x4,
                                6 => Float32,
                            ),
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad vertex buffer"),
                contents: bytemuck::cast_slice(&QUAD_VERTS),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad index buffer"),
                contents: bytemuck::cast_slice(&QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        Pipeline {
            pipeline,
            constant_layout,
            vertices,
            indices,
            layers: Vec::new(),
            prepare_layer: 0,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[layer::Quad],
        transformation: Transformation,
        scale: f32,
    ) {
        if self.layers.len() <= self.prepare_layer {
            self.layers.push(Layer::new(device, &self.constant_layout));
        }

        let layer = &mut self.layers[self.prepare_layer];
        layer.prepare(device, queue, instances, transformation, scale);

        self.prepare_layer += 1;
    }

    pub fn render<'a>(
        &'a self,
        layer: usize,
        bounds: Rectangle<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if let Some(layer) = self.layers.get(layer) {
            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_scissor_rect(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );

            render_pass.set_index_buffer(
                self.indices.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));

            layer.draw(render_pass);
        }
    }

    pub fn end_frame(&mut self) {
        self.prepare_layer = 0;
    }
}

#[derive(Debug)]
struct Layer {
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
    instances: Buffer<layer::Quad>,
    instance_count: usize,
}

impl Layer {
    pub fn new(
        device: &wgpu::Device,
        constant_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let constants_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::quad uniforms buffer"),
            size: mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::quad uniforms bind group"),
            layout: constant_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: constants_buffer.as_entire_binding(),
            }],
        });

        let instances = Buffer::new(
            device,
            "iced_wgpu::quad instance buffer",
            INITIAL_INSTANCES,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            constants,
            constants_buffer,
            instances,
            instance_count: 0,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[layer::Quad],
        transformation: Transformation,
        scale: f32,
    ) {
        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Quad", "PREPARE").entered();

        let uniforms = Uniforms::new(transformation, scale);

        queue.write_buffer(
            &self.constants_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );

        let _ = self.instances.resize(device, instances.len());
        self.instances.write(queue, 0, instances);
        self.instance_count = instances.len();
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Quad", "DRAW").entered();

        render_pass.set_bind_group(0, &self.constants, &[]);
        render_pass.set_vertex_buffer(1, self.instances.slice(..));

        render_pass.draw_indexed(
            0..QUAD_INDICES.len() as u32,
            0,
            0..self.instance_count as u32,
        );
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

const INITIAL_INSTANCES: usize = 10_000;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
    // Uniforms must be aligned to their largest member,
    // this uses a mat4x4<f32> which aligns to 16, so align to that
    _padding: [f32; 3],
}

impl Uniforms {
    fn new(transformation: Transformation, scale: f32) -> Uniforms {
        Self {
            transform: *transformation.as_ref(),
            scale,
            _padding: [0.0; 3],
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
            scale: 1.0,
            _padding: [0.0; 3],
        }
    }
}

// mod solid {
//     use crate::buffer;
//     use crate::quad::{QuadVertex, INITIAL_INSTANCES};
//     use iced_graphics::layer::quad;
//     use std::mem;
//
//     #[derive(Debug)]
//     pub struct Pipeline {
//         pub pipeline: wgpu::RenderPipeline,
//         pub instances: buffer::Static<quad::Solid>,
//     }
//
//     impl Pipeline {
//         pub fn new(
//             device: &wgpu::Device,
//             format: wgpu::TextureFormat,
//             uniforms_layout: &wgpu::BindGroupLayout,
//         ) -> Self {
//             let instances = buffer::Static::new(
//                 device,
//                 "iced_wgpu::quad::solid instance buffer",
//                 wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//                 INITIAL_INSTANCES,
//             );
//
//             let layout = device.create_pipeline_layout(
//                 &wgpu::PipelineLayoutDescriptor {
//                     label: Some("iced_wgpu::quad::solid pipeline layout"),
//                     push_constant_ranges: &[],
//                     bind_group_layouts: &[uniforms_layout],
//                 },
//             );
//
//             let shader =
//                 device.create_shader_module(wgpu::ShaderModuleDescriptor {
//                     label: Some("iced_wgpu::quad::solid shader"),
//                     source: wgpu::ShaderSource::Wgsl(
//                         std::borrow::Cow::Borrowed(include_str!(
//                             "shader/quad.wgsl"
//                         )),
//                     ),
//                 });
//
//             let pipeline = device.create_render_pipeline(
//                 &wgpu::RenderPipelineDescriptor {
//                     label: Some("iced_wgpu::quad::solid pipeline"),
//                     layout: Some(&layout),
//                     vertex: wgpu::VertexState {
//                         module: &shader,
//                         entry_point: "solid_vs_main",
//                         buffers: &[
//                             wgpu::VertexBufferLayout {
//                                 array_stride: mem::size_of::<QuadVertex>()
//                                     as u64,
//                                 step_mode: wgpu::VertexStepMode::Vertex,
//                                 attributes: &[wgpu::VertexAttribute {
//                                     shader_location: 0,
//                                     format: wgpu::VertexFormat::Float32x2,
//                                     offset: 0,
//                                 }],
//                             },
//                             wgpu::VertexBufferLayout {
//                                 array_stride: mem::size_of::<quad::Solid>()
//                                     as u64,
//                                 step_mode: wgpu::VertexStepMode::Instance,
//                                 attributes: &wgpu::vertex_attr_array!(
//                                     // Color
//                                     1 => Float32x4,
//                                     // Position
//                                     2 => Float32x2,
//                                     // Size
//                                     3 => Float32x2,
//                                     // Border color
//                                     4 => Float32x4,
//                                     // Border radius
//                                     5 => Float32x4,
//                                     // Border width
//                                     6 => Float32,
//                                 ),
//                             },
//                         ],
//                     },
//                     fragment: Some(wgpu::FragmentState {
//                         module: &shader,
//                         entry_point: "solid_fs_main",
//                         targets: &[Some(wgpu::ColorTargetState {
//                             format,
//                             blend: Some(wgpu::BlendState {
//                                 color: wgpu::BlendComponent {
//                                     src_factor: wgpu::BlendFactor::SrcAlpha,
//                                     dst_factor:
//                                     wgpu::BlendFactor::OneMinusSrcAlpha,
//                                     operation: wgpu::BlendOperation::Add,
//                                 },
//                                 alpha: wgpu::BlendComponent {
//                                     src_factor: wgpu::BlendFactor::One,
//                                     dst_factor:
//                                     wgpu::BlendFactor::OneMinusSrcAlpha,
//                                     operation: wgpu::BlendOperation::Add,
//                                 },
//                             }),
//                             write_mask: wgpu::ColorWrites::ALL,
//                         })],
//                     }),
//                     primitive: wgpu::PrimitiveState {
//                         topology: wgpu::PrimitiveTopology::TriangleList,
//                         front_face: wgpu::FrontFace::Cw,
//                         ..Default::default()
//                     },
//                     depth_stencil: None,
//                     multisample: wgpu::MultisampleState {
//                         count: 1,
//                         mask: !0,
//                         alpha_to_coverage_enabled: false,
//                     },
//                     multiview: None,
//                 },
//             );
//
//             Self {
//                 pipeline,
//                 instances,
//             }
//         }
//     }
// }
//
// mod gradient {
//     use crate::buffer;
//     use crate::quad::{QuadVertex, INITIAL_INSTANCES};
//     use iced_graphics::layer::quad;
//     use std::mem;
//
//     #[derive(Debug)]
//     pub struct Pipeline {
//         pub pipeline: wgpu::RenderPipeline,
//         pub instances: buffer::Static<quad::Gradient>,
//     }
//
//     impl Pipeline {
//         pub fn new(
//             device: &wgpu::Device,
//             format: wgpu::TextureFormat,
//             uniforms_layout: &wgpu::BindGroupLayout,
//         ) -> Self {
//             let instances = buffer::Static::new(
//                 device,
//                 "iced_wgpu::quad::gradient instance buffer",
//                 wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//                 INITIAL_INSTANCES,
//             );
//
//             let layout = device.create_pipeline_layout(
//                 &wgpu::PipelineLayoutDescriptor {
//                     label: Some("iced_wgpu::quad::gradient pipeline layout"),
//                     push_constant_ranges: &[],
//                     bind_group_layouts: &[uniforms_layout],
//                 },
//             );
//
//             let shader =
//                 device.create_shader_module(wgpu::ShaderModuleDescriptor {
//                     label: Some("iced_wgpu::quad::gradient shader"),
//                     source: wgpu::ShaderSource::Wgsl(
//                         std::borrow::Cow::Borrowed(include_str!(
//                             "shader/quad.wgsl"
//                         )),
//                     ),
//                 });
//
//             let pipeline = device.create_render_pipeline(
//                 &wgpu::RenderPipelineDescriptor {
//                     label: Some("iced_wgpu::quad::gradient pipeline"),
//                     layout: Some(&layout),
//                     vertex: wgpu::VertexState {
//                         module: &shader,
//                         entry_point: "gradient_vs_main",
//                         buffers: &[
//                             wgpu::VertexBufferLayout {
//                                 array_stride: mem::size_of::<QuadVertex>()
//                                     as u64,
//                                 step_mode: wgpu::VertexStepMode::Vertex,
//                                 attributes: &[wgpu::VertexAttribute {
//                                     shader_location: 0,
//                                     format: wgpu::VertexFormat::Float32x2,
//                                     offset: 0,
//                                 }],
//                             },
//                             wgpu::VertexBufferLayout {
//                                 array_stride: mem::size_of::<quad::Gradient>()
//                                     as u64,
//                                 step_mode: wgpu::VertexStepMode::Instance,
//                                 attributes: &wgpu::vertex_attr_array!(
//                                     // Color 1
//                                     1 => Float32x4,
//                                     // Color 2
//                                     2 => Float32x4,
//                                     // Color 3
//                                     3 => Float32x4,
//                                     // Color 4
//                                     4 => Float32x4,
//                                     // Color 5
//                                     5 => Float32x4,
//                                     // Color 6
//                                     6 => Float32x4,
//                                     // Color 7
//                                     7 => Float32x4,
//                                     // Color 8
//                                     8 => Float32x4,
//                                     // Offsets 1-4
//                                     9 => Float32x4,
//                                     // Offsets 5-8
//                                     10 => Float32x4,
//                                     // Direction
//                                     11 => Float32x4,
//                                     // Position & Scale
//                                     12 => Float32x4,
//                                     // Border color
//                                     13 => Float32x4,
//                                     // Border radius
//                                     14 => Float32x4,
//                                     // Border width
//                                     15 => Float32
//                                 ),
//                             },
//                         ],
//                     },
//                     fragment: Some(wgpu::FragmentState {
//                         module: &shader,
//                         entry_point: "gradient_fs_main",
//                         targets: &[Some(wgpu::ColorTargetState {
//                             format,
//                             blend: Some(wgpu::BlendState {
//                                 color: wgpu::BlendComponent {
//                                     src_factor: wgpu::BlendFactor::SrcAlpha,
//                                     dst_factor:
//                                     wgpu::BlendFactor::OneMinusSrcAlpha,
//                                     operation: wgpu::BlendOperation::Add,
//                                 },
//                                 alpha: wgpu::BlendComponent {
//                                     src_factor: wgpu::BlendFactor::One,
//                                     dst_factor:
//                                     wgpu::BlendFactor::OneMinusSrcAlpha,
//                                     operation: wgpu::BlendOperation::Add,
//                                 },
//                             }),
//                             write_mask: wgpu::ColorWrites::ALL,
//                         })],
//                     }),
//                     primitive: wgpu::PrimitiveState {
//                         topology: wgpu::PrimitiveTopology::TriangleList,
//                         front_face: wgpu::FrontFace::Cw,
//                         ..Default::default()
//                     },
//                     depth_stencil: None,
//                     multisample: wgpu::MultisampleState {
//                         count: 1,
//                         mask: !0,
//                         alpha_to_coverage_enabled: false,
//                     },
//                     multiview: None,
//                 },
//             );
//
//             Self {
//                 pipeline,
//                 instances,
//             }
//         }
//     }
// }