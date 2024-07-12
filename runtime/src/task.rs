//! Create runtime tasks.
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

    /// Creates a [`Task`] that runs the given [`Future`] to completion and maps its
    /// output with the given closure.
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

    /// Creates a [`Task`] that runs the given [`Stream`] to completion and maps each
    /// item with the given closure.
    pub fn run<A>(
        stream: impl Stream<Item = A> + MaybeSend + 'static,
        f: impl Fn(A) -> T + MaybeSend + 'static,
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

    /// Creates a new [`Task`] that collects all the output of the current one into a [`Vec`].
    pub fn collect(self) -> Task<Vec<T>>
    where
        T: MaybeSend + 'static,
    {
        match self.0 {
            None => Task::done(Vec::new()),
            Some(stream) => Task(Some(boxed_stream(
                stream::unfold(
                    (stream, Some(Vec::new())),
                    move |(mut stream, outputs)| async move {
                        let mut outputs = outputs?;

                        let Some(action) = stream.next().await else {
                            return Some((
                                Some(Action::Output(outputs)),
                                (stream, None),
                            ));
                        };

                        match action.output() {
                            Ok(output) => {
                                outputs.push(output);

                                Some((None, (stream, Some(outputs))))
                            }
                            Err(action) => {
                                Some((Some(action), (stream, Some(outputs))))
                            }
                        }
                    },
                )
                .filter_map(future::ready),
            ))),
        }
    }

    /// Creates a new [`Task`] that can be aborted with the returned [`Handle`].
    pub fn abortable(self) -> (Self, Handle)
    where
        T: 'static,
    {
        match self.0 {
            Some(stream) => {
                let (stream, handle) = stream::abortable(stream);

                (
                    Self(Some(boxed_stream(stream))),
                    Handle {
                        raw: Some(handle),
                        abort_on_drop: false,
                    },
                )
            }
            None => (
                Self(None),
                Handle {
                    raw: None,
                    abort_on_drop: false,
                },
            ),
        }
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
}

/// A handle to a [`Task`] that can be used for aborting it.
#[derive(Debug, Clone)]
pub struct Handle {
    raw: Option<stream::AbortHandle>,
    abort_on_drop: bool,
}

impl Handle {
    /// Aborts the [`Task`] of this [`Handle`].
    pub fn abort(&self) {
        if let Some(handle) = &self.raw {
            handle.abort();
        }
    }

    /// Returns a new [`Handle`] that will call [`Handle::abort`] whenever
    /// it is dropped.
    ///
    /// This can be really useful if you do not want to worry about calling
    /// [`Handle::abort`] yourself.
    pub fn abort_on_drop(mut self) -> Self {
        Self {
            raw: self.raw.take(),
            abort_on_drop: true,
        }
    }

    /// Returns `true` if the [`Task`] of this [`Handle`] has been aborted.
    pub fn is_aborted(&self) -> bool {
        if let Some(handle) = &self.raw {
            handle.is_aborted()
        } else {
            true
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if self.abort_on_drop {
            self.abort();
        }
    }
}

impl<T> Task<Option<T>> {
    /// Executes a new [`Task`] after this one, only when it produces `Some` value.
    ///
    /// The value is provided to the closure to create the subsequent [`Task`].
    pub fn and_then<A>(
        self,
        f: impl Fn(T) -> Task<A> + MaybeSend + 'static,
    ) -> Task<A>
    where
        T: MaybeSend + 'static,
        A: MaybeSend + 'static,
    {
        self.then(move |option| option.map_or_else(Task::none, &f))
    }
}

impl<T, E> Task<Result<T, E>> {
    /// Executes a new [`Task`] after this one, only when it succeeds with an `Ok` value.
    ///
    /// The success value is provided to the closure to create the subsequent [`Task`].
    pub fn and_then<A>(
        self,
        f: impl Fn(T) -> Task<A> + MaybeSend + 'static,
    ) -> Task<A>
    where
        T: MaybeSend + 'static,
        E: MaybeSend + 'static,
        A: MaybeSend + 'static,
    {
        self.then(move |option| option.map_or_else(|_| Task::none(), &f))
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

/// Creates a new [`Task`] that runs the given [`widget::Operation`] and produces
/// its output.
pub fn widget<T>(operation: impl widget::Operation<T> + 'static) -> Task<T>
where
    T: Send + 'static,
{
    channel(move |sender| {
        let operation =
            widget::operation::map(Box::new(operation), move |value| {
                let _ = sender.clone().try_send(value);
            });

        Action::Widget(Box::new(operation))
    })
}

/// Creates a new [`Task`] that executes the [`Action`] returned by the closure and
/// produces the value fed to the [`oneshot::Sender`].
pub fn oneshot<T>(f: impl FnOnce(oneshot::Sender<T>) -> Action<T>) -> Task<T>
where
    T: MaybeSend + 'static,
{
    let (sender, receiver) = oneshot::channel();

    let action = f(sender);

    Task(Some(boxed_stream(
        stream::once(async move { action }).chain(
            receiver.into_stream().filter_map(|result| async move {
                Some(Action::Output(result.ok()?))
            }),
        ),
    )))
}

/// Creates a new [`Task`] that executes the [`Action`] returned by the closure and
/// produces the values fed to the [`mpsc::Sender`].
pub fn channel<T>(f: impl FnOnce(mpsc::Sender<T>) -> Action<T>) -> Task<T>
where
    T: MaybeSend + 'static,
{
    let (sender, receiver) = mpsc::channel(1);

    let action = f(sender);

    Task(Some(boxed_stream(
        stream::once(async move { action })
            .chain(receiver.map(|result| Action::Output(result))),
    )))
}

/// Creates a new [`Task`] that executes the given [`Action`] and produces no output.
pub fn effect<T>(action: impl Into<Action<Never>>) -> Task<T> {
    let action = action.into();

    Task(Some(boxed_stream(stream::once(async move {
        action.output().expect_err("no output")
    }))))
}

/// Returns the underlying [`Stream`] of the [`Task`].
pub fn into_stream<T>(task: Task<T>) -> Option<BoxStream<Action<T>>> {
    task.0
}
