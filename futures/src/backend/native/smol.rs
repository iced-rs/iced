//! A `smol` backend.

/// A `smol` executor.
#[derive(Debug)]
pub struct Executor;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        smol::spawn(future).detach();
    }

    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        smol::block_on(future)
    }
}

pub mod time {
    //! Listen and react to time.
    use crate::MaybeSend;
    use crate::core::time::{Duration, Instant};
    use crate::subscription::Subscription;

    use futures::stream;

    /// Returns a [`Subscription`] that produces messages at a set interval.
    ///
    /// The first message is produced after a `duration`, and then continues to
    /// produce more messages every `duration` after that.
    pub fn every(duration: Duration) -> Subscription<Instant> {
        Subscription::run_with(duration, |duration| {
            use futures::stream::StreamExt;

            let start = Instant::now() + *duration;

            smol::Timer::interval_at(start, *duration).boxed()
        })
    }

    /// Returns a [`Subscription`] that runs the given async function at a
    /// set interval; producing the result of the function as output.
    pub fn repeat<F, T>(f: fn() -> F, interval: Duration) -> Subscription<T>
    where
        F: Future<Output = T> + MaybeSend + 'static,
        T: MaybeSend + 'static,
    {
        Subscription::run_with((f, interval), |(f, interval)| {
            let f = *f;
            let interval = *interval;

            stream::unfold(0, move |i| async move {
                if i > 0 {
                    _ = smol::Timer::after(interval).await;
                }

                Some((f().await, i + 1))
            })
        })
    }
}
