use crate::{Backend, Color, Error, Renderer, Settings, Viewport};

use core::ffi::c_void;
use glow::HasContext;
use iced_graphics::{Antialiasing, Rectangle, Size};
use iced_native::mouse;

/// A window graphics backend for iced powered by `glow`.
#[allow(missing_debug_implementations)]
pub struct Compositor {
    gl: glow::Context,
    framebuffer: Option<glow::Framebuffer>,
}

impl iced_graphics::window::GLCompositor for Compositor {
    type Settings = Settings;
    type Renderer = Renderer;

    unsafe fn new(
        settings: Self::Settings,
        viewport_size: Size<u32>,
        loader_function: impl FnMut(&str) -> *const c_void,
    ) -> Result<(Self, Self::Renderer), Error> {
        let gl = glow::Context::from_loader_function(loader_function);

        // Enable auto-conversion from/to sRGB
        gl.enable(glow::FRAMEBUFFER_SRGB);

        // Enable alpha blending
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        // Disable multisampling by default
        gl.disable(glow::MULTISAMPLE);

        let renderer = Renderer::new(Backend::new(&gl, settings));
        
        // Allocate an offscreen framebuffer if needed
        #[cfg(feature = "offscreen")]
        let framebuffer = {
            let renderbuffer = gl.create_renderbuffer().ok();
            gl.bind_renderbuffer(glow::RENDERBUFFER, renderbuffer);
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::RGB8, viewport_size.width as i32, viewport_size.height as i32);
            
            let framebuffer = gl.create_framebuffer().ok();
            gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer);
            gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::RENDERBUFFER, renderbuffer);

            framebuffer 
        };

        // `glow` translates `None` as the reserved 'zero' system-shared framebuffer (so, `None` is the same as `Some(0)`)
        #[cfg(not(feature = "offscreen"))]
        let framebuffer = None;        

        Ok((Self { gl, framebuffer }, renderer))
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

        renderer.backend_mut().draw(gl, viewport, output, overlay, self.framebuffer)
    }
    
    fn read(&self, region: Rectangle<u32>, buffer: &mut [u8]){
        let gl = &self.gl;

        // TODO: Validate the buffer
        // assert_eq!(buffer.len(), 3 * region.width as usize * region.height as usize);        
        unsafe{
            gl.read_pixels(region.x as i32, region.y as i32, region.width as i32, region.height as i32, glow::RGB, glow::UNSIGNED_BYTE, glow::PixelPackData::Slice(buffer));
        }
    }
}
