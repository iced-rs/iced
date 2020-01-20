//! Run commands and keep track of subscriptions.
use crate::{subscription, Command, Executor, Subscription};

use futures::Sink;
use std::marker::PhantomData;

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
    Sender: Sink<Message, Error = core::convert::Infallible>
        + Unpin
        + Send
        + Clone
        + 'static,
    Message: Send + 'static,
{
    pub fn new(executor: Executor, sender: Sender) -> Self {
        Self {
            executor,
            sender,
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
            let mut sender = self.sender.clone();

            self.executor.spawn(future.then(|message| {
                async move {
                    let _ = sender.send(message).await;

                    ()
                }
            }));
        }
    }

    pub fn track(
        &mut self,
        subscription: Subscription<Hasher, Event, Message>,
    ) {
        let futures =
            self.subscriptions.update(subscription, self.sender.clone());

        for future in futures {
            self.executor.spawn(future);
        }
    }

    pub fn broadcast(&mut self, event: Event) {
        self.subscriptions.broadcast(event);
    }
}
