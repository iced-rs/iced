use futures::Future;
use futures::channel::mpsc;
use futures::stream::{self, Stream, StreamExt};

pub fn channel<T, F>(f: impl Fn(mpsc::Sender<T>) -> F) -> impl Stream<Item = T>
where
    F: Future<Output = ()>,
{
    let (sender, receiver) = mpsc::channel(1);

    stream::select(
        receiver,
        stream::once(f(sender)).filter_map(|_| async { None }),
    )
}
