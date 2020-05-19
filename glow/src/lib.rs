//#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

mod defaults;
mod primitive;
mod quad;
mod renderer;
mod text;
mod transformation;
mod triangle;
mod viewport;

pub mod settings;
pub mod widget;
pub mod window;

pub use defaults::Defaults;
pub use primitive::Primitive;
pub use renderer::Renderer;
pub use settings::Settings;
pub use viewport::Viewport;

pub(crate) use quad::Quad;
pub(crate) use transformation::Transformation;

#[doc(no_inline)]
pub use widget::*;

pub use iced_native::{
    Background, Color, Command, HorizontalAlignment, Length, Vector,
    VerticalAlignment,
};

pub type Element<'a, Message> = iced_native::Element<'a, Message, Renderer>;
