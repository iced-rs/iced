//! For creating a Gradient that can be used as a [`Fill`] for a mesh.
use crate::core::Point;
pub use linear::Linear;

#[derive(Debug, Clone, PartialEq)]
/// A fill which transitions colors progressively along a direction, either linearly, radially (TBD),
/// or conically (TBD).
///
/// For a gradient which can be used as a fill for a background of a widget, see [`core::Gradient`].
pub enum Gradient {
    /// A linear gradient interpolates colors along a direction from its `start` to its `end`
    /// point.
    Linear(Linear),
}

impl Gradient {
    /// Creates a new linear [`linear::Builder`].
    ///
    /// The `start` and `end` [`Point`]s define the absolute position of the [`Gradient`].
    pub fn linear(start: Point, end: Point) -> linear::Builder {
        linear::Builder::new(start, end)
    }
}

pub mod linear {
    //! Linear gradient builder & definition.
    use crate::Gradient;
    use iced_core::gradient::ColorStop;
    use iced_core::{Color, Point};
    use std::cmp::Ordering;

    /// A linear gradient that can be used in the style of [`Fill`] or [`Stroke`].
    ///
    /// [`Fill`]: crate::widget::canvas::Fill
    /// [`Stroke`]: crate::widget::canvas::Stroke
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Linear {
        /// The absolute starting position of the gradient.
        pub start: Point,

        /// The absolute ending position of the gradient.
        pub end: Point,

        /// [`ColorStop`]s along the linear gradient path.
        pub color_stops: [Option<ColorStop>; 8],
    }

    /// A [`Linear`] builder.
    #[derive(Debug)]
    pub struct Builder {
        start: Point,
        end: Point,
        stops: [Option<ColorStop>; 8],
    }

    impl Builder {
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
        /// `offset` must be between `0.0` and `1.0` or the gradient cannot be built.
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
                log::warn!(
                    "Gradient: ColorStop must be within 0.0..=1.0 range."
                );
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

        /// Builds the linear [`Gradient`] of this [`Builder`].
        ///
        /// Returns `BuilderError` if gradient in invalid.
        pub fn build(self) -> Gradient {
            Gradient::Linear(Linear {
                start: self.start,
                end: self.end,
                color_stops: self.stops,
            })
        }
    }
}
