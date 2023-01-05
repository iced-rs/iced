//! Create interactive, native cross-platform applications.
use crate::mouse;
use crate::{Error, Executor, Runtime};

pub use iced_winit::application::StyleSheet;
pub use iced_winit::Application;

use iced_graphics::window;
use iced_winit::application;
use iced_winit::conversion;
use iced_winit::futures;
use iced_winit::futures::channel::mpsc;
use iced_winit::renderer;
use iced_winit::user_interface;
use iced_winit::winit;
use iced_winit::{Clipboard, Command, Debug, Proxy, Settings};

use glutin::config::{
    Config, ConfigSurfaceTypes, ConfigTemplateBuilder, GlConfig,
};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext,
    NotCurrentGlContextSurfaceAccessor,
    PossiblyCurrentContextGlSurfaceAccessor, PossiblyCurrentGlContext,
};
use glutin::display::{Display, DisplayApiPreference, GlDisplay};
use glutin::surface::{
    GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::num::NonZeroU32;

#[cfg(feature = "tracing")]
use tracing::{info_span, instrument::Instrument};

#[allow(unsafe_code)]
const ONE: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1) };

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::GLCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use futures::task;
    use futures::Future;
    use winit::event_loop::EventLoopBuilder;
    use winit::platform::run_return::EventLoopExtRunReturn;

    #[cfg(feature = "trace")]
    let _guard = iced_winit::Profiler::init();

    let mut debug = Debug::new();
    debug.startup_started();

    #[cfg(feature = "tracing")]
    let _ = info_span!("Application::Glutin", "RUN").entered();

    let mut event_loop = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let runtime = {
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;
        let proxy = Proxy::new(event_loop.create_proxy());

        Runtime::new(executor, proxy)
    };

    let (application, init_command) = {
        let flags = settings.flags;

        runtime.enter(|| A::new(flags))
    };

    let builder = settings.window.into_builder(
        &application.title(),
        event_loop.primary_monitor(),
        settings.id,
    );

    log::info!("Window builder: {:#?}", builder);

    #[allow(unsafe_code)]
    let (display, window, surface, context) = unsafe {
        struct Configuration(Config);
        use std::fmt;
        impl fmt::Debug for Configuration {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let config = &self.0;

                f.debug_struct("Configuration")
                    .field("raw", &config)
                    .field("samples", &config.num_samples())
                    .field("buffer_type", &config.color_buffer_type())
                    .field("surface_type", &config.config_surface_types())
                    .field("depth", &config.depth_size())
                    .field("alpha", &config.alpha_size())
                    .field("stencil", &config.stencil_size())
                    .field("float_pixels", &config.float_pixels())
                    .field("srgb", &config.srgb_capable())
                    .field("api", &config.api())
                    .finish()
            }
        }

        impl AsRef<Config> for Configuration {
            fn as_ref(&self) -> &Config {
                &self.0
            }
        }

        let display_handle = event_loop.raw_display_handle();

        #[cfg(all(
            any(windows, target_os = "macos"),
            not(target_arch = "wasm32")
        ))]
        let (window, window_handle) = {
            let window = builder
                .build(&event_loop)
                .map_err(Error::WindowCreationFailed)?;

            let handle = window.raw_window_handle();

            (window, handle)
        };

        #[cfg(target_arch = "wasm32")]
        let preference = Err(Error::GraphicsCreationFailed(
            iced_graphics::Error::BackendError(format!(
                "target not supported by backend"
            )),
        ))?;

        #[cfg(all(windows, not(target_arch = "wasm32")))]
        let preference = DisplayApiPreference::WglThenEgl(Some(window_handle));

        #[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
        let preference = DisplayApiPreference::Cgl;

        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_arch = "wasm32")
        ))]
        let preference = DisplayApiPreference::GlxThenEgl(Box::new(
            winit::platform::unix::register_xlib_error_hook,
        ));

        let display =
            Display::new(display_handle, preference).map_err(|error| {
                Error::GraphicsCreationFailed(
                    iced_graphics::Error::BackendError(format!(
                        "failed to create display: {error}"
                    )),
                )
            })?;

        log::debug!("Display: {}", display.version_string());

        let samples = C::sample_count(&compositor_settings) as u8;
        let mut template = ConfigTemplateBuilder::new()
            .with_surface_type(ConfigSurfaceTypes::WINDOW);

        if samples != 0 {
            template = template.with_multisampling(samples);
        }

        #[cfg(all(windows, not(target_arch = "wasm32")))]
        let template = template.compatible_with_native_window(window_handle);

        log::debug!("Searching for display configurations");
        let configuration = display
            .find_configs(template.build())
            .map_err(|_| {
                Error::GraphicsCreationFailed(
                    iced_graphics::Error::NoAvailablePixelFormat,
                )
            })?
            .map(Configuration)
            .inspect(|config| {
                log::trace!("{config:#?}");
            })
            .min_by_key(|config| {
                config.as_ref().num_samples().saturating_sub(samples)
            })
            .ok_or(Error::GraphicsCreationFailed(
                iced_graphics::Error::NoAvailablePixelFormat,
            ))?;

        log::debug!("Selected: {configuration:#?}");

        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_arch = "wasm32")
        ))]
        let (window, window_handle) = {
            use glutin::platform::x11::X11GlConfigExt;
            let builder =
                if let Some(visual) = configuration.as_ref().x11_visual() {
                    use winit::platform::unix::WindowBuilderExtUnix;
                    builder.with_x11_visual(visual.into_raw())
                } else {
                    builder
                };

            let window = builder
                .build(&event_loop)
                .map_err(Error::WindowCreationFailed)?;

            let handle = window.raw_window_handle();

            (window, handle)
        };

        let attributes =
            ContextAttributesBuilder::new().build(Some(window_handle));
        let fallback_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(window_handle));

        let context = display
            .create_context(configuration.as_ref(), &attributes)
            .or_else(|_| {
                display.create_context(
                    configuration.as_ref(),
                    &fallback_attributes,
                )
            })
            .map_err(|error| {
                Error::GraphicsCreationFailed(
                    iced_graphics::Error::BackendError(format!(
                        "failed to create context: {error}"
                    )),
                )
            })?;

        let surface = gl_surface(&display, configuration.as_ref(), &window)
            .map_err(|error| {
                Error::GraphicsCreationFailed(
                    iced_graphics::Error::BackendError(format!(
                        "failed to create surface: {error}"
                    )),
                )
            })?;

        let context = {
            context
                .make_current(&surface)
                .expect("make context current")
        };

        if let Err(error) = surface.set_swap_interval(
            &context,
            glutin::surface::SwapInterval::Wait(ONE),
        ) {
            log::error!("set swap interval failed: {}", error);
        }

        (display, window, surface, context)
    };

    #[allow(unsafe_code)]
    let (compositor, renderer) = unsafe {
        C::new(compositor_settings, |address| {
            let address = CString::new(address).expect("address error");
            display.get_proc_address(address.as_c_str())
        })?
    };

    let context = { context.make_not_current().expect("make context current") };

    let (mut sender, receiver) = mpsc::unbounded();

    let mut instance = Box::pin({
        let run_instance = run_instance::<A, E, C>(
            application,
            compositor,
            renderer,
            runtime,
            proxy,
            debug,
            receiver,
            window,
            surface,
            context,
            init_command,
            settings.exit_on_close_request,
        );

        #[cfg(feature = "tracing")]
        let run_instance =
            run_instance.instrument(info_span!("Application", "LOOP"));

        run_instance
    });

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    let _ = event_loop.run_return(move |event, _, control_flow| {
        use winit::event_loop::ControlFlow;

        if let ControlFlow::ExitWithCode(_) = control_flow {
            return;
        }

        let event = match event {
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        ..
                    },
                window_id,
            } => Some(winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(*new_inner_size),
                window_id,
            }),
            _ => event.to_static(),
        };

        if let Some(event) = event {
            sender.start_send(event).expect("Send event");

            let poll = instance.as_mut().poll(&mut context);

            *control_flow = match poll {
                task::Poll::Pending => ControlFlow::Wait,
                task::Poll::Ready(_) => ControlFlow::Exit,
            };
        }
    });

    Ok(())
}

