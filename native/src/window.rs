//! Build window-based GUI applications.
mod backend;
mod event;

pub use backend::{Backend, HasRawWindowHandle};
pub use event::Event;
