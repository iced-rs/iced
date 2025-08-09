//! Listen to external events in your application.
mod tracker;

pub use tracker::Tracker;

use crate::core::event;
use crate::core::window;
use crate::futures::Stream;
use crate::{BoxStream, MaybeSend};

use std::any::TypeId;
use std::hash::Hash;

/// A subscription event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A user interacted with a user interface in a window.
    Interaction {
        /// The window holding the interface of the interaction.
        window: window::Id,
        /// The [`Event`] describing the interaction.
        ///
        /// [`Event`]: event::Event
        event: event::Event,

        /// The [`event::Status`] of the interaction.
        status: event::Status,
    },

    /// A platform specific event.
    PlatformSpecific(PlatformSpecific),
}

/// A platform specific event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformSpecific {
    /// A MacOS specific event
    MacOS(MacOS),
}

/// Describes an event specific to MacOS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacOS {
    /// Triggered when the app receives an URL from the system
    ///
    /// _**Note:** For this event to be triggered, the executable needs to be properly [bundled]!_
    ///
    /// [bundled]: https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html#//apple_ref/doc/uid/10000123i-CH101-SW19
    ReceivedUrl(String),
}

/// A stream of runtime events.
///
/// It is the input of a [`Subscription`].
pub type EventStream = BoxStream<Event>;

/// The hasher used for identifying subscriptions.
pub type Hasher = rustc_hash::FxHasher;

/// A request to listen to external events.
///
/// Besides performing async actions on demand with `Task`, most
/// applications also need to listen to external events passively.
///
/// A [`Subscription`] is normally provided to some runtime, like a `Task`,
/// and it will generate events as long as the user keeps requesting it.
///
/// For instance, you can use a [`Subscription`] to listen to a `WebSocket`
/// connection, keyboard presses, mouse events, time ticks, etc.
///
/// # The Lifetime of a [`Subscription`]
/// Much like a [`Future`] or a [`Stream`], a [`Subscription`] does not produce any effects
/// on its own. For a [`Subscription`] to run, it must be returned to the iced runtime—normally
/// in the `subscription` function of an `application` or a `daemon`.
///
/// When a [`Subscription`] is provided to the runtime for the first time, the runtime will
/// start running it asynchronously. Running a [`Subscription`] consists in building its underlying
/// [`Stream`] and executing it in an async runtime.
///
/// Therefore, you can think of a [`Subscription`] as a "stream builder". It simply represents a way
/// to build a certain [`Stream`] together with some way to _identify_ it.
///
/// Identification is important because when a specific [`Subscription`] stops being returned to the
/// iced runtime, the runtime will kill its associated [`Stream`]. The runtime uses the identity of a
/// [`Subscription`] to keep track of it.
///
/// This way, iced allows you to declaratively __subscribe__ to particular streams of data temporarily
/// and whenever necessary.
///
/// ```
/// # mod iced {
/// #     pub mod time {
/// #         pub use iced_futures::backend::default::time::every;
/// #         pub use std::time::{Duration, Instant};
/// #     }
/// #
/// #     pub use iced_futures::Subscription;
/// # }
/// use iced::time::{self, Duration, Instant};
/// use iced::Subscription;
///
/// struct State {
///     timer_enabled: bool,
/// }
///
/// fn subscription(state: &State) -> Subscription<Instant> {
///     if state.timer_enabled {
///         time::every(Duration::from_secs(1))
///     } else {
///         Subscription::none()
///     }
/// }
/// ```
///
/// [`Future`]: std::future::Future
#[must_use = "`Subscription` must be returned to the runtime to take effect; normally in your `subscription` function."]
pub struct Subscription<T> {
    recipes: Vec<Box<dyn Recipe<Output = T>>>,
}

impl<T> Subscription<T> {
    /// Returns an empty [`Subscription`] that will not produce any output.
    pub fn none() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    /// Returns a [`Subscription`] that will call the given function to create and
    /// asynchronously run the given [`Stream`].
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
    /// # mod iced {
    /// #     pub use iced_futures::Subscription;   
    /// #     pub use iced_futures::futures;
    /// #     pub use iced_futures::stream;
    /// # }
    /// use iced::futures::channel::mpsc;
    /// use iced::futures::sink::SinkExt;
    /// use iced::futures::Stream;
    /// use iced::stream;
    /// use iced::Subscription;
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
    /// fn some_worker() -> impl Stream<Item = Event> {
    ///     stream::channel(100, async |mut output| {
    ///         // Create channel
    ///         let (sender, mut receiver) = mpsc::channel(100);
    ///
    ///         // Send the sender back to the application
    ///         output.send(Event::Ready(sender)).await;
    ///
    ///         loop {
    ///             use iced_futures::futures::StreamExt;
    ///
    ///             // Read next input sent from `Application`
    ///             let input = receiver.select_next_some().await;
    ///
    ///             match input {
    ///                 Input::DoSomeWork => {
    ///                     // Do some async work...
    ///
    ///                     // Finally, we can optionally produce a message to tell the
    ///                     // `Application` the work is done
    ///                     output.send(Event::WorkFinished).await;
    ///                 }
    ///             }
    ///         }
    ///     })
    /// }
    ///
    /// fn subscription() -> Subscription<Event> {
    ///     Subscription::run(some_worker)
    /// }
    /// ```
    ///
    /// Check out the [`websocket`] example, which showcases this pattern to maintain a `WebSocket`
    /// connection open.
    ///
    /// [`websocket`]: https://github.com/iced-rs/iced/tree/0.13/examples/websocket
    pub fn run<S>(builder: fn() -> S) -> Self
    where
        S: Stream<Item = T> + MaybeSend + 'static,
        T: 'static,
    {
        from_recipe(Runner {
            data: builder,
            spawn: |builder, _| builder(),
        })
    }

