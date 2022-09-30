use iced_native::Color;
use crate::gradient::Gradient;
use crate::shader::Shader;

/// The style used to fill geometry.
#[derive(Debug, Clone)]
pub struct Fill<'a> {
    /// The color or gradient of the fill.
    ///
    /// By default, it is set to [`FillStyle::Solid`] `BLACK`.
    pub style: FillStyle<'a>,

    /// The fill rule defines how to determine what is inside and what is
    /// outside of a shape.
    ///
    /// See the [SVG specification][1] for more details.
    ///
    /// By default, it is set to `NonZero`.
    ///
    /// [1]: https://www.w3.org/TR/SVG/painting.html#FillRuleProperty
    pub rule: FillRule,
}

impl <'a> Default for Fill<'a> {
    fn default() -> Fill<'a> {
        Fill {
            style: FillStyle::Solid(Color::BLACK),
            rule: FillRule::NonZero,
        }
    }
}

impl<'a> From<Color> for Fill<'a> {
    fn from(color: Color) -> Fill<'a> {
        Fill {
            style: FillStyle::Solid(color),
            ..Fill::default()
        }
    }
}

/// The color or gradient of a [`Fill`].
#[derive(Debug, Clone)]
pub enum FillStyle<'a> {
    /// A solid color
    Solid(Color),
    /// A color gradient
    Gradient(&'a Gradient),
}

impl <'a> Into<Shader> for FillStyle<'a> {
    fn into(self) -> Shader {
        match self {
            FillStyle::Solid(color) => Shader::Solid(color),
            FillStyle::Gradient(gradient) => gradient.clone().into()
        }
    }
}

/// The fill rule defines how to determine what is inside and what is outside of
/// a shape.
///
/// See the [SVG specification][1].
///
/// [1]: https://www.w3.org/TR/SVG/painting.html#FillRuleProperty
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

impl From<FillRule> for lyon::tessellation::FillRule {
    fn from(rule: FillRule) -> lyon::tessellation::FillRule {
        match rule {
            FillRule::NonZero => lyon::tessellation::FillRule::NonZero,
            FillRule::EvenOdd => lyon::tessellation::FillRule::EvenOdd,
        }
    }
}
