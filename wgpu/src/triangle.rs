//! Draw meshes of triangles.
use crate::{settings, Transformation};
use iced_graphics::layer;

use bytemuck::{Pod, Zeroable};
use std::mem;

pub use iced_graphics::triangle::{Mesh2D, Vertex2D};

mod msaa;

const UNIFORM_BUFFER_SIZE: usize = 50;
const VERTEX_BUFFER_SIZE: usize = 10_000;
const INDEX_BUFFER_SIZE: usize = 10_000;

#[derive(Debug)]
pub(crate) struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    blit: Option<msaa::Blit>,
    constants_layout: wgpu::BindGroupLayout,
    constants: wgpu::BindGroup,
    uniforms_buffer: Buffer<Uniforms>,
    vertex_buffer: Buffer<Vertex2D>,
    index_buffer: Buffer<u32>,
}

#[derive(Debug)]
struct Buffer<T> {
    label: &'static str,
    raw: wgpu::Buffer,
    size: usize,
    usage: wgpu::BufferUsages,
    _type: std::marker::PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(
        label: &'static str,
        device: &wgpu::Device,
        size: usize,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let raw = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (std::mem::size_of::<T>() * size) as u64,
            usage,
            mapped_at_creation: false,
        });

        Buffer {
            label,
            raw,
            size,
            usage,
            _type: std::marker::PhantomData,
        }
    }

    pub fn expand(&mut self, device: &wgpu::Device, size: usize) -> bool {
        let needs_resize = self.size < size;

        if needs_resize {
            self.raw = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: (std::mem::size_of::<T>() * size) as u64,
                usage: self.usage,
                mapped_at_creation: false,
            });

            self.size = size;
        }

        needs_resize
    }
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Pipeline {
        let constants_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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

        let constants_buffer = Buffer::new(
            "iced_wgpu::triangle uniforms buffer",
            device,
            UNIFORM_BUFFER_SIZE,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let constant_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::triangle uniforms bind group"),
                layout: &constants_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &constants_buffer.raw,
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

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::triangle pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&constants_layout],
            });

        let shader =
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu::triangle::shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader/triangle.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::triangle pipeline"),
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
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: u32::from(
                        antialiasing.map(|a| a.sample_count()).unwrap_or(1),
                    ),
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

        Pipeline {
            pipeline,
            blit: antialiasing.map(|a| msaa::Blit::new(device, format, a)),
            constants_layout,
            constants: constant_bind_group,
            uniforms_buffer: constants_buffer,
            vertex_buffer: Buffer::new(
                "iced_wgpu::triangle vertex buffer",
                device,
                VERTEX_BUFFER_SIZE,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            ),
            index_buffer: Buffer::new(
                "iced_wgpu::triangle index buffer",
                device,
                INDEX_BUFFER_SIZE,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            ),
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
        transformation: Transformation,
        scale_factor: f32,
        meshes: &[layer::Mesh<'_>],
    ) {
        // This looks a bit crazy, but we are just counting how many vertices
        // and indices we will need to handle.
        // TODO: Improve readability
        let (total_vertices, total_indices) = meshes
            .iter()
            .map(|layer::Mesh { buffers, .. }| {
                (buffers.vertices.len(), buffers.indices.len())
            })
            .fold((0, 0), |(total_v, total_i), (v, i)| {
                (total_v + v, total_i + i)
            });

        // Then we ensure the current buffers are big enough, resizing if
        // necessary
        let _ = self.vertex_buffer.expand(device, total_vertices);
        let _ = self.index_buffer.expand(device, total_indices);

        // If the uniforms buffer is resized, then we need to recreate its
        // bind group.
        if self.uniforms_buffer.expand(device, meshes.len()) {
            self.constants =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wgpu::triangle uniforms buffer"),
                    layout: &self.constants_layout,
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

        let mut uniforms: Vec<Uniforms> = Vec::with_capacity(meshes.len());
        let mut offsets: Vec<(
            wgpu::BufferAddress,
            wgpu::BufferAddress,
            usize,
        )> = Vec::with_capacity(meshes.len());
        let mut last_vertex = 0;
        let mut last_index = 0;

        // We upload everything upfront
        for mesh in meshes {
            let transform = (transformation
                * Transformation::translate(mesh.origin.x, mesh.origin.y))
            .into();

            let vertices = bytemuck::cast_slice(&mesh.buffers.vertices);
            let indices = bytemuck::cast_slice(&mesh.buffers.indices);

            match (
                wgpu::BufferSize::new(vertices.len() as u64),
                wgpu::BufferSize::new(indices.len() as u64),
            ) {
                (Some(vertices_size), Some(indices_size)) => {
                    {
                        let mut vertex_buffer = staging_belt.write_buffer(
                            encoder,
                            &self.vertex_buffer.raw,
                            (std::mem::size_of::<Vertex2D>() * last_vertex)
                                as u64,
                            vertices_size,
                            device,
                        );

                        vertex_buffer.copy_from_slice(vertices);
                    }

                    {
                        let mut index_buffer = staging_belt.write_buffer(
                            encoder,
                            &self.index_buffer.raw,
                            (std::mem::size_of::<u32>() * last_index) as u64,
                            indices_size,
                            device,
                        );

                        index_buffer.copy_from_slice(indices);
                    }

                    uniforms.push(transform);
                    offsets.push((
                        last_vertex as u64,
                        last_index as u64,
                        mesh.buffers.indices.len(),
                    ));

                    last_vertex += mesh.buffers.vertices.len();
                    last_index += mesh.buffers.indices.len();
                }
                _ => {}
            }
        }

        let uniforms = bytemuck::cast_slice(&uniforms);

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

        {
            let (attachment, resolve_target, load) =
                if let Some(blit) = &mut self.blit {
                    let (attachment, resolve_target) =
                        blit.targets(device, target_width, target_height);

                    (
                        attachment,
                        Some(resolve_target),
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    )
                } else {
                    (target, None, wgpu::LoadOp::Load)
                };

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu::triangle render pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: attachment,
                        resolve_target,
                        ops: wgpu::Operations { load, store: true },
                    }],
                    depth_stencil_attachment: None,
                });

            render_pass.set_pipeline(&self.pipeline);

            for (i, (vertex_offset, index_offset, indices)) in
                offsets.into_iter().enumerate()
            {
                let clip_bounds = (meshes[i].clip_bounds * scale_factor).snap();

                render_pass.set_scissor_rect(
                    clip_bounds.x,
                    clip_bounds.y,
                    clip_bounds.width,
                    clip_bounds.height,
                );

                render_pass.set_bind_group(
                    0,
                    &self.constants,
                    &[(std::mem::size_of::<Uniforms>() * i) as u32],
                );

                render_pass.set_index_buffer(
                    self.index_buffer
                        .raw
                        .slice(index_offset * mem::size_of::<u32>() as u64..),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.set_vertex_buffer(
                    0,
                    self.vertex_buffer.raw.slice(
                        vertex_offset * mem::size_of::<Vertex2D>() as u64..,
                    ),
                );

                render_pass.draw_indexed(0..indices as u32, 0, 0..1);
            }
        }

        if let Some(blit) = &mut self.blit {
            blit.draw(encoder, target);
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    transform: [f32; 16],
    // We need to align this to 256 bytes to please `wgpu`...
    // TODO: Be smarter and stop wasting memory!
    _padding_a: [f32; 32],
    _padding_b: [f32; 16],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
            _padding_a: [0.0; 32],
            _padding_b: [0.0; 16],
        }
    }
}

impl From<Transformation> for Uniforms {
    fn from(transformation: Transformation) -> Uniforms {
        Self {
            transform: transformation.into(),
            _padding_a: [0.0; 32],
            _padding_b: [0.0; 16],
        }
    }
}
