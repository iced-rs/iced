use crate::core::Color;
use crate::graphics::compositor::{self, Information, SurfaceError};
use crate::graphics::{Error, Primitive, Viewport};
use crate::{Backend, Renderer, Settings};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::marker::PhantomData;
use std::num::NonZeroU32;

pub struct Compositor<Theme> {
    clip_mask: tiny_skia::ClipMask,
    _theme: PhantomData<Theme>,
}

pub struct Surface {
    window: softbuffer::Surface,
}

impl<Theme> crate::graphics::Compositor for Compositor<Theme> {
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
        let platform = unsafe { softbuffer::Context::new(window) }
            .expect("Create softbuffer context");

        let mut window = unsafe { softbuffer::Surface::new(&platform, window) }
            .expect("Create softbuffer surface");

        window
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .expect("Resize surface");

        Surface { window }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Surface,
        width: u32,
        height: u32,
    ) {
        surface
            .window
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .expect("Resize surface");
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
    // TOD
    (
        Compositor {
            clip_mask: tiny_skia::ClipMask::new(),
            _theme: PhantomData,
        },
        Backend::new(settings),
    )
}

pub fn present<Theme, T: AsRef<str>>(
    compositor: &mut Compositor<Theme>,
    backend: &mut Backend,
    surface: &mut Surface,
    primitives: &[Primitive],
    viewport: &Viewport,
    background_color: Color,
    overlay: &[T],
) -> Result<(), compositor::SurfaceError> {
    let physical_size = viewport.physical_size();

    let mut buffer = surface.window.buffer_mut().expect("Get window buffer");

    let drawn = backend.draw(
        &mut tiny_skia::PixmapMut::from_bytes(
            bytemuck::cast_slice_mut(&mut buffer),
            physical_size.width,
            physical_size.height,
        )
        .expect("Create pixel map"),
        &mut compositor.clip_mask,
        primitives,
        viewport,
        background_color,
        overlay,
    );

    if drawn {
        let _ = buffer.present();
    }

    Ok(())
}
