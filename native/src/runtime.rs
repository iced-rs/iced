//! Run commands and subscriptions.
use crate::event::{self, Event};
use crate::Hasher;

/// A native runtime with a generic executor and receiver of results.
///
/// It can be used by shells to easily spawn a [`Command`] or track a
/// [`Subscription`].
///
/// [`Command`]: ../struct.Command.html
/// [`Subscription`]: ../struct.Subscription.html
pub type Runtime<Executor, Receiver, Message> = iced_futures::Runtime<
    Hasher,
    (Event, event::Status),
    Executor,
    Receiver,
    Message,
>;
