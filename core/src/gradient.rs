//! Colors that transition progressively.
use crate::{Color, Radians};

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
/// A fill which transitions colors progressively along a direction, either linearly, radially (TBD),
/// or conically (TBD).
///
/// For a gradient which can be used as a fill on a canvas, see [`iced_graphics::Gradient`].
pub enum Gradient {
    /// A linear gradient interpolates colors along a direction at a specific [`Angle`].
    Linear(Linear),
}

impl Gradient {
    /// Adjust the opacity of the gradient by a multiplier applied to each color stop.
    pub fn mul_alpha(mut self, alpha_multiplier: f32) -> Self {
        match &mut self {
            Gradient::Linear(linear) => {
                for stop in linear.stops.iter_mut().flatten() {
                    stop.color.a *= alpha_multiplier;
                }
            }
        }

        self
    }
}

impl From<Linear> for Gradient {
    fn from(gradient: Linear) -> Self {
        Self::Linear(gradient)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
/// A point along the gradient vector where the specified [`color`] is unmixed.
///
/// [`color`]: Self::color
pub struct ColorStop {
    /// Offset along the gradient vector.
    pub offset: f32,

    /// The color of the gradient at the specified [`offset`].
    ///
    /// [`offset`]: Self::offset
    pub color: Color,
}

/// A linear gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    /// How the [`Gradient`] is angled within its bounds.
    pub angle: Radians,
    /// [`ColorStop`]s along the linear gradient path.
    pub stops: [Option<ColorStop>; 8],
}

impl Linear {
    /// Creates a new [`Linear`] gradient with the given angle in [`Radians`].
    pub fn new(angle: impl Into<Radians>) -> Self {
        Self {
            angle: angle.into(),
            stops: [None; 8],
        }
    }

    /// Adds a new [`ColorStop`], defined by an offset and a color, to the gradient.
    ///
    /// Any `offset` that is not within `0.0..=1.0` will be silently ignored.
    ///
    /// Any stop added after the 8th will be silently ignored.
    pub fn add_stop(mut self, offset: f32, color: Color) -> Self {
        if offset.is_finite() && (0.0..=1.0).contains(&offset) {
            let (Ok(index) | Err(index)) =
                self.stops.binary_search_by(|stop| match stop {
                    None => Ordering::Greater,
                    Some(stop) => stop.offset.partial_cmp(&offset).unwrap(),
                });

            if index < 8 {
                self.stops[index] = Some(ColorStop { offset, color });
            }
        } else {
            log::warn!("Gradient color stop must be within 0.0..=1.0 range.");
        };

        self
    }

    /// Adds multiple [`ColorStop`]s to the gradient.
    ///
    /// Any stop added after the 8th will be silently ignored.
    pub fn add_stops(
        mut self,
        stops: impl IntoIterator<Item = ColorStop>,
    ) -> Self {
        for stop in stops.into_iter() {
            self = self.add_stop(stop.offset, stop.color)
        }

        self
    }
}
