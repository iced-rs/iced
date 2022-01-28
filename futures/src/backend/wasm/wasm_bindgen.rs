//! A `wasm-bindgein-futures` backend.

/// A `wasm-bindgen-futures` executor.
#[derive(Debug)]
pub struct Executor;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl futures::Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
}

pub mod time {
    //! Listen and react to time.
    use crate::subscription::{self, Subscription};
    use crate::BoxStream;

    /// Returns a [`Subscription`] that produces messages at a set interval.
    ///
    /// The first message is produced after a `duration`, and then continues to
    /// produce more messages every `duration` after that.
    pub fn every<H: std::hash::Hasher, E>(
        duration: std::time::Duration,
    ) -> Subscription<H, E, wasm_timer::Instant> {
        Subscription::from_recipe(Every(duration))
    }

    #[derive(Debug)]
    struct Every(std::time::Duration);

    impl<H, E> subscription::Recipe<H, E> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = wasm_timer::Instant;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: BoxStream<E>,
        ) -> BoxStream<Self::Output> {
            use futures::stream::StreamExt;

            wasm_timer::Interval::new(self.0)
                .map(|_| wasm_timer::Instant::now())
                .boxed_local()
        }
    }
}
