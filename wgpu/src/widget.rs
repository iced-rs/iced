//! Use the graphical widgets supported out-of-the-box.

#[cfg(feature = "canvas")]
#[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
pub mod canvas;

#[cfg(feature = "canvas")]
#[doc(no_inline)]
pub use canvas::Canvas;
