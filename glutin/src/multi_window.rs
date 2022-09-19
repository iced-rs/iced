//! Create interactive, native cross-platform applications.
use crate::{Error, Executor};

pub use iced_winit::multi_window::{Application, StyleSheet};

use iced_winit::Settings;

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub fn run<A, E, C>(
    _settings: Settings<A::Flags>,
    _compositor_settings: C::Settings,
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: iced_graphics::window::GLCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    unimplemented!("iced_glutin not implemented!")
}
