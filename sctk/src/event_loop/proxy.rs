use iced_native::futures::{
    channel::mpsc,
    task::{Context, Poll},
    Sink,
};
use sctk::reexports::calloop;
use std::pin::Pin;

/// An event loop proxy that implements `Sink`.
#[derive(Debug)]
pub struct Proxy<Message: 'static> {
    raw: calloop::channel::Sender<Message>,
}

impl<Message: 'static> Clone for Proxy<Message> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
        }
    }
}

impl<Message: 'static> Proxy<Message> {
    /// Creates a new [`Proxy`] from an `EventLoopProxy`.
    pub fn new(raw: calloop::channel::Sender<Message>) -> Self {
        Self { raw }
    }
    /// send an event
    pub fn send_event(&self, message: Message) {
        let _ = self.raw.send(message);
    }
}

impl<Message: 'static> Sink<Message> for Proxy<Message> {
    type Error = mpsc::SendError;

    fn poll_ready(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(
        self: Pin<&mut Self>,
        message: Message,
    ) -> Result<(), Self::Error> {
        let _ = self.raw.send(message);

        Ok(())
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
