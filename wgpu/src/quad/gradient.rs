use crate::graphics::gradient;
use crate::quad::{self, Quad};
use crate::Buffer;

use bytemuck::{Pod, Zeroable};
use std::ops::Range;

/// A quad filled with interpolated colors.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Gradient {
    /// The background gradient data of the quad.
    pub gradient: gradient::Packed,

    /// The [`Quad`] data of the [`Gradient`].
    pub quad: Quad,
}

#[allow(unsafe_code)]
unsafe impl Pod for Gradient {}

#[allow(unsafe_code)]
unsafe impl Zeroable for Gradient {}

#[derive(Debug)]
pub struct Layer {
    instances: Buffer<Gradient>,
    instance_count: usize,
}

impl Layer {
    pub fn new(device: &wgpu::Device) -> Self {
        let instances = Buffer::new(
            device,
            "iced_wgpu.quad.gradient.buffer",
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
        instances: &[Gradient],
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
                label: Some("iced_wgpu.quad.gradient.pipeline"),
                push_constant_ranges: &[],
                bind_group_layouts: &[constants_layout],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu.quad.gradient.shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shader/quad.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu.quad.gradient.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "gradient_vs_main",
                    buffers: &[
                        quad::Vertex::buffer_layout(),
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Gradient>()
                                as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array!(
                                // Colors 1-2
                                1 => Uint32x4,
                                // Colors 3-4
                                2 => Uint32x4,
                                // Colors 5-6
                                3 => Uint32x4,
                                // Colors 7-8
                                4 => Uint32x4,
                                // Offsets 1-8
                                5 => Uint32x4,
                                // Direction
                                6 => Float32x4,
                                // Position & Scale
                                7 => Float32x4,
                                // Border color
                                8 => Float32x4,
                                // Border radius
                                9 => Float32x4,
                                // Border width
                                10 => Float32
                            ),
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "gradient_fs_main",
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
        #[cfg(feature = "tracing")]
        let _ = tracing::info_span!("Wgpu::Quad::Gradient", "DRAW").entered();

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
