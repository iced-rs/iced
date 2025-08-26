use std::borrow::Cow;

use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct ConvertBuffers {
    size: wgpu::Extent3d,
    pipeline: wgpu::RenderPipeline,
    constant_bind_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    texture: wgpu::Texture,
    needs_clear: bool,
}

pub fn create_convert_buffers(name: &str, device: &wgpu::Device, size: wgpu::Extent3d, format: wgpu::TextureFormat) -> ConvertBuffers {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("iced_wgpu.offscreen.sampler.{name}")),
        ..wgpu::SamplerDescriptor::default()
    });

    #[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
    #[repr(C)]
    struct Ratio {
        u: f32,
        v: f32,
        // Padding field for 16-byte alignment.
        // See https://docs.rs/wgpu/latest/wgpu/struct.DownlevelFlags.html#associatedconstant.BUFFER_BINDINGS_NOT_16_BYTE_ALIGNED
        _padding: [f32; 2],
    }

    let ratio = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("iced-wgpu::triangle::msaa ratio.{name}")),
        contents: bytemuck::bytes_of(&Ratio {
            u: 1.0,
            v: 1.0,
            _padding: [0.0; 2],
        }),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    let constant_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("iced_wgpu.offscreen.blit.sampler_layout.{name}")),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::NonFiltering,
                    ),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let constant_bind_group =
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("iced_wgpu.offscreen.sampler.bind_group.{name}")),
            layout: &constant_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ratio.as_entire_binding(),
                },
            ],
        });

    let texture_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("iced_wgpu.offscreen.blit.texture_layout.{name}")),
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
            label: Some(&format!("iced_wgpu.offscreen.blit.pipeline_layout.{name}")),
            bind_group_layouts: &[&constant_layout, &texture_layout],
            push_constant_ranges: &[],
        });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("iced_wgpu.offscreen.blit.shader.{name}")),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "shader/blit.wgsl"
        ))),
    });

    let pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("iced_wgpu.offscreen.blit.pipeline.{name}")),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(
                ),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
                compilation_options: wgpu::PipelineCompilationOptions::default(
                ),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("iced_wgpu.offscreen.conversion.source_texture.{name}")),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    ConvertBuffers {
        size,
        pipeline,
        constant_bind_group,
        texture_layout,
        texture,
        needs_clear: false,
    }
}

pub fn convert_buffered<'a>(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    source: &'a wgpu::Texture,
    format: wgpu::TextureFormat,
    convert_buffers: &'a mut ConvertBuffers,
) -> &'a wgpu::Texture {
    assert_eq!(convert_buffers.size, source.size(), "ConvertBuffers were created for a different size");

    if source.format() == format {
        return source;
    }

    if convert_buffers.needs_clear {
        // if the texture is reused, it must first be cleared; otherwise the conversion is written
        // on top of the old one, not replacing it
        encoder.clear_texture(&convert_buffers.texture, &wgpu::ImageSubresourceRange {
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
    }
    convert_buffers.needs_clear = true;

    let view = &convert_buffers.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let texture_bind_group =
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu.offscreen.blit.texture_bind_group"),
            layout: &convert_buffers.texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &source
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("iced_wgpu.offscreen.blit.render_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
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
    });

    pass.set_pipeline(&convert_buffers.pipeline);
    pass.set_bind_group(0, &convert_buffers.constant_bind_group, &[]);
    pass.set_bind_group(1, &texture_bind_group, &[]);
    pass.draw(0..6, 0..1);

    &convert_buffers.texture
}
