//! A windowing shell for Iced, on top of `smithay-client-toolkit`.
//! `iced_sctk` offers some convenient abstractions on top of `iced_native`
//! It exposes an optional renderer-agnostic `Application` trait to be implemented and run.
#![feature(async_closure)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)] mod sink_clone (unpin macro)

// Public interface

// Re-exports directly used iced_native definitions
#[doc(no_inline)]
pub use iced_native::*;

// smithay-client-toolkit -> iced_native (~iced_winit/conversion)
pub mod conversion;

/*/// Extends iced_native::window
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
}*/

// impl Application

// iced_futures::Runtime::Sender: Clone
mod sink_clone;

// Futures-based event loop
use {/*std::pin::Pin,*/ futures::{stream::{LocalBoxStream, Peekable, SelectAll}}};

// Shared across the application between user message channel, display interface events, keyboard repeat timer
enum Item<Message> {
    Push(Message),
    Apply(std::io::Result<()>),
    KeyRepeat(crate::keyboard::Event<'static>),
    Close,
}

// Application state update
struct Update<'t, Item> {
    streams: &'t mut Peekable<SelectAll<LocalBoxStream<'t, Item>>>,
    events: &'t mut Vec<Event>,
}

// Track modifiers and key repetition
mod keyboard;
pub use keyboard::Keyboard;
// Track focus and reconstruct scroll events
mod pointer;
//
mod window;
pub use window::{Window, Mode};

// Async SCTK application
mod async_sctk;
// Implements an Application trait wrapped by iced
mod application;

// iced_winit/settings
pub struct Settings<Flags> {
    pub flags: Flags,
    pub window: window::Settings,
}
pub use application::Application;
