//! Run asynchronous actions.
mod action;

pub use action::Action;

use crate::core::widget;
use crate::futures::futures;
use crate::futures::MaybeSend;

use futures::channel::mpsc;
use futures::Stream;
use std::fmt;
use std::future::Future;

/// A set of asynchronous actions to be performed by some runtime.
#[must_use = "`Command` must be returned to runtime to take effect"]
pub struct Command<T>(Internal<Action<T>>);

#[derive(Debug)]
enum Internal<T> {
    None,
    Single(T),
    Batch(Vec<T>),
}

impl<T> Command<T> {
    /// Creates an empty [`Command`].
    ///
    /// In other words, a [`Command`] that does nothing.
    pub const fn none() -> Self {
        Self(Internal::None)
    }

    /// Creates a [`Command`] that performs a single [`Action`].
    pub const fn single(action: Action<T>) -> Self {
        Self(Internal::Single(action))
    }

    /// Creates a [`Command`] that performs a [`widget::Operation`].
    pub fn widget(operation: impl widget::Operation<T> + 'static) -> Self {
        Self::single(Action::Widget(Box::new(operation)))
    }

    /// Creates a [`Command`] that performs the action of the given future.
    pub fn perform<A>(
        future: impl Future<Output = A> + 'static + MaybeSend,
        f: impl FnOnce(A) -> T + 'static + MaybeSend,
    ) -> Command<T> {
        use futures::FutureExt;

        Command::single(Action::Future(Box::pin(future.map(f))))
    }

    /// Creates a [`Command`] that runs the given stream to completion.
    pub fn run<A>(
        stream: impl Stream<Item = A> + 'static + MaybeSend,
        f: impl Fn(A) -> T + 'static + MaybeSend,
    ) -> Command<T> {
        use futures::StreamExt;

        Command::single(Action::Stream(Box::pin(stream.map(f))))
    }

    /// Creates a [`Command`] that performs the actions of all the given
    /// commands.
    ///
    /// Once this command is run, all the commands will be executed at once.
    pub fn batch(commands: impl IntoIterator<Item = Command<T>>) -> Self {
        let mut batch = Vec::new();

        for Command(command) in commands {
            match command {
                Internal::None => {}
                Internal::Single(command) => batch.push(command),
                Internal::Batch(commands) => batch.extend(commands),
            }
        }

        Self(Internal::Batch(batch))
    }

    /// Applies a transformation to the result of a [`Command`].
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync + Clone,
    ) -> Command<A>
    where
        T: 'static,
        A: 'static,
    {
        match self.0 {
            Internal::None => Command::none(),
            Internal::Single(action) => Command::single(action.map(f)),
            Internal::Batch(batch) => Command(Internal::Batch(
                batch
                    .into_iter()
                    .map(|action| action.map(f.clone()))
                    .collect(),
            )),
        }
    }

    /// Returns all of the actions of the [`Command`].
    pub fn actions(self) -> Vec<Action<T>> {
        let Command(command) = self;

        match command {
            Internal::None => Vec::new(),
            Internal::Single(action) => vec![action],
            Internal::Batch(batch) => batch,
        }
    }
}

impl<T> fmt::Debug for Command<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Command(command) = self;

        command.fmt(f)
    }
}

/// Creates a [`Command`] that produces the `Message`s published from a [`Future`]
/// to an [`mpsc::Sender`] with the given bounds.
pub fn channel<Fut, Message>(
    size: usize,
    f: impl FnOnce(mpsc::Sender<Message>) -> Fut + MaybeSend + 'static,
) -> Command<Message>
where
    Fut: Future<Output = ()> + MaybeSend + 'static,
    Message: 'static + MaybeSend,
{
    use futures::future;
    use futures::stream::{self, StreamExt};

    let (sender, receiver) = mpsc::channel(size);

    let runner = stream::once(f(sender)).filter_map(|_| future::ready(None));

    Command::single(Action::Stream(Box::pin(stream::select(receiver, runner))))
}
