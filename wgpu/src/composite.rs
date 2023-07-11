use crate::core::{Rectangle, Size};
use crate::graphics::Transformation;

pub use pipeline::Pipeline;

/// A sub-surface within the viewport.
#[derive(Debug)]
pub struct Surface {
    /// The clip area for the [`Surface`] within the viewport.
    pub clip: Rectangle<u32>,
    /// Underlying texture of the [`Surface`].
    pub texture: wgpu::Texture,
    /// Texture view of the [`Surface`].
    pub view: wgpu::TextureView,
}

impl Surface {
    /// Creates a new [`Surface`].
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        clip: Rectangle<u32>,
        texture_size: Size<u32>,
        label: &'static str,
    ) -> Self {
        let texture = Self::create_texture(device, format, texture_size, label);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            clip,
            view,
        }
    }

    //TODO move this somewhere else, this is just generic 2d texture. Or remove it
    /// Create a texture for a [`Surface`].
    pub fn create_texture(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: Size<u32>,
        label: &'static str,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    /// Returns the orthographic transform of the underlying texture of the [`Surface`].
    pub fn transform(&self) -> Transformation {
        let texture_size = self.texture.size();
        Transformation::orthographic(texture_size.width, texture_size.height)
    }

    /// The ratio of the [`Surface`]'s underlying texture size : clip bounds.
    ///
    /// This could be != 1 in situations where the texture must be up/downsampled.
    pub fn ratio(&self) -> f32 {
        self.texture.width() as f32 / self.clip.width as f32
    }

    /// The size of the underlying [`wgpu::Texture`] of the [`Surface`].
    pub fn texture_size(&self) -> Size<u32> {
        let size = self.texture.size();
        Size::new(size.width, size.height)
    }

    /// Returns the texture scissor bounds. This is not necessarily the same as the [`Surface`]'s
    /// clip bounds.
    pub fn scissor(&self) -> Rectangle<u32> {
        let size = self.texture_size();

        Rectangle::<u32> {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
        }
    }

    /// Returns `true` if the backing texture of this [`Surface`] was resized.
    ///
    /// This does not effect the [`Surface`]'s clip bounds.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        new_size: Size<u32>,
        label: &'static str,
    ) -> bool {
        let texture_size = self.texture.size();

        if new_size.width != texture_size.width
            || new_size.height != texture_size.height
        {
            self.texture =
                Self::create_texture(device, self.texture.format(), new_size, label);
            self.view = self
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            true
        } else {
            false
        }
    }
}

mod pipeline {
    use crate::core::Rectangle;
    use crate::graphics::Transformation;

    /// A [`Pipeline`] which can composite a texture onto a surface.
    pub struct Pipeline {
        pipeline: wgpu::RenderPipeline,
        sampler: wgpu::Sampler,
        uniform_layout: wgpu::BindGroupLayout,
        layers: Vec<Layer>,
        prepare_layer: usize,
    }

    /// An individual layer to be composited.
    #[derive(Debug)]
    struct Layer {
        uniforms: wgpu::Buffer,
        bind_group: Option<wgpu::BindGroup>,
    }

    impl Layer {
        /// Creates a new composite [`Layer`].
        pub fn new(device: &wgpu::Device) -> Self {
            let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("iced_wgpu.composite.layer.uniforms"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            Self {
                uniforms,
                bind_group: None,
            }
        }

        /// Prepared the composite [`Layer`] for rendering.
        ///
        /// `surface_clip`: The final bounds of the surface. May be upsampled/downsampled depending
        /// on the `surface_view`'s size compared to the `surface_clip`. If the texture needs to be
        /// up or downsampled, the sampler will use bilinear filtering.
        pub fn prepare(
            &mut self,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            surface_clip: Rectangle,
            surface_view: &wgpu::TextureView,
            transform: Transformation,
            layout: &wgpu::BindGroupLayout,
            sampler: &wgpu::Sampler,
        ) {
            queue.write_buffer(
                &self.uniforms,
                0,
                bytemuck::bytes_of(&Uniforms {
                    transform: transform.into(),
                    scale: [surface_clip.width, surface_clip.height],
                    position: [surface_clip.x, surface_clip.y],
                }),
            );

            self.bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wpgu.composite.layer.bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                self.uniforms.as_entire_buffer_binding(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                surface_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                }))
        }
    }

    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    #[repr(C)]
    struct Uniforms {
        transform: [f32; 16],
        scale: [f32; 2], // size of quad
        position: [f32; 2], // position of quad
    }

    impl Pipeline {
        /// Creates a new composite [`Pipeline`].
        pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            let uniform_layout = device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("iced_wgpu.composite.layer.uniforms_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
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
                                view_dimension: wgpu::TextureViewDimension::D2,
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
                    ],
                },
            );

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("iced_wgpu.composite.pipeline_layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts: &[&uniform_layout],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("iced_wgpu.composite.shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(include_str!(
                            "shader/compositor.wgsl"
                        )),
                    ),
                });

            let pipeline =
                device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("iced_wgpu.composite.render_pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(
                                wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                            ),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

            Self {
                pipeline,
                uniform_layout,
                layers: vec![],
                sampler,
                prepare_layer: 0,
            }
        }

        /// Creates and prepares layers of the `Pipeline` for rendering.
        pub fn prepare(
            &mut self,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            transform: Transformation,
            surface_clip: Rectangle,
            surface_view: &wgpu::TextureView,
        ) {
            if self.layers.len() <= self.prepare_layer {
                self.layers.push(Layer::new(device));
            }

            let layer = &mut self.layers[self.prepare_layer];
            layer.prepare(
                device,
                queue,
                surface_clip,
                surface_view,
                transform,
                &self.uniform_layout,
                &self.sampler,
            );

            self.prepare_layer += 1;
        }

        pub fn render<'a>(
            &'a mut self,
            pass: &mut wgpu::RenderPass<'a>,
            layer: usize,
        ) {
            if let Some(layer) = self.layers.get(layer) {
                pass.set_pipeline(&self.pipeline);

                if let Some(bind_group) = &layer.bind_group {
                    pass.set_bind_group(0, bind_group, &[]);
                }
                pass.draw(0..6, 0..1);
            }
        }

        pub fn end_frame(&mut self) {
            self.prepare_layer = 0;
        }
    }
}
