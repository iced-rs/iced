//! A windowing shell for [`iced`] for the Web, on top of [`winit`].
//!
//! [`iced`]: https://github.com/hecrj/iced
//! [`winit`]: https://github.com/rust-windowing/winit
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![forbid(rust_2018_idioms)]

#[doc(no_inline)]
pub use iced_native::*;

#[cfg(target_arch = "wasm32")]
pub mod application;

pub use iced_winit::settings;
pub use iced_winit::Mode;

#[cfg(target_arch = "wasm32")]
#[doc(no_inline)]
pub use application::Application;
#[doc(no_inline)]
pub use settings::Settings;
