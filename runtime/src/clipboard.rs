//! Access the clipboard.
use crate::core::clipboard::{Content, Error, Kind};
use crate::futures::futures::channel::oneshot;
use crate::task::{self, Task};

use std::path::PathBuf;
use std::sync::Arc;

/// A clipboard action to be performed by some [`Task`].
///
/// [`Task`]: crate::Task
#[derive(Debug)]
pub enum Action {
    /// Read the clipboard and produce `T` with the result.
    Read {
        /// The [`Kind`] of [`Content`] to read.
        kind: Kind,
        /// The channel to send the read contents.
        channel: oneshot::Sender<Result<Content, Error>>,
    },

    /// Write the given contents to the clipboard.
    Write {
        /// The [`Content`] to be written.
        content: Content,

        /// The channel to send the write result.
        channel: oneshot::Sender<Result<(), Error>>,
    },
}

/// Read the given [`Kind`] of [`Content`] from the clipboard.
pub fn read(kind: Kind) -> Task<Result<Arc<Content>, Error>> {
    task::oneshot(|channel| crate::Action::Clipboard(Action::Read { kind, channel }))
        .map(|result| result.map(Arc::new))
}

/// Read the current text contents of the clipboard.
pub fn read_text() -> Task<Result<Arc<String>, Error>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            kind: Kind::Text,
            channel,
        })
    })
    .map(|result| {
        let Ok(Content::Text(text)) = result else {
            return Err(Error::ContentNotAvailable);
        };

        Ok(Arc::new(text))
    })
}

/// Read the current HTML contents of the clipboard.
pub fn read_html() -> Task<Result<Arc<String>, Error>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            kind: Kind::Html,
            channel,
        })
    })
    .map(|result| {
        let Ok(Content::Html(html)) = result else {
            return Err(Error::ContentNotAvailable);
        };

        Ok(Arc::new(html))
    })
}

/// Read the current file paths of the clipboard.
pub fn read_files() -> Task<Result<Arc<[PathBuf]>, Error>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            kind: Kind::Files,
            channel,
        })
    })
    .map(|result| {
        let Ok(Content::Files(files)) = result else {
            return Err(Error::ContentNotAvailable);
        };

        Ok(Arc::from(files))
    })
}

/// Read the current [`Image`](crate::core::clipboard::Image) of the clipboard.
#[cfg(feature = "image")]
pub fn read_image() -> Task<Result<crate::core::clipboard::Image, Error>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            kind: Kind::Image,
            channel,
        })
    })
    .map(|result| {
        let Ok(Content::Image(image)) = result else {
            return Err(Error::ContentNotAvailable);
        };

        Ok(image)
    })
}

/// Write the given [`Content`] to the clipboard.
pub fn write(content: impl Into<Content>) -> Task<Result<(), Error>> {
    let content = content.into();

    task::oneshot(|channel| crate::Action::Clipboard(Action::Write { content, channel }))
}
