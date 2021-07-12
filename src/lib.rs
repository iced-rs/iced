//! Iced is a cross-platform GUI library focused on simplicity and type-safety.
//! Inspired by [Elm].
//!
//! # Features
//! * Simple, easy-to-use, batteries-included API
//! * Type-safe, reactive programming model
//! * [Cross-platform support] (Windows, macOS, Linux, and the Web)
//! * Responsive layout
//! * Built-in widgets (including [text inputs], [scrollables], and more!)
//! * Custom widget support (create your own!)
//! * [Debug overlay with performance metrics]
//! * First-class support for async actions (use futures!)
//! * [Modular ecosystem] split into reusable parts:
//!   * A [renderer-agnostic native runtime] enabling integration with existing
//!     systems
//!   * A [built-in renderer] supporting Vulkan, Metal, DX11, and DX12
//!   * A [windowing shell]
//!   * A [web runtime] leveraging the DOM
//!
//! Check out the [repository] and the [examples] for more details!
//!
//! [Cross-platform support]: https://github.com/hecrj/iced/blob/master/docs/images/todos_desktop.jpg?raw=true
//! [text inputs]: https://gfycat.com/alertcalmcrow-rust-gui
//! [scrollables]: https://gfycat.com/perkybaggybaboon-rust-gui
//! [Debug overlay with performance metrics]: https://gfycat.com/incredibledarlingbee
//! [Modular ecosystem]: https://github.com/hecrj/iced/blob/master/ECOSYSTEM.md
//! [renderer-agnostic native runtime]: https://github.com/hecrj/iced/tree/master/native
//! [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
//! [built-in renderer]: https://github.com/hecrj/iced/tree/master/wgpu
//! [windowing shell]: https://github.com/hecrj/iced/tree/master/winit
//! [`dodrio`]: https://github.com/fitzgen/dodrio
//! [web runtime]: https://github.com/hecrj/iced/tree/master/web
//! [examples]: https://github.com/hecrj/iced/tree/0.3/examples
//! [repository]: https://github.com/hecrj/iced
//!
//! # Overview
//! Inspired by [The Elm Architecture], Iced expects you to split user
//! interfaces into four different concepts:
//!
//!   * __State__ — the state of your application
//!   * __Messages__ — user interactions or meaningful events that you care
//!   about
//!   * __View logic__ — a way to display your __state__ as widgets that
//!   may produce __messages__ on user interaction
//!   * __Update logic__ — a way to react to __messages__ and update your
//!   __state__
//!
//! We can build something to see how this works! Let's say we want a simple
//! counter that can be incremented and decremented using two buttons.
//!
//! We start by modelling the __state__ of our application:
//!
//! ```
//! use iced::button;
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
//! # use iced::button;
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
//! use iced::{Button, Column, Text};
//!
//! impl Counter {
//!     pub fn view(&mut self) -> Column<Message> {
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
//!                 Text::new(self.value.to_string()).size(50),
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
//! # use iced::button;
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
//! And that's everything! We just wrote a whole user interface. Iced is now
//! able to:
//!
//!   1. Take the result of our __view logic__ and layout its widgets.
//!   1. Process events from our system and produce __messages__ for our
//!      __update logic__.
//!   1. Draw the resulting user interface.
//!
//! # Usage
//! The [`Application`] and [`Sandbox`] traits should get you started quickly,
//! streamlining all the process described above!
//!
//! [Elm]: https://elm-lang.org/
//! [The Elm Architecture]: https://guide.elm-lang.org/architecture/
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
mod application;
mod element;
mod error;
mod result;
mod sandbox;

pub mod executor;
pub mod keyboard;
pub mod mouse;
pub mod settings;
pub mod widget;
pub mod window;

#[cfg(all(
    any(
        feature = "tokio",
        feature = "tokio_old",
        feature = "async-std",
        feature = "smol"
    ),
    not(target_arch = "wasm32")
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "tokio",
        feature = "tokio_old",
        feature = "async-std"
        feature = "smol"
    )))
)]
pub mod time;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "glow"),
    feature = "wgpu"
))]
use iced_winit as runtime;

#[cfg(all(not(target_arch = "wasm32"), feature = "glow"))]
use iced_glutin as runtime;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "glow"),
    feature = "wgpu"
))]
use iced_wgpu as renderer;

#[cfg(all(not(target_arch = "wasm32"), feature = "glow"))]
use iced_glow as renderer;

#[cfg(target_arch = "wasm32")]
use iced_web as runtime;

#[doc(no_inline)]
pub use widget::*;

pub use application::Application;
pub use element::Element;
pub use error::Error;
pub use executor::Executor;
pub use result::Result;
pub use sandbox::Sandbox;
pub use settings::Settings;

pub use runtime::{
    futures, menu, Align, Background, Clipboard, Color, Command, Font,
    HorizontalAlignment, Length, Menu, Point, Rectangle, Size, Subscription,
    Vector, VerticalAlignment,
};
