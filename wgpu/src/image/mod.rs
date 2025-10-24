pub(crate) mod cache;
pub(crate) use cache::Cache;

mod atlas;

#[cfg(feature = "image")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

use crate::Buffer;
use crate::core::{Rectangle, Size, Transformation};
use crate::graphics::Shell;

use bytemuck::{Pod, Zeroable};

use std::mem;

pub use crate::graphics::Image;

pub type Batch = Vec<Image>;

#[derive(Debug, Clone)]
pub struct Pipeline {
    raw: wgpu::RenderPipeline,
    backend: wgpu::Backend,
    nearest_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
    texture_layout: wgpu::BindGroupLayout,
    constant_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        backend: wgpu::Backend,
    ) -> Self {
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
                        include_str!("../shader/vertex.wgsl"),
                        "\n",
                        include_str!("../shader/image.wgsl"),
                    ),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::image pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array!(
                            // Position
                            0 => Float32x2,
                            // Center
                            1 => Float32x2,
                            // Scale
                            2 => Float32x2,
                            // Rotation
                            3 => Float32,
                            // Opacity
                            4 => Float32,
                            // Atlas position
                            5 => Float32x2,
                            // Atlas scale
                            6 => Float32x2,
                            // Layer
                            7 => Sint32,
                            // Snap
                            8 => Uint32,
                        ),
                    }],
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
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

        Pipeline {
            raw: pipeline,
            backend,
            nearest_sampler,
            linear_sampler,
            texture_layout,
            constant_layout,
        }
    }

    pub fn create_cache(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shell: &Shell,
    ) -> Cache {
        Cache::new(
            device,
            queue,
            self.backend,
            self.texture_layout.clone(),
            shell,
        )
    }
}

#[derive(Default)]
pub struct State {
    layers: Vec<Layer>,
    prepare_layer: usize,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prepare(
        &mut self,
        pipeline: &Pipeline,
        device: &wgpu::Device,
        belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        cache: &mut Cache,
        images: &Batch,
        transformation: Transformation,
        scale: f32,
    ) {
        if self.layers.len() <= self.prepare_layer {
            self.layers.push(Layer::new(
                device,
                &pipeline.constant_layout,
                &pipeline.nearest_sampler,
                &pipeline.linear_sampler,
            ));
        }

        let layer = &mut self.layers[self.prepare_layer];
        layer.prepare(device, encoder, belt, transformation, scale);

        let mut atlas = None;
        let nearest_instances = &mut Vec::new();
        let linear_instances = &mut Vec::new();

        for image in images {
            match &image {
                #[cfg(feature = "image")]
                Image::Raster(image, bounds) => {
                    if let Some((atlas_entry, bind_group)) = cache
                        .upload_raster(device, encoder, belt, &image.handle)
                    {
                        match atlas.as_mut() {
                            None => {
                                atlas = Some(bind_group.clone());
                            }
                            Some(atlas) if atlas != bind_group => {
                                layer.push(
                                    device,
                                    encoder,
                                    belt,
                                    atlas,
                                    nearest_instances,
                                    linear_instances,
                                );

                                *atlas = bind_group.clone();
                                nearest_instances.clear();
                                linear_instances.clear();
                            }
                            _ => {}
                        }

                        add_instances(
                            [bounds.x, bounds.y],
                            [bounds.width, bounds.height],
                            f32::from(image.rotation),
                            image.opacity,
                            image.snap,
                            atlas_entry,
                            match image.filter_method {
                                crate::core::image::FilterMethod::Nearest => {
                                    nearest_instances
                                }
                                crate::core::image::FilterMethod::Linear => {
                                    linear_instances
                                }
                            },
                        );
                    }
                }
                #[cfg(not(feature = "image"))]
                Image::Raster { .. } => continue,

                #[cfg(feature = "svg")]
                Image::Vector(svg, bounds) => {
                    let size = [bounds.width, bounds.height];

                    if let Some((atlas_entry, bind_group)) = cache
                        .upload_vector(
                            device,
                            encoder,
                            belt,
                            &svg.handle,
                            svg.color,
                            size,
                            scale,
                        )
                    {
                        match atlas.as_mut() {
                            None => {
                                atlas = Some(bind_group.clone());
                            }
                            Some(atlas) if atlas != bind_group => {
                                layer.push(
                                    device,
                                    encoder,
                                    belt,
                                    atlas,
                                    nearest_instances,
                                    linear_instances,
                                );

                                *atlas = bind_group.clone();
                                nearest_instances.clear();
                                linear_instances.clear();
                            }
                            _ => {}
                        }

                        add_instances(
                            [bounds.x, bounds.y],
                            size,
                            f32::from(svg.rotation),
                            svg.opacity,
                            true,
                            atlas_entry,
                            nearest_instances,
                        );
                    }
                }
                #[cfg(not(feature = "svg"))]
                Image::Vector { .. } => continue,
            }
        }

        if !nearest_instances.is_empty() || !linear_instances.is_empty() {
            layer.push(
                device,
                encoder,
                belt,
                &atlas.expect("atlas should be defined"),
                nearest_instances,
                linear_instances,
            );
        }

        self.prepare_layer += 1;
    }

    pub fn render<'a>(
        &'a self,
        pipeline: &'a Pipeline,
        layer: usize,
        bounds: Rectangle<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if let Some(layer) = self.layers.get(layer) {
            render_pass.set_pipeline(&pipeline.raw);

            render_pass.set_scissor_rect(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );

            layer.render(render_pass);
        }
    }

    pub fn trim(&mut self) {
        for layer in &mut self.layers[..self.prepare_layer] {
            layer.clear();
        }

        self.prepare_layer = 0;
    }
}

