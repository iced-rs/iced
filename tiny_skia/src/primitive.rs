use crate::core::Rectangle;
use crate::graphics::Damage;

pub type Primitive = crate::graphics::Primitive<Custom>;

#[derive(Debug, Clone, PartialEq)]
pub enum Custom {
    /// A path filled with some paint.
    Fill {
        /// The path to fill.
        path: tiny_skia::Path,
        /// The paint to use.
        paint: tiny_skia::Paint<'static>,
        /// The fill rule to follow.
        rule: tiny_skia::FillRule,
    },
    /// A path stroked with some paint.
    Stroke {
        /// The path to stroke.
        path: tiny_skia::Path,
        /// The paint to use.
        paint: tiny_skia::Paint<'static>,
        /// The stroke settings.
        stroke: tiny_skia::Stroke,
    },
}

impl Damage for Custom {
    fn bounds(&self) -> Rectangle {
        match self {
            Self::Fill { path, .. } | Self::Stroke { path, .. } => {
                let bounds = path.bounds();

                Rectangle {
                    x: bounds.x(),
                    y: bounds.y(),
                    width: bounds.width(),
                    height: bounds.height(),
                }
                .expand(1.0)
            }
        }
    }
}
