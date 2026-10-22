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

#[cfg(target_arch = "wasm32")]
mod platform {
    use super::*;

    use std::sync::Arc;

    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::js_sys::{Array, Object, Reflect, Uint8Array};
    use web_sys::wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Blob, BlobPropertyBag, ClipboardItem};

    /// A buffer for short-term storage and transfer within and between
    /// applications.
    pub struct Clipboard {
        clipboard: Option<web_sys::Clipboard>,
    }

    impl Clipboard {
        /// Creates a new [`Clipboard`] for the given window.
        pub fn new() -> Self {
            let clipboard = web_sys::window()
                .map(|window| window.navigator().clipboard());

            Self { clipboard }
        }

        /// Reads the current content of the [`Clipboard`].
        pub fn read(
            &self,
            kind: Kind,
            callback: impl FnOnce(Result<Content, Error>) + 'static,
        ) {
            let Some(clipboard) = self.clipboard.clone() else {
                callback(Err(Error::ClipboardUnavailable));
                return;
            };

            spawn_local(async move {
                let result = match kind {
                    Kind::Text => {
                        read_text(&clipboard).await.map(Content::Text)
                    }
                    Kind::Html => {
                        read_html(&clipboard).await.map(Content::Html)
                    }
                    #[cfg(feature = "image")]
                    Kind::Image => {
                        read_image(&clipboard).await.map(Content::Image)
                    }
                    Kind::Files => Err(Error::ContentNotAvailable),
                    kind => {
                        log::warn!("unsupported clipboard kind: {kind:?}");

                        Err(Error::ContentNotAvailable)
                    }
                };

                callback(result);
            });
        }

        /// Writes the given contents to the [`Clipboard`].
        pub fn write(
            &mut self,
            content: Content,
            callback: impl FnOnce(Result<(), Error>) + 'static,
        ) {
            let Some(clipboard) = self.clipboard.clone() else {
                callback(Err(Error::ClipboardUnavailable));
                return;
            };

            spawn_local(async move {
                let result = match content {
                    Content::Text(text) => write_text(&clipboard, &text).await,
                    Content::Html(html) => write_html(&clipboard, &html).await,
                    #[cfg(feature = "image")]
                    Content::Image(image) => {
                        write_image(&clipboard, image).await
                    }
                    content => {
                        log::warn!("unsupported clipboard content: {content:?}");

                        Err(Error::ClipboardUnavailable)
                    }
                };

                callback(result);
            });
        }
    }

    async fn read_text(
        clipboard: &web_sys::Clipboard,
    ) -> Result<String, Error> {
        let value = JsFuture::from(clipboard.read_text())
            .await
            .map_err(js_error)?;

        value.as_string().ok_or(Error::ConversionFailure)
    }

    async fn write_text(
        clipboard: &web_sys::Clipboard,
        text: &str,
    ) -> Result<(), Error> {
        let _ = JsFuture::from(clipboard.write_text(text))
            .await
            .map_err(js_error)?;

        Ok(())
    }

    async fn read_html(
        clipboard: &web_sys::Clipboard,
    ) -> Result<String, Error> {
        let blob = read_item_blob(clipboard, "text/html").await?;
        let buffer = JsFuture::from(blob.array_buffer())
            .await
            .map_err(js_error)?;
        let bytes = Uint8Array::new(&buffer).to_vec();

        String::from_utf8(bytes).map_err(|_| Error::ConversionFailure)
    }

    async fn write_html(
        clipboard: &web_sys::Clipboard,
        html: &str,
    ) -> Result<(), Error> {
        let blob = blob_from_str(html, "text/html")?;

        write_item(clipboard, "text/html", &blob).await
    }

    #[cfg(feature = "image")]
    async fn read_image(
        clipboard: &web_sys::Clipboard,
    ) -> Result<crate::core::clipboard::Image, Error> {
        use web_sys::wasm_bindgen::Clamped;

        let blob = read_item_blob(clipboard, "image/png").await?;

        let window = web_sys::window().ok_or(Error::ClipboardUnavailable)?;

        let bitmap = JsFuture::from(
            window
                .create_image_bitmap_with_blob(&blob)
                .map_err(js_error)?,
        )
        .await
        .map_err(js_error)?
        .dyn_into::<web_sys::ImageBitmap>()
        .map_err(|_| Error::ConversionFailure)?;

        let width = bitmap.width();
        let height = bitmap.height();

        let canvas = web_sys::OffscreenCanvas::new(width, height)
            .map_err(js_error)?;

        let context = canvas
            .get_context("2d")
            .map_err(js_error)?
            .ok_or(Error::ConversionFailure)?
            .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
            .map_err(|_| Error::ConversionFailure)?;

        context
            .draw_image_with_image_bitmap(&bitmap, 0.0, 0.0)
            .map_err(js_error)?;

        let image_data = context
            .get_image_data(0.0, 0.0, width as _, height as _)
            .map_err(js_error)?;

        let Clamped(rgba) = image_data.data();

        Ok(crate::core::clipboard::Image {
            rgba: crate::core::Bytes::from_owner(rgba),
            size: crate::core::Size { width, height },
        })
    }

    #[cfg(feature = "image")]
    async fn write_image(
        clipboard: &web_sys::Clipboard,
        image: crate::core::clipboard::Image,
    ) -> Result<(), Error> {
        use web_sys::wasm_bindgen::Clamped;

        let crate::core::clipboard::Image { rgba, size } = image;
        let width = size.width;
        let height = size.height;

        let canvas = web_sys::OffscreenCanvas::new(width, height)
            .map_err(js_error)?;

        let context = canvas
            .get_context("2d")
            .map_err(js_error)?
            .ok_or(Error::ConversionFailure)?
            .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
            .map_err(|_| Error::ConversionFailure)?;

        let image_data =
            web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(rgba.as_ref()),
                width,
                height,
            )
            .map_err(js_error)?;

        context
            .put_image_data(&image_data, 0.0, 0.0)
            .map_err(js_error)?;

        let options = web_sys::ImageEncodeOptions::new();
        options.set_type("image/png");

        let blob = JsFuture::from(
            canvas
                .convert_to_blob_with_options(&options)
                .map_err(js_error)?,
        )
        .await
        .map_err(js_error)?
        .dyn_into::<Blob>()
        .map_err(|_| Error::ConversionFailure)?;

        write_item(clipboard, "image/png", &blob).await
    }

    async fn read_item_blob(
        clipboard: &web_sys::Clipboard,
        mime: &str,
    ) -> Result<Blob, Error> {
        let items = JsFuture::from(clipboard.read())
            .await
            .map_err(js_error)?
            .dyn_into::<Array>()
            .map_err(|_| Error::ConversionFailure)?;

        for index in 0..items.length() {
            let Ok(item) = items.get(index).dyn_into::<ClipboardItem>() else {
                continue;
            };

            let Ok(blob_value) = JsFuture::from(item.get_type(mime)).await
            else {
                continue;
            };

            return blob_value
                .dyn_into::<Blob>()
                .map_err(|_| Error::ConversionFailure);
        }

        Err(Error::ContentNotAvailable)
    }

    fn blob_from_str(content: &str, mime: &str) -> Result<Blob, Error> {
        let parts = Array::new();
        let _ = parts.push(&JsValue::from_str(content));

        let options = BlobPropertyBag::new();
        options.set_type(mime);

        Blob::new_with_str_sequence_and_options(&parts, &options)
            .map_err(js_error)
    }

    async fn write_item(
        clipboard: &web_sys::Clipboard,
        mime: &str,
        blob: &Blob,
    ) -> Result<(), Error> {
        let record = Object::new();
        let _ = Reflect::set(
            &record,
            &JsValue::from_str(mime),
            AsRef::<JsValue>::as_ref(blob),
        )
        .map_err(js_error)?;

        let item =
            ClipboardItem::new_with_record_from_str_to_blob_promise(&record)
                .map_err(js_error)?;

        let items = Array::new();
        let _ = items.push(item.as_ref());

        let _ = JsFuture::from(clipboard.write(&items))
            .await
            .map_err(js_error)?;

        Ok(())
    }

    fn js_error(value: JsValue) -> Error {
        let description = value
            .as_string()
            .or_else(|| {
                value
                    .dyn_ref::<web_sys::js_sys::Error>()
                    .and_then(|error| error.message().as_string())
            })
            .unwrap_or_else(|| "clipboard operation failed".to_owned());

        Error::Unknown {
            description: Arc::new(description),
        }
    }
}
