use iced_native::{Color, Rectangle};

#[derive(Debug, Clone)]
pub enum Primitive {
    None,
    Group {
        primitives: Vec<Primitive>,
    },
    Text {
        content: String,
        bounds: Rectangle,
        size: f32,
    },
    Quad {
        bounds: Rectangle,
        background: Background,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    Color(Color),
}
