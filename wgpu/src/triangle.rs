//! Draw meshes of triangles.
mod msaa;

use crate::Buffer;
use crate::core::{Point, Rectangle, Size, Transformation, Vector};
use crate::graphics::Antialiasing;
use crate::graphics::mesh::{self, Mesh};

use rustc_hash::FxHashMap;
use std::collections::hash_map;
use std::sync::atomic::{self, AtomicU64};
use std::sync::{self, Arc};

const INITIAL_INDEX_COUNT: usize = 1_000;
const INITIAL_VERTEX_COUNT: usize = 1_000;

pub type Batch = Vec<Item>;

#[derive(Debug)]
pub enum Item {
    Group {
        transformation: Transformation,
        meshes: Vec<Mesh>,
    },
    Cached {
        transformation: Transformation,
        cache: Cache,
    },
}

#[derive(Debug, Clone)]
pub struct Cache {
    id: Id,
    batch: Arc<[Mesh]>,
    version: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Cache {
    pub fn new(meshes: Vec<Mesh>) -> Option<Self> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        if meshes.is_empty() {
            return None;
        }

        Some(Self {
            id: Id(NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed)),
            batch: Arc::from(meshes),
            version: 0,
        })
    }

    pub fn update(&mut self, meshes: Vec<Mesh>) {
        self.batch = Arc::from(meshes);
        self.version += 1;
    }
}

#[derive(Debug)]
struct Upload {
    layer: Layer,
    transformation: Transformation,
    version: usize,
    batch: sync::Weak<[Mesh]>,
}

#[derive(Debug, Default)]
pub struct Storage {
    uploads: FxHashMap<Id, Upload>,
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, cache: &Cache) -> Option<&Upload> {
        if cache.batch.is_empty() {
            return None;
        }

        self.uploads.get(&cache.id)
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        solid: &solid::Pipeline,
        gradient: &gradient::Pipeline,
        cache: &Cache,
        new_transformation: Transformation,
    ) {
        match self.uploads.entry(cache.id) {
            hash_map::Entry::Occupied(entry) => {
                let upload = entry.into_mut();

                if !cache.batch.is_empty()
                    && (upload.version != cache.version
                        || upload.transformation != new_transformation)
                {
                    upload.layer.prepare(
                        device,
                        encoder,
                        belt,
                        solid,
                        gradient,
                        &cache.batch,
                        new_transformation,
                    );

                    upload.batch = Arc::downgrade(&cache.batch);
                    upload.version = cache.version;
                    upload.transformation = new_transformation;
                }
            }
            hash_map::Entry::Vacant(entry) => {
                let mut layer = Layer::new(device, solid, gradient);

                layer.prepare(
                    device,
                    encoder,
                    belt,
                    solid,
                    gradient,
                    &cache.batch,
                    new_transformation,
                );

                let _ = entry.insert(Upload {
                    layer,
                    transformation: new_transformation,
                    version: 0,
                    batch: Arc::downgrade(&cache.batch),
                });

                log::debug!(
                    "New mesh upload: {} (total: {})",
                    cache.id.0,
                    self.uploads.len()
                );
            }
        }
    }

    pub fn trim(&mut self) {
        self.uploads
            .retain(|_id, upload| upload.batch.strong_count() > 0);
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    msaa: Option<msaa::Pipeline>,
    solid: solid::Pipeline,
    gradient: gradient::Pipeline,
}

pub struct State {
    msaa: Option<msaa::State>,
    layers: Vec<Layer>,
    prepare_layer: usize,
    storage: Storage,
}

impl State {
    pub fn new(device: &wgpu::Device, pipeline: &Pipeline) -> Self {
        Self {
            msaa: pipeline
                .msaa
                .as_ref()
                .map(|pipeline| msaa::State::new(device, pipeline)),
            layers: Vec::new(),
            prepare_layer: 0,
            storage: Storage::new(),
        }
    }

