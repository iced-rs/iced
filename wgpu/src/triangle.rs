//! Draw meshes of triangles.
use crate::{settings, Transformation};
use core::fmt;
use std::fmt::Formatter;

use iced_graphics::layer::Meshes;
use iced_graphics::shader::Shader;
use iced_graphics::Size;

use crate::buffers::buffer::{needs_recreate, StaticBuffer};
use crate::triangle::gradient::GradientPipeline;
use crate::triangle::solid::SolidPipeline;
pub use iced_graphics::triangle::{Mesh2D, Vertex2D};

mod gradient;
mod msaa;
mod solid;

/// Triangle pipeline for all mesh layers in a [`iced_graphics::Canvas`] widget.
#[derive(Debug)]
pub(crate) struct Pipeline {
    blit: Option<msaa::Blit>,
    // these are optional so we don't allocate any memory to the GPU if
    // application has no triangle meshes.
    vertex_buffer: Option<StaticBuffer>,
    index_buffer: Option<StaticBuffer>,
    pipelines: TrianglePipelines,
}

/// Supported triangle pipelines for different fills. Both use the same vertex shader.
pub(crate) struct TrianglePipelines {
    solid: SolidPipeline,
    gradient: GradientPipeline,
}

impl fmt::Debug for TrianglePipelines {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrianglePipelines").finish()
    }
}

impl TrianglePipelines {
    /// Resets each pipeline's buffers.
    fn clear(&mut self) {
        self.solid.buffer.clear();
        self.gradient.uniform_buffer.clear();
        self.gradient.storage_buffer.clear();
    }

    /// Writes the contents of each pipeline's CPU buffer to the GPU, resizing the GPU buffer
    /// beforehand if necessary.
    fn write(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.solid.write(device, staging_belt, encoder);
        self.gradient.write(device, staging_belt, encoder);
    }
}

impl Pipeline {
    /// Creates supported GL programs, listed in [TrianglePipelines].
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Pipeline {
        Pipeline {
            blit: antialiasing.map(|a| msaa::Blit::new(device, format, a)),
            vertex_buffer: None,
            index_buffer: None,
            pipelines: TrianglePipelines {
                solid: SolidPipeline::new(device, format, antialiasing),
                gradient: GradientPipeline::new(device, format, antialiasing),
            },
        }
    }

