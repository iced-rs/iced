use crate::Subscription;

use futures::{future::BoxFuture, sink::Sink};
use std::collections::HashMap;
use std::marker::PhantomData;

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
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            _hasher: PhantomData,
        }
    }

    pub fn update<Message, S>(
        &mut self,
        subscription: Subscription<Hasher, Event, Message>,
        sink: S,
    ) -> Vec<BoxFuture<'static, ()>>
    where
        Message: 'static + Send,
        S: 'static
            + Sink<Message, Error = core::convert::Infallible>
            + Unpin
            + Send
            + Clone,
    {
        use futures::{future::FutureExt, stream::StreamExt};

        let mut futures = Vec::new();

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
                stream.map(Ok).forward(sink.clone()),
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

            futures.push(future.boxed());
        }

        self.subscriptions.retain(|id, _| alive.contains(&id));

        futures
    }

    pub fn broadcast(&mut self, event: Event) {
        self.subscriptions
            .values_mut()
            .filter_map(|connection| connection.listener.as_mut())
            .for_each(|listener| {
                if let Err(error) = listener.try_send(event.clone()) {
                    log::error!(
                        "Error sending event to subscription: {:?}",
                        error
                    );
                }
            });
    }
}
