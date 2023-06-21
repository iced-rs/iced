//! Track and compute the damage of graphical primitives.
use crate::core::alignment;
use crate::core::{Rectangle, Size};
use crate::Primitive;

use std::sync::Arc;

pub trait Damage: PartialEq {
    /// Returns the bounds of the [`Damage`].
    fn bounds(&self) -> Rectangle;
}

impl<T: Damage> Damage for Primitive<T> {
    fn bounds(&self) -> Rectangle {
        match self {
            Self::Text {
                bounds,
                horizontal_alignment,
                vertical_alignment,
                ..
            } => {
                let mut bounds = *bounds;

                bounds.x = match horizontal_alignment {
                    alignment::Horizontal::Left => bounds.x,
                    alignment::Horizontal::Center => {
                        bounds.x - bounds.width / 2.0
                    }
                    alignment::Horizontal::Right => bounds.x - bounds.width,
                };

                bounds.y = match vertical_alignment {
                    alignment::Vertical::Top => bounds.y,
                    alignment::Vertical::Center => {
                        bounds.y - bounds.height / 2.0
                    }
                    alignment::Vertical::Bottom => bounds.y - bounds.height,
                };

                bounds.expand(1.5)
            }
            Self::Quad { bounds, .. }
            | Self::Image { bounds, .. }
            | Self::Svg { bounds, .. } => bounds.expand(1.0),
            Self::Clip { bounds, .. } => bounds.expand(1.0),
            Self::SolidMesh { size, .. } | Self::GradientMesh { size, .. } => {
                Rectangle::with_size(*size)
            }
            Self::Group { primitives } => primitives
                .iter()
                .map(Self::bounds)
                .fold(Rectangle::with_size(Size::ZERO), |a, b| {
                    Rectangle::union(&a, &b)
                }),
            Self::Translate {
                translation,
                content,
            } => content.bounds() + *translation,
            Self::Cache { content } => content.bounds(),
            Self::Custom(custom) => custom.bounds(),
        }
    }
}

fn regions<T: Damage>(a: &Primitive<T>, b: &Primitive<T>) -> Vec<Rectangle> {
    match (a, b) {
        (
            Primitive::Group {
                primitives: primitives_a,
            },
            Primitive::Group {
                primitives: primitives_b,
            },
        ) => return list(primitives_a, primitives_b),
        (
            Primitive::Clip {
                bounds: bounds_a,
                content: content_a,
                ..
            },
            Primitive::Clip {
                bounds: bounds_b,
                content: content_b,
                ..
            },
        ) => {
            if bounds_a == bounds_b {
                return regions(content_a, content_b)
                    .into_iter()
                    .filter_map(|r| r.intersection(&bounds_a.expand(1.0)))
                    .collect();
            } else {
                return vec![bounds_a.expand(1.0), bounds_b.expand(1.0)];
            }
        }
        (
            Primitive::Translate {
                translation: translation_a,
                content: content_a,
            },
            Primitive::Translate {
                translation: translation_b,
                content: content_b,
            },
        ) => {
            if translation_a == translation_b {
                return regions(content_a, content_b)
                    .into_iter()
                    .map(|r| r + *translation_a)
                    .collect();
            }
        }
        (
            Primitive::Cache { content: content_a },
            Primitive::Cache { content: content_b },
        ) => {
            if Arc::ptr_eq(content_a, content_b) {
                return vec![];
            }
        }
        _ if a == b => return vec![],
        _ => {}
    }

    let bounds_a = a.bounds();
    let bounds_b = b.bounds();

    if bounds_a == bounds_b {
        vec![bounds_a]
    } else {
        vec![bounds_a, bounds_b]
    }
}

/// Computes the damage regions between the two given lists of primitives.
pub fn list<T: Damage>(
    previous: &[Primitive<T>],
    current: &[Primitive<T>],
) -> Vec<Rectangle> {
    let damage = previous
        .iter()
        .zip(current)
        .flat_map(|(a, b)| regions(a, b));

    if previous.len() == current.len() {
        damage.collect()
    } else {
        let (smaller, bigger) = if previous.len() < current.len() {
            (previous, current)
        } else {
            (current, previous)
        };

        // Extend damage by the added/removed primitives
        damage
            .chain(bigger[smaller.len()..].iter().map(Damage::bounds))
            .collect()
    }
}

/// Groups the given damage regions that are close together inside the given
/// bounds.
pub fn group(
    mut damage: Vec<Rectangle>,
    scale_factor: f32,
    bounds: Size<u32>,
) -> Vec<Rectangle> {
    use std::cmp::Ordering;

    const AREA_THRESHOLD: f32 = 20_000.0;

    let bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: bounds.width as f32,
        height: bounds.height as f32,
    };

    damage.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal))
    });

    let mut output = Vec::new();
    let mut scaled = damage
        .into_iter()
        .filter_map(|region| (region * scale_factor).intersection(&bounds))
        .filter(|region| region.width >= 1.0 && region.height >= 1.0);

    if let Some(mut current) = scaled.next() {
        for region in scaled {
            let union = current.union(&region);

            if union.area() - current.area() - region.area() <= AREA_THRESHOLD {
                current = union;
            } else {
                output.push(current);
                current = region;
            }
        }

        output.push(current);
    }

    output
}
