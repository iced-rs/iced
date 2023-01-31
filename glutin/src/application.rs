//! Create interactive, native cross-platform applications.

use iced_graphics::window;
use iced_winit::{application::StyleSheet, Executor, Settings};

pub use iced_winit::Application;

use crate::compositor;

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), iced_winit::Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::GLCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    let compositor_settings = compositor::Settings {
        gl_settings: compositor_settings,
        try_opengles_first: settings.try_opengles_first,
    };
    iced_winit::application::run::<A, E, compositor::Compositor<C>>(
        settings,
        compositor_settings,
    )
}