    /// Returns a [`Subscription`] that will create and asynchronously run the
    /// given [`Stream`].
    ///
    /// Both the `data` and the function pointer will be used to uniquely identify
    /// the [`Subscription`].
    pub fn run_with<D, S>(data: D, builder: fn(&D) -> S) -> Self
    where
        D: Hash + 'static,
        S: Stream<Item = T> + MaybeSend + 'static,
        T: 'static,
    {
        from_recipe(Runner {
            data: (data, builder),
            spawn: |(data, builder), _| builder(data),
        })
    }

    /// Batches all the provided subscriptions and returns the resulting
    /// [`Subscription`].
    pub fn batch(
        subscriptions: impl IntoIterator<Item = Subscription<T>>,
    ) -> Self {
        Self {
            recipes: subscriptions
                .into_iter()
                .flat_map(|subscription| subscription.recipes)
                .collect(),
        }
    }

    /// Adds a value to the [`Subscription`] context.
    ///
    /// The value will be part of the identity of a [`Subscription`].
    pub fn with<A>(mut self, value: A) -> Subscription<(A, T)>
    where
        T: 'static,
        A: std::hash::Hash + Clone + Send + Sync + 'static,
    {
        Subscription {
            recipes: self
                .recipes
                .drain(..)
                .map(|recipe| {
                    Box::new(With::new(recipe, value.clone()))
                        as Box<dyn Recipe<Output = (A, T)>>
                })
                .collect(),
        }
    }

    /// Transforms the [`Subscription`] output with the given function.
    ///
    /// # Panics
    /// The closure provided must be a non-capturing closure. The method
    /// will panic in debug mode otherwise.
    pub fn map<F, A>(mut self, f: F) -> Subscription<A>
    where
        T: 'static,
        F: Fn(T) -> A + MaybeSend + Clone + 'static,
        A: 'static,
    {
        debug_assert!(
            std::mem::size_of::<F>() == 0,
            "the closure {} provided in `Subscription::map` is capturing",
            std::any::type_name::<F>(),
        );

        Subscription {
            recipes: self
                .recipes
                .drain(..)
                .map(move |recipe| {
                    Box::new(Map::new(recipe, f.clone()))
                        as Box<dyn Recipe<Output = A>>
                })
                .collect(),
        }
    }

    /// Returns the amount of recipe units in this [`Subscription`].
    pub fn units(&self) -> usize {
        self.recipes.len()
    }
}

/// Creates a [`Subscription`] from a [`Recipe`] describing it.
pub fn from_recipe<T>(
    recipe: impl Recipe<Output = T> + 'static,
) -> Subscription<T> {
    Subscription {
        recipes: vec![Box::new(recipe)],
    }
}

/// Returns the different recipes of the [`Subscription`].
pub fn into_recipes<T>(
    subscription: Subscription<T>,
) -> Vec<Box<dyn Recipe<Output = T>>> {
    subscription.recipes
}

impl<T> std::fmt::Debug for Subscription<T> {
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
///   a dummy file of 100 MB and tracks the download progress.
/// - [`stopwatch`], a watch with start/stop and reset buttons showcasing how
///   to listen to time.
///
/// [examples]: https://github.com/iced-rs/iced/tree/0.13/examples
/// [`download_progress`]: https://github.com/iced-rs/iced/tree/0.13/examples/download_progress
/// [`stopwatch`]: https://github.com/iced-rs/iced/tree/0.13/examples/stopwatch
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

struct Map<A, B, F>
where
    F: Fn(A) -> B + 'static,
{
    recipe: Box<dyn Recipe<Output = A>>,
    mapper: F,
}

impl<A, B, F> Map<A, B, F>
where
    F: Fn(A) -> B + 'static,
{
    fn new(recipe: Box<dyn Recipe<Output = A>>, mapper: F) -> Self {
        Map { recipe, mapper }
    }
}

impl<A, B, F> Recipe for Map<A, B, F>
where
    A: 'static,
    B: 'static,
    F: Fn(A) -> B + 'static + MaybeSend,
{
    type Output = B;

    fn hash(&self, state: &mut Hasher) {
        TypeId::of::<F>().hash(state);
        self.recipe.hash(state);
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

pub(crate) fn filter_map<I, F, T>(id: I, f: F) -> Subscription<T>
where
    I: Hash + 'static,
    F: Fn(Event) -> Option<T> + MaybeSend + 'static,
    T: 'static + MaybeSend,
{
    from_recipe(Runner {
        data: id,
        spawn: |_, events| {
            use futures::future;
            use futures::stream::StreamExt;

            events.filter_map(move |event| future::ready(f(event)))
        },
    })
}

struct Runner<I, F, S, T>
where
    F: FnOnce(&I, EventStream) -> S,
    S: Stream<Item = T>,
{
    data: I,
    spawn: F,
}

impl<I, F, S, T> Recipe for Runner<I, F, S, T>
where
    I: Hash + 'static,
    F: FnOnce(&I, EventStream) -> S,
    S: Stream<Item = T> + MaybeSend + 'static,
{
    type Output = T;

    fn hash(&self, state: &mut Hasher) {
        std::any::TypeId::of::<I>().hash(state);
        self.data.hash(state);
    }

    fn stream(self: Box<Self>, input: EventStream) -> BoxStream<Self::Output> {
        crate::boxed_stream((self.spawn)(&self.data, input))
    }
}
