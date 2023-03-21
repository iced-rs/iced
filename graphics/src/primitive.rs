use iced_core::alignment;
use iced_core::image;
use iced_core::svg;
use iced_core::{Background, Color, Font, Gradient, Rectangle, Size, Vector};

use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

/// A rendering primitive.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Primitive {
    /// A text primitive
    Text {
        /// The contents of the text
        content: String,
        /// The bounds of the text
        bounds: Rectangle,
        /// The color of the text
        color: Color,
        /// The size of the text
        size: f32,
        /// The font of the text
        font: Font,
        /// The horizontal alignment of the text
        horizontal_alignment: alignment::Horizontal,
        /// The vertical alignment of the text
        vertical_alignment: alignment::Vertical,
    },
    /// A quad primitive
    Quad {
        /// The bounds of the quad
        bounds: Rectangle,
        /// The background of the quad
        background: Background,
        /// The border radius of the quad
        border_radius: [f32; 4],
        /// The border width of the quad
        border_width: f32,
        /// The border color of the quad
        border_color: Color,
    },
    /// An image primitive
    Image {
        /// The handle of the image
        handle: image::Handle,
        /// The bounds of the image
        bounds: Rectangle,
    },
    /// An SVG primitive
    Svg {
        /// The path of the SVG file
        handle: svg::Handle,

        /// The [`Color`] filter
        color: Option<Color>,

        /// The bounds of the viewport
        bounds: Rectangle,
    },
    /// A low-level primitive to render a mesh of triangles with a solid color.
    ///
    /// It can be used to render many kinds of geometry freely.
    SolidMesh {
        /// The vertices and indices of the mesh.
        buffers: Mesh2D<ColoredVertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,
    },
    /// A low-level primitive to render a mesh of triangles with a gradient.
    ///
    /// It can be used to render many kinds of geometry freely.
    GradientMesh {
        /// The vertices and indices of the mesh.
        buffers: Mesh2D<Vertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,

        /// The [`Gradient`] to apply to the mesh.
        gradient: Gradient,
    },
    #[cfg(feature = "tiny-skia")]
    Fill {
        path: tiny_skia::Path,
        paint: tiny_skia::Paint<'static>,
        rule: tiny_skia::FillRule,
        transform: tiny_skia::Transform,
    },
    #[cfg(feature = "tiny-skia")]
    Stroke {
        path: tiny_skia::Path,
        paint: tiny_skia::Paint<'static>,
        stroke: tiny_skia::Stroke,
        transform: tiny_skia::Transform,
    },
    /// A group of primitives
    Group {
        /// The primitives of the group
        primitives: Vec<Primitive>,
    },
    /// A clip primitive
    Clip {
        /// The bounds of the clip
        bounds: Rectangle,
        /// The content of the clip
        content: Box<Primitive>,
    },
    /// A primitive that applies a translation
    Translate {
        /// The translation vector
        translation: Vector,

        /// The primitive to translate
        content: Box<Primitive>,
    },
    /// A cached primitive.
    ///
    /// This can be useful if you are implementing a widget where primitive
    /// generation is expensive.
    Cache {
        /// The cached primitive
        content: Arc<Primitive>,
    },
}

impl Primitive {
    pub fn group(primitives: Vec<Self>) -> Self {
        Self::Group { primitives }
    }

    pub fn clip(self, bounds: Rectangle) -> Self {
        Self::Clip {
            bounds,
            content: Box::new(self),
        }
    }

    pub fn translate(self, translation: Vector) -> Self {
        Self::Translate {
            translation,
            content: Box::new(self),
        }
    }

