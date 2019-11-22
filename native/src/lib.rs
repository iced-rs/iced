//! A renderer-agnostic native GUI runtime.
//!
//! ![`iced_native` crate graph](https://github.com/hecrj/iced/blob/cae26cb7bc627f4a5b3bcf1cd023a0c552e8c65e/docs/graphs/native.png?raw=true)
//!
//! `iced_native` takes [`iced_core`] and builds a native runtime on top of it,
//! featuring:
//!
//! - A custom layout engine, greatly inspired by [`druid`]
//! - Event handling for all the built-in widgets
//! - A renderer-agnostic API
//!
//! To achieve this, it introduces a bunch of reusable interfaces:
//!
//! - A [`Widget`] trait, which is used to implement new widgets: from layout
//!   requirements to event and drawing logic.
//! - A bunch of `Renderer` traits, meant to keep the crate renderer-agnostic.
//! - A [`Windowed`] trait, leveraging [`raw-window-handle`], which can be
//!   implemented by graphical renderers that target _windows_. Window-based
//!   shells (like [`iced_winit`]) can use this trait to stay renderer-agnostic.
//!
//! # Usage
//! Check out the [`UserInterface`] type to learn how to wire everything up!
//!
//! [`iced_core`]: https://github.com/hecrj/iced/tree/master/core
//! [`iced_winit`]: https://github.com/hecrj/iced/tree/master/winit
//! [`druid`]: https://github.com/xi-editor/druid
//! [`raw-window-handle`]: https://github.com/rust-windowing/raw-window-handle
//! [`Widget`]: widget/trait.Widget.html
//! [`Windowed`]: renderer/trait.Windowed.html
//! [`UserInterface`]: struct.UserInterface.html
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
pub mod input;
pub mod layout;
pub mod renderer;
pub mod widget;

mod element;
mod event;
mod hasher;
mod mouse_cursor;
mod size;
mod user_interface;

pub use iced_core::{
    Align, Background, Color, Command, Font, HorizontalAlignment, Length,
    Point, Rectangle, Vector, VerticalAlignment,
};

pub use element::Element;
pub use event::Event;
pub use hasher::Hasher;
pub use layout::Layout;
pub use mouse_cursor::MouseCursor;
pub use renderer::Renderer;
pub use size::Size;
pub use user_interface::{Cache, UserInterface};
pub use widget::*;
