/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector<T = f32> {
    /// The X component of the [`Vector`]
    pub x: T,

    /// The Y component of the [`Vector`]
    pub y: T,
}

impl<T> Vector<T> {
    /// Creates a new [`Vector`] with the given components.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Vector {
    /// The zero [`Vector`].
    pub const ZERO: Self = Self::new(0.0, 0.0);
}

impl<T> Default for Vector<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T> From<[T; 2]> for Vector<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<Vector<T>> for [T; 2]
where
    T: Copy,
{
    fn from(other: Vector<T>) -> Self {
        [other.x, other.y]
    }
}

// Addition with own type
impl<T> std::ops::Add for Vector<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> std::ops::AddAssign for Vector<T>
where
    T: std::ops::AddAssign + Copy,
{
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

// Addition with scalar
impl<T> std::ops::Add<T> for Vector<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, other: T) -> Self::Output {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl<T> std::ops::AddAssign<T> for Vector<T>
where
    T: std::ops::AddAssign + Copy,
{
    fn add_assign(&mut self, other: T) {
        self.x += other;
        self.y += other;
    }
}

impl<T> std::ops::Div for Vector<T>
where
    T: std::ops::Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl<T> std::ops::DivAssign for Vector<T>
where
    T: std::ops::DivAssign + Copy,
{
    fn div_assign(&mut self, other: Self) {
        self.x /= other.x;
        self.y /= other.y;
    }
}

impl<T> std::ops::Div<T> for Vector<T>
where
    T: std::ops::Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: T) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl<T> std::ops::DivAssign<T> for Vector<T>
where
    T: std::ops::DivAssign + Copy,
{
    fn div_assign(&mut self, other: T) {
        self.x /= other;
        self.y /= other;
    }
}

impl<T> std::ops::Mul for Vector<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T> std::ops::MulAssign for Vector<T>
where
    T: std::ops::MulAssign + Copy,
{
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl<T> std::ops::Mul<T> for Vector<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: T) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<T> std::ops::MulAssign<T> for Vector<T>
where
    T: std::ops::MulAssign + Copy,
{
    fn mul_assign(&mut self, other: T) {
        self.x *= other;
        self.y *= other;
    }
}

impl<T> std::ops::Rem for Vector<T>
where
    T: std::ops::Rem<Output = T> + Copy,
{
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        Self {
            x: self.x % other.x,
            y: self.y % other.y,
        }
    }
}

impl<T> std::ops::RemAssign for Vector<T>
where
    T: std::ops::RemAssign + Copy,
{
    fn rem_assign(&mut self, other: Self) {
        self.x %= other.x;
        self.y %= other.y;
    }
}

impl<T> std::ops::Rem<T> for Vector<T>
where
    T: std::ops::Rem<Output = T> + Copy,
{
    type Output = Self;

    fn rem(self, other: T) -> Self {
        Self {
            x: self.x % other,
            y: self.y % other,
        }
    }
}

impl<T> std::ops::RemAssign<T> for Vector<T>
where
    T: std::ops::RemAssign + Copy,
{
    fn rem_assign(&mut self, other: T) {
        self.x %= other;
        self.y %= other;
    }
}

impl<T> std::ops::Sub for Vector<T>
where
    T: std::ops::Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> std::ops::SubAssign for Vector<T>
where
    T: std::ops::SubAssign + Copy,
{
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl<T> std::ops::Sub<T> for Vector<T>
where
    T: std::ops::Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, other: T) -> Self::Output {
        Self {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl<T> std::ops::SubAssign<T> for Vector<T>
where
    T: std::ops::SubAssign + Copy,
{
    fn sub_assign(&mut self, other: T) {
        self.x -= other;
        self.y -= other;
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add() {
        let mut a = Vector::new(1.0, 1.0);
        let other = Vector::new(10.0, 20.0);
        let r = Vector::new(11.0, 21.0);
        assert_eq!(r, a + other, "a + other self");
        a += other;
        assert_eq!(r, a, "a += other self");
        
        a = Vector::new(8.0, 18.0);
        let other = 3.0_f32;
        assert_eq!(r, a + other, "a + other scalar");
        a += other;
        assert_eq!(r, a, "a += other scalar");
    }

    #[test]
    fn test_div() {
        let mut a = Vector::new(1.0, 1.0);
        let other = Vector::new(10.0, 20.0);
        let r = Vector::new(0.1, 0.05);
        assert_eq!(r, a / other, "a / other self");
        a /= other;
        assert_eq!(r, a, "a /= other self");
        
        a = Vector::new(0.3, 0.15);
        let other = 3.0_f32;
        assert_eq!(r, a / other, "a / other scalar");
        a /= other;
        assert_eq!(r, a, "a /= other scalar");
    }

    #[test]
    fn test_mul() {
        let mut a = Vector::new(1.0, 1.0);
        let other = Vector::new(10.0, 20.0);
        let r = Vector::new(10.0, 20.0);
        assert_eq!(r, a * other, "a * other self");
        a *= other;
        assert_eq!(r, a, "a *= other self");
        
        a = Vector::new(5.0, 10.0);
        let other = 2.0_f32;
        assert_eq!(r, a * other, "a * other scalar");
        a *= other;
        assert_eq!(r, a, "a *= other scalar");
    }

    #[test]
    fn test_rem() {
        let mut a = Vector::new(1.0, 1.0);
        let other = Vector::new(10.0, 20.0);
        let r = Vector::new(1.0, 1.0);
        assert_eq!(r, a % other, "a % other self");
        a %= other;
        assert_eq!(r, a, "a %= other self");
        
        a = Vector::new(5.0, 0.8);
        let other = 0.5_f32;
        let r = Vector::new(0.0, 0.3);
        assert_eq!(r, a % other, "a % other scalar");
        a %= other;
        assert_eq!(r, a, "a %= other scalar");
    }

    #[test]
    fn test_sub() {
        let mut a = Vector::new(1.0, 1.0);
        let other = Vector::new(10.0, 20.0);
        let r = Vector::new(-9.0, -19.0);
        assert_eq!(r, a - other, "a - other self");
        a -= other;
        assert_eq!(r, a, "a -= other self");
        
        a = Vector::new(-6.0, -16.0);
        let other = 3.0_f32;
        assert_eq!(r, a - other, "a - other scalar");
        a -= other;
        assert_eq!(r, a, "a -= other scalar");
    }
}