use crate::core::Color;
use crate::graphics::compositor::{Information, SurfaceError};
use crate::graphics::{Error, Viewport};
use crate::{Renderer, Settings};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub enum Compositor<Theme> {
    #[cfg(feature = "wgpu")]
    Wgpu(iced_wgpu::window::Compositor<Theme>),
    #[cfg(feature = "tiny-skia")]
    TinySkia(iced_tiny_skia::window::Compositor<Theme>),
}

pub enum Surface {
    #[cfg(feature = "wgpu")]
    Wgpu(iced_wgpu::window::Surface),
    #[cfg(feature = "tiny-skia")]
    TinySkia(iced_tiny_skia::window::Surface),
}

impl<Theme> crate::graphics::Compositor for Compositor<Theme> {
    type Settings = Settings;
    type Renderer = Renderer<Theme>;
    type Surface = Surface;

    fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        settings: Self::Settings,
        compatible_window: Option<&W>,
    ) -> Result<(Self, Self::Renderer), Error> {
        #[cfg(feature = "wgpu")]
        let new_wgpu = |settings: Self::Settings, compatible_window| {
            let (compositor, backend) = iced_wgpu::window::compositor::new(
                iced_wgpu::Settings {
                    default_font: settings.default_font,
                    default_text_size: settings.default_text_size,
                    antialiasing: settings.antialiasing,
                    ..iced_wgpu::Settings::from_env()
                },
                compatible_window,
            )?;

            Ok((
                Self::Wgpu(compositor),
                Renderer::new(crate::Backend::Wgpu(backend)),
            ))
        };

        #[cfg(feature = "tiny-skia")]
        let new_tiny_skia = |settings: Self::Settings, _compatible_window| {
            let (compositor, backend) = iced_tiny_skia::window::compositor::new(
                iced_tiny_skia::Settings {
                    default_font: settings.default_font,
                    default_text_size: settings.default_text_size,
                },
            );

            Ok((
                Self::TinySkia(compositor),
                Renderer::new(crate::Backend::TinySkia(backend)),
            ))
        };

        let fail = |_, _| Err(Error::GraphicsAdapterNotFound);

        let candidates = &[
            #[cfg(feature = "wgpu")]
            new_wgpu,
            #[cfg(feature = "tiny-skia")]
            new_tiny_skia,
            fail,
        ];

        let mut error = Error::GraphicsAdapterNotFound;

        for candidate in candidates {
            match candidate(settings, compatible_window) {
                Ok((compositor, renderer)) => {
                    return Ok((compositor, renderer))
                }
                Err(new_error) => {
                    error = new_error;
                }
            }
        }

        Err(error)
    }

    fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &mut self,
        window: &W,
        width: u32,
        height: u32,
    ) -> Surface {
        match self {
            #[cfg(feature = "wgpu")]
            Self::Wgpu(compositor) => {
                Surface::Wgpu(compositor.create_surface(window, width, height))
            }
            #[cfg(feature = "tiny-skia")]
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
            #[cfg(feature = "wgpu")]
            (Self::Wgpu(compositor), Surface::Wgpu(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            #[cfg(feature = "tiny-skia")]
            (Self::TinySkia(compositor), Surface::TinySkia(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            #[allow(unreachable_patterns)]
            _ => panic!(
                "The provided surface is not compatible with the compositor."
            ),
        }
    }

    fn fetch_information(&self) -> Information {
        match self {
            #[cfg(feature = "wgpu")]
            Self::Wgpu(compositor) => compositor.fetch_information(),
            #[cfg(feature = "tiny-skia")]
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
                #[cfg(feature = "wgpu")]
                (
                    Self::Wgpu(compositor),
                    crate::Backend::Wgpu(backend),
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
                #[cfg(feature = "tiny-skia")]
                (
                    Self::TinySkia(compositor),
                    crate::Backend::TinySkia(backend),
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
                #[allow(unreachable_patterns)]
                _ => panic!(
                    "The provided renderer or surface are not compatible \
                    with the compositor."
                ),
            }
        })
    }

    fn render_offscreen<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Vec<u8> {
        renderer.with_primitives(|backend, primitives| match (self, backend) {
            #[cfg(feature = "wgpu")]
            (Self::Wgpu(compositor), crate::Backend::Wgpu(backend)) => {
                iced_wgpu::window::compositor::render_offscreen(
                    compositor,
                    backend,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                )
            },
            #[cfg(feature = "tiny-skia")]
            (Self::TinySkia(compositor), crate::Backend::TinySkia(backend)) => {
                iced_tiny_skia::window::compositor::render_offscreen(
                    compositor,
                    backend,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                )
            }
            #[allow(unreachable_patterns)]
            _ => panic!(
                "The provided renderer or backend are not compatible with the compositor."
            ),
        })
    }
}
