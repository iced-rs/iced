//! Create interactive, native cross-platform applications.
use crate::{mouse, Error, Executor, Runtime, Size};
use iced_graphics::window;
use iced_graphics::Viewport;
use iced_winit::application;
use iced_winit::conversion;
use iced_winit::{Clipboard, Debug, Proxy, Settings};

pub use iced_winit::Application;
pub use iced_winit::{program, Program};

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
///
/// [`Application`]: trait.Application.html
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::GLCompositor<Renderer = A::Renderer> + 'static,
{
    use glutin::{
        event,
        event_loop::{ControlFlow, EventLoop},
        ContextBuilder,
    };

    let mut debug = Debug::new();
    debug.startup_started();

    let event_loop = EventLoop::with_user_event();
    let mut runtime = {
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;
        let proxy = Proxy::new(event_loop.create_proxy());

        Runtime::new(executor, proxy)
    };

    let flags = settings.flags;
    let (application, init_command) = runtime.enter(|| A::new(flags));
    runtime.spawn(init_command);

    let subscription = application.subscription();
    runtime.track(subscription);

    let mut title = application.title();
    let mut mode = application.mode();
    let mut background_color = application.background_color();
    let mut scale_factor = application.scale_factor();

    let context = {
        let builder = settings.window.into_builder(
            &title,
            mode,
            event_loop.primary_monitor(),
        );

        let context = ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(C::sample_count(&compositor_settings) as u16)
            .build_windowed(builder, &event_loop)
            .map_err(|error| {
                use glutin::CreationError;

                match error {
                    CreationError::Window(error) => {
                        Error::WindowCreationFailed(error)
                    }
                    _ => Error::GraphicsAdapterNotFound,
                }
            })?;

        #[allow(unsafe_code)]
        unsafe {
            context.make_current().expect("Make OpenGL context current")
        }
    };

    let clipboard = Clipboard::new(&context.window());
    let mut cursor_position = glutin::dpi::PhysicalPosition::new(-1.0, -1.0);
    let mut mouse_interaction = mouse::Interaction::default();
    let mut modifiers = glutin::event::ModifiersState::default();

    let physical_size = context.window().inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        context.window().scale_factor() * scale_factor,
    );
    let mut resized = false;

    #[allow(unsafe_code)]
    let (mut compositor, mut renderer) = unsafe {
        C::new(compositor_settings, |address| {
            context.get_proc_address(address)
        })?
    };

    let mut state = program::State::new(
        application,
        viewport.logical_size(),
        conversion::cursor_position(cursor_position, viewport.scale_factor()),
        &mut renderer,
        &mut debug,
    );
    debug.startup_finished();

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::MainEventsCleared => {
            if state.is_queue_empty() {
                return;
            }

            let command = runtime.enter(|| {
                state.update(
                    viewport.logical_size(),
                    conversion::cursor_position(
                        cursor_position,
                        viewport.scale_factor(),
                    ),
                    clipboard.as_ref().map(|c| c as _),
                    &mut renderer,
                    &mut debug,
                )
            });

            // If the application was updated
            if let Some(command) = command {
                runtime.spawn(command);

                let program = state.program();

                // Update subscriptions
                let subscription = program.subscription();
                runtime.track(subscription);

                // Update window title
                let new_title = program.title();

                if title != new_title {
                    context.window().set_title(&new_title);

                    title = new_title;
                }

                // Update window mode
                let new_mode = program.mode();

                if mode != new_mode {
                    context.window().set_fullscreen(conversion::fullscreen(
                        context.window().current_monitor(),
                        new_mode,
                    ));

                    mode = new_mode;
                }

                // Update background color
                background_color = program.background_color();

                // Update scale factor
                let new_scale_factor = program.scale_factor();

                if scale_factor != new_scale_factor {
                    let size = context.window().inner_size();

                    viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        context.window().scale_factor() * new_scale_factor,
                    );

                    // We relayout the UI with the new logical size.
                    // The queue is empty, therefore this will never produce
                    // a `Command`.
                    //
                    // TODO: Properly queue `WindowResized`
                    let _ = state.update(
                        viewport.logical_size(),
                        conversion::cursor_position(
                            cursor_position,
                            viewport.scale_factor(),
                        ),
                        clipboard.as_ref().map(|c| c as _),
                        &mut renderer,
                        &mut debug,
                    );

                    scale_factor = new_scale_factor;
                }
            }

            context.window().request_redraw();
        }
        event::Event::UserEvent(message) => {
            state.queue_message(message);
        }
        event::Event::RedrawRequested(_) => {
            debug.render_started();

            if resized {
                let physical_size = viewport.physical_size();

                context.resize(glutin::dpi::PhysicalSize::new(
                    physical_size.width,
                    physical_size.height,
                ));

                compositor.resize_viewport(physical_size);

                resized = false;
            }

            let new_mouse_interaction = compositor.draw(
                &mut renderer,
                &viewport,
                background_color,
                state.primitive(),
                &debug.overlay(),
            );

            context.swap_buffers().expect("Swap buffers");

            debug.render_finished();

            if new_mouse_interaction != mouse_interaction {
                context.window().set_cursor_icon(
                    conversion::mouse_interaction(new_mouse_interaction),
                );

                mouse_interaction = new_mouse_interaction;
            }

            // TODO: Handle animations!
            // Maybe we can use `ControlFlow::WaitUntil` for this.
        }
        event::Event::WindowEvent {
            event: window_event,
            ..
        } => {
            application::handle_window_event(
                &window_event,
                context.window(),
                scale_factor,
                control_flow,
                &mut cursor_position,
                &mut modifiers,
                &mut viewport,
                &mut resized,
                &mut debug,
            );

            if let Some(event) = conversion::window_event(
                &window_event,
                viewport.scale_factor(),
                modifiers,
            ) {
                state.queue_event(event.clone());
                runtime.broadcast(event);
            }
        }
        _ => {
            *control_flow = ControlFlow::Wait;
        }
    })
}
