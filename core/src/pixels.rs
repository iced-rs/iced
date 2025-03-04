/// An amount of logical pixels.
///
/// Normally used to represent an amount of space, or the size of something.
///
/// This type is normally asked as an argument in a generic way
/// (e.g. `impl Into<Pixels>`) and, since `Pixels` implements `From` both for
/// `f32` and `u16`, you should be able to provide both integers and float
/// literals as needed.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Pixels(pub f32);

impl Pixels {
    /// Zero pixels.
    pub const ZERO: Self = Self(0.0);
}

impl From<f32> for Pixels {
    fn from(amount: f32) -> Self {
        Self(amount)
    }
}

impl From<u32> for Pixels {
    fn from(amount: u32) -> Self {
        Self(amount as f32)
    }
}

impl From<Pixels> for f32 {
    fn from(pixels: Pixels) -> Self {
        pixels.0
    }
}

impl std::ops::Add for Pixels {
    type Output = Pixels;

    fn add(self, rhs: Self) -> Self {
        Pixels(self.0 + rhs.0)
    }
}

impl std::ops::Add<f32> for Pixels {
    type Output = Pixels;

    fn add(self, rhs: f32) -> Self {
        Pixels(self.0 + rhs)
    }
}

impl std::ops::Mul for Pixels {
    type Output = Pixels;

    fn mul(self, rhs: Self) -> Self {
        Pixels(self.0 * rhs.0)
    }
}

impl std::ops::Mul<f32> for Pixels {
    type Output = Pixels;

    fn mul(self, rhs: f32) -> Self {
        Pixels(self.0 * rhs)
    }
}

impl std::ops::Div for Pixels {
    type Output = Pixels;

    fn div(self, rhs: Self) -> Self {
        Pixels(self.0 / rhs.0)
    }
}

impl std::ops::Div<f32> for Pixels {
    type Output = Pixels;

    fn div(self, rhs: f32) -> Self {
        Pixels(self.0 / rhs)
    }
}

impl std::ops::Div<u32> for Pixels {
    type Output = Pixels;

    fn div(self, rhs: u32) -> Self {
        Pixels(self.0 / rhs as f32)
    }
}
