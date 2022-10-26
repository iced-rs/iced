mod atlas;

#[cfg(feature = "image_rs")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

use crate::Transformation;
use atlas::Atlas;

use iced_graphics::layer;
use iced_native::Rectangle;
use std::cell::RefCell;
use std::mem;

use bytemuck::{Pod, Zeroable};

#[cfg(feature = "image_rs")]
use iced_native::image;

#[cfg(feature = "svg")]
use iced_native::svg;

#[derive(Debug)]
pub struct Pipeline {
    #[cfg(feature = "image_rs")]
    raster_cache: RefCell<raster::Cache>,
    #[cfg(feature = "svg")]
    vector_cache: RefCell<vector::Cache>,

    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
    constants: wgpu::BindGroup,
    texture: wgpu::BindGroup,
    texture_version: usize,
    texture_layout: wgpu::BindGroupLayout,
    texture_atlas: Atlas,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        use wgpu::util::DeviceExt;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::image constants layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<Uniforms>() as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::image uniforms buffer"),
            size: mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let constant_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::image constants bind group"),
                layout: &constant_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &uniforms_buffer,
                                offset: 0,
                                size: None,
                            },
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::image texture atlas layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::image pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&constant_layout, &texture_layout],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu::image::shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader/image.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::image pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: mem::size_of::<Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: mem::size_of::<Instance>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array!(
                                1 => Float32x2,
                                2 => Float32x2,
                                3 => Float32x2,
                                4 => Float32x2,
                                5 => Sint32,
                            ),
                        },
                    ],
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
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::image vertex buffer"),
                contents: bytemuck::cast_slice(&QUAD_VERTS),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::image index buffer"),
                contents: bytemuck::cast_slice(&QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::image instance buffer"),
            size: mem::size_of::<Instance>() as u64 * Instance::MAX as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_atlas = Atlas::new(device);

        let texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::image texture atlas bind group"),
            layout: &texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    texture_atlas.view(),
                ),
            }],
        });

        Pipeline {
            #[cfg(feature = "image_rs")]
            raster_cache: RefCell::new(raster::Cache::new()),

            #[cfg(feature = "svg")]
            vector_cache: RefCell::new(vector::Cache::new()),

            pipeline,
            uniforms: uniforms_buffer,
            vertices,
            indices,
            instances,
            constants: constant_bind_group,
            texture,
            texture_version: texture_atlas.layer_count(),
            texture_layout,
            texture_atlas,
        }
    }

    #[cfg(feature = "image_rs")]
    pub fn dimensions(&self, handle: &image::Handle) -> (u32, u32) {
        let mut cache = self.raster_cache.borrow_mut();
        let memory = cache.load(handle);

        memory.dimensions()
    }

    #[cfg(feature = "svg")]
    pub fn viewport_dimensions(&self, handle: &svg::Handle) -> (u32, u32) {
        let mut cache = self.vector_cache.borrow_mut();
        let svg = cache.load(handle);

        svg.viewport_dimensions()
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        images: &[layer::Image],
        transformation: Transformation,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        _scale: f32,
    ) {
        let instances: &mut Vec<Instance> = &mut Vec::new();

        #[cfg(feature = "image_rs")]
        let mut raster_cache = self.raster_cache.borrow_mut();

        #[cfg(feature = "svg")]
        let mut vector_cache = self.vector_cache.borrow_mut();

        for image in images {
            match &image {
                #[cfg(feature = "image_rs")]
                layer::Image::Raster { handle, bounds } => {
                    if let Some(atlas_entry) = raster_cache.upload(
                        handle,
                        device,
                        encoder,
                        &mut self.texture_atlas,
                    ) {
                        add_instances(
                            [bounds.x, bounds.y],
                            [bounds.width, bounds.height],
                            atlas_entry,
                            instances,
                        );
                    }
                }
                #[cfg(not(feature = "image_rs"))]
                layer::Image::Raster { .. } => {}

                #[cfg(feature = "svg")]
                layer::Image::Vector { handle, bounds } => {
                    let size = [bounds.width, bounds.height];

                    if let Some(atlas_entry) = vector_cache.upload(
                        handle,
                        size,
                        _scale,
                        device,
                        encoder,
                        &mut self.texture_atlas,
                    ) {
                        add_instances(
                            [bounds.x, bounds.y],
                            size,
                            atlas_entry,
                            instances,
                        );
                    }
                }
                #[cfg(not(feature = "svg"))]
                layer::Image::Vector { .. } => {}
            }
        }

        if instances.is_empty() {
            return;
        }

        let texture_version = self.texture_atlas.layer_count();

        if self.texture_version != texture_version {
            log::info!("Atlas has grown. Recreating bind group...");

            self.texture =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wgpu::image texture atlas bind group"),
                    layout: &self.texture_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            self.texture_atlas.view(),
                        ),
                    }],
                });

            self.texture_version = texture_version;
        }

        {
            let mut uniforms_buffer = staging_belt.write_buffer(
                encoder,
                &self.uniforms,
                0,
                wgpu::BufferSize::new(mem::size_of::<Uniforms>() as u64)
                    .unwrap(),
                device,
            );

            uniforms_buffer.copy_from_slice(bytemuck::bytes_of(&Uniforms {
                transform: transformation.into(),
            }));
        }

        let mut i = 0;
        let total = instances.len();

        while i < total {
            let end = (i + Instance::MAX).min(total);
            let amount = end - i;

            let mut instances_buffer = staging_belt.write_buffer(
                encoder,
                &self.instances,
                0,
                wgpu::BufferSize::new(
                    (amount * std::mem::size_of::<Instance>()) as u64,
                )
                .unwrap(),
                device,
            );

            instances_buffer.copy_from_slice(bytemuck::cast_slice(
                &instances[i..i + amount],
            ));

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu::image render pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: target,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.constants, &[]);
            render_pass.set_bind_group(1, &self.texture, &[]);
            render_pass.set_index_buffer(
                self.indices.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));
            render_pass.set_vertex_buffer(1, self.instances.slice(..));

            render_pass.set_scissor_rect(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );

            render_pass.draw_indexed(
                0..QUAD_INDICES.len() as u32,
                0,
                0..amount as u32,
            );

            i += Instance::MAX;
        }
    }

    pub fn trim_cache(&mut self) {
        #[cfg(feature = "image_rs")]
        self.raster_cache.borrow_mut().trim(&mut self.texture_atlas);

        #[cfg(feature = "svg")]
        self.vector_cache.borrow_mut().trim(&mut self.texture_atlas);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct Vertex {
    _position: [f32; 2],
}

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        _position: [0.0, 0.0],
    },
    Vertex {
        _position: [1.0, 0.0],
    },
    Vertex {
        _position: [1.0, 1.0],
    },
    Vertex {
        _position: [0.0, 1.0],
    },
];

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Instance {
    _position: [f32; 2],
    _size: [f32; 2],
    _position_in_atlas: [f32; 2],
    _size_in_atlas: [f32; 2],
    _layer: u32,
}

