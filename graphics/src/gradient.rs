//! A gradient that can be used as a [`Fill`] for some geometry.
//!
//! For a gradient that you can use as a background variant for a widget, see [`Gradient`].
//!
//! [`Gradient`]: crate::core::Gradient;
use crate::color;
use crate::core::gradient::ColorStop;
use crate::core::{self, Color, Point, Rectangle};

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
/// A fill which linearly interpolates colors along a direction.
///
/// For a gradient which can be used as a fill for a background of a widget, see [`crate::core::Gradient`].
pub enum Gradient {
    /// A linear gradient interpolates colors along a direction from its `start` to its `end`
    /// point.
    Linear(Linear),
}

impl From<Linear> for Gradient {
    fn from(gradient: Linear) -> Self {
        Self::Linear(gradient)
    }
}

impl Gradient {
    /// Packs the [`Gradient`] for use in shader code.
    pub fn pack(&self) -> Packed {
        match self {
            Gradient::Linear(linear) => linear.pack(),
        }
    }
}

/// A linear gradient that can be used in the style of [`Fill`] or [`Stroke`].
///
/// [`Fill`]: crate::geometry::Fill;
/// [`Stroke`]: crate::geometry::Stroke;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    /// The absolute starting position of the gradient.
    pub start: Point,

    /// The absolute ending position of the gradient.
    pub end: Point,

    /// [`ColorStop`]s along the linear gradient direction.
    pub stops: [Option<ColorStop>; 8],
}

impl Linear {
    /// Creates a new [`Builder`].
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
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
            log::warn!("Gradient: ColorStop must be within 0.0..=1.0 range.");
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

    /// Packs the [`Gradient`] for use in shader code.
    pub fn pack(&self) -> Packed {
        let mut data: [f32; 44] = [0.0; 44];

        for (index, stop) in self.stops.iter().enumerate() {
            let [r, g, b, a] =
                color::pack(stop.map_or(Color::default(), |s| s.color))
                    .components();

            data[index * 4] = r;
            data[(index * 4) + 1] = g;
            data[(index * 4) + 2] = b;
            data[(index * 4) + 3] = a;

            data[32 + index] = stop.map_or(2.0, |s| s.offset);
        }

        data[40] = self.start.x;
        data[41] = self.start.y;
        data[42] = self.end.x;
        data[43] = self.end.y;

        Packed(data)
    }
}

/// Packed [`Gradient`] data for use in shader code.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Packed([f32; 44]);

/// Creates a new [`Packed`] gradient for use in shader code.
pub fn pack(gradient: &core::Gradient, bounds: Rectangle) -> Packed {
    match gradient {
        core::Gradient::Linear(linear) => {
            let mut data: [f32; 44] = [0.0; 44];

            for (index, stop) in linear.stops.iter().enumerate() {
                let [r, g, b, a] =
                    color::pack(stop.map_or(Color::default(), |s| s.color))
                        .components();

                data[index * 4] = r;
                data[(index * 4) + 1] = g;
                data[(index * 4) + 2] = b;
                data[(index * 4) + 3] = a;
                data[32 + index] = stop.map_or(2.0, |s| s.offset);
            }

            let (start, end) = linear.angle.to_distance(&bounds);

            data[40] = start.x;
            data[41] = start.y;
            data[42] = end.x;
            data[43] = end.y;

            Packed(data)
        }
    }
}
