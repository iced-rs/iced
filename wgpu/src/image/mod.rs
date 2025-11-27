pub(crate) mod cache;
pub(crate) use cache::Cache;

mod atlas;

#[cfg(feature = "image")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

use crate::Buffer;
use crate::core::border;
use crate::core::{Rectangle, Size, Transformation};
use crate::graphics::Shell;

use bytemuck::{Pod, Zeroable};

use std::mem;
use std::sync::Arc;

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
                            // Center
                            0 => Float32x2,
                            // Clip bounds
                            1 => Float32x4,
                            // Border radius
                            2 => Float32x4,
                            // Tile
                            3 => Float32x4,
                            // Rotation
                            4 => Float32,
                            // Opacity
                            5 => Float32,
                            // Atlas position
                            6 => Float32x2,
                            // Atlas scale
                            7 => Float32x2,
                            // Layer
                            8 => Sint32,
                            // Snap
                            9 => Uint32,
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
    nearest_instances: Vec<Instance>,
    linear_instances: Vec<Instance>,
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

        let mut atlas: Option<Arc<wgpu::BindGroup>> = None;

        for image in images {
            match &image {
                #[cfg(feature = "image")]
                Image::Raster {
                    image,
                    bounds,
                    clip_bounds,
                } => {
                    if let Some((atlas_entry, bind_group)) = cache
                        .upload_raster(device, encoder, belt, &image.handle)
                    {
                        match atlas.as_mut() {
                            None => {
                                atlas = Some(bind_group.clone());
                            }
                            Some(atlas) if atlas != bind_group => {
                                layer.push(
                                    atlas,
                                    &self.nearest_instances,
                                    &self.linear_instances,
                                );

                                *atlas = Arc::clone(bind_group);
                            }
                            _ => {}
                        }

                        add_instances(
                            *bounds,
                            *clip_bounds,
                            image.border_radius,
                            f32::from(image.rotation),
                            image.opacity,
                            image.snap,
                            atlas_entry,
                            match image.filter_method {
                                crate::core::image::FilterMethod::Nearest => {
                                    &mut self.nearest_instances
                                }
                                crate::core::image::FilterMethod::Linear => {
                                    &mut self.linear_instances
                                }
                            },
                        );
                    }
                }
                #[cfg(not(feature = "image"))]
                Image::Raster { .. } => continue,

                #[cfg(feature = "svg")]
                Image::Vector {
                    svg,
                    bounds,
                    clip_bounds,
                } => {
                    if let Some((atlas_entry, bind_group)) = cache
                        .upload_vector(
                            device,
                            encoder,
                            belt,
                            &svg.handle,
                            svg.color,
                            bounds.size(),
                            scale,
                        )
                    {
                        match atlas.as_mut() {
                            None => {
                                atlas = Some(bind_group.clone());
                            }
                            Some(atlas) if atlas != bind_group => {
                                layer.push(
                                    atlas,
                                    &self.nearest_instances,
                                    &self.linear_instances,
                                );

                                *atlas = bind_group.clone();
                            }
                            _ => {}
                        }

                        add_instances(
                            *bounds,
                            *clip_bounds,
                            border::radius(0),
                            f32::from(svg.rotation),
                            svg.opacity,
                            true,
                            atlas_entry,
                            &mut self.nearest_instances,
                        );
                    }
                }
                #[cfg(not(feature = "svg"))]
                Image::Vector { .. } => continue,
            }
        }

        if let Some(atlas) = &atlas {
            layer.push(atlas, &self.nearest_instances, &self.linear_instances);
        }

        layer.prepare(
            device,
            encoder,
            belt,
            transformation,
            scale,
            &self.nearest_instances,
            &self.linear_instances,
        );

        self.prepare_layer += 1;
        self.nearest_instances.clear();
        self.linear_instances.clear();
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
    nearest: Vec<Group>,
    nearest_layout: wgpu::BindGroup,
    nearest_total: usize,
    linear: Vec<Group>,
    linear_layout: wgpu::BindGroup,
    linear_total: usize,
}

