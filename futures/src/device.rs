//! Listen to device events.

use crate::MaybeSend;
use crate::core::device::Event;
use crate::subscription::{self, Subscription};

/// Listen to all device events with a filter function.
///
/// The filter function receives raw device events and returns `Option<Message>`.
/// Return `None` for events you don't care about to avoid producing messages.
///
/// # Performance Warning
///
/// Device events fire at very high frequency! Always filter and return
/// `None` for events you don't need.
pub fn listen<Message>(f: fn(Event) -> Option<Message>) -> Subscription<Message>
where
    Message: MaybeSend + 'static,
{
    #[derive(Hash)]
    struct Listen;

    subscription::filter_map((Listen, f), move |event| match event {
        subscription::Event::Device { event, .. } => f(event),
        _ => None,
    })
}
