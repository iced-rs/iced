/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

    /// Rounds the [`Vector`].
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
}

impl<T> std::ops::Neg for Vector<T>
where
    T: std::ops::Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T> std::ops::Add for Vector<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
}

impl<T> std::ops::AddAssign for Vector<T>
where
    T: std::ops::AddAssign,
{
    fn add_assign(&mut self, b: Self) {
        self.x += b.x;
        self.y += b.y;
    }
}

impl<T> std::ops::Sub for Vector<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
}

impl<T> std::ops::SubAssign for Vector<T>
where
    T: std::ops::SubAssign,
{
    fn sub_assign(&mut self, b: Self) {
        self.x -= b.x;
        self.y -= b.y;
    }
}

impl<T> std::ops::Mul<T> for Vector<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, scale: T) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }
}

impl<T> std::ops::MulAssign<T> for Vector<T>
where
    T: std::ops::MulAssign + Copy,
{
    fn mul_assign(&mut self, scale: T) {
        self.x *= scale;
        self.y *= scale;
    }
}

impl<T> std::ops::Div<T> for Vector<T>
where
    T: std::ops::Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, scale: T) -> Self {
        Self::new(self.x / scale, self.y / scale)
    }
}

impl<T> std::ops::DivAssign<T> for Vector<T>
where
    T: std::ops::DivAssign + Copy,
{
    fn div_assign(&mut self, scale: T) {
        self.x /= scale;
        self.y /= scale;
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
