//! Listen to external events in your application.
use crate::{Event, Hasher};
use iced_futures::futures::stream::BoxStream;

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
/// [`Command`]: ../struct.Command.html
/// [`Subscription`]: struct.Subscription.html
pub type Subscription<T> = iced_futures::Subscription<Hasher, Event, T>;

/// A stream of runtime events.
///
/// It is the input of a [`Subscription`] in the native runtime.
///
/// [`Subscription`]: type.Subscription.html
pub type EventStream = BoxStream<'static, Event>;

/// A native [`Subscription`] tracker.
///
/// [`Subscription`]: type.Subscription.html
pub type Tracker = iced_futures::subscription::Tracker<Hasher, Event>;

pub use iced_futures::subscription::Recipe;

mod events;

use events::Events;

/// Returns a [`Subscription`] to all the runtime events.
///
/// This subscription will notify your application of any [`Event`] handled by
/// the runtime.
///
/// [`Subscription`]: type.Subscription.html
/// [`Event`]: ../enum.Event.html
pub fn events() -> Subscription<Event> {
    Subscription::from_recipe(Events)
}
