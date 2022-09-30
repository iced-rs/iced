//! Supported shaders;

use crate::Color;
use crate::gradient::Gradient;

#[derive(Debug, Clone)]
/// Supported shaders for primitives.
pub enum Shader {
    /// Fill a primitive with a solid color.
    Solid(Color),
    /// Fill a primitive with an interpolated color.
    Gradient(Gradient)
}

impl <'a> Into<Shader> for Gradient {
    fn into(self) -> Shader {
        match self {
            Gradient::Linear(linear) => {
                Shader::Gradient(Gradient::Linear(linear))
            }
        }
    }
}