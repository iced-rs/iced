//! Listen to external events in your application.
use crate::event::{self, Event};
use crate::Hasher;
use iced_futures::BoxStream;

/// A request to listen to external events.
///
/// Besides performing async actions on demand with [`Command`], most
/// applications also need to listen to external events passively.
///
/// A [`Subscription`] is normally provided to some runtime, like a [`Command`],
/// and it will generate events as long as the user keeps requesting it.
///
/// For instance, you can use a [`Subscription`] to listen to a WebSocket
/// connection, keyboard presses, mouse events, time ticks, etc.
///
/// [`Command`]: crate::Command
pub type Subscription<T> =
    iced_futures::Subscription<Hasher, (Event, event::Status), T>;

/// A stream of runtime events.
///
/// It is the input of a [`Subscription`] in the native runtime.
pub type EventStream = BoxStream<(Event, event::Status)>;

/// A native [`Subscription`] tracker.
pub type Tracker =
    iced_futures::subscription::Tracker<Hasher, (Event, event::Status)>;

pub use iced_futures::subscription::Recipe;

mod events;

use events::Events;

/// Returns a [`Subscription`] to all the runtime events.
///
/// This subscription will notify your application of any [`Event`] that was
/// not captured by any widget.
pub fn events() -> Subscription<Event> {
    Subscription::from_recipe(Events {
        f: |event, status| match status {
            event::Status::Ignored => Some(event),
            event::Status::Captured => None,
        },
    })
}

/// Returns a [`Subscription`] that filters all the runtime events with the
/// provided function, producing messages accordingly.
///
/// This subscription will call the provided function for every [`Event`]
/// handled by the runtime. If the function:
///
/// - Returns `None`, the [`Event`] will be discarded.
/// - Returns `Some` message, the `Message` will be produced.
pub fn events_with<Message>(
    f: fn(Event, event::Status) -> Option<Message>,
) -> Subscription<Message>
where
    Message: 'static + Send,
{
    Subscription::from_recipe(Events { f })
}
