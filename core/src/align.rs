/// Alignment on the cross axis of a container.
///
///   * On a [`Column`], it describes __horizontal__ alignment.
///   * On a [`Row`], it describes __vertical__ alignment.
///
/// [`Column`]: widget/struct.Column.html
/// [`Row`]: widget/struct.Row.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// Align at the start of the cross axis.
    Start,

    /// Align at the center of the cross axis.
    Center,

    /// Align at the end of the cross axis.
    End,

    /// Stretch over the cross axis.
    Stretch,
}
