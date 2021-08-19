use crate::settings;

#[derive(Debug)]
pub struct Blit {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    sample_count: u32,
    targets: Option<Targets>,
}

impl Blit {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: settings::Antialiasing,
    ) -> Blit {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle:msaa uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                }],
            });

        let constant_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::triangle::msaa uniforms bind group"),
                layout: &constant_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }],
            });

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle::msaa texture layout"),
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

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::triangle::msaa pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&constant_layout, &texture_layout],
            });

        let shader =
            device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu::triangle::blit_shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shader/blit.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::triangle::msaa pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
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
                    }],
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
            });

        Blit {
            format,
            pipeline,
            constants: constant_bind_group,
            texture_layout,
            sample_count: antialiasing.sample_count(),
            targets: None,
        }
    }

    pub fn targets(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (&wgpu::TextureView, &wgpu::TextureView) {
        match &mut self.targets {
            None => {
                self.targets = Some(Targets::new(
                    &device,
                    self.format,
                    &self.texture_layout,
                    self.sample_count,
                    width,
                    height,
                ));
            }
            Some(targets) => {
                if targets.width != width || targets.height != height {
                    self.targets = Some(Targets::new(
                        &device,
                        self.format,
                        &self.texture_layout,
                        self.sample_count,
                        width,
                        height,
                    ));
                }
            }
        }

        let targets = self.targets.as_ref().unwrap();

        (&targets.attachment, &targets.resolve)
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu::triangle::msaa render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.constants, &[]);
        render_pass.set_bind_group(
            1,
            &self.targets.as_ref().unwrap().bind_group,
            &[],
        );
        render_pass.draw(0..6, 0..1);
    }
}

#[derive(Debug)]
struct Targets {
    attachment: wgpu::TextureView,
    resolve: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl Targets {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        texture_layout: &wgpu::BindGroupLayout,
        sample_count: u32,
        width: u32,
        height: u32,
    ) -> Targets {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let attachment = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_wgpu::triangle::msaa attachment"),
            size: extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let resolve = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_wgpu::triangle::msaa resolve target"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let attachment =
            attachment.create_view(&wgpu::TextureViewDescriptor::default());

        let resolve =
            resolve.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::triangle::msaa texture bind group"),
            layout: texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&resolve),
            }],
        });

        Targets {
            attachment,
            resolve,
            bind_group,
            width,
            height,
        }
    }
}
