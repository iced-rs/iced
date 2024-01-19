use crate::core::Color;
use crate::graphics::compositor::{Information, SurfaceError, Window};
use crate::graphics::{Error, Viewport};
use crate::{Renderer, Settings};

use std::env;

pub enum Compositor<Theme> {
    TinySkia(iced_tiny_skia::window::Compositor<Theme>),
    #[cfg(feature = "wgpu")]
    Wgpu(iced_wgpu::window::Compositor<Theme>),
}

pub enum Surface {
    TinySkia(iced_tiny_skia::window::Surface),
    #[cfg(feature = "wgpu")]
    Wgpu(iced_wgpu::window::Surface<'static>),
}

impl<Theme> crate::graphics::Compositor for Compositor<Theme> {
    type Settings = Settings;
    type Renderer = Renderer<Theme>;
    type Surface = Surface;

    fn new<W: Window + Clone>(
        settings: Self::Settings,
        compatible_window: W,
    ) -> Result<Self, Error> {
        let candidates =
            Candidate::list_from_env().unwrap_or(Candidate::default_list());

        let mut error = Error::GraphicsAdapterNotFound;

        for candidate in candidates {
            match candidate.build(settings, compatible_window.clone()) {
                Ok(compositor) => return Ok(compositor),
                Err(new_error) => {
                    error = new_error;
                }
            }
        }

        Err(error)
    }

    fn create_renderer(&self) -> Self::Renderer {
        match self {
            Compositor::TinySkia(compositor) => {
                Renderer::TinySkia(compositor.create_renderer())
            }
            #[cfg(feature = "wgpu")]
            Compositor::Wgpu(compositor) => {
                Renderer::Wgpu(compositor.create_renderer())
            }
        }
    }

    fn create_surface<W: Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Surface {
        match self {
            Self::TinySkia(compositor) => Surface::TinySkia(
                compositor.create_surface(window, width, height),
            ),
            #[cfg(feature = "wgpu")]
            Self::Wgpu(compositor) => {
                Surface::Wgpu(compositor.create_surface(window, width, height))
            }
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Surface,
        width: u32,
        height: u32,
    ) {
        match (self, surface) {
            (Self::TinySkia(compositor), Surface::TinySkia(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            #[cfg(feature = "wgpu")]
            (Self::Wgpu(compositor), Surface::Wgpu(surface)) => {
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
            Self::TinySkia(compositor) => compositor.fetch_information(),
            #[cfg(feature = "wgpu")]
            Self::Wgpu(compositor) => compositor.fetch_information(),
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
        match (self, renderer, surface) {
            (
                Self::TinySkia(_compositor),
                crate::Renderer::TinySkia(renderer),
                Surface::TinySkia(surface),
            ) => renderer.with_primitives(|backend, primitives| {
                iced_tiny_skia::window::compositor::present(
                    backend,
                    surface,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                )
            }),
            #[cfg(feature = "wgpu")]
            (
                Self::Wgpu(compositor),
                crate::Renderer::Wgpu(renderer),
                Surface::Wgpu(surface),
            ) => renderer.with_primitives(|backend, primitives| {
                iced_wgpu::window::compositor::present(
                    compositor,
                    backend,
                    surface,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                )
            }),
            #[allow(unreachable_patterns)]
            _ => panic!(
                "The provided renderer or surface are not compatible \
                    with the compositor."
            ),
        }
    }

    fn screenshot<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Vec<u8> {
        match (self, renderer, surface) {
            (
                Self::TinySkia(_compositor),
                Renderer::TinySkia(renderer),
                Surface::TinySkia(surface),
            ) => renderer.with_primitives(|backend, primitives| {
                iced_tiny_skia::window::compositor::screenshot(
                    surface,
                    backend,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                )
            }),
            #[cfg(feature = "wgpu")]
            (
                Self::Wgpu(compositor),
                Renderer::Wgpu(renderer),
                Surface::Wgpu(_),
            ) => renderer.with_primitives(|backend, primitives| {
                iced_wgpu::window::compositor::screenshot(
                    compositor,
                    backend,
                    primitives,
                    viewport,
                    background_color,
                    overlay,
                )
            }),
            #[allow(unreachable_patterns)]
            _ => panic!(
                "The provided renderer or backend are not compatible \
                with the compositor."
            ),
        }
    }
}

enum Candidate {
    Wgpu,
    TinySkia,
}

impl Candidate {
    fn default_list() -> Vec<Self> {
        vec![
            #[cfg(feature = "wgpu")]
            Self::Wgpu,
            Self::TinySkia,
        ]
    }

    fn list_from_env() -> Option<Vec<Self>> {
        let backends = env::var("ICED_BACKEND").ok()?;

        Some(
            backends
                .split(',')
                .map(str::trim)
                .map(|backend| match backend {
                    "wgpu" => Self::Wgpu,
                    "tiny-skia" => Self::TinySkia,
                    _ => panic!("unknown backend value: \"{backend}\""),
                })
                .collect(),
        )
    }

    fn build<Theme, W: Window>(
        self,
        settings: Settings,
        _compatible_window: W,
    ) -> Result<Compositor<Theme>, Error> {
        match self {
            Self::TinySkia => {
                let compositor = iced_tiny_skia::window::compositor::new(
                    iced_tiny_skia::Settings {
                        default_font: settings.default_font,
                        default_text_size: settings.default_text_size,
                    },
                    _compatible_window,
                );

                Ok(Compositor::TinySkia(compositor))
            }
            #[cfg(feature = "wgpu")]
            Self::Wgpu => {
                let compositor = iced_wgpu::window::compositor::new(
                    iced_wgpu::Settings {
                        default_font: settings.default_font,
                        default_text_size: settings.default_text_size,
                        antialiasing: settings.antialiasing,
                        ..iced_wgpu::Settings::from_env()
                    },
                    _compatible_window,
                )?;

                Ok(Compositor::Wgpu(compositor))
            }
            #[cfg(not(feature = "wgpu"))]
            Self::Wgpu => {
                panic!("`wgpu` feature was not enabled in `iced_renderer`")
            }
        }
    }
}
