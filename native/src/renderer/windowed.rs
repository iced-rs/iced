use crate::MouseCursor;

use raw_window_handle::HasRawWindowHandle;

use crate::Color;
#[derive(Debug)]
pub struct Style {
    pub window_background : Color,
}

pub trait Windowed: super::Renderer + Sized {
    type Target: Target<Renderer = Self>;

    fn new(style : Style) -> Self;

    fn draw<T: AsRef<str>>(
        &mut self,
        output: &Self::Output,
        overlay: &[T],
        target: &mut Self::Target,
    ) -> MouseCursor;
}

pub trait Target {
    type Renderer;

    fn new<W: HasRawWindowHandle>(
        window: &W,
        width: u16,
        height: u16,
        dpi: f32,
        renderer: &Self::Renderer,
    ) -> Self;

    fn resize(
        &mut self,
        width: u16,
        height: u16,
        dpi: f32,
        renderer: &Self::Renderer,
    );
}
