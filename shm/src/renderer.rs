use crate::{
    quad, text, Defaults, Primitive, Quad, Settings, Target, Transformation,
};

#[cfg(any(feature = "image", feature = "svg"))]
use crate::image::{self, Image};

use iced_native::{
    layout, Background, Color, Layout, mouse::Interaction, Point, Rectangle, Vector,
    Widget,
};

mod widget;

/// A renderer.
#[derive(Debug)]
pub struct Renderer {
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
}

struct Layer<'t> {
    bounds: Rectangle<u32>,
    quads: Vec<Quad>,
    text: Vec<text::Section<'t>>,

    #[cfg(any(feature = "image", feature = "svg"))]
    images: Vec<Image>,
}

impl<'t> Layer<'t> {
    pub fn new(bounds: Rectangle<u32>) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
            text: Vec::new(),

            #[cfg(any(feature = "image", feature = "svg"))]
            images: Vec::new(),
        }
    }
}

impl Renderer {
    /// Creates a new [`Renderer`].
    ///
    /// [`Renderer`]: struct.Renderer.html
    pub fn new(device: &mut (), _settings: Settings) -> Self {
        Self {
            quad_pipeline: quad::Pipeline::new(device),
            text_pipeline: text::Pipeline::new(None),
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
        target: Target<'_>,
        (primitive, mouse_cursor): &(Primitive, Interaction),
        scale_factor: f64,
        overlay: &[T],
    ) -> Interaction{
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
                scale_factor,
                transformation,
                &layer,
                target.texture,
                width,
                height,
            );
        }

        #[cfg(any(feature = "image", feature = "svg"))]
        self.image_pipeline.trim_cache();

        *mouse_cursor
    }

    fn draw_primitive<'t>(
        &mut self,
        translation: Vector,
        primitive: &'t Primitive,
        layers: &mut Vec<Layer<'t>>,
    ) {
        match *primitive {
            Primitive::None => {}
            Primitive::Group { ref primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    self.draw_primitive(translation, &primitive, layers)
                }
            }
            Primitive::Text {
                ref content,
                bounds,
                size,
                color,
                font,
                horizontal_alignment,
                vertical_alignment,
            } => {
                let layer = layers.last_mut().unwrap();

                layer.text.push(text::Section {
                    content: &content,
                    bounds: Rectangle {
                        x: (bounds.x + translation.x) as u32,
                        y: (bounds.y + translation.y) as u32,
                        ..bounds.into()
                    },
                    size,
                    color,
                    font,
                    horizontal_alignment,
                    vertical_alignment,
                })
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
                    border_radius: border_radius as f32,
                    border_width: border_width as f32,
                    border_color: border_color.into_linear(),
                });
            }
            Primitive::Clip {
                bounds,
                offset,
                ref content,
            } => {
                let layer = layers.last_mut().unwrap();

                let layer_bounds: Rectangle<f32> = layer.bounds.into();

                let clip = Rectangle {
                    x: bounds.x + translation.x,
                    y: bounds.y + translation.y,
                    ..bounds
                };

                // Only draw visible content
                if let Some(clip_bounds) = layer_bounds.intersection(&clip) {
                    let clip_layer = Layer::new(clip_bounds.into());
                    let new_layer = Layer::new(layer.bounds);

                    layers.push(clip_layer);
                    self.draw_primitive(
                        translation
                            - Vector::new(offset.x as f32, offset.y as f32),
                        &content,
                        layers,
                    );
                    layers.push(new_layer);
                }
            }

            Primitive::Cached { origin, ref cache } => {
                self.draw_primitive(
                    translation + Vector::new(origin.x, origin.y),
                    &cache,
                    layers,
                );
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

    fn draw_overlay<T: AsRef<str>>(
        &mut self,
        _lines: &[T],
        _layers: &mut Vec<Layer>,
    ) {
        /*let mut overlay = Layer::new(layers.bounds);

        for (i, line) in lines.iter().enumerate() {
            overlay.text.push(Section {
                text: line.as_ref(),
                color: [0.9, 0.9, 0.9, 1.0],
                ..Default::default()
            });
        }

        layers.push(overlay);*/
    }

    fn flush(
        &mut self,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer,
        target: &mut framework::widget::Target,
        _target_width: u32,
        _target_height: u32,
    ) {
        let bounds = layer.bounds * scale_factor;

        if !layer.quads.is_empty() {
            self.quad_pipeline.draw(
                &layer.quads,
                transformation,
                scale_factor,
                bounds,
                target,
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
                let text = text::Section {
                    /*screen_position: (
                        (text.screen_position.0 * scale_factor),
                        (text.screen_position.1 * scale_factor),
                    ),
                    bounds: (
                        (text.bounds.0 * scale_factor),
                        (text.bounds.1 * scale_factor),
                    ),
                    scale: wgpu_glyph::Scale {
                        x: text.scale.x * scale_factor,
                        y: text.scale.y * scale_factor,
                    },*/
                    ..*text
                };
                self.text_pipeline.queue(text);
            }

            self.text_pipeline
                .draw_queued(target, transformation, bounds);
        }
    }
}

impl iced_native::Renderer for Renderer {
    type Output = (Primitive, Interaction);
    type Defaults = Defaults;

    fn layout<'a, Message>(
        &mut self,
        element: &iced_native::Element<'a, Message, Self>,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        element.layout(self, limits)
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
