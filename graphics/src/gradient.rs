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
    use iced_core::gradient::linear::BuilderError;
    use iced_core::gradient::ColorStop;
    use iced_core::{Color, Point};

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
        pub color_stops: [ColorStop; 8],
    }

    /// A [`Linear`] builder.
    #[derive(Debug)]
    pub struct Builder {
        start: Point,
        end: Point,
        stops: [ColorStop; 8],
        error: Option<BuilderError>,
    }

    impl Builder {
        /// Creates a new [`Builder`].
        pub fn new(start: Point, end: Point) -> Self {
            Self {
                start,
                end,
                stops: std::array::from_fn(|_| ColorStop {
                    offset: 2.0, //default offset = invalid
                    color: Default::default(),
                }),
                error: None,
            }
        }

        /// Adds a new [`ColorStop`], defined by an offset and a color, to the gradient.
        ///
        /// `offset` must be between `0.0` and `1.0` or the gradient cannot be built.
        ///
        /// Any stop added after the 8th will be silently ignored.
        pub fn add_stop(mut self, offset: f32, color: Color) -> Self {
            if offset.is_finite() && (0.0..=1.0).contains(&offset) {
                match self.stops.binary_search_by(|stop| {
                    stop.offset.partial_cmp(&offset).unwrap()
                }) {
                    Ok(_) => {
                        self.error = Some(BuilderError::DuplicateOffset(offset))
                    }
                    Err(index) => {
                        if index < 8 {
                            self.stops[index] = ColorStop { offset, color };
                        }
                    }
                }
            } else {
                self.error = Some(BuilderError::InvalidOffset(offset))
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
        pub fn build(self) -> Result<Gradient, BuilderError> {
            if self.stops.is_empty() {
                Err(BuilderError::MissingColorStop)
            } else if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(Gradient::Linear(Linear {
                    start: self.start,
                    end: self.end,
                    color_stops: self.stops,
                }))
            }
        }
    }
}
