use crate::Buffer;
use crate::graphics::gradient;
use crate::quad::{self, Quad};

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
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        instances: &[Gradient],
    ) {
        let _ = self.instances.resize(device, instances.len());
        let _ = self.instances.write(device, encoder, belt, 0, instances);

        self.instance_count = instances.len();
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    #[cfg(not(target_arch = "wasm32"))]
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    #[allow(unused_variables)]
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        constants_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::graphics::color;

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("iced_wgpu.quad.gradient.pipeline"),
                    push_constant_ranges: &[],
                    bind_group_layouts: &[constants_layout],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("iced_wgpu.quad.gradient.shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(
                            if color::GAMMA_CORRECTION {
                                concat!(
                                    include_str!("../shader/quad.wgsl"),
                                    "\n",
                                    include_str!("../shader/vertex.wgsl"),
                                    "\n",
                                    include_str!(
                                        "../shader/quad/gradient.wgsl"
                                    ),
                                    "\n",
                                    include_str!("../shader/color/oklab.wgsl")
                                )
                            } else {
                                concat!(
                                    include_str!("../shader/quad.wgsl"),
                                    "\n",
                                    include_str!("../shader/vertex.wgsl"),
                                    "\n",
                                    include_str!(
                                        "../shader/quad/gradient.wgsl"
                                    ),
                                    "\n",
                                    include_str!(
                                        "../shader/color/linear_rgb.wgsl"
                                    )
                                )
                            },
                        ),
                    ),
                });

            let pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("iced_wgpu.quad.gradient.pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("gradient_vs_main"),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Gradient>()
                                as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array!(
                                // Colors 1-2
                                0 => Uint32x4,
                                // Colors 3-4
                                1 => Uint32x4,
                                // Colors 5-6
                                2 => Uint32x4,
                                // Colors 7-8
                                3 => Uint32x4,
                                // Offsets 1-8
                                4 => Uint32x4,
                                // Direction
                                5 => Float32x4,
                                // Position & Scale
                                6 => Float32x4,
                                // Border color
                                7 => Float32x4,
                                // Border radius
                                8 => Float32x4,
                                // Border width
                                9 => Float32
                            ),
                        }],
                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("gradient_fs_main"),
                        targets: &quad::color_target_state(format),
                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),
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
                    cache: None,
                },
            );

            Self { pipeline }
        }

        #[cfg(target_arch = "wasm32")]
        Self {}
    }

    #[allow(unused_variables)]
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        constants: &'a wgpu::BindGroup,
        layer: &'a Layer,
        range: Range<usize>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, constants, &[]);
            render_pass.set_vertex_buffer(0, layer.instances.slice(..));

            render_pass.draw(0..6, range.start as u32..range.end as u32);
        }
    }
}
