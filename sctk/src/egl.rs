use std::num::NonZeroU32;

use glutin::{
    api::egl, config::ConfigSurfaceTypes, prelude::GlDisplay,
    surface::WindowSurface,
};
use sctk::reexports::client::{protocol::wl_surface, Proxy};

/// helper for initializing egl after creation of the first layer surface / window
pub fn init_egl(
    surface: &wl_surface::WlSurface,
    width: u32,
    height: u32,
) -> (
    egl::display::Display,
    egl::context::NotCurrentContext,
    glutin::api::egl::config::Config,
    egl::surface::Surface<glutin::surface::WindowSurface>,
) {
    let mut display_handle = raw_window_handle::WaylandDisplayHandle::empty();
    display_handle.display = surface
        .backend()
        .upgrade()
        .expect("Connection has been closed")
        .display_ptr() as *mut _;
    let display_handle =
        raw_window_handle::RawDisplayHandle::Wayland(display_handle);
    let mut window_handle = raw_window_handle::WaylandWindowHandle::empty();
    window_handle.surface = surface.id().as_ptr() as *mut _;
    let window_handle =
        raw_window_handle::RawWindowHandle::Wayland(window_handle);

    // Initialize the EGL Wayland platform
    //
    // SAFETY: The connection is valid.
    let display = unsafe { egl::display::Display::new(display_handle) }
        .expect("Failed to initialize Wayland EGL platform");

    // Find a suitable config for the window.
    let config_template = glutin::config::ConfigTemplateBuilder::default()
        .compatible_with_native_window(window_handle)
        .with_surface_type(ConfigSurfaceTypes::WINDOW)
        .with_api(glutin::config::Api::GLES2)
        .build();
    let config = unsafe { display.find_configs(config_template) }
        .unwrap()
        .next()
        .expect("No available configs");
    let gl_attrs = glutin::context::ContextAttributesBuilder::default()
        .with_context_api(glutin::context::ContextApi::OpenGl(None))
        .build(Some(window_handle));
    let gles_attrs = glutin::context::ContextAttributesBuilder::default()
        .with_context_api(glutin::context::ContextApi::Gles(None))
        .build(Some(window_handle));

    let context = unsafe { display.create_context(&config, &gl_attrs) }
        .or_else(|_| unsafe { display.create_context(&config, &gles_attrs) })
        .expect("Failed to create context");

    let surface_attrs =
        glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::default()
            .build(
                window_handle,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );
    let surface =
        unsafe { display.create_window_surface(&config, &surface_attrs) }
            .expect("Failed to create surface");

    (display, context, config, surface)
}

pub fn get_surface(
    display: &egl::display::Display,
    config: &glutin::api::egl::config::Config,
    surface: &wl_surface::WlSurface,
    width: u32,
    height: u32,
) -> egl::surface::Surface<glutin::surface::WindowSurface> {
    let mut window_handle = raw_window_handle::WaylandWindowHandle::empty();
    window_handle.surface = surface.id().as_ptr() as *mut _;
    let window_handle =
        raw_window_handle::RawWindowHandle::Wayland(window_handle);
    let surface_attrs =
        glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::default()
            .build(
                window_handle,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );
    let surface =
        unsafe { display.create_window_surface(&config, &surface_attrs) }
            .expect("Failed to create surface");
    surface
}
