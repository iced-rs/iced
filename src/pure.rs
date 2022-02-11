//! Leverage pure, virtual widgets in your application.
//!
//! The widgets found in this module are completely stateless versions of
//! [the original widgets].
//!
//! Effectively, this means that, as a user of the library, you do not need to
//! keep track of the local state of each widget (e.g. [`button::State`]).
//! Instead, the runtime will keep track of everything for you!
//!
//! You can embed pure widgets anywhere in your [impure `Application`] using the
//! [`Pure`] widget and some [`State`].
//!
//! In case you want to only use pure widgets in your application, this module
//! offers an alternate [`Application`] trait with a completely pure `view`
//! method.
//!
//! [the original widgets]: crate::widget
//! [`button::State`]: crate::widget::button::State
//! [impure `Application`]: crate::Application
pub use iced_pure::{Element as _, *};

/// A generic, pure [`Widget`].
pub type Element<Message> = iced_pure::Element<Message, crate::Renderer>;

mod application;
mod sandbox;

pub use application::Application;
pub use sandbox::Sandbox;
