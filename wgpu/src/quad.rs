mod gradient;
mod solid;

use gradient::Gradient;
use solid::Solid;

use crate::core::{Background, Rectangle};
use crate::graphics::color;
use crate::graphics::{self, Transformation};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use std::mem;

#[cfg(feature = "tracing")]
use tracing::info_span;

const INITIAL_INSTANCES: usize = 2_000;

#[derive(Debug)]
pub struct Pipeline {
    solid: solid::Pipeline,
    gradient: gradient::Pipeline,
    constant_layout: wgpu::BindGroupLayout,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    layers: Vec<Layer>,
    prepare_layer: usize,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipeline {
        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::quad uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<Uniforms>() as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
            });

        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad vertex buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad index buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertices,
            indices,
            solid: solid::Pipeline::new(device, format, &constant_layout),
            gradient: gradient::Pipeline::new(device, format, &constant_layout),
            layers: Vec::new(),
            prepare_layer: 0,
            constant_layout,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        quads: &Batch,
        transformation: Transformation,
        scale: f32,
    ) {
        if self.layers.len() <= self.prepare_layer {
            self.layers.push(Layer::new(device, &self.constant_layout));
        }

        let layer = &mut self.layers[self.prepare_layer];
        layer.prepare(device, queue, quads, transformation, scale);

        self.prepare_layer += 1;
    }

    pub fn render<'a>(
        &'a self,
        layer: usize,
        bounds: Rectangle<u32>,
        quads: &Batch,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if let Some(layer) = self.layers.get(layer) {
            render_pass.set_scissor_rect(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );
            render_pass.set_index_buffer(
                self.indices.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));

            let mut solid_offset = 0;
            let mut gradient_offset = 0;

            for (kind, count) in &quads.order {
                match kind {
                    Kind::Solid => {
                        self.solid.render(
                            render_pass,
                            &layer.constants,
                            &layer.solid,
                            solid_offset..(solid_offset + count),
                        );

                        solid_offset += count;
                    }
                    Kind::Gradient => {
                        self.gradient.render(
                            render_pass,
                            &layer.constants,
                            &layer.gradient,
                            gradient_offset..(gradient_offset + count),
                        );

                        gradient_offset += count;
                    }
                }
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.prepare_layer = 0;
    }
}

#[derive(Debug)]
struct Layer {
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
    solid: solid::Layer,
    gradient: gradient::Layer,
}

impl Layer {
    pub fn new(
        device: &wgpu::Device,
        constant_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let constants_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::quad uniforms buffer"),
            size: mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::quad uniforms bind group"),
            layout: constant_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: constants_buffer.as_entire_binding(),
            }],
        });

        Self {
            constants,
            constants_buffer,
            solid: solid::Layer::new(device),
            gradient: gradient::Layer::new(device),
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        quads: &Batch,
        transformation: Transformation,
        scale: f32,
    ) {
        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Quad", "PREPARE").entered();

        let uniforms = Uniforms::new(transformation, scale);

        queue.write_buffer(
            &self.constants_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );

        self.solid.prepare(device, queue, &quads.solids);
        self.gradient.prepare(device, queue, &quads.gradients);
    }
}

/// The properties of a quad.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Quad {
    /// The position of the [`Quad`].
    pub position: [f32; 2],

    /// The size of the [`Quad`].
    pub size: [f32; 2],

    /// The border color of the [`Quad`], in __linear RGB__.
    pub border_color: color::Packed,

    /// The border radii of the [`Quad`].
    pub border_radius: [f32; 4],

    /// The border width of the [`Quad`].
    pub border_width: f32,
}

/// A group of [`Quad`]s rendered together.
#[derive(Default, Debug)]
pub struct Batch {
    /// The solid quads of the [`Layer`].
    solids: Vec<Solid>,

    /// The gradient quads of the [`Layer`].
    gradients: Vec<Gradient>,

    /// The quad order of the [`Layer`]; stored as a tuple of the quad type & its count.
    order: Vec<(Kind, usize)>,
}

impl Batch {
    /// Returns true if there are no quads of any type in [`Quads`].
    pub fn is_empty(&self) -> bool {
        self.solids.is_empty() && self.gradients.is_empty()
    }

    /// Adds a [`Quad`] with the provided `Background` type to the quad [`Layer`].
    pub fn add(&mut self, quad: Quad, background: &Background) {
        let kind = match background {
            Background::Color(color) => {
                self.solids.push(Solid {
                    color: color::pack(*color),
                    quad,
                });

                Kind::Solid
            }
            Background::Gradient(gradient) => {
                self.gradients.push(Gradient {
                    gradient: graphics::gradient::pack(
                        gradient,
                        Rectangle::new(quad.position.into(), quad.size.into()),
                    ),
                    quad,
                });

                Kind::Gradient
            }
        };

        match self.order.last_mut() {
            Some((last_kind, count)) if kind == *last_kind => {
                *count += 1;
            }
            _ => {
                self.order.push((kind, 1));
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The kind of a quad.
enum Kind {
    /// A solid quad
    Solid,
    /// A gradient quad
    Gradient,
}

fn color_target_state(
    format: wgpu::TextureFormat,
) -> [Option<wgpu::ColorTargetState>; 1] {
    [Some(wgpu::ColorTargetState {
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
    })]
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    _position: [f32; 2],
}

impl Vertex {
    fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
            }],
        }
    }
}

const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const VERTICES: [Vertex; 4] = [
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
    // Uniforms must be aligned to their largest member,
    // this uses a mat4x4<f32> which aligns to 16, so align to that
    _padding: [f32; 3],
}

impl Uniforms {
    fn new(transformation: Transformation, scale: f32) -> Uniforms {
        Self {
            transform: *transformation.as_ref(),
            scale,
            _padding: [0.0; 3],
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
            scale: 1.0,
            _padding: [0.0; 3],
        }
    }
}
