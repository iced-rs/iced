use iced_native::{Point, Rectangle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursor {
    Available(Point),
    Unavailable,
}

impl Cursor {
    // TODO: Remove this once this type is used in `iced_native` to encode
    // proper cursor availability
    pub(crate) fn from_window_position(position: Point) -> Self {
        if position.x < 0.0 || position.y < 0.0 {
            Cursor::Unavailable
        } else {
            Cursor::Available(position)
        }
    }

    pub fn position(&self) -> Option<Point> {
        match self {
            Cursor::Available(position) => Some(*position),
            Cursor::Unavailable => None,
        }
    }

    pub fn position_in(&self, bounds: &Rectangle) -> Option<Point> {
        if self.is_over(bounds) {
            self.position_from(bounds.position())
        } else {
            None
        }
    }

    pub fn position_from(&self, origin: Point) -> Option<Point> {
        match self {
            Cursor::Available(position) => {
                Some(Point::new(position.x - origin.x, position.y - origin.y))
            }
            Cursor::Unavailable => None,
        }
    }

    pub fn is_over(&self, bounds: &Rectangle) -> bool {
        match self {
            Cursor::Available(position) => bounds.contains(*position),
            Cursor::Unavailable => false,
        }
    }
}
