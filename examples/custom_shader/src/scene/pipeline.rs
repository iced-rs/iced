pub mod cube;

mod buffer;
mod uniforms;
mod vertex;

pub use uniforms::Uniforms;

use buffer::Buffer;
use vertex::Vertex;

use crate::wgpu;
use crate::wgpu::util::DeviceExt;

use iced::{Rectangle, Size};

const SKY_TEXTURE_SIZE: u32 = 128;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    cubes: Buffer,
    uniforms: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture_size: Size<u32>,
    depth_view: wgpu::TextureView,
    depth_pipeline: DepthPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Self {
        //vertices of one cube
        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("cubes vertex buffer"),
                contents: bytemuck::cast_slice(&cube::Raw::vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            });

        //cube instance data
        let cubes_buffer = Buffer::new(
            device,
            "cubes instance buffer",
            std::mem::size_of::<cube::Raw>() as u64,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        //uniforms for all cubes
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cubes uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //depth buffer
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cubes depth texture"),
            size: wgpu::Extent3d {
                width: target_size.width,
                height: target_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view =
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let normal_map_data = load_normal_map_data();

        //normal map
        let normal_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("cubes normal map texture"),
                size: wgpu::Extent3d {
                    width: 1024,
                    height: 1024,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &normal_map_data,
        );

        let normal_view =
            normal_texture.create_view(&wgpu::TextureViewDescriptor::default());

        //skybox texture for reflection/refraction
        let skybox_data = load_skybox_data();

        let skybox_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("cubes skybox texture"),
                size: wgpu::Extent3d {
                    width: SKY_TEXTURE_SIZE,
                    height: SKY_TEXTURE_SIZE,
                    depth_or_array_layers: 6, //one for each face of the cube
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &skybox_data,
        );

        let sky_view =
            skybox_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("cubes skybox texture view"),
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });

        let sky_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cubes skybox sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("cubes uniform bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("cubes uniform bind group"),
                layout: &uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniforms.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&sky_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sky_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(
                            &normal_view,
                        ),
                    },
                ],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("cubes pipeline layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("cubes shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shaders/cubes.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("cubes pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), cube::Raw::desc()],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
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
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Max,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        let depth_pipeline = DepthPipeline::new(
            device,
            format,
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        );

        Self {
            pipeline,
            cubes: cubes_buffer,
            uniforms,
            uniform_bind_group,
            vertices,
            depth_texture_size: target_size,
            depth_view,
            depth_pipeline,
        }
    }

    fn update_depth_texture(&mut self, device: &wgpu::Device, size: Size<u32>) {
        if self.depth_texture_size.height != size.height
            || self.depth_texture_size.width != size.width
        {
            let text = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("cubes depth texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            self.depth_view =
                text.create_view(&wgpu::TextureViewDescriptor::default());
            self.depth_texture_size = size;

            self.depth_pipeline.update(device, &text);
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        uniforms: &Uniforms,
        num_cubes: usize,
        cubes: &[cube::Raw],
    ) {
        //recreate depth texture if surface texture size has changed
        self.update_depth_texture(device, target_size);

        // update uniforms
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(uniforms));

        //resize cubes vertex buffer if cubes amount changed
        let new_size = num_cubes * std::mem::size_of::<cube::Raw>();
        self.cubes.resize(device, new_size as u64);

        //always write new cube data since they are constantly rotating
        queue.write_buffer(&self.cubes.raw, 0, bytemuck::cast_slice(cubes));
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: Rectangle<u32>,
        num_cubes: u32,
        show_depth: bool,
    ) {
        {
            let mut pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("cubes.pipeline.pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: target,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        },
                    ),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            pass.set_scissor_rect(
                viewport.x,
                viewport.y,
                viewport.width,
                viewport.height,
            );
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_vertex_buffer(1, self.cubes.raw.slice(..));
            pass.draw(0..36, 0..num_cubes);
        }

        if show_depth {
            self.depth_pipeline.render(encoder, target, viewport);
        }
    }
}

struct DepthPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    depth_view: wgpu::TextureView,
}

impl DepthPipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_texture: wgpu::TextureView,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cubes.depth_pipeline.sampler"),
            ..Default::default()
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("cubes.depth_pipeline.bind_group_layout"),
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
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cubes.depth_pipeline.bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &depth_texture,
                    ),
                },
            ],
        });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("cubes.depth_pipeline.layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("cubes.depth_pipeline.shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shaders/depth.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("cubes.depth_pipeline.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
            sampler,
            depth_view: depth_texture,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        depth_texture: &wgpu::Texture,
    ) {
        self.depth_view =
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("cubes.depth_pipeline.bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.depth_view,
                        ),
                    },
                ],
            });
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        viewport: Rectangle<u32>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("cubes.pipeline.depth_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: None,
                    stencil_ops: None,
                },
            ),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_scissor_rect(
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
        );
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}

fn load_skybox_data() -> Vec<u8> {
    let pos_x: &[u8] = include_bytes!("../../textures/skybox/pos_x.jpg");
    let neg_x: &[u8] = include_bytes!("../../textures/skybox/neg_x.jpg");
    let pos_y: &[u8] = include_bytes!("../../textures/skybox/pos_y.jpg");
    let neg_y: &[u8] = include_bytes!("../../textures/skybox/neg_y.jpg");
    let pos_z: &[u8] = include_bytes!("../../textures/skybox/pos_z.jpg");
    let neg_z: &[u8] = include_bytes!("../../textures/skybox/neg_z.jpg");

    let data: [&[u8]; 6] = [pos_x, neg_x, pos_y, neg_y, pos_z, neg_z];

    data.iter().fold(vec![], |mut acc, bytes| {
        let i = image::load_from_memory_with_format(
            bytes,
            image::ImageFormat::Jpeg,
        )
        .unwrap()
        .to_rgba8()
        .into_raw();

        acc.extend(i);
        acc
    })
}

fn load_normal_map_data() -> Vec<u8> {
    let bytes: &[u8] = include_bytes!("../../textures/ice_cube_normal_map.png");

    image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .unwrap()
        .to_rgba8()
        .into_raw()
}
