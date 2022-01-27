//! Build different kinds of 2D shapes.
pub mod arc;

mod builder;

#[doc(no_inline)]
pub use arc::Arc;
pub use builder::Builder;

use crate::canvas::LineDash;

use iced_native::{Point, Size};
use lyon::path::iterator::PathIterator;
use lyon_algorithms::walk::{walk_along_path, RepeatedPattern};

/// An immutable set of points that may or may not be connected.
///
/// A single [`Path`] can represent different kinds of 2D shapes!
#[derive(Debug, Clone)]
pub struct Path {
    raw: lyon::path::Path,
}

impl Path {
    /// Creates a new [`Path`] with the provided closure.
    ///
    /// Use the [`Builder`] to configure your [`Path`].
    pub fn new(f: impl FnOnce(&mut Builder)) -> Self {
        let mut builder = Builder::new();

        // TODO: Make it pure instead of side-effect-based (?)
        f(&mut builder);

        builder.build()
    }

    /// Creates a new [`Path`] representing a line segment given its starting
    /// and end points.
    pub fn line(from: Point, to: Point) -> Self {
        Self::new(|p| {
            p.move_to(from);
            p.line_to(to);
        })
    }

    /// Creates a new [`Path`] representing a rectangle given its top-left
    /// corner coordinate and its `Size`.
    pub fn rectangle(top_left: Point, size: Size) -> Self {
        Self::new(|p| p.rectangle(top_left, size))
    }

    /// Creates a new [`Path`] representing a circle given its center
    /// coordinate and its radius.
    pub fn circle(center: Point, radius: f32) -> Self {
        Self::new(|p| p.circle(center, radius))
    }

    #[inline]
    pub(crate) fn raw(&self) -> &lyon::path::Path {
        &self.raw
    }

    #[inline]
    pub(crate) fn transformed(
        &self,
        transform: &lyon::math::Transform,
    ) -> Path {
        Path {
            raw: self.raw.clone().transformed(transform),
        }
    }
}

pub(super) fn dashed(path: &Path, line_dash: LineDash) -> Path {
    let segments_odd = line_dash.segments.len() % 2 == 1;

    let segments = segments_odd
        .then(|| [&line_dash.segments[..], &line_dash.segments[..]].concat())
        .unwrap_or(line_dash.segments);

    let mut points = vec![];

    walk_along_path(
        path.raw().iter().flattened(0.01),
        0.0,
        &mut RepeatedPattern {
            callback: |position: lyon_algorithms::math::Point,
                       _tangent,
                       _distance| {
                points.push(Point {
                    x: position.x,
                    y: position.y,
                });
                true
            },
            index: line_dash.offset,
            intervals: &segments,
        },
    );

    Path::new(|builder| {
        for (idx, point) in points.into_iter().enumerate() {
            let is_even = idx % 2 == 0;

            if is_even {
                builder.move_to(point);
            } else {
                builder.line_to(point);
            }
        }
    })
}
