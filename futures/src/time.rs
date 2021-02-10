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

struct Every(std::time::Duration);

#[cfg(all(
    not(any(feature = "tokio_old", feature = "tokio", feature = "async-std")),
    feature = "smol"
))]
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

#[cfg(feature = "async-std")]
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

        async_std::stream::interval(self.0)
            .map(|_| std::time::Instant::now())
            .boxed()
    }
}

#[cfg(all(
    any(feature = "tokio", feature = "tokio_old"),
    not(any(feature = "async-std", feature = "smol"))
))]
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

        #[cfg(feature = "tokio_old")]
        use tokio_old as tokio;

        let start = tokio::time::Instant::now() + self.0;

        let stream = {
            #[cfg(feature = "tokio")]
            {
                futures::stream::unfold(
                    tokio::time::interval_at(start, self.0),
                    |mut interval| async move {
                        Some((interval.tick().await, interval))
                    },
                )
            }
            #[cfg(feature = "tokio_old")]
            tokio::time::interval_at(start, self.0)
        };

        stream.map(tokio::time::Instant::into_std).boxed()
    }
}
