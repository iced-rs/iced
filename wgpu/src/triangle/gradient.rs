use crate::buffers::dynamic;
use crate::settings;
use crate::triangle;
use encase::ShaderType;
use glam::{IVec4, Vec4};
use iced_graphics::gradient::Gradient;
use iced_graphics::Transformation;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    pub(super) uniform_buffer: dynamic::Buffer<Uniforms>,
    pub(super) storage_buffer: dynamic::Buffer<Storage>,
    color_stop_offset: i32,
    //Need to store these and then write them all at once
    //or else they will be padded to 256 and cause gaps in the storage buffer
    color_stops_pending_write: Storage,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

#[derive(Debug, ShaderType)]
pub(super) struct Uniforms {
    transform: glam::Mat4,
    //xy = start, zw = end
    direction: Vec4,
    //x = start stop, y = end stop, zw = padding
    stop_range: IVec4,
}

#[derive(Debug, ShaderType)]
pub(super) struct ColorStop {
    color: Vec4,
    offset: f32,
}

#[derive(ShaderType)]
pub(super) struct Storage {
    #[size(runtime)]
    pub color_stops: Vec<ColorStop>,
}

impl Pipeline {
    /// Creates a new [GradientPipeline] using `gradient.wgsl` shader.
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Self {
        let uniform_buffer = dynamic::Buffer::uniform(
            device,
            "iced_wgpu::triangle::gradient uniforms",
        );

        //Note: with a WASM target storage buffers are not supported. Will need to use UBOs & static
        // sized array (eg like the 32-sized array on OpenGL side right now) to make gradients work
        let storage_buffer = dynamic::Buffer::storage(
            device,
            "iced_wgpu::triangle::gradient storage",
        );

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle::gradient bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(Uniforms::min_size()),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: true,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: Some(Storage::min_size()),
                        },
                        count: None,
                    },
                ],
            });

        let bind_group = Pipeline::bind_group(
            device,
            uniform_buffer.raw(),
            storage_buffer.raw(),
            &bind_group_layout,
        );

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::triangle::gradient pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(
                    "iced_wgpu::triangle::gradient create shader module",
                ),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shader/gradient.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::triangle::gradient pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[triangle::vertex_buffer_layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[triangle::fragment_target(format)],
                }),
                primitive: triangle::primitive_state(),
                depth_stencil: None,
                multisample: triangle::multisample_state(antialiasing),
                multiview: None,
            });

        Self {
            pipeline,
            uniform_buffer,
            storage_buffer,
            color_stop_offset: 0,
            color_stops_pending_write: Storage {
                color_stops: vec![],
            },
            bind_group_layout,
            bind_group,
        }
    }

    /// Pushes a new gradient uniform to the CPU buffer.
    pub fn push(&mut self, transform: Transformation, gradient: &Gradient) {
        match gradient {
            Gradient::Linear(linear) => {
                let start_offset = self.color_stop_offset;
                let end_offset =
                    (linear.color_stops.len() as i32) + start_offset - 1;

                self.uniform_buffer.push(&Uniforms {
                    transform: transform.into(),
                    direction: Vec4::new(
                        linear.start.x,
                        linear.start.y,
                        linear.end.x,
                        linear.end.y,
                    ),
                    stop_range: IVec4::new(start_offset, end_offset, 0, 0),
                });

                self.color_stop_offset = end_offset + 1;

                let stops: Vec<ColorStop> = linear
                    .color_stops
                    .iter()
                    .map(|stop| ColorStop {
                        offset: stop.offset,
                        color: Vec4::new(
                            stop.color.r,
                            stop.color.g,
                            stop.color.b,
                            stop.color.a,
                        ),
                    })
                    .collect();

                self.color_stops_pending_write.color_stops.extend(stops);
            }
        }
    }

    fn bind_group(
        device: &wgpu::Device,
        uniform_buffer: &wgpu::Buffer,
        storage_buffer: &wgpu::Buffer,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::triangle::gradient bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: uniform_buffer,
                            offset: 0,
                            size: Some(Uniforms::min_size()),
                        },
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: storage_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Writes the contents of the gradient CPU buffer to the GPU buffer, resizing the GPU buffer
    /// beforehand if necessary.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        //first write the pending color stops to the CPU buffer
        self.storage_buffer.push(&self.color_stops_pending_write);

        //resize buffers if needed
        let uniforms_resized = self.uniform_buffer.resize(device);
        let storage_resized = self.storage_buffer.resize(device);

        if uniforms_resized || storage_resized {
            //recreate bind groups if any buffers were resized
            self.bind_group = Pipeline::bind_group(
                device,
                self.uniform_buffer.raw(),
                self.storage_buffer.raw(),
                &self.bind_group_layout,
            );
        }

        //write to GPU
        self.uniform_buffer.write(device, staging_belt, encoder);
        self.storage_buffer.write(device, staging_belt, encoder);

        //cleanup
        self.color_stop_offset = 0;
        self.color_stops_pending_write.color_stops.clear();
    }

    /// Configures the current render pass to draw the gradient at its offset stored in the
    /// [DynamicBuffer] at [index].
    pub fn configure_render_pass<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        count: usize,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(
            0,
            &self.bind_group,
            &[self.uniform_buffer.offset_at_index(count)],
        )
    }
}
