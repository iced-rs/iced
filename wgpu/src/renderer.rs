use crate::{
    quad, text, triangle, Defaults, Primitive, Quad, Settings, Target,
    Transformation,
};

#[cfg(any(feature = "image", feature = "svg"))]
use crate::image::{self, Image};

use iced_native::{
    layout, mouse, Background, Color, Font, HorizontalAlignment, Layout, Point,
    Rectangle, Size, Vector, VerticalAlignment, Widget,
};

mod widget;

/// A [`wgpu`] renderer.
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
#[derive(Debug)]
pub struct Renderer {
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
    triangle_pipeline: triangle::Pipeline,

    #[cfg(any(feature = "image", feature = "svg"))]
    image_pipeline: image::Pipeline,
}

struct Layer<'a> {
    bounds: Rectangle<u32>,
    quads: Vec<Quad>,
    meshes: Vec<(Vector, Rectangle<u32>, &'a triangle::Mesh2D)>,
    text: Vec<Text<'a>>,

    #[cfg(any(feature = "image", feature = "svg"))]
    images: Vec<Image>,
}

#[derive(Debug, Clone, Copy)]
pub struct Text<'a> {
    pub content: &'a str,
    pub bounds: Rectangle,
    pub color: [f32; 4],
    pub size: f32,
    pub font: Font,
    pub horizontal_alignment: HorizontalAlignment,
    pub vertical_alignment: VerticalAlignment,
}

impl<'a> Layer<'a> {
    pub fn new(bounds: Rectangle<u32>) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
            text: Vec::new(),
            meshes: Vec::new(),

            #[cfg(any(feature = "image", feature = "svg"))]
            images: Vec::new(),
        }
    }

    pub fn intersection(&self, rectangle: Rectangle) -> Option<Rectangle<u32>> {
        let layer_bounds: Rectangle<f32> = self.bounds.into();

        layer_bounds.intersection(&rectangle).map(Into::into)
    }
}

impl Renderer {
    /// Creates a new [`Renderer`].
    ///
    /// [`Renderer`]: struct.Renderer.html
    pub fn new(device: &wgpu::Device, settings: Settings) -> Self {
        let text_pipeline =
            text::Pipeline::new(device, settings.format, settings.default_font);
        let quad_pipeline = quad::Pipeline::new(device, settings.format);
        let triangle_pipeline = triangle::Pipeline::new(
            device,
            settings.format,
            settings.antialiasing,
        );

        #[cfg(any(feature = "image", feature = "svg"))]
        let image_pipeline = image::Pipeline::new(device, settings.format);

        Self {
            quad_pipeline,
            text_pipeline,
            triangle_pipeline,

            #[cfg(any(feature = "image", feature = "svg"))]
            image_pipeline,
        }
    }

    /// Draws the provided primitives in the given [`Target`].
    ///
    /// The text provided as overlay will be renderer on top of the primitives.
    /// This is useful for rendering debug information.
    ///
    /// [`Target`]: struct.Target.html
    pub fn draw<T: AsRef<str>>(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: Target<'_>,
        (primitive, mouse_interaction): &(Primitive, mouse::Interaction),
        scale_factor: f64,
        overlay: &[T],
    ) -> mouse::Interaction {
        log::debug!("Drawing");

        let (width, height) = target.viewport.dimensions();
        let scale_factor = scale_factor as f32;
        let transformation = target.viewport.transformation();

        let mut layers = Vec::new();

        layers.push(Layer::new(Rectangle {
            x: 0,
            y: 0,
            width,
            height,
        }));

        self.draw_primitive(Vector::new(0.0, 0.0), primitive, &mut layers);
        self.draw_overlay(overlay, &mut layers);

        for layer in layers {
            self.flush(
                device,
                scale_factor,
                transformation,
                &layer,
                encoder,
                target.texture,
                width,
                height,
            );
        }

        #[cfg(any(feature = "image", feature = "svg"))]
        self.image_pipeline.trim_cache();

        *mouse_interaction
    }

