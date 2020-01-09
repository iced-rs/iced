//! Use the widgets supported out-of-the-box.
//!
//! # Re-exports
//! For convenience, the contents of this module are available at the root
//! module. Therefore, you can directly type:
//!
//! ```
//! use iced_wgpu::{button, Button};
//! ```
pub mod button;
pub mod checkbox;
pub mod container;
pub mod progress_bar;
pub mod radio;
pub mod scrollable;
pub mod slider;
pub mod text_input;

#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use checkbox::Checkbox;
#[doc(no_inline)]
pub use container::Container;
#[doc(no_inline)]
pub use progress_bar::ProgressBar;
#[doc(no_inline)]
pub use radio::Radio;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use slider::Slider;
#[doc(no_inline)]
pub use text_input::TextInput;
