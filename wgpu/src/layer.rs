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
use crate::core::{
    Color, Font, Pixels, Point, Rectangle, Size, Transformation, Vector,
};
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
                Transformation::IDENTITY,
                primitive,
                0,
            );
        }

        layers
    }

    fn process_primitive(
        layers: &mut Vec<Self>,
        transformation: Transformation,
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
                    position: *position,
                    color: *color,
                    clip_bounds: *clip_bounds,
                    transformation,
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
                    position: *position,
                    color: *color,
                    clip_bounds: *clip_bounds,
                    transformation,
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
                    bounds: *bounds + transformation.translation(),
                    size: *size * transformation.scale_factor(),
                    line_height: *line_height,
                    color: *color,
                    font: *font,
                    horizontal_alignment: *horizontal_alignment,
                    vertical_alignment: *vertical_alignment,
                    shaping: *shaping,
                    clip_bounds: *clip_bounds * transformation,
                }));
            }
            graphics::Primitive::RawText(raw) => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text::Raw {
                    raw: raw.clone(),
                    transformation,
                });
            }
            Primitive::Quad {
                bounds,
                background,
                border,
                shadow,
            } => {
                let layer = &mut layers[current_layer];
                let bounds = *bounds * transformation;

                let quad = Quad {
                    position: [bounds.x, bounds.y],
                    size: [bounds.width, bounds.height],
                    border_color: color::pack(border.color),
                    border_radius: border.radius.into(),
                    border_width: border.width,
                    shadow_color: shadow.color.into_linear(),
                    shadow_offset: shadow.offset.into(),
                    shadow_blur_radius: shadow.blur_radius,
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
                    bounds: *bounds * transformation,
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
                    bounds: *bounds * transformation,
                });
            }
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    Self::process_primitive(
                        layers,
                        transformation,
                        primitive,
                        current_layer,
                    );
                }
            }
            Primitive::Clip { bounds, content } => {
                let layer = &mut layers[current_layer];
                let translated_bounds = *bounds * transformation;

                // Only draw visible content
                if let Some(clip_bounds) =
                    layer.bounds.intersection(&translated_bounds)
                {
                    let clip_layer = Layer::new(clip_bounds);
                    layers.push(clip_layer);

                    Self::process_primitive(
                        layers,
                        transformation,
                        content,
                        layers.len() - 1,
                    );
                }
            }
            Primitive::Transform {
                transformation: new_transformation,
                content,
            } => {
                Self::process_primitive(
                    layers,
                    transformation * *new_transformation,
                    content,
                    current_layer,
                );
            }
            Primitive::Cache { content } => {
                Self::process_primitive(
                    layers,
                    transformation,
                    content,
                    current_layer,
                );
            }
            Primitive::Custom(custom) => match custom {
                primitive::Custom::Mesh(mesh) => match mesh {
                    graphics::Mesh::Solid { buffers, size } => {
                        let layer = &mut layers[current_layer];

                        let bounds =
                            Rectangle::with_size(*size) * transformation;

                        // Only draw visible content
                        if let Some(clip_bounds) =
                            layer.bounds.intersection(&bounds)
                        {
                            layer.meshes.push(Mesh::Solid {
                                transformation,
                                buffers,
                                clip_bounds,
                            });
                        }
                    }
                    graphics::Mesh::Gradient { buffers, size } => {
                        let layer = &mut layers[current_layer];

                        let bounds =
                            Rectangle::with_size(*size) * transformation;

                        // Only draw visible content
                        if let Some(clip_bounds) =
                            layer.bounds.intersection(&bounds)
                        {
                            layer.meshes.push(Mesh::Gradient {
                                transformation,
                                buffers,
                                clip_bounds,
                            });
                        }
                    }
                },
                primitive::Custom::Pipeline(pipeline) => {
                    let layer = &mut layers[current_layer];
                    let bounds = pipeline.bounds * transformation;

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
