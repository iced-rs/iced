use raw_window_handle::HasRawWindowHandle;

/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
pub struct Clipboard(
    window_clipboard::Clipboard,
    raw_window_handle::RawWindowHandle,
    pub crate::Viewport,
);

impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn new(
        window: &winit::window::Window,
        viewport: crate::Viewport,
    ) -> Option<Clipboard> {
        window_clipboard::Clipboard::new(window)
            .map(|clipboard| {
                Clipboard(clipboard, window.raw_window_handle(), viewport)
            })
            .ok()
    }
}

impl iced_native::Clipboard for Clipboard {
    fn content(&self) -> Option<String> {
        self.0.read().ok()
    }

    fn set_input_method_position(&self, position: iced_core::Point) {
        #[cfg(target_os = "windows")]
        {
            if let raw_window_handle::RawWindowHandle::Windows(handle) = self.1
            {
                let himc =
                    unsafe { winapi::um::imm::ImmGetContext(handle.hwnd as _) };

                let mut composition_form = winapi::um::imm::COMPOSITIONFORM {
                    dwStyle: winapi::um::imm::CFS_POINT,
                    ptCurrentPos: winapi::shared::windef::POINT {
                        x: (position.x * self.2.scale_factor() as f32) as _,
                        y: (position.y * self.2.scale_factor() as f32) as _,
                    },
                    rcArea: winapi::shared::windef::RECT {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                    },
                };

                let _ = unsafe {
                    winapi::um::imm::ImmSetCompositionWindow(
                        himc,
                        &mut composition_form as _,
                    )
                };

                let _ = unsafe {
                    winapi::um::imm::ImmReleaseContext(handle.hwnd as _, himc)
                };
            }
        }
    }
}
