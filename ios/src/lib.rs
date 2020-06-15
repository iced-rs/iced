<<<<<<< HEAD
#[macro_use] extern crate objc;
=======
>>>>>>> 781da4d53cd84f58dab486eff1bf4f46fdf3347c
pub use iced_futures::{executor, futures, Command};

#[doc(no_inline)]
pub use executor::Executor;
mod application;
pub mod widget;
pub mod keyboard;
pub mod mouse;
pub use widget::{
    Element, Widget, Text,
};
mod proxy;
mod bus;
use proxy::Proxy;
mod layout;
pub use layout::Layout;
pub use application::Application;
/*
//! Run commands and subscriptions.
use crate::{Event, Hasher};

/// A native runtime with a generic executor and receiver of results.
///
/// It can be used by shells to easily spawn a [`Command`] or track a
/// [`Subscription`].
///
/// [`Command`]: ../struct.Command.html
/// [`Subscription`]: ../struct.Subscription.html
pub type Runtime<Executor, Receiver, Message> =
    iced_futures::Runtime<Hasher, Event, Executor, Receiver, Message>;
*/

#[derive(PartialEq, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum iOSEvent {
    Test,
}

pub type Runtime<Executor, Receiver, Message> =
    iced_futures::Runtime<
    std::collections::hash_map::DefaultHasher,
    iOSEvent, Executor, Receiver, Message>;

pub type Subscription<T> = iced_futures::Subscription<
    std::collections::hash_map::DefaultHasher,
    iOSEvent,
    Event, Executor, Receiver, Message>;
    T,
>;

pub use iced_core::{
    Align, Background, Color, Font, HorizontalAlignment, Length, Point,
    Rectangle, Size, Vector, VerticalAlignment,
};