impl Instance {
    pub const MAX: usize = 1_000;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    transform: [f32; 16],
}

fn add_instances(
    image_position: [f32; 2],
    image_size: [f32; 2],
    entry: &atlas::Entry,
    instances: &mut Vec<Instance>,
) {
    match entry {
        atlas::Entry::Contiguous(allocation) => {
            add_instance(image_position, image_size, allocation, instances);
        }
        atlas::Entry::Fragmented { fragments, size } => {
            let scaling_x = image_size[0] / size.0 as f32;
            let scaling_y = image_size[1] / size.1 as f32;

            for fragment in fragments {
                let allocation = &fragment.allocation;

                let [x, y] = image_position;
                let (fragment_x, fragment_y) = fragment.position;
                let (fragment_width, fragment_height) = allocation.size();

                let position = [
                    x + fragment_x as f32 * scaling_x,
                    y + fragment_y as f32 * scaling_y,
                ];

                let size = [
                    fragment_width as f32 * scaling_x,
                    fragment_height as f32 * scaling_y,
                ];

                add_instance(position, size, allocation, instances);
            }
        }
    }
}

#[inline]
fn add_instance(
    position: [f32; 2],
    size: [f32; 2],
    allocation: &atlas::Allocation,
    instances: &mut Vec<Instance>,
) {
    let (x, y) = allocation.position();
    let (width, height) = allocation.size();
    let layer = allocation.layer();

    let instance = Instance {
        _position: position,
        _size: size,
        _position_in_atlas: [
            x as f32 / atlas::SIZE as f32,
            y as f32 / atlas::SIZE as f32,
        ],
        _size_in_atlas: [
            width as f32 / atlas::SIZE as f32,
            height as f32 / atlas::SIZE as f32,
        ],
        _layer: layer as u32,
    };

    instances.push(instance);
}