#[derive(Debug)]
struct Layer {
    uniforms: wgpu::Buffer,
    instances: Buffer<Instance>,
    total: usize,
    nearest: Vec<Group>,
    nearest_layout: wgpu::BindGroup,
    linear: Vec<Group>,
    linear_layout: wgpu::BindGroup,
}

#[derive(Debug)]
struct Group {
    atlas: wgpu::BindGroup,
    instance_count: usize,
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

        let instances = Buffer::new(
            device,
            "iced_wgpu::image instance buffer",
            Instance::INITIAL,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        let nearest_layout =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::image constants bind group"),
                layout: constant_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &uniforms,
                                offset: 0,
                                size: None,
                            },
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            nearest_sampler,
                        ),
                    },
                ],
            });

        let linear_layout =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::image constants bind group"),
                layout: constant_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &uniforms,
                                offset: 0,
                                size: None,
                            },
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            linear_sampler,
                        ),
                    },
                ],
            });

        Self {
            uniforms,
            instances,
            total: 0,
            nearest: Vec::new(),
            nearest_layout,
            linear: Vec::new(),
            linear_layout,
        }
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        transformation: Transformation,
        scale_factor: f32,
    ) {
        let uniforms = Uniforms {
            transform: transformation.into(),
            scale_factor,
            _padding: [0.0; 3],
        };

        let bytes = bytemuck::bytes_of(&uniforms);

        belt.write_buffer(
            encoder,
            &self.uniforms,
            0,
            (bytes.len() as u64).try_into().expect("Sized uniforms"),
            device,
        )
        .copy_from_slice(bytes);
    }

    fn push(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        atlas: &wgpu::BindGroup,
        nearest: &[Instance],
        linear: &[Instance],
    ) {
        let new = nearest.len() + linear.len();

        let _ = self.instances.resize(device, self.total + new);

        if !nearest.is_empty() {
            self.total += self
                .instances
                .write(device, encoder, belt, self.total, nearest);

            self.nearest.push(Group {
                atlas: atlas.clone(),
                instance_count: nearest.len(),
            });
        }

        if !linear.is_empty() {
            self.total += self
                .instances
                .write(device, encoder, belt, self.total, linear);

            self.linear.push(Group {
                atlas: atlas.clone(),
                instance_count: linear.len(),
            });
        }
    }

    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.instances.slice(..));

        let mut offset = 0;

        if !self.nearest.is_empty() {
            render_pass.set_bind_group(0, &self.nearest_layout, &[]);

            for group in &self.nearest {
                render_pass.set_bind_group(1, &group.atlas, &[]);
                render_pass
                    .draw(0..6, offset..offset + group.instance_count as u32);

                offset += group.instance_count as u32;
            }
        }

        if !self.linear.is_empty() {
            render_pass.set_bind_group(0, &self.linear_layout, &[]);

            for group in &self.linear {
                render_pass.set_bind_group(1, &group.atlas, &[]);
                render_pass
                    .draw(0..6, offset..offset + group.instance_count as u32);

                offset += group.instance_count as u32;
            }
        }
    }

    fn clear(&mut self) {
        self.instances.clear();
        self.nearest.clear();
        self.linear.clear();
        self.total = 0;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Instance {
    _position: [f32; 2],
    _center: [f32; 2],
    _size: [f32; 2],
    _rotation: f32,
    _opacity: f32,
    _position_in_atlas: [f32; 2],
    _size_in_atlas: [f32; 2],
    _layer: u32,
    _snap: u32,
}

impl Instance {
    pub const INITIAL: usize = 20;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    transform: [f32; 16],
    scale_factor: f32,
    // Uniforms must be aligned to their largest member,
    // this uses a mat4x4<f32> which aligns to 16, so align to that
    _padding: [f32; 3],
}

fn add_instances(
    image_position: [f32; 2],
    image_size: [f32; 2],
    rotation: f32,
    opacity: f32,
    snap: bool,
    entry: &atlas::Entry,
    instances: &mut Vec<Instance>,
) {
    let center = [
        image_position[0] + image_size[0] / 2.0,
        image_position[1] + image_size[1] / 2.0,
    ];

    match entry {
        atlas::Entry::Contiguous(allocation) => {
            add_instance(
                image_position,
                center,
                image_size,
                rotation,
                opacity,
                snap,
                allocation,
                instances,
            );
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

                add_instance(
                    position, center, size, rotation, opacity, snap,
                    allocation, instances,
                );
            }
        }
    }
}

#[inline]
fn add_instance(
    position: [f32; 2],
    center: [f32; 2],
    size: [f32; 2],
    rotation: f32,
    opacity: f32,
    snap: bool,
    allocation: &atlas::Allocation,
    instances: &mut Vec<Instance>,
) {
    let (x, y) = allocation.position();
    let Size { width, height } = allocation.size();
    let layer = allocation.layer();
    let atlas_size = allocation.atlas_size();

    let instance = Instance {
        _position: position,
        _center: center,
        _size: size,
        _rotation: rotation,
        _opacity: opacity,
        _position_in_atlas: [
            (x as f32 + 0.5) / atlas_size as f32,
            (y as f32 + 0.5) / atlas_size as f32,
        ],
        _size_in_atlas: [
            (width as f32 - 1.0) / atlas_size as f32,
            (height as f32 - 1.0) / atlas_size as f32,
        ],
        _layer: layer as u32,
        _snap: snap as u32,
    };

    instances.push(instance);
}
