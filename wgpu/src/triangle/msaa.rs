use crate::core::{Size, Transformation};
use crate::graphics;

use std::num::NonZeroU64;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Pipeline {
    format: wgpu::TextureFormat,
    sampler: wgpu::Sampler,
    raw: wgpu::RenderPipeline,
    constant_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    sample_count: u32,
    targets: Arc<RwLock<Option<Targets>>>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: graphics::Antialiasing,
    ) -> Pipeline {
        let sampler =
            device.create_sampler(&wgpu::SamplerDescriptor::default());

        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle:msaa uniforms layout"),
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
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu triangle blit_shader"),
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
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(
                            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                        ),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
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
            });

        Self {
            format,
            sampler,
            raw: pipeline,
            constant_layout,
            texture_layout,
            sample_count: antialiasing.sample_count(),
            targets: Arc::new(RwLock::new(None)),
        }
    }

    fn targets(
        &self,
        device: &wgpu::Device,
        region_size: Size<u32>,
    ) -> Targets {
        let mut targets = self.targets.write().expect("Write MSAA targets");

        match targets.as_mut() {
            Some(targets)
                if region_size.width <= targets.size.width
                    && region_size.height <= targets.size.height => {}
            _ => {
                *targets = Some(Targets::new(
                    device,
                    self.format,
                    &self.texture_layout,
                    self.sample_count,
                    region_size,
                ));
            }
        }

        targets.as_ref().unwrap().clone()
    }

    pub fn render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        let targets = self.targets.read().expect("Read MSAA targets");
        let targets = targets.as_ref().unwrap();

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_wgpu.triangle.render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &targets.attachment,
                resolve_target: Some(&targets.resolve),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}

#[derive(Debug, Clone)]
struct Targets {
    attachment: wgpu::TextureView,
    resolve: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    size: Size<u32>,
}

impl Targets {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        texture_layout: &wgpu::BindGroupLayout,
        sample_count: u32,
        size: Size<u32>,
    ) -> Targets {
        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
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
            view_formats: &[],
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
            view_formats: &[],
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
            size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Ratio {
    u: f32,
    v: f32,
    // Padding field for 16-byte alignment.
    // See https://docs.rs/wgpu/latest/wgpu/struct.DownlevelFlags.html#associatedconstant.BUFFER_BINDINGS_NOT_16_BYTE_ALIGNED
    _padding: [f32; 2],
}

pub struct State {
    ratio: wgpu::Buffer,
    constants: wgpu::BindGroup,
    last_ratio: Option<Ratio>,
}

impl State {
    pub fn new(device: &wgpu::Device, pipeline: &Pipeline) -> Self {
        let ratio = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::triangle::msaa ratio"),
            size: std::mem::size_of::<Ratio>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::triangle::msaa uniforms bind group"),
            layout: &pipeline.constant_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ratio.as_entire_binding(),
                },
            ],
        });

        Self {
            ratio,
            constants,
            last_ratio: None,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        pipeline: &Pipeline,
        region_size: Size<u32>,
    ) -> Transformation {
        let targets = pipeline.targets(device, region_size);

        let ratio = Ratio {
            u: region_size.width as f32 / targets.size.width as f32,
            v: region_size.height as f32 / targets.size.height as f32,
            _padding: [0.0; 2],
        };

        if Some(ratio) != self.last_ratio {
            belt.write_buffer(
                encoder,
                &self.ratio,
                0,
                NonZeroU64::new(std::mem::size_of::<Ratio>() as u64)
                    .expect("non-empty ratio"),
                device,
            )
            .copy_from_slice(bytemuck::bytes_of(&ratio));

            self.last_ratio = Some(ratio);
        }

        Transformation::orthographic(targets.size.width, targets.size.height)
    }

    pub fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu::triangle::msaa render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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

        render_pass.set_pipeline(&pipeline.raw);
        render_pass.set_bind_group(0, &self.constants, &[]);
        render_pass.set_bind_group(
            1,
            &pipeline
                .targets
                .read()
                .expect("Read MSAA targets")
                .as_ref()
                .unwrap()
                .bind_group,
            &[],
        );
        render_pass.draw(0..6, 0..1);
    }
}