    pub fn bounds(&self) -> Rectangle {
        match self {
            Self::Text {
                bounds,
                horizontal_alignment,
                vertical_alignment,
                ..
            } => {
                let mut bounds = *bounds;

                bounds.x = match horizontal_alignment {
                    alignment::Horizontal::Left => bounds.x,
                    alignment::Horizontal::Center => {
                        bounds.x - bounds.width / 2.0
                    }
                    alignment::Horizontal::Right => bounds.x - bounds.width,
                };

                bounds.y = match vertical_alignment {
                    alignment::Vertical::Top => bounds.y,
                    alignment::Vertical::Center => {
                        bounds.y - bounds.height / 2.0
                    }
                    alignment::Vertical::Bottom => bounds.y - bounds.height,
                };

                bounds.expand(1.0)
            }
            Self::Quad { bounds, .. }
            | Self::Image { bounds, .. }
            | Self::Svg { bounds, .. }
            | Self::Clip { bounds, .. } => bounds.expand(1.0),
            Self::SolidMesh { size, .. } | Self::GradientMesh { size, .. } => {
                Rectangle::with_size(*size)
            }
            #[cfg(feature = "tiny-skia")]
            Self::Fill { path, .. } | Self::Stroke { path, .. } => {
                let bounds = path.bounds();

                Rectangle {
                    x: bounds.x(),
                    y: bounds.y(),
                    width: bounds.width(),
                    height: bounds.height(),
                }
                .expand(1.0)
            }
            Self::Group { primitives } => primitives
                .iter()
                .map(Self::bounds)
                .fold(Rectangle::with_size(Size::ZERO), |a, b| {
                    Rectangle::union(&a, &b)
                }),
            Self::Translate {
                translation,
                content,
            } => content.bounds() + *translation,
            Self::Cache { content } => content.bounds(),
        }
    }

    pub fn damage(&self, other: &Self) -> Vec<Rectangle> {
        match (self, other) {
            (
                Primitive::Group {
                    primitives: primitives_a,
                },
                Primitive::Group {
                    primitives: primitives_b,
                },
            ) => return Self::damage_list(primitives_a, primitives_b),
            (
                Primitive::Clip {
                    bounds: bounds_a,
                    content: content_a,
                },
                Primitive::Clip {
                    bounds: bounds_b,
                    content: content_b,
                },
            ) => {
                if bounds_a == bounds_b {
                    return content_a.damage(content_b);
                } else {
                    return vec![*bounds_a, *bounds_b];
                }
            }
            (
                Primitive::Translate {
                    translation: translation_a,
                    content: content_a,
                },
                Primitive::Translate {
                    translation: translation_b,
                    content: content_b,
                },
            ) => {
                if translation_a == translation_b {
                    return content_a.damage(content_b);
                }
            }
            (
                Primitive::Cache { content: content_a },
                Primitive::Cache { content: content_b },
            ) => {
                if Arc::ptr_eq(content_a, content_b) {
                    return vec![];
                }
            }
            _ if self == other => return vec![],
            _ => {}
        }

        let bounds_a = self.bounds();
        let bounds_b = other.bounds();

        if bounds_a == bounds_b {
            vec![bounds_a]
        } else {
            vec![bounds_a, bounds_b]
        }
    }

    pub fn damage_list(previous: &[Self], current: &[Self]) -> Vec<Rectangle> {
        let damage =
            previous.iter().zip(current).flat_map(|(a, b)| a.damage(b));

        if previous.len() == current.len() {
            damage.collect()
        } else {
            let (smaller, bigger) = if previous.len() < current.len() {
                (previous, current)
            } else {
                (current, previous)
            };

            // Extend damage by the added/removed primitives
            damage
                .chain(bigger[smaller.len()..].iter().map(Primitive::bounds))
                .collect()
        }
    }
}

/// A set of [`Vertex2D`] and indices representing a list of triangles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mesh2D<T> {
    /// The vertices of the mesh
    pub vertices: Vec<T>,

    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of 3.
    pub indices: Vec<u32>,
}

/// A two-dimensional vertex.
#[derive(Copy, Clone, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],
}

/// A two-dimensional vertex with a color.
#[derive(Copy, Clone, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct ColoredVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The color of the vertex in __linear__ RGBA.
    pub color: [f32; 4],
}

impl From<()> for Primitive {
    fn from(_: ()) -> Self {
        Self::Group { primitives: vec![] }
    }
}
