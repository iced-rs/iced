use iced_native::{
    Background, Color, Font, HorizontalAlignment, Rectangle, Vector,
    VerticalAlignment,
};

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
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
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
