//! A `tokio` backend.
use futures::Future;

/// A `tokio` executor.
pub type Executor = tokio::runtime::Runtime;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        tokio::runtime::Runtime::new()
    }

    #[allow(clippy::let_underscore_future)]
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = tokio::runtime::Runtime::spawn(self, future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = tokio::runtime::Runtime::enter(self);
        f()
    }
}

pub mod time {
    //! Listen and react to time.
    use crate::subscription::{self, Subscription};

    /// Returns a [`Subscription`] that produces messages at a set interval.
    ///
    /// The first message is produced after a `duration`, and then continues to
    /// produce more messages every `duration` after that.
    pub fn every<H: std::hash::Hasher, E>(
        duration: std::time::Duration,
    ) -> Subscription<H, E, std::time::Instant> {
        Subscription::from_recipe(Every(duration))
    }

    #[derive(Debug)]
    struct Every(std::time::Duration);

    impl<H, E> subscription::Recipe<H, E> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, E>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            let start = tokio::time::Instant::now() + self.0;

            let stream = {
                futures::stream::unfold(
                    tokio::time::interval_at(start, self.0),
                    |mut interval| async move {
                        Some((interval.tick().await, interval))
                    },
                )
            };

            stream.map(tokio::time::Instant::into_std).boxed()
        }
    }
}
