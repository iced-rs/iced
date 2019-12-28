use futures::future::{BoxFuture, Future, FutureExt};

/// A collection of async operations.
///
/// You should be able to turn a future easily into a [`Command`], either by
/// using the `From` trait or [`Command::perform`].
///
/// [`Command`]: struct.Command.html
pub struct Command<T> {
    futures: Vec<BoxFuture<'static, T>>,
}

impl<T> Command<T> {
    /// Creates an empty [`Command`].
    ///
    /// In other words, a [`Command`] that does nothing.
    ///
    /// [`Command`]: struct.Command.html
    pub fn none() -> Self {
        Self {
            futures: Vec::new(),
        }
    }

    /// Creates a [`Command`] that performs the action of the given future.
    ///
    /// [`Command`]: struct.Command.html
    pub fn perform<A>(
        future: impl Future<Output = T> + 'static + Send,
        f: impl Fn(T) -> A + 'static + Send,
    ) -> Command<A> {
        Command {
            futures: vec![future.map(f).boxed()],
        }
    }

    /// Applies a transformation to the result of a [`Command`].
    ///
    /// [`Command`]: struct.Command.html
    pub fn map<A>(
        mut self,
        f: impl Fn(T) -> A + 'static + Send + Sync,
    ) -> Command<A>
    where
        T: 'static,
    {
        let f = std::sync::Arc::new(f);

        Command {
            futures: self
                .futures
                .drain(..)
                .map(|future| {
                    let f = f.clone();

                    future.map(move |result| f(result)).boxed()
                })
                .collect(),
        }
    }

    /// Creates a [`Command`] that performs the actions of all the given
    /// commands.
    ///
    /// Once this command is run, all the commands will be exectued at once.
    ///
    /// [`Command`]: struct.Command.html
    pub fn batch(commands: impl Iterator<Item = Command<T>>) -> Self {
        Self {
            futures: commands.flat_map(|command| command.futures).collect(),
        }
    }

    /// Converts a [`Command`] into its underlying list of futures.
    ///
    /// [`Command`]: struct.Command.html
    pub fn futures(self) -> Vec<BoxFuture<'static, T>> {
        self.futures
    }
}

impl<T, A> From<A> for Command<T>
where
    A: Future<Output = T> + 'static + Send,
{
    fn from(future: A) -> Self {
        Self {
            futures: vec![future.boxed()],
        }
    }
}

impl<T> std::fmt::Debug for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").finish()
    }
}
