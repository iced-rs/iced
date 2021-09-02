mod action;

pub use action::Action;

#[cfg(target_arch = "wasm32")]
use std::future::Future;

/// A set of asynchronous actions to be performed by some runtime.
pub enum Command<T> {
    None,
    Single(Action<T>),
    Batch(Vec<Action<T>>),
}

impl<T> Command<T> {
    /// Creates an empty [`Command`].
    ///
    /// In other words, a [`Command`] that does nothing.
    pub fn none() -> Self {
        Self::None
    }

    /// Creates a [`Command`] that performs the action of the given future.
    #[cfg(target_arch = "wasm32")]
    pub fn perform<A>(
        future: impl Future<Output = T> + 'static,
        f: impl Fn(T) -> A + 'static + Send,
    ) -> Command<A> {
        use iced_futures::futures::FutureExt;

        Command::Single(Action::Future(Box::pin(future.map(f))))
    }

    /// Applies a transformation to the result of a [`Command`].
    #[cfg(target_arch = "wasm32")]
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static + Clone) -> Command<A>
    where
        T: 'static,
    {
        match self {
            Self::None => Command::None,
            Self::Single(action) => Command::Single(action.map(f)),
            Self::Batch(batch) => Command::Batch(
                batch
                    .into_iter()
                    .map(|action| action.map(f.clone()))
                    .collect(),
            ),
        }
    }

    /// Creates a [`Command`] that performs the actions of all the given
    /// commands.
    ///
    /// Once this command is run, all the commands will be executed at once.
    pub fn batch(commands: impl IntoIterator<Item = Command<T>>) -> Self {
        let mut batch = Vec::new();

        for command in commands {
            match command {
                Self::None => {}
                Self::Single(command) => batch.push(command),
                Self::Batch(commands) => batch.extend(commands),
            }
        }

        Self::Batch(batch)
    }

    pub fn actions(self) -> Vec<Action<T>> {
        match self {
            Self::None => Vec::new(),
            Self::Single(action) => vec![action],
            Self::Batch(batch) => batch,
        }
    }
}
