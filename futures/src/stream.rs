//! Create asynchronous streams of data.
use futures::channel::mpsc;
use futures::stream::{self, Stream, StreamExt};

use std::future::Future;

/// Creates a new [`Stream`] that produces the items sent from a [`Future`]
/// to the [`mpsc::Sender`] provided to the closure.
///
/// This is a more ergonomic [`stream::unfold`], which allows you to go
/// from the "world of futures" to the "world of streams" by simply looping
/// and publishing to an async channel from inside a [`Future`].
pub fn channel<T, F>(
    size: usize,
    f: impl FnOnce(mpsc::Sender<T>) -> F,
) -> impl Stream<Item = T>
where
    F: Future<Output = ()>,
{
    let (sender, receiver) = mpsc::channel(size);

    let runner = stream::once(f(sender)).filter_map(|_| async { None });

    stream::select(receiver, runner)
}

/// Creates a new [`Stream`] that produces the items sent from a [`Future`]
/// that can fail to the [`mpsc::Sender`] provided to the closure.
pub fn try_channel<T, E, F>(
    size: usize,
    f: impl FnOnce(mpsc::Sender<T>) -> F,
) -> impl Stream<Item = Result<T, E>>
where
    F: Future<Output = Result<(), E>>,
{
    let (sender, receiver) = mpsc::channel(size);

    let runner = stream::once(f(sender)).filter_map(|result| async {
        match result {
            Ok(()) => None,
            Err(error) => Some(Err(error)),
        }
    });

    stream::select(receiver.map(Ok), runner)
}
