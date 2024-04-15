//! A renderer-agnostic native GUI runtime.
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/master/docs/graphs/native.png?raw=true)
//!
//! `iced_runtime` takes [`iced_core`] and builds a native runtime on top of it.
//!
//! [`iced_core`]: https://github.com/iced-rs/iced/tree/0.12/core
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub mod clipboard;
pub mod command;
pub mod font;
pub mod keyboard;
pub mod overlay;
pub mod program;
pub mod system;
pub mod user_interface;
pub mod window;

#[cfg(feature = "multi-window")]
pub mod multi_window;

// We disable debug capabilities on release builds unless the `debug` feature
// is explicitly enabled.
#[cfg(feature = "debug")]
#[path = "debug/basic.rs"]
mod debug;
#[cfg(not(feature = "debug"))]
#[path = "debug/null.rs"]
mod debug;

pub use iced_core as core;
pub use iced_futures as futures;

pub use command::Command;
pub use debug::Debug;
pub use font::Font;
pub use program::Program;
pub use user_interface::UserInterface;
