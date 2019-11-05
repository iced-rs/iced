//! Use the essential widgets.
//!
//! # Re-exports
//! For convenience, the contents of this module are available at the root
//! module. Therefore, you can directly type:
//!
//! ```
//! use iced_core::{button, Button};
//! ```
mod checkbox;
mod column;
mod image;
mod radio;
mod row;

pub mod button;
pub mod scrollable;
pub mod slider;
pub mod text;
pub mod text_input;

#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use slider::Slider;
#[doc(no_inline)]
pub use text::Text;
#[doc(no_inline)]
pub use text_input::TextInput;

pub use checkbox::Checkbox;
pub use column::Column;
pub use image::Image;
pub use radio::Radio;
pub use row::Row;
