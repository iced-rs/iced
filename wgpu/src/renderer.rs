use crate::{quad, Primitive, Quad, Transformation};
use iced_native::{
    renderer::Debugger, renderer::Windowed, Background, Color, Layout,
    MouseCursor, Point, Widget,
};

use raw_window_handle::HasRawWindowHandle;
use wgpu::{
    Adapter, BackendBit, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Extensions, Limits, PowerPreference, Queue, RequestAdapterOptions, Surface,
    SwapChain, SwapChainDescriptor, TextureFormat, TextureUsage,
};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section};

use std::{cell::RefCell, rc::Rc};

mod button;
mod checkbox;
mod column;
mod image;
mod radio;
mod row;
mod slider;
mod text;

pub struct Renderer {
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    quad_pipeline: quad::Pipeline,

    quads: Vec<Quad>,
    glyph_brush: Rc<RefCell<GlyphBrush<'static, ()>>>,
}

pub struct Target {
    width: u16,
    height: u16,
    transformation: Transformation,
    swap_chain: SwapChain,
}

impl Renderer {
    fn new<W: HasRawWindowHandle>(window: &W) -> Self {
        let adapter = Adapter::request(&RequestAdapterOptions {
            power_preference: PowerPreference::LowPower,
            backends: BackendBit::all(),
        })
        .expect("Request adapter");

        let (mut device, queue) = adapter.request_device(&DeviceDescriptor {
            extensions: Extensions {
                anisotropic_filtering: false,
            },
            limits: Limits { max_bind_groups: 1 },
        });

        let surface = Surface::create(window);

        // TODO: Think about font loading strategy
        // Loading system fonts with fallback may be a good idea
        let font: &[u8] =
            include_bytes!("../../examples/resources/Roboto-Regular.ttf");

        let glyph_brush = GlyphBrushBuilder::using_font_bytes(font)
            .build(&mut device, TextureFormat::Bgra8UnormSrgb);

        let quad_pipeline = quad::Pipeline::new(&mut device);

        Self {
            surface,
            adapter,
            device,
            queue,
            quad_pipeline,

            quads: Vec::new(),
            glyph_brush: Rc::new(RefCell::new(glyph_brush)),
        }
    }

    fn target(&self, width: u16, height: u16) -> Target {
        Target {
            width,
            height,
            transformation: Transformation::orthographic(width, height),
            swap_chain: self.device.create_swap_chain(
                &self.surface,
                &SwapChainDescriptor {
                    usage: TextureUsage::OUTPUT_ATTACHMENT,
                    format: TextureFormat::Bgra8UnormSrgb,
                    width: u32::from(width),
                    height: u32::from(height),
                    present_mode: wgpu::PresentMode::Vsync,
                },
            ),
        }
    }

    fn draw(
        &mut self,
        (primitive, mouse_cursor): &(Primitive, MouseCursor),
        target: &mut Target,
    ) -> MouseCursor {
        log::debug!("Drawing");

        let frame = target.swap_chain.get_next_texture();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { todo: 0 });

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.draw_primitive(primitive);

        self.quad_pipeline.draw(
            &mut self.device,
            &mut encoder,
            &self.quads,
            target.transformation,
            &frame.view,
        );

        self.quads.clear();

        self.glyph_brush
            .borrow_mut()
            .draw_queued(
                &mut self.device,
                &mut encoder,
                &frame.view,
                u32::from(target.width),
                u32::from(target.height),
            )
            .expect("Draw text");

        self.queue.submit(&[encoder.finish()]);

        *mouse_cursor
    }

    fn draw_primitive(&mut self, primitive: &Primitive) {
        match primitive {
            Primitive::None => {}
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    self.draw_primitive(primitive)
                }
            }
            Primitive::Text {
                content,
                bounds,
                size,
                color,
                horizontal_alignment,
                vertical_alignment,
            } => {
                let x = match horizontal_alignment {
                    iced_native::text::HorizontalAlignment::Left => bounds.x,
                    iced_native::text::HorizontalAlignment::Center => {
                        bounds.x + bounds.width / 2.0
                    }
                    iced_native::text::HorizontalAlignment::Right => {
                        bounds.x + bounds.width
                    }
                };

                let y = match vertical_alignment {
                    iced_native::text::VerticalAlignment::Top => bounds.y,
                    iced_native::text::VerticalAlignment::Center => {
                        bounds.y + bounds.height / 2.0
                    }
                    iced_native::text::VerticalAlignment::Bottom => {
                        bounds.y + bounds.height
                    }
                };

                self.glyph_brush.borrow_mut().queue(Section {
                    text: &content,
                    screen_position: (x, y),
                    bounds: (bounds.width, bounds.height),
                    scale: wgpu_glyph::Scale { x: *size, y: *size },
                    color: color.into_linear(),
                    layout: wgpu_glyph::Layout::default()
                        .h_align(match horizontal_alignment {
                            iced_native::text::HorizontalAlignment::Left => {
                                wgpu_glyph::HorizontalAlign::Left
                            }
                            iced_native::text::HorizontalAlignment::Center => {
                                wgpu_glyph::HorizontalAlign::Center
                            }
                            iced_native::text::HorizontalAlignment::Right => {
                                wgpu_glyph::HorizontalAlign::Right
                            }
                        })
                        .v_align(match vertical_alignment {
                            iced_native::text::VerticalAlignment::Top => {
                                wgpu_glyph::VerticalAlign::Top
                            }
                            iced_native::text::VerticalAlignment::Center => {
                                wgpu_glyph::VerticalAlign::Center
                            }
                            iced_native::text::VerticalAlignment::Bottom => {
                                wgpu_glyph::VerticalAlign::Bottom
                            }
                        }),
                    ..Default::default()
                })
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
            } => {
                self.quads.push(Quad {
                    position: [bounds.x, bounds.y],
                    scale: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: u32::from(*border_radius),
                });
            }
        }
    }
}

impl iced_native::Renderer for Renderer {
    type Output = (Primitive, MouseCursor);
}

impl Windowed for Renderer {
    type Target = Target;

    fn new<W: HasRawWindowHandle>(window: &W) -> Self {
        Self::new(window)
    }

    fn target(&self, width: u16, height: u16) -> Target {
        self.target(width, height)
    }

    fn draw(
        &mut self,
        output: &Self::Output,
        target: &mut Target,
    ) -> MouseCursor {
        self.draw(output, target)
    }
}

impl Debugger for Renderer {
    fn explain<Message>(
        &mut self,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        _color: Color,
    ) -> Self::Output {
        // TODO: Include a bordered box to display layout bounds
        widget.draw(self, layout, cursor_position)
    }
}
