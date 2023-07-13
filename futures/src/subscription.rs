//! Listen to external events in your application.
mod tracker;

pub use tracker::Tracker;

use crate::core::event::{self, Event};
use crate::core::window;
use crate::core::Hasher;
use crate::futures::{Future, Stream};
use crate::{BoxStream, MaybeSend};

use futures::channel::mpsc;
use futures::never::Never;
use std::hash::Hash;

/// A stream of runtime events.
///
/// It is the input of a [`Subscription`].
pub type EventStream = BoxStream<(Event, event::Status)>;

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
#[must_use = "`Subscription` must be returned to runtime to take effect"]
pub struct Subscription<Message> {
    recipes: Vec<Box<dyn Recipe<Output = Message>>>,
}

impl<Message> Subscription<Message> {
    /// Returns an empty [`Subscription`] that will not produce any output.
    pub fn none() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    /// Creates a [`Subscription`] from a [`Recipe`] describing it.
    pub fn from_recipe(
        recipe: impl Recipe<Output = Message> + 'static,
    ) -> Self {
        Self {
            recipes: vec![Box::new(recipe)],
        }
    }

    /// Batches all the provided subscriptions and returns the resulting
    /// [`Subscription`].
    pub fn batch(
        subscriptions: impl IntoIterator<Item = Subscription<Message>>,
    ) -> Self {
        Self {
            recipes: subscriptions
                .into_iter()
                .flat_map(|subscription| subscription.recipes)
                .collect(),
        }
    }

    /// Returns the different recipes of the [`Subscription`].
    pub fn into_recipes(self) -> Vec<Box<dyn Recipe<Output = Message>>> {
        self.recipes
    }

    /// Adds a value to the [`Subscription`] context.
    ///
    /// The value will be part of the identity of a [`Subscription`].
    pub fn with<T>(mut self, value: T) -> Subscription<(T, Message)>
    where
        Message: 'static,
        T: std::hash::Hash + Clone + Send + Sync + 'static,
    {
        Subscription {
            recipes: self
                .recipes
                .drain(..)
                .map(|recipe| {
                    Box::new(With::new(recipe, value.clone()))
                        as Box<dyn Recipe<Output = (T, Message)>>
                })
                .collect(),
        }
    }

    /// Transforms the [`Subscription`] output with the given function.
    pub fn map<A>(mut self, f: fn(Message) -> A) -> Subscription<A>
    where
        Message: 'static,
        A: 'static,
    {
        Subscription {
            recipes: self
                .recipes
                .drain(..)
                .map(|recipe| {
                    Box::new(Map::new(recipe, f)) as Box<dyn Recipe<Output = A>>
                })
                .collect(),
        }
    }
}

impl<Message> std::fmt::Debug for Subscription<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription").finish()
    }
}

/// The description of a [`Subscription`].
///
/// A [`Recipe`] is the internal definition of a [`Subscription`]. It is used
/// by runtimes to run and identify subscriptions. You can use it to create your
/// own!
///
/// # Examples
/// The repository has a couple of [examples] that use a custom [`Recipe`]:
///
/// - [`download_progress`], a basic application that asynchronously downloads
/// a dummy file of 100 MB and tracks the download progress.
/// - [`stopwatch`], a watch with start/stop and reset buttons showcasing how
/// to listen to time.
///
/// [examples]: https://github.com/iced-rs/iced/tree/0.9/examples
/// [`download_progress`]: https://github.com/iced-rs/iced/tree/0.9/examples/download_progress
/// [`stopwatch`]: https://github.com/iced-rs/iced/tree/0.9/examples/stopwatch
pub trait Recipe {
    /// The events that will be produced by a [`Subscription`] with this
    /// [`Recipe`].
    type Output;

    /// Hashes the [`Recipe`].
    ///
    /// This is used by runtimes to uniquely identify a [`Subscription`].
    fn hash(&self, state: &mut Hasher);

    /// Executes the [`Recipe`] and produces the stream of events of its
    /// [`Subscription`].
    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<Self::Output>;
}

