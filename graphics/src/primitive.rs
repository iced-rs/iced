use iced_native::image;
use iced_native::svg;
use iced_native::{Background, Color, Font, Rectangle, Size, Vector};

use crate::alignment;
use crate::gradient::Gradient;
use crate::triangle;

use std::sync::Arc;

/// A rendering primitive.
#[derive(Debug, Clone)]
pub enum Primitive {
    /// An empty primitive
    None,
    /// A group of primitives
    Group {
        /// The primitives of the group
        primitives: Vec<Primitive>,
    },
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
    /// A low-level primitive to render a mesh of triangles with a solid color.
    ///
    /// It can be used to render many kinds of geometry freely.
    SolidMesh {
        /// The vertices and indices of the mesh.
        buffers: triangle::Mesh2D<triangle::ColoredVertex2D>,

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
        buffers: triangle::Mesh2D<triangle::Vertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,

        /// The [`Gradient`] to apply to the mesh.
        gradient: Gradient,
    },
    /// A cached primitive.
    ///
    /// This can be useful if you are implementing a widget where primitive
    /// generation is expensive.
    Cached {
        /// The cached primitive
        cache: Arc<Primitive>,
    },
}

impl Default for Primitive {
    fn default() -> Primitive {
        Primitive::None
    }
}
