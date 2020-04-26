use crate::BoxFuture;
use futures::future::{Future, FutureExt};

/// A collection of async operations.
///
/// You should be able to turn a future easily into a [`Command`], either by
/// using the `From` trait or [`Command::perform`].
///
/// [`Command`]: struct.Command.html
/// [`Command::perform`]: #method.perform
pub struct Command<T> {
    futures: Vec<BoxFuture<T>>,
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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn perform<A>(
        future: impl Future<Output = T> + 'static + Send,
        f: impl Fn(T) -> A + 'static + Send,
    ) -> Command<A> {
        Command {
            futures: vec![Box::pin(future.map(f))],
        }
    }

    /// Creates a [`Command`] that performs the action of the given future.
    ///
    /// [`Command`]: struct.Command.html
    #[cfg(target_arch = "wasm32")]
    pub fn perform<A>(
        future: impl Future<Output = T> + 'static,
        f: impl Fn(T) -> A + 'static + Send,
    ) -> Command<A> {
        Command {
            futures: vec![Box::pin(future.map(f))],
        }
    }

    /// Applies a transformation to the result of a [`Command`].
    ///
    /// [`Command`]: struct.Command.html
    #[cfg(not(target_arch = "wasm32"))]
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

                    Box::pin(future.map(move |result| f(result)))
                        as BoxFuture<A>
                })
                .collect(),
        }
    }

    /// Applies a transformation to the result of a [`Command`].
    ///
    /// [`Command`]: struct.Command.html
    #[cfg(target_arch = "wasm32")]
    pub fn map<A>(mut self, f: impl Fn(T) -> A + 'static) -> Command<A>
    where
        T: 'static,
    {
        let f = std::rc::Rc::new(f);

        Command {
            futures: self
                .futures
                .drain(..)
                .map(|future| {
                    let f = f.clone();

                    Box::pin(future.map(move |result| f(result)))
                        as BoxFuture<A>
                })
                .collect(),
        }
    }

    /// Creates a [`Command`] that performs the actions of all the given
    /// commands.
    ///
    /// Once this command is run, all the commands will be executed at once.
    ///
    /// [`Command`]: struct.Command.html
    pub fn batch(commands: impl IntoIterator<Item = Command<T>>) -> Self {
        Self {
            futures: commands
                .into_iter()
                .flat_map(|command| command.futures)
                .collect(),
        }
    }

    /// Converts a [`Command`] into its underlying list of futures.
    ///
    /// [`Command`]: struct.Command.html
    pub fn futures(self) -> Vec<BoxFuture<T>> {
        self.futures
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
impl<T, A> From<A> for Command<T>
where
    A: Future<Output = T> + 'static,
{
    fn from(future: A) -> Self {
        Self {
            futures: vec![future.boxed_local()],
        }
    }
}

impl<T> std::fmt::Debug for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").finish()
    }
}
