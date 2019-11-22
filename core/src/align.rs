/// Alignment on an unspecified axis of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// Align at the start of the cross axis.
    Start,

    /// Align at the center of the cross axis.
    Center,

    /// Align at the end of the cross axis.
    End,
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
