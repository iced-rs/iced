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
//! # The Elm Architecture, purity, and continuity
//! As you may know, applications made with `iced` use [The Elm Architecture].
//!
//! In a nutshell, this architecture defines the initial state of the application, a way to `view` it, and a way to `update` it after a user interaction. The `update` logic is called after a meaningful user interaction, which in turn updates the state of the application. Then, the `view` logic is executed to redisplay the application.
//!
//! Since `view` logic is only run after an `update`, all of the mutations to the application state must only happen in the `update` logic. If the application state changes anywhere else, the `view` logic will not be rerun and, therefore, the previously generated `view` may stay outdated.
//!
//! However, the `Application` trait in `iced` defines `view` as:
//!
//! ```ignore
//! pub trait Application {
//!     fn view(&mut self) -> Element<Self::Message>;
//! }
//! ```
//!
//! As a consequence, the application state can be mutated in `view` logic. The `view` logic in `iced` is __impure__.
//!
//! This impurity is necessary because `iced` puts the burden of widget __continuity__ on its users. In other words, it's up to you to provide `iced` with the internal state of each widget every time `view` is called.
//!
//! If we take a look at the classic `counter` example:
//!
//! ```ignore
//! struct Counter {
//!     value: i32,
//!     increment_button: button::State,
//!     decrement_button: button::State,
//! }
//!
//! // ...
//!
//! impl Counter {
//!     pub fn view(&mut self) -> Column<Message> {
//!         Column::new()
//!             .push(
//!                 Button::new(&mut self.increment_button, Text::new("+"))
//!                     .on_press(Message::IncrementPressed),
//!             )
//!             .push(Text::new(self.value.to_string()).size(50))
//!             .push(
//!                 Button::new(&mut self.decrement_button, Text::new("-"))
//!                     .on_press(Message::DecrementPressed),
//!             )
//!     }
//! }
//! ```
//!
//! We can see how we need to keep track of the `button::State` of each `Button` in our `Counter` state and provide a mutable reference to the widgets in our `view` logic. The widgets produced by `view` are __stateful__.
//!
//! While this approach forces users to keep track of widget state and causes impurity, I originally chose it because it allows `iced` to directly consume the widget tree produced by `view`. Since there is no internal state decoupled from `view` maintained by the runtime, `iced` does not need to compare (e.g. reconciliate) widget trees in order to ensure continuity.
//!
//! # Stateless widgets
//! As the library matures, the need for some kind of persistent widget data (see #553) between `view` calls becomes more apparent (e.g. incremental rendering, animations, accessibility, etc.).
//!
//! If we are going to end up having persistent widget data anyways... There is no reason to have impure, stateful widgets anymore!
//!
//! With the help of this module, we can now write a pure `counter` example:
//!
//! ```ignore
//! struct Counter {
//!     value: i32,
//! }
//!
//! // ...
//!
//! impl Counter {
//!     fn view(&self) -> Column<Message> {
//!         Column::new()
//!             .push(Button::new("Increment").on_press(Message::IncrementPressed))
//!             .push(Text::new(self.value.to_string()).size(50))
//!             .push(Button::new("Decrement").on_press(Message::DecrementPressed))
//!     }
//! }
//! ```
//!
//! Notice how we no longer need to keep track of the `button::State`! The widgets in `iced_pure` do not take any mutable application state in `view`. They are __stateless__ widgets. As a consequence, we do not need mutable access to `self` in `view` anymore. `view` becomes __pure__.
//!
//! [The Elm Architecture]: https://guide.elm-lang.org/architecture/
//!
//! [the original widgets]: crate::widget
//! [`button::State`]: crate::widget::button::State
//! [impure `Application`]: crate::Application
pub mod widget;

mod application;
mod sandbox;

pub use application::Application;
pub use sandbox::Sandbox;

pub use iced_pure::helpers::*;
pub use iced_pure::Widget;
pub use iced_pure::{Pure, State};

/// A generic, pure [`Widget`].
pub type Element<'a, Message> =
    iced_pure::Element<'a, Message, crate::Renderer>;
