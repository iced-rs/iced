#[cfg(feature = "image")]
mod raster;
#[cfg(feature = "svg")]
mod vector;

#[cfg(feature = "image")]
use crate::image::raster::Memory;

use crate::Transformation;
use iced_native::{image, svg, Rectangle};

use std::mem;

#[cfg(any(feature = "image", feature = "svg"))]
use std::cell::RefCell;

use guillotiere::{Allocation, AtlasAllocator, Size};

#[derive(Debug)]
pub struct Pipeline {
    #[cfg(feature = "image")]
    raster_cache: RefCell<raster::Cache>,
    #[cfg(feature = "svg")]
    vector_cache: RefCell<vector::Cache>,

    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    constants: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    texture_array: TextureArray,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                ],
            });

        let uniforms = Uniforms {
            transform: Transformation::identity().into(),
        };

        let uniforms_buffer = device
            .create_buffer_mapped(
                1,
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            )
            .fill_from_slice(&[uniforms]);

        let constant_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &constant_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &uniforms_buffer,
                            range: 0..std::mem::size_of::<Uniforms>() as u64,
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                }],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&constant_layout, &texture_layout],
            });

        let vs = include_bytes!("shader/image.vert.spv");
        let vs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&vs[..]))
                .expect("Read image vertex shader as SPIR-V"),
        );

        let fs = include_bytes!("shader/image.frag.spv");
        let fs_module = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&fs[..]))
                .expect("Read image fragment shader as SPIR-V"),
        );

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: &layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[wgpu::VertexAttributeDescriptor {
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float2,
                            offset: 0,
                        }],
                    },
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float2,
                                offset: 0,
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float2,
                                offset: 4 * 2,
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float2,
                                offset: 4 * 4,
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float2,
                                offset: 4 * 6,
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float,
                                offset: 4 * 8,
                            },
                        ],
                    },
                ],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        let vertices = device
            .create_buffer_mapped(QUAD_VERTS.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&QUAD_VERTS);

        let indices = device
            .create_buffer_mapped(QUAD_INDICES.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&QUAD_INDICES);

        let texture_array = TextureArray::new(device);

        Pipeline {
            #[cfg(feature = "image")]
            raster_cache: RefCell::new(raster::Cache::new()),

            #[cfg(feature = "svg")]
            vector_cache: RefCell::new(vector::Cache::new()),

            pipeline,
            uniforms: uniforms_buffer,
            vertices,
            indices,
            constants: constant_bind_group,
            texture_layout,
            texture_array,
        }
    }

    #[cfg(feature = "image")]
    pub fn dimensions(&self, handle: &image::Handle) -> (u32, u32) {
        let mut cache = self.raster_cache.borrow_mut();
        let memory = cache.load(&handle);

        memory.dimensions()
    }

    #[cfg(feature = "svg")]
    pub fn viewport_dimensions(&self, handle: &svg::Handle) -> (u32, u32) {
        let mut cache = self.vector_cache.borrow_mut();
        let svg = cache.load(&handle);

        svg.viewport_dimensions()
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        images: &[Image],
        transformation: Transformation,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        _scale: f32,
    ) {
        let uniforms_buffer = device
            .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&[Uniforms {
                transform: transformation.into(),
            }]);

        encoder.copy_buffer_to_buffer(
            &uniforms_buffer,
            0,
            &self.uniforms,
            0,
            std::mem::size_of::<Uniforms>() as u64,
        );

        let mut instances: Vec<Instance> = Vec::new();

        for image in images {
            match &image.handle {
                Handle::Raster(_handle) => {
                    #[cfg(feature = "image")]
                    {
                        let mut raster_cache = self.raster_cache.borrow_mut();

                        if let Memory::Device(allocation) = raster_cache.upload(
                            _handle,
                            device,
                            encoder,
                            &mut self.texture_array,
                        ) {
                            add_instances(
                                image,
                                allocation,
                                &mut instances,
                            );
                        }
                    }
                }
                Handle::Vector(_handle) => {
                    #[cfg(feature = "svg")]
                    {
                        let mut vector_cache = self.vector_cache.borrow_mut();

                        // Upload rasterized svg to texture atlas
                        if let Some(allocation) = vector_cache.upload(
                            _handle,
                            image.scale,
                            _scale,
                            device,
                            encoder,
                            &mut self.texture_array,
                        ) {
                            add_instances(
                                image,
                                allocation,
                                &mut instances,
                            );
                        }
                    }
                }
            }
        }

        let texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &self.texture_array.texture.create_default_view(),
                ),
            }],
        });

        let instances_buffer = device.create_buffer_mapped(
            instances.len(),
            wgpu::BufferUsage::VERTEX,
        ).fill_from_slice(&instances);

        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: target,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Load,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            },
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.constants, &[]);
        render_pass.set_bind_group(1, &texture, &[]);
        render_pass.set_index_buffer(&self.indices, 0);
        render_pass.set_vertex_buffers(
            0,
            &[(&self.vertices, 0), (&instances_buffer, 0)],
        );

        render_pass.set_scissor_rect(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
        );

        render_pass.draw_indexed(
            0..QUAD_INDICES.len() as u32,
            0,
            0..instances.len() as u32,
        );
    }

    pub fn trim_cache(&mut self) {
        #[cfg(feature = "image")]
        self.raster_cache.borrow_mut().trim(&mut self.texture_array);

        #[cfg(feature = "svg")]
        self.vector_cache.borrow_mut().trim(&mut self.texture_array);
    }
}

