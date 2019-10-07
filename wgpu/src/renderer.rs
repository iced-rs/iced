use crate::{quad, Background, Primitive, Quad, Transformation};
use iced_native::{renderer::Debugger, Color, Layout, Point, Widget};

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
    pub fn new<W: HasRawWindowHandle>(window: &W) -> Self {
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

    pub fn target(&self, width: u16, height: u16) -> Target {
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

    pub fn draw(&mut self, target: &mut Target, primitive: &Primitive) {
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
            } => self.glyph_brush.borrow_mut().queue(Section {
                text: &content,
                screen_position: (bounds.x, bounds.y),
                bounds: (bounds.width, bounds.height),
                scale: wgpu_glyph::Scale { x: *size, y: *size },
                ..Default::default()
            }),
            Primitive::Quad { bounds, background } => {
                self.quads.push(Quad {
                    position: [bounds.x, bounds.y],
                    scale: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                });
            }
        }
    }
}

impl iced_native::Renderer for Renderer {
    // TODO: Add `MouseCursor` here (?)
    type Primitive = Primitive;
}

impl Debugger for Renderer {
    fn explain<Message>(
        &mut self,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        _color: Color,
    ) -> Self::Primitive {
        // TODO: Include a bordered box to display layout bounds
        widget.draw(self, layout, cursor_position)
    }
}
