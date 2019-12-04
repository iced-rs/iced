/// The strategy used to fill space in a specific dimension.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Length {
    /// Fill all the remaining space
    Fill,

    /// Fill the least amount of space
    Shrink,

    /// Fill a fixed amount of space
    Units(u16),
}

impl Length {
    /// Returns the _fill factor_ of the [`Length`].
    ///
    /// The _fill factor_ is a relative unit describing how much of the
    /// remaining space should be filled when compared to other elements. It
    /// is only meant to be used by layout engines.
    ///
    /// [`Length`]: enum.Length.html
    pub fn fill_factor(&self) -> u16 {
        match self {
            Length::Fill => 1,
            Length::Shrink => 0,
            Length::Units(_) => 0,
        }
    }
}

impl From<u16> for Length {
    fn from(units: u16) -> Self {
        Length::Units(units)
    }
}
