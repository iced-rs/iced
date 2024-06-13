use crate::core::widget;
use crate::futures::futures::channel::mpsc;
use crate::futures::futures::channel::oneshot;
use crate::futures::futures::future::{self, FutureExt};
use crate::futures::futures::never::Never;
use crate::futures::futures::stream::{self, Stream, StreamExt};
use crate::futures::{boxed_stream, BoxStream, MaybeSend};
use crate::Action;

use std::future::Future;

/// A set of concurrent actions to be performed by the iced runtime.
///
/// A [`Task`] _may_ produce a bunch of values of type `T`.
#[allow(missing_debug_implementations)]
pub struct Task<T>(Option<BoxStream<Action<T>>>);

impl<T> Task<T> {
    /// Creates a [`Task`] that does nothing.
    pub fn none() -> Self {
        Self(None)
    }

    /// Creates a new [`Task`] that instantly produces the given value.
    pub fn done(value: T) -> Self
    where
        T: MaybeSend + 'static,
    {
        Self::future(future::ready(value))
    }

    /// Creates a new [`Task`] that runs the given [`Future`] and produces
    /// its output.
    pub fn future(future: impl Future<Output = T> + MaybeSend + 'static) -> Self
    where
        T: 'static,
    {
        Self::stream(stream::once(future))
    }

    /// Creates a new [`Task`] that runs the given [`Stream`] and produces
    /// each of its items.
    pub fn stream(stream: impl Stream<Item = T> + MaybeSend + 'static) -> Self
    where
        T: 'static,
    {
        Self(Some(boxed_stream(stream.map(Action::Output))))
    }

    /// Creates a new [`Task`] that runs the given [`widget::Operation`] and produces
    /// its output.
    pub fn widget(operation: impl widget::Operation<T> + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        Self::channel(move |sender| {
            let operation =
                widget::operation::map(Box::new(operation), move |value| {
                    let _ = sender.clone().try_send(value);
                });

            Action::Widget(Box::new(operation))
        })
    }

    /// Creates a new [`Task`] that executes the [`Action`] returned by the closure and
    /// produces the value fed to the [`oneshot::Sender`].
    pub fn oneshot(f: impl FnOnce(oneshot::Sender<T>) -> Action<T>) -> Task<T>
    where
        T: MaybeSend + 'static,
    {
        let (sender, receiver) = oneshot::channel();

        let action = f(sender);

        Self(Some(boxed_stream(
            stream::once(async move { action }).chain(
                receiver.into_stream().filter_map(|result| async move {
                    Some(Action::Output(result.ok()?))
                }),
            ),
        )))
    }

    /// Creates a new [`Task`] that executes the [`Action`] returned by the closure and
    /// produces the values fed to the [`mpsc::Sender`].
    pub fn channel(f: impl FnOnce(mpsc::Sender<T>) -> Action<T>) -> Task<T>
    where
        T: MaybeSend + 'static,
    {
        let (sender, receiver) = mpsc::channel(1);

        let action = f(sender);

        Self(Some(boxed_stream(
            stream::once(async move { action })
                .chain(receiver.map(|result| Action::Output(result))),
        )))
    }

    /// Creates a new [`Task`] that executes the given [`Action`] and produces no output.
    pub fn effect(action: impl Into<Action<Never>>) -> Self {
        let action = action.into();

        Self(Some(boxed_stream(stream::once(async move {
            action.output().expect_err("no output")
        }))))
    }

    /// Maps the output of a [`Task`] with the given closure.
    pub fn map<O>(
        self,
        mut f: impl FnMut(T) -> O + MaybeSend + 'static,
    ) -> Task<O>
    where
        T: MaybeSend + 'static,
        O: MaybeSend + 'static,
    {
        self.then(move |output| Task::done(f(output)))
    }

    /// Performs a new [`Task`] for every output of the current [`Task`] using the
    /// given closure.
    ///
    /// This is the monadic interface of [`Task`]—analogous to [`Future`] and
    /// [`Stream`].
    pub fn then<O>(
        self,
        mut f: impl FnMut(T) -> Task<O> + MaybeSend + 'static,
    ) -> Task<O>
    where
        T: MaybeSend + 'static,
        O: MaybeSend + 'static,
    {
        Task(match self.0 {
            None => None,
            Some(stream) => {
                Some(boxed_stream(stream.flat_map(move |action| {
                    match action.output() {
                        Ok(output) => f(output)
                            .0
                            .unwrap_or_else(|| boxed_stream(stream::empty())),
                        Err(action) => {
                            boxed_stream(stream::once(async move { action }))
                        }
                    }
                })))
            }
        })
    }

    /// Chains a new [`Task`] to be performed once the current one finishes completely.
    pub fn chain(self, task: Self) -> Self
    where
        T: 'static,
    {
        match self.0 {
            None => task,
            Some(first) => match task.0 {
                None => Task::none(),
                Some(second) => Task(Some(boxed_stream(first.chain(second)))),
            },
        }
    }

    /// Creates a [`Task`] that runs the given [`Future`] to completion.
    pub fn perform<A>(
        future: impl Future<Output = A> + MaybeSend + 'static,
        f: impl Fn(A) -> T + MaybeSend + 'static,
    ) -> Self
    where
        T: MaybeSend + 'static,
        A: MaybeSend + 'static,
    {
        Self::future(future.map(f))
    }

    /// Creates a [`Task`] that runs the given [`Stream`] to completion.
    pub fn run<A>(
        stream: impl Stream<Item = A> + MaybeSend + 'static,
        f: impl Fn(A) -> T + 'static + MaybeSend,
    ) -> Self
    where
        T: 'static,
    {
        Self::stream(stream.map(f))
    }

    /// Combines the given tasks and produces a single [`Task`] that will run all of them
    /// in parallel.
    pub fn batch(tasks: impl IntoIterator<Item = Self>) -> Self
    where
        T: 'static,
    {
        Self(Some(boxed_stream(stream::select_all(
            tasks.into_iter().filter_map(|task| task.0),
        ))))
    }

    /// Returns the underlying [`Stream`] of the [`Task`].
    pub fn into_stream(self) -> Option<BoxStream<Action<T>>> {
        self.0
    }
}

impl<T> From<()> for Task<T>
where
    T: MaybeSend + 'static,
{
    fn from(_value: ()) -> Self {
        Self::none()
    }
}
