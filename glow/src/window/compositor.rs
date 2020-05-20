use crate::{Backend, Renderer, Settings, Viewport};

use core::ffi::c_void;
use glow::HasContext;
use iced_graphics::Size;
use iced_native::mouse;

/// A window graphics backend for iced powered by `glow`.
#[allow(missing_debug_implementations)]
pub struct Compositor {
    gl: glow::Context,
}

impl iced_graphics::window::GLCompositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;

    unsafe fn new(
        settings: Self::Settings,
        loader_function: impl FnMut(&str) -> *const c_void,
    ) -> (Self, Self::Renderer) {
        let gl = glow::Context::from_loader_function(loader_function);

        gl.clear_color(1.0, 1.0, 1.0, 1.0);

        // Enable auto-conversion from/to sRGB
        gl.enable(glow::FRAMEBUFFER_SRGB);

        // Enable alpha blending
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        let renderer = Renderer::new(Backend::new(&gl, settings));

        (Self { gl }, renderer)
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

    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        overlay: &[T],
    ) -> mouse::Interaction {
        let gl = &self.gl;

        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        renderer.backend_mut().draw(gl, viewport, output, overlay)
    }
}
