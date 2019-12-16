use crate::{
    subscription::{EventStream, Recipe},
    Event, Hasher,
};

pub struct Events;

impl Recipe<Hasher, EventStream> for Events {
    type Output = Event;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        event_stream: EventStream,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        event_stream
    }
}
