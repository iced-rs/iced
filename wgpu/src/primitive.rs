use iced_native::{text, Background, Color, Font, Rectangle, Vector};

#[derive(Debug, Clone)]
pub enum Primitive {
    None,
    Group {
        primitives: Vec<Primitive>,
    },
    Text {
        content: String,
        bounds: Rectangle,
        color: Color,
        size: f32,
        font: Font,
        horizontal_alignment: text::HorizontalAlignment,
        vertical_alignment: text::VerticalAlignment,
    },
    Quad {
        bounds: Rectangle,
        background: Background,
        border_radius: u16,
    },
    Image {
        path: String,
        bounds: Rectangle,
    },
    Clip {
        bounds: Rectangle,
        offset: Vector<u32>,
        content: Box<Primitive>,
    },
}
