use glutin::{
    context::{NotCurrentGlContext, PossiblyCurrentContextGlSurfaceAccessor},
    display::GlDisplay,
    surface::GlSurface,
};
use iced_graphics::{
    compositor::{Information, SurfaceError},
    window, Color, Error, Size, Viewport,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::{ffi::CString, num::NonZeroU32};

pub use iced_winit::Application;

/// Settings for the [`Compositor`]
#[derive(Debug, Default)]
pub struct Settings<T: Default> {
    /// Settings of the underlying [`window::GLCompositor`].
    pub gl_settings: T,
    /// Try to build the context using OpenGL ES first then OpenGL.
    pub try_opengles_first: bool,
}

/// Wraps a [`window::GLCompositor`] with a [`window::Compositor`] that uses Glutin for OpenGL
/// context creation.
#[derive(Debug)]
pub struct Compositor<C: window::GLCompositor> {
    gl_compositor: C,
    config: glutin::config::Config,
    display: glutin::display::Display,
    context: glutin::context::PossiblyCurrentContext,
}

impl<C: window::GLCompositor> window::Compositor for Compositor<C> {
    type Settings = Settings<C::Settings>;
    type Renderer = C::Renderer;
    type Surface = glutin::surface::Surface<glutin::surface::WindowSurface>;

    fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        settings: Self::Settings,
        compatible_window: Option<&W>,
    ) -> Result<(Self, Self::Renderer), Error> {
        let compatible_window = compatible_window.unwrap(); // XXX None?

        let display = create_display(&compatible_window).map_err(glutin_err)?;

        // XXX Is a different config (and context) potentially needed for
        // different windows?
        let sample_count = C::sample_count(&settings.gl_settings) as u8;
        let config = get_config(&display, compatible_window, sample_count)
            .map_err(glutin_err)?;
        let context = create_context(
            &display,
            compatible_window,
            &config,
            settings.try_opengles_first,
        )
        .map_err(glutin_err)?;

        // `C::new` seems to segfault in glow without a current context
        let surface =
            create_surface(&display, compatible_window, &config).unwrap();
        context
            .make_current(&surface)
            .expect("Make OpenGL context current");

        #[allow(unsafe_code)]
        let (gl_compositor, renderer) = unsafe {
            C::new(settings.gl_settings, |address| {
                display.get_proc_address(&CString::new(address).unwrap())
            })
        }?;

        Ok((
            Self {
                gl_compositor,
                config,
                display,
                context,
            },
            renderer,
        ))
    }

    fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &mut self,
        window: &W,
    ) -> Result<Self::Surface, Error> {
        let surface = create_surface(&self.display, window, &self.config)
            .map_err(glutin_err)?;

        // Enable vsync
        self.context
            .make_current(&surface)
            .expect("Make OpenGL context current");
        surface
            .set_swap_interval(
                &self.context,
                glutin::surface::SwapInterval::Wait(
                    NonZeroU32::new(1).unwrap(),
                ),
            )
            .expect("Set swap interval");

        Ok(surface)
    }

    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    ) {
        surface.resize(
            &self.context,
            NonZeroU32::new(width).unwrap_or(NonZeroU32::new(1).unwrap()),
            NonZeroU32::new(height).unwrap_or(NonZeroU32::new(1).unwrap()),
        );
        self.gl_compositor.resize_viewport(Size { width, height });
    }

    fn fetch_information(&self) -> Information {
        self.gl_compositor.fetch_information()
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), SurfaceError> {
        self.context
            .make_current(surface)
            .expect("Make OpenGL context current");
        self.gl_compositor.present(
            renderer,
            viewport,
            background_color,
            overlay,
        );
        surface.swap_buffers(&self.context).expect("Swap buffers");
        Ok(())
    }
}

fn create_display<W: HasRawDisplayHandle + HasRawWindowHandle>(
    window: &W,
) -> Result<glutin::display::Display, glutin::error::Error> {
    #[cfg(target_os = "windows")]
    let api_preference = glutin::display::DisplayApiPreference::WglThenEgl(
        Some(window.raw_window_handle()),
    );
    #[cfg(target_os = "macos")]
    let api_preference = glutin::display::DisplayApiPreference::Cgl;
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let api_preference = glutin::display::DisplayApiPreference::EglThenGlx(
        Box::new(iced_winit::winit::platform::unix::register_xlib_error_hook),
    );

    #[allow(unsafe_code)]
    unsafe {
        glutin::display::Display::new(
            window.raw_display_handle(),
            api_preference,
        )
    }
}

fn get_config<W: HasRawWindowHandle>(
    display: &glutin::display::Display,
    window: &W,
    sample_count: u8,
) -> Result<glutin::config::Config, glutin::error::Error> {
    let mut template_builder = glutin::config::ConfigTemplateBuilder::new()
        .compatible_with_native_window(window.raw_window_handle())
        .with_transparency(true);
    if sample_count != 0 {
        template_builder = template_builder.with_multisampling(sample_count);
    }
    let template = template_builder.build();

    #[allow(unsafe_code)]
    Ok(unsafe { display.find_configs(template) }?.next().unwrap()) // XXX unwrap; first config?
}

fn create_context<W: HasRawWindowHandle>(
    display: &glutin::display::Display,
    window: &W,
    config: &glutin::config::Config,
    try_opengles_first: bool,
) -> Result<glutin::context::PossiblyCurrentContext, glutin::error::Error> {
    let opengl_attributes = glutin::context::ContextAttributesBuilder::new()
        .build(Some(window.raw_window_handle()));
    let opengles_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::Gles(Some(
            glutin::context::Version { major: 2, minor: 0 },
        )))
        .build(Some(window.raw_window_handle()));

    let (first_attributes, second_attributes) = if try_opengles_first {
        (opengles_attributes, opengl_attributes)
    } else {
        (opengl_attributes, opengles_attributes)
    };

    #[allow(unsafe_code)]
    Ok(unsafe { display.create_context(config, &first_attributes) }
        .or_else(|_| {
            log::info!("Trying second attributes: {:#?}", second_attributes);
            unsafe { display.create_context(config, &second_attributes) }
        })?
        .treat_as_possibly_current())
}

fn create_surface<W: HasRawWindowHandle>(
    display: &glutin::display::Display,
    window: &W,
    config: &glutin::config::Config,
) -> Result<
    glutin::surface::Surface<glutin::surface::WindowSurface>,
    glutin::error::Error,
> {
    let surface_attributes = glutin::surface::SurfaceAttributesBuilder::<
        glutin::surface::WindowSurface,
    >::new()
    .build(
        window.raw_window_handle(),
        NonZeroU32::new(1).unwrap(),
        NonZeroU32::new(1).unwrap(),
    );

    #[allow(unsafe_code)]
    unsafe {
        display.create_window_surface(config, &surface_attributes)
    }
}

fn glutin_err(err: glutin::error::Error) -> iced_graphics::Error {
    // TODO: match error kind? Doesn't seem to match `iced_grapihcs::Error` well
    iced_graphics::Error::BackendError(err.to_string())
}
