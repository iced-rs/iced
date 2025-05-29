use crate::Selector;

use std::io;
use std::sync::Arc;

/// A test error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// No matching widget was found for the [`Selector`].
    #[error("no matching widget was found for the selector: {0:?}")]
    NotFound(Selector),
    /// An IO operation failed.
    #[error("an IO operation failed: {0}")]
    IOFailed(Arc<io::Error>),
    /// The decoding of some PNG image failed.
    #[error("the decoding of some PNG image failed: {0}")]
    PngDecodingFailed(Arc<png::DecodingError>),
    /// The encoding of some PNG image failed.
    #[error("the encoding of some PNG image failed: {0}")]
    PngEncodingFailed(Arc<png::EncodingError>),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOFailed(Arc::new(error))
    }
}

impl From<png::DecodingError> for Error {
    fn from(error: png::DecodingError) -> Self {
        Self::PngDecodingFailed(Arc::new(error))
    }
}

impl From<png::EncodingError> for Error {
    fn from(error: png::EncodingError) -> Self {
        Self::PngEncodingFailed(Arc::new(error))
    }
}
