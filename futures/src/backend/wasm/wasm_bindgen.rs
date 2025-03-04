//! A `wasm-bindgen-futures` backend.

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
    use crate::subscription::Subscription;

    use wasmtimer::std::Instant;

    /// Returns a [`Subscription`] that produces messages at a set interval.
    ///
    /// The first message is produced after a `duration`, and then continues to
    /// produce more messages every `duration` after that.
    pub fn every(duration: std::time::Duration) -> Subscription<Instant> {
        Subscription::run_with(duration, |duration| {
            use futures::stream::StreamExt;

            let mut interval = wasmtimer::tokio::interval(*duration);
            interval.set_missed_tick_behavior(
                wasmtimer::tokio::MissedTickBehavior::Skip,
            );

            let stream = {
                futures::stream::unfold(interval, |mut interval| async move {
                    Some((interval.tick().await, interval))
                })
            };

            stream.boxed()
        })
    }
}
