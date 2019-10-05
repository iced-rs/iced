use iced_native::Rectangle;

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
}
