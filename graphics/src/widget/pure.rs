//! Leverage pure, virtual widgets in your application.
#[cfg(feature = "canvas")]
pub mod canvas;

#[cfg(feature = "canvas")]
pub use canvas::Canvas;