fn add_instances(
    image: &Image,
    allocation: &ImageAllocation,
    instances: &mut Vec<Instance>,
)  {
    match allocation {
        ImageAllocation::SingleAllocation(allocation) => {
            add_instance(image.position, image.scale, allocation, instances);
        }
        ImageAllocation::MultipleAllocations { mappings, size } => {
            let scaling_x = image.scale[0] / size.0 as f32;
            let scaling_y = image.scale[1] / size.1 as f32;

            for mapping in mappings {
                let allocation = &mapping.allocation;
                let mut position = image.position;
                let mut scale = image.scale;

                position[0] += mapping.src_pos.0 as f32 * scaling_x;
                position[1] += mapping.src_pos.1 as f32 * scaling_y;
                scale[0] = allocation.size().0 as f32 * scaling_x;
                scale[1] = allocation.size().1 as f32 * scaling_y;

                add_instance(position, scale, allocation, instances);
            }
        }
    }
}

fn add_instance(
    position: [f32; 2],
    scale: [f32; 2],
    allocation: &ArrayAllocation,
    instances: &mut Vec<Instance>,
) {
    let x = (allocation.position().0 as f32 + 0.5) / (ATLAS_SIZE as f32);
    let y = (allocation.position().1 as f32 + 0.5) / (ATLAS_SIZE as f32);
    let w = (allocation.size().0 as f32 - 0.5) / (ATLAS_SIZE as f32);
    let h = (allocation.size().1 as f32 - 0.5) / (ATLAS_SIZE as f32);
    let layer = allocation.layer() as f32;

    let instance = Instance {
        _position: position,
        _scale: scale,
        _position_in_atlas: [x, y],
        _scale_in_atlas: [w, h],
        _layer: layer,
    };

    instances.push(instance);
}

pub struct Image {
    pub handle: Handle,
    pub position: [f32; 2],
    pub scale: [f32; 2],
}

pub enum Handle {
    Raster(image::Handle),
    Vector(svg::Handle),
}

#[derive(Debug)]
pub struct ArrayAllocationMapping {
    src_pos: (u32, u32),
    allocation: ArrayAllocation,
}

