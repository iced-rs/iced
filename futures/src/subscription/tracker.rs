use crate::{BoxFuture, Subscription};

use futures::{channel::mpsc, sink::Sink};
use std::{collections::HashMap, marker::PhantomData};

/// A registry of subscription streams.
///
/// If you have an application that continuously returns a [`Subscription`],
/// you can use a [`Tracker`] to keep track of the different recipes and keep
/// its executions alive.
#[derive(Debug)]
pub struct Tracker<Hasher, Event> {
    subscriptions: HashMap<u64, Execution<Event>>,
    _hasher: PhantomData<Hasher>,
}

#[derive(Debug)]
pub struct Execution<Event> {
    _cancel: futures::channel::oneshot::Sender<()>,
    listener: Option<futures::channel::mpsc::Sender<Event>>,
}

impl<Hasher, Event> Tracker<Hasher, Event>
where
    Hasher: std::hash::Hasher + Default,
    Event: 'static + Send + Clone,
{
    /// Creates a new empty [`Tracker`].
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            _hasher: PhantomData,
        }
    }

    /// Updates the [`Tracker`] with the given [`Subscription`].
    ///
    /// A [`Subscription`] can cause new streams to be spawned or old streams
    /// to be closed.
    ///
    /// The [`Tracker`] keeps track of these streams between calls to this
    /// method:
    ///
    /// - If the provided [`Subscription`] contains a new [`Recipe`] that is
    /// currently not being run, it will spawn a new stream and keep it alive.
    /// - On the other hand, if a [`Recipe`] is currently in execution and the
    /// provided [`Subscription`] does not contain it anymore, then the
    /// [`Tracker`] will close and drop the relevant stream.
    ///
    /// It returns a list of futures that need to be spawned to materialize
    /// the [`Tracker`] changes.
    ///
    /// [`Recipe`]: crate::subscription::Recipe
    pub fn update<Message, Receiver>(
        &mut self,
        subscription: Subscription<Hasher, Event, Message>,
        receiver: Receiver,
    ) -> Vec<BoxFuture<()>>
    where
        Message: 'static + Send,
        Receiver: 'static
            + Sink<Message, Error = mpsc::SendError>
            + Unpin
            + Send
            + Clone,
    {
        use futures::{future::FutureExt, stream::StreamExt};

        let mut futures: Vec<BoxFuture<()>> = Vec::new();

        let recipes = subscription.recipes();
        let mut alive = std::collections::HashSet::new();

        for recipe in recipes {
            let id = {
                let mut hasher = Hasher::default();
                recipe.hash(&mut hasher);

                hasher.finish()
            };

            let _ = alive.insert(id);

            if self.subscriptions.contains_key(&id) {
                continue;
            }

            let (cancel, cancelled) = futures::channel::oneshot::channel();

            // TODO: Use bus if/when it supports async
            let (event_sender, event_receiver) =
                futures::channel::mpsc::channel(100);

            let stream = recipe.stream(event_receiver.boxed());

            let future = futures::future::select(
                cancelled,
                stream.map(Ok).forward(receiver.clone()),
            )
            .map(|_| ());

            let _ = self.subscriptions.insert(
                id,
                Execution {
                    _cancel: cancel,
                    listener: if event_sender.is_closed() {
                        None
                    } else {
                        Some(event_sender)
                    },
                },
            );

            futures.push(Box::pin(future));
        }

        self.subscriptions.retain(|id, _| alive.contains(&id));

        futures
    }

    /// Broadcasts an event to the subscriptions currently alive.
    ///
    /// A subscription's [`Recipe::stream`] always receives a stream of events
    /// as input. This stream can be used by some subscription to listen to
    /// shell events.
    ///
    /// This method publishes the given event to all the subscription streams
    /// currently open.
    ///
    /// [`Recipe::stream`]: crate::subscription::Recipe::stream
    pub fn broadcast(&mut self, event: Event) {
        self.subscriptions
            .values_mut()
            .filter_map(|connection| connection.listener.as_mut())
            .for_each(|listener| {
                if let Err(error) = listener.try_send(event.clone()) {
                    log::warn!(
                        "Error sending event to subscription: {:?}",
                        error
                    );
                }
            });
    }
}
