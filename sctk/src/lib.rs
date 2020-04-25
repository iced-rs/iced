//! A windowing shell for Iced, on top of [`smithay-client-toolkit`].
//!
//! ![`iced_sctk` crate graph](https://github.com/hecrj/iced/blob/cae26cb7bc627f4a5b3bcf1cd023a0c552e8c65e/docs/graphs/winit.png?raw=true)
//!
//! `iced_sctk` offers some convenient abstractions on top of [`iced_native`]
//! to quickstart development when using [`smithay-client-toolkit`].
//!
//! It exposes a renderer-agnostic [`Application`] trait that can be implemented
//! and then run with a simple call. The use of this trait is optional.
//!
//! Additionally, a [`conversion`] module is available for users that decide to
//! implement a custom event loop.
//!
//! [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
//! [`smithay-client-toolkit`]: https://github.com/smithay/client-toolkit
//! [`Application`]: trait.Application.html
//! [`conversion`]: conversion
#![feature(async_closure)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
// mod sink_clone (unpin macro)

// Public interface

// Re-exports directly used iced_native definitions
#[doc(no_inline)]
pub use iced_native::*;

// smithay-client-toolkit -> iced_native (~iced_winit/conversion)
pub mod conversion;

/// Extends iced_native::window
pub mod window_ext {
    /// Renderers such as Mesa require a wayland-client[system-lib] handle to implement display extensions such as KHR_display_surface using libwayland-client.
    /// This is presented by window::Backend through HasRawWindowHandle.
    /// This trait extends Backend by providing an alternative create_surface for non-Mesa renderers (i.e software)
    /// The alternative constraint interface is to be determined
    pub trait NoHasRawWindowHandleBackend: crate::window::Backend {
        /// Crates a new [`Surface`] for the given window.
        ///
        /// [`Surface`]: #associatedtype.Surface
        fn create_surface<W>(&mut self, window: &W) -> Self::Surface;
    }
}

// Settings module compatible with iced_winit/settings
pub mod settings;

// Private definitions used by modules of this crate to implement Application

// iced_futures::Runtime::Sender: Clone to send futures
mod sink_clone;

// Futures-based event loop
use {std::marker::Unpin, futures::stream::{Stream, Peekable, SelectAll}};

// Shared across the application between user message channel, display interface events, keyboard repeat timer
enum Item<Message> {
    Push(Message),
    Apply,
    KeyRepeat(crate::keyboard::Event<'static>),
}

// Application state update
struct Update<'t, Item> {
    streams: &'t mut Peekable<SelectAll<Box<dyn Stream<Item>>>>,
    events: &'t mut Vec<Event>,
}

// Track modifiers and key repetition
mod keyboard;
// Track focus and reconstruct scroll events
mod pointer;
//
mod window;
// Implements an Application trait wrapped by iced
mod application;
// Implements additional functionality wrapped by iced
mod mode;

pub use application::Application;
//pub use clipboard::Clipboard;
pub use mode::Mode;
pub use settings::Settings;
