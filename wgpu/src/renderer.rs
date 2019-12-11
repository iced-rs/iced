use crate::{quad, text, Image, Primitive, Quad, Svg, Transformation};
use iced_native::{
    renderer::{Debugger, Windowed},
    Background, Color, Layout, MouseCursor, Point, Rectangle, Vector, Widget,
};

use wgpu::{
    Adapter, BackendBit, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Extensions, Limits, PowerPreference, Queue, RequestAdapterOptions,
};

mod target;
mod widget;

pub use target::Target;

/// A [`wgpu`] renderer.
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
#[derive(Debug)]
pub struct Renderer {
    device: Device,
    queue: Queue,
    quad_pipeline: quad::Pipeline,
    image_pipeline: crate::image::Pipeline,
    svg_pipeline: crate::svg::Pipeline,
    text_pipeline: text::Pipeline,
}

struct Layer<'a> {
    bounds: Rectangle<u32>,
    offset: Vector<u32>,
    quads: Vec<Quad>,
    images: Vec<Image>,
    svgs: Vec<Svg>,
    text: Vec<wgpu_glyph::Section<'a>>,
}

impl<'a> Layer<'a> {
    pub fn new(bounds: Rectangle<u32>, offset: Vector<u32>) -> Self {
        Self {
            bounds,
            offset,
            quads: Vec::new(),
            images: Vec::new(),
            svgs: Vec::new(),
            text: Vec::new(),
        }
    }
}

impl Renderer {
    fn new() -> Self {
        let adapter = Adapter::request(&RequestAdapterOptions {
            power_preference: PowerPreference::Default,
            backends: BackendBit::all(),
        })
        .expect("Request adapter");

        let (mut device, queue) = adapter.request_device(&DeviceDescriptor {
            extensions: Extensions {
                anisotropic_filtering: false,
            },
            limits: Limits { max_bind_groups: 2 },
        });

        let text_pipeline = text::Pipeline::new(&mut device);
        let quad_pipeline = quad::Pipeline::new(&mut device);
        let image_pipeline = crate::image::Pipeline::new(&mut device);
        let svg_pipeline = crate::svg::Pipeline::new(&mut device);

        Self {
            device,
            queue,
            quad_pipeline,
            image_pipeline,
            svg_pipeline,
            text_pipeline,
        }
    }

    fn draw<T: AsRef<str>>(
        &mut self,
        (primitive, mouse_cursor): &(Primitive, MouseCursor),
        overlay: &[T],
        target: &mut Target,
    ) -> MouseCursor {
        log::debug!("Drawing");

        let (width, height) = target.dimensions();
        let dpi = target.dpi();
        let transformation = target.transformation();
        let frame = target.next_frame();

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

        let mut layers = Vec::new();

        layers.push(Layer::new(
            Rectangle {
                x: 0,
                y: 0,
                width: u32::from(width),
                height: u32::from(height),
            },
            Vector::new(0, 0),
        ));

        self.draw_primitive(primitive, &mut layers);
        self.draw_overlay(overlay, &mut layers);

        for layer in layers {
            self.flush(dpi, transformation, &layer, &mut encoder, &frame.view);
        }

        self.queue.submit(&[encoder.finish()]);
        self.image_pipeline.trim_cache();
        self.svg_pipeline.trim_cache();

        *mouse_cursor
    }

