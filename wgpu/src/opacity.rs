//! Composite an isolated opacity group into a target at a given opacity.
//!
//! An opacity group is first rendered to its own offscreen texture and then
//! blended as a whole with this pipeline, so overlapping primitives fade
//! together instead of each one independently.
use std::borrow::Cow;

use wgpu::util::DeviceExt;

#[derive(Debug, Clone)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    constants_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.opacity.sampler"),
            ..wgpu::SamplerDescriptor::default()
        });

        let constants_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_wgpu.opacity.constants_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_wgpu.opacity.texture_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_wgpu.opacity.pipeline_layout"),
            bind_group_layouts: &[Some(&constants_layout), Some(&texture_layout)],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.opacity.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/opacity.wgsl"))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.opacity.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            sampler,
            constants_layout,
            texture_layout,
        }
    }

    /// Composites `source` into `target` at the given `opacity`, optionally
    /// scissored to `scissor` (physical pixels: `x, y, width, height`).
    ///
    /// `source` must be a premultiplied-alpha texture that matches the target's
    /// dimensions and projection (the group is rendered full-viewport).
    #[allow(clippy::too_many_arguments)]
    pub fn composite(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        source: &wgpu::TextureView,
        opacity: f32,
        scissor: Option<(u32, u32, u32, u32)>,
    ) {
        #[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
        #[repr(C)]
        struct Opacity {
            opacity: f32,
            _padding: [f32; 3],
        }

        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("iced_wgpu.opacity.uniform"),
            contents: bytemuck::bytes_of(&Opacity {
                opacity,
                _padding: [0.0; 3],
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu.opacity.constants"),
            layout: &self.constants_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });

        let texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu.opacity.texture"),
            layout: &self.texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            }],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_wgpu.opacity.composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &constants, &[]);
        pass.set_bind_group(1, &texture, &[]);

        if let Some((x, y, width, height)) = scissor {
            pass.set_scissor_rect(x, y, width, height);
        }

        pass.draw(0..6, 0..1);
    }
}
