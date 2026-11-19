use crate::Buffer;
use crate::core::{self, Rectangle};
use crate::gradient;
use crate::quad::{self, Quad};

use bytemuck::{Pod, Zeroable};
use std::ops::Range;

/// A quad filled with interpolated colors.
#[derive(Clone, Copy, Debug)]
pub struct Gradient {
    /// The background gradient of the quad.
    pub gradient: core::Gradient,

    /// The [`Quad`] data of the [`Gradient`].
    pub quad: Quad,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Instance {
    direction: [f32; 4],
    stops: gradient::Stops,
    quad: Quad,
}

#[derive(Debug)]
pub struct Layer {
    instances: Buffer<Instance>,
    instance_count: usize,
    stops: gradient::Storage,
    staging: Vec<Instance>,
}

impl Layer {
    pub fn new(device: &wgpu::Device, stops_layout: &wgpu::BindGroupLayout) -> Self {
        let instances = Buffer::new(
            device,
            "iced_wgpu.quad.gradient.buffer",
            quad::INITIAL_INSTANCES,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            instances,
            instance_count: 0,
            stops: gradient::Storage::new(device, stops_layout),
            staging: Vec::new(),
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        gradients: &[Gradient],
    ) {
        self.stops.clear();
        self.staging.clear();

        for Gradient { gradient, quad } in gradients {
            let instance = match gradient {
                core::Gradient::Linear(linear) => {
                    let bounds = Rectangle::new(quad.position.into(), quad.size.into());
                    let (start, end) = bounds.chord(linear.angle);

                    Instance {
                        direction: [start.x, start.y, end.x, end.y],
                        stops: self.stops.push(&linear.stops),
                        quad: *quad,
                    }
                }
            };

            // this is stored as a field so we don't have to
            // reallocate over and over again
            self.staging.push(instance);
        }

        self.stops.upload(device, encoder, belt);

        let _ = self.instances.resize(device, self.staging.len());
        let _ = self.instances.write(encoder, belt, 0, &self.staging);

        self.instance_count = self.staging.len();
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    pub(crate) stops_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        constants_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let stops_layout = gradient::bind_group_layout(device);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_wgpu.quad.gradient.pipeline"),
            bind_group_layouts: &[Some(constants_layout), Some(&stops_layout)],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.quad.gradient.shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(concat!(
                include_str!("../shader/quad/snap.wgsl"),
                "\n",
                include_str!("../shader/quad.wgsl"),
                "\n",
                include_str!("../shader/vertex.wgsl"),
                "\n",
                include_str!("../shader/quad/shadow.wgsl"),
                "\n",
                include_str!("../shader/quad/gradient.wgsl"),
                "\n",
                include_str!("../shader/gradient.wgsl"),
                "\n",
                include_str!("../shader/color.wgsl"),
                "\n",
                include_str!("../shader/color/linear_rgb.wgsl")
            ))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.quad.gradient.pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("gradient_vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array!(
                        // Direction
                        0 => Float32x4,
                        // Stops
                        1 => Uint32x2,
                        // Position
                        2 => Float32x2,
                        // Scale
                        3 => Float32x2,
                        // Border color
                        4 => Float32x4,
                        // Border radius
                        5 => Float32x4,
                        // Border width
                        6 => Float32,
                        // Shadow color
                        7 => Float32x4,
                        // Shadow offset
                        8 => Float32x2,
                        // Shadow blur radius
                        9 => Float32,
                        // Snap
                        10 => Uint32,
                    ),
                })],
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

        Self {
            pipeline,
            stops_layout,
        }
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
        render_pass.set_bind_group(1, layer.stops.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, layer.instances.slice(..));

        render_pass.draw(0..6, range.start as u32..range.end as u32);
    }
}
