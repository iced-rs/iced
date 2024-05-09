use crate::futures::futures::{
    channel::mpsc,
    select,
    task::{Context, Poll},
    Future, Sink, StreamExt,
};
use std::pin::Pin;

/// An event loop proxy with backpressure that implements `Sink`.
#[derive(Debug)]
pub struct Proxy<Message: 'static> {
    raw: winit::event_loop::EventLoopProxy<Message>,
    sender: mpsc::Sender<Message>,
    notifier: mpsc::Sender<usize>,
}

impl<Message: 'static> Clone for Proxy<Message> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            sender: self.sender.clone(),
            notifier: self.notifier.clone(),
        }
    }
}

impl<Message: 'static> Proxy<Message> {
    const MAX_SIZE: usize = 100;

    /// Creates a new [`Proxy`] from an `EventLoopProxy`.
    pub fn new(
        raw: winit::event_loop::EventLoopProxy<Message>,
    ) -> (Self, impl Future<Output = ()>) {
        let (notifier, mut processed) = mpsc::channel(Self::MAX_SIZE);
        let (sender, mut receiver) = mpsc::channel(Self::MAX_SIZE);
        let proxy = raw.clone();

        let worker = async move {
            let mut count = 0;

            loop {
                if count < Self::MAX_SIZE {
                    select! {
                        message = receiver.select_next_some() => {
                            let _ = proxy.send_event(message);
                            count += 1;

                        }
                        amount = processed.select_next_some() => {
                            count = count.saturating_sub(amount);
                        }
                        complete => break,
                    }
                } else {
                    select! {
                        amount = processed.select_next_some() => {
                            count = count.saturating_sub(amount);
                        }
                        complete => break,
                    }
                }
            }
        };

        (
            Self {
                raw,
                sender,
                notifier,
            },
            worker,
        )
    }

    /// Sends a `Message` to the event loop.
    ///
    /// Note: This skips the backpressure mechanism with an unbounded
    /// channel. Use sparingly!
    pub fn send(&mut self, message: Message)
    where
        Message: std::fmt::Debug,
    {
        self.raw
            .send_event(message)
            .expect("Send message to event loop");
    }

    /// Frees an amount of slots for additional messages to be queued in
    /// this [`Proxy`].
    pub fn free_slots(&mut self, amount: usize) {
        let _ = self.notifier.start_send(amount);
    }
}

impl<Message: 'static> Sink<Message> for Proxy<Message> {
    type Error = mpsc::SendError;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.sender.poll_ready(cx)
    }

    fn start_send(
        mut self: Pin<&mut Self>,
        message: Message,
    ) -> Result<(), Self::Error> {
        self.sender.start_send(message)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        match self.sender.poll_ready(cx) {
            Poll::Ready(Err(ref e)) if e.is_disconnected() => {
                // If the receiver disconnected, we consider the sink to be flushed.
                Poll::Ready(Ok(()))
            }
            x => x,
        }
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.sender.disconnect();
        Poll::Ready(Ok(()))
    }
}
