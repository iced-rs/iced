//! A renderer-agnostic native GUI runtime.
//!
//! ![The native path of the Iced ecosystem](https://github.com/hecrj/iced/raw/improvement/update-ecosystem-and-roadmap/docs/graphs/native.png)
//!
//! `iced_native` takes [`iced_core`] and builds a native runtime on top of it,
//! featuring:
//!
//! - A custom layout engine, greatly inspired by [`druid`]
//! - Event handling for all the built-in widgets
//! - A renderer-agnostic API
//!
//! To achieve this, it introduces a couple of reusable interfaces:
//!
//! - A [`Widget`] trait, which is used to implement new widgets: from layout
//!   requirements to event and drawing logic.
//! - A bunch of `Renderer` traits, meant to keep the crate renderer-agnostic.
//!
//! # Usage
//! The strategy to use this crate depends on your particular use case. If you
//! want to:
//! - Implement a custom shell or integrate it in your own system, check out the
//! [`UserInterface`] type.
//! - Build a new renderer, see the [renderer] module.
//! - Build a custom widget, start at the [`Widget`] trait.
//!
//! [`iced_core`]: https://github.com/hecrj/iced/tree/master/core
//! [`iced_winit`]: https://github.com/hecrj/iced/tree/master/winit
//! [`druid`]: https://github.com/xi-editor/druid
//! [`raw-window-handle`]: https://github.com/rust-windowing/raw-window-handle
//! [renderer]: crate::renderer
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]
pub mod clipboard;
pub mod event;
pub mod keyboard;
pub mod layout;
pub mod mouse;
pub mod overlay;
pub mod program;
pub mod renderer;
pub mod subscription;
pub mod touch;
pub mod widget;
pub mod window;

mod element;
mod hasher;
mod runtime;
mod user_interface;

// We disable debug capabilities on release builds unless the `debug` feature
// is explicitly enabled.
#[cfg(feature = "debug")]
#[path = "debug/basic.rs"]
mod debug;
#[cfg(not(feature = "debug"))]
#[path = "debug/null.rs"]
mod debug;

pub use iced_core::{
    menu, Align, Background, Color, Font, HorizontalAlignment, Length, Menu,
    Padding, Point, Rectangle, Size, Vector, VerticalAlignment,
};
pub use iced_futures::{executor, futures, Command};

#[doc(no_inline)]
pub use executor::Executor;

pub use clipboard::Clipboard;
pub use debug::Debug;
pub use element::Element;
pub use event::Event;
pub use hasher::Hasher;
pub use layout::Layout;
pub use overlay::Overlay;
pub use program::Program;
pub use renderer::Renderer;
pub use runtime::Runtime;
pub use subscription::Subscription;
pub use user_interface::{Cache, UserInterface};
pub use widget::*;
