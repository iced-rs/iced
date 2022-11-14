//! Listen to external events in your application.
mod tracker;

pub use tracker::Tracker;

use crate::BoxStream;

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
/// This type is normally aliased by runtimes with a specific `Event` and/or
/// `Hasher`.
///
/// [`Command`]: crate::Command
pub struct Subscription<Hasher, Event, Output> {
    recipes: Vec<Box<dyn Recipe<Hasher, Event, Output = Output>>>,
}

impl<H, E, O> Subscription<H, E, O>
where
    H: std::hash::Hasher,
{
    /// Returns an empty [`Subscription`] that will not produce any output.
    pub fn none() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    /// Creates a [`Subscription`] from a [`Recipe`] describing it.
    pub fn from_recipe(
        recipe: impl Recipe<H, E, Output = O> + 'static,
    ) -> Self {
        Self {
            recipes: vec![Box::new(recipe)],
        }
    }

    /// Batches all the provided subscriptions and returns the resulting
    /// [`Subscription`].
    pub fn batch(
        subscriptions: impl IntoIterator<Item = Subscription<H, E, O>>,
    ) -> Self {
        Self {
            recipes: subscriptions
                .into_iter()
                .flat_map(|subscription| subscription.recipes)
                .collect(),
        }
    }

    /// Returns the different recipes of the [`Subscription`].
    pub fn recipes(self) -> Vec<Box<dyn Recipe<H, E, Output = O>>> {
        self.recipes
    }

    /// Adds a value to the [`Subscription`] context.
    ///
    /// The value will be part of the identity of a [`Subscription`].
    pub fn with<T>(mut self, value: T) -> Subscription<H, E, (T, O)>
    where
        H: 'static,
        E: 'static,
        O: 'static,
        T: std::hash::Hash + Clone + Send + Sync + 'static,
    {
        Subscription {
            recipes: self
                .recipes
                .drain(..)
                .map(|recipe| {
                    Box::new(With::new(recipe, value.clone()))
                        as Box<dyn Recipe<H, E, Output = (T, O)>>
                })
                .collect(),
        }
    }

    /// Transforms the [`Subscription`] output with the given function.
    pub fn map<A>(mut self, f: fn(O) -> A) -> Subscription<H, E, A>
    where
        H: 'static,
        E: 'static,
        O: 'static,
        A: 'static,
    {
        Subscription {
            recipes: self
                .recipes
                .drain(..)
                .map(|recipe| {
                    Box::new(Map::new(recipe, f))
                        as Box<dyn Recipe<H, E, Output = A>>
                })
                .collect(),
        }
    }
}

impl<I, O, H> std::fmt::Debug for Subscription<I, O, H> {
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
/// [examples]: https://github.com/iced-rs/iced/tree/0.5/examples
/// [`download_progress`]: https://github.com/iced-rs/iced/tree/0.5/examples/download_progress
/// [`stopwatch`]: https://github.com/iced-rs/iced/tree/0.5/examples/stopwatch
pub trait Recipe<Hasher: std::hash::Hasher, Event> {
    /// The events that will be produced by a [`Subscription`] with this
    /// [`Recipe`].
    type Output;

    /// Hashes the [`Recipe`].
    ///
    /// This is used by runtimes to uniquely identify a [`Subscription`].
    fn hash(&self, state: &mut Hasher);

    /// Executes the [`Recipe`] and produces the stream of events of its
    /// [`Subscription`].
    ///
    /// It receives some stream of generic events, which is normally defined by
    /// shells.
    fn stream(
        self: Box<Self>,
        input: BoxStream<Event>,
    ) -> BoxStream<Self::Output>;
}

struct Map<Hasher, Event, A, B> {
    recipe: Box<dyn Recipe<Hasher, Event, Output = A>>,
    mapper: fn(A) -> B,
}

impl<H, E, A, B> Map<H, E, A, B> {
    fn new(
        recipe: Box<dyn Recipe<H, E, Output = A>>,
        mapper: fn(A) -> B,
    ) -> Self {
        Map { recipe, mapper }
    }
}

impl<H, E, A, B> Recipe<H, E> for Map<H, E, A, B>
where
    A: 'static,
    B: 'static,
    H: std::hash::Hasher,
{
    type Output = B;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        self.recipe.hash(state);
        self.mapper.hash(state);
    }

    fn stream(self: Box<Self>, input: BoxStream<E>) -> BoxStream<Self::Output> {
        use futures::StreamExt;

        let mapper = self.mapper;

        Box::pin(self.recipe.stream(input).map(mapper))
    }
}

struct With<Hasher, Event, A, B> {
    recipe: Box<dyn Recipe<Hasher, Event, Output = A>>,
    value: B,
}

impl<H, E, A, B> With<H, E, A, B> {
    fn new(recipe: Box<dyn Recipe<H, E, Output = A>>, value: B) -> Self {
        With { recipe, value }
    }
}

impl<H, E, A, B> Recipe<H, E> for With<H, E, A, B>
where
    A: 'static,
    B: 'static + std::hash::Hash + Clone + Send + Sync,
    H: std::hash::Hasher,
{
    type Output = (B, A);

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<B>().hash(state);
        self.value.hash(state);
        self.recipe.hash(state);
    }

    fn stream(self: Box<Self>, input: BoxStream<E>) -> BoxStream<Self::Output> {
        use futures::StreamExt;

        let value = self.value;

        Box::pin(
            self.recipe
                .stream(input)
                .map(move |element| (value.clone(), element)),
        )
    }
}
