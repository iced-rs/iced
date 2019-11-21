mod application;
#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "native.rs")]
mod platform;
mod sandbox;

pub use application::Application;
pub use platform::*;
pub use sandbox::Sandbox;
