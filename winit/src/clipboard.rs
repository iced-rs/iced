pub use inner::*;

#[cfg(target_arch = "wasm32-unknown-unknown")]
mod inner {
    /// A buffer for short-term storage and transfer within and between
    /// applications.
    #[allow(missing_debug_implementations)]
    pub struct Clipboard(window_clipboard::Clipboard);

    impl Clipboard {
        /// Creates a new [`Clipboard`] for the given window.
        ///
        /// [`Clipboard`]: struct.Clipboard.html
        pub fn new(window: &winit::window::Window) -> Option<Clipboard> {
            window_clipboard::Clipboard::new(window).map(Clipboard).ok()
        }
    }

    impl iced_native::Clipboard for Clipboard {
        fn content(&self) -> Option<String> {
            self.0.read().ok()
        }
    }
}

#[cfg(not(target_arch = "wasm32-unknown-unknown"))]
mod inner {
    /// A buffer for short-term storage and transfer within and between
    /// applications.
    #[allow(missing_debug_implementations)]
    pub struct Clipboard;

    impl Clipboard {
        /// Creates a new [`Clipboard`] for the given window.
        ///
        /// [`Clipboard`]: struct.Clipboard.html
        pub fn new(_window: &winit::window::Window) -> Option<Clipboard> {
            Some(Clipboard)
        }
    }

    impl iced_native::Clipboard for Clipboard {
        fn content(&self) -> Option<String> {
            Some("".to_string())
        }
    }
}
