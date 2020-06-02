use iced_native::Color;

/// The style used to fill geometry.
#[derive(Debug, Clone, Copy)]
pub struct Fill {
    /// The color used to fill geometry.
    ///
    /// By default, it is set to `BLACK`.
    pub color: Color,

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
    fn default() -> Fill {
        Fill {
            color: Color::BLACK,
            rule: FillRule::NonZero,
        }
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Fill {
        Fill {
            color,
            ..Fill::default()
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
