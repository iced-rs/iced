/// A color in the sRGB color space.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// The black color.
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}
