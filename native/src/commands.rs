//! Buffer of asynchronous actions.

use std::future::Future;

use iced_futures::MaybeSend;

use crate::command::Command;

/// Send commands to an iced application.
pub trait Commands<M> {
    /// Perform a single asynchronous action and map its output into the message
    /// type `M`.
    fn perform<F>(
        &mut self,
        future: F,
        f: impl Fn(F::Output) -> M + MaybeSend + Sync + 'static,
    ) where
        F: Future + 'static + MaybeSend;

    /// Insert a command into the command buffer.
    fn command(&mut self, command: Command<M>);

    /// Extend the current command buffer with an iterator.
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Command<M>>,
    {
        for command in iter {
            self.command(command);
        }
    }

    /// Map the current command buffer so that it can be used with a different
    /// message type.
    #[inline]
    fn map<F>(self, f: F) -> Map<Self, F, M>
    where
        Self: Sized,
    {
        Map {
            inner: self,
            map: f,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Output of [CommandBuf::map].
#[derive(Debug)]
pub struct Map<C, MapFn, M> {
    inner: C,
    map: MapFn,
    _marker: std::marker::PhantomData<M>,
}

impl<M: 'static, C, MapFn: 'static, U: 'static> Commands<M> for Map<C, MapFn, M>
where
    C: Commands<U>,
    MapFn: MaybeSend + Sync + Clone + Fn(M) -> U,
{
    #[inline]
    fn perform<F>(
        &mut self,
        future: F,
        f: impl Fn(F::Output) -> M + MaybeSend + Sync + 'static,
    ) where
        F: Future + 'static + MaybeSend,
    {
        let map = self.map.clone();
        self.inner.perform(future, move |m| map(f(m)));
    }

    #[inline]
    fn command(&mut self, command: Command<M>) {
        let map = self.map.clone();
        self.inner.command(command.map(move |m| map(m)));
    }
}

impl<C, M> Commands<M> for &mut C
where
    C: Commands<M>,
{
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
