//! Supported shaders;

use crate::{Color, widget};
use crate::gradient::Gradient;
use crate::widget::canvas::{FillStyle, StrokeStyle};

#[derive(Debug, Clone)]
/// Supported shaders for primitives.
pub enum Shader {
    /// Fill a primitive with a solid color.
    Solid(Color),
    /// Fill a primitive with an interpolated color.
    Gradient(Gradient)
}

impl <'a> Into<Shader> for StrokeStyle<'a> {
    fn into(self) -> Shader {
        match self {
            StrokeStyle::Solid(color) => Shader::Solid(color),
            StrokeStyle::Gradient(gradient) => gradient.clone().into()
        }
    }
}

impl <'a> Into<Shader> for FillStyle<'a> {
    fn into(self) -> Shader {
        match self {
            FillStyle::Solid(color) => Shader::Solid(color),
            FillStyle::Gradient(gradient) => gradient.clone().into()
        }
    }
}

impl <'a> Into<Shader> for widget::canvas::Gradient {
    fn into(self) -> Shader {
        match self {
            widget::canvas::Gradient::Linear(linear) => {
                Shader::Gradient(Gradient::Linear(linear))
            }
        }
    }
}