#[derive(Debug)]
pub enum ImageAllocation {
    SingleAllocation(ArrayAllocation),
    MultipleAllocations {
        mappings: Vec<ArrayAllocationMapping>,
        size: (u32, u32),
    },
}

impl ImageAllocation {
    #[cfg(feature = "image")]
    pub fn size(&self) -> (u32, u32) {
        match self {
            ImageAllocation::SingleAllocation(allocation) => {
                allocation.size()
            }
            ImageAllocation::MultipleAllocations { size, .. } => {
                *size
            }
        }
    }
}

pub enum ArrayAllocation {
    AtlasAllocation {
        layer: usize,
        allocation: Allocation,
    },
    WholeLayer {
        layer: usize,
    }
}

impl ArrayAllocation {
    pub fn size(&self) -> (u32, u32) {
        match self {
            ArrayAllocation::AtlasAllocation { allocation, .. } => {
                let size = allocation.rectangle.size();
                (size.width as u32, size.height as u32)
            }
            ArrayAllocation::WholeLayer { .. } => (ATLAS_SIZE, ATLAS_SIZE)
        }
    }

    pub fn position(&self) -> (u32, u32) {
        match self {
            ArrayAllocation::AtlasAllocation { allocation, .. } => {
                let min = &allocation.rectangle.min;
                (min.x as u32, min.y as u32)
            }
            ArrayAllocation::WholeLayer { .. } => (0, 0)
        }
    }

    pub fn layer(&self) -> usize {
        match self {
            ArrayAllocation::AtlasAllocation { layer, .. } => *layer,
            ArrayAllocation::WholeLayer { layer } => *layer,
        }
    }
}

impl std::fmt::Debug for ArrayAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayAllocation::AtlasAllocation { layer, .. } => {
                write!(f, "ArrayAllocation::AtlasAllocation {{ layer: {} }}", layer)
            },
            ArrayAllocation::WholeLayer { layer } => {
                write!(f, "ArrayAllocation::WholeLayer {{ layer: {} }}", layer)
            }
        }
    }
}

pub enum TextureLayer {
    Whole,
    Atlas(AtlasAllocator),
    Empty,
}

impl std::fmt::Debug for TextureLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureLayer::Whole => write!(f, "TextureLayer::Whole"),
            TextureLayer::Atlas(_) => write!(f, "TextureLayer::Atlas"),
            TextureLayer::Empty => write!(f, "TextureLayer::Empty"),
        }
    }
}

#[derive(Debug)]
pub struct TextureArray {
    texture: wgpu::Texture,
    texture_array_size: u32,
    layers: Vec<TextureLayer>,
}

impl TextureArray {
    fn new(device: &wgpu::Device) -> Self {
        let (width, height) = (ATLAS_SIZE, ATLAS_SIZE);

        let extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        TextureArray {
            texture,
            texture_array_size: 1,
            layers: vec!(TextureLayer::Empty),
        }
    }

