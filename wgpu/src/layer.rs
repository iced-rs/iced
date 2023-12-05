//! Organize rendering primitives into a flattened list of layers.
mod image;
mod pipeline;
mod text;

pub mod mesh;

pub use image::Image;
pub use mesh::Mesh;
pub use pipeline::Pipeline;
pub use text::Text;

use crate::core;
use crate::core::alignment;
use crate::core::{Color, Font, Pixels, Point, Rectangle, Size, Vector};
use crate::graphics;
use crate::graphics::color;
use crate::graphics::Viewport;
use crate::primitive::{self, Primitive};
use crate::quad::{self, Quad};

/// A group of primitives that should be clipped together.
#[derive(Debug)]
pub struct Layer<'a> {
    /// The clipping bounds of the [`Layer`].
    pub bounds: Rectangle,

    /// The quads of the [`Layer`].
    pub quads: quad::Batch,

    /// The triangle meshes of the [`Layer`].
    pub meshes: Vec<Mesh<'a>>,

    /// The text of the [`Layer`].
    pub text: Vec<Text<'a>>,

    /// The images of the [`Layer`].
    pub images: Vec<Image>,

    /// The custom pipelines of this [`Layer`].
    pub pipelines: Vec<Pipeline>,
}

impl<'a> Layer<'a> {
    /// Creates a new [`Layer`] with the given clipping bounds.
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: quad::Batch::default(),
            meshes: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
            pipelines: Vec::new(),
        }
    }

    /// Creates a new [`Layer`] for the provided overlay text.
    ///
    /// This can be useful for displaying debug information.
    pub fn overlay(lines: &'a [impl AsRef<str>], viewport: &Viewport) -> Self {
        let mut overlay =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        for (i, line) in lines.iter().enumerate() {
            let text = text::Cached {
                content: line.as_ref(),
                bounds: Rectangle::new(
                    Point::new(11.0, 11.0 + 25.0 * i as f32),
                    Size::INFINITY,
                ),
                color: Color::new(0.9, 0.9, 0.9, 1.0),
                size: Pixels(20.0),
                line_height: core::text::LineHeight::default(),
                font: Font::MONOSPACE,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: core::text::Shaping::Basic,
                clip_bounds: Rectangle::with_size(Size::INFINITY),
            };

            overlay.text.push(Text::Cached(text.clone()));

            overlay.text.push(Text::Cached(text::Cached {
                bounds: text.bounds + Vector::new(-1.0, -1.0),
                color: Color::BLACK,
                ..text
            }));
        }

        overlay
    }

    /// Distributes the given [`Primitive`] and generates a list of layers based
    /// on its contents.
    pub fn generate(
        primitives: &'a [Primitive],
        viewport: &Viewport,
    ) -> Vec<Self> {
        let first_layer =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        let mut layers = vec![first_layer];

        for primitive in primitives {
            Self::process_primitive(
                &mut layers,
                Vector::new(0.0, 0.0),
                primitive,
                0,
            );
        }

        layers
    }

    fn process_primitive(
        layers: &mut Vec<Self>,
        translation: Vector,
        primitive: &'a Primitive,
        current_layer: usize,
    ) {
        match primitive {
            Primitive::Paragraph {
                paragraph,
                position,
                color,
                clip_bounds,
            } => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text::Paragraph {
                    paragraph: paragraph.clone(),
                    position: *position + translation,
                    color: *color,
                    clip_bounds: *clip_bounds + translation,
                });
            }
            Primitive::Editor {
                editor,
                position,
                color,
                clip_bounds,
            } => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text::Editor {
                    editor: editor.clone(),
                    position: *position + translation,
                    color: *color,
                    clip_bounds: *clip_bounds + translation,
                });
            }
            Primitive::Text {
                content,
                bounds,
                size,
                line_height,
                color,
                font,
                horizontal_alignment,
                vertical_alignment,
                shaping,
                clip_bounds,
            } => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text::Cached(text::Cached {
                    content,
                    bounds: *bounds + translation,
                    size: *size,
                    line_height: *line_height,
                    color: *color,
                    font: *font,
                    horizontal_alignment: *horizontal_alignment,
                    vertical_alignment: *vertical_alignment,
                    shaping: *shaping,
                    clip_bounds: *clip_bounds + translation,
                }));
            }
            graphics::Primitive::RawText(graphics::text::Raw {
                buffer,
                position,
                color,
                clip_bounds,
            }) => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text::Raw(graphics::text::Raw {
                    buffer: buffer.clone(),
                    position: *position + translation,
                    color: *color,
                    clip_bounds: *clip_bounds + translation,
                }));
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let layer = &mut layers[current_layer];

                let quad = Quad {
                    position: [
                        bounds.x + translation.x,
                        bounds.y + translation.y,
                    ],
                    size: [bounds.width, bounds.height],
                    border_color: color::pack(*border_color),
                    border_radius: *border_radius,
                    border_width: *border_width,
                };

                layer.quads.add(quad, background);
            }
            Primitive::Image {
                handle,
                filter_method,
                bounds,
            } => {
                let layer = &mut layers[current_layer];

                layer.images.push(Image::Raster {
                    handle: handle.clone(),
                    filter_method: *filter_method,
                    bounds: *bounds + translation,
                });
            }
            Primitive::Svg {
                handle,
                color,
                bounds,
            } => {
                let layer = &mut layers[current_layer];

                layer.images.push(Image::Vector {
                    handle: handle.clone(),
                    color: *color,
                    bounds: *bounds + translation,
                });
            }
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    Self::process_primitive(
                        layers,
                        translation,
                        primitive,
                        current_layer,
                    );
                }
            }
            Primitive::Clip { bounds, content } => {
                let layer = &mut layers[current_layer];
                let translated_bounds = *bounds + translation;

                // Only draw visible content
                if let Some(clip_bounds) =
                    layer.bounds.intersection(&translated_bounds)
                {
                    let clip_layer = Layer::new(clip_bounds);
                    layers.push(clip_layer);

                    Self::process_primitive(
                        layers,
                        translation,
                        content,
                        layers.len() - 1,
                    );
                }
            }
            Primitive::Translate {
                translation: new_translation,
                content,
            } => {
                Self::process_primitive(
                    layers,
                    translation + *new_translation,
                    content,
                    current_layer,
                );
            }
            Primitive::Cache { content } => {
                Self::process_primitive(
                    layers,
                    translation,
                    content,
                    current_layer,
                );
            }
            Primitive::Custom(custom) => match custom {
                primitive::Custom::Mesh(mesh) => match mesh {
                    graphics::Mesh::Solid { buffers, size } => {
                        let layer = &mut layers[current_layer];

                        let bounds = Rectangle::new(
                            Point::new(translation.x, translation.y),
                            *size,
                        );

                        // Only draw visible content
                        if let Some(clip_bounds) =
                            layer.bounds.intersection(&bounds)
                        {
                            layer.meshes.push(Mesh::Solid {
                                origin: Point::new(
                                    translation.x,
                                    translation.y,
                                ),
                                buffers,
                                clip_bounds,
                            });
                        }
                    }
                    graphics::Mesh::Gradient { buffers, size } => {
                        let layer = &mut layers[current_layer];

                        let bounds = Rectangle::new(
                            Point::new(translation.x, translation.y),
                            *size,
                        );

                        // Only draw visible content
                        if let Some(clip_bounds) =
                            layer.bounds.intersection(&bounds)
                        {
                            layer.meshes.push(Mesh::Gradient {
                                origin: Point::new(
                                    translation.x,
                                    translation.y,
                                ),
                                buffers,
                                clip_bounds,
                            });
                        }
                    }
                },
                primitive::Custom::Pipeline(pipeline) => {
                    let layer = &mut layers[current_layer];
                    let bounds = pipeline.bounds + translation;

                    if let Some(clip_bounds) =
                        layer.bounds.intersection(&bounds)
                    {
                        layer.pipelines.push(Pipeline {
                            bounds,
                            viewport: clip_bounds,
                            primitive: pipeline.primitive.clone(),
                        });
                    }
                }
            },
        }
    }
}
