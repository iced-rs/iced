use crate::{Size, Viewport};
use iced_native::mouse;

use core::ffi::c_void;

pub trait GLCompositor: Sized {
    type Renderer: iced_native::Renderer;
    type Settings: Default;

    unsafe fn new(
        settings: Self::Settings,
        loader_function: impl FnMut(&str) -> *const c_void,
    ) -> (Self, Self::Renderer);

    fn sample_count(settings: &Self::Settings) -> u32;

    fn resize_viewport(&mut self, physical_size: Size<u32>);

    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        overlay: &[T],
    ) -> mouse::Interaction;
}
