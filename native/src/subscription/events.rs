use crate::{
    subscription::{EventStream, Recipe},
    Event, Hasher,
};
use iced_futures::futures::stream::BoxStream;

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
    ) -> BoxStream<'static, Self::Output> {
        event_stream
    }
}
