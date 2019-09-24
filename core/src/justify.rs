/// Distribution on the main axis of a container.
///
///   * On a [`Column`], it describes __vertical__ distribution.
///   * On a [`Row`], it describes __horizontal__ distribution.
///
/// [`Column`]: widget/struct.Column.html
/// [`Row`]: widget/struct.Row.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Justify {
    /// Place items at the start of the main axis.
    Start,

    /// Place items at the center of the main axis.
    Center,

    /// Place items at the end of the main axis.
    End,

    /// Place items with space between.
    SpaceBetween,

    /// Place items with space around.
    SpaceAround,

    /// Place items with evenly distributed space.
    SpaceEvenly,
}
