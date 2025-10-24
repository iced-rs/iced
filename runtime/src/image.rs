//! Allocate images explicitly to control presentation.
use crate::core::image::Handle;
use crate::futures::futures::channel::oneshot;
use crate::task::{self, Task};

pub use crate::core::image::Allocation;

/// An image action.
#[derive(Debug)]
pub enum Action {
    /// Allocates the given [`Handle`].
    Allocate(Handle, oneshot::Sender<Allocation>),
}

/// Allocates an image [`Handle`].
///
/// When you obtain an [`Allocation`] explicitly, you get the guarantee
/// that using a [`Handle`] will draw the corresponding image immediately
/// in the next frame.
pub fn allocate(handle: Handle) -> Task<Allocation> {
    task::oneshot(|sender| {
        crate::Action::Image(Action::Allocate(handle, sender))
    })
}
