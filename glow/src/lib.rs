//#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

mod backend;
mod quad;
mod text;
mod triangle;

pub mod settings;
pub mod widget;
pub mod window;

pub use settings::Settings;

pub(crate) use backend::Backend;
pub(crate) use iced_graphics::Transformation;
pub(crate) use quad::Quad;

pub type Renderer = iced_graphics::Renderer<Backend>;

#[doc(no_inline)]
pub use widget::*;

pub type Element<'a, Message> = iced_native::Element<'a, Message, Renderer>;

pub use iced_graphics::Viewport;
pub use iced_native::{
    Background, Color, Command, HorizontalAlignment, Length, Vector,
    VerticalAlignment,
};
