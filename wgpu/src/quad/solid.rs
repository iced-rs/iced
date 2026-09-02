use crate::Buffer;
use crate::graphics::color;
use crate::quad::{self, Quad, Uniform as UniformQuad};

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

/// A solid quad with a uniform border.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Uniform {
    /// The background color data of the quad.
    pub color: color::Packed,

    /// The compact [`Quad`] data of the [`Uniform`].
    pub quad: UniformQuad,
}

#[derive(Debug)]
pub struct Layer {
    uniform_instances: Buffer<Uniform>,
    instances: Buffer<Solid>,
}

impl Layer {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform_instances = Buffer::new(
            device,
            "iced_wgpu.quad.solid.uniform.buffer",
            quad::INITIAL_INSTANCES,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );
        let instances = Buffer::new(
            device,
            "iced_wgpu.quad.solid.buffer",
            quad::INITIAL_INSTANCES,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            uniform_instances,
            instances,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        uniform_instances: &[Uniform],
        instances: &[Solid],
    ) {
        if !uniform_instances.is_empty() {
            let _ = self
                .uniform_instances
                .resize(device, uniform_instances.len());
            let _ = self
                .uniform_instances
                .write(encoder, belt, 0, uniform_instances);
        }

        if !instances.is_empty() {
            let _ = self.instances.resize(device, instances.len());
            let _ = self.instances.write(encoder, belt, 0, instances);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    uniform_pipeline: wgpu::RenderPipeline,
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        constants_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_wgpu.quad.solid.pipeline"),
            bind_group_layouts: &[Some(constants_layout)],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.quad.solid.shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(concat!(
                include_str!("../shader/color.wgsl"),
                "\n",
                include_str!("../shader/quad.wgsl"),
                "\n",
                include_str!("../shader/vertex.wgsl"),
                "\n",
                include_str!("../shader/quad/solid.wgsl"),
            ))),
        });

        let uniform_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.quad.solid.uniform.shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(concat!(
                include_str!("../shader/color.wgsl"),
                "\n",
                include_str!("../shader/quad.wgsl"),
                "\n",
                include_str!("../shader/vertex.wgsl"),
                "\n",
                include_str!("../shader/quad/solid_uniform.wgsl"),
            ))),
        });

        let uniform_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.quad.solid.uniform.pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &uniform_shader,
                entry_point: Some("uniform_solid_vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Uniform>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array!(
                        // Color
                        0 => Float32x4,
                        // Position
                        1 => Float32x2,
                        // Size
                        2 => Float32x2,
                        // Border color
                        3 => Float32x4,
                        // Border radius
                        4 => Float32x4,
                        // Border width
                        5 => Float32,
                        // Shadow color
                        6 => Float32x4,
                        // Shadow offset
                        7 => Float32x2,
                        // Shadow blur radius
                        8 => Float32,
                        // Snap
                        9 => Uint32,
                    ),
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &uniform_shader,
                entry_point: Some("uniform_solid_fs_main"),
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

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.quad.solid.pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("solid_vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Solid>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array!(
                        // Color
                        0 => Float32x4,
                        // Position
                        1 => Float32x2,
                        // Size
                        2 => Float32x2,
                        // Border colors
                        3 => Float32x4,
                        4 => Float32x4,
                        5 => Float32x4,
                        6 => Float32x4,
                        // Border radius
                        7 => Float32x4,
                        // Border widths
                        8 => Float32x4,
                        // Shadow color
                        9 => Float32x4,
                        // Shadow offset
                        10 => Float32x2,
                        // Shadow blur radius
                        11 => Float32,
                        // Snap
                        12 => Uint32,
                    ),
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("solid_fs_main"),
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
            uniform_pipeline,
            pipeline,
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
        render_pass.set_vertex_buffer(0, layer.instances.slice(..));

        render_pass.draw(0..6, range.start as u32..range.end as u32);
    }

    pub fn render_uniform<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        constants: &'a wgpu::BindGroup,
        layer: &'a Layer,
        range: Range<usize>,
    ) {
        render_pass.set_pipeline(&self.uniform_pipeline);
        render_pass.set_bind_group(0, constants, &[]);
        render_pass.set_vertex_buffer(0, layer.uniform_instances.slice(..));

        render_pass.draw(0..6, range.start as u32..range.end as u32);
    }
}
