use crate::{Metrics, MouseCursor};

use raw_window_handle::HasRawWindowHandle;

pub trait Windowed: super::Renderer + Sized {
    type Target: Target<Renderer = Self>;

    fn new() -> Self;

    fn draw(
        &mut self,
        output: &Self::Output,
        metrics: Option<Metrics>,
        target: &mut Self::Target,
    ) -> MouseCursor;
}

pub trait Target {
    type Renderer;

    fn new<W: HasRawWindowHandle>(
        window: &W,
        width: u16,
        height: u16,
        renderer: &Self::Renderer,
    ) -> Self;

    fn resize(&mut self, width: u16, height: u16, renderer: &Self::Renderer);
}
