use crate::{Padding, Vector};

/// An amount of space in 2 dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size<T = f32> {
    /// The width.
    pub width: T,
    /// The height.
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new  [`Size`] with the given width and height.
    pub const fn new(width: T, height: T) -> Self {
        Size { width, height }
    }
}

impl Size {
    /// A [`Size`] with zero width and height.
    pub const ZERO: Size = Size::new(0., 0.);

    /// A [`Size`] with a width and height of 1 unit.
    pub const UNIT: Size = Size::new(1., 1.);

    /// A [`Size`] with infinite width and height.
    pub const INFINITY: Size = Size::new(f32::INFINITY, f32::INFINITY);

    /// Increments the [`Size`] to account for the given padding.
    pub fn pad(&self, padding: Padding) -> Self {
        Size {
            width: self.width + padding.horizontal(),
            height: self.height + padding.vertical(),
        }
    }

    /// Returns the minimum of each component of this size and another
    pub fn min(self, other: Self) -> Self {
        Size {
            width: self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }

    /// Returns the maximum of each component of this size and another
    pub fn max(self, other: Self) -> Self {
        Size {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Size { width, height }
    }
}

impl From<[u16; 2]> for Size {
    fn from([width, height]: [u16; 2]) -> Self {
        Size::new(width.into(), height.into())
    }
}

impl From<Vector<f32>> for Size {
    fn from(vector: Vector<f32>) -> Self {
        Size {
            width: vector.x,
            height: vector.y,
        }
    }
}

impl From<Size> for [f32; 2] {
    fn from(size: Size) -> [f32; 2] {
        [size.width, size.height]
    }
}

impl From<Size> for Vector<f32> {
    fn from(size: Size) -> Self {
        Vector::new(size.width, size.height)
    }
}

// Addition with own type
impl<T> std::ops::Add for Size<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl<T> std::ops::AddAssign for Size<T>
where
    T: std::ops::AddAssign + Copy,
{
    fn add_assign(&mut self, other: Self) {
        self.width += other.width;
        self.height += other.height;
    }
}

// Addition with scalar
impl<T> std::ops::Add<T> for Size<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, other: T) -> Self::Output {
        Self {
            width: self.width + other,
            height: self.height + other,
        }
    }
}

impl<T> std::ops::AddAssign<T> for Size<T>
where
    T: std::ops::AddAssign + Copy,
{
    fn add_assign(&mut self, other: T) {
        self.width += other;
        self.height += other;
    }
}

impl<T> std::ops::Div for Size<T>
where
    T: std::ops::Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            width: self.width / other.width,
            height: self.height / other.height,
        }
    }
}

impl<T> std::ops::DivAssign for Size<T>
where
    T: std::ops::DivAssign + Copy,
{
    fn div_assign(&mut self, other: Self) {
        self.width /= other.width;
        self.height /= other.height;
    }
}

impl<T> std::ops::Div<T> for Size<T>
where
    T: std::ops::Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: T) -> Self {
        Self {
            width: self.width / other,
            height: self.height / other,
        }
    }
}

impl<T> std::ops::DivAssign<T> for Size<T>
where
    T: std::ops::DivAssign + Copy,
{
    fn div_assign(&mut self, other: T) {
        self.width /= other;
        self.height /= other;
    }
}

impl<T> std::ops::Mul for Size<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            width: self.width * other.width,
            height: self.height * other.height,
        }
    }
}

impl<T> std::ops::MulAssign for Size<T>
where
    T: std::ops::MulAssign + Copy,
{
    fn mul_assign(&mut self, other: Self) {
        self.width *= other.width;
        self.height *= other.height;
    }
}

impl<T> std::ops::Mul<T> for Size<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: T) -> Self {
        Self {
            width: self.width * other,
            height: self.height * other,
        }
    }
}

impl<T> std::ops::MulAssign<T> for Size<T>
where
    T: std::ops::MulAssign + Copy,
{
    fn mul_assign(&mut self, other: T) {
        self.width *= other;
        self.height *= other;
    }
}

impl<T> std::ops::Rem for Size<T>
where
    T: std::ops::Rem<Output = T> + Copy,
{
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        Self {
            width: self.width % other.width,
            height: self.height % other.height,
        }
    }
}

