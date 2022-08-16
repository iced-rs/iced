//! TODO
pub mod gradient;

pub use gradient::Gradient;

#[derive(Debug, Clone, Default)]
/// TODO
pub enum Shader {
    /// TODO
    #[default]
    Solid,
    /// TODO
    Gradient(Gradient),
}

impl Shader {
    /// TODO
    pub fn is_gradient(&self) -> bool {
        matches!(self, Shader::Gradient(_))
    }
}
