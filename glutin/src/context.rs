use crate::{
    mouse,
    program::{Program, State},
    Application, Executor, Mode, Runtime, Size,
};
use glutin::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    ContextBuilder, PossiblyCurrent, WindowedContext,
};
use iced_graphics::window::GLCompositor;
use iced_graphics::Viewport;
use iced_winit::conversion;
use iced_winit::{Clipboard, Debug, Proxy, Settings};

/// Information needed for `iced_glutin` to run.
#[allow(missing_debug_implementations)]
pub struct Context<A, E, C, Message>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: GLCompositor<Renderer = A::Renderer> + 'static,
    Message: std::fmt::Debug + Send + 'static,
{
    runtime: Runtime<E, Proxy<Message>, Message>,
    title: String,
    mode: Mode,
    debug: Debug,
    compositor: C,
    renderer: A::Renderer,
    gl_context: WindowedContext<PossiblyCurrent>,
    clipboard: Option<Clipboard>,
    mouse_interaction: mouse::Interaction,
    modifiers: glutin::event::ModifiersState,
    viewport: Viewport,
    resized: bool,
    state: State<A>,
}

impl<A, E, C, Message> Context<A, E, C, Message>
where
    A: Application + Program<Message = Message> + 'static,
    E: Executor + 'static,
    C: GLCompositor<Renderer = A::Renderer> + 'static,
    Message: std::fmt::Debug + Send + 'static,
{
    /// Initializes and returns an `iced::glutin` application context.
    pub fn new(
        event_loop: &mut EventLoop<A::Message>,
        settings: Settings<A::Flags>,
        compositor_settings: C::Settings,
    ) -> Self {
        let mut debug = Debug::new();
        debug.startup_started();

        let mut runtime = {
            let executor = E::new().expect("Create executor");
            let proxy = Proxy::new(event_loop.create_proxy());

            Runtime::new(executor, proxy)
        };

        let flags = settings.flags;
        let (application, init_command) = runtime.enter(|| A::new(flags));
        runtime.spawn(init_command);

        let subscription = application.subscription();
        runtime.track(subscription);

        let title = application.title();
        let mode = application.mode();

        let gl_context = {
            let builder = settings.window.into_builder(
                &title,
                mode,
                event_loop.primary_monitor(),
            );

            let gl_context =
                ContextBuilder::new()
                    .with_vsync(true)
                    .with_multisampling(
                        C::sample_count(&compositor_settings) as u16
                    )
                    .build_windowed(builder, &event_loop)
                    .expect("Open window");

            #[allow(unsafe_code)]
            unsafe {
                gl_context
                    .make_current()
                    .expect("Make OpenGL context current")
            }
        };

        let clipboard = Clipboard::new(&gl_context.window());
        let mouse_interaction = mouse::Interaction::default();
        let modifiers = glutin::event::ModifiersState::default();

        let physical_size = gl_context.window().inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            gl_context.window().scale_factor(),
        );
        let resized = false;

        #[allow(unsafe_code)]
        let (compositor, mut renderer) = unsafe {
            C::new(compositor_settings, |address| {
                gl_context.get_proc_address(address)
            })
        };

        let state = State::new(
            application,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );
        debug.startup_finished();

        Self {
            runtime,
            title,
            mode,
            debug,
            compositor,
            renderer,
            gl_context,
            clipboard,
            mouse_interaction,
            modifiers,
            viewport,
            resized,
            state,
        }
    }

    /// Manages the `iced_glutin` application based on the `winit` event.
    pub fn handle_winit_event<'e>(
        &mut self,
        event: Event<'e, A::Message>,
        control_flow: &mut ControlFlow,
    ) {
        let Context {
            runtime,
            title,
            mode,
            debug,
            compositor,
            renderer,
            gl_context,
            clipboard,
            mouse_interaction,
            modifiers,
            viewport,
            resized,
            state,
        } = self;

        match event {
            Event::MainEventsCleared => {
                let command = runtime.enter(|| {
                    state.update(
                        clipboard.as_ref().map(|c| c as _),
                        viewport.logical_size(),
                        renderer,
                        debug,
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

                    if title != &new_title {
                        gl_context.window().set_title(&new_title);

                        *title = new_title;
                    }

                    // Update window mode
                    let new_mode = program.mode();

                    if *mode != new_mode {
                        gl_context.window().set_fullscreen(
                            conversion::fullscreen(
                                gl_context.window().current_monitor(),
                                new_mode,
                            ),
                        );

                        *mode = new_mode;
                    }
                }

                gl_context.window().request_redraw();
            }
            Event::UserEvent(message) => {
                state.queue_message(message);
            }
            Event::RedrawRequested(_) => {
                debug.render_started();

                if *resized {
                    let physical_size = viewport.physical_size();

                    gl_context.resize(glutin::dpi::PhysicalSize::new(
                        physical_size.width,
                        physical_size.height,
                    ));

                    compositor.resize_viewport(physical_size);

                    *resized = false;
                }

                let new_mouse_interaction = compositor.draw(
                    renderer,
                    viewport,
                    state.primitive(),
                    &debug.overlay(),
                );

                gl_context.swap_buffers().expect("Swap buffers");

                debug.render_finished();

                if new_mouse_interaction != *mouse_interaction {
                    gl_context.window().set_cursor_icon(
                        conversion::mouse_interaction(new_mouse_interaction),
                    );

                    *mouse_interaction = new_mouse_interaction;
                }

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
            }
            Event::WindowEvent {
                event: window_event,
                ..
            } => {
                iced_winit::application::handle_window_event(
                    &window_event,
                    gl_context.window(),
                    control_flow,
                    modifiers,
                    viewport,
                    resized,
                    debug,
                );

                if let Some(event) = conversion::window_event(
                    &window_event,
                    viewport.scale_factor(),
                    *modifiers,
                ) {
                    state.queue_event(event.clone());
                    runtime.broadcast(event);
                }
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        }
    }
}
