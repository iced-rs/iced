//! Image loading and caching

use std::fmt::Debug;

#[cfg(feature = "image_rs")]
pub mod raster;

#[cfg(feature = "svg")]
pub mod vector;

/// Entry in the texture store
pub trait TextureStoreEntry: Debug {
    /// Width and height of the entry
    fn size(&self) -> (u32, u32);
}

/// Stores cached image data for use in rendering
pub trait TextureStore {
    /// Entry in the texture store
    type Entry: TextureStoreEntry;
    /// State passed to upload/remove
    type State<'a>;

    /// Upload image data
    fn upload(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        state: &mut Self::State<'_>,
    ) -> Option<Self::Entry>;
    /// Remome image from store
    fn remove(&mut self, entry: &Self::Entry, state: &mut Self::State<'_>);
}
