mod atlas;

#[cfg(feature = "image")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

use atlas::Atlas;

use crate::core::{Rectangle, Size};
use crate::graphics::Transformation;
use crate::layer;
use crate::Buffer;

use std::cell::RefCell;
use std::mem;

use bytemuck::{Pod, Zeroable};

#[cfg(feature = "image")]
use crate::core::image;

#[cfg(feature = "svg")]
use crate::core::svg;

#[cfg(feature = "tracing")]
use tracing::info_span;

#[derive(Debug)]
pub struct Pipeline {
    #[cfg(feature = "image")]
    raster_cache: RefCell<raster::Cache>,
    #[cfg(feature = "svg")]
    vector_cache: RefCell<vector::Cache>,

    pipeline: wgpu::RenderPipeline,
    nearest_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
    texture: wgpu::BindGroup,
    texture_version: usize,
    texture_atlas: Atlas,
    texture_layout: wgpu::BindGroupLayout,
    constant_layout: wgpu::BindGroupLayout,

    layers: Vec<Layer>,
    prepare_layer: usize,
}

#[derive(Debug)]
struct Layer {
    uniforms: wgpu::Buffer,
    nearest: Data,
    linear: Data,
}

impl Layer {
    fn new(
        device: &wgpu::Device,
        constant_layout: &wgpu::BindGroupLayout,
        nearest_sampler: &wgpu::Sampler,
        linear_sampler: &wgpu::Sampler,
    ) -> Self {
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::image uniforms buffer"),
            size: mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let nearest =
            Data::new(device, constant_layout, nearest_sampler, &uniforms);

        let linear =
            Data::new(device, constant_layout, linear_sampler, &uniforms);

        Self {
            uniforms,
            nearest,
            linear,
        }
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        nearest_instances: &[Instance],
        linear_instances: &[Instance],
        transformation: Transformation,
    ) {
        queue.write_buffer(
            &self.uniforms,
            0,
            bytemuck::bytes_of(&Uniforms {
                transform: transformation.into(),
            }),
        );

        self.nearest.upload(device, queue, nearest_instances);
        self.linear.upload(device, queue, linear_instances);
    }

    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.nearest.render(render_pass);
        self.linear.render(render_pass);
    }
}

#[derive(Debug)]
struct Data {
    constants: wgpu::BindGroup,
    instances: Buffer<Instance>,
    instance_count: usize,
}

