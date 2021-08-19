//! This example showcases a widget that uses a custom shader to be rendered.
use std::cell::Cell;

use iced_graphics::{Backend, Defaults, Primitive, Renderer};
use iced_native::{
    event, layout, mouse, Background, Clipboard, Color, Element, Event, Hasher,
    Layout, Length, Point, Rectangle, Size, Widget,
};
use iced_wgpu::wgpu;

use bytemuck::{Pod, Zeroable};

use iced_graphics::Container;
use iced_graphics::Space;
use iced_winit::{slider, Align, Column, Command, Program, Row, Slider, Text};

pub struct CustomRenderedWidget<'a> {
    size: f32,
    bounds: &'a Cell<Rectangle>,
}

impl<'a> CustomRenderedWidget<'a> {
    pub fn new(size: f32, bounds: &'a Cell<Rectangle>) -> Self {
        Self { size, bounds }
    }
}

impl<'a, Message, B> Widget<Message, Renderer<B>> for CustomRenderedWidget<'a>
where
    B: Backend,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        _renderer: &Renderer<B>,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.size, self.size))
    }

    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _renderer: &Renderer<B>,
        _clipboard: &mut dyn Clipboard,
        _messages: &mut Vec<Message>,
    ) -> event::Status {
        event::Status::Ignored
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        self.size.to_bits().hash(state);
    }

    fn draw(
        &self,
        _renderer: &mut Renderer<B>,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        // We update the current bounds of this custom widget, so that the custom rendering
        // step will render it in the correct spot
        self.bounds.replace(layout.bounds());
        (
            Primitive::Quad {
                bounds: layout.bounds(),
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 0.0,
                border_width: 1.0,
                border_color: Color::WHITE,
            },
            mouse::Interaction::default(),
        )
    }
}

impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>>
    for CustomRenderedWidget<'a>
where
    B: Backend,
{
    fn into(self) -> Element<'a, Message, Renderer<B>> {
        Element::new(self)
    }
}

pub struct Controls {
    position: f32,
    sliders: [slider::State; 1],
    // A cell is used for interior mutability so that the draw call can update the bounds
    bounds: Cell<Rectangle>,
    pipeline: Pipeline,
}

#[derive(Debug, Clone)]
pub enum Message {
    PositionChanged(f32),
}

impl Controls {
    pub fn new(device: &wgpu::Device) -> Controls {
        Controls {
            position: 0.0,
            sliders: Default::default(),
            bounds: Default::default(),
            pipeline: Pipeline::new(device),
        }
    }

    pub fn draw_custom_widgets<'a>(
        &'a self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        physical_size: iced_graphics::Size,
    ) {
        self.pipeline.draw_custom_widgets(
            device,
            staging_belt,
            encoder,
            frame,
            physical_size,
            self.bounds.get(),
            self.position,
        );
    }
}

impl Program for Controls {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;
    type Clipboard = iced_winit::Clipboard;

    fn update(
        &mut self,
        message: Message,
        _clipboard: &mut iced_winit::Clipboard,
    ) -> Command<Message> {
        match message {
            Message::PositionChanged(position) => {
                self.position = position;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, iced_wgpu::Renderer> {
        let [r] = &mut self.sliders;

        let circle = CustomRenderedWidget::new(100.0, &self.bounds);

        let slider = Slider::new(r, 0.0..=1.0, self.position, move |r| {
            Message::PositionChanged(r)
        })
        .step(0.001)
        .width(Length::Fill);

        let fill_factor = u16::MAX / 2;

        Container::new(
            Row::new()
                .push(Space::with_width(Length::FillPortion(1)))
                .push(
                    Column::new()
                        .align_items(Align::Center)
                        .width(Length::FillPortion(2))
                        .spacing(10)
                        .push(
                            Row::new()
                                .push(Space::with_width(Length::FillPortion(
                                    (fill_factor as f32 * self.position) as u16
                                        + 1,
                                )))
                                .push(circle)
                                .push(Space::with_width(Length::FillPortion(
                                    (fill_factor as f32 * (1.0 - self.position))
                                        as u16
                                        + 1,
                                ))),
                        )
                        .push(Text::new("Position").color(Color::WHITE))
                        .push(slider),
                )
                .push(Space::with_width(Length::FillPortion(1))),
        )
        .center_y()
        .height(Length::Fill)
        .into()
    }
}

struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
}

impl Pipeline {
    fn new(device: &wgpu::Device) -> Pipeline {
        let vs_module = device
            .create_shader_module(&wgpu::include_spirv!("shader/vert.spv"));

        let fs_module = device
            .create_shader_module(&wgpu::include_spirv!("shader/frag.spv"));

        let uniform_size = std::mem::size_of::<Uniforms>() as u64;

        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            uniform_size as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                push_constant_ranges: &[],
                bind_group_layouts: &[&constant_layout],
            });

        let constants_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: uniform_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &constant_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: constants_buffer.as_entire_binding(),
            }],
        });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

        Pipeline {
            pipeline,
            constants,
            constants_buffer,
        }
    }

    pub fn draw_custom_widgets<'a>(
        &'a self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        physical_size: iced_graphics::Size,
        bounds: Rectangle,
        position: f32,
    ) {
        let snapped_bounds = bounds.snap();
        let min_x = snapped_bounds.x as f32 / physical_size.width * 2.0 - 1.0;
        let min_y = snapped_bounds.y as f32 / physical_size.height * 2.0 - 1.0;
        let bottom_left = [min_x, -min_y];
        let top_right = [
            min_x + snapped_bounds.width as f32 / physical_size.width * 2.0,
            -min_y - snapped_bounds.height as f32 / physical_size.height * 2.0,
        ];

        let uniforms = Uniforms {
            bottom_left,
            top_right,
            position,
            padding: [0.0],
        };

        let mut constants_buffer = staging_belt.write_buffer(
            encoder,
            &self.constants_buffer,
            0,
            wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64)
                .unwrap(),
            device,
        );

        constants_buffer.copy_from_slice(bytemuck::bytes_of(&uniforms));

        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Custom rendered widget pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.constants, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    bottom_left: [f32; 2],
    top_right: [f32; 2],
    position: f32,
    padding: [f32; 1],
}
