mod action;

pub use action::Action;

use std::fmt;

#[cfg(target_arch = "wasm32")]
use std::future::Future;

/// A set of asynchronous actions to be performed by some runtime.
pub struct Command<T>(iced_futures::Command<Action<T>>);

impl<T> Command<T> {
    /// Creates an empty [`Command`].
    ///
    /// In other words, a [`Command`] that does nothing.
    pub const fn none() -> Self {
        Self(iced_futures::Command::none())
    }

    /// Creates a [`Command`] that performs a single [`Action`].
    pub const fn single(action: Action<T>) -> Self {
        Self(iced_futures::Command::single(action))
    }

    /// Creates a [`Command`] that performs the action of the given future.
    #[cfg(target_arch = "wasm32")]
    pub fn perform<A>(
        future: impl Future<Output = T> + 'static,
        f: impl Fn(T) -> A + 'static + Send,
    ) -> Command<A> {
        use iced_futures::futures::FutureExt;

        Command::single(Action::Future(Box::pin(future.map(f))))
    }

    /// Creates a [`Command`] that performs the actions of all the given
    /// commands.
    ///
    /// Once this command is run, all the commands will be executed at once.
    pub fn batch(commands: impl IntoIterator<Item = Command<T>>) -> Self {
        Self(iced_futures::Command::batch(
            commands.into_iter().map(|Command(command)| command),
        ))
    }

    /// Applies a transformation to the result of a [`Command`].
    #[cfg(target_arch = "wasm32")]
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static + Clone) -> Command<A>
    where
        T: 'static,
    {
        let Command(command) = self;

        Command(command.map(move |action| action.map(f.clone())))
    }

    /// Returns all of the actions of the [`Command`].
    pub fn actions(self) -> Vec<Action<T>> {
        let Command(command) = self;

        command.actions()
    }
}

impl<T> fmt::Debug for Command<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Command(command) = self;

        command.fmt(f)
    }
}