    pub fn prepare(
        &mut self,
        pipeline: &Pipeline,
        device: &wgpu::Device,
        belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        items: &[Item],
        scale: Transformation,
        target_size: Size<u32>,
    ) {
        let projection = if let Some((state, pipeline)) =
            self.msaa.as_mut().zip(pipeline.msaa.as_ref())
        {
            state.prepare(device, encoder, belt, pipeline, target_size) * scale
        } else {
            Transformation::orthographic(target_size.width, target_size.height)
                * scale
        };

        for item in items {
            match item {
                Item::Group {
                    transformation,
                    meshes,
                } => {
                    if self.layers.len() <= self.prepare_layer {
                        self.layers.push(Layer::new(
                            device,
                            &pipeline.solid,
                            &pipeline.gradient,
                        ));
                    }

                    let layer = &mut self.layers[self.prepare_layer];
                    layer.prepare(
                        device,
                        encoder,
                        belt,
                        &pipeline.solid,
                        &pipeline.gradient,
                        meshes,
                        projection * *transformation,
                    );

                    self.prepare_layer += 1;
                }
                Item::Cached {
                    transformation,
                    cache,
                } => {
                    self.storage.prepare(
                        device,
                        encoder,
                        belt,
                        &pipeline.solid,
                        &pipeline.gradient,
                        cache,
                        projection * *transformation,
                    );
                }
            }
        }
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        start: usize,
        batch: &Batch,
        bounds: Rectangle,
        screen_transformation: Transformation,
    ) -> usize {
        let mut layer_count = 0;

        let items = batch.iter().filter_map(|item| match item {
            Item::Group {
                transformation,
                meshes,
            } => {
                let layer = &self.layers[start + layer_count];
                layer_count += 1;

                Some((
                    layer,
                    meshes.as_slice(),
                    screen_transformation * *transformation,
                ))
            }
            Item::Cached {
                transformation,
                cache,
            } => {
                let upload = self.storage.get(cache)?;

                Some((
                    &upload.layer,
                    &cache.batch,
                    screen_transformation * *transformation,
                ))
            }
        });

        render(
            encoder,
            target,
            self.msaa.as_ref().zip(pipeline.msaa.as_ref()),
            &pipeline.solid,
            &pipeline.gradient,
            bounds,
            items,
        );

        layer_count
    }

    pub fn trim(&mut self) {
        self.storage.trim();

        self.prepare_layer = 0;
    }
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<Antialiasing>,
    ) -> Pipeline {
        Pipeline {
            msaa: antialiasing.map(|a| msaa::Pipeline::new(device, format, a)),
            solid: solid::Pipeline::new(device, format, antialiasing),
            gradient: gradient::Pipeline::new(device, format, antialiasing),
        }
    }
}

fn render<'a>(
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    mut msaa: Option<(&msaa::State, &msaa::Pipeline)>,
    solid: &solid::Pipeline,
    gradient: &gradient::Pipeline,
    bounds: Rectangle,
    group: impl Iterator<Item = (&'a Layer, &'a [Mesh], Transformation)>,
) {
    {
        let mut render_pass = if let Some((_state, pipeline)) = &mut msaa {
            pipeline.render_pass(encoder)
        } else {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu.triangle.render_pass"),
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
            })
        };

        for (layer, meshes, transformation) in group {
            layer.render(
                solid,
                gradient,
                meshes,
                bounds,
                transformation,
                &mut render_pass,
            );
        }
    }

    if let Some((state, pipeline)) = msaa {
        state.render(pipeline, encoder, target);
    }
}

#[derive(Debug)]
pub struct Layer {
    index_buffer: Buffer<u32>,
    index_strides: Vec<u32>,
    solid: solid::Layer,
    gradient: gradient::Layer,
}

