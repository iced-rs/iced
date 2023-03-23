//! Organize rendering primitives into a flattened list of layers.
mod image;
mod text;

pub mod mesh;
pub mod quad;

pub use image::Image;
pub use mesh::Mesh;
pub use quad::{Gradient, Solid};
pub use text::Text;

use crate::core::alignment;
use crate::core::{Background, Color, Font, Point, Rectangle, Size, Vector};
use crate::graphics::{Primitive, Viewport};

/// A group of primitives that should be clipped together.
#[derive(Debug)]
pub struct Layer<'a> {
    /// The clipping bounds of the [`Layer`].
    pub bounds: Rectangle,

    /// The quads of the [`Layer`].
    pub quads: Quads,

    /// The triangle meshes of the [`Layer`].
    pub meshes: Vec<Mesh<'a>>,

    /// The text of the [`Layer`].
    pub text: Vec<Text<'a>>,

    /// The images of the [`Layer`].
    pub images: Vec<Image>,
}

/// The quads of the [`Layer`].
#[derive(Default, Debug)]
pub struct Quads {
    /// The solid quads of the [`Layer`].
    pub solids: Vec<quad::Solid>,

    /// The gradient quads of the [`Layer`].
    pub gradients: Vec<quad::Gradient>,
}

impl Quads {
    /// Returns whether both solid & gradient quads are empty or not.
    pub fn is_empty(&self) -> bool {
        self.solids.is_empty() && self.gradients.is_empty()
    }
}

impl<'a> Layer<'a> {
    /// Creates a new [`Layer`] with the given clipping bounds.
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: Quads::default(),
            meshes: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Creates a new [`Layer`] for the provided overlay text.
    ///
    /// This can be useful for displaying debug information.
    pub fn overlay(lines: &'a [impl AsRef<str>], viewport: &Viewport) -> Self {
        let mut overlay =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        for (i, line) in lines.iter().enumerate() {
            let text = Text {
                content: line.as_ref(),
                bounds: Rectangle::new(
                    Point::new(11.0, 11.0 + 25.0 * i as f32),
                    Size::INFINITY,
                ),
                color: Color::new(0.9, 0.9, 0.9, 1.0),
                size: 20.0,
                font: Font::Monospace,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
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
            Primitive::Text {
                content,
                bounds,
                size,
                color,
                font,
                horizontal_alignment,
                vertical_alignment,
            } => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text {
                    content,
                    bounds: *bounds + translation,
                    size: *size,
                    color: *color,
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
                let layer = &mut layers[current_layer];

                let properties = quad::Properties {
                    position: [
                        bounds.x + translation.x,
                        bounds.y + translation.y,
                    ],
                    size: [bounds.width, bounds.height],
                    border_color: border_color.into_linear(),
                    border_radius: *border_radius,
                    border_width: *border_width,
                };

                match background {
                    Background::Color(color) => {
                        layer.quads.solids.push(quad::Solid {
                            color: color.into_linear(),
                            properties,
                        });
                    }
                    Background::Gradient(gradient) => {
                        let quad = quad::Gradient {
                            gradient: pack_gradient(
                                gradient,
                                Rectangle::new(
                                    properties.position.into(),
                                    properties.size.into(),
                                ),
                            ),
                            properties,
                        };

                        layer.quads.gradients.push(quad);
                    }
                };
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
            Primitive::SolidMesh { buffers, size } => {
                let layer = &mut layers[current_layer];

                let bounds = Rectangle::new(
                    Point::new(translation.x, translation.y),
                    *size,
                );

                // Only draw visible content
                if let Some(clip_bounds) = layer.bounds.intersection(&bounds) {
                    layer.meshes.push(Mesh::Solid {
                        origin: Point::new(translation.x, translation.y),
                        buffers,
                        clip_bounds,
                    });
                }
            }
            Primitive::GradientMesh { buffers, size } => {
                let layer = &mut layers[current_layer];

                let bounds = Rectangle::new(
                    Point::new(translation.x, translation.y),
                    *size,
                );

                // Only draw visible content
                if let Some(clip_bounds) = layer.bounds.intersection(&bounds) {
                    layer.meshes.push(Mesh::Gradient {
                        origin: Point::new(translation.x, translation.y),
                        buffers,
                        clip_bounds,
                    });
                }
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
            _ => {
                // Unsupported!
            }
        }
    }
}

/// Packs the [`Gradient`] for use in shader code.
fn pack_gradient(
    gradient: &crate::core::Gradient,
    bounds: Rectangle,
) -> [f32; 44] {
    match gradient {
        crate::core::Gradient::Linear(linear) => {
            let mut pack: [f32; 44] = [0.0; 44];

            for (index, stop) in linear.color_stops.iter().enumerate() {
                let [r, g, b, a] =
                    stop.map_or(Color::default(), |s| s.color).into_linear();

                pack[(index * 4)] = r;
                pack[(index * 4) + 1] = g;
                pack[(index * 4) + 2] = b;
                pack[(index * 4) + 3] = a;
                pack[32 + index] = stop.map_or(2.0, |s| s.offset);
            }

            let (start, end) = linear.angle.to_distance(&bounds);

            pack[40] = start.x;
            pack[41] = start.y;
            pack[42] = end.x;
            pack[43] = end.y;

            pack
        }
    }
}
