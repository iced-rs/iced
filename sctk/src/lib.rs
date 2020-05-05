//! A windowing shell for Iced, on top of `smithay-client-toolkit`.
//! `iced_sctk` offers some convenient abstractions on top of `iced_native`
//! It exposes an `Application` trait to be implemented and run.
#![feature(async_closure,trait_alias)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)] mod sink_clone (unpin macro)

// smithay-client-toolkit -> iced_native (~iced_winit/conversion)
pub mod conversion;

// Async SCTK application
mod async_sctk;

// Application trait
mod application;

#[doc(no_inline)]
pub use {iced_native::{
//iced/lib
    futures, Align, Background, Color, Command, Font, HorizontalAlignment,
    Length, Point, Rectangle, Size, Subscription, Vector, VerticalAlignment,
//iced/executor
    executor, Executor,
//iced/keyboard,mouse
    keyboard, mouse,
//iced/element,widget
    Element, Space, Column, Row
},
//iced/settings, application
    application::{Settings, Application, Mode},
    async_sctk::Settings as WindowSettings
};
