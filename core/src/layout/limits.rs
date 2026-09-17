#![allow(clippy::manual_clamp)]
use crate::length;
use crate::{Length, Size};

/// A set of size constraints for layouting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// The minimum bounds.
    pub min: Size,
    /// The maximum bounds.
    pub max: Size,
    /// Whether fluid lengths should be compressed to intrinsic dimensions.
    pub compression: Size<bool>,
    /// Whether intrinsic dimensions can exceed maximum bounds. The maximum
    /// boundary becomes a hint in this case.
    pub infinite: Size<bool>,
}

impl Limits {
    /// Creates new [`Limits`] with the given minimum and maximum [`Size`].
    pub const fn new(min: Size, max: Size) -> Limits {
        Limits::with_flags(min, max, Size::new(false, false), Size::new(false, false))
    }

    /// Creates new [`Limits`] with the given minimum and maximum [`Size`],
    /// whether fluid lengths should be compressed to intrinsic dimensions,
    /// and whether the upper bounds may grow indefinitely.
    pub const fn with_flags(
        min: Size,
        max: Size,
        compression: Size<bool>,
        infinite: Size<bool>,
    ) -> Self {
        Limits {
            min,
            max,
            compression,
            infinite,
        }
    }

    /// Returns the maximum [`Size`] of the [`Limits`].
    ///
    /// On axes with [`infinite`](Self::infinite) bounds, the value is
    /// [`f32::INFINITY`], as the maximum boundary becomes a hint in this
    /// case.
    pub fn bounds(&self) -> Size {
        Size::new(
            if self.infinite.width {
                f32::INFINITY
            } else {
                self.max.width
            },
            if self.infinite.height {
                f32::INFINITY
            } else {
                self.max.height
            },
        )
    }

    /// Applies a width constraint to the current [`Limits`].
    pub fn width(mut self, width: impl Into<Length>) -> Limits {
        match width.into() {
            Length::Shrink => {
                self.compression.width = true;
            }
            Length::Fit | Length::Fluid(_) => {
                self.compression.width = false;
            }
            Length::Fixed(amount) => {
                let new_width = amount.min(self.max.width).max(self.min.width);

                self.min.width = new_width;
                self.max.width = new_width;
                self.compression.width = false;
            }
            Length::Bounded { bounds, sizing } => {
                match bounds {
                    length::Bounds::Min(min) => {
                        self.min.width = min.min(self.max.width).max(self.min.width);
                    }
                    length::Bounds::Max(max) => {
                        self.max.width = max.min(self.max.width).max(self.min.width);
                    }
                    length::Bounds::Both { min, max } => {
                        self.min.width = min.min(self.max.width).max(self.min.width);
                        self.max.width = max.min(self.max.width).max(self.min.width);
                    }
                }

                match sizing {
                    length::Sizing::Shrink => {
                        self.compression.width = true;
                    }
                    length::Sizing::Fit => {
                        self.compression.width = false;
                    }
                    length::Sizing::Fill(_) => {}
                }
            }
            Length::Fill | Length::FillPortion(_) => {}
        }

        self
    }

    /// Applies a height constraint to the current [`Limits`].
    pub fn height(mut self, height: impl Into<Length>) -> Limits {
        match height.into() {
            Length::Shrink => {
                self.compression.height = true;
            }
            Length::Fit | Length::Fluid(_) => {
                self.compression.height = false;
            }
            Length::Fixed(amount) => {
                let new_height = amount.min(self.max.height).max(self.min.height);

                self.min.height = new_height;
                self.max.height = new_height;
                self.compression.height = false;
            }
            Length::Bounded { bounds, sizing } => {
                match bounds {
                    length::Bounds::Min(min) => {
                        self.min.height = min.min(self.max.height).max(self.min.height);
                    }
                    length::Bounds::Max(max) => {
                        self.max.height = max.min(self.max.height).max(self.min.height);
                    }
                    length::Bounds::Both { min, max } => {
                        self.min.height = min.min(self.max.height).max(self.min.height);
                        self.max.height = max.min(self.max.height).max(self.min.height);
                    }
                }

                match sizing {
                    length::Sizing::Shrink => {
                        self.compression.height = true;
                    }
                    length::Sizing::Fit => {
                        self.compression.height = false;
                    }
                    length::Sizing::Fill(_) => {}
                }
            }
            Length::Fill | Length::FillPortion(_) => {}
        }

        self
    }

    /// Shrinks the current [`Limits`] by the given [`Size`].
    pub fn shrink(&self, size: impl Into<Size>) -> Limits {
        let size = size.into();

        let min = Size::new(
            (self.min.width - size.width).max(0.0),
            (self.min.height - size.height).max(0.0),
        );

        let max = Size::new(
            (self.max.width - size.width).max(0.0),
            (self.max.height - size.height).max(0.0),
        );

        Limits {
            min,
            max,
            compression: self.compression,
            infinite: self.infinite,
        }
    }

    /// Removes the minimum [`Size`] constraint for the current [`Limits`].
    pub fn loose(&self) -> Limits {
        Limits {
            min: Size::ZERO,
            max: self.max,
            compression: self.compression,
            infinite: self.infinite,
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
        Size::new(
            self.resolve_width(width, intrinsic_size.width),
            self.resolve_height(height, intrinsic_size.height),
        )
    }

    /// [Resolves](Self::resolve) only the width of the [`Limits`].
    pub fn resolve_width(&self, width: impl Into<Length>, intrinsic_width: f32) -> f32 {
        resolve(
            self.min.width,
            self.max.width,
            self.compression.width,
            self.infinite.width,
            width.into(),
            intrinsic_width,
        )
    }

    /// [Resolves](Self::resolve) only the height of the [`Limits`].
    pub fn resolve_height(&self, height: impl Into<Length>, intrinsic_height: f32) -> f32 {
        resolve(
            self.min.height,
            self.max.height,
            self.compression.height,
            self.infinite.height,
            height.into(),
            intrinsic_height,
        )
    }
}

fn resolve(
    min: f32,
    max: f32,
    compression: bool,
    infinite: bool,
    length: Length,
    intrinsic: f32,
) -> f32 {
    match length {
        Length::Fill
        | Length::FillPortion(_)
        | Length::Bounded {
            sizing: length::Sizing::Fill(_),
            ..
        } if !compression => if infinite { max.max(intrinsic) } else { max }.max(min),
        Length::Fixed(amount) => if infinite { amount } else { amount.min(max) }.max(min),
        _ => if infinite {
            intrinsic
        } else {
            intrinsic.min(max)
        }
        .max(min),
    }
}
