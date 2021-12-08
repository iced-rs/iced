//! Configure winit windows
use crate::winit::window::WindowBuilder;
use std::fmt::Debug;

/// Allows to perform any custom settings on the winit::WindowBuilder
///
/// Is called after all other window settings have been applied
pub trait WindowConfigurator<A>: Debug {
    /// Apply custom settings on the window_builder
    fn configure_builder(
        self,
        available_monitors: &winit::event_loop::EventLoopWindowTarget<A>,
        window_builder: WindowBuilder,
    ) -> WindowBuilder;
}

/// A WindowConfigurator that does nothing
#[derive(Debug)]
pub struct NoopWindowConfigurator;

impl<A> WindowConfigurator<A> for NoopWindowConfigurator {
    fn configure_builder(
        self,
        _available_monitors: &winit::event_loop::EventLoopWindowTarget<A>,
        window_builder: WindowBuilder,
    ) -> WindowBuilder {
        window_builder
    }
}
