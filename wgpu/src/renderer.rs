use crate::Primitive;
use iced_native::{renderer::Debugger, Color, Layout, Point, Widget};

use raw_window_handle::HasRawWindowHandle;
use wgpu::{
    Adapter, CommandEncoderDescriptor, Device, DeviceDescriptor, Extensions,
    Instance, Limits, PowerPreference, RequestAdapterOptions, Surface,
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
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    glyph_brush: Rc<RefCell<GlyphBrush<'static, ()>>>,
}

pub struct Target {
    width: u16,
    height: u16,
    swap_chain: SwapChain,
}

impl Renderer {
    pub fn new<W: HasRawWindowHandle>(window: &W) -> Self {
        let instance = Instance::new();

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::LowPower,
        });

        let mut device = adapter.request_device(&DeviceDescriptor {
            extensions: Extensions {
                anisotropic_filtering: false,
            },
            limits: Limits { max_bind_groups: 1 },
        });

        let surface = instance.create_surface(window.raw_window_handle());

        // TODO: Think about font loading strategy
        // Loading system fonts with fallback may be a good idea
        let font: &[u8] =
            include_bytes!("../../examples/resources/Roboto-Regular.ttf");

        let glyph_brush = GlyphBrushBuilder::using_font_bytes(font)
            .build(&mut device, TextureFormat::Bgra8UnormSrgb);

        Self {
            instance,
            surface,
            adapter,
            device,
            glyph_brush: Rc::new(RefCell::new(glyph_brush)),
        }
    }

    pub fn target(&self, width: u16, height: u16) -> Target {
        Target {
            width,
            height,
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

        self.device.get_queue().submit(&[encoder.finish()]);
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
            Primitive::Box { bounds, background } => {
                // TODO: Batch boxes and draw them all at once
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
