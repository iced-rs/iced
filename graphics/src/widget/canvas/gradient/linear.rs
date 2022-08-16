//! A linear color gradient.
use iced_native::{Color, Point};

use crate::triangle::shader;
use crate::triangle::shader::gradient::ColorStop;

use super::Gradient;

/// A linear gradient that can be used in the style of [`super::Fill`] or [`super::Stroke`].
#[derive(Debug, Clone)]
pub struct Linear {
    start: Point,
    end: Point,
    color_stops: Vec<ColorStop>,
}

impl Linear {
    pub(super) fn gradient(&self) -> shader::Gradient {
        shader::Gradient::Linear {
            start: self.start,
            end: self.end,
            stops: self.color_stops.clone(),
        }
    }
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

        Some(Gradient::Linear(Linear {
            start: self.start,
            end: self.end,
            color_stops: self.stops.into_iter().map(ColorStop::from).collect(),
        }))
    }
}
