pub mod tour;

pub use tour::{Message, Tour};

mod widget;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub mod iced_ggez;
