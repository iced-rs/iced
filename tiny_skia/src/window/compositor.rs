use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use iced_graphics::window::compositor::{self, Information, SurfaceError};
use iced_graphics::Primitive;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::marker::PhantomData;

pub struct Compositor<Theme> {
    _theme: PhantomData<Theme>,
}

pub struct Surface;

impl<Theme> iced_graphics::window::Compositor for Compositor<Theme> {
    type Settings = Settings;
    type Renderer = Renderer<Theme>;
    type Surface = Surface;

    fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        settings: Self::Settings,
        _compatible_window: Option<&W>,
    ) -> Result<(Self, Self::Renderer), Error> {
        let (compositor, backend) = new(settings);

        Ok((compositor, Renderer::new(backend)))
    }

    fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &mut self,
        _window: &W,
    ) -> Surface {
        // TODO
        Surface
    }

    fn configure_surface(
        &mut self,
        _surface: &mut Surface,
        _width: u32,
        _height: u32,
    ) {
        // TODO
    }

    fn fetch_information(&self) -> Information {
        Information {
            adapter: String::from("CPU"),
            backend: String::from("tiny-skia"),
        }
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), SurfaceError> {
        renderer.with_primitives(|backend, primitives| {
            present(
                self,
                backend,
                surface,
                primitives,
                viewport,
                background_color,
                overlay,
            )
        })
    }
}

pub fn new<Theme>(settings: Settings) -> (Compositor<Theme>, Backend) {
    // TODO
    (
        Compositor {
            _theme: PhantomData,
        },
        Backend::new(settings),
    )
}

pub fn present<Theme, T: AsRef<str>>(
    _compositor: &mut Compositor<Theme>,
    _backend: &mut Backend,
    _surface: &mut Surface,
    _primitives: &[Primitive],
    _viewport: &Viewport,
    _background_color: Color,
    _overlay: &[T],
) -> Result<(), compositor::SurfaceError> {
    // TODO
    Ok(())
}
