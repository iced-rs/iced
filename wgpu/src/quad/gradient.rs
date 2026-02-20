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
        let _ = self.instances.write(encoder, belt, 0, instances);

        self.instance_count = instances.len();
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        constants_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_wgpu.quad.gradient.pipeline"),
            bind_group_layouts: &[constants_layout],
            immediate_size: 0,
        });

        // Use WebGL-compatible shader on WASM (fewer inter-stage varyings)
        #[cfg(target_arch = "wasm32")]
        let gradient_shader = include_str!("../shader/quad/gradient_webgl.wgsl");

        #[cfg(not(target_arch = "wasm32"))]
        let gradient_shader = include_str!("../shader/quad/gradient.wgsl");

        // Create the shader with the right gradient file
        let shader_source = format!(
            "{}\n{}\n{}\n{}\n{}",
            include_str!("../shader/quad.wgsl"),
            include_str!("../shader/vertex.wgsl"),
            gradient_shader,
            include_str!("../shader/color.wgsl"),
            include_str!("../shader/color/linear_rgb.wgsl"),
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.quad.gradient.shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader_source)),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.quad.gradient.pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("gradient_vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Gradient>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    // WebGL2 has stricter attribute limits, so we use a reduced layout
                    // that skips colors_3, colors_4, and _padding (WebGL only uses 4 stops anyway).
                    // We use explicit offsets to read from correct buffer positions.
                    #[cfg(target_arch = "wasm32")]
                    attributes: &[
                        // Colors 1-2 (offset 0)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        // Colors 3-4 (offset 16)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        // Skip colors_3 @ 32, colors_4 @ 48
                        // Offsets 1-8 (offset 64)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 64,
                            shader_location: 2,
                        },
                        // Direction (offset 80)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 80,
                            shader_location: 3,
                        },
                        // Gradient type (offset 96)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 96,
                            shader_location: 4,
                        },
                        // Skip _padding @ 100
                        // Position & Scale (offset 112) - start of Quad
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 112,
                            shader_location: 5,
                        },
                        // Border color (offset 128)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 128,
                            shader_location: 6,
                        },
                        // Border radius (offset 144)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 144,
                            shader_location: 7,
                        },
                        // Border width (offset 160)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 160,
                            shader_location: 8,
                        },
                        // Shadow color (offset 164)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 164,
                            shader_location: 9,
                        },
                        // Shadow offset (offset 180)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 180,
                            shader_location: 10,
                        },
                        // Shadow blur radius (offset 188)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 188,
                            shader_location: 11,
                        },
                        // Shadow inset + snap + border_only (offset 192)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 192,
                            shader_location: 12,
                        },
                    ],
                    // Native has full 8 gradient stops with all fields
                    // Use explicit offsets to match the Gradient struct layout
                    #[cfg(not(target_arch = "wasm32"))]
                    attributes: &[
                        // Colors 1-2 (offset 0)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        // Colors 3-4 (offset 16)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        // Colors 5-6 (offset 32)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                        // Colors 7-8 (offset 48)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 48,
                            shader_location: 3,
                        },
                        // Offsets 1-8 (offset 64)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 64,
                            shader_location: 4,
                        },
                        // Direction (offset 80)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 80,
                            shader_location: 5,
                        },
                        // Gradient type (offset 96)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 96,
                            shader_location: 6,
                        },
                        // Padding (offset 100) - 3 u32s
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x3,
                            offset: 100,
                            shader_location: 7,
                        },
                        // Position & Scale (offset 112) - start of Quad
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 112,
                            shader_location: 8,
                        },
                        // Border color (offset 128)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 128,
                            shader_location: 9,
                        },
                        // Border radius (offset 144)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 144,
                            shader_location: 10,
                        },
                        // Border width (offset 160)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 160,
                            shader_location: 11,
                        },
                        // Shadow color (offset 164)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 164,
                            shader_location: 12,
                        },
                        // Shadow offset (offset 180)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 180,
                            shader_location: 13,
                        },
                        // Shadow blur radius (offset 188)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 188,
                            shader_location: 14,
                        },
                        // Shadow inset + snap + border_only + padding (offset 192)
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32x4,
                            offset: 192,
                            shader_location: 15,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("gradient_fs_main"),
                targets: &quad::color_target_state(format),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            multiview_mask: None,
            cache: None,
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
        render_pass.set_vertex_buffer(0, layer.instances.slice(..));

        render_pass.draw(0..6, range.start as u32..range.end as u32);
    }
}
