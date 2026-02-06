//! Access the clipboard.
use crate::core::clipboard::{Content, Error, Kind};

pub use platform::*;

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use super::*;

    use std::sync::{Arc, Mutex};
    use std::thread;

    /// A buffer for short-term storage and transfer within and between
    /// applications.
    pub struct Clipboard {
        state: State,
    }

    enum State {
        Connected {
            clipboard: Arc<Mutex<arboard::Clipboard>>,
        },
        Unavailable,
    }

    impl Clipboard {
        /// Creates a new [`Clipboard`] for the given window.
        pub fn new() -> Self {
            let clipboard = arboard::Clipboard::new();

            let state = match clipboard {
                Ok(clipboard) => State::Connected {
                    clipboard: Arc::new(Mutex::new(clipboard)),
                },
                Err(_) => State::Unavailable,
            };

            Clipboard { state }
        }

        /// Reads the current content of the [`Clipboard`] as text.
        pub fn read(
            &self,
            kind: Kind,
            callback: impl FnOnce(Result<Content, Error>) + Send + 'static,
        ) {
            let State::Connected { clipboard } = &self.state else {
                callback(Err(Error::ClipboardUnavailable));
                return;
            };

            let clipboard = clipboard.clone();

            let _ = thread::spawn(move || {
                let Ok(mut clipboard) = clipboard.lock() else {
                    callback(Err(Error::ClipboardUnavailable));
                    return;
                };

                let get = clipboard.get();

                let result = match kind {
                    Kind::Text => get.text().map(Content::Text),
                    Kind::Html => get.html().map(Content::Html),
                    #[cfg(feature = "image")]
                    Kind::Image => get.image().map(|image| {
                        let rgba = crate::core::Bytes::from_owner(image.bytes);
                        let size = crate::core::Size {
                            width: image.width as u32,
                            height: image.height as u32,
                        };

                        Content::Image(crate::core::clipboard::Image { rgba, size })
                    }),
                    Kind::Files => get.file_list().map(Content::Files),
                    kind => {
                        log::warn!("unsupported clipboard kind: {kind:?}");

                        Err(arboard::Error::ContentNotAvailable)
                    }
                }
                .map_err(to_error);

                callback(result);
            });
        }

        /// Writes the given text contents to the [`Clipboard`].
        pub fn write(
            &mut self,
            content: Content,
            callback: impl FnOnce(Result<(), Error>) + Send + 'static,
        ) {
            let State::Connected { clipboard } = &self.state else {
                callback(Err(Error::ClipboardUnavailable));
                return;
            };

            let clipboard = clipboard.clone();

            let _ = thread::spawn(move || {
                let Ok(mut clipboard) = clipboard.lock() else {
                    callback(Err(Error::ClipboardUnavailable));
                    return;
                };

                let set = clipboard.set();

                let result = match content {
                    Content::Text(text) => set.text(text),
                    Content::Html(html) => set.html(html, None),
                    #[cfg(feature = "image")]
                    Content::Image(image) => set.image(arboard::ImageData {
                        bytes: image.rgba.as_ref().into(),
                        width: image.size.width as usize,
                        height: image.size.height as usize,
                    }),
                    Content::Files(files) => set.file_list(&files),
                    content => {
                        log::warn!("unsupported clipboard content: {content:?}");

                        Err(arboard::Error::ClipboardNotSupported)
                    }
                }
                .map_err(to_error);

                callback(result);
            });
        }
    }

    fn to_error(error: arboard::Error) -> Error {
        match error {
            arboard::Error::ContentNotAvailable => Error::ContentNotAvailable,
            arboard::Error::ClipboardNotSupported => Error::ClipboardUnavailable,
            arboard::Error::ClipboardOccupied => Error::ClipboardOccupied,
            arboard::Error::ConversionFailure => Error::ConversionFailure,
            arboard::Error::Unknown { description } => Error::Unknown {
                description: Arc::new(description),
            },
            error => Error::Unknown {
                description: Arc::new(error.to_string()),
            },
        }
    }
}

// TODO: Wasm support
#[cfg(target_arch = "wasm32")]
mod platform {
    use super::*;

    /// A buffer for short-term storage and transfer within and between
    /// applications.
    pub struct Clipboard;

    impl Clipboard {
        /// Creates a new [`Clipboard`] for the given window.
        pub fn new() -> Self {
            Self
        }

        /// Reads the current content of the [`Clipboard`] as text.
        pub fn read(&self, _kind: Kind, callback: impl FnOnce(Result<Content, Error>)) {
            callback(Err(Error::ClipboardUnavailable));
        }

        /// Writes the given text contents to the [`Clipboard`].
        pub fn write(&mut self, _content: Content, callback: impl FnOnce(Result<(), Error>)) {
            callback(Err(Error::ClipboardUnavailable));
        }
    }
}