struct Map<A, B> {
    recipe: Box<dyn Recipe<Output = A>>,
    mapper: fn(A) -> B,
}

impl<A, B> Map<A, B> {
    fn new(recipe: Box<dyn Recipe<Output = A>>, mapper: fn(A) -> B) -> Self {
        Map { recipe, mapper }
    }
}

impl<A, B> Recipe for Map<A, B>
where
    A: 'static,
    B: 'static,
{
    type Output = B;

    fn hash(&self, state: &mut Hasher) {
        self.recipe.hash(state);
        self.mapper.hash(state);
    }

    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<Self::Output> {
        use futures::StreamExt;

        let mapper = self.mapper;

        Box::pin(self.recipe.stream(input).map(mapper))
    }
}

struct With<A, B> {
    recipe: Box<dyn Recipe<Output = A>>,
    value: B,
}

impl<A, B> With<A, B> {
    fn new(recipe: Box<dyn Recipe<Output = A>>, value: B) -> Self {
        With { recipe, value }
    }
}

impl<A, B> Recipe for With<A, B>
where
    A: 'static,
    B: 'static + std::hash::Hash + Clone + Send + Sync,
{
    type Output = (B, A);

    fn hash(&self, state: &mut Hasher) {
        std::any::TypeId::of::<B>().hash(state);
        self.value.hash(state);
        self.recipe.hash(state);
    }

    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<Self::Output> {
        use futures::StreamExt;

        let value = self.value;

        Box::pin(
            self.recipe
                .stream(input)
                .map(move |element| (value.clone(), element)),
        )
    }
}

/// Returns a [`Subscription`] to all the ignored runtime events.
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
    Message: 'static + MaybeSend,
{
    #[derive(Hash)]
    struct EventsWith;

    Subscription::from_recipe(Runner {
        id: (EventsWith, f),
        spawn: move |events| {
            use futures::future;
            use futures::stream::StreamExt;

            events.filter_map(move |(event, status)| {
                future::ready(match event {
                    Event::Window(_, window::Event::RedrawRequested(_)) => None,
                    _ => f(event, status),
                })
            })
        },
    })
}

/// Returns a [`Subscription`] that produces a message for every runtime event,
/// including the redraw request events.
///
/// **Warning:** This [`Subscription`], if unfiltered, may produce messages in
/// an infinite loop.
pub fn raw_events<Message>(
    f: fn(Event, event::Status) -> Option<Message>,
) -> Subscription<Message>
where
    Message: 'static + MaybeSend,
{
    #[derive(Hash)]
    struct RawEvents;

    Subscription::from_recipe(Runner {
        id: (RawEvents, f),
        spawn: move |events| {
            use futures::future;
            use futures::stream::StreamExt;

            events.filter_map(move |(event, status)| {
                future::ready(f(event, status))
            })
        },
    })
}

/// Returns a [`Subscription`] that will call the given function to create and
/// asynchronously run the given [`Stream`].
pub fn run<S, Message>(builder: fn() -> S) -> Subscription<Message>
where
    S: Stream<Item = Message> + MaybeSend + 'static,
    Message: 'static,
{
    Subscription::from_recipe(Runner {
        id: builder,
        spawn: move |_| builder(),
    })
}

/// Returns a [`Subscription`] that will create and asynchronously run the
/// given [`Stream`].
///
/// The `id` will be used to uniquely identify the [`Subscription`].
pub fn run_with_id<I, S, Message>(id: I, stream: S) -> Subscription<Message>
where
    I: Hash + 'static,
    S: Stream<Item = Message> + MaybeSend + 'static,
    Message: 'static,
{
    Subscription::from_recipe(Runner {
        id,
        spawn: move |_| stream,
    })
}

