//! Create interactive, native cross-platform applications.
use crate::{Context, Executor};
use glutin::event_loop::EventLoopWindowTarget;
use iced_graphics::window;
use iced_winit::Settings;

pub use iced_winit::Application;
pub use iced_winit::{program, Program};

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
    C: window::GLCompositor<Renderer = A::Renderer> + 'static,
{
    use glutin::event_loop::{ControlFlow, EventLoop};

    let mut event_loop = EventLoop::with_user_event();
    let mut context = Context::<A, E, C, A::Message>::new(
        &mut event_loop,
        settings,
        compositor_settings,
    );

    event_loop.run(
        move |event: glutin::event::Event<'_, A::Message>,
              _: &EventLoopWindowTarget<A::Message>,
              control_flow: &mut ControlFlow| {
            context.handle_winit_event(event, control_flow)
        },
    )
}
