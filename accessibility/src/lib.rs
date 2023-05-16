mod a11y_tree;
pub mod id;
mod node;
mod traits;

pub use a11y_tree::*;
pub use accesskit;
pub use id::*;
pub use node::*;
pub use traits::*;

#[cfg(feature = "accesskit_macos")]
pub use accesskit_macos;
#[cfg(feature = "accesskit_unix")]
pub use accesskit_unix;
#[cfg(feature = "accesskit_windows")]
pub use accesskit_windows;
#[cfg(feature = "accesskit_winit")]
pub use accesskit_winit;
