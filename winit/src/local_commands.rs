use std::future::Future;

use iced_futures::futures::FutureExt;
use iced_futures::MaybeSend;

use crate::command::Action;
use crate::Commands;

/// A command buffer used for an application.
#[derive(Debug)]
pub struct LocalCommands<M> {
    buf: Vec<Action<M>>,
}

impl<M> LocalCommands<M> {
    /// Drain actions from this command buffer.
    #[inline]
    pub fn actions(&mut self) -> impl Iterator<Item = Action<M>> + '_ {
        self.buf.drain(..)
    }
}

impl<M> Commands<M> for LocalCommands<M> {
    #[inline]
    fn perform<F>(
        &mut self,
        future: F,
        map: impl Fn(F::Output) -> M + MaybeSend + 'static,
    ) where
        F: Future + 'static + MaybeSend,
    {
        self.buf.push(Action::Future(Box::pin(future.map(map))));
    }

    fn command(&mut self, command: crate::Command<M>) {
        for action in command.actions() {
            self.buf.push(action);
        }
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