    fn draw_primitive<'a>(
        &mut self,
        primitive: &'a Primitive,
        layers: &mut Vec<Layer<'a>>,
    ) {
        let layer = layers.last_mut().unwrap();

        match primitive {
            Primitive::None => {}
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    self.draw_primitive(primitive, layers)
                }
            }
            Primitive::Text {
                content,
                bounds,
                size,
                color,
                font,
                horizontal_alignment,
                vertical_alignment,
            } => {
                let x = match horizontal_alignment {
                    iced_native::HorizontalAlignment::Left => bounds.x,
                    iced_native::HorizontalAlignment::Center => {
                        bounds.x + bounds.width / 2.0
                    }
                    iced_native::HorizontalAlignment::Right => {
                        bounds.x + bounds.width
                    }
                };

                let y = match vertical_alignment {
                    iced_native::VerticalAlignment::Top => bounds.y,
                    iced_native::VerticalAlignment::Center => {
                        bounds.y + bounds.height / 2.0
                    }
                    iced_native::VerticalAlignment::Bottom => {
                        bounds.y + bounds.height
                    }
                };

                layer.text.push(wgpu_glyph::Section {
                    text: &content,
                    screen_position: (
                        x - layer.offset.x as f32,
                        y - layer.offset.y as f32,
                    ),
                    bounds: (bounds.width, bounds.height),
                    scale: wgpu_glyph::Scale { x: *size, y: *size },
                    color: color.into_linear(),
                    font_id: self.text_pipeline.find_font(*font),
                    layout: wgpu_glyph::Layout::default()
                        .h_align(match horizontal_alignment {
                            iced_native::HorizontalAlignment::Left => {
                                wgpu_glyph::HorizontalAlign::Left
                            }
                            iced_native::HorizontalAlignment::Center => {
                                wgpu_glyph::HorizontalAlign::Center
                            }
                            iced_native::HorizontalAlignment::Right => {
                                wgpu_glyph::HorizontalAlign::Right
                            }
                        })
                        .v_align(match vertical_alignment {
                            iced_native::VerticalAlignment::Top => {
                                wgpu_glyph::VerticalAlign::Top
                            }
                            iced_native::VerticalAlignment::Center => {
                                wgpu_glyph::VerticalAlign::Center
                            }
                            iced_native::VerticalAlignment::Bottom => {
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
                // TODO: Move some of this computations to the GPU (?)
                layer.quads.push(Quad {
                    position: [
                        bounds.x - layer.offset.x as f32,
                        bounds.y - layer.offset.y as f32,
                    ],
                    scale: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: *border_radius as f32,
                });
            }
            Primitive::Image { handle, bounds } => {
                layer.images.push(Image {
                    handle: handle.clone(),
                    position: [bounds.x, bounds.y],
                    scale: [bounds.width, bounds.height],
                });
            }
            Primitive::Svg { handle, bounds } => layer.svgs.push(Svg {
                handle: handle.clone(),
                position: [bounds.x, bounds.y],
                scale: [bounds.width, bounds.height],
            }),
            Primitive::Clip {
                bounds,
                offset,
                content,
            } => {
                let x = bounds.x - layer.offset.x as f32;
                let y = bounds.y - layer.offset.y as f32;
                let width = (bounds.width + x).min(bounds.width);
                let height = (bounds.height + y).min(bounds.height);

                // Only draw visible content on-screen
                // TODO: Also, check for parent layer bounds to avoid further
                // drawing in some circumstances.
                if width > 0.0 && height > 0.0 {
                    let clip_layer = Layer::new(
                        Rectangle {
                            x: x.max(0.0).floor() as u32,
                            y: y.max(0.0).floor() as u32,
                            width: width.ceil() as u32,
                            height: height.ceil() as u32,
                        },
                        layer.offset + *offset,
                    );

                    let new_layer = Layer::new(layer.bounds, layer.offset);

                    layers.push(clip_layer);
                    self.draw_primitive(content, layers);
                    layers.push(new_layer);
                }
            }
        }
    }

    fn draw_overlay<'a, T: AsRef<str>>(
        &mut self,
        lines: &'a [T],
        layers: &mut Vec<Layer<'a>>,
    ) {
        let first = layers.first().unwrap();
        let mut overlay = Layer::new(first.bounds, Vector::new(0, 0));

        let font_id = self.text_pipeline.overlay_font();
        let scale = wgpu_glyph::Scale { x: 20.0, y: 20.0 };

        for (i, line) in lines.iter().enumerate() {
            overlay.text.push(wgpu_glyph::Section {
                text: line.as_ref(),
                screen_position: (11.0, 11.0 + 25.0 * i as f32),
                color: [0.9, 0.9, 0.9, 1.0],
                scale,
                font_id,
                ..wgpu_glyph::Section::default()
            });

            overlay.text.push(wgpu_glyph::Section {
                text: line.as_ref(),
                screen_position: (10.0, 10.0 + 25.0 * i as f32),
                color: [0.0, 0.0, 0.0, 1.0],
                scale,
                font_id,
                ..wgpu_glyph::Section::default()
            });
        }

        layers.push(overlay);
    }

    fn flush(
        &mut self,
        dpi: f32,
        transformation: Transformation,
        layer: &Layer<'_>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let bounds = layer.bounds * dpi;

        if layer.quads.len() > 0 {
            self.quad_pipeline.draw(
                &mut self.device,
                encoder,
                &layer.quads,
                transformation,
                dpi,
                bounds,
                target,
            );
        }

        if layer.images.len() > 0 {
            let translated_and_scaled = transformation
                * Transformation::scale(dpi, dpi)
                * Transformation::translate(
                    -(layer.offset.x as f32),
                    -(layer.offset.y as f32),
                );

            self.image_pipeline.draw(
                &mut self.device,
                encoder,
                &layer.images,
                translated_and_scaled,
                bounds,
                target,
            );
        }

        if layer.svgs.len() > 0 {
            let translated = transformation
                * Transformation::translate(
                    -(layer.offset.x as f32),
                    -(layer.offset.y as f32),
                );

            self.svg_pipeline.draw(
                &mut self.device,
                encoder,
                &layer.svgs,
                translated,
                bounds,
                target,
                dpi,
            );
        }

        if layer.text.len() > 0 {
            for text in layer.text.iter() {
                // Target physical coordinates directly to avoid blurry text
                let text = wgpu_glyph::Section {
                    // TODO: We `round` here to avoid rerasterizing text when
                    // its position changes slightly. This can make text feel a
                    // bit "jumpy". We may be able to do better once we improve
                    // our text rendering/caching pipeline.
                    screen_position: (
                        (text.screen_position.0 * dpi).round(),
                        (text.screen_position.1 * dpi).round(),
                    ),
                    // TODO: Fix precision issues with some DPI factors.
                    //
                    // The `ceil` here can cause some words to render on the
                    // same line when they should not.
                    //
                    // Ideally, `wgpu_glyph` should be able to compute layout
                    // using logical positions, and then apply the proper
                    // DPI scaling. This would ensure that both measuring and
                    // rendering follow the same layout rules.
                    bounds: (
                        (text.bounds.0 * dpi).ceil(),
                        (text.bounds.1 * dpi).ceil(),
                    ),
                    scale: wgpu_glyph::Scale {
                        x: text.scale.x * dpi,
                        y: text.scale.y * dpi,
                    },
                    ..*text
                };

                self.text_pipeline.queue(text);
            }

            self.text_pipeline.draw_queued(
                &mut self.device,
                encoder,
                target,
                transformation,
                wgpu_glyph::Region {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
            );
        }
    }
}

impl iced_native::Renderer for Renderer {
    type Output = (Primitive, MouseCursor);

    fn layout<'a, Message>(
        &mut self,
        element: &iced_native::Element<'a, Message, Self>,
    ) -> iced_native::layout::Node {
        let node = element.layout(self, &iced_native::layout::Limits::NONE);

        self.text_pipeline.clear_measurement_cache();

        node
    }
}

impl Windowed for Renderer {
    type Target = Target;

    fn new() -> Self {
        Self::new()
    }

    fn draw<T: AsRef<str>>(
        &mut self,
        output: &Self::Output,
        overlay: &[T],
        target: &mut Target,
    ) -> MouseCursor {
        self.draw(output, overlay, target)
    }
}

impl Debugger for Renderer {
    fn explain<Message>(
        &mut self,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        color: Color,
    ) -> Self::Output {
        let mut primitives = Vec::new();
        let (primitive, cursor) = widget.draw(self, layout, cursor_position);

        explain_layout(layout, color, &mut primitives);
        primitives.push(primitive);

        (Primitive::Group { primitives }, cursor)
    }
}

fn explain_layout(
    layout: Layout<'_>,
    color: Color,
    primitives: &mut Vec<Primitive>,
) {
    // TODO: Draw borders instead
    primitives.push(Primitive::Quad {
        bounds: layout.bounds(),
        background: Background::Color([0.0, 0.0, 0.0, 0.05].into()),
        border_radius: 0,
    });

    for child in layout.children() {
        explain_layout(child, color, primitives);
    }
}
