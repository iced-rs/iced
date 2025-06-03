//! Create runtime tasks.
use crate::Action;
use crate::core::widget;
use crate::futures::futures::channel::mpsc;
use crate::futures::futures::channel::oneshot;
use crate::futures::futures::future::{self, FutureExt};
use crate::futures::futures::stream::{self, Stream, StreamExt};
use crate::futures::{BoxStream, MaybeSend, boxed_stream};

use std::convert::Infallible;
use std::sync::Arc;

#[cfg(feature = "sipper")]
#[doc(no_inline)]
pub use sipper::{Never, Sender, Sipper, Straw, sipper, stream};

/// A set of concurrent actions to be performed by the iced runtime.
///
/// A [`Task`] _may_ produce a bunch of values of type `T`.
#[must_use = "`Task` must be returned to the runtime to take effect; normally in your `update` or `new` functions."]
pub struct Task<T> {
    stream: Option<BoxStream<Action<T>>>,
    units: usize,
}

impl<T> Task<T> {
    /// Creates a [`Task`] that does nothing.
    pub fn none() -> Self {
        Self {
            stream: None,
            units: 0,
        }
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
        f: impl FnOnce(A) -> T + MaybeSend + 'static,
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

    /// Creates a [`Task`] that runs the given [`Sipper`] to completion, mapping
    /// progress with the first closure and the output with the second one.
    #[cfg(feature = "sipper")]
    pub fn sip<S>(
        sipper: S,
        on_progress: impl FnMut(S::Progress) -> T + MaybeSend + 'static,
        on_output: impl FnOnce(<S as Future>::Output) -> T + MaybeSend + 'static,
    ) -> Self
    where
        S: sipper::Core + MaybeSend + 'static,
        T: MaybeSend + 'static,
    {
        Self::stream(stream(sipper::sipper(move |sender| async move {
            on_output(sipper.with(on_progress).run(sender).await)
        })))
    }

    /// Combines the given tasks and produces a single [`Task`] that will run all of them
    /// in parallel.
    pub fn batch(tasks: impl IntoIterator<Item = Self>) -> Self
    where
        T: 'static,
    {
        let mut select_all = stream::SelectAll::new();
        let mut units = 0;

        for task in tasks.into_iter() {
            if let Some(stream) = task.stream {
                select_all.push(stream);
            }

            units += task.units;
        }

        Self {
            stream: Some(boxed_stream(select_all)),
            units,
        }
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
        Task {
            stream: match self.stream {
                None => None,
                Some(stream) => {
                    Some(boxed_stream(stream.flat_map(move |action| {
                        match action.output() {
                            Ok(output) => {
                                f(output).stream.unwrap_or_else(|| {
                                    boxed_stream(stream::empty())
                                })
                            }
                            Err(action) => boxed_stream(stream::once(
                                async move { action },
                            )),
                        }
                    })))
                }
            },
            units: self.units,
        }
    }

    /// Chains a new [`Task`] to be performed once the current one finishes completely.
    pub fn chain(self, task: Self) -> Self
    where
        T: 'static,
    {
        match self.stream {
            None => task,
            Some(first) => match task.stream {
                None => Self {
                    stream: Some(first),
                    units: self.units,
                },
                Some(second) => Self {
                    stream: Some(boxed_stream(first.chain(second))),
                    units: self.units + task.units,
                },
            },
        }
    }

    /// Creates a new [`Task`] that collects all the output of the current one into a [`Vec`].
    pub fn collect(self) -> Task<Vec<T>>
    where
        T: MaybeSend + 'static,
    {
        match self.stream {
            None => Task::done(Vec::new()),
            Some(stream) => Task {
                stream: Some(boxed_stream(
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
                                Err(action) => Some((
                                    Some(action),
                                    (stream, Some(outputs)),
                                )),
                            }
                        },
                    )
                    .filter_map(future::ready),
                )),
                units: self.units,
            },
        }
    }

    /// Creates a new [`Task`] that discards the result of the current one.
    ///
    /// Useful if you only care about the side effects of a [`Task`].
    pub fn discard<O>(self) -> Task<O>
    where
        T: MaybeSend + 'static,
        O: MaybeSend + 'static,
    {
        self.then(|_| Task::none())
    }

    /// Creates a new [`Task`] that can be aborted with the returned [`Handle`].
    pub fn abortable(self) -> (Self, Handle)
    where
        T: 'static,
    {
        let (stream, handle) = match self.stream {
            Some(stream) => {
                let (stream, handle) = stream::abortable(stream);

                (Some(boxed_stream(stream)), InternalHandle::Manual(handle))
            }
            None => (
                None,
                InternalHandle::Manual(stream::AbortHandle::new_pair().0),
            ),
        };

        (
            Self {
                stream,
                units: self.units,
            },
            Handle { internal: handle },
        )
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
        Self {
            stream: Some(boxed_stream(stream.map(Action::Output))),
            units: 1,
        }
    }

    /// Returns the amount of work "units" of the [`Task`].
    pub fn units(&self) -> usize {
        self.units
    }
}

