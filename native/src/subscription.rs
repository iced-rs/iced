use crate::{Event, Hasher};
use futures::stream::BoxStream;

pub type EventStream = BoxStream<'static, Event>;

pub type Subscription<T> = iced_core::Subscription<Hasher, EventStream, T>;

pub use iced_core::subscription::Recipe;

mod events;

use events::Events;

pub fn events() -> Subscription<Event> {
    Subscription::from_recipe(Events)
}
