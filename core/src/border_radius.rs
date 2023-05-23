/// The border radii for the corners of a graphics primitive in the order:
/// top-left, top-right, bottom-right, bottom-left.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadius([f32; 4]);

impl From<f32> for BorderRadius {
    fn from(w: f32) -> Self {
        Self([w; 4])
    }
}

impl From<[f32; 4]> for BorderRadius {
    fn from(radi: [f32; 4]) -> Self {
        Self(radi)
    }
}

impl From<BorderRadius> for [f32; 4] {
    fn from(radi: BorderRadius) -> Self {
        radi.0
    }
}
