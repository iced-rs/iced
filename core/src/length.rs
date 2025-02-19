use crate::Pixels;

/// The strategy used to fill space in a specific dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// Fill all the remaining space
    Fill,

    /// Fill a portion of the remaining space relative to other elements.
    ///
    /// Let's say we have two elements: one with `FillPortion(2)` and one with
    /// `FillPortion(3)`. The first will get 2 portions of the available space,
    /// while the second one would get 3.
    ///
    /// `Length::Fill` is equivalent to `Length::FillPortion(1)`.
    FillPortion(u16),

    /// Fill the least amount of space
    Shrink,

    /// Fill a fixed amount of space
    Fixed(f32),
}

impl Length {
    /// Returns the _fill factor_ of the [`Length`].
    ///
    /// The _fill factor_ is a relative unit describing how much of the
    /// remaining space should be filled when compared to other elements. It
    /// is only meant to be used by layout engines.
    pub fn fill_factor(&self) -> u16 {
        match self {
            Length::Fill => 1,
            Length::FillPortion(factor) => *factor,
            Length::Shrink => 0,
            Length::Fixed(_) => 0,
        }
    }

    /// Returns `true` iff the [`Length`] is either [`Length::Fill`] or
    // [`Length::FillPortion`].
    pub fn is_fill(&self) -> bool {
        self.fill_factor() != 0
    }

    /// Returns the "fluid" variant of the [`Length`].
    ///
    /// Specifically:
    /// - [`Length::Shrink`] if [`Length::Shrink`] or [`Length::Fixed`].
    /// - [`Length::Fill`] otherwise.
    pub fn fluid(&self) -> Self {
        match self {
            Length::Fill | Length::FillPortion(_) => Length::Fill,
            Length::Shrink | Length::Fixed(_) => Length::Shrink,
        }
    }

    /// Adapts the [`Length`] so it can contain the other [`Length`] and
    /// match its fluidity.
    pub fn enclose(self, other: Length) -> Self {
        match (self, other) {
            (Length::Shrink, Length::Fill | Length::FillPortion(_)) => other,
            _ => self,
        }
    }
}

impl From<Pixels> for Length {
    fn from(amount: Pixels) -> Self {
        Length::Fixed(f32::from(amount))
    }
}

impl From<f32> for Length {
    fn from(amount: f32) -> Self {
        Length::Fixed(amount)
    }
}

impl From<u32> for Length {
    fn from(units: u32) -> Self {
        Length::Fixed(units as f32)
    }
}
