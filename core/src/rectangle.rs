use crate::{Padding, Point, Radians, Size, Vector};

/// An axis-aligned rectangle.
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

impl<T> Rectangle<T>
where
    T: Default,
{
    /// Creates a new [`Rectangle`] with its top-left corner at the origin
    /// and with the provided [`Size`].
    pub fn with_size(size: Size<T>) -> Self {
        Self {
            x: T::default(),
            y: T::default(),
            width: size.width,
            height: size.height,
        }
    }
}

impl Rectangle<f32> {
    /// A rectangle starting at [`Point::ORIGIN`] with infinite width and height.
    pub const INFINITE: Self = Self::new(Point::ORIGIN, Size::INFINITY);

    /// Creates a new [`Rectangle`] with its top-left corner in the given
    /// [`Point`] and with the provided [`Size`].
    pub const fn new(top_left: Point, size: Size) -> Self {
        Self {
            x: top_left.x,
            y: top_left.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Creates a new square [`Rectangle`] with the center at the origin and
    /// with the given radius.
    pub fn with_radius(radius: f32) -> Self {
        Self {
            x: -radius,
            y: -radius,
            width: radius * 2.0,
            height: radius * 2.0,
        }
    }

    /// Creates a new axis-aligned [`Rectangle`] from the given vertices; returning the
    /// rotation in [`Radians`] that must be applied to the axis-aligned [`Rectangle`]
    /// to obtain the desired result.
    pub fn with_vertices(
        top_left: Point,
        top_right: Point,
        bottom_left: Point,
    ) -> (Rectangle, Radians) {
        let width = (top_right.x - top_left.x).hypot(top_right.y - top_left.y);

        let height =
            (bottom_left.x - top_left.x).hypot(bottom_left.y - top_left.y);

        let rotation =
            (top_right.y - top_left.y).atan2(top_right.x - top_left.x);

        let rotation = if rotation < 0.0 {
            2.0 * std::f32::consts::PI + rotation
        } else {
            rotation
        };

        let position = {
            let center = Point::new(
                (top_right.x + bottom_left.x) / 2.0,
                (top_right.y + bottom_left.y) / 2.0,
            );

            let rotation = -rotation - std::f32::consts::PI * 2.0;

            Point::new(
                center.x + (top_left.x - center.x) * rotation.cos()
                    - (top_left.y - center.y) * rotation.sin(),
                center.y
                    + (top_left.x - center.x) * rotation.sin()
                    + (top_left.y - center.y) * rotation.cos(),
            )
        };

        (
            Rectangle::new(position, Size::new(width, height)),
            Radians(rotation),
        )
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
            && point.x < self.x + self.width
            && self.y <= point.y
            && point.y < self.y + self.height
    }

    /// Returns the minimum distance from the given [`Point`] to any of the edges
    /// of the [`Rectangle`].
    pub fn distance(&self, point: Point) -> f32 {
        let center = self.center();

        let distance_x =
            ((point.x - center.x).abs() - self.width / 2.0).max(0.0);

        let distance_y =
            ((point.y - center.y).abs() - self.height / 2.0).max(0.0);

        distance_x.hypot(distance_y)
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
    pub fn snap(self) -> Option<Rectangle<u32>> {
        let width = self.width as u32;
        let height = self.height as u32;

        if width < 1 || height < 1 {
            return None;
        }

        Some(Rectangle {
            x: self.x as u32,
            y: self.y as u32,
            width,
            height,
        })
    }

    /// Expands the [`Rectangle`] a given amount.
    pub fn expand(self, padding: impl Into<Padding>) -> Self {
        let padding = padding.into();

        Self {
            x: self.x - padding.left,
            y: self.y - padding.top,
            width: self.width + padding.horizontal(),
            height: self.height + padding.vertical(),
        }
    }

    /// Shrinks the [`Rectangle`] a given amount.
    pub fn shrink(self, padding: impl Into<Padding>) -> Self {
        let padding = padding.into();

        Self {
            x: self.x + padding.left,
            y: self.y + padding.top,
            width: self.width - padding.horizontal(),
            height: self.height - padding.vertical(),
        }
    }

    /// Rotates the [`Rectangle`] and returns the smallest [`Rectangle`]
    /// containing it.
    pub fn rotate(self, rotation: Radians) -> Self {
        let size = self.size().rotate(rotation);
        let position = Point::new(
            self.center_x() - size.width / 2.0,
            self.center_y() - size.height / 2.0,
        );

        Self::new(position, size)
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

impl<T> std::ops::Mul<Vector<T>> for Rectangle<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Rectangle<T>;

    fn mul(self, scale: Vector<T>) -> Self {
        Rectangle {
            x: self.x * scale.x,
            y: self.y * scale.y,
            width: self.width * scale.x,
            height: self.height * scale.y,
        }
    }
}
