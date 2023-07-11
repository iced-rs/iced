use crate::graphics::color;
use crate::quad::{self, Quad};
use crate::Buffer;

use bytemuck::{Pod, Zeroable};
use std::ops::Range;

/// A quad filled with a solid color.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Solid {
    /// The background color data of the quad.
    pub color: color::Packed,

    /// The [`Quad`] data of the [`Solid`].
    pub quad: Quad,
}

#[derive(Debug)]
pub struct Layer {
    instances: Buffer<Solid>,
    instance_count: usize,
}

impl Layer {
    pub fn new(device: &wgpu::Device) -> Self {
        let instances = Buffer::new(
            device,
            "iced_wgpu.quad.solid.buffer",
            quad::INITIAL_INSTANCES,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            instances,
            instance_count: 0,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[Solid],
    ) {
        let _ = self.instances.resize(device, instances.len());
        let _ = self.instances.write(queue, 0, instances);

        self.instance_count = instances.len();
    }
}

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        constants_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu.quad.solid.pipeline"),
                push_constant_ranges: &[],
                bind_group_layouts: &[constants_layout],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu.quad.solid.shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shader/quad.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu.quad.solid.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "solid_vs_main",
                    buffers: &[
                        quad::Vertex::buffer_layout(),
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Solid>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array!(
                                // Color
                                1 => Float32x4,
                                // Position
                                2 => Float32x2,
                                // Size
                                3 => Float32x2,
                                // Border color
                                4 => Float32x4,
                                // Border radius
                                5 => Float32x4,
                                // Border width
                                6 => Float32,
                            ),
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "solid_fs_main",
                    targets: &quad::color_target_state(format),
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

        Self { pipeline }
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        constants: &'a wgpu::BindGroup,
        layer: &'a Layer,
        range: Range<usize>,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, constants, &[]);
        render_pass.set_vertex_buffer(1, layer.instances.slice(..));

        render_pass.draw_indexed(
            0..quad::INDICES.len() as u32,
            0,
            range.start as u32..range.end as u32,
        );
    }
}
