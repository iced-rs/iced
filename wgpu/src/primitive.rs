use iced_native::{
    Background, Color, Font, HorizontalAlignment, Rectangle, Vector,
    VerticalAlignment,
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
    },
    /// An image primitive
    Image {
        /// The path of the image
        path: String,
        /// The bounds of the image
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