impl<T> std::ops::RemAssign for Size<T>
where
    T: std::ops::RemAssign + Copy,
{
    fn rem_assign(&mut self, other: Self) {
        self.width %= other.width;
        self.height %= other.height;
    }
}

impl<T> std::ops::Rem<T> for Size<T>
where
    T: std::ops::Rem<Output = T> + Copy,
{
    type Output = Self;

    fn rem(self, other: T) -> Self {
        Self {
            width: self.width % other,
            height: self.height % other,
        }
    }
}

impl<T> std::ops::RemAssign<T> for Size<T>
where
    T: std::ops::RemAssign + Copy,
{
    fn rem_assign(&mut self, other: T) {
        self.width %= other;
        self.height %= other;
    }
}

impl<T> std::ops::Sub for Size<T>
where
    T: std::ops::Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}

impl<T> std::ops::SubAssign for Size<T>
where
    T: std::ops::SubAssign + Copy,
{
    fn sub_assign(&mut self, other: Self) {
        self.width -= other.width;
        self.height -= other.height;
    }
}

impl<T> std::ops::Sub<T> for Size<T>
where
    T: std::ops::Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, other: T) -> Self::Output {
        Self {
            width: self.width - other,
            height: self.height - other,
        }
    }
}

impl<T> std::ops::SubAssign<T> for Size<T>
where
    T: std::ops::SubAssign + Copy,
{
    fn sub_assign(&mut self, other: T) {
        self.width -= other;
        self.height -= other;
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add() {
        let mut a = Size::UNIT;
        let other = Size::new(10.0, 20.0);
        let r = Size::new(11.0, 21.0);
        assert_eq!(r, a + other, "a + other self");
        a += other;
        assert_eq!(r, a, "a += other self");
        
        a = Size::new(8.0, 18.0);
        let other = 3.0_f32;
        assert_eq!(r, a + other, "a + other scalar");
        a += other;
        assert_eq!(r, a, "a += other scalar");
    }

    #[test]
    fn test_div() {
        let mut a = Size::UNIT;
        let other = Size::new(10.0, 20.0);
        let r = Size::new(0.1, 0.05);
        assert_eq!(r, a / other, "a / other self");
        a /= other;
        assert_eq!(r, a, "a /= other self");
        
        a = Size::new(0.3, 0.15);
        let other = 3.0_f32;
        assert_eq!(r, a / other, "a / other scalar");
        a /= other;
        assert_eq!(r, a, "a /= other scalar");
    }

    #[test]
    fn test_mul() {
        let mut a = Size::UNIT;
        let other = Size::new(10.0, 20.0);
        let r = Size::new(10.0, 20.0);
        assert_eq!(r, a * other, "a * other self");
        a *= other;
        assert_eq!(r, a, "a *= other self");
        
        a = Size::new(5.0, 10.0);
        let other = 2.0_f32;
        assert_eq!(r, a * other, "a * other scalar");
        a *= other;
        assert_eq!(r, a, "a *= other scalar");
    }

    #[test]
    fn test_rem() {
        let mut a = Size::UNIT;
        let other = Size::new(10.0, 20.0);
        let r = Size::new(1.0, 1.0);
        assert_eq!(r, a % other, "a % other self");
        a %= other;
        assert_eq!(r, a, "a %= other self");
        
        a = Size::new(5.0, 0.8);
        let other = 0.5_f32;
        let r = Size::new(0.0, 0.3);
        assert_eq!(r, a % other, "a % other scalar");
        a %= other;
        assert_eq!(r, a, "a %= other scalar");
    }

    #[test]
    fn test_sub() {
        let mut a = Size::UNIT;
        let other = Size::new(10.0, 20.0);
        let r = Size::new(-9.0, -19.0);
        assert_eq!(r, a - other, "a - other self");
        a -= other;
        assert_eq!(r, a, "a -= other self");
        
        a = Size::new(-6.0, -16.0);
        let other = 3.0_f32;
        assert_eq!(r, a - other, "a - other scalar");
        a -= other;
        assert_eq!(r, a, "a -= other scalar");
    }
}