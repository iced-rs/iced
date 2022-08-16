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
use iced_winit::{Clipboard, Command, Debug, Proxy, Settings};

use glutin::window::Window;
use std::mem::ManuallyDrop;

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
    use glutin::event_loop::EventLoop;
    use glutin::platform::run_return::EventLoopExtRunReturn;
    use glutin::ContextBuilder;

    let mut debug = Debug::new();
    debug.startup_started();

    let mut event_loop = EventLoop::with_user_event();
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

    let context = {
        let builder = settings.window.into_builder(
            &application.title(),
            application.mode(),
            event_loop.primary_monitor(),
            settings.id,
        );

        log::info!("Window builder: {:#?}", builder);

        let opengl_builder = ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(C::sample_count(&compositor_settings) as u16);

        let opengles_builder = opengl_builder.clone().with_gl(
            glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0)),
        );

        let (first_builder, second_builder) = if settings.try_opengles_first {
            (opengles_builder, opengl_builder)
        } else {
            (opengl_builder, opengles_builder)
        };

        log::info!("Trying first builder: {:#?}", first_builder);

        let context = first_builder
            .build_windowed(builder.clone(), &event_loop)
            .or_else(|_| {
                log::info!("Trying second builder: {:#?}", second_builder);
                second_builder.build_windowed(builder, &event_loop)
            })
            .map_err(|error| {
                use glutin::CreationError;
                use iced_graphics::Error as ContextError;

                match error {
                    CreationError::Window(error) => {
                        Error::WindowCreationFailed(error)
                    }
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
                    error => Error::GraphicsCreationFailed(
                        ContextError::BackendError(error.to_string()),
                    ),
                }
            })?;

        #[allow(unsafe_code)]
        unsafe {
            context.make_current().expect("Make OpenGL context current")
        }
    };

    #[allow(unsafe_code)]
    let (compositor, renderer) = unsafe {
        C::new(compositor_settings, |address| {
            context.get_proc_address(address)
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
        context,
        init_command,
        settings.exit_on_close_request,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    event_loop.run_return(move |event, _, control_flow| {
        use glutin::event_loop::ControlFlow;

        if let ControlFlow::Exit = control_flow {
            return;
        }

        let event = match event {
            glutin::event::Event::WindowEvent {
                event:
                    glutin::event::WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        ..
                    },
                window_id,
            } => Some(glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::Resized(*new_inner_size),
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
    mut proxy: glutin::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<glutin::event::Event<'_, A::Message>>,
    mut context: glutin::ContextWrapper<glutin::PossiblyCurrent, Window>,
    init_command: Command<A::Message>,
    exit_on_close_request: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::GLCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use glutin::event;
    use iced_winit::futures::stream::StreamExt;

    let mut clipboard = Clipboard::connect(context.window());
    let mut cache = user_interface::Cache::default();
    let mut state = application::State::new(&application, context.window());
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
        context.window(),
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
                        context.window(),
                        || compositor.fetch_information(),
                    );

                    // Update window
                    state.synchronize(&application, context.window());

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
                    context.window().set_cursor_icon(
                        conversion::mouse_interaction(new_mouse_interaction),
                    );

                    mouse_interaction = new_mouse_interaction;
                }

                context.window().request_redraw();
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
                unsafe {
                    if !context.is_current() {
                        context = context
                            .make_current()
                            .expect("Make OpenGL context current");
                    }
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
                        context.window().set_cursor_icon(
                            conversion::mouse_interaction(
                                new_mouse_interaction,
                            ),
                        );

                        mouse_interaction = new_mouse_interaction;
                    }

                    context.resize(glutin::dpi::PhysicalSize::new(
                        physical_size.width,
                        physical_size.height,
                    ));

                    compositor.resize_viewport(physical_size);

                    viewport_version = current_viewport_version;
                }

                compositor.present(
                    &mut renderer,
                    state.viewport(),
                    state.background_color(),
                    &debug.overlay(),
                );

                context.swap_buffers().expect("Swap buffers");

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

                state.update(context.window(), &window_event, &mut debug);

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
