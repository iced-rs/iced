//! A message paired with its `Clone` implementation, captured where the bound is known.

/// Lets a widget reproduce a stored message on every event without
/// requiring a `Message: Clone` bound on its [`Widget`](crate::core::Widget) implementation.
pub struct Cloneable<Message> {
    message: Message,
    clone: fn(&Message) -> Message,
}

impl<Message> Cloneable<Message> {
    /// Captures a `message` and its [`Clone`] implementation.
    pub fn new(message: Message) -> Self
    where
        Message: Clone,
    {
        Self {
            message,
            clone: Message::clone,
        }
    }

    /// Reproduces the captured message.
    pub fn get(&self) -> Message {
        (self.clone)(&self.message)
    }
}

/// A message handler that is either a stored message reproduced with [`Clone`],
/// or a closure invoked to lazily produce a fresh message without it.
pub enum Emit<'a, Message> {
    Direct(Cloneable<Message>),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

impl<'a, Message> Emit<'a, Message> {
    /// Captures a `message` that will be reproduced with [`Clone`] every time it is emitted.
    pub fn direct(message: Message) -> Self
    where
        Message: Clone,
    {
        Self::Direct(Cloneable::new(message))
    }

    /// Captures a closure that produces a fresh message every time it is called,
    /// bypassing the need for `Message: Clone`.
    pub fn closure(f: impl Fn() -> Message + 'a) -> Self {
        Self::Closure(Box::new(f))
    }

    /// Produces the message.
    pub fn get(&self) -> Message {
        match self {
            Self::Direct(message) => message.get(),
            Self::Closure(f) => f(),
        }
    }
}
