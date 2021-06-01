use crate::{Length, Padding, Size};

/// A set of size constraints for layouting.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    min: Size,
    max: Size,
    fill: Size,
}

impl Limits {
    /// No limits
    pub const NONE: Limits = Limits {
        min: Size::ZERO,
        max: Size::INFINITY,
        fill: Size::INFINITY,
    };

    /// Creates new [`Limits`] with the given minimum and maximum [`Size`].
    pub const fn new(min: Size, max: Size) -> Limits {
        Limits {
            min,
            max,
            fill: Size::INFINITY,
        }
    }

    /// Returns the minimum [`Size`] of the [`Limits`].
    pub fn min(&self) -> Size {
        self.min
    }

    /// Returns the maximum [`Size`] of the [`Limits`].
    pub fn max(&self) -> Size {
        self.max
    }

    /// Returns the fill [`Size`] of the [`Limits`].
    pub fn fill(&self) -> Size {
        self.fill
    }

    /// Applies a width constraint to the current [`Limits`].
    pub fn width(mut self, width: Length) -> Limits {
        match width {
            Length::Shrink => {
                self.fill.width = self.min.width;
            }
            Length::Fill | Length::FillPortion(_) => {
                self.fill.width = self.fill.width.min(self.max.width);
            }
            Length::Units(units) => {
                let new_width =
                    (units as f32).min(self.max.width).max(self.min.width);

                self.min.width = new_width;
                self.max.width = new_width;
                self.fill.width = new_width;
            }
        }

        self
    }

    /// Applies a height constraint to the current [`Limits`].
    pub fn height(mut self, height: Length) -> Limits {
        match height {
            Length::Shrink => {
                self.fill.height = self.min.height;
            }
            Length::Fill | Length::FillPortion(_) => {
                self.fill.height = self.fill.height.min(self.max.height);
            }
            Length::Units(units) => {
                let new_height =
                    (units as f32).min(self.max.height).max(self.min.height);

                self.min.height = new_height;
                self.max.height = new_height;
                self.fill.height = new_height;
            }
        }

        self
    }

    /// Applies a minimum width constraint to the current [`Limits`].
    pub fn min_width(mut self, min_width: u32) -> Limits {
        self.min.width =
            self.min.width.max(min_width as f32).min(self.max.width);

        self
    }

    /// Applies a maximum width constraint to the current [`Limits`].
    pub fn max_width(mut self, max_width: u32) -> Limits {
        self.max.width =
            self.max.width.min(max_width as f32).max(self.min.width);

        self
    }

    /// Applies a minimum height constraint to the current [`Limits`].
    pub fn min_height(mut self, min_height: u32) -> Limits {
        self.min.height =
            self.min.height.max(min_height as f32).min(self.max.height);

        self
    }

    /// Applies a maximum height constraint to the current [`Limits`].
    pub fn max_height(mut self, max_height: u32) -> Limits {
        self.max.height =
            self.max.height.min(max_height as f32).max(self.min.height);

        self
    }

    /// Shrinks the current [`Limits`] to account for the given padding.
    pub fn pad(&self, padding: Padding) -> Limits {
        self.shrink(Size::new(
            padding.horizontal() as f32,
            padding.vertical() as f32,
        ))
    }

    /// Shrinks the current [`Limits`] by the given [`Size`].
    pub fn shrink(&self, size: Size) -> Limits {
        let min = Size::new(
            (self.min().width - size.width).max(0.0),
            (self.min().height - size.height).max(0.0),
        );

        let max = Size::new(
            (self.max().width - size.width).max(0.0),
            (self.max().height - size.height).max(0.0),
        );

        let fill = Size::new(
            (self.fill.width - size.width).max(0.0),
            (self.fill.height - size.height).max(0.0),
        );

        Limits { min, max, fill }
    }

    /// Removes the minimum width constraint for the current [`Limits`].
    pub fn loose(&self) -> Limits {
        Limits {
            min: Size::ZERO,
            max: self.max,
            fill: self.fill,
        }
    }

    /// Computes the resulting [`Size`] that fits the [`Limits`] given the
    /// intrinsic size of some content.
    pub fn resolve(&self, intrinsic_size: Size) -> Size {
        Size::new(
            intrinsic_size
                .width
                .min(self.max.width)
                .max(self.fill.width),
            intrinsic_size
                .height
                .min(self.max.height)
                .max(self.fill.height),
        )
    }
}
