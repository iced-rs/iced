//! Reuse basic mouse types.
mod button;
mod event;
mod interaction;

pub use button::Button;
pub use event::{Event, ScrollDelta};
pub use interaction::Interaction;
