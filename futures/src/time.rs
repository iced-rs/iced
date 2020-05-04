//! Listen and react to time.
use crate::subscription::{self, Subscription};

/// Returns a [`Subscription`] that produces messages at a set interval.
///
/// The first message is produced after a `duration`, and then continues to
/// produce more messages every `duration` after that.
///
/// [`Subscription`]: ../subscription/struct.Subscription.html
pub fn every<H: std::hash::Hasher, E>(
    duration: std::time::Duration,
) -> Subscription<H, E, std::time::Instant> {
    Subscription::from_recipe(Every(duration))
}

struct Every(std::time::Duration);

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

#[cfg(all(feature = "tokio", not(feature = "async-std")))]
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

        tokio::time::interval_at(start, self.0)
            .map(|_| std::time::Instant::now())
            .boxed()
    }
}
