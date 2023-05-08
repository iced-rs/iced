//! Use the built-in widgets or create your own.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    missing_debug_implementations,
    //missing_docs,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![forbid(unsafe_code, rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
pub use iced_renderer as renderer;
pub use iced_renderer::graphics;
pub use iced_runtime as runtime;
pub use iced_runtime::core;
pub use iced_style as style;

mod column;
mod mouse_area;
mod row;

pub mod button;
pub mod checkbox;
pub mod container;
pub mod overlay;
pub mod pane_grid;
pub mod pick_list;
pub mod progress_bar;
pub mod radio;
pub mod rule;
pub mod scrollable;
pub mod slider;
pub mod space;
pub mod text;
pub mod text_input;
pub mod toggler;
pub mod tooltip;
pub mod vertical_slider;

mod helpers;

pub use helpers::*;

#[cfg(feature = "lazy")]
mod lazy;

#[cfg(feature = "lazy")]
pub use crate::lazy::{Component, Lazy, Responsive};

#[cfg(feature = "lazy")]
pub use crate::lazy::helpers::*;

#[doc(no_inline)]
pub use button::Button;
#[doc(no_inline)]
pub use checkbox::Checkbox;
#[doc(no_inline)]
pub use column::Column;
#[doc(no_inline)]
pub use container::Container;
#[doc(no_inline)]
pub use mouse_area::MouseArea;
#[doc(no_inline)]
pub use pane_grid::PaneGrid;
#[doc(no_inline)]
pub use pick_list::PickList;
#[doc(no_inline)]
pub use progress_bar::ProgressBar;
#[doc(no_inline)]
pub use radio::Radio;
#[doc(no_inline)]
pub use row::Row;
#[doc(no_inline)]
pub use rule::Rule;
#[doc(no_inline)]
pub use scrollable::Scrollable;
#[doc(no_inline)]
pub use slider::Slider;
#[doc(no_inline)]
pub use space::Space;
#[doc(no_inline)]
pub use text::Text;
#[doc(no_inline)]
pub use text_input::TextInput;
#[doc(no_inline)]
pub use toggler::Toggler;
#[doc(no_inline)]
pub use tooltip::Tooltip;
#[doc(no_inline)]
pub use vertical_slider::VerticalSlider;

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "svg")]
#[doc(no_inline)]
pub use svg::Svg;

#[cfg(feature = "image")]
pub mod image;

#[cfg(feature = "image")]
#[doc(no_inline)]
pub use image::Image;

#[cfg(feature = "canvas")]
pub mod canvas;

#[cfg(feature = "canvas")]
#[doc(no_inline)]
pub use canvas::Canvas;

#[cfg(feature = "qr_code")]
pub mod qr_code;

#[cfg(feature = "qr_code")]
#[doc(no_inline)]
pub use qr_code::QRCode;

#[cfg(feature = "wayland")]
#[doc(no_inline)]
pub mod dnd_listener;
#[cfg(feature = "wayland")]
#[doc(no_inline)]
pub mod dnd_source;

type Renderer<Theme = style::Theme> = renderer::Renderer<Theme>;
