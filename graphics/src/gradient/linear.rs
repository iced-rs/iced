//! Linear gradient builder & definition.
use crate::gradient::{ColorStop, Gradient, Position};
use crate::{Color, Point};

/// A linear gradient that can be used in the style of [`Fill`] or [`Stroke`].
///
/// [`Fill`]: crate::widget::canvas::Fill
/// [`Stroke`]: crate::widget::canvas::Stroke
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
    stops: Vec<ColorStop>,
    error: Option<BuilderError>,
}

impl Builder {
    /// Creates a new [`Builder`].
    pub fn new(position: Position) -> Self {
        let (start, end) = match position {
            Position::Absolute { start, end } => (start, end),
            Position::Relative {
                top_left,
                size,
                start,
                end,
            } => (
                start.to_absolute(top_left, size),
                end.to_absolute(top_left, size),
            ),
        };

        Self {
            start,
            end,
            stops: vec![],
            error: None,
        }
    }

    /// Adds a new stop, defined by an offset and a color, to the gradient.
    ///
    /// `offset` must be between `0.0` and `1.0` or the gradient cannot be built.
    ///
    /// Note: when using the [`glow`] backend, any color stop added after the 16th
    /// will not be displayed.
    ///
    /// On the [`wgpu`] backend this limitation does not exist (technical limit is 524,288 stops).
    ///
    /// [`glow`]: https://docs.rs/iced_glow
    /// [`wgpu`]: https://docs.rs/iced_wgpu
    pub fn add_stop(mut self, offset: f32, color: Color) -> Self {
        if offset.is_finite() && (0.0..=1.0).contains(&offset) {
            match self.stops.binary_search_by(|stop| {
                stop.offset.partial_cmp(&offset).unwrap()
            }) {
                Ok(_) => {
                    self.error = Some(BuilderError::DuplicateOffset(offset))
                }
                Err(index) => {
                    self.stops.insert(index, ColorStop { offset, color });
                }
            }
        } else {
            self.error = Some(BuilderError::InvalidOffset(offset))
        };

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