impl<T> std::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("Task<{}>", std::any::type_name::<T>()))
            .field("units", &self.units)
            .finish()
    }
}

/// A handle to a [`Task`] that can be used for aborting it.
#[derive(Debug, Clone)]
pub struct Handle {
    internal: InternalHandle,
}

#[derive(Debug, Clone)]
enum InternalHandle {
    Manual(stream::AbortHandle),
    AbortOnDrop(Arc<stream::AbortHandle>),
}

impl InternalHandle {
    pub fn as_ref(&self) -> &stream::AbortHandle {
        match self {
            InternalHandle::Manual(handle) => handle,
            InternalHandle::AbortOnDrop(handle) => handle.as_ref(),
        }
    }
}

impl Handle {
    /// Aborts the [`Task`] of this [`Handle`].
    pub fn abort(&self) {
        self.internal.as_ref().abort();
    }

    /// Returns a new [`Handle`] that will call [`Handle::abort`] whenever
    /// all of its instances are dropped.
    ///
    /// If a [`Handle`] is cloned, [`Handle::abort`] will only be called
    /// once all of the clones are dropped.
    ///
    /// This can be really useful if you do not want to worry about calling
    /// [`Handle::abort`] yourself.
    pub fn abort_on_drop(self) -> Self {
        match &self.internal {
            InternalHandle::Manual(handle) => Self {
                internal: InternalHandle::AbortOnDrop(Arc::new(handle.clone())),
            },
            InternalHandle::AbortOnDrop(_) => self,
        }
    }

    /// Returns `true` if the [`Task`] of this [`Handle`] has been aborted.
    pub fn is_aborted(&self) -> bool {
        self.internal.as_ref().is_aborted()
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if let InternalHandle::AbortOnDrop(handle) = &mut self.internal {
            let handle = std::mem::replace(
                handle,
                Arc::new(stream::AbortHandle::new_pair().0),
            );

            if let Some(handle) = Arc::into_inner(handle) {
                handle.abort();
            }
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

impl<T> From<()> for Task<T> {
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

    Task {
        stream: Some(boxed_stream(stream::once(async move { action }).chain(
            receiver.into_stream().filter_map(|result| async move {
                Some(Action::Output(result.ok()?))
            }),
        ))),
        units: 1,
    }
}

/// Creates a new [`Task`] that executes the [`Action`] returned by the closure and
/// produces the values fed to the [`mpsc::Sender`].
pub fn channel<T>(f: impl FnOnce(mpsc::Sender<T>) -> Action<T>) -> Task<T>
where
    T: MaybeSend + 'static,
{
    let (sender, receiver) = mpsc::channel(1);

    let action = f(sender);

    Task {
        stream: Some(boxed_stream(
            stream::once(async move { action })
                .chain(receiver.map(|result| Action::Output(result))),
        )),
        units: 1,
    }
}

/// Creates a new [`Task`] that executes the given [`Action`] and produces no output.
pub fn effect<T>(action: impl Into<Action<Infallible>>) -> Task<T> {
    let action = action.into();

    Task {
        stream: Some(boxed_stream(stream::once(async move {
            action.output().expect_err("no output")
        }))),
        units: 1,
    }
}

/// Returns the underlying [`Stream`] of the [`Task`].
pub fn into_stream<T>(task: Task<T>) -> Option<BoxStream<Action<T>>> {
    task.stream
}
