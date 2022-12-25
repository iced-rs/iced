use std::future::Future;

use iced_futures::futures::FutureExt;
use iced_futures::MaybeSend;

use crate::{Command, Commands};

/// A command buffer used for an application.
#[derive(Debug)]
pub struct LocalCommands<M> {
    buf: Vec<Command<M>>,
}

impl<M> LocalCommands<M> {
    /// Drain commands from this command buffer.
    #[inline]
    pub fn commands(&mut self) -> impl Iterator<Item = Command<M>> + '_ {
        self.buf.drain(..)
    }
}

impl<M> Commands<M> for LocalCommands<M> {
    type AsMut<'a> = &'a mut Self where Self: 'a;

    #[inline]
    fn as_mut(&mut self) -> Self::AsMut<'_> {
        self
    }

    #[inline]
    fn perform<F>(
        &mut self,
        future: F,
        map: impl Fn(F::Output) -> M + MaybeSend + 'static,
    ) where
        F: Future + 'static + MaybeSend,
    {
        self.buf.push(Command::Future(Box::pin(future.map(map))));
    }

    #[inline]
    fn command(&mut self, command: Command<M>) {
        self.buf.push(command);
    }
}

impl<M> Default for LocalCommands<M> {
    #[inline]
    fn default() -> Self {
        Self {
            buf: Vec::default(),
        }
    }
}
