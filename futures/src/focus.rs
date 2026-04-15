//! Listen to focus change events.
use crate::core;
use crate::core::focus::Event;
use crate::subscription::{self, Subscription};

/// Returns a [`Subscription`] that listens to focus change events.
///
/// A focus change event is produced whenever a widget gains or loses
/// focus — whether through keyboard/gamepad navigation, mouse clicks,
/// auto-focus, or programmatic focus operations.
pub fn changed() -> Subscription<Event> {
    #[derive(Hash)]
    struct Changed;

    subscription::filter_map(Changed, move |event| match event {
        subscription::Event::Interaction {
            event: core::Event::Focus(event),
            ..
        } => Some(event),
        _ => None,
    })
}