    fn allocate(&mut self, size: Size) -> Option<ImageAllocation> {
        // Allocate one layer if allocation fits perfectly
        if size.width == ATLAS_SIZE as i32 && size.height == ATLAS_SIZE as i32 {
            for (i, layer) in self.layers.iter_mut().enumerate() {
                if let TextureLayer::Empty = layer
                {
                    *layer = TextureLayer::Whole;
                    return Some(ImageAllocation::SingleAllocation(
                        ArrayAllocation::WholeLayer { layer: i }
                    ));
                }
            }

            self.layers.push(TextureLayer::Whole);
            return Some(ImageAllocation::SingleAllocation(
                ArrayAllocation::WholeLayer { layer: self.layers.len() - 1 }
            ));
        }

        // Split big allocations across multiple layers
        if size.width > ATLAS_SIZE as i32 || size.height > ATLAS_SIZE as i32 {
            let mut mappings = Vec::new();

            let mut y = 0;
            while y < size.height {
                let height = std::cmp::min(size.height - y, ATLAS_SIZE as i32);
                let mut x = 0;

                while x < size.width {
                    let width = std::cmp::min(size.width - x, ATLAS_SIZE as i32);
                    let allocation = self
                        .allocate(Size::new(width, height))
                        .expect("Allocating texture space");

                    if let ImageAllocation::SingleAllocation(allocation) = allocation {
                        let src_pos = (x as u32, y as u32);
                        mappings.push(ArrayAllocationMapping { src_pos, allocation });
                    }

                    x += width;
                }
                y += height;
            }

            return Some(ImageAllocation::MultipleAllocations {
                mappings,
                size: (size.width as u32, size.height as u32),
            });
        }

        // Try allocating on an existing layer
        for (i, layer) in self.layers.iter_mut().enumerate() {
            if let TextureLayer::Atlas(allocator) = layer {
                if let Some(allocation) = allocator.allocate(size.clone()) {
                    let array_allocation = ArrayAllocation::AtlasAllocation { layer: i, allocation };
                    return Some(ImageAllocation::SingleAllocation(array_allocation));
                }
            }
        }

        // Create new layer with atlas allocator
        let mut allocator = AtlasAllocator::new(Size::new(ATLAS_SIZE as i32, ATLAS_SIZE as i32));
        if let Some(allocation) = allocator.allocate(size) {
            self.layers.push(TextureLayer::Atlas(allocator));

            return Some(ImageAllocation::SingleAllocation(
                ArrayAllocation::AtlasAllocation {
                    layer: self.layers.len() - 1,
                    allocation,
                }
            ));
        }

        // One of the above should have worked
        None
    }

    fn deallocate(&mut self, allocation: &ImageAllocation) {
        match allocation {
            ImageAllocation::SingleAllocation(allocation) => {
                if let Some(layer) = self.layers.get_mut(allocation.layer()) {
                    match allocation {
                        ArrayAllocation::WholeLayer { .. } => {
                            *layer = TextureLayer::Empty;
                        }
                        ArrayAllocation::AtlasAllocation { allocation, .. } => {
                            if let TextureLayer::Atlas(allocator) = layer {
                                allocator.deallocate(allocation.id);
                            }
                        }
                    }
                }
            }
            ImageAllocation::MultipleAllocations { mappings, .. } => {
                for mapping in mappings {
                    if let Some(layer) = self.layers.get_mut(mapping.allocation.layer()) {
                        match &mapping.allocation {
                            ArrayAllocation::WholeLayer { .. } => {
                                *layer = TextureLayer::Empty;
                            }
                            ArrayAllocation::AtlasAllocation { allocation, .. } => {
                                if let TextureLayer::Atlas(allocator) = layer {
                                    allocator.deallocate(allocation.id);
                                }
                            }
                        }
                    }
                }
            }
        }

    }

