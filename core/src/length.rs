//! Describe amounts of space accurately.
use crate::Pixels;

/// The strategy used to fill space in a specific dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// Fill the minimum amount of space based on the intrinsic size of the
    /// element; normally defined by its contents.
    ///
    /// This is the default sizing strategy of most widgets.
    Fit,

    /// Fill all the remaining space.
    Fill,

    /// Fill a portion of the remaining space relative to other elements.
    ///
    /// Let's say we have two elements: one with `FillPortion(2)` and one with
    /// `FillPortion(3)`. The first will get 2 portions of the available space,
    /// while the second one would get 3.
    ///
    /// `Length::Fill` is equivalent to `Length::FillPortion(1)`.
    FillPortion(u16),

    /// Fill the least amount of space; compressing contents if possible.
    Shrink,

    /// Fill a fixed amount of space in pixels.
    Fixed(f32),

    /// Fill a certain amount of space inside the given [`Bounds`] with some [`Sizing`] strategy.
    Bounded {
        /// The [`Bounds`] of space that can be filled.
        bounds: Bounds,
        /// The [`Sizing`] strategy with which the [`Bounds`] should be filled.
        sizing: Sizing,
    },

    /// Fill the remaining space like [`Fill`](Self::Fill), but subject to a single
    /// open-ended [`Constraint`].
    Fluid(Constraint),
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The limit of a [`Fluid`](Length::Fluid) length.
pub enum Constraint {
    /// Fill available space, but never shrinks below the given amount.
    Min(f32),
    /// Fill available space, but never beyond the size the element resolves to on its own;
    /// releasing any share it doesn't use back to its siblings.
    Max,
}

impl Constraint {
    fn stack(self, other: Self) -> Self {
        match (self, other) {
            (Constraint::Min(a), Constraint::Min(b)) => Self::Min(a + b),
            (Constraint::Min(min), Constraint::Max) | (Constraint::Max, Constraint::Min(min)) => {
                Constraint::Min(min)
            }
            (Constraint::Max, Constraint::Max) => Self::Max,
        }
    }

    fn cross(self, other: Self) -> Self {
        match (self, other) {
            (Constraint::Min(a), Constraint::Min(b)) => Self::Min(a.max(b)),
            (Constraint::Min(min), Constraint::Max) | (Constraint::Max, Constraint::Min(min)) => {
                Constraint::Min(min)
            }
            (Constraint::Max, Constraint::Max) => Self::Max,
        }
    }
}

/// The space limits of a [`Bounded`](Length::Bounded) length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bounds {
    /// The length must be at least a certain amount of pixels.
    Min(f32),
    /// The length must not exceed a certain amount of pixels.
    Max(f32),
    /// The length must be inside a range of pixels.
    Both {
        /// The minimum boundary.
        min: f32,
        /// The maximum boundary.
        max: f32,
    },
}

impl Bounds {
    /// Applies a new minimum boundary to the current [`Bounds`].
    pub fn min(self, min: f32) -> Self {
        match self {
            Self::Min(_) => Self::Min(min),
            Self::Max(max) | Self::Both { max, .. } => Self::Both { min, max },
        }
    }

    /// Applies a new maximum boundary to the current [`Bounds`].
    pub fn max(self, max: f32) -> Self {
        match self {
            Self::Max(_) => Self::Max(max),
            Self::Min(min) | Self::Both { min, .. } => Self::Both { min, max },
        }
    }

    /// Returns a [`Constraint`] that represents the current [`Bounds`].
    pub fn constraint(self) -> Constraint {
        match self {
            Bounds::Min(min) | Bounds::Both { min, .. } => Constraint::Min(min),
            Bounds::Max(_) => Constraint::Max,
        }
    }
}

/// The growth strategy of a [`Bounded`](Length::Bounded) length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sizing {
    /// Equivalent to [`Length::Fit`].
    Fit,
    /// Equivalent to [`Length::FillPortion`].
    Fill(u16),
    /// Equivalent to [`Length::Shrink`].
    Shrink,
}

impl Length {
    /// Returns a new [`Bounded`](Self::Bounded) length with the given minimum bounds.
    pub fn min(self, min: impl Into<Pixels>) -> Self {
        let min = min.into().0;

        let with = match self {
            Self::Fit | Self::Fluid(_) => Sizing::Fit,
            Self::Fill => Sizing::Fill(1),
            Self::FillPortion(factor) => Sizing::Fill(factor),
            Self::Shrink => Sizing::Shrink,
            Self::Fixed(_) => return self,
            Self::Bounded {
                bounds,
                sizing: with,
            } => {
                return Self::Bounded {
                    bounds: bounds.min(min),
                    sizing: with,
                };
            }
        };

        Self::Bounded {
            bounds: Bounds::Min(min),
            sizing: with,
        }
    }

