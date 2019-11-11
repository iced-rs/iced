use crate::MouseCursor;

use raw_window_handle::HasRawWindowHandle;

use crate::Color;
#[derive(Debug)]
pub struct Style {
    pub window_background : Color,
}

impl Style {
    pub fn default() -> Self { Self{ window_background: Color{r:1.,g:1.,b:1.,a:1.}, } }
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