impl Data {
    pub fn new(
        device: &wgpu::Device,
        constant_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        uniforms: &wgpu::Buffer,
    ) -> Self {
        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::image constants bind group"),
            layout: constant_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: uniforms,
                            offset: 0,
                            size: None,
                        },
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        let instances = Buffer::new(
            device,
            "iced_wgpu::image instance buffer",
            Instance::INITIAL,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            constants,
            instances,
            instance_count: 0,
        }
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[Instance],
    ) {
        let _ = self.instances.resize(device, instances.len());
        let _ = self.instances.write(queue, 0, instances);

        self.instance_count = instances.len();
    }

    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(0, &self.constants, &[]);
        render_pass.set_vertex_buffer(0, self.instances.slice(..));

        render_pass.draw(0..6, 0..self.instance_count as u32);
    }
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Nearest,
            mag_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
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
                label: Some("iced_wgpu image shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    concat!(
                        include_str!("shader/vertex.wgsl"),
                        "\n",
                        include_str!("shader/image.wgsl"),
                    ),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::image pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array!(
                            // Position
                            0 => Float32x2,
                            // Scale
                            1 => Float32x2,
                            // Atlas position
                            2 => Float32x2,
                            // Atlas scale
                            3 => Float32x2,
                            // Layer
                            4 => Sint32,
                        ),
                    }],
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
            #[cfg(feature = "image")]
            raster_cache: RefCell::new(raster::Cache::default()),

            #[cfg(feature = "svg")]
            vector_cache: RefCell::new(vector::Cache::default()),

            pipeline,
            nearest_sampler,
            linear_sampler,
            texture,
            texture_version: texture_atlas.layer_count(),
            texture_atlas,
            texture_layout,
            constant_layout,

            layers: Vec::new(),
            prepare_layer: 0,
        }
    }

    #[cfg(feature = "image")]
    pub fn dimensions(&self, handle: &image::Handle) -> Size<u32> {
        let mut cache = self.raster_cache.borrow_mut();
        let memory = cache.load(handle);

        memory.dimensions()
    }

    #[cfg(feature = "svg")]
    pub fn viewport_dimensions(&self, handle: &svg::Handle) -> Size<u32> {
        let mut cache = self.vector_cache.borrow_mut();
        let svg = cache.load(handle);

        svg.viewport_dimensions()
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        images: &[layer::Image],
        transformation: Transformation,
        _scale: f32,
    ) {
        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Image", "PREPARE").entered();

        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Image", "DRAW").entered();

        let nearest_instances: &mut Vec<Instance> = &mut Vec::new();
        let linear_instances: &mut Vec<Instance> = &mut Vec::new();

        #[cfg(feature = "image")]
        let mut raster_cache = self.raster_cache.borrow_mut();

        #[cfg(feature = "svg")]
        let mut vector_cache = self.vector_cache.borrow_mut();

        for image in images {
            match &image {
                #[cfg(feature = "image")]
                layer::Image::Raster {
                    handle,
                    filter_method,
                    bounds,
                } => {
                    if let Some(atlas_entry) = raster_cache.upload(
                        device,
                        encoder,
                        handle,
                        &mut self.texture_atlas,
                    ) {
                        add_instances(
                            [bounds.x, bounds.y],
                            [bounds.width, bounds.height],
                            atlas_entry,
                            match filter_method {
                                image::FilterMethod::Nearest => {
                                    nearest_instances
                                }
                                image::FilterMethod::Linear => linear_instances,
                            },
                        );
                    }
                }
                #[cfg(not(feature = "image"))]
                layer::Image::Raster { .. } => {}

                #[cfg(feature = "svg")]
                layer::Image::Vector {
                    handle,
                    color,
                    bounds,
                } => {
                    let size = [bounds.width, bounds.height];

                    if let Some(atlas_entry) = vector_cache.upload(
                        device,
                        encoder,
                        handle,
                        *color,
                        size,
                        _scale,
                        &mut self.texture_atlas,
                    ) {
                        add_instances(
                            [bounds.x, bounds.y],
                            size,
                            atlas_entry,
                            nearest_instances,
                        );
                    }
                }
                #[cfg(not(feature = "svg"))]
                layer::Image::Vector { .. } => {}
            }
        }

        if nearest_instances.is_empty() && linear_instances.is_empty() {
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

        if self.layers.len() <= self.prepare_layer {
            self.layers.push(Layer::new(
                device,
                &self.constant_layout,
                &self.nearest_sampler,
                &self.linear_sampler,
            ));
        }

        let layer = &mut self.layers[self.prepare_layer];

        layer.prepare(
            device,
            queue,
            nearest_instances,
            linear_instances,
            transformation,
        );

        self.prepare_layer += 1;
    }

    pub fn render<'a>(
        &'a self,
        layer: usize,
        bounds: Rectangle<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if let Some(layer) = self.layers.get(layer) {
            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_scissor_rect(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );

            render_pass.set_bind_group(1, &self.texture, &[]);

            layer.render(render_pass);
        }
    }

    pub fn end_frame(&mut self) {
        #[cfg(feature = "image")]
        self.raster_cache.borrow_mut().trim(&mut self.texture_atlas);

        #[cfg(feature = "svg")]
        self.vector_cache.borrow_mut().trim(&mut self.texture_atlas);

        self.prepare_layer = 0;
    }
}

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
    pub const INITIAL: usize = 20;
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
            let scaling_x = image_size[0] / size.width as f32;
            let scaling_y = image_size[1] / size.height as f32;

            for fragment in fragments {
                let allocation = &fragment.allocation;

                let [x, y] = image_position;
                let (fragment_x, fragment_y) = fragment.position;
                let Size {
                    width: fragment_width,
                    height: fragment_height,
                } = allocation.size();

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
    let Size { width, height } = allocation.size();
    let layer = allocation.layer();

    let instance = Instance {
        _position: position,
        _size: size,
        _position_in_atlas: [
            (x as f32 + 0.5) / atlas::SIZE as f32,
            (y as f32 + 0.5) / atlas::SIZE as f32,
        ],
        _size_in_atlas: [
            (width as f32 - 1.0) / atlas::SIZE as f32,
            (height as f32 - 1.0) / atlas::SIZE as f32,
        ],
        _layer: layer as u32,
    };

    instances.push(instance);
}
