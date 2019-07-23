//#![deny(missing_docs)]
//#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
pub mod input;
pub mod widget;

mod element;
mod event;
mod hasher;
mod layout;
mod mouse_cursor;
mod node;
mod point;
mod rectangle;
mod renderer;
mod runtime;
mod style;
mod vector;

#[doc(no_inline)]
pub use stretch::{geometry::Size, number::Number};

pub use element::Element;
pub use event::Event;
pub use hasher::Hasher;
pub use layout::Layout;
pub use mouse_cursor::MouseCursor;
pub use node::Node;
pub use point::Point;
pub use rectangle::Rectangle;
pub use renderer::Renderer;
pub use runtime::{Interface, Runtime};
pub use style::{Align, Justify, Style};
pub use vector::Vector;
pub use widget::*;
