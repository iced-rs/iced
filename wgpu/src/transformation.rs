#[derive(Debug, Clone, Copy)]
pub struct Transformation([f32; 16]);

impl Transformation {
    #[rustfmt::skip]
    pub fn identity() -> Self {
        Transformation([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn orthographic(width: u16, height: u16) -> Self {
        Transformation([
            2.0 / width as f32, 0.0, 0.0, 0.0,
            0.0, 2.0 / height as f32, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -1.0, -1.0, 0.0, 1.0,
        ])
    }
}

impl From<Transformation> for [f32; 16] {
    fn from(transformation: Transformation) -> [f32; 16] {
        transformation.0
    }
}
