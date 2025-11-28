use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("input/output operation failed: {0}")]
    IOFailed(#[from] io::Error),
    #[error("decoding failed: {0}")]
    DecodingFailed(#[from] Box<bincode::ErrorKind>),
}
