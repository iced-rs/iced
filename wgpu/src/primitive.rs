use iced_native::{text, Background, Color, Rectangle};

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
}
