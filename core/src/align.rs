/// Alignment on an axis of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// Align at the start of the axis.
    Start,

    /// Align at the center of the axis.
    Center,

    /// Align at the end of the axis.
    End,
}

/// Alignment on the cross axis of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossAlign {
    /// Align at the start of the axis.
    Start,

    /// Align at the center of the axis.
    Center,

    /// Align at the end of the axis.
    End,

    /// Fill the entire axis.
    Fill,
}

impl From<Align> for CrossAlign {
    fn from(align: Align) -> Self {
        match align {
            Align::Start => Self::Start,
            Align::Center => Self::Center,
            Align::End => Self::End,
        }
    }
}

/// The horizontal alignment of some resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    /// Align left
    Left,

    /// Horizontally centered
    Center,

    /// Align right
    Right,
}

/// The vertical alignment of some resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    /// Align top
    Top,

    /// Vertically centered
    Center,

    /// Align bottom
    Bottom,
}
