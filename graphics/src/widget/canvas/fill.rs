//! Fill [crate::widget::canvas::Geometry] with a certain style.
use crate::{Color, Gradient};

pub use crate::triangle::Style;

/// The style used to fill geometry.
#[derive(Debug, Clone)]
pub struct Fill {
    /// The color or gradient of the fill.
    ///
    /// By default, it is set to [`Style::Solid`] with [`Color::BLACK`].
    pub style: Style,

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

impl Default for Fill {
    fn default() -> Self {
        Self {
            style: Style::Solid(Color::BLACK),
            rule: FillRule::NonZero,
        }
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Fill {
        Fill {
            style: Style::Solid(color),
            ..Fill::default()
        }
    }
}

impl From<Gradient> for Fill {
    fn from(gradient: Gradient) -> Self {
        Fill {
            style: Style::Gradient(gradient),
            ..Default::default()
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
