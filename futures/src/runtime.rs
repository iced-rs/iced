//! Run commands and keep track of subscriptions.
use crate::{subscription, Command, Executor, Subscription};

use futures::{channel::mpsc, Sink};
use std::marker::PhantomData;

/// A batteries-included runtime of commands and subscriptions.
///
/// If you have an [`Executor`], a [`Runtime`] can be leveraged to run any
/// [`Command`] or [`Subscription`] and get notified of the results!
///
/// [`Runtime`]: struct.Runtime.html
/// [`Executor`]: executor/trait.Executor.html
/// [`Command`]: struct.Command.html
/// [`Subscription`]: subscription/struct.Subscription.html
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
    ///
    /// [`Runtime`]: struct.Runtime.html
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
    ///
    /// [`Executor`]: executor/trait.Executor.html
    /// [`Runtime`]: struct.Runtime.html
    /// [`Executor::enter`]: executor/trait.Executor.html#method.enter
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        self.executor.enter(f)
    }

    /// Spawns a [`Command`] in the [`Runtime`].
    ///
    /// The resulting `Message` will be forwarded to the `Sender` of the
    /// [`Runtime`].
    ///
    /// [`Command`]: struct.Command.html
    /// [`Runtime`]: struct.Runtime.html
    pub fn spawn(&mut self, command: Command<Message>) {
        use futures::{FutureExt, SinkExt};

        let futures = command.futures();

        for future in futures {
            let mut sender = self.sender.clone();

            let future = future.then(|message| async move {
                let _ = sender.send(message).await;

                ()
            });

            self.executor.spawn(future);
        }
    }

    /// Tracks a [`Subscription`] in the [`Runtime`].
    ///
    /// It will spawn new streams or close old ones as necessary! See
    /// [`Tracker::update`] to learn more about this!
    ///
    /// [`Subscription`]: subscription/struct.Subscription.html
    /// [`Runtime`]: struct.Runtime.html
    /// [`Tracker::update`]: subscription/struct.Tracker.html#method.update
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
    /// [`Runtime`]: struct.Runtime.html
    /// [`Tracker::broadcast`]:
    /// subscription/struct.Tracker.html#method.broadcast
    pub fn broadcast(&mut self, event: Event) {
        self.subscriptions.broadcast(event);
    }
}
