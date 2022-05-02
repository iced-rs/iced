//! A `smol` backend.

use futures::Future;

/// A `smol` executor.
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
#[derive(Debug)]
pub struct Executor;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        smol::spawn(future).detach();
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

            smol::Timer::interval(self.0).boxed()
        }
    }
}
