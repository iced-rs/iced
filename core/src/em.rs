/// A value in EM units.
///
/// An EM is a typographic unit of measurement equal to the current font size.
/// It's commonly used for sizing that scales proportionally with text, such as
/// margins, padding, widths, and spacing.
///
/// This type is normally used as an argument in a generic way
/// (e.g. `impl Into<Em>`) and, since `Em` implements `From` both for
/// `f32` and integer types, you should be able to provide both integers and float
/// literals as needed.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Em(pub f32);

impl Em {
    /// Zero EM units.
    pub const ZERO: Self = Self(0.0);
}

impl From<f32> for Em {
    fn from(amount: f32) -> Self {
        Self(amount)
    }
}

impl From<i32> for Em {
    fn from(amount: i32) -> Self {
        Self(amount as f32)
    }
}

impl From<u32> for Em {
    fn from(amount: u32) -> Self {
        Self(amount as f32)
    }
}

impl From<usize> for Em {
    fn from(amount: usize) -> Self {
        Self(amount as f32)
    }
}

impl From<Em> for f32 {
    fn from(spacing: Em) -> Self {
        spacing.0
    }
}
