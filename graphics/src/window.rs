//! Draw graphics to window surfaces.
mod compositor;

#[cfg(feature = "opengl")]
mod gl_compositor;

pub use compositor::{Compositor, SwapChainError};

#[cfg(feature = "opengl")]
pub use gl_compositor::GLCompositor;
