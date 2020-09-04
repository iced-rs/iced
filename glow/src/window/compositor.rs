use crate::{Backend, Color, Renderer, Settings, Viewport};

use core::ffi::c_void;
use glow::HasContext;
use iced_graphics::{Antialiasing, Size};
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
        #[cfg(not(target_arch = "wasm32"))]
        let gl = glow::Context::from_loader_function(loader_function);

        #[cfg(all(target_arch = "wasm32"))]
        let (gl, _render_loop, _shader_version) = {
            use wasm_bindgen::JsCast;
            let canvas = web_sys::window()
                .expect("iced web_sys window")
                .document()
                .expect("iced web_sys window.document")
                .get_element_by_id("iced-is-good-gui")
                .expect("iced web_sys window.document.get_element_by_id(\"canvas\")")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("iced web_sys window.document.get_element_by_id(\"canvas\") as web_sys::HtmlCanvasElement");
            let webgl2_context = canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::WebGl2RenderingContext>()
                .unwrap();
            (
                glow::Context::from_webgl2_context(webgl2_context),
                glow::RenderLoop::from_request_animation_frame(),
                "#version 300 es",
            )
        };

        // Enable auto-conversion from/to sRGB
        gl.enable(glow::FRAMEBUFFER_SRGB);

        // Enable alpha blending
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        // Disable multisampling by default
        gl.disable(glow::MULTISAMPLE);

        let renderer = Renderer::new(Backend::new(&gl, settings));

        (Self { gl }, renderer)
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

    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        color: Color,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        overlay: &[T],
    ) -> mouse::Interaction {
        let gl = &self.gl;

        let [r, g, b, a] = color.into_linear();

        unsafe {
            gl.clear_color(r, g, b, a);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        renderer.backend_mut().draw(gl, viewport, output, overlay)
    }
}
