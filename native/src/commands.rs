//! Buffer of asynchronous actions.

use std::future::Future;

use iced_futures::MaybeSend;

use crate::command::Command;

/// Send commands to an iced application.
pub trait Commands<T> {
    /// Type of the reborrowed command buffer.
    ///
    /// See [Commands::as_mut].
    type AsMut<'a>: Commands<T>
    where
        Self: 'a;

    /// Helper to generically reborrow the command buffer.
    ///
    /// This is useful if you have a function that takes `mut commands: impl
    /// Commands<T>` and you want to use a method such as [Commands::map] which
    /// would otherwise consume the command buffer.
    ///
    /// This can still be done through an expression like `(&mut
    /// commands).map(/*  */)`, but having a method like this reduces the number
    /// of references involves in case the `impl Command<T>` is already a
    /// reference.
    fn as_mut(&mut self) -> Self::AsMut<'_>;

    /// Perform a single asynchronous action and map its output into the message
    /// type `M`.
    fn perform<F>(
        &mut self,
        future: F,
        map: impl Fn(F::Output) -> T + MaybeSend + Sync + 'static,
    ) where
        F: Future + 'static + MaybeSend;

    /// Insert a command into the command buffer.
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

/// Output of [CommandBuf::map].
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
    type AsMut<'a> = &'a mut Self where Self: 'a;

    #[inline]
    fn as_mut(&mut self) -> Self::AsMut<'_> {
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
    type AsMut<'a> = &'a mut C where Self: 'a;

    #[inline]
    fn as_mut(&mut self) -> Self::AsMut<'_> {
        *self
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
