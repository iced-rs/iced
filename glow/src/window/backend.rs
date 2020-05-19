use crate::{Renderer, Settings, Viewport};

use glow::HasContext;
use iced_native::mouse;
use raw_window_handle::HasRawWindowHandle;

/// A window graphics backend for iced powered by `glow`.
#[allow(missing_debug_implementations)]
pub struct Backend {
    connection: surfman::Connection,
    device: surfman::Device,
    gl_context: surfman::Context,
    gl: Option<glow::Context>,
}

impl iced_native::window::Backend for Backend {
    type Settings = Settings;
    type Renderer = Renderer;
    type Surface = ();
    type SwapChain = Viewport;

    fn new(settings: Self::Settings) -> Backend {
        let connection = surfman::Connection::new().expect("Create connection");

        let adapter = connection
            .create_hardware_adapter()
            .expect("Create adapter");

        let mut device =
            connection.create_device(&adapter).expect("Create device");

        let context_descriptor = device
            .create_context_descriptor(&surfman::ContextAttributes {
                version: surfman::GLVersion::new(3, 0),
                flags: surfman::ContextAttributeFlags::empty(),
            })
            .expect("Create context descriptor");

        let gl_context = device
            .create_context(&context_descriptor)
            .expect("Create context");

        Backend {
            connection,
            device,
            gl_context,
            gl: None,
        }
    }

    fn create_renderer(&mut self, settings: Settings) -> Renderer {
        self.device
            .make_context_current(&self.gl_context)
            .expect("Make context current");

        Renderer::new(self.gl.as_ref().unwrap(), settings)
    }

    fn create_surface<W: HasRawWindowHandle>(
        &mut self,
        window: &W,
    ) -> Self::Surface {
        let native_widget = self
            .connection
            .create_native_widget_from_rwh(window.raw_window_handle())
            .expect("Create widget");

        let surface = self
            .device
            .create_surface(
                &self.gl_context,
                surfman::SurfaceAccess::GPUOnly,
                surfman::SurfaceType::Widget { native_widget },
            )
            .expect("Create surface");

        let surfman::SurfaceInfo { .. } = self.device.surface_info(&surface);

        self.device
            .bind_surface_to_context(&mut self.gl_context, surface)
            .expect("Bind surface to context");

        self.device
            .make_context_current(&self.gl_context)
            .expect("Make context current");

        self.gl = Some(glow::Context::from_loader_function(|s| {
            self.device.get_proc_address(&self.gl_context, s)
        }));

        //let mut framebuffer =
        //    skia_safe::gpu::gl::FramebufferInfo::from_fboid(framebuffer_object);

        //framebuffer.format = gl::RGBA8;

        //framebuffer
    }

    fn create_swap_chain(
        &mut self,
        _surface: &Self::Surface,
        width: u32,
        height: u32,
    ) -> Self::SwapChain {
        let mut surface = self
            .device
            .unbind_surface_from_context(&mut self.gl_context)
            .expect("Unbind surface")
            .expect("Active surface");

        self.device
            .resize_surface(
                &self.gl_context,
                &mut surface,
                euclid::Size2D::new(width as i32, height as i32),
            )
            .expect("Resize surface");

        self.device
            .bind_surface_to_context(&mut self.gl_context, surface)
            .expect("Bind surface to context");

        let gl = self.gl.as_ref().unwrap();

        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
            gl.clear_color(1.0, 1.0, 1.0, 1.0);

            // Enable auto-conversion from/to sRGB
            gl.enable(glow::FRAMEBUFFER_SRGB);

            // Enable alpha blending
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        Viewport::new(width, height)
    }

    fn draw<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        swap_chain: &mut Self::SwapChain,
        output: &<Self::Renderer as iced_native::Renderer>::Output,
        scale_factor: f64,
        overlay: &[T],
    ) -> mouse::Interaction {
        let gl = self.gl.as_ref().unwrap();

        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        let mouse =
            renderer.draw(gl, swap_chain, output, scale_factor, overlay);

        {
            let mut surface = self
                .device
                .unbind_surface_from_context(&mut self.gl_context)
                .expect("Unbind surface")
                .expect("Active surface");

            self.device
                .present_surface(&self.gl_context, &mut surface)
                .expect("Present surface");

            self.device
                .bind_surface_to_context(&mut self.gl_context, surface)
                .expect("Bind surface to context");
        }

        mouse
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        self.device
            .destroy_context(&mut self.gl_context)
            .expect("Destroy context");
    }
}
