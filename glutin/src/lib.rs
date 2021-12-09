//! A windowing shell for [`iced`], on top of [`glutin`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/hecrj/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`iced`]: https://github.com/hecrj/iced
//! [`glutin`]: https://github.com/rust-windowing/glutin
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![forbid(rust_2018_idioms)]

pub use glutin;
#[doc(no_inline)]
pub use iced_native::*;

pub mod application;

pub use iced_winit::clipboard;
pub use iced_winit::settings;
pub use iced_winit::window;
pub use iced_winit::{Error, Mode};

#[doc(no_inline)]
pub use application::Application;
#[doc(no_inline)]
pub use clipboard::Clipboard;
#[doc(no_inline)]
pub use settings::Settings;
