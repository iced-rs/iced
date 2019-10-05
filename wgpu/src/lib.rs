use iced_native::{
    button, checkbox, column, image, radio, renderer::Debugger, row, slider,
    text, Button, Checkbox, Color, Column, Image, Layout, Node, Point, Radio,
    Rectangle, Row, Slider, Style, Text, Widget,
};

use raw_window_handle::HasRawWindowHandle;
use wgpu::{
    Adapter, CommandEncoderDescriptor, Device, DeviceDescriptor, Extensions,
    Instance, Limits, PowerPreference, RequestAdapterOptions, Surface,
    SwapChain, SwapChainDescriptor, TextureFormat, TextureUsage,
};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, GlyphCruncher, Section};

use std::f32;
use std::{cell::RefCell, rc::Rc};

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

        let font: &[u8] =
            include_bytes!("../../examples/tour/resources/Roboto-Regular.ttf");

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
        }
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    None,
    Group {
        primitives: Vec<Primitive>,
    },
    Text {
        content: String,
        bounds: Rectangle,
        size: f32,
    },
}

impl iced_native::Renderer for Renderer {
    type Primitive = Primitive;
}

impl column::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        column: &Column<'_, Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::Group {
            primitives: column
                .children
                .iter()
                .zip(layout.children())
                .map(|(child, layout)| {
                    child.draw(self, layout, cursor_position)
                })
                .collect(),
        }
    }
}

impl row::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        row: &Row<'_, Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::Group {
            primitives: row
                .children
                .iter()
                .zip(layout.children())
                .map(|(child, layout)| {
                    child.draw(self, layout, cursor_position)
                })
                .collect(),
        }
    }
}

impl text::Renderer for Renderer {
    fn node(&self, text: &Text) -> Node {
        let glyph_brush = self.glyph_brush.clone();
        let content = text.content.clone();

        // TODO: Investigate why stretch tries to measure this MANY times
        // with every ancestor's bounds.
        // Bug? Using the library wrong? I should probably open an issue on
        // the stretch repository.
        // I noticed that the first measure is the one that matters in
        // practice. Here, we use a RefCell to store the cached measurement.
        let measure = RefCell::new(None);
        let size = text.size.map(f32::from).unwrap_or(20.0);

        let style = Style::default().width(text.width);

        iced_native::Node::with_measure(style, move |bounds| {
            let mut measure = measure.borrow_mut();

            if measure.is_none() {
                let bounds = (
                    match bounds.width {
                        iced_native::Number::Undefined => f32::INFINITY,
                        iced_native::Number::Defined(w) => w,
                    },
                    match bounds.height {
                        iced_native::Number::Undefined => f32::INFINITY,
                        iced_native::Number::Defined(h) => h,
                    },
                );

                let text = Section {
                    text: &content,
                    scale: wgpu_glyph::Scale { x: size, y: size },
                    bounds,
                    ..Default::default()
                };

                let (width, height) = if let Some(bounds) =
                    glyph_brush.borrow_mut().glyph_bounds(&text)
                {
                    (bounds.width(), bounds.height())
                } else {
                    (0.0, 0.0)
                };

                let size = iced_native::Size { width, height };

                // If the text has no width boundary we avoid caching as the
                // layout engine may just be measuring text in a row.
                if bounds.0 == f32::INFINITY {
                    return size;
                } else {
                    *measure = Some(size);
                }
            }

            measure.unwrap()
        })
    }

    fn draw(&mut self, text: &Text, layout: Layout<'_>) -> Self::Primitive {
        Primitive::Text {
            content: text.content.clone(),
            size: f32::from(text.size.unwrap_or(20)),
            bounds: layout.bounds(),
        }
    }
}

impl checkbox::Renderer for Renderer {
    fn node<Message>(&mut self, _checkbox: &Checkbox<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _checkbox: &Checkbox<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::None
    }
}

impl radio::Renderer for Renderer {
    fn node<Message>(&mut self, _checkbox: &Radio<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _radio: &Radio<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::None
    }
}

impl slider::Renderer for Renderer {
    fn node<Message>(&self, _slider: &Slider<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _slider: &Slider<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::None
    }
}

impl image::Renderer<&str> for Renderer {
    fn node(&mut self, _image: &Image<&str>) -> Node {
        Node::new(Style::default())
    }

    fn draw(
        &mut self,
        _checkbox: &Image<&str>,
        _layout: Layout<'_>,
    ) -> Self::Primitive {
        Primitive::None
    }
}

impl button::Renderer for Renderer {
    fn node<Message>(&self, _button: &Button<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _button: &Button<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::None
    }
}

impl Debugger for Renderer {
    fn explain<Message>(
        &mut self,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        _color: Color,
    ) -> Self::Primitive {
        widget.draw(self, layout, cursor_position)
    }
}
