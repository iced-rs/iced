//! Iced is a renderer-agnostic GUI library focused on simplicity and
//! type-safety. Inspired by [Elm].
//!
//! # Features
//!   * Simple, easy-to-use, renderer-agnostic API
//!   * Responsive, flexbox-based layouting
//!   * Type-safe, reactive programming model
//!   * Built-in widgets
//!   * Custom widget support
//!
//! Check out the [repository] and the [examples] for more details!
//!
//! [examples]: https://github.com/hecrj/iced/tree/0.1.0/examples
//! [repository]: https://github.com/hecrj/iced
//!
//! # Usage
//! Inspired by [The Elm Architecture], Iced expects you to split user interfaces
//! into four different concepts:
//!
//!   * __State__ — the state of your application
//!   * __Messages__ — user interactions or meaningful events that you care
//!   about
//!   * __View logic__ — a way to display your __state__ as widgets that
//!   may produce __messages__ on user interaction
//!   * __Update logic__ — a way to react to __messages__ and update your
//!   __state__
//!
//! We can build something to see how this works! Let's say we want a simple counter
//! that can be incremented and decremented using two buttons.
//!
//! We start by modelling the __state__ of our application:
//!
//! ```
//! use iced_native::button;
//!
//! struct Counter {
//!     // The counter value
//!     value: i32,
//!
//!     // The local state of the two buttons
//!     increment_button: button::State,
//!     decrement_button: button::State,
//! }
//! ```
//!
//! Next, we need to define the possible user interactions of our counter:
//! the button presses. These interactions are our __messages__:
//!
//! ```
//! #[derive(Debug, Clone, Copy)]
//! pub enum Message {
//!     IncrementPressed,
//!     DecrementPressed,
//! }
//! ```
//!
//! Now, let's show the actual counter by putting it all together in our
//! __view logic__:
//!
//! ```
//! # use iced_native::button;
//! #
//! # struct Counter {
//! #     // The counter value
//! #     value: i32,
//! #
//! #     // The local state of the two buttons
//! #     increment_button: button::State,
//! #     decrement_button: button::State,
//! # }
//! #
//! # #[derive(Debug, Clone, Copy)]
//! # pub enum Message {
//! #     IncrementPressed,
//! #     DecrementPressed,
//! # }
//! #
//! # mod iced_wgpu {
//! #     use iced_native::{
//! #         button, text, Button, Text, Node, Point, Rectangle, Style, Color, Layout
//! #     };
//! #
//! #     pub struct Renderer {}
//! #
//! #     impl iced_native::Renderer for Renderer {
//! #         type Output = ();
//! #     }
//! #
//! #     impl button::Renderer for Renderer {
//! #         fn node<Message>(
//! #             &self,
//! #             _button: &Button<'_, Message, Self>
//! #         ) -> Node {
//! #             Node::new(Style::default())
//! #         }
//! #
//! #         fn draw<Message>(
//! #             &mut self,
//! #             _button: &Button<'_, Message, Self>,
//! #             _layout: Layout<'_>,
//! #             _cursor_position: Point,
//! #         ) {}
//! #     }
//! #
//! #     impl text::Renderer for Renderer {
//! #         fn node(&self, _text: &Text) -> Node {
//! #             Node::new(Style::default())
//! #         }
//! #
//! #         fn draw(
//! #             &mut self,
//! #             _text: &Text,
//! #             _layout: Layout<'_>,
//! #         ) {
//! #         }
//! #     }
//! # }
//! use iced_native::{Button, Column, Text};
//! use iced_wgpu::Renderer; // Iced does not include a renderer! We need to bring our own!
//!
//! impl Counter {
//!     pub fn view(&mut self) -> Column<Message, Renderer> {
//!         // We use a column: a simple vertical layout
//!         Column::new()
//!             .push(
//!                 // The increment button. We tell it to produce an
//!                 // `IncrementPressed` message when pressed
//!                 Button::new(&mut self.increment_button, Text::new("+"))
//!                     .on_press(Message::IncrementPressed),
//!             )
//!             .push(
//!                 // We show the value of the counter here
//!                 Text::new(&self.value.to_string()).size(50),
//!             )
//!             .push(
//!                 // The decrement button. We tell it to produce a
//!                 // `DecrementPressed` message when pressed
//!                 Button::new(&mut self.decrement_button, Text::new("-"))
//!                     .on_press(Message::DecrementPressed),
//!             )
//!     }
//! }
//! ```
//!
//! Finally, we need to be able to react to any produced __messages__ and change
//! our __state__ accordingly in our __update logic__:
//!
//! ```
//! # use iced_native::button;
//! #
//! # struct Counter {
//! #     // The counter value
//! #     value: i32,
//! #
//! #     // The local state of the two buttons
//! #     increment_button: button::State,
//! #     decrement_button: button::State,
//! # }
//! #
//! # #[derive(Debug, Clone, Copy)]
//! # pub enum Message {
//! #     IncrementPressed,
//! #     DecrementPressed,
//! # }
//! impl Counter {
//!     // ...
//!
//!     pub fn update(&mut self, message: Message) {
//!         match message {
//!             Message::IncrementPressed => {
//!                 self.value += 1;
//!             }
//!             Message::DecrementPressed => {
//!                 self.value -= 1;
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! And that's everything! We just wrote a whole user interface. Iced is now able
//! to:
//!
//!   1. Take the result of our __view logic__ and layout its widgets.
//!   1. Process events from our system and produce __messages__ for our
//!      __update logic__.
//!   1. Draw the resulting user interface using our chosen __renderer__.
//!
//! Check out the [`UserInterface`] type to learn how to wire everything up!
//!
//! [Elm]: https://elm-lang.org/
//! [The Elm Architecture]: https://guide.elm-lang.org/architecture/
//! [documentation]: https://docs.rs/iced
//! [examples]: https://github.com/hecrj/iced/tree/master/examples
//! [`UserInterface`]: struct.UserInterface.html
//#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
pub mod input;
pub mod renderer;
pub mod widget;

mod element;
mod event;
mod hasher;
mod layout;
mod mouse_cursor;
mod node;
mod style;
mod user_interface;

pub use iced_core::{
    Align, Background, Color, Justify, Length, Point, Rectangle, Vector,
};

#[doc(no_inline)]
pub use stretch::{geometry::Size, number::Number};

pub use element::Element;
pub use event::Event;
pub use hasher::Hasher;
pub use layout::Layout;
pub use mouse_cursor::MouseCursor;
pub use node::Node;
pub use renderer::Renderer;
pub use style::Style;
pub use user_interface::{Cache, UserInterface};
pub use widget::*;
