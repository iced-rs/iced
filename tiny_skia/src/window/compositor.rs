use crate::core::{Color, Rectangle};
use crate::graphics::compositor::{self, Information, SurfaceError};
use crate::graphics::damage;
use crate::graphics::{Error, Primitive, Viewport};
use crate::{Backend, Renderer, Settings};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::marker::PhantomData;

pub struct Compositor<Theme> {
    _theme: PhantomData<Theme>,
}

pub struct Surface {
    window: softbuffer::GraphicsContext,
    buffer: Vec<u32>,
    clip_mask: tiny_skia::Mask,
    last_primitives: Vec<Primitive>,
    last_background_color: Color,
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
        let window =
            unsafe { softbuffer::GraphicsContext::new(window, window) }
                .expect("Create softbuffer for window");

        Surface {
            window,
            buffer: vec![0; width as usize * height as usize],
            clip_mask: tiny_skia::Mask::new(width, height)
                .expect("Create clip mask"),
            last_primitives: Vec::new(),
            last_background_color: Color::BLACK,
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Surface,
        width: u32,
        height: u32,
    ) {
        surface.buffer.resize((width * height) as usize, 0);
        surface.clip_mask =
            tiny_skia::Mask::new(width, height).expect("Create clip mask");

        surface.last_primitives.clear();
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
    (
        Compositor {
            _theme: PhantomData,
        },
        Backend::new(settings),
    )
}

pub fn present<T: AsRef<str>>(
    backend: &mut Backend,
    surface: &mut Surface,
    primitives: &[Primitive],
    viewport: &Viewport,
    background_color: Color,
    overlay: &[T],
) -> Result<(), compositor::SurfaceError> {
    let physical_size = viewport.physical_size();
    let scale_factor = viewport.scale_factor() as f32;

    let mut pixels = tiny_skia::PixmapMut::from_bytes(
        bytemuck::cast_slice_mut(&mut surface.buffer),
        physical_size.width,
        physical_size.height,
    )
    .expect("Create pixel map");

    let damage = if surface.last_background_color == background_color {
        damage::list(&surface.last_primitives, primitives)
    } else {
        vec![Rectangle::with_size(viewport.logical_size())]
    };

    if damage.is_empty() {
        return Ok(());
    }

    surface.last_primitives = primitives.to_vec();
    surface.last_background_color = background_color;

    let damage = damage::group(damage, scale_factor, physical_size);

    backend.draw(
        &mut pixels,
        &mut surface.clip_mask,
        primitives,
        viewport,
        &damage,
        background_color,
        overlay,
    );

    surface.window.set_buffer(
        &surface.buffer,
        physical_size.width as u16,
        physical_size.height as u16,
    );

    Ok(())
}