impl Layer {
    fn new(
        device: &wgpu::Device,
        solid: &solid::Pipeline,
        gradient: &gradient::Pipeline,
    ) -> Self {
        Self {
            index_buffer: Buffer::new(
                device,
                "iced_wgpu.triangle.index_buffer",
                INITIAL_INDEX_COUNT,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            ),
            index_strides: Vec::new(),
            solid: solid::Layer::new(device, &solid.constants_layout),
            gradient: gradient::Layer::new(device, &gradient.constants_layout),
        }
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        solid: &solid::Pipeline,
        gradient: &gradient::Pipeline,
        meshes: &[Mesh],
        transformation: Transformation,
    ) {
        // Count the total amount of vertices & indices we need to handle
        let count = mesh::attribute_count_of(meshes);

        // Then we ensure the current attribute buffers are big enough, resizing if necessary.
        // We are not currently using the return value of these functions as we have no system in
        // place to calculate mesh diff, or to know whether or not that would be more performant for
        // the majority of use cases. Therefore we will write GPU data every frame (for now).
        let _ = self.index_buffer.resize(device, count.indices);
        let _ = self.solid.vertices.resize(device, count.solid_vertices);
        let _ = self
            .gradient
            .vertices
            .resize(device, count.gradient_vertices);

        if self.solid.uniforms.resize(device, count.solids) {
            self.solid.constants = solid::Layer::bind_group(
                device,
                &self.solid.uniforms.raw,
                &solid.constants_layout,
            );
        }

        if self.gradient.uniforms.resize(device, count.gradients) {
            self.gradient.constants = gradient::Layer::bind_group(
                device,
                &self.gradient.uniforms.raw,
                &gradient.constants_layout,
            );
        }

        self.index_strides.clear();
        self.index_buffer.clear();
        self.solid.vertices.clear();
        self.solid.uniforms.clear();
        self.gradient.vertices.clear();
        self.gradient.uniforms.clear();

        let mut solid_vertex_offset = 0;
        let mut solid_uniform_offset = 0;
        let mut gradient_vertex_offset = 0;
        let mut gradient_uniform_offset = 0;
        let mut index_offset = 0;

        for mesh in meshes {
            let clip_bounds = mesh.clip_bounds() * transformation;
            let snap_distance = clip_bounds
                .snap()
                .map(|snapped_bounds| {
                    Point::new(snapped_bounds.x as f32, snapped_bounds.y as f32)
                        - clip_bounds.position()
                })
                .unwrap_or(Vector::ZERO);

            let uniforms = Uniforms::new(
                transformation
                    * mesh.transformation()
                    * Transformation::translate(
                        snap_distance.x,
                        snap_distance.y,
                    ),
            );

            let indices = mesh.indices();

            index_offset += self.index_buffer.write(
                device,
                encoder,
                belt,
                index_offset,
                indices,
            );

            self.index_strides.push(indices.len() as u32);

            match mesh {
                Mesh::Solid { buffers, .. } => {
                    solid_vertex_offset += self.solid.vertices.write(
                        device,
                        encoder,
                        belt,
                        solid_vertex_offset,
                        &buffers.vertices,
                    );

                    solid_uniform_offset += self.solid.uniforms.write(
                        device,
                        encoder,
                        belt,
                        solid_uniform_offset,
                        &[uniforms],
                    );
                }
                Mesh::Gradient { buffers, .. } => {
                    gradient_vertex_offset += self.gradient.vertices.write(
                        device,
                        encoder,
                        belt,
                        gradient_vertex_offset,
                        &buffers.vertices,
                    );

                    gradient_uniform_offset += self.gradient.uniforms.write(
                        device,
                        encoder,
                        belt,
                        gradient_uniform_offset,
                        &[uniforms],
                    );
                }
            }
        }
    }

    fn render<'a>(
        &'a self,
        solid: &'a solid::Pipeline,
        gradient: &'a gradient::Pipeline,
        meshes: &[Mesh],
        bounds: Rectangle,
        transformation: Transformation,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let mut num_solids = 0;
        let mut num_gradients = 0;
        let mut last_is_solid = None;

        for (index, mesh) in meshes.iter().enumerate() {
            let Some(clip_bounds) = bounds
                .intersection(&(mesh.clip_bounds() * transformation))
                .and_then(Rectangle::snap)
            else {
                match mesh {
                    Mesh::Solid { .. } => {
                        num_solids += 1;
                    }
                    Mesh::Gradient { .. } => {
                        num_gradients += 1;
                    }
                }
                continue;
            };

            render_pass.set_scissor_rect(
                clip_bounds.x,
                clip_bounds.y,
                clip_bounds.width,
                clip_bounds.height,
            );

            match mesh {
                Mesh::Solid { .. } => {
                    if !last_is_solid.unwrap_or(false) {
                        render_pass.set_pipeline(&solid.pipeline);

                        last_is_solid = Some(true);
                    }

                    render_pass.set_bind_group(
                        0,
                        &self.solid.constants,
                        &[(num_solids * std::mem::size_of::<Uniforms>())
                            as u32],
                    );

                    render_pass.set_vertex_buffer(
                        0,
                        self.solid.vertices.slice_from_index(num_solids),
                    );

                    num_solids += 1;
                }
                Mesh::Gradient { .. } => {
                    if last_is_solid.unwrap_or(true) {
                        render_pass.set_pipeline(&gradient.pipeline);

                        last_is_solid = Some(false);
                    }

                    render_pass.set_bind_group(
                        0,
                        &self.gradient.constants,
                        &[(num_gradients * std::mem::size_of::<Uniforms>())
                            as u32],
                    );

                    render_pass.set_vertex_buffer(
                        0,
                        self.gradient.vertices.slice_from_index(num_gradients),
                    );

                    num_gradients += 1;
                }
            };

            render_pass.set_index_buffer(
                self.index_buffer.slice_from_index(index),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.draw_indexed(0..self.index_strides[index], 0, 0..1);
        }
    }
}

