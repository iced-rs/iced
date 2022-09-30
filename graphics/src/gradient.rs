//! For creating a Gradient.
use iced_native::Color;
use crate::gradient::linear::Linear;
use crate::Point;

#[derive(Debug, Clone, PartialEq)]
/// A fill which transitions colors progressively along a direction, either linearly, radially (TBD),
/// or conically (TBD).
pub enum Gradient {
    /// A linear gradient interpolates colors along a direction from its [`start`] to its [`end`]
    /// point.
    Linear(Linear),
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A point along the gradient vector where the specified [`color`] is unmixed.
pub struct ColorStop {
    /// Offset along the gradient vector.
    pub offset: f32,
    /// The color of the gradient at the specified [`offset`].
    pub color: Color,
}

impl Gradient {
    /// Creates a new linear [`linear::Builder`].
    pub fn linear(start: Point, end: Point) -> linear::Builder {
        linear::Builder::new(start, end)
    }
}

/// Linear gradient builder & definition.
pub mod linear {
    use crate::gradient::{ColorStop, Gradient};
    use crate::{Color, Point};

    /// A linear gradient that can be used in the style of [`super::Fill`] or [`super::Stroke`].
    #[derive(Debug, Clone, PartialEq)]
    pub struct Linear {
        /// The point where the linear gradient begins.
        pub start: Point,
        /// The point where the linear gradient ends.
        pub end: Point,
        /// [`ColorStop`]s along the linear gradient path.
        pub color_stops: Vec<ColorStop>,
    }

    /// A [`Linear`] builder.
    #[derive(Debug)]
    pub struct Builder {
        start: Point,
        end: Point,
        stops: Vec<(f32, Color)>,
        valid: bool,
    }

    impl Builder {
        /// Creates a new [`Builder`].
        pub fn new(start: Point, end: Point) -> Self {
            Self {
                start,
                end,
                stops: vec![],
                valid: true,
            }
        }

        /// Adds a new stop, defined by an offset and a color, to the gradient.
        ///
        /// `offset` must be between `0.0` and `1.0`.
        pub fn add_stop(mut self, offset: f32, color: Color) -> Self {
            if !(0.0..=1.0).contains(&offset) {
                self.valid = false;
            }

            self.stops.push((offset, color));
            self
        }

        /// Builds the linear [`Gradient`] of this [`Builder`].
        ///
        /// Returns `None` if no stops were added to the builder or
        /// if stops not between 0.0 and 1.0 were added.
        pub fn build(self) -> Option<Gradient> {
            if self.stops.is_empty() || !self.valid {
                return None;
            }

            let mut stops: Vec<ColorStop> = self.stops.clone().into_iter().map(|f| ColorStop {
                offset: f.0,
                color: f.1
            }).collect();

            stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());

            Some(Gradient::Linear(Linear {
                start: self.start,
                end: self.end,
                color_stops: stops
            }))
        }
    }
}

