//! Store images.
use iced_core::Size;

use std::fmt::Debug;

/// Stores cached image data for use in rendering
pub trait Storage {
    /// The type of an [`Entry`] in the [`Storage`].
    type Entry: Entry;

    /// State provided to upload or remove a [`Self::Entry`].
    type State<'a>;

    /// Upload the image data of a [`Self::Entry`].
    fn upload(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        state: &mut Self::State<'_>,
    ) -> Option<Self::Entry>;

    /// Remove a [`Self::Entry`] from the [`Storage`].
    fn remove(&mut self, entry: &Self::Entry, state: &mut Self::State<'_>);
}

/// An entry in some [`Storage`],
pub trait Entry: Debug {
    /// The [`Size`] of the [`Entry`].
    fn size(&self) -> Size<u32>;
}
