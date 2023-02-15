//! Build window-based GUI applications.
mod action;
mod event;
mod icon;
mod id;
mod mode;
mod position;
mod redraw_request;
mod settings;
mod user_attention;

pub use action::Action;
pub use event::Event;
pub use icon::Icon;
pub use id::Id;
pub use mode::Mode;
pub use position::Position;
pub use redraw_request::RedrawRequest;
pub use settings::Settings;
pub use user_attention::UserAttention;

use crate::subscription::{self, Subscription};
use crate::time::Instant;

/// Subscribes to the frames of the window of the running application.
///
/// The resulting [`Subscription`] will produce items at a rate equal to the
/// refresh rate of the window. Note that this rate may be variable, as it is
/// normally managed by the graphics driver and/or the OS.
///
/// In any case, this [`Subscription`] is useful to smoothly draw application-driven
/// animations without missing any frames.
pub fn frames() -> Subscription<Frame> {
    subscription::raw_events(|event, _status| match event {
        crate::Event::Window(id, Event::RedrawRequested(at)) => {
            Some(Frame { id, at })
        }
        _ => None,
    })
}

/// The returned `Frame` for a framerate subscription.
#[derive(Debug)]
pub struct Frame {
    pub id: Id,
    pub at: Instant,
}
