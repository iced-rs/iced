//! A windowing shell for Iced, on top of `smithay-client-toolkit`.
//! `iced_sctk` offers some convenient abstractions on top of `iced_native`
//! It exposes an optional renderer-agnostic `Application` trait to be implemented and run.
#![feature(async_closure,trait_alias)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)] mod sink_clone (unpin macro)

// Public interface

// Re-exports directly used iced_native definitions
#[doc(no_inline)]
pub use iced_native::*;

/// The graphics backend either software rendering to shared memory (iced_shm) or WGPU (iced_wgpu)
#[cfg(feature="wayland-client/use_system_lib")]
pub trait Backend = iced_shm::window::ShmBackend;
#[cfg(not(feature="wayland-client/use_system_lib"))]
pub trait Backend = iced_shm::window::ShmBackend<Surface=smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface>;

#[cfg(feature="smithay-client-toolkit/frames")]
type Frame = sctk::ConceptFrame;
#[cfg(not(feature="smithay-client-toolkit/frames"))] mod frame;
#[cfg(not(feature="smithay-client-toolkit/frames"))] pub use frame::NoFrame as Frame;

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

/// iced_winit/Settings
#[derive(Debug)]
pub struct Settings<Flags> {
    /// Data needed to initialize an [`Application`].
    pub flags: Flags,
    /// Window settings
    pub window: window::Settings,
}

///
pub mod window;

// Implements an Application trait wrapped by iced
mod application;
pub use application::{Application, Mode}; // required by iced

// Async SCTK application
mod async_sctk;
use async_sctk::{Item, Streams};

// Track modifiers and key repetition
mod keyboard;
// Track focus and reconstruct scroll events
mod pointer;
