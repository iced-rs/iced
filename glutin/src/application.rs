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

use glutin::prelude::*;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

use std::mem::ManuallyDrop;
use std::num::NonZeroU32;

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

    let mut debug = Debug::new();
    debug.startup_started();

    let mut event_loop = EventLoopBuilder::with_user_event().build();
    let window = winit::window::WindowBuilder::new()
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();
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

    let (gl_display, context, surface) = {
        let builder = settings.window.into_builder(
            &application.title(),
            event_loop.primary_monitor(),
            settings.id,
        );

        log::info!("Window builder: {:#?}", builder);

        let api_preference = glutin::display::DisplayApiPreference::Egl; // XXX No good way to use whatever is perferred per platform?
        #[allow(unsafe_code)]
        let gl_display = unsafe {
            glutin::display::Display::new(
                window.raw_display_handle(),
                api_preference,
            )
            .unwrap()
        }; // XXX unwrap? vsync?
        let mut template_builder = glutin::config::ConfigTemplateBuilder::new()
            .compatible_with_native_window(window.raw_window_handle())
            .with_transparency(true);
        let sample_count = C::sample_count(&compositor_settings) as u8;
        if sample_count != 0 {
            template_builder =
                template_builder.with_multisampling(sample_count);
        }
        let template = template_builder.build();
        #[allow(unsafe_code)]
        let config = unsafe { gl_display.find_configs(template) }
            .unwrap()
            .next()
            .unwrap(); // XXX unwrap; first config?
        let physical_size = window.inner_size();
        let physical_width = NonZeroU32::new(physical_size.width)
            .unwrap_or(NonZeroU32::new(1).unwrap());
        let physical_height = NonZeroU32::new(physical_size.height)
            .unwrap_or(NonZeroU32::new(1).unwrap());
        let surface_attributes = glutin::surface::SurfaceAttributesBuilder::<
            glutin::surface::WindowSurface,
        >::new()
        .build(window.raw_window_handle(), physical_width, physical_height);
        #[allow(unsafe_code)]
        let surface = unsafe {
            gl_display
                .create_window_surface(&config, &surface_attributes)
                .unwrap()
        };
        let opengl_attributes =
            glutin::context::ContextAttributesBuilder::new()
                .build(Some(window.raw_window_handle()));
        let opengles_attributes =
            glutin::context::ContextAttributesBuilder::new()
                .with_context_api(glutin::context::ContextApi::Gles(Some(
                    glutin::context::Version { major: 2, minor: 0 },
                )))
                .build(Some(window.raw_window_handle()));

        let (first_attributes, second_attributes) =
            if settings.try_opengles_first {
                (opengles_attributes, opengl_attributes)
            } else {
                (opengl_attributes, opengles_attributes)
            };

        log::info!("Trying first attributes: {:#?}", first_attributes);

        #[allow(unsafe_code)]
        let context =
            unsafe { gl_display.create_context(&config, &first_attributes) }
                .or_else(|_| {
                    log::info!(
                        "Trying second attributes: {:#?}",
                        second_attributes
                    );
                    unsafe {
                        gl_display.create_context(&config, &second_attributes)
                    }
                })
                .map_err(|error| {
                    use glutin::error::ErrorKind as GlutinErrorKind;
                    use iced_graphics::Error as ContextError;

                    match error.error_kind() {
                        // XXX return on winit error
                        //CreationError::Window(error) => {
                        //    Error::WindowCreationFailed(error)
                        //}
                        /*
                        CreationError::OpenGlVersionNotSupported => {
                            Error::GraphicsCreationFailed(
                                ContextError::VersionNotSupported,
                            )
                        }
                        CreationError::NoAvailablePixelFormat => {
                            Error::GraphicsCreationFailed(
                                ContextError::NoAvailablePixelFormat,
                            )
                        }
                        */
                        error => Error::GraphicsCreationFailed(
                            ContextError::BackendError(error.to_string()),
                        ),
                    }
                })?;

        #[allow(unsafe_code)]
        (
            gl_display,
            context
                .make_current(&surface)
                .expect("Make OpenGL context current"),
            surface,
        )
    };

    #[allow(unsafe_code)]
    let (compositor, renderer) = unsafe {
        C::new(compositor_settings, |address| {
            gl_display
                .get_proc_address(&std::ffi::CString::new(address).unwrap())
        })?
    };

    let (mut sender, receiver) = mpsc::unbounded();

    let mut instance = Box::pin(run_instance::<A, E, C>(
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
    ));

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
    window: Window,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
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

    let mut clipboard = Clipboard::connect(&window);
    let mut cache = user_interface::Cache::default();
    let mut state = application::State::new(&application, &window);
    let mut viewport_version = state.viewport_version();

    application::run_command(
        &application,
        &mut cache,
        &state,
        &mut renderer,
        init_command,
        &mut runtime,
        &mut clipboard,
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
                        &mut proxy,
                        &mut debug,
                        &mut messages,
                        &window,
                        || compositor.fetch_information(),
                    );

                    // Update window
                    state.synchronize(&application, &window);

                    let should_exit = application.should_exit();

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
                debug.render_started();

                #[allow(unsafe_code)]
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
                        NonZeroU32::new(physical_size.width)
                            .unwrap_or(NonZeroU32::new(1).unwrap()),
                        NonZeroU32::new(physical_size.height)
                            .unwrap_or(NonZeroU32::new(1).unwrap()),
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
