//! Listen to gamepad events.
use crate::core;
use crate::core::gamepad::Event;
use crate::subscription::{self, Subscription};

/// Returns a [`Subscription`] that listens to ignored gamepad events.
pub fn listen() -> Subscription<Event> {
    #[derive(Hash)]
    struct Listen;

    subscription::filter_map(Listen, move |event| match event {
        subscription::Event::Interaction {
            event: core::Event::Gamepad(event),
            status: core::event::Status::Ignored,
            ..
        } => Some(event),
        _ => None,
    })
}