#[derive(Debug)]
struct Group {
    atlas: Arc<wgpu::BindGroup>,
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
            nearest: Vec::new(),
            nearest_layout,
            nearest_total: 0,
            linear: Vec::new(),
            linear_layout,
            linear_total: 0,
        }
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        transformation: Transformation,
        scale_factor: f32,
        nearest: &[Instance],
        linear: &[Instance],
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

        let _ = self
            .instances
            .resize(device, self.nearest_total + self.linear_total);

        let mut offset = 0;

        if !nearest.is_empty() {
            offset += self.instances.write(device, encoder, belt, 0, nearest);
        }

        if !linear.is_empty() {
            let _ = self.instances.write(device, encoder, belt, offset, linear);
        }
    }

    fn push(
        &mut self,
        atlas: &Arc<wgpu::BindGroup>,
        nearest: &[Instance],
        linear: &[Instance],
    ) {
        let new_nearest = nearest.len() - self.nearest_total;

        if new_nearest > 0 {
            self.nearest.push(Group {
                atlas: atlas.clone(),
                instance_count: new_nearest,
            });

            self.nearest_total = nearest.len();
        }

        let new_linear = linear.len() - self.linear_total;

        if new_linear > 0 {
            self.linear.push(Group {
                atlas: atlas.clone(),
                instance_count: new_linear,
            });

            self.linear_total = linear.len();
        }
    }

    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.instances.slice(..));

        let mut offset = 0;

        if !self.nearest.is_empty() {
            render_pass.set_bind_group(0, &self.nearest_layout, &[]);

            for group in &self.nearest {
                render_pass.set_bind_group(1, group.atlas.as_ref(), &[]);
                render_pass
                    .draw(0..6, offset..offset + group.instance_count as u32);

                offset += group.instance_count as u32;
            }
        }

        if !self.linear.is_empty() {
            render_pass.set_bind_group(0, &self.linear_layout, &[]);

            for group in &self.linear {
                render_pass.set_bind_group(1, group.atlas.as_ref(), &[]);
                render_pass
                    .draw(0..6, offset..offset + group.instance_count as u32);

                offset += group.instance_count as u32;
            }
        }
    }

    fn clear(&mut self) {
        self.nearest.clear();
        self.nearest_total = 0;

        self.linear.clear();
        self.linear_total = 0;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Instance {
    _center: [f32; 2],
    _clip_bounds: [f32; 4],
    _border_radius: [f32; 4],
    _tile: [f32; 4],
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
    bounds: Rectangle,
    clip_bounds: Rectangle,
    border_radius: border::Radius,
    rotation: f32,
    opacity: f32,
    snap: bool,
    entry: &atlas::Entry,
    instances: &mut Vec<Instance>,
) {
    let center = [
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    ];

    let clip_bounds = [
        clip_bounds.x,
        clip_bounds.y,
        clip_bounds.width,
        clip_bounds.height,
    ];

    let border_radius = border_radius.into();

    match entry {
        atlas::Entry::Contiguous(allocation) => {
            add_instance(
                center,
                clip_bounds,
                border_radius,
                [bounds.x, bounds.y, bounds.width, bounds.height],
                rotation,
                opacity,
                snap,
                allocation,
                instances,
            );
        }
        atlas::Entry::Fragmented { fragments, size } => {
            let scaling_x = bounds.width / size.width as f32;
            let scaling_y = bounds.height / size.height as f32;

            for fragment in fragments {
                let allocation = &fragment.allocation;
                let (fragment_x, fragment_y) = fragment.position;

                let Size {
                    width: fragment_width,
                    height: fragment_height,
                } = allocation.size();

                let tile = [
                    bounds.x + fragment_x as f32 * scaling_x,
                    bounds.y + fragment_y as f32 * scaling_y,
                    fragment_width as f32 * scaling_x,
                    fragment_height as f32 * scaling_y,
                ];

                add_instance(
                    center,
                    clip_bounds,
                    border_radius,
                    tile,
                    rotation,
                    opacity,
                    snap,
                    allocation,
                    instances,
                );
            }
        }
    }
}

#[inline]
fn add_instance(
    center: [f32; 2],
    clip_bounds: [f32; 4],
    border_radius: [f32; 4],
    tile: [f32; 4],
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
        _center: center,
        _clip_bounds: clip_bounds,
        _border_radius: border_radius,
        _tile: tile,
        _rotation: rotation,
        _opacity: opacity,
        _position_in_atlas: [
            x as f32 / atlas_size as f32,
            y as f32 / atlas_size as f32,
        ],
        _size_in_atlas: [
            width as f32 / atlas_size as f32,
            height as f32 / atlas_size as f32,
        ],
        _layer: layer as u32,
        _snap: snap as u32,
    };

    instances.push(instance);
}
