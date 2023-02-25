use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use iced_graphics::window::compositor::{self, Information, SurfaceError};
use iced_graphics::Primitive;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::marker::PhantomData;

pub struct Compositor<Theme> {
    _theme: PhantomData<Theme>,
}

pub struct Surface {
    window: softbuffer::GraphicsContext,
    pixels: tiny_skia::Pixmap,
    buffer: Vec<u32>,
}

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
        window: &W,
        width: u32,
        height: u32,
    ) -> Surface {
        let window =
            unsafe { softbuffer::GraphicsContext::new(window, window) }
                .expect("Create softbuffer for window");

        let pixels = tiny_skia::Pixmap::new(width, height)
            .expect("Create pixmap for window");

        Surface {
            window,
            pixels,
            buffer: vec![0; (width * height) as usize],
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Surface,
        width: u32,
        height: u32,
    ) {
        surface.pixels = tiny_skia::Pixmap::new(width, height)
            .expect("Create pixmap for window");

        surface.buffer.resize((width * height) as usize, 0);
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
    backend: &mut Backend,
    surface: &mut Surface,
    primitives: &[Primitive],
    viewport: &Viewport,
    background_color: Color,
    overlay: &[T],
) -> Result<(), compositor::SurfaceError> {
    backend.draw(
        &mut surface.pixels,
        primitives,
        viewport,
        background_color,
        overlay,
    );

    for (i, pixel) in surface.pixels.pixels_mut().iter().enumerate() {
        surface.buffer[i] = u32::from(pixel.red()) << 16
            | u32::from(pixel.green()) << 8
            | u32::from(pixel.blue());
    }

    surface.window.set_buffer(
        &surface.buffer,
        surface.pixels.width() as u16,
        surface.pixels.height() as u16,
    );

    // TODO
    Ok(())
}
