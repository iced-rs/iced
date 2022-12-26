//! Buffer of asynchronous actions.

use std::future::Future;

use iced_futures::MaybeSend;

use crate::command::Command;

/// Send commands to an iced application.
pub trait Commands<T> {
    /// Type of the reborrowed command buffer.
    ///
    /// See [`Commands::by_ref`].
    type ByRef<'this>: Commands<T>
    where
        Self: 'this;

    /// Helper to generically reborrow the command buffer mutably.
    ///
    /// This is useful if you have a function that takes `mut commands: impl
    /// Commands<T>` and you want to use a method such as [Commands::map] which
    /// would otherwise consume the command buffer.
    ///
    /// This can still be done through an expression like `(&mut
    /// commands).map(/*  */)`, but having a method like this reduces the number
    /// of references involves in case the `impl Command<T>` is already a
    /// reference.
    ///
    /// Note that naming is inspired by [`Iterator::by_ref`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use iced_native::Commands;
    /// enum Message {
    ///     Component1(component1::Message),
    ///     Component2(component2::Message),
    /// }
    ///
    /// fn update(mut commands: impl Commands<Message>) {
    ///     component1::update(commands.by_ref().map(Message::Component1));
    ///     component2::update(commands.by_ref().map(Message::Component2));
    /// }
    ///
    /// mod component1 {
    ///     # use iced_native::Commands;
    ///     pub(crate) enum Message {
    ///         Tick,
    ///     }
    ///
    ///     pub(crate) fn update(mut commands: impl Commands<Message>) {
    ///         // emit commands
    ///     }
    /// }
    ///
    /// mod component2 {
    ///     #    use iced_native::Commands;
    ///     pub(crate) enum Message {
    ///         Tick,
    ///     }
    ///
    ///     pub(crate) fn update(mut commands: impl Commands<Message>) {
    ///         // emit commands
    ///     }
    /// }
    /// ```
    ///
    /// Without this method, you'd have to do the following while also
    /// potentially constructing another reference that you don't really need:
    ///
    /// ```
    /// # use iced_native::Commands;
    /// # enum Message { Component1(component1::Message), Component2(component2::Message) }
    /// fn update(mut commands: impl Commands<Message>) {
    ///     component1::update((&mut commands).map(Message::Component1));
    ///     component2::update((&mut commands).map(Message::Component2));
    /// }
    /// # mod component1 {
    /// # use iced_native::Commands;
    /// # pub(crate) enum Message { Tick }
    /// # pub(crate) fn update(mut commands: impl Commands<Message>) { }
    /// # }
    /// # mod component2 {
    /// # use iced_native::Commands;
    /// # pub(crate) enum Message { Tick }
    /// # pub(crate) fn update(mut commands: impl Commands<Message>) { }
    /// # }
    /// ```
    fn by_ref(&mut self) -> Self::ByRef<'_>;

    /// Perform a single asynchronous action and map its output into the
    /// expected message type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use iced_native::Commands;
    /// enum Message {
    ///     Greeting(String),
    /// }
    ///
    /// async fn asynchronous_update() -> String {
    ///     "Hello World".to_string()
    /// }
    ///
    /// fn update(mut commands: impl Commands<Message>) {
    ///     commands.perform(asynchronous_update(), Message::Greeting);
    /// }
    /// ```
    fn perform<F>(
        &mut self,
        future: F,
        map: impl Fn(F::Output) -> T + MaybeSend + Sync + 'static,
    ) where
        F: Future + 'static + MaybeSend;

    /// Insert a command directly into the command buffer.
    ///
    /// This is primarily used for built-in commands such as window messages.
    ///
    /// # Examples
    ///
    /// ```
    /// # // NB: we don't have access to iced here so faking it.
    /// # mod iced { pub(crate) mod window { pub(crate) fn close<Message>() -> iced_native::Command<Message> { todo!() } } }
    /// # use iced_native::Commands;
    /// enum Message {
    ///     /* snip */
    /// }
    ///
    /// fn update(mut commands: impl Commands<Message>) {
    ///     commands.command(iced::window::close());
    /// }
    /// ```
    fn command(&mut self, command: Command<T>);

    /// Extend the current command buffer with an iterator.
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Command<T>>,
    {
        for command in iter {
            self.command(command);
        }
    }

    /// Map the current command buffer so that it can be used with a different
    /// message type `U`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use iced_native::Commands;
    /// enum Message {
    ///     Component1(component1::Message),
    ///     Component2(component2::Message),
    /// }
    ///
    /// fn update(mut commands: impl Commands<Message>) {
    ///     component1::update(commands.by_ref().map(Message::Component1));
    ///     component2::update(commands.by_ref().map(Message::Component2));
    /// }
    ///
    /// mod component1 {
    ///     # use iced_native::Commands;
    ///     pub(crate) enum Message {
    ///         Tick,
    ///     }
    ///
    ///     pub(crate) fn update(mut commands: impl Commands<Message>) {
    ///         // emit commands
    ///     }
    /// }
    ///
    /// mod component2 {
    ///     #    use iced_native::Commands;
    ///     pub(crate) enum Message {
    ///         Tick,
    ///     }
    ///
    ///     pub(crate) fn update(mut commands: impl Commands<Message>) {
    ///         // emit commands
    ///     }
    /// }
    /// ```
    #[inline]
    fn map<M, U>(self, map: M) -> Map<Self, M>
    where
        Self: Sized,
        M: MaybeSend + Sync + Clone + Fn(U) -> T,
    {
        Map {
            commands: self,
            map,
        }
    }
}

/// Wrapper produced by [`Commands::map`].
#[derive(Debug)]
pub struct Map<C, M> {
    commands: C,
    map: M,
}

impl<T: 'static, C, M: 'static, U: 'static> Commands<U> for Map<C, M>
where
    C: Commands<T>,
    M: MaybeSend + Sync + Clone + Fn(U) -> T,
{
    type ByRef<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn by_ref(&mut self) -> Self::ByRef<'_> {
        self
    }

    #[inline]
    fn perform<F>(
        &mut self,
        future: F,
        outer: impl Fn(F::Output) -> U + MaybeSend + Sync + 'static,
    ) where
        F: Future + 'static + MaybeSend,
    {
        let map = self.map.clone();
        self.commands
            .perform(future, move |message| map(outer(message)));
    }

    #[inline]
    fn command(&mut self, command: Command<U>) {
        let map = self.map.clone();
        self.commands
            .command(command.map(move |message| map(message)));
    }
}

impl<C, M> Commands<M> for &mut C
where
    C: Commands<M>,
{
    type ByRef<'this> = C::ByRef<'this> where Self: 'this;

    #[inline]
    fn by_ref(&mut self) -> Self::ByRef<'_> {
        (*self).by_ref()
    }

    #[inline]
    fn perform<F>(
        &mut self,
        future: F,
        map: impl Fn(F::Output) -> M + MaybeSend + Sync + 'static,
    ) where
        F: Future + 'static + MaybeSend,
    {
        (**self).perform(future, map);
    }

    #[inline]
    fn command(&mut self, command: Command<M>) {
        (**self).command(command);
    }
}
