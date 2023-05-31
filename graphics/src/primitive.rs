//! Draw using different graphical primitives.
use crate::color;
use crate::core::alignment;
use crate::core::image;
use crate::core::svg;
use crate::core::text;
use crate::core::{Background, Color, Font, Rectangle, Size, Vector};
use crate::gradient;

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
        /// The size of the text in logical pixels
        size: f32,
        /// The line height of the text
        line_height: text::LineHeight,
        /// The font of the text
        font: Font,
        /// The horizontal alignment of the text
        horizontal_alignment: alignment::Horizontal,
        /// The vertical alignment of the text
        vertical_alignment: alignment::Vertical,
        /// The shaping strategy of the text.
        shaping: text::Shaping,
    },
    /// A quad primitive
    Quad {
        /// The bounds of the quad
        bounds: Rectangle,
        /// The background of the quad
        background: Background,
        /// The border radii of the quad
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
        buffers: Mesh2D<GradientVertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,
    },
    /// A [`tiny_skia`] path filled with some paint.
    #[cfg(feature = "tiny-skia")]
    Fill {
        /// The path to fill.
        path: tiny_skia::Path,
        /// The paint to use.
        paint: tiny_skia::Paint<'static>,
        /// The fill rule to follow.
        rule: tiny_skia::FillRule,
        /// The transform to apply to the path.
        transform: tiny_skia::Transform,
    },
    /// A [`tiny_skia`] path stroked with some paint.
    #[cfg(feature = "tiny-skia")]
    Stroke {
        /// The path to stroke.
        path: tiny_skia::Path,
        /// The paint to use.
        paint: tiny_skia::Paint<'static>,
        /// The stroke settings.
        stroke: tiny_skia::Stroke,
        /// The transform to apply to the path.
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
    /// Creates a [`Primitive::Group`].
    pub fn group(primitives: Vec<Self>) -> Self {
        Self::Group { primitives }
    }

    /// Creates a [`Primitive::Clip`].
    pub fn clip(self, bounds: Rectangle) -> Self {
        Self::Clip {
            bounds,
            content: Box::new(self),
        }
    }

    /// Creates a [`Primitive::Translate`].
    pub fn translate(self, translation: Vector) -> Self {
        Self::Translate {
            translation,
            content: Box::new(self),
        }
    }

    /// Returns the bounds of the [`Primitive`].
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

                bounds.expand(1.5)
            }
            Self::Quad { bounds, .. }
            | Self::Image { bounds, .. }
            | Self::Svg { bounds, .. } => bounds.expand(1.0),
            Self::Clip { bounds, .. } => bounds.expand(1.0),
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

/// A two-dimensional vertex with a color.
#[derive(Copy, Clone, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct ColoredVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The color of the vertex in __linear__ RGBA.
    pub color: color::Packed,
}

/// A vertex which contains 2D position & packed gradient data.
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct GradientVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The packed vertex data of the gradient.
    pub gradient: gradient::Packed,
}

#[allow(unsafe_code)]
unsafe impl Zeroable for GradientVertex2D {}

#[allow(unsafe_code)]
unsafe impl Pod for GradientVertex2D {}

impl From<()> for Primitive {
    fn from(_: ()) -> Self {
        Self::Group { primitives: vec![] }
    }
}
