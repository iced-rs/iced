use crate::core::Color;
use crate::graphics::compositor::{self, Information, SurfaceError};
use crate::graphics::{Error, Primitive, Viewport};
use crate::{Backend, Renderer, Settings};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::marker::PhantomData;

pub struct Compositor<Theme> {
    clip_mask: tiny_skia::ClipMask,
    _theme: PhantomData<Theme>,
}

pub enum Surface {
    Cpu {
        window: softbuffer::GraphicsContext,
        buffer: Vec<u32>,
    },
    #[cfg(feature = "gpu")]
    Gpu { pixels: pixels::Pixels },
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
        #[cfg(feature = "gpu")]
        {
            let surface_texture =
                pixels::SurfaceTexture::new(width, height, window);

            if let Ok(pixels) =
                pixels::PixelsBuilder::new(width, height, surface_texture)
                    .texture_format(pixels::wgpu::TextureFormat::Bgra8UnormSrgb)
                    .build()
            {
                log::info!("GPU surface created");

                return Surface::Gpu { pixels };
            }
        }

        let window =
            unsafe { softbuffer::GraphicsContext::new(window, window) }
                .expect("Create softbuffer for window");

        log::info!("CPU surface created");

        Surface::Cpu {
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
        match surface {
            Surface::Cpu { buffer, .. } => {
                buffer.resize((width * height) as usize, 0);
            }
            #[cfg(feature = "gpu")]
            Surface::Gpu { pixels } => {
                pixels
                    .resize_surface(width, height)
                    .expect("Resize surface");

                pixels.resize_buffer(width, height).expect("Resize buffer");
            }
        }
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

    let buffer = match surface {
        Surface::Cpu { buffer, .. } => bytemuck::cast_slice_mut(buffer),
        #[cfg(feature = "gpu")]
        Surface::Gpu { pixels } => pixels.frame_mut(),
    };

    let drawn = backend.draw(
        &mut tiny_skia::PixmapMut::from_bytes(
            buffer,
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
        match surface {
            Surface::Cpu { window, buffer } => {
                window.set_buffer(
                    buffer,
                    physical_size.width as u16,
                    physical_size.height as u16,
                );

                Ok(())
            }
            #[cfg(feature = "gpu")]
            Surface::Gpu { pixels } => {
                pixels.render().map_err(|_| compositor::SurfaceError::Lost)
            }
        }
    } else {
        Ok(())
    }
}
