//! Draw meshes of triangles.
#[cfg(not(target_arch = "wasm32"))]
mod gradient;
mod msaa;
mod solid;

use crate::buffer::r#static::Buffer;
use crate::settings;
use crate::Transformation;

use iced_graphics::layer::mesh::{self, Mesh};
use iced_graphics::triangle::{self, Vertex2D};
use iced_graphics::Size;

use core::fmt;
use std::fmt::Formatter;

/// Triangle pipeline for all mesh layers in a [`iced_graphics::Canvas`] widget.
#[derive(Debug)]
pub(crate) struct Pipeline {
    blit: Option<msaa::Blit>,
    vertex_buffer: Buffer<Vertex2D>,
    index_buffer: Buffer<u32>,
    index_strides: Vec<u32>,
    pipelines: PipelineList,
}

/// Supported triangle pipelines for different fills.
pub(crate) struct PipelineList {
    solid: solid::Pipeline,
    /// Gradients are currently not supported on WASM targets due to their need of storage buffers.
    #[cfg(not(target_arch = "wasm32"))]
    gradient: gradient::Pipeline,
}

impl fmt::Debug for PipelineList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrianglePipelines").finish()
    }
}

impl PipelineList {
    /// Resets each pipeline's buffers.
    fn clear(&mut self) {
        self.solid.buffer.clear();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.gradient.uniform_buffer.clear();
            self.gradient.storage_buffer.clear();
        }
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
        #[cfg(not(target_arch = "wasm32"))]
        self.gradient.write(device, staging_belt, encoder);
    }
}

impl Pipeline {
    /// Creates supported pipelines, listed in [TrianglePipelines].
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Pipeline {
        Pipeline {
            blit: antialiasing.map(|a| msaa::Blit::new(device, format, a)),
            vertex_buffer: Buffer::new(
                device,
                "iced_wgpu::triangle vertex buffer",
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            ),
            index_buffer: Buffer::new(
                device,
                "iced_wgpu::triangle vertex buffer",
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            ),
            index_strides: Vec::new(),
            pipelines: PipelineList {
                solid: solid::Pipeline::new(device, format, antialiasing),
                #[cfg(not(target_arch = "wasm32"))]
                gradient: gradient::Pipeline::new(device, format, antialiasing),
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
        meshes: &[Mesh<'_>],
    ) {
        //count the total amount of vertices & indices we need to handle
        let (total_vertices, total_indices) = mesh::attribute_count_of(meshes);

        // Then we ensure the current attribute buffers are big enough, resizing if necessary.

        //We are not currently using the return value of these functions as we have no system in
        //place to calculate mesh diff, or to know whether or not that would be more performant for
        //the majority of use cases. Therefore we will write GPU data every frame (for now).
        let _ = self.vertex_buffer.resize(device, total_vertices);
        let _ = self.index_buffer.resize(device, total_indices);

        //prepare dynamic buffers & data store for writing
        self.index_strides.clear();
        self.pipelines.clear();

        let mut vertex_offset = 0;
        let mut index_offset = 0;

        for mesh in meshes {
            let transform = transformation
                * Transformation::translate(mesh.origin.x, mesh.origin.y);

            //write to both buffers
            let new_vertex_offset = self.vertex_buffer.write(
                device,
                staging_belt,
                encoder,
                vertex_offset,
                &mesh.buffers.vertices,
            );

            let new_index_offset = self.index_buffer.write(
                device,
                staging_belt,
                encoder,
                index_offset,
                &mesh.buffers.indices,
            );

            vertex_offset += new_vertex_offset;
            index_offset += new_index_offset;

            self.index_strides.push(mesh.buffers.indices.len() as u32);

            //push uniform data to CPU buffers
            match mesh.style {
                triangle::Style::Solid(color) => {
                    self.pipelines.solid.push(transform, color);
                }
                #[cfg(not(target_arch = "wasm32"))]
                triangle::Style::Gradient(gradient) => {
                    self.pipelines.gradient.push(transform, gradient);
                }
            }
        }

        //write uniform data to GPU
        self.pipelines.write(device, staging_belt, encoder);

        //configure the render pass now that the data is uploaded to the GPU
        {
            //configure antialiasing pass
            let (attachment, resolve_target, load) = if let Some(blit) =
                &mut self.blit
            {
                let (attachment, resolve_target) =
                    blit.targets(device, target_size.width, target_size.height);

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
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: attachment,
                            resolve_target,
                            ops: wgpu::Operations { load, store: true },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            let mut num_solids = 0;
            #[cfg(not(target_arch = "wasm32"))]
            let mut num_gradients = 0;
            let mut last_is_solid = None;

            for (index, mesh) in meshes.iter().enumerate() {
                let clip_bounds = (mesh.clip_bounds * scale_factor).snap();

                render_pass.set_scissor_rect(
                    clip_bounds.x,
                    clip_bounds.y,
                    clip_bounds.width,
                    clip_bounds.height,
                );

                match mesh.style {
                    triangle::Style::Solid(_) => {
                        if !last_is_solid.unwrap_or(false) {
                            self.pipelines
                                .solid
                                .set_render_pass_pipeline(&mut render_pass);

                            last_is_solid = Some(true);
                        }

                        self.pipelines.solid.configure_render_pass(
                            &mut render_pass,
                            num_solids,
                        );

                        num_solids += 1;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    triangle::Style::Gradient(_) => {
                        if last_is_solid.unwrap_or(true) {
                            self.pipelines
                                .gradient
                                .set_render_pass_pipeline(&mut render_pass);

                            last_is_solid = Some(false);
                        }

                        self.pipelines.gradient.configure_render_pass(
                            &mut render_pass,
                            num_gradients,
                        );

                        num_gradients += 1;
                    }
                };

                render_pass.set_vertex_buffer(
                    0,
                    self.vertex_buffer.slice_from_index(index),
                );

                render_pass.set_index_buffer(
                    self.index_buffer.slice_from_index(index),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(
                    0..(self.index_strides[index] as u32),
                    0,
                    0..1,
                );
            }
        }

        self.vertex_buffer.clear();
        self.index_buffer.clear();

        if let Some(blit) = &mut self.blit {
            blit.draw(encoder, target);
        }
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

fn fragment_target(
    texture_format: wgpu::TextureFormat,
) -> Option<wgpu::ColorTargetState> {
    Some(wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    })
}

fn primitive_state() -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        front_face: wgpu::FrontFace::Cw,
        ..Default::default()
    }
}

fn multisample_state(
    antialiasing: Option<settings::Antialiasing>,
) -> wgpu::MultisampleState {
    wgpu::MultisampleState {
        count: antialiasing.map(|a| a.sample_count()).unwrap_or(1),
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}
