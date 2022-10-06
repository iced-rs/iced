//! Fill [crate::widget::canvas::Geometry] with a certain style.

use crate::gradient::Gradient;
use crate::layer::mesh;
use iced_native::Color;
use crate::widget::canvas::frame::Transform;

/// The style used to fill geometry.
#[derive(Debug, Clone)]
pub struct Fill<'a> {
    /// The color or gradient of the fill.
    ///
    /// By default, it is set to [`FillStyle::Solid`] `BLACK`.
    pub style: Style<'a>,

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

impl<'a> Default for Fill<'a> {
    fn default() -> Fill<'a> {
        Fill {
            style: Style::Solid(Color::BLACK),
            rule: FillRule::NonZero,
        }
    }
}

impl<'a> From<Color> for Fill<'a> {
    fn from(color: Color) -> Fill<'a> {
        Fill {
            style: Style::Solid(color),
            ..Fill::default()
        }
    }
}

/// The color or gradient of a [`Fill`].
#[derive(Debug, Clone)]
pub enum Style<'a> {
    /// A solid color
    Solid(Color),
    /// A color gradient
    Gradient(&'a Gradient),
}

impl<'a> Style<'a> {
    /// Converts a fill's [Style] to a [mesh::Style] for use in the renderer's shader.
    pub(crate) fn as_mesh_style(&self, transform: &Transform) -> mesh::Style {
        match self {
            Style::Solid(color) => {
                mesh::Style::Solid(*color)
            },
            Style::Gradient(gradient) => {
                mesh::Style::Gradient((*gradient).clone().transform(transform))
            }
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
