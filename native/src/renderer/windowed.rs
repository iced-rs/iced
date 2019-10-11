use crate::MouseCursor;

use raw_window_handle::HasRawWindowHandle;

pub trait Windowed: super::Renderer {
    type Target;

    fn new<W: HasRawWindowHandle>(window: &W) -> Self;

    fn target(&self, width: u16, height: u16) -> Self::Target;

    fn draw(
        &mut self,
        output: &Self::Output,
        target: &mut Self::Target,
    ) -> MouseCursor;
}
