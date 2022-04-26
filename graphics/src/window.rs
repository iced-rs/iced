//! Draw graphics to window surfaces.
pub mod compositor;

#[cfg(feature = "opengl")]
pub mod gl_compositor;

pub use compositor::Compositor;

#[cfg(feature = "opengl")]
pub use gl_compositor::GLCompositor;
