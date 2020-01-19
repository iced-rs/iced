//! Run commands and subscriptions.
mod executor;

pub use executor::Executor;

use crate::{subscription, Command, Subscription};

use futures::Sink;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Runtime<Hasher, Event, Executor, Receiver, Message> {
    executor: Executor,
    receiver: Receiver,
    subscriptions: subscription::Tracker<Hasher, Event>,
    _message: PhantomData<Message>,
}

impl<Hasher, Event, Executor, Receiver, Message>
    Runtime<Hasher, Event, Executor, Receiver, Message>
where
    Hasher: std::hash::Hasher + Default,
    Event: Send + Clone + 'static,
    Executor: self::Executor,
    Receiver: Sink<Message, Error = core::convert::Infallible>
        + Unpin
        + Send
        + Clone
        + 'static,
    Message: Send + 'static,
{
    pub fn new(executor: Executor, receiver: Receiver) -> Self {
        Self {
            executor,
            receiver,
            subscriptions: subscription::Tracker::new(),
            _message: PhantomData,
        }
    }

    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        self.executor.enter(f)
    }

    pub fn spawn(&mut self, command: Command<Message>) {
        use futures::{FutureExt, SinkExt};

        let futures = command.futures();

        for future in futures {
            let mut receiver = self.receiver.clone();

            self.executor.spawn(future.then(|message| {
                async move {
                    let _ = receiver.send(message).await;

                    ()
                }
            }));
        }
    }

    pub fn track(
        &mut self,
        subscription: Subscription<Hasher, Event, Message>,
    ) {
        let futures = self
            .subscriptions
            .update(subscription, self.receiver.clone());

        for future in futures {
            self.executor.spawn(future);
        }
    }

    pub fn broadcast(&mut self, event: Event) {
        self.subscriptions.broadcast(event);
    }
}
