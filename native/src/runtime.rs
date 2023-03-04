//! Run commands and subscriptions.
use iced_core::event::{self, Event};
use iced_core::Hasher;

/// A native runtime with a generic executor and receiver of results.
///
/// It can be used by shells to easily spawn a [`Command`] or track a
/// [`Subscription`].
///
/// [`Command`]: crate::Command
/// [`Subscription`]: crate::Subscription
pub type Runtime<Executor, Receiver, Message> = iced_futures::Runtime<
    Hasher,
    (Event, event::Status),
    Executor,
    Receiver,
    Message,
>;
