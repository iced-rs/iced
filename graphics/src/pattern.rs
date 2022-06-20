//! TODO
mod gradient;

pub use gradient::{ColorStop, Gradient};

#[derive(Debug, Clone)]
/// TODO
pub enum Pattern {
    /// TODO
    Gradient(Gradient),
}

impl Pattern {
    /// TODO
    pub fn is_gradient(&self) -> bool {
        matches!(self, Pattern::Gradient(_))
    }
}
