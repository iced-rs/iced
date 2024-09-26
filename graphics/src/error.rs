//! See what can go wrong when creating graphical backends.
use crate::Backend;

use std::fmt;

/// An error that occurred while creating an application's graphical context.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The requested backend version is not supported.
    #[error("the requested backend version is not supported")]
    VersionNotSupported,

    /// Failed to find any pixel format that matches the criteria.
    #[error("failed to find any pixel format that matches the criteria")]
    NoAvailablePixelFormat,

    /// A suitable graphics adapter or device could not be found.
    #[error("a suitable graphics adapter could not be found: {reason}")]
    GraphicsAdapterNotFound {
        /// The name of the backend where the error happened
        backend: &'static str,
        /// The reason why this backend could not be used
        reason: Reason,
    },

    /// An error occurred in the context's internal backend
    #[error("an error occurred in the context's internal backend")]
    BackendError(String),

    /// Multiple errors occurred
    #[error("multiple errors occurred:\n{}", error_list(.0))]
    List(Vec<Self>),
}

/// The reason why a graphics adapter could not be found
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// The backend did not match the preference
    DidNotMatch {
        /// The preferred backend
        preferred_backend: Backend,
    },
    /// The request to create the backend failed
    RequestFailed(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reason::DidNotMatch { preferred_backend } => {
                write!(
                    f,
                    "the backend did not match \
                    the preference: {preferred_backend}"
                )
            }
            Reason::RequestFailed(error) => f.write_str(error),
        }
    }
}

fn error_list(errors: &Vec<Error>) -> String {
    let mut list = String::new();

    for error in errors {
        list.push_str("- ");
        list.push_str(&error.to_string());
        list.push('\n');
    }

    list
}
