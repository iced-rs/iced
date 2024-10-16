/// Defines the orientation that a window resize will be performed.
#[derive(Debug, Clone, Copy)]
pub enum ResizeDirection {
    /// The window will be resized along the eastern ( right ) edge.
    East,

    /// The window will be resized along the eastern ( top ) edge.
    North,

    /// The window will be resized at the northeastern ( top-right ) corner.
    NorthEast,

    /// The window will be resized at the northwestern ( top-left ) corner.
    NorthWest,

    /// The window will be resized along the southern ( bottom ) edge.
    South,

    /// The window will be resized at the southeastern ( bottom-right ) corner.
    SouthEast,

    /// The window will be resized at the southwestern ( bottom-left ) corner.
    SouthWest,

    /// The window will be resized along the western ( left ) edge.
    West,
}
