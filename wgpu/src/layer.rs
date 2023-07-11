//! Organize rendering primitives into a flattened list of layers.
mod image;
mod text;

pub mod mesh;

pub use image::Image;
pub use mesh::Mesh;
pub use text::Text;

use crate::core;
use crate::core::alignment;
use crate::core::renderer::Effect;
use crate::core::{Color, Font, Point, Rectangle, Size, Vector};
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

    /// The kind of [`Layer`].
    pub kind: Kind,
}

/// The [`Kind`] of a layer.
#[derive(Debug)]
pub enum Kind {
    /// A [`Layer`] which is rendered directly to the viewport's surface.
    Immediate,
    /// A [`Layer`] which is first rendered to an intermediate texture, and then composited on to the
    /// viewport's surface. A composited layer must have some `effect` which requires intermediate
    /// processing, or else could just be rendered directly to the surface.
    Deferred {
        /// The supported [`Effect`] that this [`Layer`] has.
        effect: Effect,
    },
}

impl<'a> Layer<'a> {
    /// Creates a new [`Layer`] which is rendered directly to the viewport.
    pub fn viewport(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: quad::Batch::default(),
            meshes: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
            kind: Kind::Immediate,
        }
    }

    /// Creates a new [`Layer`] which is rendered to its own texture.
    pub fn composite(bounds: Rectangle, effect: Effect) -> Self {
        Self {
            bounds,
            quads: quad::Batch::default(),
            meshes: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
            kind: Kind::Deferred { effect },
        }
    }

    /// Creates a new [`Layer`] for the provided overlay text.
    ///
    /// This can be useful for displaying debug information.
    pub fn overlay(lines: &'a [impl AsRef<str>], viewport: &Viewport) -> Self {
        let mut overlay =
            Layer::viewport(Rectangle::with_size(viewport.logical_size()));

        for (i, line) in lines.iter().enumerate() {
            let text = Text {
                content: line.as_ref(),
                bounds: Rectangle::new(
                    Point::new(11.0, 11.0 + 25.0 * i as f32),
                    Size::INFINITY,
                ),
                color: Color::new(0.9, 0.9, 0.9, 1.0),
                size: 20.0,
                line_height: core::text::LineHeight::default(),
                font: Font::MONOSPACE,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: core::text::Shaping::Basic,
            };

            overlay.text.push(text);

            overlay.text.push(Text {
                bounds: text.bounds + Vector::new(-1.0, -1.0),
                color: Color::BLACK,
                ..text
            });
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
            Layer::viewport(Rectangle::with_size(viewport.logical_size()));

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
            } => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text {
                    content,
                    bounds: *bounds + translation,
                    size: *size,
                    line_height: *line_height,
                    color: *color,
                    font: *font,
                    horizontal_alignment: *horizontal_alignment,
                    vertical_alignment: *vertical_alignment,
                    shaping: *shaping,
                });
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
            Primitive::Image { handle, bounds } => {
                let layer = &mut layers[current_layer];

                layer.images.push(Image::Raster {
                    handle: handle.clone(),
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
                    )
                }
            }
            Primitive::Clip { bounds, content } => {
                let layer = &mut layers[current_layer];
                let translated_bounds = *bounds + translation;

                // Only draw visible content
                if let Some(clip_bounds) =
                    layer.bounds.intersection(&translated_bounds)
                {
                    let clip_layer = Layer::viewport(clip_bounds);
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
            },
            Primitive::Effect {
                bounds,
                effect,
                content,
            } => {
                match effect {
                    Effect::Blur { radius } => {
                        let blurred_layer = Layer::composite(
                            *bounds,
                            Effect::Blur { radius: *radius },
                        );

                        layers.push(blurred_layer);

                        Self::process_primitive(
                            layers,
                            //shift up to origin to properly align to texture
                            translation - Vector::new(bounds.x, bounds.y),
                            content,
                            layers.len() - 1,
                        );
                    }
                }
            }
        }
    }
}