    fn upload<C, I>(
        &mut self,
        image: &I,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> ImageAllocation
    where
        I: RawImageData<Chunk = C>,
        C: Copy + 'static,
    {
        let size = Size::new(image.width() as i32, image.height() as i32);
        let allocation = self.allocate(size).expect("Allocating texture space");

        match &allocation {
            ImageAllocation::SingleAllocation(allocation) => {
                let data = image.data();
                let buffer = device
                    .create_buffer_mapped(
                        data.len(),
                        wgpu::BufferUsage::COPY_SRC,
                    )
                    .fill_from_slice(data);

                if allocation.layer() >= self.texture_array_size as usize {
                    self.grow(1, device, encoder);
                }

                self.upload_texture(
                    &buffer,
                    &allocation,
                    encoder,
                );
            }
            ImageAllocation::MultipleAllocations { mappings, .. } => {
                let chunks_per_pixel = 4 / std::mem::size_of::<C>();
                let chunks_per_line = chunks_per_pixel * image.width() as usize;

                let highest_layer = mappings
                    .iter()
                    .map(|m| m.allocation.layer() as u32)
                    .max()
                    .unwrap_or(0);

                if highest_layer >= self.texture_array_size {
                    let grow_by = 1 + highest_layer - self.texture_array_size;
                    self.grow(grow_by, device, encoder);
                }

                for mapping in mappings {
                    let sub_width = mapping.allocation.size().0 as usize;
                    let sub_height = mapping.allocation.size().1 as usize;
                    let sub_line_start = mapping.src_pos.0 as usize * chunks_per_pixel;
                    let sub_line_end = (mapping.src_pos.0 as usize + sub_width) * chunks_per_pixel;

                    let mut sub_lines = image
                        .data()
                        .chunks(chunks_per_line)
                        .skip(mapping.src_pos.1 as usize)
                        .take(sub_height)
                        .map(|line| &line[sub_line_start..sub_line_end]);

                    let buffer = device
                        .create_buffer_mapped(
                            chunks_per_pixel * sub_width * sub_height,
                            wgpu::BufferUsage::COPY_SRC,
                        );

                    let mut buffer_lines = buffer.data.chunks_mut(sub_width * chunks_per_pixel);

                    while let (Some(buffer_line), Some(sub_line)) = (buffer_lines.next(), sub_lines.next()) {
                        buffer_line.copy_from_slice(sub_line);
                    }

                    self.upload_texture(
                        &buffer.finish(),
                        &mapping.allocation,
                        encoder,
                    );
                }
            }
        }

        allocation
    }

    fn upload_texture(
        &mut self,
        buffer: &wgpu::Buffer,
        allocation: &ArrayAllocation,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let array_layer = allocation.layer() as u32;

        let (width, height) = allocation.size();

        let extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let (x, y) = allocation.position();

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer,
                offset: 0,
                row_pitch: 4 * width,
                image_height: height,
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                array_layer,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as f32,
                    y: y as f32,
                    z: 0.0,
                },
            },
            extent,
        );
    }

    fn grow(
        &mut self,
        grow_by: u32,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if grow_by == 0 {
            return;
        }

        let old_texture_array_size = self.texture_array_size;

        let new_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth: 1,
            },
            array_layer_count: old_texture_array_size + grow_by,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        for (i, layer) in self.layers.iter().enumerate() {
            if i >= old_texture_array_size as usize {
                break;
            }

            if let TextureLayer::Empty = layer {
                continue;
            }

            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &self.texture,
                    array_layer: i as u32,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                wgpu::TextureCopyView {
                    texture: &new_texture,
                    array_layer: i as u32,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                wgpu::Extent3d {
                    width: ATLAS_SIZE,
                    height: ATLAS_SIZE,
                    depth: 1,
                }
            );
        }

        self.texture_array_size += grow_by;
        self.texture = new_texture;
    }
}

trait RawImageData {
    type Chunk;

    fn data(&self) -> &[Self::Chunk];
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

#[cfg(feature = "image")]
impl RawImageData for ::image::ImageBuffer<::image::Bgra<u8>, Vec<u8>> {
    type Chunk = u8;

    fn data(&self) -> &[Self::Chunk] {
        &self
    }

    fn width(&self) -> u32 {
        self.dimensions().0
    }

    fn height(&self) -> u32 {
        self.dimensions().1
    }
}

#[cfg(feature = "svg")]
impl RawImageData for resvg::raqote::DrawTarget {
    type Chunk = u32;

    fn data(&self) -> &[Self::Chunk] {
        self.get_data()
    }

    fn width(&self) -> u32 {
        self.width() as u32
    }

    fn height(&self) -> u32 {
        self.height() as u32
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
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

const ATLAS_SIZE: u32 = 4096;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Instance {
    _position: [f32; 2],
    _scale: [f32; 2],
    _position_in_atlas: [f32; 2],
    _scale_in_atlas: [f32; 2],
    _layer: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
}
