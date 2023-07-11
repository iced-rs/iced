use crate::composite;
use crate::core::{Rectangle, Size};

/// A pipeline which can blur a set of primitives.
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    uniform_layout: wgpu::BindGroupLayout,
    layers: Vec<Layer>,
    prepare_layer: usize,
}

/// Describes a blurred region.
#[derive(Debug)]
struct Layer {
    surface: composite::Surface,
    vertical_texture: wgpu::Texture,
    vertical_uniforms: wgpu::Buffer,
    vertical_bind_group: wgpu::BindGroup,
    horizontal_uniforms: wgpu::Buffer,
    horizontal_bind_group: wgpu::BindGroup,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    texture_size: [u32; 2],
    direction: f32, // horizontal == 0.0 vertical = 1.0
    blur: f32,
}

impl Pipeline {
    /// Creates a new blur [`Pipeline`].
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.blur.sampler"),
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu.blur.uniform.bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Uniforms>() as u64,
                            ),
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
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu.blur.shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader/blur.wgsl"),
                )),
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu.blur.pipeline_layout"),
                bind_group_layouts: &[&uniform_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu.blur.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
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
            sampler,
            uniform_layout,
            layers: vec![],
            prepare_layer: 0,
        }
    }

    /// Returns the [`composite::Surface`] to render primitives to.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        radius: u16,
        bounds: Rectangle<u32>, //the physical layer bounds of the primitives
    ) -> &composite::Surface {
        if self.layers.len() <= self.prepare_layer {
            self.layers.push(Layer::new(
                device,
                format,
                radius,
                bounds,
                &self.sampler,
                &self.uniform_layout,
            ));
        }

        let layer = &mut self.layers[self.prepare_layer];
        layer.prepare(
            device,
            queue,
            &self.sampler,
            &self.uniform_layout,
            radius,
            bounds,
        );

        self.prepare_layer += 1;

        &layer.surface
    }

    pub fn surface(&self, layer: usize) -> Option<&composite::Surface> {
        self.layers.get(layer).map(|layer| &layer.surface)
    }

    pub fn end_frame(&mut self) {
        self.prepare_layer = 0;
    }

    //TODO optimizations to test
    // 1) test if writing all blur regions to a single large texture & picking regions vs 2 textures per blur layer
    // is more performant
    // 2) instance rendering if more than one set of blurred primitives per layer
    pub fn render(&mut self, layer: usize, encoder: &mut wgpu::CommandEncoder) {
        if let Some(layer) = self.layers.get(layer) {
            //first do a vertical render pass to the vertical texture
            {
                let vertical_view = layer
                    .vertical_texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut pass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu.blur.vertical_pass"),
                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {
                                view: &vertical_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(
                                        wgpu::Color::TRANSPARENT,
                                    ),
                                    store: true,
                                },
                            },
                        )],
                        depth_stencil_attachment: None,
                    });

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &layer.vertical_bind_group, &[]);
                pass.draw(0..6, 0..1);
            }

            //perform horizontal pass using vertical pass as input, render back to surface texture
            {
                let mut pass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu.blur.horizontal_pass"),
                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {
                                view: &layer.surface.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(
                                        wgpu::Color::TRANSPARENT,
                                    ),
                                    store: true,
                                },
                            },
                        )],
                        depth_stencil_attachment: None,
                    });

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &layer.horizontal_bind_group, &[]);
                pass.draw(0..6, 0..1);
            }

            //done, now we can composite this texture later!
        }
    }
}

impl Layer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        blur: u16,
        bounds: Rectangle<u32>,
        sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let vertical_uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wpgu.blur.vertical.uniforms_buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let horizontal_uniforms =
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("iced_wpgu.blur.horizontal.uniforms_buffer"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        let (texture_size, _) = calc_uniforms(bounds, blur);

        let surface = composite::Surface::new(
            device,
            format,
            bounds,
            texture_size,
            "blur.src_texture",
        );

        let vertical_texture = composite::Surface::create_texture(
            device,
            format,
            texture_size,
            "blur.vertical_texture",
        );

        let vertical_view = vertical_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let vertical_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu.blur.vertical.uniform_bind_group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: vertical_uniforms.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &surface.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });

        let horizontal_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu.blur.horizontal.uniform_bind_group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: horizontal_uniforms.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &vertical_view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });

        Self {
            surface,
            vertical_texture,
            vertical_uniforms,
            vertical_bind_group,
            horizontal_uniforms,
            horizontal_bind_group,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
        blur: u16,
        bounds: Rectangle<u32>,
    ) {
        self.surface.clip = bounds;
        let (texture_size, radius) = calc_uniforms(bounds, blur);

        let surface_resized =
            self.surface
                .resize(device, texture_size, "blur.src_texture");

        if surface_resized {
            self.vertical_texture = composite::Surface::create_texture(
                device,
                self.vertical_texture.format(),
                texture_size,
                "blur.vertical_texture",
            );

            self.vertical_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wgpu.blur.vertical.uniform_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self
                                .vertical_uniforms
                                .as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &self.surface.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                });

            self.horizontal_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wgpu.blur.uniform_bind_group.horizontal"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self
                                .horizontal_uniforms
                                .as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &self.vertical_texture.create_view(
                                    &wgpu::TextureViewDescriptor::default(),
                                ),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                });
        }

        let texture_size = self.surface.texture_size();

        queue.write_buffer(
            &self.vertical_uniforms,
            0,
            bytemuck::bytes_of(&Uniforms {
                texture_size: [texture_size.width, texture_size.height],
                direction: 1.0, //vertical
                blur: radius,
            }),
        );

        queue.write_buffer(
            &self.horizontal_uniforms,
            0,
            bytemuck::bytes_of(&Uniforms {
                texture_size: [texture_size.width, texture_size.height],
                direction: 0.0, //horizontal
                blur: radius,
            }),
        );
    }
}

/// Downsample the texture bounds based on blur `radius`.
fn calc_uniforms(bounds: Rectangle<u32>, radius: u16) -> (Size<u32>, f32) {
    let area = bounds.width as f32 * bounds.height as f32;

    // max amount of pixels before we begin to experience some lag
    const MAX_PIXELS: f32 = 250_000.0;

    if area > MAX_PIXELS {
        let ratio = f32::sqrt(MAX_PIXELS / area);

        (
            Size::new(
                (bounds.width as f32 * ratio) as u32,
                (bounds.height as f32 * ratio) as u32,
            ),
            radius as f32 * ratio,
        )
    } else {
        (Size::new(bounds.width, bounds.height), radius as f32)
    }
}
