use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use glow::HasContext;
use iced_graphics::{compositor, Antialiasing, Size};

use core::ffi::c_void;
use std::marker::PhantomData;

/// A window graphics backend for iced powered by `glow`.
#[allow(missing_debug_implementations)]
pub struct Compositor<Theme> {
    gl: glow::Context,
    theme: PhantomData<Theme>,
}

impl<Theme> iced_graphics::window::GLCompositor for Compositor<Theme> {
    type Settings = Settings;
    type Renderer = Renderer<Theme>;

    unsafe fn new(
        settings: Self::Settings,
        loader_function: impl FnMut(&str) -> *const c_void,
    ) -> Result<(Self, Self::Renderer), Error> {
        let gl = glow::Context::from_loader_function(loader_function);

        log::info!("{:#?}", settings);

        let version = gl.version();
        log::info!("Version: {:?}", version);
        log::info!("Embedded: {}", version.is_embedded);

        let renderer = gl.get_parameter_string(glow::RENDERER);
        log::info!("Renderer: {}", renderer);

        // Enable auto-conversion from/to sRGB
        gl.enable(glow::FRAMEBUFFER_SRGB);

        // Enable alpha blending
        gl.enable(glow::BLEND);
        gl.blend_func_separate(
            glow::SRC_ALPHA,
            glow::ONE_MINUS_SRC_ALPHA,
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
        );

        // Disable multisampling by default
        gl.disable(glow::MULTISAMPLE);

        let renderer = Renderer::new(Backend::new(&gl, settings));

        Ok((
            Self {
                gl,
                theme: PhantomData,
            },
            renderer,
        ))
    }

    fn sample_count(settings: &Settings) -> u32 {
        settings
            .antialiasing
            .map(Antialiasing::sample_count)
            .unwrap_or(0)
    }

    fn resize_viewport(&mut self, physical_size: Size<u32>) {
        unsafe {
            self.gl.viewport(
                0,
                0,
                physical_size.width as i32,
                physical_size.height as i32,
            );
        }
    }

    fn fetch_information(&self) -> compositor::Information {
        let adapter = unsafe { self.gl.get_parameter_string(glow::RENDERER) };

        compositor::Information {
            backend: format!("{:?}", self.gl.version()),
            adapter,
        }
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        color: Color,
        overlay: &[T],
    ) {
        let gl = &self.gl;

        let [r, g, b, a] = color.into_linear();

        unsafe {
            gl.clear_color(r, g, b, a);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        renderer.with_primitives(|backend, primitive| {
            backend.present(gl, primitive, viewport, overlay);
        });
    }
}
