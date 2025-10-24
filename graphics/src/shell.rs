//! Control the windowing runtime from a renderer.
use std::sync::Arc;

/// A windowing shell.
#[derive(Clone)]
pub struct Shell(Arc<dyn Notifier>);

impl Shell {
    /// Creates a new [`Shell`].
    pub fn new(notifier: impl Notifier) -> Self {
        Self(Arc::new(notifier))
    }

    /// Creates a headless [`Shell`].
    pub fn headless() -> Self {
        struct Headless;

        impl Notifier for Headless {
            fn request_redraw(&self) {}

            fn invalidate_layout(&self) {}
        }

        Self::new(Headless)
    }

    /// Requests for all windows of the [`Shell`] to be redrawn.
    pub fn request_redraw(&self) {
        self.0.request_redraw();
    }

    /// Requests for all layouts of the [`Shell`] to be recomputed.
    pub fn invalidate_layout(&self) {
        self.0.invalidate_layout();
    }
}

/// A type that can notify a shell of certain events.
pub trait Notifier: Send + Sync + 'static {
    /// Requests for all windows of the [`Shell`] to be redrawn.
    fn request_redraw(&self);

    /// Requests for all layouts of the [`Shell`] to be recomputed.
    fn invalidate_layout(&self);
}