async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut proxy: winit::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<winit::event::Event<'_, A::Message>>,
    window: winit::window::Window,
    surface: Surface<WindowSurface>,
    context: NotCurrentContext,
    init_command: Command<A::Message>,
    exit_on_close_request: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::GLCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use iced_winit::futures::stream::StreamExt;
    use winit::event;

    let context = {
        context
            .make_current(&surface)
            .expect("make context current")
    };

    let mut clipboard = Clipboard::connect(&window);
    let mut cache = user_interface::Cache::default();
    let mut state = application::State::new(&application, &window);
    let mut viewport_version = state.viewport_version();
    let mut should_exit = false;

    application::run_command(
        &application,
        &mut cache,
        &state,
        &mut renderer,
        init_command,
        &mut runtime,
        &mut clipboard,
        &mut should_exit,
        &mut proxy,
        &mut debug,
        &window,
        || compositor.fetch_information(),
    );
    runtime.track(application.subscription());

    let mut user_interface =
        ManuallyDrop::new(application::build_user_interface(
            &application,
            user_interface::Cache::default(),
            &mut renderer,
            state.logical_size(),
            &mut debug,
        ));

    let mut mouse_interaction = mouse::Interaction::default();
    let mut events = Vec::new();
    let mut messages = Vec::new();

    debug.startup_finished();

    while let Some(event) = receiver.next().await {
        match event {
            event::Event::MainEventsCleared => {
                if events.is_empty() && messages.is_empty() {
                    continue;
                }

                debug.event_processing_started();

                let (interface_state, statuses) = user_interface.update(
                    &events,
                    state.cursor_position(),
                    &mut renderer,
                    &mut clipboard,
                    &mut messages,
                );

                debug.event_processing_finished();

                for event in events.drain(..).zip(statuses.into_iter()) {
                    runtime.broadcast(event);
                }

                if !messages.is_empty()
                    || matches!(
                        interface_state,
                        user_interface::State::Outdated
                    )
                {
                    let mut cache =
                        ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    application::update(
                        &mut application,
                        &mut cache,
                        &state,
                        &mut renderer,
                        &mut runtime,
                        &mut clipboard,
                        &mut should_exit,
                        &mut proxy,
                        &mut debug,
                        &mut messages,
                        &window,
                        || compositor.fetch_information(),
                    );

                    // Update window
                    state.synchronize(&application, &window);

                    user_interface =
                        ManuallyDrop::new(application::build_user_interface(
                            &application,
                            cache,
                            &mut renderer,
                            state.logical_size(),
                            &mut debug,
                        ));

                    if should_exit {
                        break;
                    }
                }

                debug.draw_started();
                let new_mouse_interaction = user_interface.draw(
                    &mut renderer,
                    state.theme(),
                    &renderer::Style {
                        text_color: state.text_color(),
                    },
                    state.cursor_position(),
                );
                debug.draw_finished();

                if new_mouse_interaction != mouse_interaction {
                    window.set_cursor_icon(conversion::mouse_interaction(
                        new_mouse_interaction,
                    ));

                    mouse_interaction = new_mouse_interaction;
                }

                window.request_redraw();
            }
            event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
                event::MacOS::ReceivedUrl(url),
            )) => {
                use iced_native::event;
                events.push(iced_native::Event::PlatformSpecific(
                    event::PlatformSpecific::MacOS(event::MacOS::ReceivedUrl(
                        url,
                    )),
                ));
            }
            event::Event::UserEvent(message) => {
                messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                #[cfg(feature = "tracing")]
                let _ = info_span!("Application", "FRAME").entered();

                debug.render_started();

                if !context.is_current() {
                    context
                        .make_current(&surface)
                        .expect("Make OpenGL context current");
                }

                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let physical_size = state.physical_size();
                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    let new_mouse_interaction = user_interface.draw(
                        &mut renderer,
                        state.theme(),
                        &renderer::Style {
                            text_color: state.text_color(),
                        },
                        state.cursor_position(),
                    );
                    debug.draw_finished();

                    if new_mouse_interaction != mouse_interaction {
                        window.set_cursor_icon(conversion::mouse_interaction(
                            new_mouse_interaction,
                        ));

                        mouse_interaction = new_mouse_interaction;
                    }

                    surface.resize(
                        &context,
                        NonZeroU32::new(physical_size.width).unwrap_or(ONE),
                        NonZeroU32::new(physical_size.height).unwrap_or(ONE),
                    );

                    compositor.resize_viewport(physical_size);

                    viewport_version = current_viewport_version;
                }

                compositor.present(
                    &mut renderer,
                    state.viewport(),
                    state.background_color(),
                    &debug.overlay(),
                );

                surface.swap_buffers(&context).expect("Swap buffers");

                debug.render_finished();

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
            }
            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
                if application::requests_exit(&window_event, state.modifiers())
                    && exit_on_close_request
                {
                    break;
                }

                state.update(&window, &window_event, &mut debug);

                if let Some(event) = conversion::window_event(
                    crate::window::Id::MAIN,
                    &window_event,
                    state.scale_factor(),
                    state.modifiers(),
                ) {
                    events.push(event);
                }
            }
            _ => {}
        }
    }

    // Manually drop the user interface
    drop(ManuallyDrop::into_inner(user_interface));
}

#[allow(unsafe_code)]
/// Creates a new [`glutin::Surface<WindowSurface>`].
pub fn gl_surface(
    display: &Display,
    gl_config: &Config,
    window: &winit::window::Window,
) -> Result<Surface<WindowSurface>, glutin::error::Error> {
    let (width, height) = window.inner_size().into();

    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(Some(true))
        .build(
            window.raw_window_handle(),
            NonZeroU32::new(width).unwrap_or(ONE),
            NonZeroU32::new(height).unwrap_or(ONE),
        );

    unsafe { display.create_window_surface(gl_config, &surface_attributes) }
}
