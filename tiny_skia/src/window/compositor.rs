use crate::core::Color;
use crate::graphics::compositor::{Information, SurfaceError};
use crate::graphics::{Error, Primitive, Viewport};
use crate::{Backend, Renderer, Settings};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::marker::PhantomData;

pub struct Compositor<Theme> {
    clip_mask: tiny_skia::ClipMask,
    _theme: PhantomData<Theme>,
}

pub struct Surface {
    window: softbuffer::GraphicsContext,
    buffer: Vec<u32>,
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
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Surface,
        width: u32,
        height: u32,
    ) {
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

    fn screenshot<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Vec<u8> {
        renderer.with_primitives(|backend, primitives| {
            screenshot(
                self,
                backend,
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
) -> Result<(), SurfaceError> {
    let physical_size = viewport.physical_size();

    backend.draw(
        &mut tiny_skia::PixmapMut::from_bytes(
            bytemuck::cast_slice_mut(&mut surface.buffer),
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

    surface.window.set_buffer(
        &surface.buffer,
        physical_size.width as u16,
        physical_size.height as u16,
    );

    Ok(())
}

pub fn screenshot<Theme, T: AsRef<str>>(
    compositor: &mut Compositor<Theme>,
    backend: &mut Backend,
    primitives: &[Primitive],
    viewport: &Viewport,
    background_color: Color,
    overlay: &[T],
) -> Vec<u8> {
    #[cfg(feature = "tracing")]
    let _ = tracing::info_span!("iced_tiny_skia::RENDER_OFFSCREEN").entered();

    let size = viewport.physical_size();

    let mut offscreen_buffer: Vec<u32> =
        vec![0; size.width as usize * size.height as usize];

    backend.draw(
        &mut tiny_skia::PixmapMut::from_bytes(
            bytemuck::cast_slice_mut(&mut offscreen_buffer),
            size.width,
            size.height,
        )
        .expect("Create offscreen pixel map"),
        &mut compositor.clip_mask,
        primitives,
        viewport,
        background_color,
        overlay,
    );

    offscreen_buffer.iter().fold(
        Vec::with_capacity(offscreen_buffer.len() * 4),
        |mut acc, pixel| {
            const A_MASK: u32 = 0xFF_00_00_00;
            const R_MASK: u32 = 0x00_FF_00_00;
            const G_MASK: u32 = 0x00_00_FF_00;
            const B_MASK: u32 = 0x00_00_00_FF;

            let a = ((A_MASK & pixel) >> 24) as u8;
            let r = ((R_MASK & pixel) >> 16) as u8;
            let g = ((G_MASK & pixel) >> 8) as u8;
            let b = (B_MASK & pixel) as u8;

            acc.extend([r, g, b, a]);
            acc
        },
    )
}