fn fragment_target(
    texture_format: wgpu::TextureFormat,
) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

fn primitive_state() -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        front_face: wgpu::FrontFace::Cw,
        ..Default::default()
    }
}

fn multisample_state(
    antialiasing: Option<Antialiasing>,
) -> wgpu::MultisampleState {
    wgpu::MultisampleState {
        count: antialiasing.map(Antialiasing::sample_count).unwrap_or(1),
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    transform: [f32; 16],
    /// Uniform values must be 256-aligned;
    /// see: [`wgpu::Limits`] `min_uniform_buffer_offset_alignment`.
    _padding: [f32; 48],
}

impl Uniforms {
    pub fn new(transform: Transformation) -> Self {
        Self {
            transform: transform.into(),
            _padding: [0.0; 48],
        }
    }

    pub fn entry() -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<Self>() as u64,
                ),
            },
            count: None,
        }
    }

    pub fn min_size() -> Option<wgpu::BufferSize> {
        wgpu::BufferSize::new(std::mem::size_of::<Self>() as u64)
    }
}

mod solid {
    use crate::Buffer;
    use crate::graphics::Antialiasing;
    use crate::graphics::mesh;
    use crate::triangle;

    #[derive(Debug, Clone)]
    pub struct Pipeline {
        pub pipeline: wgpu::RenderPipeline,
        pub constants_layout: wgpu::BindGroupLayout,
    }

    #[derive(Debug)]
    pub struct Layer {
        pub vertices: Buffer<mesh::SolidVertex2D>,
        pub uniforms: Buffer<triangle::Uniforms>,
        pub constants: wgpu::BindGroup,
    }

    impl Layer {
        pub fn new(
            device: &wgpu::Device,
            constants_layout: &wgpu::BindGroupLayout,
        ) -> Self {
            let vertices = Buffer::new(
                device,
                "iced_wgpu.triangle.solid.vertex_buffer",
                triangle::INITIAL_VERTEX_COUNT,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            );

            let uniforms = Buffer::new(
                device,
                "iced_wgpu.triangle.solid.uniforms",
                1,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            );

            let constants =
                Self::bind_group(device, &uniforms.raw, constants_layout);

            Self {
                vertices,
                uniforms,
                constants,
            }
        }

        pub fn bind_group(
            device: &wgpu::Device,
            buffer: &wgpu::Buffer,
            layout: &wgpu::BindGroupLayout,
        ) -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu.triangle.solid.bind_group"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer,
                            offset: 0,
                            size: triangle::Uniforms::min_size(),
                        },
                    ),
                }],
            })
        }
    }

    impl Pipeline {
        pub fn new(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            antialiasing: Option<Antialiasing>,
        ) -> Self {
            let constants_layout = device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("iced_wgpu.triangle.solid.bind_group_layout"),
                    entries: &[triangle::Uniforms::entry()],
                },
            );

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("iced_wgpu.triangle.solid.pipeline_layout"),
                    bind_group_layouts: &[&constants_layout],
                    push_constant_ranges: &[],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("iced_wgpu.triangle.solid.shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(concat!(
                            include_str!("shader/triangle.wgsl"),
                            "\n",
                            include_str!("shader/triangle/solid.wgsl"),
                            "\n",
                            include_str!("shader/color.wgsl"),
                        )),
                    ),
                });

            let pipeline =
                device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: Some("iced_wgpu::triangle::solid pipeline"),
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("solid_vs_main"),
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: std::mem::size_of::<
                                    mesh::SolidVertex2D,
                                >(
                                )
                                    as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &wgpu::vertex_attr_array!(
                                    // Position
                                    0 => Float32x2,
                                    // Color
                                    1 => Float32x4,
                                ),
                            }],
                            compilation_options:
                                wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("solid_fs_main"),
                            targets: &[Some(triangle::fragment_target(format))],
                            compilation_options:
                                wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: triangle::primitive_state(),
                        depth_stencil: None,
                        multisample: triangle::multisample_state(antialiasing),
                        multiview: None,
                        cache: None,
                    },
                );

            Self {
                pipeline,
                constants_layout,
            }
        }
    }
}

