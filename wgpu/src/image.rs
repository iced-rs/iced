#[cfg(feature = "image")]
mod raster;
#[cfg(feature = "svg")]
mod vector;

#[cfg(feature = "image")]
use crate::image::raster::Memory;

use crate::Transformation;
use iced_native::{image, svg, Rectangle};

use std::{collections::{HashMap, HashSet}, mem};

#[cfg(any(feature = "image", feature = "svg"))]
use std::cell::RefCell;

use guillotiere::{Allocation, AtlasAllocator, Size};
use debug_stub_derive::*;

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
    instances: wgpu::Buffer,
    constants: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    atlas_array: AtlasArray,
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

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            size: mem::size_of::<Instance>() as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let atlas_array = AtlasArray::new(1, device);

        Pipeline {
            #[cfg(feature = "image")]
            raster_cache: RefCell::new(raster::Cache::new()),

            #[cfg(feature = "svg")]
            vector_cache: RefCell::new(vector::Cache::new()),

            pipeline,
            uniforms: uniforms_buffer,
            vertices,
            indices,
            instances,
            constants: constant_bind_group,
            texture_layout,
            atlas_array,
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
        instances: &[Image],
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

        #[cfg(any(feature = "image", feature = "svg"))]
        let mut recs = HashMap::new();

        for (index, image) in instances.iter().enumerate() {
            match &image.handle {
                Handle::Raster(_handle) => {
                    #[cfg(feature = "image")]
                    {
                        let mut raster_cache = self.raster_cache.borrow_mut();

                        if let Memory::Device { layer, allocation } = raster_cache.upload(
                            _handle,
                            device,
                            encoder,
                            &mut self.atlas_array,
                        ) {
                            let rec = (*layer, allocation.rectangle);

                            let _ = recs.insert(index, rec);
                        }
                    }
                }
                Handle::Vector(_handle) => {
                    #[cfg(feature = "svg")]
                    {
                        let mut vector_cache = self.vector_cache.borrow_mut();

                        // Upload rasterized svg to texture atlas
                        if let Some((layer, allocation)) = vector_cache.upload(
                            _handle,
                            image.scale,
                            _scale,
                            device,
                            encoder,
                            &mut self.atlas_array,
                        ) {
                            let rec = (*layer, allocation.rectangle);

                            let _ = recs.insert(index, rec);
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
                    &self.atlas_array.texture().create_default_view(),
                ),
            }],
        });

        #[cfg(any(feature = "image", feature = "svg"))]
        for (index, image) in instances.iter().enumerate() {
            if let Some((layer, rec)) = recs.get(&index) {
                let x = (rec.min.x as f32 + 0.5) / (ATLAS_SIZE as f32);
                let y = (rec.min.y as f32 + 0.5) / (ATLAS_SIZE as f32);
                let w = (rec.size().width as f32 - 0.5) / (ATLAS_SIZE as f32);
                let h = (rec.size().height as f32 - 0.5) / (ATLAS_SIZE as f32);

                let instance_buffer = device
                    .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
                    .fill_from_slice(&[Instance {
                        _position: image.position,
                        _scale: image.scale,
                        _position_in_atlas: [x, y],
                        _scale_in_atlas: [w, h],
                        _layer: *layer as f32,
                    }]);

                encoder.copy_buffer_to_buffer(
                    &instance_buffer,
                    0,
                    &self.instances,
                    0,
                    mem::size_of::<Instance>() as u64,
                );

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
                    &[(&self.vertices, 0), (&self.instances, 0)],
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
                    0..1 as u32,
                );
            }
        }
    }

    pub fn trim_cache(&mut self) {
        #[cfg(feature = "image")]
        self.raster_cache.borrow_mut().trim(&mut self.atlas_array);

        #[cfg(feature = "svg")]
        self.vector_cache.borrow_mut().trim(&mut self.atlas_array);
    }
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

#[derive(DebugStub)]
pub struct AtlasArray {
    texture: wgpu::Texture,
    #[debug_stub="ReplacementValue"]
    allocators: HashMap<u32, AtlasAllocator>,
    layers_without_allocators: HashSet<u32>,
    size: u32,
}

impl AtlasArray {
    pub fn new(array_size: u32, device: &wgpu::Device) -> Self {
        let (width, height) = (ATLAS_SIZE, ATLAS_SIZE);

        let extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: array_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        AtlasArray {
            texture,
            allocators: HashMap::new(),
            layers_without_allocators: HashSet::new(),
            size: array_size,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn allocate(&mut self, size: Size) -> Option<(u32, Allocation)> {
        for layer in 0..self.size {
            if self.layers_without_allocators.contains(&layer) {
                continue;
            }

            let allocator = self.allocators.entry(layer)
                .or_insert_with(|| AtlasAllocator::new(
                    Size::new(ATLAS_SIZE as i32, ATLAS_SIZE as i32)
                ));

            if let Some(a) = allocator.allocate(size.clone()) {
                return Some((layer, a));
            }
        }

        None
    }

    pub fn deallocate(&mut self, layer: u32, allocation: &Allocation) {
        if let Some(allocator) = self.allocators.get_mut(&layer) {
            allocator.deallocate(allocation.id);
        }
    }

    pub fn upload<T: Copy + 'static>(
        &mut self,
        data: &[T],
        layer: u32,
        allocation: &guillotiere::Allocation,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let size = allocation.rectangle.size();
        let (width, height) = (size.width as u32, size.height as u32);

        let extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let temp_buf = device
            .create_buffer_mapped(
                data.len(),
                wgpu::BufferUsage::COPY_SRC,
            )
            .fill_from_slice(data);

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * width,
                image_height: height,
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                array_layer: layer as u32,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: allocation.rectangle.min.x as f32,
                    y: allocation.rectangle.min.y as f32,
                    z: 0.0,
                },
            },
            extent,
        );
    }

    pub fn grow(
        &mut self,
        grow_by: u32,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let old_atlas_array_size = self.size;

        let new_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth: 1,
            },
            array_layer_count: old_atlas_array_size + grow_by,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        for i in 0..old_atlas_array_size {
            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &self.texture,
                    array_layer: i,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                wgpu::TextureCopyView {
                    texture: &new_texture,
                    array_layer: i,
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

        self.texture = new_texture;
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

const ATLAS_SIZE: u32 = 8192;

#[repr(C)]
#[derive(Clone, Copy)]
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
