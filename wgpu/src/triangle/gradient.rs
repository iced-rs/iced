use std::mem;

use bytemuck::{Pod, Zeroable};
use iced_graphics::layer::Mesh;
use iced_graphics::triangle::shader;
use iced_graphics::triangle::shader::gradient::ColorStop;
use iced_graphics::triangle::{Shader, Vertex2D};
use iced_graphics::{Rectangle, Transformation};

use crate::settings;

use super::Buffer;

const UNIFORM_BUFFER_SIZE: usize = 50;
const COLOR_STOPS_BUFFER_SIZE: usize = 4;

#[derive(Debug)]
pub(super) struct Gradient {
    pipeline: wgpu::RenderPipeline,
    uniforms: Vec<Uniforms>,
    uniforms_bind_group: wgpu::BindGroup,
    uniforms_layout: wgpu::BindGroupLayout,
    uniforms_buffer: Buffer<Uniforms>,
    color_stops: Vec<ColorStop>,
    color_stops_bind_group: wgpu::BindGroup,
    color_stops_layout: wgpu::BindGroupLayout,
    color_stops_buffer: Buffer<ColorStop>,
    color_stops_offset: usize,
}

impl Gradient {
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Self {
        let uniforms_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle::gradient uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<Uniforms>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let uniforms_buffer = Buffer::new(
            "iced_wgpu::triangle::gradient uniforms buffer",
            device,
            UNIFORM_BUFFER_SIZE,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let uniforms_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(
                    "iced_wgpu::triangle::gradient uniforms bind group",
                ),
                layout: &uniforms_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &uniforms_buffer.raw,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<
                                Uniforms,
                            >(
                            )
                                as u64),
                        },
                    ),
                }],
            });

        let color_stops_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle::gradient color_stops layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ColorStop>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let color_stops_buffer = Buffer::new(
            "iced_wgpu::triangle::gradient color_stops buffer",
            device,
            COLOR_STOPS_BUFFER_SIZE,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );

        let color_stops_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(
                    "iced_wgpu::triangle::gradient color_stops bind group",
                ),
                layout: &color_stops_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &color_stops_buffer.raw,
                            offset: 0,
                            size: None,
                        },
                    ),
                }],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::triangle::gradient pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&uniforms_layout, &color_stops_layout],
            });

        let shader =
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu::triangle::gradient shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shader/gradient.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::triangle::gradient pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex2D>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array!(
                            // Position
                            0 => Float32x2,
                            // Color
                            1 => Float32x4,
                        ),
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: antialiasing.map(|a| a.sample_count()).unwrap_or(1),
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Self {
            pipeline,
            uniforms: vec![],
            uniforms_bind_group,
            uniforms_layout,
            uniforms_buffer,
            color_stops: vec![],
            color_stops_bind_group,
            color_stops_layout,
            color_stops_buffer,
            color_stops_offset: 0,
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        meshes: &[Mesh<'_>],
    ) {
        let total_color_stops = meshes
            .iter()
            .filter_map(|mesh| match mesh.shader {
                Shader::Solid => None,
                Shader::Gradient(shader::Gradient::Linear {
                    stops, ..
                }) => Some(stops),
            })
            .flatten()
            .count();

        self.uniforms = Vec::with_capacity(meshes.len());
        if self.uniforms_buffer.expand(device, meshes.len()) {
            self.uniforms_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(
                        "iced_wgpu::triangle::gradient uniforms bind group",
                    ),
                    layout: &self.uniforms_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &self.uniforms_buffer.raw,
                                offset: 0,
                                size: wgpu::BufferSize::new(
                                    std::mem::size_of::<Uniforms>() as u64,
                                ),
                            },
                        ),
                    }],
                });
        }

        self.color_stops_offset = 0;
        self.color_stops = Vec::with_capacity(total_color_stops);
        if self.color_stops_buffer.expand(device, total_color_stops) {
            self.color_stops_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(
                        "iced_wgpu::triangle::gradient color_stops bind group",
                    ),
                    layout: &self.color_stops_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &self.color_stops_buffer.raw,
                                offset: 0,
                                size: None,
                            },
                        ),
                    }],
                });
        }
    }

    pub(super) fn add(
        &mut self,
        transformation: Transformation,
        gradient: &shader::Gradient,
    ) {
        let shader::Gradient::Linear { start, end, stops } = gradient;

        let start_stop = self.color_stops_offset as i32;
        self.color_stops_offset += stops.len();
        let end_stop = self.color_stops_offset as i32 - 1;

        self.color_stops.extend(stops);

        self.uniforms.push(Uniforms {
            transform: transformation.into(),
            start: (*start).into(),
            end: (*end).into(),
            start_stop,
            end_stop,
            _padding: [0; 2],
        });
    }

    pub(super) fn upload(
        &self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if !self.uniforms.is_empty() {
            let uniforms = bytemuck::cast_slice(&self.uniforms);

            if let Some(uniforms_size) =
                wgpu::BufferSize::new(uniforms.len() as u64)
            {
                let mut uniforms_buffer = staging_belt.write_buffer(
                    encoder,
                    &self.uniforms_buffer.raw,
                    0,
                    uniforms_size,
                    device,
                );

                uniforms_buffer.copy_from_slice(uniforms);
            }
        }

        if !self.color_stops.is_empty() {
            let color_stops = bytemuck::cast_slice(&self.color_stops);

            if let Some(color_stops_size) =
                wgpu::BufferSize::new(color_stops.len() as u64)
            {
                let mut color_stops_buffer = staging_belt.write_buffer(
                    encoder,
                    &self.color_stops_buffer.raw,
                    0,
                    color_stops_size,
                    device,
                );

                color_stops_buffer.copy_from_slice(color_stops);
            }
        }
    }

    pub(super) fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        clip_bounds: Rectangle<u32>,
        index_buffer: &'a Buffer<u32>,
        index_offset: u64,
        vertex_buffer: &'a Buffer<Vertex2D>,
        vertex_offset: u64,
        indices: usize,
        i: usize,
    ) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );

        render_pass.set_bind_group(
            0,
            &self.uniforms_bind_group,
            &[(std::mem::size_of::<Uniforms>() * i) as u32],
        );

        render_pass.set_bind_group(1, &self.color_stops_bind_group, &[]);

        render_pass.set_index_buffer(
            index_buffer
                .raw
                .slice(index_offset * mem::size_of::<u32>() as u64..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.set_vertex_buffer(
            0,
            vertex_buffer
                .raw
                .slice(vertex_offset * mem::size_of::<Vertex2D>() as u64..),
        );

        render_pass.draw_indexed(0..indices as u32, 0, 0..1);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    transform: [f32; 16],
    start: [f32; 2],
    end: [f32; 2],
    start_stop: i32,
    end_stop: i32,
    _padding: [u32; 2],
}
