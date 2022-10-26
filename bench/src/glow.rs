use iced_glow::glow::{self, HasContext};
use iced_glutin::glutin;

fn glutin_context(
    width: u32,
    height: u32,
) -> glutin::Context<glutin::NotCurrent> {
    let el = glutin::event_loop::EventLoop::new();
    #[cfg(target_os = "linux")]
    use glutin::platform::unix::HeadlessContextExt;
    #[cfg(target_os = "linux")]
    if let Ok(context) = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0)))
        .build_surfaceless(&el)
    {
        return context;
    }
    glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0)))
        .build_headless(&el, glutin::dpi::PhysicalSize::new(width, height))
        .unwrap()
}

pub struct GlowBench {
    _context: glutin::Context<glutin::PossiblyCurrent>,
    gl: glow::Context,
    renderer: iced_glow::Renderer<iced::Theme>,
    viewport: iced_graphics::Viewport,
    _framebuffer: glow::NativeFramebuffer,
    _renderbuffer: glow::NativeRenderbuffer,
    width: u32,
    height: u32,
}

impl GlowBench {
    pub fn new(width: u32, height: u32) -> Self {
        let context =
            unsafe { glutin_context(width, height).make_current().unwrap() };
        let (gl, framebuffer, renderbuffer);
        unsafe {
            gl = glow::Context::from_loader_function(|name| {
                context.get_proc_address(name)
            });
            gl.viewport(0, 0, width as i32, height as i32);

            renderbuffer = gl.create_renderbuffer().unwrap();
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
            gl.renderbuffer_storage(
                glow::RENDERBUFFER,
                glow::RGBA8,
                width as i32,
                height as i32,
            );
            gl.bind_renderbuffer(glow::RENDERBUFFER, None);

            framebuffer = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(renderbuffer),
            );
            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE
            );
        };
        let renderer = iced_glow::Renderer::<iced::Theme>::new(
            iced_glow::Backend::new(&gl, Default::default()),
        );
        let viewport = iced_graphics::Viewport::with_physical_size(
            iced::Size::new(width, height),
            1.0,
        );
        Self {
            _context: context,
            gl,
            renderer,
            viewport,
            _framebuffer: framebuffer,
            _renderbuffer: renderbuffer,
            width,
            height,
        }
    }
}

impl super::Bench for GlowBench {
    type Backend = iced_glow::Backend;
    type RenderState = ();
    const BACKEND_NAME: &'static str = "glow";

    fn clear(&self) {
        unsafe {
            self.gl.clear_color(1., 1., 1., 1.);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    fn present(&mut self, _state: ()) {
        self.renderer.with_primitives(|backend, primitive| {
            backend.present::<&str>(&self.gl, primitive, &self.viewport, &[]);
        });
        unsafe { self.gl.finish() };
    }

    fn read_pixels(&self) -> image_rs::RgbaImage {
        let mut pixels = image_rs::RgbaImage::new(self.width, self.height);
        unsafe {
            self.gl.read_pixels(
                0,
                0,
                self.width as i32,
                self.height as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(&mut pixels),
            );
        }
        image_rs::imageops::flip_vertical_in_place(&mut pixels);
        pixels
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn renderer(&mut self) -> &mut iced_glow::Renderer<iced::Theme> {
        &mut self.renderer
    }
}
