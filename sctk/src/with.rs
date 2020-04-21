/// SinkExt::with should be with_mut, it takes FnMut, so it is not Clone.
use iced_native::futures;
use {
    std::{marker::PhantomData, pin::Pin},
    pin_utils::{unsafe_pinned, unsafe_unpinned},
    futures::{sink::Sink, ready, task::{Poll,Context}, future::ready, Future}
};

pub trait SinkExt<Item>: Sink<Item> {
    /// Composes a function *in front of* the sink.
    ///
    /// This adapter produces a new sink that passes each value through the
    /// given function `f` before sending it to `self`.
    ///
    /// To process each value, `f` produces a *future*, which is then polled to
    /// completion before passing its result down to the underlying sink. If the
    /// future produces an error, that error is returned by the new sink.
    ///
    /// Note that this function consumes the given sink, returning a wrapped
    /// version, much like `Iterator::map`.
    fn with<U, Fut, F, E>(self, f: F) -> With<Self, Item, U, Fut, F>
        where F: Fn(U) -> Fut,
              Fut: Future<Output = Result<Item, E>>,
              E: From<Self::Error>,
              Self: Sized
    {
        With::new(self, f)
    }
}

#[derive(Clone)]
pub struct With<Si, Item, U, Fut, F> {
    sink: Si,
    f: F,
    state: Option<Fut>,
    _phantom: PhantomData<fn(U) -> Item>,
}

impl<Si, Item, U, Fut, F> With<Si, Item, U, Fut, F>
where Si: Sink<Item>,
      F: Fn(U) -> Fut,
      Fut: Future,
{
    unsafe_pinned!(sink: Si);
    unsafe_unpinned!(f: F);
    unsafe_pinned!(state: Option<Fut>);

    pub(super) fn new<E>(sink: Si, f: F) -> Self
        where
            Fut: Future<Output = Result<Item, E>>,
            E: From<Si::Error>,
    {
        With {
            state: None,
            sink,
            f,
            _phantom: PhantomData,
        }
    }
}

impl<Si, Item, U, Fut, F, E> Sink<U> for With<Si, Item, U, Fut, F>
    where Si: Sink<Item>,
          F: Fn(U) -> Fut,
          Fut: Future<Output = Result<Item, E>>,
          E: From<Si::Error>,
{
    type Error = E;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll(cx))?;
        ready!(self.as_mut().sink().poll_ready(cx)?);
        Poll::Ready(Ok(()))
    }

    fn start_send(
        mut self: Pin<&mut Self>,
        item: U,
    ) -> Result<(), Self::Error> {
        let future = (self.as_mut().f())(item);
        self.as_mut().state().set(Some(future));
        Ok(())
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll(cx))?;
        ready!(self.as_mut().sink().poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll(cx))?;
        ready!(self.as_mut().sink().poll_close(cx))?;
        Poll::Ready(Ok(()))
    }
}
