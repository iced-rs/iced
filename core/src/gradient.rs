//! For creating a Gradient.
pub use linear::Linear;

use crate::{Color, Radians};

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
    /// Creates a new linear [`linear::Builder`].
    ///
    /// This must be defined by an angle (in [`Degrees`] or [`Radians`])
    /// which will use the bounds of the widget as a guide.
    pub fn linear(angle: impl Into<Radians>) -> linear::Builder {
        linear::Builder::new(angle.into())
    }

    /// Adjust the opacity of the gradient by a multiplier applied to each color stop.
    pub fn mul_alpha(mut self, alpha_multiplier: f32) -> Self {
        match &mut self {
            Gradient::Linear(linear) => {
                for stop in linear.color_stops.iter_mut().flatten() {
                    stop.color.a *= alpha_multiplier;
                }
            }
        }

        self
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

pub mod linear {
    //! Linear gradient builder & definition.
    use crate::gradient::{ColorStop, Gradient};
    use crate::{Color, Radians};
    use std::cmp::Ordering;

    /// A linear gradient that can be used as a [`Background`].
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Linear {
        /// How the [`Gradient`] is angled within its bounds.
        pub angle: Radians,
        /// [`ColorStop`]s along the linear gradient path.
        pub color_stops: [Option<ColorStop>; 8],
    }

    /// A [`Linear`] builder.
    #[derive(Debug)]
    pub struct Builder {
        angle: Radians,
        stops: [Option<ColorStop>; 8],
    }

    impl Builder {
        /// Creates a new [`Builder`].
        pub fn new(angle: Radians) -> Self {
            Self {
                angle,
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
                angle: self.angle,
                color_stops: self.stops,
            })
        }
    }
}
