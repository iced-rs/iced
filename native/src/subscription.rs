//! Listen to external events in your application.
use crate::event::{self, Event};
use crate::Hasher;

use iced_futures::futures::{self, Future, Stream};
use iced_futures::BoxStream;

use std::hash::Hash;

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

/// Returns a [`Subscription`] to all the runtime events.
///
/// This subscription will notify your application of any [`Event`] that was
/// not captured by any widget.
pub fn events() -> Subscription<Event> {
    events_with(|event, status| match status {
        event::Status::Ignored => Some(event),
        event::Status::Captured => None,
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
    Subscription::from_recipe(Runner {
        id: f,
        initial: (),
        spawn: move |_, events| {
            use futures::future;
            use futures::stream::StreamExt;

            events.filter_map(move |(event, status)| {
                future::ready(f(event, status))
            })
        },
    })
}

/// Returns a [`Subscription`] that will create and asynchronously run the
/// [`Stream`] returned by the provided closure.
///
/// The `initial` state will be used to uniquely identify the [`Subscription`].
pub fn run<I, T, S, Message>(
    id: I,
    initial: T,
    f: impl FnOnce(T) -> S + 'static,
) -> Subscription<Message>
where
    I: Hash + 'static,
    T: 'static,
    S: Stream<Item = Message> + Send + 'static,
    Message: 'static,
{
    Subscription::from_recipe(Runner {
        id,
        initial,
        spawn: move |initial, _| f(initial),
    })
}

/// Returns a [`Subscription`] that will create and asynchronously run a
/// [`Stream`] that will call the provided closure to produce every `Message`.
///
/// The `initial` state will be used to uniquely identify the [`Subscription`].
pub fn unfold<I, T, Fut, Message>(
    id: I,
    initial: T,
    mut f: impl FnMut(T) -> Fut + Send + Sync + 'static,
) -> Subscription<Message>
where
    I: Hash + 'static,
    T: Send + 'static,
    Fut: Future<Output = (Option<Message>, T)> + Send + 'static,
    Message: 'static + Send,
{
    use futures::future::{self, FutureExt};
    use futures::stream::StreamExt;

    run(id, initial, move |initial| {
        futures::stream::unfold(initial, move |state| f(state).map(Some))
            .filter_map(future::ready)
    })
}

struct Runner<I, T, F, S, Message>
where
    F: FnOnce(T, EventStream) -> S,
    S: Stream<Item = Message>,
{
    id: I,
    initial: T,
    spawn: F,
}

impl<I, T, S, F, Message> Recipe<Hasher, (Event, event::Status)>
    for Runner<I, T, F, S, Message>
where
    I: Hash + 'static,
    T: 'static,
    F: FnOnce(T, EventStream) -> S,
    S: Stream<Item = Message> + Send + 'static,
{
    type Output = Message;

    fn hash(&self, state: &mut Hasher) {
        std::any::TypeId::of::<T>().hash(state);

        self.id.hash(state);
    }

    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<Self::Output> {
        use futures::stream::StreamExt;

        (self.spawn)(self.initial, input).boxed()
    }
}
