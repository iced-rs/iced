use std::fmt;
use std::future::Future;

use iced_futures::futures::FutureExt;
use iced_futures::MaybeSend;

use crate::{Command, Commands};

/// A command buffer used for an application.
pub struct LocalCommands<M> {
    futures: Vec<iced_futures::BoxFuture<M>>,
    commands: Vec<Command<M>>,
}

impl<M> LocalCommands<M> {
    /// Drain futures from command buffer.
    #[inline]
    pub fn futures(
        &mut self,
    ) -> impl Iterator<Item = iced_futures::BoxFuture<M>> + '_ {
        self.futures.drain(..)
    }

    /// Drain commands from this command buffer.
    #[inline]
    pub fn commands(&mut self) -> impl Iterator<Item = Command<M>> + '_ {
        self.commands.drain(..)
    }
}

impl<M> Commands<M> for LocalCommands<M> {
    type ByRef<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn by_ref(&mut self) -> Self::ByRef<'_> {
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
        self.futures.push(Box::pin(future.map(map)));
    }

    #[inline]
    fn command(&mut self, command: Command<M>) {
        self.commands.push(command);
    }
}

impl<M> Default for LocalCommands<M> {
    #[inline]
    fn default() -> Self {
        Self {
            futures: Vec::new(),
            commands: Vec::new(),
        }
    }
}

impl<M> fmt::Debug for LocalCommands<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalCommands")
            .field("commands", &self.commands)
            .finish_non_exhaustive()
    }
}