mod gradient {
    use crate::Buffer;
    use crate::graphics::Antialiasing;
    use crate::graphics::color;
    use crate::graphics::mesh;
    use crate::triangle;

    #[derive(Debug, Clone)]
    pub struct Pipeline {
        pub pipeline: wgpu::RenderPipeline,
        pub constants_layout: wgpu::BindGroupLayout,
    }

    #[derive(Debug)]
    pub struct Layer {
        pub vertices: Buffer<mesh::GradientVertex2D>,
        pub uniforms: Buffer<triangle::Uniforms>,
        pub constants: wgpu::BindGroup,
    }

    impl Layer {
        pub fn new(
            device: &wgpu::Device,
            constants_layout: &wgpu::BindGroupLayout,
        ) -> Self {
            let vertices = Buffer::new(
                device,
                "iced_wgpu.triangle.gradient.vertex_buffer",
                triangle::INITIAL_VERTEX_COUNT,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            );

            let uniforms = Buffer::new(
                device,
                "iced_wgpu.triangle.gradient.uniforms",
                1,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            );

            let constants =
                Self::bind_group(device, &uniforms.raw, constants_layout);

            Self {
                vertices,
                uniforms,
                constants,
            }
        }

        pub fn bind_group(
            device: &wgpu::Device,
            uniform_buffer: &wgpu::Buffer,
            layout: &wgpu::BindGroupLayout,
        ) -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu.triangle.gradient.bind_group"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: uniform_buffer,
                            offset: 0,
                            size: triangle::Uniforms::min_size(),
                        },
                    ),
                }],
            })
        }
    }

    impl Pipeline {
        pub fn new(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            antialiasing: Option<Antialiasing>,
        ) -> Self {
            let constants_layout = device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some(
                        "iced_wgpu.triangle.gradient.bind_group_layout",
                    ),
                    entries: &[triangle::Uniforms::entry()],
                },
            );

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("iced_wgpu.triangle.gradient.pipeline_layout"),
                    bind_group_layouts: &[&constants_layout],
                    push_constant_ranges: &[],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("iced_wgpu.triangle.gradient.shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(
                            if color::GAMMA_CORRECTION {
                                concat!(
                                    include_str!("shader/triangle.wgsl"),
                                    "\n",
                                    include_str!(
                                        "shader/triangle/gradient.wgsl"
                                    ),
                                    "\n",
                                    include_str!("shader/color.wgsl"),
                                    "\n",
                                    include_str!("shader/color/oklab.wgsl")
                                )
                            } else {
                                concat!(
                                    include_str!("shader/triangle.wgsl"),
                                    "\n",
                                    include_str!(
                                        "shader/triangle/gradient.wgsl"
                                    ),
                                    "\n",
                                    include_str!("shader/color.wgsl"),
                                    "\n",
                                    include_str!(
                                        "shader/color/linear_rgb.wgsl"
                                    )
                                )
                            },
                        ),
                    ),
                });

            let pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("iced_wgpu.triangle.gradient.pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("gradient_vs_main"),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<
                                mesh::GradientVertex2D,
                            >()
                                as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array!(
                                // Position
                                0 => Float32x2,
                                // Colors 1-2
                                1 => Uint32x4,
                                // Colors 3-4
                                2 => Uint32x4,
                                // Colors 5-6
                                3 => Uint32x4,
                                // Colors 7-8
                                4 => Uint32x4,
                                // Offsets
                                5 => Uint32x4,
                                // Direction
                                6 => Float32x4
                            ),
                        }],
                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("gradient_fs_main"),
                        targets: &[Some(triangle::fragment_target(format))],
                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: triangle::primitive_state(),
                    depth_stencil: None,
                    multisample: triangle::multisample_state(antialiasing),
                    multiview: None,
                    cache: None,
                },
            );

            Self {
                pipeline,
                constants_layout,
            }
        }
    }
}
