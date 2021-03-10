/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
pub struct Clipboard(window_clipboard::Clipboard);

impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn connect(window: &winit::window::Window) -> Option<Clipboard> {
        window_clipboard::Clipboard::connect(window)
            .map(Clipboard)
            .ok()
    }
}

impl iced_native::Clipboard for Clipboard {
    fn read(&self) -> Option<String> {
        self.0.read().ok()
    }

    fn write(&mut self, contents: String) {
        match self.0.write(contents) {
            Ok(()) => {}
            Err(error) => log::warn!("error writing to clipboard: {}", error),
        }
    }
}
