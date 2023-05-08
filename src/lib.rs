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
//! [Cross-platform support]: https://github.com/iced-rs/iced/blob/master/docs/images/todos_desktop.jpg?raw=true
//! [text inputs]: https://gfycat.com/alertcalmcrow-rust-gui
//! [scrollables]: https://gfycat.com/perkybaggybaboon-rust-gui
//! [Debug overlay with performance metrics]: https://gfycat.com/incredibledarlingbee
//! [Modular ecosystem]: https://github.com/iced-rs/iced/blob/master/ECOSYSTEM.md
//! [renderer-agnostic native runtime]: https://github.com/iced-rs/iced/tree/0.9/native
//! [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
//! [built-in renderer]: https://github.com/iced-rs/iced/tree/0.9/wgpu
//! [windowing shell]: https://github.com/iced-rs/iced/tree/0.9/winit
//! [`dodrio`]: https://github.com/fitzgen/dodrio
//! [web runtime]: https://github.com/iced-rs/iced_web
//! [examples]: https://github.com/iced-rs/iced/tree/0.9/examples
//! [repository]: https://github.com/iced-rs/iced
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
//! struct Counter {
//!     // The counter value
//!     value: i32,
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
//! # struct Counter {
//! #     // The counter value
//! #     value: i32,
//! # }
//! #
//! # #[derive(Debug, Clone, Copy)]
//! # pub enum Message {
//! #     IncrementPressed,
//! #     DecrementPressed,
//! # }
//! #
//! use iced::widget::{button, column, text, Column};
//!
//! impl Counter {
//!     pub fn view(&mut self) -> Column<Message> {
//!         // We use a column: a simple vertical layout
//!         column![
//!             // The increment button. We tell it to produce an
//!             // `IncrementPressed` message when pressed
//!             button("+").on_press(Message::IncrementPressed),
//!
//!             // We show the value of the counter here
//!             text(self.value).size(50),
//!
//!             // The decrement button. We tell it to produce a
//!             // `DecrementPressed` message when pressed
//!             button("-").on_press(Message::DecrementPressed),
//!         ]
//!     }
//! }
//! ```
//!
//! Finally, we need to be able to react to any produced __messages__ and change
//! our __state__ accordingly in our __update logic__:
//!
//! ```
//! # struct Counter {
//! #     // The counter value
//! #     value: i32,
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
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    missing_debug_implementations,
    missing_docs,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![forbid(rust_2018_idioms, unsafe_code)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use iced_futures::futures;
use iced_widget::graphics;
use iced_widget::renderer;
use iced_widget::style;

#[cfg(feature = "wayland")]
use iced_sctk as shell;
#[cfg(feature = "winit")]
use iced_winit as shell;
use shell::core;
use shell::runtime;

mod error;

pub mod settings;
pub mod time;
pub mod window;

#[cfg(feature = "winit")]
mod sandbox;
#[cfg(feature = "winit")]
pub mod application;

/// wayland application
#[cfg(feature = "wayland")]
pub mod wayland;
#[cfg(feature = "wayland")]
pub use wayland::sandbox;
#[cfg(feature = "wayland")]
pub use wayland::Application;

#[cfg(feature = "advanced")]
pub mod advanced;

pub use style::theme;

pub use crate::core::alignment;
pub use crate::core::event;
pub use crate::core::{
    color, id, Alignment, Background, Color, ContentFit, Length, Padding,
    Pixels, Point, Rectangle, Size, Vector,
};
pub use crate::runtime::Command;

pub mod clipboard {
    //! Access the clipboard.
    pub use crate::runtime::clipboard::{read, write};
}

pub mod executor {
    //! Choose your preferred executor to power your application.
    pub use iced_futures::Executor;

    /// A default cross-platform executor.
    ///
    /// - On native platforms, it will use:
    ///   - `iced_futures::backend::native::tokio` when the `tokio` feature is enabled.
    ///   - `iced_futures::backend::native::async-std` when the `async-std` feature is
    ///     enabled.
    ///   - `iced_futures::backend::native::smol` when the `smol` feature is enabled.
    ///   - `iced_futures::backend::native::thread_pool` otherwise.
    ///
    /// - On Wasm, it will use `iced_futures::backend::wasm::wasm_bindgen`.
    pub type Default = iced_futures::backend::default::Executor;
}

pub mod font {
    //! Load and use fonts.
    pub use crate::core::font::*;
    pub use crate::runtime::font::*;
}

pub mod keyboard {
    //! Listen and react to keyboard events.
    pub use crate::core::keyboard::{Event, KeyCode, Modifiers};
}

pub mod mouse {
    //! Listen and react to mouse events.
    pub use crate::core::mouse::{Button, Event, Interaction, ScrollDelta};
}

pub mod subscription {
    //! Listen to external events in your application.
    pub use iced_futures::subscription::{
        channel, events, events_with, run, run_with_id, unfold, Subscription,
    };
}

#[cfg(feature = "system")]
pub mod system {
    //! Retrieve system information.
    pub use crate::runtime::system::Information;
    pub use crate::shell::system::*;
}

pub mod overlay {
    //! Display interactive elements on top of other widgets.

    /// A generic [`Overlay`].
    ///
    /// This is an alias of an `iced_native` element with a default `Renderer`.
    ///
    /// [`Overlay`]: iced_native::Overlay
    pub type Element<'a, Message, Renderer = crate::Renderer> =
        crate::core::overlay::Element<'a, Message, Renderer>;

    pub use iced_widget::overlay::*;
}

pub mod touch {
    //! Listen and react to touch events.
    pub use crate::core::touch::{Event, Finger};
}

pub mod widget {
    //! Use the built-in widgets or create your own.
    pub use iced_widget::*;

    // We hide the re-exported modules by `iced_widget`
    mod core {}
    mod graphics {}
    mod native {}
    mod renderer {}
    mod style {}
}

#[cfg(feature = "winit")]
pub use application::Application;

pub use error::Error;
pub use event::Event;
pub use executor::Executor;
pub use font::Font;
pub use sandbox::Sandbox;
pub use settings::Settings;
pub use subscription::Subscription;
pub use theme::Theme;

/// The default renderer.
pub type Renderer<Theme = style::Theme> = renderer::Renderer<Theme>;

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message, Renderer = crate::Renderer> =
    crate::core::Element<'a, Message, Renderer>;

/// The result of running an [`Application`].
///
/// [`Application`]: crate::Application
pub type Result = std::result::Result<(), Error>;
