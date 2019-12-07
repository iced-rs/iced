//! The core library of [Iced].
//!
//! ![`iced_core` crate graph](https://github.com/hecrj/iced/blob/cae26cb7bc627f4a5b3bcf1cd023a0c552e8c65e/docs/graphs/core.png?raw=true)
//!
//! This library holds basic types that can be reused and re-exported in
//! different runtime implementations. For instance, both [`iced_native`] and
//! [`iced_web`] are built on top of `iced_core`.
//!
//! [Iced]: https://github.com/hecrj/iced
//! [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
//! [`iced_web`]: https://github.com/hecrj/iced/tree/master/web
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]

mod align;
mod background;
mod color;
mod font;
mod length;
mod palette;
mod point;
mod rectangle;
mod vector;

pub use align::{Align, HorizontalAlignment, VerticalAlignment};
pub use background::Background;
pub use color::Color;
pub use font::Font;
pub use length::Length;
pub use palette::Palette;
pub use point::Point;
pub use rectangle::Rectangle;
pub use vector::Vector;

#[cfg(feature = "command")]
mod command;

#[cfg(feature = "command")]
pub use command::Command;
