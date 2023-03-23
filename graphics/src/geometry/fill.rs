//! Fill [crate::widget::canvas::Geometry] with a certain style.
use crate::core::Color;
use crate::Gradient;

pub use crate::geometry::Style;

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
    pub rule: Rule,
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            style: Style::Solid(Color::BLACK),
            rule: Rule::NonZero,
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
pub enum Rule {
    NonZero,
    EvenOdd,
}
