#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
pub mod component;
pub mod responsive;

#[cfg(feature = "pure")]
pub mod pure;

pub use component::Component;
pub use responsive::Responsive;

mod cache;

use cache::{Cache, CacheBuilder};
