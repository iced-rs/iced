pub struct Clipboard(window_clipboard::Clipboard);

impl Clipboard {
    pub fn new(window: &winit::window::Window) -> Option<Clipboard> {
        window_clipboard::Clipboard::new(window).map(Clipboard).ok()
    }
}

impl iced_native::Clipboard for Clipboard {
    fn content(&self) -> Option<String> {
        self.0.read().ok()
    }
}
