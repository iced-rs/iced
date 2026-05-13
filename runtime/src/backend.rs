//! Configure a [`Backend`](crate::core::Backend) at runtime.
use crate::Task;
use crate::core::backend;
use crate::futures::futures::channel::oneshot;
use crate::task;

/// An backend operation.
#[derive(Debug)]
pub enum Action {
    /// Switches the [`backend::Settings`] of the current application.
    Configure(
        backend::Settings,
        oneshot::Sender<Result<(), backend::Error>>,
    ),
}

/// Returns a [`Task`] that switches the [`backend::Settings`] of the current application.
///
/// This can be leveraged to switch renderers at runtime.
pub fn configure(settings: backend::Settings) -> Task<Result<(), backend::Error>> {
    task::oneshot(|sender| crate::Action::Backend(Action::Configure(settings, sender)))
}