/// Returns a [`Subscription`] that will create and asynchronously run a
/// [`Stream`] that will call the provided closure to produce every `Message`.
///
/// The `id` will be used to uniquely identify the [`Subscription`].
pub fn unfold<I, T, Fut, Message>(
    id: I,
    initial: T,
    mut f: impl FnMut(T) -> Fut + MaybeSend + Sync + 'static,
) -> Subscription<Message>
where
    I: Hash + 'static,
    T: MaybeSend + 'static,
    Fut: Future<Output = (Message, T)> + MaybeSend + 'static,
    Message: 'static + MaybeSend,
{
    use futures::future::FutureExt;

    run_with_id(
        id,
        futures::stream::unfold(initial, move |state| f(state).map(Some)),
    )
}

/// Creates a [`Subscription`] that publishes the events sent from a [`Future`]
/// to an [`mpsc::Sender`] with the given bounds.
///
/// # Creating an asynchronous worker with bidirectional communication
/// You can leverage this helper to create a [`Subscription`] that spawns
/// an asynchronous worker in the background and establish a channel of
/// communication with an `iced` application.
///
/// You can achieve this by creating an `mpsc` channel inside the closure
/// and returning the `Sender` as a `Message` for the `Application`:
///
/// ```
/// use iced_futures::subscription::{self, Subscription};
/// use iced_futures::futures::channel::mpsc;
/// use iced_futures::futures::sink::SinkExt;
///
/// pub enum Event {
///     Ready(mpsc::Sender<Input>),
///     WorkFinished,
///     // ...
/// }
///
/// enum Input {
///     DoSomeWork,
///     // ...
/// }
///
/// enum State {
///     Starting,
///     Ready(mpsc::Receiver<Input>),
/// }
///
/// fn some_worker() -> Subscription<Event> {
///     struct SomeWorker;
///
///     subscription::channel(std::any::TypeId::of::<SomeWorker>(), 100, |mut output| async move {
///         let mut state = State::Starting;
///
///         loop {
///             match &mut state {
///                 State::Starting => {
///                     // Create channel
///                     let (sender, receiver) = mpsc::channel(100);
///
///                     // Send the sender back to the application
///                     output.send(Event::Ready(sender)).await;
///
///                     // We are ready to receive messages
///                     state = State::Ready(receiver);
///                 }
///                 State::Ready(receiver) => {
///                     use iced_futures::futures::StreamExt;
///
///                     // Read next input sent from `Application`
///                     let input = receiver.select_next_some().await;
///
///                     match input {
///                         Input::DoSomeWork => {
///                             // Do some async work...
///
///                             // Finally, we can optionally produce a message to tell the
///                             // `Application` the work is done
///                             output.send(Event::WorkFinished).await;
///                         }
///                     }
///                 }
///             }
///         }
///     })
/// }
/// ```
///
/// Check out the [`websocket`] example, which showcases this pattern to maintain a WebSocket
/// connection open.
///
/// [`websocket`]: https://github.com/iced-rs/iced/tree/0.9/examples/websocket
pub fn channel<I, Fut, Message>(
    id: I,
    size: usize,
    f: impl FnOnce(mpsc::Sender<Message>) -> Fut + MaybeSend + 'static,
) -> Subscription<Message>
where
    I: Hash + 'static,
    Fut: Future<Output = Never> + MaybeSend + 'static,
    Message: 'static + MaybeSend,
{
    use futures::stream::{self, StreamExt};

    Subscription::from_recipe(Runner {
        id,
        spawn: move |_| {
            let (sender, receiver) = mpsc::channel(size);

            let runner = stream::once(f(sender)).map(|_| unreachable!());

            stream::select(receiver, runner)
        },
    })
}

struct Runner<I, F, S, Message>
where
    F: FnOnce(EventStream) -> S,
    S: Stream<Item = Message>,
{
    id: I,
    spawn: F,
}

impl<I, S, F, Message> Recipe for Runner<I, F, S, Message>
where
    I: Hash + 'static,
    F: FnOnce(EventStream) -> S,
    S: Stream<Item = Message> + MaybeSend + 'static,
{
    type Output = Message;

    fn hash(&self, state: &mut Hasher) {
        std::any::TypeId::of::<I>().hash(state);
        self.id.hash(state);
    }

    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<Self::Output> {
        crate::boxed_stream((self.spawn)(input))
    }
}
