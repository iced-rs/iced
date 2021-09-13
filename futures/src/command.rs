/// A set of asynchronous actions to be performed by some runtime.
#[derive(Debug)]
pub struct Command<T>(Internal<T>);

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
    pub const fn single(action: T) -> Self {
        Self(Internal::Single(action))
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
    pub fn map<A>(self, f: impl Fn(T) -> A) -> Command<A>
    where
        T: 'static,
    {
        let Command(command) = self;

        match command {
            Internal::None => Command::none(),
            Internal::Single(action) => Command::single(f(action)),
            Internal::Batch(batch) => {
                Command(Internal::Batch(batch.into_iter().map(f).collect()))
            }
        }
    }

    /// Returns all of the actions of the [`Command`].
    pub fn actions(self) -> Vec<T> {
        let Command(command) = self;

        match command {
            Internal::None => Vec::new(),
            Internal::Single(action) => vec![action],
            Internal::Batch(batch) => batch,
        }
    }
}
