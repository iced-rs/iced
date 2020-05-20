mod compositor;

#[cfg(feature = "opengl")]
mod gl_compositor;

pub use compositor::Compositor;

#[cfg(feature = "opengl")]
pub use gl_compositor::GLCompositor;
