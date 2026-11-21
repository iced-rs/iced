//! The reading/layout direction.

/// The reading/layout direction of a widget or application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    /// Left-to-right layout.
    #[default]
    LeftToRight,
    /// Right-to-left layout.
    RightToLeft,
}
