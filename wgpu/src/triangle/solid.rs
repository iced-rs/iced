use crate::buffers::dynamic;
use crate::triangle;
use crate::{settings, Color};
use encase::ShaderType;
use glam::Vec4;
use iced_graphics::Transformation;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    pub(super) buffer: dynamic::Buffer<Uniforms>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

#[derive(Debug, Clone, Copy, ShaderType)]
pub(super) struct Uniforms {
    transform: glam::Mat4,
    color: Vec4,
}

impl Uniforms {
    pub fn new(transform: Transformation, color: Color) -> Self {
        let [r, g, b, a] = color.into_linear();

        Self {
            transform: transform.into(),
            color: Vec4::new(r, g, b, a),
        }
    }
}

impl Pipeline {
    /// Creates a new [SolidPipeline] using `solid.wgsl` shader.
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Self {
        let buffer = dynamic::Buffer::uniform(
            device,
            "iced_wgpu::triangle::solid uniforms",
        );

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::triangle::solid bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(Uniforms::min_size()),
                    },
                    count: None,
                }],
            });

        let bind_group =
            Pipeline::bind_group(device, buffer.raw(), &bind_group_layout);

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::triangle::solid pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu::triangle::solid create shader module"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("../shader/solid.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::triangle::solid pipeline"),
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
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    fn bind_group(
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::triangle::solid bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: Some(Uniforms::min_size()),
                }),
            }],
        })
    }

    /// Pushes a new solid uniform to the CPU buffer.
    pub fn push(&mut self, transform: Transformation, color: &Color) {
        self.buffer.push(&Uniforms::new(transform, *color));
    }

    /// Writes the contents of the solid CPU buffer to the GPU buffer, resizing the GPU buffer
    /// beforehand if necessary.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let uniforms_resized = self.buffer.resize(device);

        if uniforms_resized {
            self.bind_group = Pipeline::bind_group(
                device,
                self.buffer.raw(),
                &self.bind_group_layout,
            )
        }

        self.buffer.write(device, staging_belt, encoder);
    }

    /// Configures the current render pass to draw the solid at its offset stored in the
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
            &[self.buffer.offset_at_index(count)],
        )
    }
}
