//! A [`glow`] renderer for [`iced_native`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/hecrj/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`glow`]: https://github.com/grovesNL/glow
//! [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod backend;
mod program;
mod quad;
mod text;
mod triangle;

pub mod settings;
pub mod widget;
pub mod window;

pub use backend::Backend;
pub use settings::Settings;

pub(crate) use iced_graphics::Transformation;

#[doc(no_inline)]
pub use widget::*;

pub use iced_graphics::{Error, Viewport};

pub use iced_native::alignment;
pub use iced_native::{Alignment, Background, Color, Command, Length, Vector};

/// A [`glow`] graphics renderer for [`iced`].
///
/// [`glow`]: https://github.com/grovesNL/glow
/// [`iced`]: https://github.com/hecrj/iced
pub type Renderer = iced_graphics::Renderer<Backend>;
