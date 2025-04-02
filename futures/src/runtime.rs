//! Run commands and keep track of subscriptions.
use crate::subscription;
use crate::{BoxStream, Executor, MaybeSend};

use futures::{Sink, channel::mpsc};
use std::marker::PhantomData;

/// A batteries-included runtime of commands and subscriptions.
///
/// If you have an [`Executor`], a [`Runtime`] can be leveraged to run any
/// `Command` or [`Subscription`] and get notified of the results!
///
/// [`Subscription`]: crate::Subscription
#[derive(Debug)]
pub struct Runtime<Executor, Sender, Message> {
    executor: Executor,
    sender: Sender,
    subscriptions: subscription::Tracker,
    _message: PhantomData<Message>,
}

impl<Executor, Sender, Message> Runtime<Executor, Sender, Message>
where
    Executor: self::Executor,
    Sender: Sink<Message, Error = mpsc::SendError>
        + Unpin
        + MaybeSend
        + Clone
        + 'static,
    Message: MaybeSend + 'static,
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

    /// Runs a future to completion in the current thread within the [`Runtime`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn block_on<T>(&mut self, future: impl Future<Output = T>) -> T {
        self.executor.block_on(future)
    }

    /// Runs a [`Stream`] in the [`Runtime`] until completion.
    ///
    /// The resulting `Message`s will be forwarded to the `Sender` of the
    /// [`Runtime`].
    ///
    /// [`Stream`]: BoxStream
    pub fn run(&mut self, stream: BoxStream<Message>) {
        use futures::{FutureExt, StreamExt};

        let sender = self.sender.clone();
        let future =
            stream.map(Ok).forward(sender).map(|result| match result {
                Ok(()) => (),
                Err(error) => {
                    log::warn!(
                        "Stream could not run until completion: {error}"
                    );
                }
            });

        self.executor.spawn(future);
    }

    /// Tracks a [`Subscription`] in the [`Runtime`].
    ///
    /// It will spawn new streams or close old ones as necessary! See
    /// [`Tracker::update`] to learn more about this!
    ///
    /// [`Tracker::update`]: subscription::Tracker::update
    /// [`Subscription`]: crate::Subscription
    pub fn track(
        &mut self,
        recipes: impl IntoIterator<
            Item = Box<dyn subscription::Recipe<Output = Message>>,
        >,
    ) {
        let Runtime {
            executor,
            subscriptions,
            sender,
            ..
        } = self;

        let futures = executor.enter(|| {
            subscriptions.update(recipes.into_iter(), sender.clone())
        });

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
    pub fn broadcast(&mut self, event: subscription::Event) {
        self.subscriptions.broadcast(event);
    }
}
