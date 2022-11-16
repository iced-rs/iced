//! The styling library of Iced.
//!
//! It contains a set of styles and stylesheets for most of the built-in
//! widgets.
//!
//! ![The foundations of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/foundations.png?raw=true)
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![deny(missing_docs, unused_results)]
#![forbid(unsafe_code, rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
pub use iced_core::{Background, Color};

pub mod application;
pub mod button;
pub mod checkbox;
pub mod container;
pub mod menu;
pub mod pane_grid;
pub mod pick_list;
pub mod progress_bar;
pub mod radio;
pub mod rule;
pub mod scrollable;
pub mod slider;
pub mod svg;
pub mod text;
pub mod text_input;
pub mod theme;
pub mod toggler;

pub use theme::Theme;
