use crate::{Event, Hasher};

pub type Subscription<T> = iced_core::Subscription<Hasher, Input, T>;
pub type Input = futures::channel::mpsc::Receiver<Event>;

pub use iced_core::subscription::Recipe;

pub fn events() -> Subscription<Event> {
    Subscription::from_recipe(Events)
}

struct Events;

impl Recipe<Hasher, Input> for Events {
    type Output = Event;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        input: Input,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use futures::StreamExt;

        input.boxed()
    }
}
