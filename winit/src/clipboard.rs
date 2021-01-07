/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
pub struct Clipboard<'a>(
    window_clipboard::Clipboard,
    &'a winit::window::Window,
);

impl<'a> Clipboard<'a> {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn new(window: &'a winit::window::Window) -> Option<Clipboard<'a>> {
        window_clipboard::Clipboard::new(window)
            .map(|clipboard| Clipboard(clipboard, window))
            .ok()
    }
}

impl<'a> iced_native::Clipboard for Clipboard<'a> {
    fn content(&self) -> Option<String> {
        self.0.read().ok()
    }

    fn set_ime_position(&self, position: iced_core::Point) {
        self.1.set_ime_position(winit::dpi::LogicalPosition::new(
            position.x, position.y,
        ));
    }
}
