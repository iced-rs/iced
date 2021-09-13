//! Run commands and keep track of subscriptions.
use crate::BoxFuture;
use crate::{subscription, Executor, Subscription};

use futures::{channel::mpsc, Sink};
use std::marker::PhantomData;

/// A batteries-included runtime of commands and subscriptions.
///
/// If you have an [`Executor`], a [`Runtime`] can be leveraged to run any
/// [`Command`] or [`Subscription`] and get notified of the results!
#[derive(Debug)]
pub struct Runtime<Hasher, Event, Executor, Sender, Message> {
    executor: Executor,
    sender: Sender,
    subscriptions: subscription::Tracker<Hasher, Event>,
    _message: PhantomData<Message>,
}

impl<Hasher, Event, Executor, Sender, Message>
    Runtime<Hasher, Event, Executor, Sender, Message>
where
    Hasher: std::hash::Hasher + Default,
    Event: Send + Clone + 'static,
    Executor: self::Executor,
    Sender:
        Sink<Message, Error = mpsc::SendError> + Unpin + Send + Clone + 'static,
    Message: Send + 'static,
{
    /// Creates a new empty [`Runtime`].
    ///
    /// You need to provide:
    /// - an [`Executor`] to spawn futures
    /// - a `Sender` implementing `Sink` to receive the results
    pub fn new(executor: Executor, sender: Sender) -> Self {
        Self {
            executor,
            sender,
            subscriptions: subscription::Tracker::new(),
            _message: PhantomData,
        }
    }

    /// Runs the given closure inside the [`Executor`] of the [`Runtime`].
    ///
    /// See [`Executor::enter`] to learn more.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        self.executor.enter(f)
    }

    /// Spawns a [`Command`] in the [`Runtime`].
    ///
    /// The resulting `Message` will be forwarded to the `Sender` of the
    /// [`Runtime`].
    pub fn spawn(&mut self, future: BoxFuture<Message>) {
        use futures::{FutureExt, SinkExt};

        let mut sender = self.sender.clone();

        let future = future.then(|message| async move {
            let _ = sender.send(message).await;

            ()
        });

        self.executor.spawn(future);
    }

    /// Tracks a [`Subscription`] in the [`Runtime`].
    ///
    /// It will spawn new streams or close old ones as necessary! See
    /// [`Tracker::update`] to learn more about this!
    ///
    /// [`Tracker::update`]: subscription::Tracker::update
    pub fn track(
        &mut self,
        subscription: Subscription<Hasher, Event, Message>,
    ) {
        let Runtime {
            executor,
            subscriptions,
            sender,
            ..
        } = self;

        let futures = executor
            .enter(|| subscriptions.update(subscription, sender.clone()));

        for future in futures {
            executor.spawn(future);
        }
    }

    /// Broadcasts an event to all the subscriptions currently alive in the
    /// [`Runtime`].
    ///
    /// See [`Tracker::broadcast`] to learn more.
    ///
    /// [`Tracker::broadcast`]: subscription::Tracker::broadcast
    pub fn broadcast(&mut self, event: Event) {
        self.subscriptions.broadcast(event);
    }
}
