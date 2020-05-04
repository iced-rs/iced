//! Display rendering results on windows.
mod backend;
mod swap_chain;

pub use backend::{ShmBackend, Backend};
pub use swap_chain::SwapChain;
