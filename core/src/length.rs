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

    /// Fill a certain amount of space inside the given [`Bounds`] with some [`Fluidity`].
    Bounded {
        /// The amount of space that can be filled.
        bounds: Bounds,
        /// The strategy in which the space should be filled.
        with: Fluidity,
    },

    /// TODO
    Fluid(Constraint),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constraint {
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bounds {
    Min(f32),
    Max(f32),
    Both { min: f32, max: f32 },
}

impl Bounds {
    pub fn min(self, min: f32) -> Self {
        match self {
            Self::Min(_) => Self::Min(min),
            Self::Max(max) => Self::Both { min, max },
            Self::Both { max, .. } => Self::Both { min, max },
        }
    }

    pub fn max(self, max: f32) -> Self {
        match self {
            Self::Min(min) => Self::Both { min, max },
            Self::Max(_) => Self::Max(max),
            Self::Both { min, .. } => Self::Both { min, max },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fluidity {
    Fit,
    Fill(u16),
    Shrink,
}

impl Length {
    /// Returns a new [`Bounded`](Self::Bounded) length with the given minimum bounds.
    pub fn min(self, min: impl Into<Pixels>) -> Self {
        let min = min.into().0;

        let with = match self {
            Self::Fit | Self::Fluid(_) => Fluidity::Fit,
            Self::Fill => Fluidity::Fill(1),
            Self::FillPortion(factor) => Fluidity::Fill(factor),
            Self::Shrink => Fluidity::Shrink,
            Self::Fixed(_) => return self,
            Self::Bounded { bounds, with } => {
                return Self::Bounded {
                    bounds: bounds.min(min),
                    with,
                };
            }
        };

        Self::Bounded {
            bounds: Bounds::Min(min),
            with,
        }
    }

    /// Returns a new [`Bounded`](Self::Bounded) length with the given maximum bounds.
    pub fn max(self, max: impl Into<Pixels>) -> Self {
        let max = max.into().0;

        let with = match self {
            Self::Fit | Self::Fluid(_) => Fluidity::Fit,
            Self::Fill => Fluidity::Fill(1),
            Self::FillPortion(factor) => Fluidity::Fill(factor),
            Self::Shrink => Fluidity::Shrink,
            Self::Fixed(_) => return self,
            Self::Bounded { bounds, with } => {
                return Self::Bounded {
                    bounds: bounds.max(max),
                    with,
                };
            }
        };

        Self::Bounded {
            bounds: Bounds::Max(max),
            with,
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
                with: Fluidity::Fill(factor),
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

    /// Returns the "fluid" variant of the [`Length`].
    ///
    /// Specifically:
    /// - [`Length::Shrink`] if [`Length::Shrink`] or [`Length::Fixed`].
    /// - [`Length::Fill`] otherwise.
    pub fn fluid(&self) -> Self {
        if self.fill_factor() == 0 {
            Self::Shrink
        } else {
            Self::Fill
        }
    }

    /// Adapts the [`Length`] so it can contain the other [`Length`] and
    /// match its fluidity.
    #[inline]
    pub fn enclose(self, other: Length) -> Self {
        match (self, other) {
            (
                Length::Fit | Length::Bounded { .. },
                Length::Fill
                | Length::FillPortion(_)
                | Length::Bounded {
                    bounds: Bounds::Min(_),
                    with: Fluidity::Fill(_),
                },
            ) => Length::Fill,
            (
                Length::Fit,
                Length::Bounded {
                    with: Fluidity::Fill(_),
                    bounds,
                },
            ) => Self::Fluid(match bounds {
                Bounds::Min(_) => Constraint::Min,
                Bounds::Max(_) | Bounds::Both { .. } => Constraint::Max,
            }),
            _ => self,
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
