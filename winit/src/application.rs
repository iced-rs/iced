//! Create interactive, native cross-platform applications.
use crate::{
    Command, Context, Debug, Executor, Mode, Settings, Size, Subscription,
};
use iced_graphics::window;
use iced_graphics::Viewport;
use iced_native::program::Program;

/// An interactive, native cross-platform application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run). It will run in
/// its own window.
///
/// An [`Application`](trait.Application.html) can execute asynchronous actions
/// by returning a [`Command`](struct.Command.html) in some of its methods.
///
/// When using an [`Application`] with the `debug` feature enabled, a debug view
/// can be toggled by pressing `F12`.
///
/// [`Application`]: trait.Application.html
pub trait Application: Program {
    /// The data needed to initialize your [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    type Flags;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`](struct.Command.html) if you
    /// need to perform some async action in the background on startup. This is
    /// useful if you want to load state from a file, perform an initial HTTP
    /// request, etc.
    ///
    /// [`Application`]: trait.Application.html
    /// [`run`]: #method.run.html
    /// [`Settings`]: ../settings/struct.Settings.html
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    ///
    /// [`Application`]: trait.Application.html
    fn title(&self) -> String;

    /// Returns the event `Subscription` for the current state of the
    /// application.
    ///
    /// The messages produced by the `Subscription` will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// A `Subscription` will be kept alive as long as you keep returning it!
    ///
    /// By default, it returns an empty subscription.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Returns the current [`Application`] mode.
    ///
    /// The runtime will automatically transition your application if a new mode
    /// is returned.
    ///
    /// By default, an application will run in windowed mode.
    ///
    /// [`Application`]: trait.Application.html
    fn mode(&self) -> Mode {
        Mode::Windowed
    }
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
///
/// [`Application`]: trait.Application.html
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};

    let mut event_loop = EventLoop::with_user_event();
    let mut context = Context::<A, E, C, A::Message, C::SwapChain>::new(
        &mut event_loop,
        settings,
        compositor_settings,
    );

    event_loop.run(
        move |event: winit::event::Event<'_, A::Message>,
              _: &EventLoopWindowTarget<A::Message>,
              control_flow: &mut ControlFlow| {
            context.handle_winit_event(event, control_flow)
        },
    )
}

/// Handles a `WindowEvent` and mutates the provided control flow, keyboard
/// modifiers, viewport, and resized flag accordingly.
pub fn handle_window_event(
    event: &winit::event::WindowEvent<'_>,
    window: &winit::window::Window,
    control_flow: &mut winit::event_loop::ControlFlow,
    modifiers: &mut winit::event::ModifiersState,
    viewport: &mut Viewport,
    resized: &mut bool,
    _debug: &mut Debug,
) {
    use winit::{event::WindowEvent, event_loop::ControlFlow};

    match event {
        WindowEvent::Resized(new_size) => {
            let size = Size::new(new_size.width, new_size.height);

            *viewport =
                Viewport::with_physical_size(size, window.scale_factor());
            *resized = true;
        }
        WindowEvent::CloseRequested => {
            *control_flow = ControlFlow::Exit;
        }
        WindowEvent::ModifiersChanged(new_modifiers) => {
            *modifiers = *new_modifiers;
        }
        #[cfg(target_os = "macos")]
        WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Q),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } if modifiers.logo() => {
            *control_flow = ControlFlow::Exit;
        }
        #[cfg(feature = "debug")]
        WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(winit::event::VirtualKeyCode::F12),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } => _debug.toggle(),
        _ => {}
    }
}
