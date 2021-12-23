//! Draw graphics to window surfaces.
mod compositor;

mod virtual_window;

#[cfg(feature = "opengl")]
mod gl_compositor;

pub use compositor::{Compositor, SurfaceError};

pub use virtual_window::VirtualCompositor;

#[cfg(feature = "opengl")]
pub use gl_compositor::GLCompositor;
