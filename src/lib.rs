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
//! [text inputs]: https://iced.rs/examples/text_input.mp4
//! [scrollables]: https://iced.rs/examples/scrollable.mp4
//! [Debug overlay with performance metrics]: https://iced.rs/examples/debug.mp4
//! [Modular ecosystem]: https://github.com/iced-rs/iced/blob/master/ECOSYSTEM.md
//! [renderer-agnostic native runtime]: https://github.com/iced-rs/iced/tree/0.12/runtime
//! [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
//! [built-in renderer]: https://github.com/iced-rs/iced/tree/0.12/wgpu
//! [windowing shell]: https://github.com/iced-rs/iced/tree/0.12/winit
//! [`dodrio`]: https://github.com/fitzgen/dodrio
//! [web runtime]: https://github.com/iced-rs/iced_web
//! [examples]: https://github.com/iced-rs/iced/tree/0.12/examples
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
//!     Increment,
//!     Decrement,
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
//! #     Increment,
//! #     Decrement,
//! # }
//! #
//! use iced::widget::{button, column, text, Column};
//!
//! impl Counter {
//!     pub fn view(&self) -> Column<Message> {
//!         // We use a column: a simple vertical layout
//!         column![
//!             // The increment button. We tell it to produce an
//!             // `Increment` message when pressed
//!             button("+").on_press(Message::Increment),
//!
//!             // We show the value of the counter here
//!             text(self.value).size(50),
//!
//!             // The decrement button. We tell it to produce a
//!             // `Decrement` message when pressed
//!             button("-").on_press(Message::Decrement),
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
//! #     Increment,
//! #     Decrement,
//! # }
//! impl Counter {
//!     // ...
//!
//!     pub fn update(&mut self, message: Message) {
//!         match message {
//!             Message::Increment => {
//!                 self.value += 1;
//!             }
//!             Message::Decrement => {
//!                 self.value -= 1;
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! And that's everything! We just wrote a whole user interface. Let's run it:
//!
//! ```no_run
//! # #[derive(Default)]
//! # struct Counter;
//! # impl Counter {
//! #     fn update(&mut self, _message: ()) {}
//! #     fn view(&self) -> iced::Element<()> { unimplemented!() }
//! # }
//! #
//! fn main() -> iced::Result {
//!     iced::run("A cool counter", Counter::update, Counter::view)
//! }
//! ```
//!
//! Iced will automatically:
//!
//!   1. Take the result of our __view logic__ and layout its widgets.
//!   1. Process events from our system and produce __messages__ for our
//!      __update logic__.
//!   1. Draw the resulting user interface.
//!
//! # Usage
//! Use [`run`] or the [`program`] builder.
//!
//! [Elm]: https://elm-lang.org/
//! [The Elm Architecture]: https://guide.elm-lang.org/architecture/
//! [`program`]: program()
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg))]
use iced_widget::graphics;
use iced_widget::renderer;
use iced_winit as shell;
use iced_winit::core;
use iced_winit::runtime;

pub use iced_futures::futures;

#[cfg(feature = "highlighter")]
pub use iced_highlighter as highlighter;

mod application;
mod error;

pub mod program;
pub mod settings;
pub mod time;
pub mod window;

#[cfg(feature = "advanced")]
pub mod advanced;

#[cfg(feature = "multi-window")]
pub mod multi_window;

pub use crate::core::alignment;
pub use crate::core::border;
pub use crate::core::color;
pub use crate::core::gradient;
pub use crate::core::theme;
pub use crate::core::{
    Alignment, Background, Border, Color, ContentFit, Degrees, Gradient,
    Length, Padding, Pixels, Point, Radians, Rectangle, Shadow, Size, Theme,
    Transformation, Vector,
};

pub mod clipboard {
    //! Access the clipboard.
    pub use crate::runtime::clipboard::{
        read, read_primary, write, write_primary,
    };
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

pub mod event {
    //! Handle events of a user interface.
    pub use crate::core::event::{Event, MacOS, PlatformSpecific, Status};
    pub use iced_futures::event::{listen, listen_raw, listen_with};
}

pub mod keyboard {
    //! Listen and react to keyboard events.
    pub use crate::core::keyboard::key;
    pub use crate::core::keyboard::{Event, Key, Location, Modifiers};
    pub use iced_futures::keyboard::{on_key_press, on_key_release};
}

pub mod mouse {
    //! Listen and react to mouse events.
    pub use crate::core::mouse::{
        Button, Cursor, Event, Interaction, ScrollDelta,
    };
}

pub mod command {
    //! Run asynchronous actions.
    pub use crate::runtime::command::{channel, Command};
}

pub mod subscription {
    //! Listen to external events in your application.
    pub use iced_futures::subscription::{
        channel, run, run_with_id, unfold, Subscription,
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

    /// A generic overlay.
    ///
    /// This is an alias of an [`overlay::Element`] with a default `Renderer`.
    ///
    /// [`overlay::Element`]: crate::core::overlay::Element
    pub type Element<
        'a,
        Message,
        Theme = crate::Renderer,
        Renderer = crate::Renderer,
    > = crate::core::overlay::Element<'a, Message, Theme, Renderer>;

    pub use iced_widget::overlay::*;
}

pub mod touch {
    //! Listen and react to touch events.
    pub use crate::core::touch::{Event, Finger};
}

#[allow(hidden_glob_reexports)]
pub mod widget {
    //! Use the built-in widgets or create your own.
    pub use iced_widget::*;

    // We hide the re-exported modules by `iced_widget`
    mod core {}
    mod graphics {}
    mod native {}
    mod renderer {}
    mod style {}
    mod runtime {}
}

pub use command::Command;
pub use error::Error;
pub use event::Event;
pub use executor::Executor;
pub use font::Font;
pub use program::Program;
pub use renderer::Renderer;
pub use settings::Settings;
pub use subscription::Subscription;

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> = crate::core::Element<'a, Message, Theme, Renderer>;

/// The result of running a [`Program`].
pub type Result = std::result::Result<(), Error>;

/// Runs a basic iced application with default [`Settings`] given its title,
/// update, and view logic.
///
/// This is equivalent to chaining [`program`] with [`Program::run`].
///
/// [`program`]: program()
///
/// # Example
/// ```no_run
/// use iced::widget::{button, column, text, Column};
///
/// pub fn main() -> iced::Result {
///     iced::run("A counter", update, view)
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     Increment,
/// }
///
/// fn update(value: &mut u64, message: Message) {
///     match message {
///         Message::Increment => *value += 1,
///     }
/// }
///
/// fn view(value: &u64) -> Column<Message> {
///     column![
///         text(value),
///         button("+").on_press(Message::Increment),
///     ]
/// }
/// ```
pub fn run<State, Message, Theme, Renderer>(
    title: impl program::Title<State> + 'static,
    update: impl program::Update<State, Message> + 'static,
    view: impl for<'a> program::View<'a, State, Message, Theme, Renderer> + 'static,
) -> Result
where
    State: Default + 'static,
    Message: std::fmt::Debug + Send + 'static,
    Theme: Default + program::DefaultStyle + 'static,
    Renderer: program::Renderer + 'static,
{
    program(title, update, view).run()
}

#[doc(inline)]
pub use program::program;
