use iced_native::{
    image, svg, Background, Color, Font, HorizontalAlignment, Rectangle,
    Vector, VerticalAlignment,
};

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
        horizontal_alignment: HorizontalAlignment,
        /// The vertical alignment of the text
        vertical_alignment: VerticalAlignment,
    },
    /// A quad primitive
    Quad {
        /// The bounds of the quad
        bounds: Rectangle,
        /// The background of the quad
        background: Background,
        /// The border radius of the quad
        border_radius: u16,
        /// The border width of the quad
        border_width: u16,
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

        /// The bounds of the viewport
        bounds: Rectangle,
    },
    /// A clip primitive
    Clip {
        /// The bounds of the clip
        bounds: Rectangle,
        /// The offset transformation of the clip
        offset: Vector<u32>,
        /// The content of the clip
        content: Box<Primitive>,
    },
}

impl Default for Primitive {
    fn default() -> Primitive {
        Primitive::None
    }
}
