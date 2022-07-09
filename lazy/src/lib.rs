#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    missing_debug_implementations,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![forbid(unsafe_code)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub mod component;
pub mod responsive;

#[cfg(feature = "pure")]
#[cfg_attr(docsrs, doc(cfg(feature = "pure")))]
pub mod pure;

pub use component::Component;
pub use responsive::Responsive;

mod cache;

use cache::{Cache, CacheBuilder};
