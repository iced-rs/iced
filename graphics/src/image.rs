//! Render images.
#[cfg(feature = "image_rs")]
pub mod raster;

#[cfg(feature = "svg")]
pub mod vector;

pub mod storage;

pub use storage::Storage;
