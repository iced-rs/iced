#![allow(clippy::manual_clamp)]
use crate::{Length, Size};

/// A set of size constraints for layouting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    min: Size,
    max: Size,
}

impl Limits {
    /// No limits
    pub const NONE: Limits = Limits {
        min: Size::ZERO,
        max: Size::INFINITY,
    };

    /// Creates new [`Limits`] with the given minimum and maximum [`Size`].
    pub const fn new(min: Size, max: Size) -> Limits {
        Limits { min, max }
    }

    /// Returns the minimum [`Size`] of the [`Limits`].
    pub fn min(&self) -> Size {
        self.min
    }

    /// Returns the maximum [`Size`] of the [`Limits`].
    pub fn max(&self) -> Size {
        self.max
    }

    /// Applies a width constraint to the current [`Limits`].
    pub fn width(mut self, width: impl Into<Length>) -> Limits {
        match width.into() {
            Length::Shrink | Length::Fill | Length::FillPortion(_) => {}
            Length::Fixed(amount) => {
                let new_width = amount.min(self.max.width).max(self.min.width);

                self.min.width = new_width;
                self.max.width = new_width;
            }
        }

        self
    }

    /// Applies a height constraint to the current [`Limits`].
    pub fn height(mut self, height: impl Into<Length>) -> Limits {
        match height.into() {
            Length::Shrink | Length::Fill | Length::FillPortion(_) => {}
            Length::Fixed(amount) => {
                let new_height =
                    amount.min(self.max.height).max(self.min.height);

                self.min.height = new_height;
                self.max.height = new_height;
            }
        }

        self
    }

    /// Applies a minimum width constraint to the current [`Limits`].
    pub fn min_width(mut self, min_width: f32) -> Limits {
        self.min.width = self.min.width.max(min_width).min(self.max.width);

        self
    }

    /// Applies a maximum width constraint to the current [`Limits`].
    pub fn max_width(mut self, max_width: f32) -> Limits {
        self.max.width = self.max.width.min(max_width).max(self.min.width);

        self
    }

    /// Applies a minimum height constraint to the current [`Limits`].
    pub fn min_height(mut self, min_height: f32) -> Limits {
        self.min.height = self.min.height.max(min_height).min(self.max.height);

        self
    }

    /// Applies a maximum height constraint to the current [`Limits`].
    pub fn max_height(mut self, max_height: f32) -> Limits {
        self.max.height = self.max.height.min(max_height).max(self.min.height);

        self
    }

    /// Shrinks the current [`Limits`] by the given [`Size`].
    pub fn shrink(&self, size: impl Into<Size>) -> Limits {
        let size = size.into();

        let min = Size::new(
            (self.min().width - size.width).max(0.0),
            (self.min().height - size.height).max(0.0),
        );

        let max = Size::new(
            (self.max().width - size.width).max(0.0),
            (self.max().height - size.height).max(0.0),
        );

        Limits { min, max }
    }

    /// Removes the minimum width constraint for the current [`Limits`].
    pub fn loose(&self) -> Limits {
        Limits {
            min: Size::ZERO,
            max: self.max,
        }
    }

    /// Computes the resulting [`Size`] that fits the [`Limits`] given
    /// some width and height requirements and the intrinsic size of
    /// some content.
    pub fn resolve(
        &self,
        width: impl Into<Length>,
        height: impl Into<Length>,
        intrinsic_size: Size,
    ) -> Size {
        let width = match width.into() {
            Length::Fill | Length::FillPortion(_) => self.max.width,
            Length::Fixed(amount) => {
                amount.min(self.max.width).max(self.min.width)
            }
            Length::Shrink => {
                intrinsic_size.width.min(self.max.width).max(self.min.width)
            }
        };

        let height = match height.into() {
            Length::Fill | Length::FillPortion(_) => self.max.height,
            Length::Fixed(amount) => {
                amount.min(self.max.height).max(self.min.height)
            }
            Length::Shrink => intrinsic_size
                .height
                .min(self.max.height)
                .max(self.min.height),
        };

        Size::new(width, height)
    }
}
