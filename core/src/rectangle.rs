use crate::{Point, Size, Vector};

/// A rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rectangle<T = f32> {
    /// X coordinate of the top-left corner.
    pub x: T,

    /// Y coordinate of the top-left corner.
    pub y: T,

    /// Width of the rectangle.
    pub width: T,

    /// Height of the rectangle.
    pub height: T,
}

impl Rectangle<f32> {
    /// Creates a new [`Rectangle`] with its top-left corner in the given
    /// [`Point`] and with the provided [`Size`].
    pub fn new(top_left: Point, size: Size) -> Self {
        Self {
            x: top_left.x,
            y: top_left.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Creates a new [`Rectangle`] with its top-left corner at the origin
    /// and with the provided [`Size`].
    pub fn with_size(size: Size) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        }
    }

    /// Returns the [`Point`] at the center of the [`Rectangle`].
    pub fn center(&self) -> Point {
        Point::new(self.center_x(), self.center_y())
    }

    /// Returns the X coordinate of the [`Point`] at the center of the
    /// [`Rectangle`].
    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    /// Returns the Y coordinate of the [`Point`] at the center of the
    /// [`Rectangle`].
    pub fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }

    /// Returns the position of the top left corner of the [`Rectangle`].
    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Returns the [`Size`] of the [`Rectangle`].
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns the area of the [`Rectangle`].
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Returns true if the given [`Point`] is contained in the [`Rectangle`].
    pub fn contains(&self, point: Point) -> bool {
        self.x <= point.x
            && point.x <= self.x + self.width
            && self.y <= point.y
            && point.y <= self.y + self.height
    }

    /// Returns true if the current [`Rectangle`] is completely within the given
    /// `container`.
    pub fn is_within(&self, container: &Rectangle) -> bool {
        container.contains(self.position())
            && container.contains(
                self.position() + Vector::new(self.width, self.height),
            )
    }

    /// Computes the intersection with the given [`Rectangle`].
    pub fn intersection(
        &self,
        other: &Rectangle<f32>,
    ) -> Option<Rectangle<f32>> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);

        let lower_right_x = (self.x + self.width).min(other.x + other.width);
        let lower_right_y = (self.y + self.height).min(other.y + other.height);

        let width = lower_right_x - x;
        let height = lower_right_y - y;

        if width > 0.0 && height > 0.0 {
            Some(Rectangle {
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }

    /// Returns whether the [`Rectangle`] intersects with the given one.
    pub fn intersects(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }

    /// Computes the union with the given [`Rectangle`].
    pub fn union(&self, other: &Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);

        let lower_right_x = (self.x + self.width).max(other.x + other.width);
        let lower_right_y = (self.y + self.height).max(other.y + other.height);

        let width = lower_right_x - x;
        let height = lower_right_y - y;

        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Snaps the [`Rectangle`] to __unsigned__ integer coordinates.
    pub fn snap(self) -> Rectangle<u32> {
        Rectangle {
            x: self.x as u32,
            y: self.y as u32,
            width: self.width as u32,
            height: self.height as u32,
        }
    }

    /// Expands the [`Rectangle`] a given amount.
    pub fn expand(self, amount: f32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            width: self.width + amount * 2.0,
            height: self.height + amount * 2.0,
        }
    }
}

impl std::ops::Mul<f32> for Rectangle<f32> {
    type Output = Self;

    fn mul(self, scale: f32) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

impl From<Rectangle<u32>> for Rectangle<f32> {
    fn from(rectangle: Rectangle<u32>) -> Rectangle<f32> {
        Rectangle {
            x: rectangle.x as f32,
            y: rectangle.y as f32,
            width: rectangle.width as f32,
            height: rectangle.height as f32,
        }
    }
}

impl<T> std::ops::Add<Vector<T>> for Rectangle<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Rectangle<T>;

    fn add(self, translation: Vector<T>) -> Self {
        Rectangle {
            x: self.x + translation.x,
            y: self.y + translation.y,
            ..self
        }
    }
}

impl<T> std::ops::Sub<Vector<T>> for Rectangle<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Rectangle<T>;

    fn sub(self, translation: Vector<T>) -> Self {
        Rectangle {
            x: self.x - translation.x,
            y: self.y - translation.y,
            ..self
        }
    }
}
