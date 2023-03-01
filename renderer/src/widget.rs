#[cfg(feature = "canvas")]
pub mod canvas;

#[cfg(feature = "canvas")]
pub use canvas::Canvas;

#[cfg(feature = "qr_code")]
pub mod qr_code;

#[cfg(feature = "qr_code")]
pub use qr_code::QRCode;
