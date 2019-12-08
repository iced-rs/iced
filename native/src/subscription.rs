use crate::Event;

pub type Subscription<T> = iced_core::Subscription<Input, T>;
pub type Input = futures::channel::mpsc::Receiver<Event>;

pub use iced_core::subscription::Connection;
