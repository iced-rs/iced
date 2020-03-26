use crate::{
    subscription::{EventStream, Recipe},
    Event, Hasher,
};
use iced_futures::BoxStream;

pub struct Events;

impl Recipe<Hasher, Event> for Events {
    type Output = Event;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        event_stream: EventStream,
    ) -> BoxStream<Self::Output> {
        event_stream
    }
}