    fn draw_primitive<'a>(
        &mut self,
        translation: Vector,
        primitive: &'a Primitive,
        layers: &mut Vec<Layer<'a>>,
    ) {
        match primitive {
            Primitive::None => {}
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    self.draw_primitive(translation, primitive, layers)
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
                let layer = layers.last_mut().unwrap();

                layer.text.push(Text {
                    content,
                    bounds: *bounds + translation,
                    size: *size,
                    color: color.into_linear(),
                    font: *font,
                    horizontal_alignment: *horizontal_alignment,
                    vertical_alignment: *vertical_alignment,
                });
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let layer = layers.last_mut().unwrap();

                // TODO: Move some of these computations to the GPU (?)
                layer.quads.push(Quad {
                    position: [
                        bounds.x + translation.x,
                        bounds.y + translation.y,
                    ],
                    scale: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: *border_radius as f32,
                    border_width: *border_width as f32,
                    border_color: border_color.into_linear(),
                });
            }
            Primitive::Mesh2D { size, buffers } => {
                let layer = layers.last_mut().unwrap();

                // Only draw visible content
                if let Some(clip_bounds) = layer.intersection(Rectangle::new(
                    Point::new(translation.x, translation.y),
                    *size,
                )) {
                    layer.meshes.push((
                        translation,
                        clip_bounds.into(),
                        buffers,
                    ));
                }
            }
            Primitive::Clip {
                bounds,
                offset,
                content,
            } => {
                let layer = layers.last_mut().unwrap();

                // Only draw visible content
                if let Some(clip_bounds) =
                    layer.intersection(*bounds + translation)
                {
                    let clip_layer = Layer::new(clip_bounds.into());
                    let new_layer = Layer::new(layer.bounds);

                    layers.push(clip_layer);
                    self.draw_primitive(
                        translation
                            - Vector::new(offset.x as f32, offset.y as f32),
                        content,
                        layers,
                    );
                    layers.push(new_layer);
                }
            }
            Primitive::Translate {
                translation: new_translation,
                content,
            } => {
                self.draw_primitive(
                    translation + *new_translation,
                    &content,
                    layers,
                );
            }

            Primitive::Cached { cache } => {
                self.draw_primitive(translation, &cache, layers);
            }

            #[cfg(feature = "image")]
            Primitive::Image { handle, bounds } => {
                let layer = layers.last_mut().unwrap();

                layer.images.push(Image {
                    handle: image::Handle::Raster(handle.clone()),
                    position: [
                        bounds.x + translation.x,
                        bounds.y + translation.y,
                    ],
                    size: [bounds.width, bounds.height],
                });
            }
            #[cfg(not(feature = "image"))]
            Primitive::Image { .. } => {}

            #[cfg(feature = "svg")]
            Primitive::Svg { handle, bounds } => {
                let layer = layers.last_mut().unwrap();

                layer.images.push(Image {
                    handle: image::Handle::Vector(handle.clone()),
                    position: [
                        bounds.x + translation.x,
                        bounds.y + translation.y,
                    ],
                    size: [bounds.width, bounds.height],
                });
            }
            #[cfg(not(feature = "svg"))]
            Primitive::Svg { .. } => {}
        }
    }

    fn draw_overlay<'a, T: AsRef<str>>(
        &mut self,
        lines: &'a [T],
        layers: &mut Vec<Layer<'a>>,
    ) {
        let first = layers.first().unwrap();
        let mut overlay = Layer::new(first.bounds);

        for (i, line) in lines.iter().enumerate() {
            let text = Text {
                content: line.as_ref(),
                bounds: Rectangle::new(
                    Point::new(11.0, 11.0 + 25.0 * i as f32),
                    Size::INFINITY,
                ),
                color: [0.9, 0.9, 0.9, 1.0],
                size: 20.0,
                font: Font::Default,
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
            };

            overlay.text.push(text);

            overlay.text.push(Text {
                bounds: text.bounds + Vector::new(-1.0, -1.0),
                color: [0.0, 0.0, 0.0, 1.0],
                ..text
            });
        }

        layers.push(overlay);
    }

    fn flush(
        &mut self,
        device: &wgpu::Device,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer<'_>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
    ) {
        let bounds = layer.bounds * scale_factor;

        if !layer.quads.is_empty() {
            self.quad_pipeline.draw(
                device,
                encoder,
                &layer.quads,
                transformation,
                scale_factor,
                bounds,
                target,
            );
        }

        if !layer.meshes.is_empty() {
            let scaled = transformation
                * Transformation::scale(scale_factor, scale_factor);

            self.triangle_pipeline.draw(
                device,
                encoder,
                target,
                target_width,
                target_height,
                scaled,
                scale_factor,
                &layer.meshes,
            );
        }

        #[cfg(any(feature = "image", feature = "svg"))]
        {
            if !layer.images.is_empty() {
                let scaled = transformation
                    * Transformation::scale(scale_factor, scale_factor);

                self.image_pipeline.draw(
                    device,
                    encoder,
                    &layer.images,
                    scaled,
                    bounds,
                    target,
                    scale_factor,
                );
            }
        }

        if !layer.text.is_empty() {
            for text in layer.text.iter() {
                // Target physical coordinates directly to avoid blurry text
                let text = wgpu_glyph::Section {
                    // TODO: We `round` here to avoid rerasterizing text when
                    // its position changes slightly. This can make text feel a
                    // bit "jumpy". We may be able to do better once we improve
                    // our text rendering/caching pipeline.
                    screen_position: (
                        (text.bounds.x * scale_factor).round(),
                        (text.bounds.y * scale_factor).round(),
                    ),
                    // TODO: Fix precision issues with some scale factors.
                    //
                    // The `ceil` here can cause some words to render on the
                    // same line when they should not.
                    //
                    // Ideally, `wgpu_glyph` should be able to compute layout
                    // using logical positions, and then apply the proper
                    // scaling when rendering. This would ensure that both
                    // measuring and rendering follow the same layout rules.
                    bounds: (
                        (text.bounds.width * scale_factor).ceil(),
                        (text.bounds.height * scale_factor).ceil(),
                    ),
                    text: vec![wgpu_glyph::Text {
                        text: text.content,
                        scale: wgpu_glyph::ab_glyph::PxScale {
                            x: text.size * scale_factor,
                            y: text.size * scale_factor,
                        },
                        font_id: self.text_pipeline.find_font(text.font),
                        extra: wgpu_glyph::Extra {
                            color: text.color,
                            z: 0.0,
                        },
                    }],
                    layout: wgpu_glyph::Layout::default()
                        .h_align(match text.horizontal_alignment {
                            HorizontalAlignment::Left => {
                                wgpu_glyph::HorizontalAlign::Left
                            }
                            HorizontalAlignment::Center => {
                                wgpu_glyph::HorizontalAlign::Center
                            }
                            HorizontalAlignment::Right => {
                                wgpu_glyph::HorizontalAlign::Right
                            }
                        })
                        .v_align(match text.vertical_alignment {
                            VerticalAlignment::Top => {
                                wgpu_glyph::VerticalAlign::Top
                            }
                            VerticalAlignment::Center => {
                                wgpu_glyph::VerticalAlign::Center
                            }
                            VerticalAlignment::Bottom => {
                                wgpu_glyph::VerticalAlign::Bottom
                            }
                        }),
                    ..Default::default()
                };

                self.text_pipeline.queue(text);
            }

            self.text_pipeline.draw_queued(
                device,
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
    type Output = (Primitive, mouse::Interaction);
    type Defaults = Defaults;

    fn layout<'a, Message>(
        &mut self,
        element: &iced_native::Element<'a, Message, Self>,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        let node = element.layout(self, limits);

        self.text_pipeline.clear_measurement_cache();

        node
    }
}

impl layout::Debugger for Renderer {
    fn explain<Message>(
        &mut self,
        defaults: &Defaults,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        color: Color,
    ) -> Self::Output {
        let mut primitives = Vec::new();
        let (primitive, cursor) =
            widget.draw(self, defaults, layout, cursor_position);

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
    primitives.push(Primitive::Quad {
        bounds: layout.bounds(),
        background: Background::Color(Color::TRANSPARENT),
        border_radius: 0,
        border_width: 1,
        border_color: [0.6, 0.6, 0.6, 0.5].into(),
    });

    for child in layout.children() {
        explain_layout(child, color, primitives);
    }
}
