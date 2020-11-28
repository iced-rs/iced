/// The state of a touch interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// A touch interaction was started.
    Started,

    /// An on-going touch interaction was moved.
    Moved,

    /// A touch interaction was ended.
    Ended,

    /// A touch interaction was canceled.
    Canceled,
}