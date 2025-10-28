use crate::Instruction;
use crate::ice;

use std::io;
use std::path::PathBuf;
use std::sync::Arc;

/// A test error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// No matching widget was found for the [`Selector`](crate::Selector).
    #[error("no matching widget was found for the selector: {selector}")]
    SelectorNotFound {
        /// A description of the selector.
        selector: String,
    },
    /// A target matched, but is not visible.
    #[error("the matching target is not visible: {target:?}")]
    TargetNotVisible {
        /// The target
        target: Arc<dyn std::fmt::Debug + Send + Sync>,
    },
    /// An IO operation failed.
    #[error("an IO operation failed: {0}")]
    IOFailed(Arc<io::Error>),
    /// The decoding of some PNG image failed.
    #[error("the decoding of some PNG image failed: {0}")]
    PngDecodingFailed(Arc<png::DecodingError>),
    /// The encoding of some PNG image failed.
    #[error("the encoding of some PNG image failed: {0}")]
    PngEncodingFailed(Arc<png::EncodingError>),
    /// The parsing of an [`Ice`](crate::Ice) test failed.
    #[error("the ice test ({file}) is invalid: {error}")]
    IceParsingFailed {
        /// The path of the test.
        file: PathBuf,
        /// The parse error.
        error: ice::ParseError,
    },
    /// The execution of an [`Ice`](crate::Ice) test failed.
    #[error("the ice test ({file}) failed")]
    IceTestingFailed {
        /// The path of the test.
        file: PathBuf,
        /// The [`Instruction`] that failed.
        instruction: Instruction,
    },
    /// The [`Preset`](crate::program::Preset) of a program could not be found.
    #[error(
        "the preset \"{name}\" does not exist (available presets: {available:?})"
    )]
    PresetNotFound {
        /// The name of the [`Preset`](crate::program::Preset).
        name: String,
        /// The available set of presets.
        available: Vec<String>,
    },
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
