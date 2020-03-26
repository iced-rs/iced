/// The nesting relative to some container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Depth {
    /// Same as the container
    // TODO: Rethink
    None,
    /// Above the container
    Above,
    /// Below the container
    Below,
    /// Above everything (Global)
    Topmost,
}

impl Default for Depth {
    fn default() -> Self {
        Depth::None
    }
}
