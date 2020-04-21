//! Build window-based GUI applications.
mod backend;
mod event;

pub use backend::{HasRawWindowHandle, Backend};
pub use event::Event;
