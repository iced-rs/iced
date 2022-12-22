//! For creating a Gradient.
pub use linear::Linear;

use crate::{Color, Radians, Rectangle, Vector};

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
    pub fn transparentize(mut self, alpha_multiplier: f32) -> Self {
        match &mut self {
            Gradient::Linear(linear) => {
                for stop in &mut linear.color_stops {
                    stop.color.a *= alpha_multiplier;
                }
            }
        }

        self
    }

    /// Packs the [`Gradient`] into a buffer for use in shader code.
    pub fn pack(&self, bounds: Rectangle) -> [f32; 44] {
        match self {
            Gradient::Linear(linear) => {
                let mut pack: [f32; 44] = [0.0; 44];
                let mut offsets: [f32; 8] = [2.0; 8];

                for (index, stop) in
                    linear.color_stops.iter().enumerate().take(8)
                {
                    let [r, g, b, a] = stop.color.into_linear();

                    pack[(index * 4)] = r;
                    pack[(index * 4) + 1] = g;
                    pack[(index * 4) + 2] = b;
                    pack[(index * 4) + 3] = a;

                    offsets[index] = stop.offset;
                }

                pack[32] = offsets[0];
                pack[33] = offsets[1];
                pack[34] = offsets[2];
                pack[35] = offsets[3];
                pack[36] = offsets[4];
                pack[37] = offsets[5];
                pack[38] = offsets[6];
                pack[39] = offsets[7];

                let v1 = Vector::new(
                    f32::cos(linear.angle.0),
                    f32::sin(linear.angle.0),
                );

                let distance_to_rect = f32::min(
                    f32::abs((bounds.y - bounds.center().y) / v1.y),
                    f32::abs(
                        ((bounds.x + bounds.width) - bounds.center().x) / v1.x,
                    ),
                );

                let start = bounds.center() + v1 * distance_to_rect;
                let end = bounds.center() - v1 * distance_to_rect;

                pack[40] = start.x;
                pack[41] = start.y;
                pack[42] = end.x;
                pack[43] = end.y;

                pack
            }
        }
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

    /// A linear gradient that can be used as a [`Background`].
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Linear {
        /// How the [`Gradient`] is angled within its bounds.
        pub angle: Radians,
        /// [`ColorStop`]s along the linear gradient path.
        pub color_stops: [ColorStop; 8],
    }

    /// A [`Linear`] builder.
    #[derive(Debug)]
    pub struct Builder {
        angle: Radians,
        stops: [ColorStop; 8],
        error: Option<BuilderError>,
    }

    impl Builder {
        /// Creates a new [`Builder`].
        pub fn new(angle: Radians) -> Self {
            Self {
                angle,
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
                    angle: self.angle,
                    color_stops: self.stops,
                }))
            }
        }
    }

    /// An error that happened when building a [`Linear`] gradient.
    #[derive(Debug, thiserror::Error)]
    pub enum BuilderError {
        #[error("Gradients must contain at least one color stop.")]
        /// Gradients must contain at least one color stop.
        MissingColorStop,
        #[error("Offset {0} must be a unique, finite number.")]
        /// Offsets in a gradient must all be unique & finite.
        DuplicateOffset(f32),
        #[error("Offset {0} must be between 0.0..=1.0.")]
        /// Offsets in a gradient must be between 0.0..=1.0.
        InvalidOffset(f32),
    }
}