    /// Draws the contents of the current layer's meshes to the [target].
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_size: Size<u32>,
        transformation: Transformation,
        scale_factor: f32,
        meshes: &Meshes<'_>,
    ) {
        //count the total number of vertices & indices we need to handle
        let (total_vertices, total_indices) = meshes.attribute_count();
        println!("total vertices: {}, total indices: {}", total_vertices, total_indices);

        //Only create buffers if they need to be re-sized or don't exist
        if needs_recreate(&self.vertex_buffer, total_vertices) {
            //mapped to GPU at creation with total vertices
            self.vertex_buffer = Some(StaticBuffer::new(
                device,
                "iced_wgpu::triangle vertex buffer",
                //TODO: a more reasonable default to prevent frequent resizing calls
                // before this was 10_000
                (std::mem::size_of::<Vertex2D>() * total_vertices) as u64,
                wgpu::BufferUsages::VERTEX,
                meshes.0.len(),
            ))
        }

        if needs_recreate(&self.index_buffer, total_indices) {
            //mapped to GPU at creation with total indices
            self.index_buffer = Some(StaticBuffer::new(
                device,
                "iced_wgpu::triangle index buffer",
                //TODO: a more reasonable default to prevent frequent resizing calls
                // before this was 10_000
                (std::mem::size_of::<Vertex2D>() * total_indices) as u64,
                wgpu::BufferUsages::INDEX,
                meshes.0.len(),
            ));
        }

        if let Some(vertex_buffer) = &mut self.vertex_buffer {
            if let Some(index_buffer) = &mut self.index_buffer {
                let mut offset_v = 0;
                let mut offset_i = 0;
                //TODO: store this more efficiently
                let mut indices_lengths = Vec::with_capacity(meshes.0.len());

                //iterate through meshes to write all attribute data
                for mesh in meshes.0.iter() {
                    let transform = transformation
                        * Transformation::translate(
                            mesh.origin.x,
                            mesh.origin.y,
                        );

                    println!("Mesh attribute data: Vertex: {:?}, Index: {:?}", mesh.buffers.vertices, mesh.buffers.indices);

                    let vertices = bytemuck::cast_slice(&mesh.buffers.vertices);
                    let indices = bytemuck::cast_slice(&mesh.buffers.indices);

                    //TODO: it's (probably) more efficient to reduce this write command and
                    // iterate first and then upload
                    println!("vertex buffer len: {}, index length: {}", vertices.len(), indices.len());
                    vertex_buffer.write(offset_v, vertices);
                    index_buffer.write(offset_i, indices);

                    offset_v += vertices.len() as u64;
                    offset_i += indices.len() as u64;
                    indices_lengths.push(mesh.buffers.indices.len());

                    match mesh.shader {
                        Shader::Solid(color) => {
                            self.pipelines.solid.push(transform, color);
                        }
                        Shader::Gradient(gradient) => {
                            self.pipelines.gradient.push(transform, gradient);
                        }
                    }
                }

                //done writing to gpu buffer, unmap from host memory since we don't need it
                //anymore
                vertex_buffer.flush();
                index_buffer.flush();

                //resize & memcpy uniforms from CPU buffers to GPU buffers for all pipelines
                self.pipelines.write(device, staging_belt, encoder);

                //configure the render pass now that the data is uploaded to the GPU
                {
                    //configure antialiasing pass
                    let (attachment, resolve_target, load) =
                        if let Some(blit) = &mut self.blit {
                            let (attachment, resolve_target) = blit.targets(
                                device,
                                target_size.width,
                                target_size.height,
                            );

                            (
                                attachment,
                                Some(resolve_target),
                                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            )
                        } else {
                            (target, None, wgpu::LoadOp::Load)
                        };

                    let mut render_pass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some("iced_wgpu::triangle render pass"),
                            color_attachments: &[Some(
                                wgpu::RenderPassColorAttachment {
                                    view: attachment,
                                    resolve_target,
                                    ops: wgpu::Operations { load, store: true },
                                },
                            )],
                            depth_stencil_attachment: None,
                        },
                    );

                    //TODO: do this a better way; store it in the respective pipelines perhaps
                    // to be more readable
                    let mut num_solids = 0;
                    let mut num_gradients = 0;

                    //TODO: try to avoid this extra iteration if possible
                    for index in 0..meshes.0.len() {
                        let clip_bounds =
                            (meshes.0[index].clip_bounds * scale_factor).snap();

                        render_pass.set_scissor_rect(
                            clip_bounds.x,
                            clip_bounds.y,
                            clip_bounds.width,
                            clip_bounds.height,
                        );

                        match meshes.0[index].shader {
                            Shader::Solid(_) => {
                                self.pipelines.solid.configure_render_pass(
                                    &mut render_pass,
                                    num_solids,
                                );
                                num_solids += 1;
                            }
                            Shader::Gradient(_) => {
                                self.pipelines.gradient.configure_render_pass(
                                    &mut render_pass,
                                    num_gradients,
                                );
                                num_gradients += 1;
                            }
                        }

                        render_pass.set_index_buffer(
                            index_buffer.slice_from_index::<u32>(index),
                            wgpu::IndexFormat::Uint32,
                        );

                        render_pass.set_vertex_buffer(
                            0,
                            vertex_buffer.slice_from_index::<Vertex2D>(index),
                        );

                        render_pass.draw_indexed(
                            0..(indices_lengths[index] as u32),
                            0,
                            0..1,
                        );
                    }
                }
            }
        }

        if let Some(blit) = &mut self.blit {
            blit.draw(encoder, target);
        }

        //cleanup
        self.pipelines.clear();
    }
}

//utility functions for individual pipelines with shared functionality
fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex2D>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    }
}

fn default_fragment_target(
    texture_format: wgpu::TextureFormat,
) -> Option<wgpu::ColorTargetState> {
    Some(wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    })
}

fn default_triangle_primitive_state() -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        front_face: wgpu::FrontFace::Cw,
        ..Default::default()
    }
}

fn default_multisample_state(
    antialiasing: Option<settings::Antialiasing>,
) -> wgpu::MultisampleState {
    wgpu::MultisampleState {
        count: antialiasing.map(|a| a.sample_count()).unwrap_or(1),
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}
