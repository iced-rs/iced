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

#[cfg(feature = "stretch")]
#[doc(hidden)]
impl From<Align> for stretch::style::AlignItems {
    fn from(align: Align) -> Self {
        match align {
            Align::Start => stretch::style::AlignItems::FlexStart,
            Align::Center => stretch::style::AlignItems::Center,
            Align::End => stretch::style::AlignItems::FlexEnd,
            Align::Stretch => stretch::style::AlignItems::Stretch,
        }
    }
}

#[cfg(feature = "stretch")]
#[doc(hidden)]
impl From<Align> for stretch::style::AlignSelf {
    fn from(align: Align) -> Self {
        match align {
            Align::Start => stretch::style::AlignSelf::FlexStart,
            Align::Center => stretch::style::AlignSelf::Center,
            Align::End => stretch::style::AlignSelf::FlexEnd,
            Align::Stretch => stretch::style::AlignSelf::Stretch,
        }
    }
}
