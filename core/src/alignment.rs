//! Align and position widgets.

/// Alignment on the axis of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
    /// Align at the start of the axis.
    Start,

    /// Align at the center of the axis.
    Center,

    /// Align at the end of the axis.
    End,

    /// Fill the entire axis.
    Fill,
}

impl From<Horizontal> for Alignment {
    fn from(horizontal: Horizontal) -> Self {
        match horizontal {
            Horizontal::Left => Self::Start,
            Horizontal::Center => Self::Center,
            Horizontal::Right => Self::End,
        }
    }
}

impl From<Vertical> for Alignment {
    fn from(vertical: Vertical) -> Self {
        match vertical {
            Vertical::Top => Self::Start,
            Vertical::Center => Self::Center,
            Vertical::Bottom => Self::End,
        }
    }
}

/// The horizontal [`Alignment`] of some resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Horizontal {
    /// Align left
    Left,

    /// Horizontally centered
    Center,

    /// Align right
    Right,
}

/// The vertical [`Alignment`] of some resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vertical {
    /// Align top
    Top,

    /// Vertically centered
    Center,

    /// Align bottom
    Bottom,
}
