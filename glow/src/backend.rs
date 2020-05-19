use crate::quad;
use crate::text;
use crate::triangle;
use crate::{Quad, Settings, Transformation, Viewport};
use iced_graphics::backend;
use iced_graphics::font;
use iced_graphics::Primitive;
use iced_native::mouse;
use iced_native::{Background, Font, Point, Rectangle, Size, Vector};

/// A [`glow`] renderer.
///
/// [`glow`]: https://github.com/grovesNL/glow
#[derive(Debug)]
pub struct Backend {
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
    triangle_pipeline: triangle::Pipeline,
}

struct Layer<'a> {
    bounds: Rectangle<u32>,
    quads: Vec<Quad>,
    text: Vec<glow_glyph::Section<'a>>,
    meshes: Vec<(Vector, Rectangle<u32>, &'a triangle::Mesh2D)>,
}

impl<'a> Layer<'a> {
    pub fn new(bounds: Rectangle<u32>) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
            text: Vec::new(),
            meshes: Vec::new(),
        }
    }

    pub fn intersection(&self, rectangle: Rectangle) -> Option<Rectangle<u32>> {
        let layer_bounds: Rectangle<f32> = self.bounds.into();

        layer_bounds.intersection(&rectangle).map(Into::into)
    }
}

impl Backend {
    /// Creates a new [`Renderer`].
    ///
    /// [`Renderer`]: struct.Renderer.html
    pub fn new(gl: &glow::Context, settings: Settings) -> Self {
        let text_pipeline = text::Pipeline::new(gl, settings.default_font);
        let quad_pipeline = quad::Pipeline::new(gl);
        let triangle_pipeline =
            triangle::Pipeline::new(gl, settings.antialiasing);

        Self {
            quad_pipeline,
            text_pipeline,
            triangle_pipeline,
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
        gl: &glow::Context,
        viewport: &Viewport,
        (primitive, mouse_interaction): &(Primitive, mouse::Interaction),
        scale_factor: f64,
        overlay: &[T],
    ) -> mouse::Interaction {
        let (width, height) = viewport.dimensions();
        let scale_factor = scale_factor as f32;
        let transformation = viewport.transformation();

        let mut layers = Vec::new();

        layers.push(Layer::new(Rectangle {
            x: 0,
            y: 0,
            width: (width as f32 / scale_factor).round() as u32,
            height: (height as f32 / scale_factor).round() as u32,
        }));

        self.draw_primitive(Vector::new(0.0, 0.0), primitive, &mut layers);
        self.draw_overlay(overlay, &mut layers);

        for layer in layers {
            self.flush(
                gl,
                viewport,
                scale_factor,
                transformation,
                &layer,
                width,
                height,
            );
        }

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

                layer.text.push(glow_glyph::Section {
                    text: &content,
                    screen_position: (
                        bounds.x + translation.x,
                        bounds.y + translation.y,
                    ),
                    bounds: (bounds.width, bounds.height),
                    scale: glow_glyph::Scale { x: *size, y: *size },
                    color: color.into_linear(),
                    font_id: self.text_pipeline.find_font(*font),
                    layout: glow_glyph::Layout::default()
                        .h_align(match horizontal_alignment {
                            iced_native::HorizontalAlignment::Left => {
                                glow_glyph::HorizontalAlign::Left
                            }
                            iced_native::HorizontalAlignment::Center => {
                                glow_glyph::HorizontalAlign::Center
                            }
                            iced_native::HorizontalAlignment::Right => {
                                glow_glyph::HorizontalAlign::Right
                            }
                        })
                        .v_align(match vertical_alignment {
                            iced_native::VerticalAlignment::Top => {
                                glow_glyph::VerticalAlign::Top
                            }
                            iced_native::VerticalAlignment::Center => {
                                glow_glyph::VerticalAlign::Center
                            }
                            iced_native::VerticalAlignment::Bottom => {
                                glow_glyph::VerticalAlign::Bottom
                            }
                        }),
                    ..Default::default()
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

        let font_id = self.text_pipeline.overlay_font();
        let scale = glow_glyph::Scale { x: 20.0, y: 20.0 };

        for (i, line) in lines.iter().enumerate() {
            overlay.text.push(glow_glyph::Section {
                text: line.as_ref(),
                screen_position: (11.0, 11.0 + 25.0 * i as f32),
                color: [0.9, 0.9, 0.9, 1.0],
                scale,
                font_id,
                ..glow_glyph::Section::default()
            });

            overlay.text.push(glow_glyph::Section {
                text: line.as_ref(),
                screen_position: (10.0, 10.0 + 25.0 * i as f32),
                color: [0.0, 0.0, 0.0, 1.0],
                scale,
                font_id,
                ..glow_glyph::Section::default()
            });
        }

        layers.push(overlay);
    }

    fn flush(
        &mut self,
        gl: &glow::Context,
        viewport: &Viewport,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer<'_>,
        target_width: u32,
        target_height: u32,
    ) {
        let bounds = layer.bounds * scale_factor;

        if !layer.quads.is_empty() {
            self.quad_pipeline.draw(
                gl,
                viewport,
                &layer.quads,
                transformation,
                scale_factor,
                bounds,
            );
        }

        if !layer.meshes.is_empty() {
            let scaled = transformation
                * Transformation::scale(scale_factor, scale_factor);

            self.triangle_pipeline.draw(
                gl,
                target_width,
                target_height,
                scaled,
                scale_factor,
                &layer.meshes,
            );
        }

        if !layer.text.is_empty() {
            for text in layer.text.iter() {
                // Target physical coordinates directly to avoid blurry text
                let text = glow_glyph::Section {
                    // TODO: We `round` here to avoid rerasterizing text when
                    // its position changes slightly. This can make text feel a
                    // bit "jumpy". We may be able to do better once we improve
                    // our text rendering/caching pipeline.
                    screen_position: (
                        (text.screen_position.0 * scale_factor).round(),
                        (text.screen_position.1 * scale_factor).round(),
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
                        (text.bounds.0 * scale_factor).ceil(),
                        (text.bounds.1 * scale_factor).ceil(),
                    ),
                    scale: glow_glyph::Scale {
                        x: text.scale.x * scale_factor,
                        y: text.scale.y * scale_factor,
                    },
                    ..*text
                };

                self.text_pipeline.queue(text);
            }

            self.text_pipeline.draw_queued(
                gl,
                transformation,
                glow_glyph::Region {
                    x: bounds.x,
                    y: viewport.height()
                        - (bounds.y + bounds.height).min(viewport.height()),
                    width: bounds.width,
                    height: bounds.height,
                },
            );
        }
    }
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        self.text_pipeline.trim_measurement_cache()
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = font::ICONS;
    const CHECKMARK_ICON: char = font::CHECKMARK_ICON;

    fn measure(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.text_pipeline.measure(contents, size, font, bounds)
    }

    fn space_width(&self, size: f32) -> f32 {
        self.text_pipeline.space_width(size)
    }
}
