//! The core library of [Iced].
//!
//! This library holds basic types that can be reused and re-exported in
//! different runtime implementations. For instance, both [`iced_native`] and
//! [`iced_web`] are built on top of `iced_core`.
//!
//! ![The foundations of the Iced ecosystem](https://github.com/hecrj/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/foundations.png?raw=true)
//!
//! [Iced]: https://github.com/hecrj/iced
//! [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
//! [`iced_web`]: https://github.com/hecrj/iced/tree/master/web
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]
pub mod alignment;
pub mod keyboard;
pub mod mouse;

mod background;
mod color;
mod font;
mod length;
mod padding;
mod point;
mod rectangle;
mod size;
mod vector;

pub use alignment::Alignment;
pub use background::Background;
pub use color::Color;
pub use font::Font;
pub use length::Length;
pub use padding::Padding;
pub use point::Point;
pub use rectangle::Rectangle;
pub use size::Size;
pub use vector::Vector;
