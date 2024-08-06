//! Draw lines around containers.
use crate::{Color, Pixels};

/// A border.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Border {
    /// The color of the border.
    pub color: Color,

    /// The width of the border.
    pub width: f32,

    /// The [`Radius`] of the border.
    pub radius: Radius,
}

/// Creates a new [`Border`] with the given [`Radius`].
///
/// ```
/// # use iced_core::border::{self, Border};
/// #
/// assert_eq!(border::rounded(10), Border::default().rounded(10));
/// ```
pub fn rounded(radius: impl Into<Radius>) -> Border {
    Border::default().rounded(radius)
}

/// Creates a new [`Border`] with the given [`Color`].
///
/// ```
/// # use iced_core::border::{self, Border};
/// # use iced_core::Color;
/// #
/// assert_eq!(border::color(Color::BLACK), Border::default().color(Color::BLACK));
/// ```
pub fn color(color: impl Into<Color>) -> Border {
    Border::default().color(color)
}

/// Creates a new [`Border`] with the given `width`.
///
/// ```
/// # use iced_core::border::{self, Border};
/// # use iced_core::Color;
/// #
/// assert_eq!(border::width(10), Border::default().width(10));
/// ```
pub fn width(width: impl Into<Pixels>) -> Border {
    Border::default().width(width)
}

impl Border {
    /// Sets the [`Color`] of the [`Border`].
    pub fn color(self, color: impl Into<Color>) -> Self {
        Self {
            color: color.into(),
            ..self
        }
    }

    /// Sets the [`Radius`] of the [`Border`].
    pub fn rounded(self, radius: impl Into<Radius>) -> Self {
        Self {
            radius: radius.into(),
            ..self
        }
    }

    /// Sets the width of the [`Border`].
    pub fn width(self, width: impl Into<Pixels>) -> Self {
        Self {
            width: width.into().0,
            ..self
        }
    }
}

/// The border radii for the corners of a graphics primitive in the order:
/// top-left, top-right, bottom-right, bottom-left.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radius {
    /// Top left radius
    pub top_left: f32,
    /// Top right radius
    pub top_right: f32,
    /// Bottom right radius
    pub bottom_right: f32,
    /// Bottom left radius
    pub bottom_left: f32,
}

/// Creates a new [`Radius`] with the same value for each corner.
pub fn radius(value: impl Into<Pixels>) -> Radius {
    Radius::new(value)
}

/// Creates a new [`Radius`] with the given top left value.
pub fn top_left(value: impl Into<Pixels>) -> Radius {
    Radius::default().top_left(value)
}

/// Creates a new [`Radius`] with the given top right value.
pub fn top_right(value: impl Into<Pixels>) -> Radius {
    Radius::default().top_right(value)
}

/// Creates a new [`Radius`] with the given bottom right value.
pub fn bottom_right(value: impl Into<Pixels>) -> Radius {
    Radius::default().bottom_right(value)
}

/// Creates a new [`Radius`] with the given bottom left value.
pub fn bottom_left(value: impl Into<Pixels>) -> Radius {
    Radius::default().bottom_left(value)
}

/// Creates a new [`Radius`] with the given value as top left and top right.
pub fn top(value: impl Into<Pixels>) -> Radius {
    Radius::default().top(value)
}

/// Creates a new [`Radius`] with the given value as bottom left and bottom right.
pub fn bottom(value: impl Into<Pixels>) -> Radius {
    Radius::default().bottom(value)
}

/// Creates a new [`Radius`] with the given value as top left and bottom left.
pub fn left(value: impl Into<Pixels>) -> Radius {
    Radius::default().left(value)
}

/// Creates a new [`Radius`] with the given value as top right and bottom right.
pub fn right(value: impl Into<Pixels>) -> Radius {
    Radius::default().right(value)
}

impl Radius {
    /// Creates a new [`Radius`] with the same value for each corner.
    pub fn new(value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }

    /// Sets the top left value of the [`Radius`].
    pub fn top_left(self, value: impl Into<Pixels>) -> Self {
        Self {
            top_left: value.into().0,
            ..self
        }
    }

    /// Sets the top right value of the [`Radius`].
    pub fn top_right(self, value: impl Into<Pixels>) -> Self {
        Self {
            top_right: value.into().0,
            ..self
        }
    }

    /// Sets the bottom right value of the [`Radius`].
    pub fn bottom_right(self, value: impl Into<Pixels>) -> Self {
        Self {
            bottom_right: value.into().0,
            ..self
        }
    }

    /// Sets the bottom left value of the [`Radius`].
    pub fn bottom_left(self, value: impl Into<Pixels>) -> Self {
        Self {
            bottom_left: value.into().0,
            ..self
        }
    }

    /// Sets the top left and top right values of the [`Radius`].
    pub fn top(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_left: value,
            top_right: value,
            ..self
        }
    }

    /// Sets the bottom left and bottom right values of the [`Radius`].
    pub fn bottom(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            bottom_left: value,
            bottom_right: value,
            ..self
        }
    }

    /// Sets the top left and bottom left values of the [`Radius`].
    pub fn left(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_left: value,
            bottom_left: value,
            ..self
        }
    }

    /// Sets the top right and bottom right values of the [`Radius`].
    pub fn right(self, value: impl Into<Pixels>) -> Self {
        let value = value.into().0;

        Self {
            top_right: value,
            bottom_right: value,
            ..self
        }
    }
}

impl From<f32> for Radius {
    fn from(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

impl From<u8> for Radius {
    fn from(w: u8) -> Self {
        Self::from(f32::from(w))
    }
}

impl From<u16> for Radius {
    fn from(w: u16) -> Self {
        Self::from(f32::from(w))
    }
}

impl From<i32> for Radius {
    fn from(w: i32) -> Self {
        Self::from(w as f32)
    }
}

impl From<Radius> for [f32; 4] {
    fn from(radi: Radius) -> Self {
        [
            radi.top_left,
            radi.top_right,
            radi.bottom_right,
            radi.bottom_left,
        ]
    }
}
