//! Test your `iced` applications in headless mode.
//!
//! # Basic Usage
//! Let's assume we want to test [the classical counter interface].
//!
//! First, we will want to create a [`Simulator`] of our interface:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! use iced_test::simulator;
//!
//! let mut counter = Counter { value: 0 };
//! let mut ui = simulator(counter.view());
//! ```
//!
//! Now we can simulate a user interacting with our interface. Let's use [`Simulator::click`] to click
//! the counter buttons:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! # let mut ui = simulator(counter.view());
//!
//! let _ = ui.click("+");
//! let _ = ui.click("+");
//! let _ = ui.click("-");
//! ```
//!
//! [`Simulator::click`] takes a [`Selector`]. A [`Selector`] describes a way to query the widgets of an interface. In this case,
//! [`selector::text`] lets us select a widget by the text it contains.
//!
//! We can now process any messages produced by these interactions and then assert that the final value of our counter is
//! indeed `1`!
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn update(&mut self, message: ()) {}
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! # let mut ui = simulator(counter.view());
//! #
//! # let _ = ui.click("+");
//! # let _ = ui.click("+");
//! # let _ = ui.click("-");
//! #
//! for message in ui.into_messages() {
//!     counter.update(message);
//! }
//!
//! assert_eq!(counter.value, 1);
//! ```
//!
//! We can even rebuild the interface to make sure the counter _displays_ the proper value with [`Simulator::find`]:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! let mut ui = simulator(counter.view());
//!
//! assert!(ui.find("1").is_ok(), "Counter should display 1!");
//! ```
//!
//! And that's it! That's the gist of testing `iced` applications!
//!
//! [`Simulator`] contains additional operations you can use to simulate more interactions—like [`tap_key`](Simulator::tap_key) or
//! [`typewrite`](Simulator::typewrite)—and even perform [_snapshot testing_](Simulator::snapshot)!
//!
//! [the classical counter interface]: https://book.iced.rs/architecture.html#dissecting-an-interface
#![allow(missing_docs)]
use iced_renderer as renderer;
use iced_runtime as runtime;
use iced_runtime::core;

pub mod instruction;
pub mod selector;
pub mod simulator;

mod error;

pub use error::Error;
pub use instruction::Instruction;
pub use selector::Selector;
pub use simulator::{Simulator, simulator};

#[derive(Debug, Clone)]
pub struct Test {
    instructions: Vec<Instruction>,
}
