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
// application.rs:417: get_connection_fd: only poll
// application.rs:418: UnixStream::from_raw_fd:  hidden ownership transfer

#[doc(no_inline)]
pub use iced_native::*;
pub use smithay_client_toolkit;

pub mod conversion;
pub mod settings;

/// Extends iced_native::window
pub mod window_ext {
    /// window::Backend requires to pass a HasRawWindowHandle which requires wayland-client[system-lib]
    /// To fix this the RawWindowHandle::Wayland FFI should be a wayland object id
    pub trait NoHasRawWindowHandleBackend: crate::window::Backend {
        /// Crates a new [`Surface`] for the given window.
        ///
        /// [`Surface`]: #associatedtype.Surface
        fn create_surface<W>(&mut self, window: &W) -> Self::Surface;
    }
}

mod sink_clone;

use {std::marker::Unpin, futures::stream::{Stream, Peekable, SelectAll}};

enum ControlFlow {
    Wait,
    Exit,
}

// A frame of event processing
struct Frame<'t, St: Stream+Unpin> {
    control_flow: ControlFlow,
    streams: &'t mut Peekable<SelectAll<St>>,
    events: &'t mut Vec<Event>,
}

mod keyboard;
mod pointer;
mod application;

mod mode;

// We disable debug capabilities on release builds unless the `debug` feature
// is explicitly enabled.
#[cfg(feature = "debug")]
#[path = "debug/basic.rs"]
mod debug;
#[cfg(not(feature = "debug"))]
#[path = "debug/null.rs"]
mod debug;

pub use application::Application;
//pub use clipboard::Clipboard;
pub use mode::Mode;
pub use settings::Settings;

use debug::Debug;
