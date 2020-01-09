use iced_native::{Event, Hasher, Subscription};
use std::collections::HashMap;

pub struct Pool {
    alive: HashMap<u64, Handle>,
}

pub struct Handle {
    _cancel: futures::channel::oneshot::Sender<()>,
    listener: Option<futures::channel::mpsc::Sender<Event>>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            alive: HashMap::new(),
        }
    }

    pub fn update<Message: std::fmt::Debug + Send>(
        &mut self,
        subscription: Subscription<Message>,
        thread_pool: &mut futures::executor::ThreadPool,
        proxy: &winit::event_loop::EventLoopProxy<Message>,
    ) {
        use futures::{future::FutureExt, stream::StreamExt};

        let recipes = subscription.recipes();
        let mut alive = std::collections::HashSet::new();

        for recipe in recipes {
            let id = {
                use std::hash::Hasher as _;

                let mut hasher = Hasher::default();
                recipe.hash(&mut hasher);

                hasher.finish()
            };

            let _ = alive.insert(id);

            if !self.alive.contains_key(&id) {
                let (cancel, cancelled) = futures::channel::oneshot::channel();

                // TODO: Use bus if/when it supports async
                let (event_sender, event_receiver) =
                    futures::channel::mpsc::channel(100);

                let stream = recipe.stream(event_receiver.boxed());
                let proxy = proxy.clone();

                let future = futures::future::select(
                    cancelled,
                    stream.for_each(move |message| {
                        proxy
                            .send_event(message)
                            .expect("Send subscription result to event loop");

                        futures::future::ready(())
                    }),
                )
                .map(|_| ());

                thread_pool.spawn_ok(future);

                let _ = self.alive.insert(
                    id,
                    Handle {
                        _cancel: cancel,
                        listener: if event_sender.is_closed() {
                            None
                        } else {
                            Some(event_sender)
                        },
                    },
                );
            }
        }

        self.alive.retain(|id, _| alive.contains(&id));
    }

    pub fn broadcast_event(&mut self, event: Event) {
        self.alive
            .values_mut()
            .filter_map(|connection| connection.listener.as_mut())
            .for_each(|listener| {
                if let Err(error) = listener.try_send(event) {
                    log::error!(
                        "Error sending event to subscription: {:?}",
                        error
                    );
                }
            });
    }
}
