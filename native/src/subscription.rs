use crate::{Event, Hasher};

pub type Subscription<T> = iced_core::Subscription<Hasher, Input, T>;
pub type Input = futures::channel::mpsc::Receiver<Event>;

pub use iced_core::subscription::Recipe;
