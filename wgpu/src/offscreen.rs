use crate::graphics::color;
use std::borrow::Cow;

/// A simple compute pipeline to convert any texture to Rgba8UnormSrgb.
#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    sampler_bind_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
}

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    ndc: [f32; 2],
    uv: [f32; 2],
}

impl Pipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.offscreen.sampler"),
            ..Default::default()
        });

        //sampler in 0
        let sampler_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu.offscreen.blit.sampler_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::NonFiltering,
                    ),
                    count: None,
                }],
            });

        let sampler_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu.offscreen.sampler.bind_group"),
                layout: &sampler_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }],
            });

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu.offscreen.blit.texture_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: false,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu.offscreen.blit.pipeline_layout"),
                bind_group_layouts: &[&sampler_layout, &texture_layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu.offscreen.blit.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shader/blit.wgsl"
                ))),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu.offscreen.blit.pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: if color::GAMMA_CORRECTION {
                            wgpu::TextureFormat::Rgba8UnormSrgb
                        } else {
                            wgpu::TextureFormat::Rgba8Unorm
                        },
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
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
            });

        Self {
            pipeline,
            sampler_bind_group,
            texture_layout,
        }
    }

    pub fn convert(
        &self,
        device: &wgpu::Device,
        frame: &wgpu::TextureView,
        size: wgpu::Extent3d,
        encoder: &mut wgpu::CommandEncoder,
    ) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_wgpu.offscreen.conversion.source_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if color::GAMMA_CORRECTION {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view =
            &texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu.offscreen.blit.texture_bind_group"),
                layout: &self.texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(frame),
                }],
            });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_wgpu.offscreen.blit.render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.sampler_bind_group, &[]);
        pass.set_bind_group(1, &texture_bind_group, &[]);
        pass.draw(0..6, 0..1);

        texture
    }
}
