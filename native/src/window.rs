//! Build window-based GUI applications.
mod action;

pub use action::Action;

use crate::core::time::Instant;
use crate::core::window::Event;
use crate::futures::subscription::{self, Subscription};

/// Subscribes to the frames of the window of the running application.
///
/// The resulting [`Subscription`] will produce items at a rate equal to the
/// refresh rate of the window. Note that this rate may be variable, as it is
/// normally managed by the graphics driver and/or the OS.
///
/// In any case, this [`Subscription`] is useful to smoothly draw application-driven
/// animations without missing any frames.
pub fn frames() -> Subscription<Instant> {
    subscription::raw_events(|event, _status| match event {
        iced_core::Event::Window(Event::RedrawRequested(at)) => Some(at),
        _ => None,
    })
}
