use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub use iced_graphics::window::compositor::{Information, SurfaceError};

pub enum Compositor<Theme> {
    Wgpu(iced_wgpu::window::Compositor<Theme>),
    TinySkia(iced_tiny_skia::window::Compositor<Theme>),
}

pub enum Surface {
    Wgpu(iced_wgpu::window::Surface),
    TinySkia(iced_tiny_skia::window::Surface),
}

impl<Theme> iced_graphics::window::Compositor for Compositor<Theme> {
    type Settings = Settings;
    type Renderer = Renderer<Theme>;
    type Surface = Surface;

    fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        settings: Self::Settings,
        _compatible_window: Option<&W>,
    ) -> Result<(Self, Self::Renderer), Error> {
        //let (compositor, backend) = iced_wgpu::window::compositor::new(
        //    iced_wgpu::Settings {
        //        default_font: settings.default_font,
        //        default_text_size: settings.default_text_size,
        //        antialiasing: settings.antialiasing,
        //        ..iced_wgpu::Settings::from_env()
        //    },
        //    compatible_window,
        //)?;

        //Ok((
        //    Self::Wgpu(compositor),
        //    Renderer::new(Backend::Wgpu(backend)),
        //))
        let (compositor, backend) =
            iced_tiny_skia::window::compositor::new(iced_tiny_skia::Settings {
                default_font: settings.default_font,
                default_text_size: settings.default_text_size,
            });

        Ok((
            Self::TinySkia(compositor),
            Renderer::new(Backend::TinySkia(backend)),
        ))
    }

    fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &mut self,
        window: &W,
        width: u32,
        height: u32,
    ) -> Surface {
        match self {
            Self::Wgpu(compositor) => {
                Surface::Wgpu(compositor.create_surface(window, width, height))
            }
            Self::TinySkia(compositor) => Surface::TinySkia(
                compositor.create_surface(window, width, height),
            ),
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Surface,
        width: u32,
        height: u32,
    ) {
        match (self, surface) {
            (Self::Wgpu(compositor), Surface::Wgpu(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            (Self::TinySkia(compositor), Surface::TinySkia(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            _ => unreachable!(),
        }
    }

    fn fetch_information(&self) -> Information {
        match self {
            Self::Wgpu(compositor) => compositor.fetch_information(),
            Self::TinySkia(compositor) => compositor.fetch_information(),
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
            match (self, backend, surface) {
                (
                    Self::Wgpu(compositor),
                    Backend::Wgpu(backend),
                    Surface::Wgpu(surface),
                ) => iced_wgpu::window::compositor::present(
                    compositor,
                    backend,
                    surface,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                ),
                (
                    Self::TinySkia(compositor),
                    Backend::TinySkia(backend),
                    Surface::TinySkia(surface),
                ) => iced_tiny_skia::window::compositor::present(
                    compositor,
                    backend,
                    surface,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                ),
                _ => unreachable!(),
            }
        })
    }
}
