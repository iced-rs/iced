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

    /// Overrides for the top edge of the border.
    pub top: Side,

    /// Overrides for the right edge of the border.
    pub right: Side,

    /// Overrides for the bottom edge of the border.
    pub bottom: Side,

    /// Overrides for the left edge of the border.
    pub left: Side,
}

/// Optional overrides for one side of a [`Border`].
///
/// A [`Side`] inherits the border's [`Border::color`] and [`Border::width`]
/// when its corresponding value is `None`.
///
/// This is useful for patterns such as a collapsible header, where the open
/// state can keep only a bottom divider:
///
/// ```
/// # use iced_core::{border, Color};
/// let header = border::color(Color::BLACK)
///     .width(1)
///     .bottom(border::Side::default().color(Color::from_rgb(0.2, 0.4, 1.0)));
/// let collapsed = header.bottom(border::Side::default().width(0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Side {
    /// An optional color override for this side.
    pub color: Option<Color>,

    /// An optional width override for this side.
    pub width: Option<f32>,
}

impl Side {
    /// Sets the color override for this side.
    pub fn color(self, color: impl Into<Color>) -> Self {
        Self {
            color: Some(color.into()),
            ..self
        }
    }

    /// Sets the width override for this side.
    pub fn width(self, width: impl Into<Pixels>) -> Self {
        Self {
            width: Some(width.into().0),
            ..self
        }
    }
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

    /// Sets the overrides for the top edge of the [`Border`].
    pub fn top(self, top: Side) -> Self {
        Self { top, ..self }
    }

    /// Sets the overrides for the right edge of the [`Border`].
    pub fn right(self, right: Side) -> Self {
        Self { right, ..self }
    }

    /// Sets the overrides for the bottom edge of the [`Border`].
    pub fn bottom(self, bottom: Side) -> Self {
        Self { bottom, ..self }
    }

    /// Sets the overrides for the left edge of the [`Border`].
    pub fn left(self, left: Side) -> Self {
        Self { left, ..self }
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

impl From<u32> for Radius {
    fn from(w: u32) -> Self {
        Self::from(w as f32)
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

impl std::ops::Mul<f32> for Radius {
    type Output = Self;

    fn mul(self, scale: f32) -> Self::Output {
        Self {
            top_left: self.top_left * scale,
            top_right: self.top_right * scale,
            bottom_right: self.bottom_right * scale,
            bottom_left: self.bottom_left * scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sides_fall_back_to_the_uniform_values() {
        let border = Border::default().color(Color::BLACK).width(2);

        assert_eq!(border.top.color.unwrap_or(border.color), Color::BLACK);
        assert_eq!(border.right.width.unwrap_or(border.width), 2.0);
    }

    #[test]
    fn sides_override_color_and_width_independently() {
        let border = Border::default()
            .color(Color::BLACK)
            .width(2)
            .left(Side::default().color(Color::WHITE))
            .right(Side::default().width(4));

        assert_eq!(border.left.color, Some(Color::WHITE));
        assert_eq!(border.left.width, None);
        assert_eq!(border.right.color, None);
        assert_eq!(border.right.width, Some(4.0));
    }

    #[test]
    fn uniform_builders_do_not_replace_side_overrides() {
        let side = Side::default().color(Color::WHITE).width(4);
        let before = Border::default().left(side).color(Color::BLACK).width(2);
        let after = Border::default().color(Color::BLACK).width(2).left(side);

        assert_eq!(before, after);
    }

    #[test]
    fn replacing_a_side_with_default_clears_its_overrides() {
        let border = Border::default()
            .top(Side::default().color(Color::WHITE).width(3))
            .top(Side::default());

        assert_eq!(border.top, Side::default());
    }

    #[test]
    fn zero_width_is_a_valid_side_override() {
        let border = Border::default().width(2).bottom(Side::default().width(0));

        assert_eq!(border.bottom.width.unwrap_or(border.width), 0.0);
    }
}