    /// Returns a new [`Bounded`](Self::Bounded) length with the given maximum bounds.
    pub fn max(self, max: impl Into<Pixels>) -> Self {
        let max = max.into().0;

        let with = match self {
            Self::Fit | Self::Fluid(_) => Sizing::Fit,
            Self::Fill => Sizing::Fill(1),
            Self::FillPortion(factor) => Sizing::Fill(factor),
            Self::Shrink => Sizing::Shrink,
            Self::Fixed(_) => return self,
            Self::Bounded {
                bounds,
                sizing: with,
            } => {
                return Self::Bounded {
                    bounds: bounds.max(max),
                    sizing: with,
                };
            }
        };

        Self::Bounded {
            bounds: Bounds::Max(max),
            sizing: with,
        }
    }

    /// Returns the _fill factor_ of the [`Length`].
    ///
    /// The _fill factor_ is a relative unit describing how much of the
    /// remaining space should be filled when compared to other elements. It
    /// is only meant to be used by layout engines.
    pub fn fill_factor(&self) -> u16 {
        match self {
            Length::Fill => 1,
            Length::FillPortion(factor)
            | Length::Bounded {
                sizing: Sizing::Fill(factor),
                ..
            } => *factor,
            Length::Fluid(_) => 1,
            Length::Shrink | Length::Fit | Length::Fixed(_) | Length::Bounded { .. } => 0,
        }
    }

    /// Returns `true` if the [`Length`] is either [`Length::Fill`] or
    /// [`Length::FillPortion`].
    pub fn is_fill(&self) -> bool {
        self.fill_factor() != 0
    }

    /// Returns `true` if the [`Length`] is [`Fit`](Self::Fit).
    pub fn is_fit(&self) -> bool {
        matches!(self, Self::Fit)
    }

    /// Stacks the constraints of the current [`Length`] with the given one, if applicable.
    ///
    /// Specifically, minimum constraints will be _added_ together and accumulated.
    ///
    /// You should use this when a container lays out multiple elements along a given axis and
    /// need to inherit their constraints in the _main_ axis.
    pub fn stack(self, other: Length) -> Self {
        self.merge_with(other, Constraint::stack)
    }

    /// Crosses the constraints of the current [`Length`] with the given one, if applicable.
    ///
    /// Specifically, minimum constraints will be compared and the _maximum_ will be returned.
    ///
    /// You should use this when a container lays out multiple elements along a given axis and
    /// need to inherit their constraints in the _cross_ axis.
    pub fn cross(self, other: Length) -> Self {
        self.merge_with(other, Constraint::cross)
    }

    fn merge_with(self, other: Self, merge: impl Fn(Constraint, Constraint) -> Constraint) -> Self {
        match (self, other) {
            // Shrink, Fixed, and Fill are unmergeable
            (Length::Shrink | Length::Fixed(_) | Length::Fill | Length::FillPortion(_), _) => self,

            // Fluid elements are merged
            (Length::Fluid(a), Length::Fluid(b)) => Length::Fluid(merge(a, b)),
            (
                Length::Fluid(a),
                Length::Bounded {
                    bounds,
                    sizing: Sizing::Fill(_),
                },
            ) => Length::Fluid(merge(a, bounds.constraint())),
            (Length::Fluid(constraint), _) => Length::Fluid(constraint),

            (
                Length::Bounded {
                    bounds,
                    sizing: Sizing::Fit,
                },
                Length::Fill
                | Length::FillPortion(_)
                | Length::Bounded {
                    sizing: Sizing::Fill(_),
                    ..
                },
            ) => Length::Bounded {
                bounds,
                sizing: Sizing::Fill(1),
            },
            (
                Length::Bounded {
                    bounds,
                    sizing: with,
                },
                _,
            ) => Length::Bounded {
                bounds,
                sizing: with,
            },

            // Fluid and bounded constraints must be propagated
            (
                _,
                Length::Bounded {
                    bounds,
                    sizing: Sizing::Fill(_),
                },
            ) => Length::Fluid(bounds.constraint()),
            (_, Length::Fluid(constraint)) => Length::Fluid(constraint),

            // Fill wins over Fit
            (_, Length::Fill | Length::FillPortion(_)) => Length::Fill,

            // Fall back to Fit
            _ => Length::Fit,
        }
    }
}

impl From<Pixels> for Length {
    fn from(amount: Pixels) -> Self {
        Self::Fixed(f32::from(amount))
    }
}

impl From<f32> for Length {
    fn from(amount: f32) -> Self {
        Self::Fixed(amount)
    }
}

impl From<u32> for Length {
    fn from(units: u32) -> Self {
        Self::Fixed(units as f32)
    }
}
